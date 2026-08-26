//! The bound 0100 states in place of a time nobody has measured.
//!
//! With no sink supplied, emitting an event tests one reference and does nothing
//! else: no clock is read and nothing is allocated. The first of the two is
//! asserted inside the crate, against a source that counts its own readings.
//! This file asserts the second, which needs something the crate cannot hold: an
//! allocator that counts.
//!
//! # Why this is a file of its own
//!
//! A global allocator is a property of a whole test binary. In its own file it is
//! one binary running one measured case, so nothing else in the suite is measured
//! by it and nothing else is slowed by it. The spans in 0061 have a file of their
//! own for the same reason, and two binaries each installing an allocator is what
//! that costs.
//!
//! # The unsafe code, and why it is here
//!
//! `src/lib.rs` carries `#![forbid(unsafe_code)]` and this file is not inside it,
//! which is a fact about where the boundary is rather than a way around it.
//! Implementing an allocator is a contract with the language that cannot be
//! written in safe code, and the alternative is not a safe version of this test:
//! it is no test, and 0100's bound stated as a claim.
//!
//! # What the count is, and what it is not
//!
//! It is per thread, in a cell that allocates nothing itself. It counts
//! allocations rather than bytes, and it says nothing about how long anything
//! took, which is #65's harness and #66's gate.

use flowfin_core::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
use flowfin_core::diagnostics::{
    Diagnostics, DiagnosticsSink, Event, EventName, Field, FieldValue, Severity,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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
        WallMoment::from_epoch(1_700_000_000, 0)
    }
}

/// A sink that keeps nothing, so that the neighbour below measures the facility
/// rather than whatever a collecting sink does with what it is handed.
struct CountingSink {
    seen: AtomicUsize,
}

impl DiagnosticsSink for CountingSink {
    fn event(&self, _event: &Event<'_>) {
        self.seen.fetch_add(1, Ordering::Relaxed);
    }
}

const REQUEST_ABANDONED: EventName = EventName::declared("server.request-abandoned");
const ENTRY_SERVED_STALE: EventName = EventName::declared("cache.entry-served-stale");

/// Two hundred tiles' worth of events, each carrying the four shapes of field
/// 0100 names, which is the case with the most reporting and the least room.
fn a_wall_of_events(diagnostics: &Diagnostics<'_>) {
    for tile in 0..200_u64 {
        diagnostics.emit(
            Severity::Detail,
            ENTRY_SERVED_STALE,
            &[
                Field::new("age", FieldValue::Interval(Duration::from_secs(tile))),
                Field::new("entries", FieldValue::Count(tile)),
            ],
        );
        diagnostics.emit(
            Severity::Failure,
            REQUEST_ABANDONED,
            &[
                Field::new("kind", FieldValue::Text("timed-out")),
                Field::new("reached-the-server", FieldValue::Truth(false)),
            ],
        );
    }
}

#[test]
fn with_no_sink_a_wall_of_events_allocates_nothing() {
    let clocks = FixedClocks;
    let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);

    // The counter is read once before the measured region so that anything the
    // harness or this file allocated on the way in is outside it.
    let warm_up = allocations_during(|| a_wall_of_events(&diagnostics));
    assert_eq!(warm_up, 0, "the first pass allocated");

    let measured = allocations_during(|| a_wall_of_events(&diagnostics));
    assert_eq!(
        measured, 0,
        "400 events emitted with no sink allocated {measured} time(s)"
    );
}

/// The same wall, with a sink supplied and the level below every event in it.
/// This is where a client running with `detail` off sits, and 0100 says it pays
/// what a client with no sink pays.
#[test]
fn under_the_level_the_same_wall_allocates_nothing() {
    let clocks = FixedClocks;
    let sink = CountingSink {
        seen: AtomicUsize::new(0),
    };
    let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Fault);

    let warm_up = allocations_during(|| a_wall_of_events(&diagnostics));
    assert_eq!(warm_up, 0, "the first pass allocated");

    let measured = allocations_during(|| a_wall_of_events(&diagnostics));
    assert_eq!(measured, 0, "400 events under the level allocated");
    assert_eq!(
        sink.seen.load(Ordering::Relaxed),
        0,
        "nothing under the level should have been delivered"
    );
}

/// The one-change neighbour. Without it the assertions above would hold for a
/// facility that never allocates whatever anybody supplied, which would prove
/// that events nobody listens to cost nothing rather than that the decision is
/// taken before the event is built.
///
/// It asserts that the events were delivered rather than that a particular
/// number of allocations happened. What a sink costs is that sink's, and a
/// number here would be a bound on somebody else's code.
#[test]
fn with_a_sink_the_same_wall_is_delivered() {
    let clocks = FixedClocks;
    let sink = CountingSink {
        seen: AtomicUsize::new(0),
    };
    let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail);

    a_wall_of_events(&diagnostics);

    assert_eq!(
        sink.seen.load(Ordering::Relaxed),
        400,
        "every event emitted should have arrived once"
    );
}
