//! The bound 0061 states in place of a time nobody has measured.
//!
//! With no subscriber supplied, opening a span tests one reference and does
//! nothing else: no clock is read, nothing is allocated, and no lock is taken.
//! The first of the three is asserted inside the crate, against a source that
//! counts its own readings. This file asserts the second, which needs something
//! the crate cannot hold: an allocator that counts.
//!
//! # Why this is a file of its own
//!
//! A global allocator is a property of a whole test binary. In its own file it
//! is one binary running one test, so nothing else in the suite is measured by
//! it and nothing else is slowed by it.
//!
//! # The one piece of unsafe code in this repository, and why it is here
//!
//! `src/lib.rs` carries `#![forbid(unsafe_code)]` and this file is not inside
//! it, which is a fact about where the boundary is rather than a way around it.
//! Implementing an allocator is a contract with the language that cannot be
//! written in safe code, and the alternative is not a safe version of this test:
//! it is no test, and 0061's bound stated as a claim. The unsafe block below
//! forwards every call to the system allocator unchanged and adds a count.
//!
//! # What the count is, and what it is not
//!
//! It is per thread, in a cell that allocates nothing itself, so a background
//! thread the harness happens to run cannot move it. It counts allocations
//! rather than bytes. It says nothing about how long anything took, which is
//! #65's harness and #66's gate.

use flowfin_core::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
use flowfin_core::measurement::{ClosedSpan, Measurement, MeasurementSink, SpanName, SpanOutcome};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// Allocations on this thread since the last reading.
    ///
    /// A `Cell` with a constant initialiser, because the allocator below runs
    /// inside every allocation and a thread-local that allocated to initialise
    /// itself would recurse.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

/// The system allocator, with a count in front of it.
struct Counting;

// SAFETY: every call is forwarded to the system allocator with the same
// arguments and the same return value, so the contract this implementation has
// to keep is the one `System` already keeps. What is added is a counter that
// touches no allocation of its own.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        // SAFETY: `layout` is passed through unchanged, which is what the caller
        // already guaranteed to be valid.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: both arguments are passed through unchanged, and this
        // allocator hands back exactly what `System` handed it.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        // SAFETY: all three arguments are passed through unchanged.
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Reads how many allocations `work` made on this thread.
fn allocations_during(work: impl FnOnce()) -> usize {
    ALLOCATIONS.with(|n| n.set(0));
    work();
    ALLOCATIONS.with(Cell::get)
}

/// A source that answers without allocating, so that a reading of the counter is
/// about the facility rather than about the clock behind it.
struct FixedClocks;

impl Clocks for FixedClocks {
    fn steady(&self) -> SteadyInstant {
        SteadyInstant::from_nanos(7)
    }

    fn elapsed(&self) -> ElapsedInstant {
        ElapsedInstant::from_nanos(7)
    }

    fn wall(&self) -> WallMoment {
        WallMoment::from_epoch(0, 0)
    }
}

/// A subscriber that keeps nothing, so that the neighbour below measures the
/// facility rather than whatever a collecting subscriber does with what it is
/// handed.
struct CountingSink {
    seen: std::sync::atomic::AtomicUsize,
}

impl MeasurementSink for CountingSink {
    fn span_closed(&self, _span: &ClosedSpan) {
        self.seen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The wall of two hundred tiles from #53, six sub-intervals deep, which is the
/// case 0061 names as the one that decides whether the bound matters.
fn a_wall_of_tiles(measurement: &Measurement<'_>) {
    for _ in 0..200 {
        let tile = measurement.open(SpanName::FirstTileCoreCold, None);
        for _ in 0..6 {
            let part = measurement.open(SpanName::FirstTileCoreWarm, tile.id());
            part.close(SpanOutcome::Completed);
        }
        tile.close(SpanOutcome::Cancelled);
    }
}

#[test]
fn with_no_subscriber_a_wall_of_tiles_allocates_nothing() {
    let clocks = FixedClocks;
    let measurement = Measurement::new(&clocks, None);

    // The counter is read once before the measured region so that anything the
    // harness or this file allocated on the way in is outside it.
    let warm_up = allocations_during(|| a_wall_of_tiles(&measurement));
    assert_eq!(warm_up, 0, "the first pass allocated");

    let measured = allocations_during(|| a_wall_of_tiles(&measurement));
    assert_eq!(
        measured, 0,
        "1400 spans opened and closed with no subscriber allocated {measured} time(s)"
    );
}

/// The one-change neighbour. Without it the assertion above would hold for a
/// facility that never allocates whatever anybody supplies, which would prove
/// that a wall of tiles nothing measures costs nothing rather than that the
/// decision is taken at open.
///
/// This one asserts that the spans were delivered rather than that a particular
/// number of allocations happened. What a subscriber costs is that subscriber's,
/// and a number here would be a bound on somebody else's code.
#[test]
fn with_a_subscriber_the_same_wall_is_delivered() {
    let clocks = FixedClocks;
    let sink = CountingSink {
        seen: std::sync::atomic::AtomicUsize::new(0),
    };
    let measurement = Measurement::new(&clocks, Some(&sink));

    a_wall_of_tiles(&measurement);

    assert_eq!(
        sink.seen.load(std::sync::atomic::Ordering::Relaxed),
        200 * 7,
        "every span opened should have arrived once"
    );
}
