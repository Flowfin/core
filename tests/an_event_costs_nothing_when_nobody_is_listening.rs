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
use flowfin_core::diagnostics::redaction::{CorrelatorSalt, FieldName};
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

/// The names the wall below carries, each with the treatment 0071 gives it.
///
/// All four are carried whole, which is what the case measuring the wall needs:
/// an age, a count of entries, an error kind and a truth are the shapes 0100
/// names and none of them can differ between two people running the same build
/// against the same server.
const AGE: FieldName = FieldName::carried_whole("age");
const ENTRIES: FieldName = FieldName::carried_whole("entries");
const KIND: FieldName = FieldName::carried_whole("kind");
const REACHED_THE_SERVER: FieldName = FieldName::carried_whole("reached-the-server");

/// The one name in this file that 0071 reduces, used only by the case that
/// measures what the reduction costs.
const ITEM: FieldName = FieldName::reduced("item");

/// A salt for the suite, fixed so that a case gives the same answer on every
/// run. A real one is created when the core is created and is unpredictable;
/// nothing here depends on that, because no case in this file reads a
/// correlator.
fn a_salt() -> CorrelatorSalt {
    CorrelatorSalt::from_bytes([0x5a; CorrelatorSalt::WIDTH])
}

/// Two hundred tiles' worth of events, each carrying the four shapes of field
/// 0100 names, which is the case with the most reporting and the least room.
fn a_wall_of_events(diagnostics: &Diagnostics<'_>) {
    for tile in 0..200_u64 {
        diagnostics.emit(
            Severity::Detail,
            ENTRY_SERVED_STALE,
            &[
                Field::new(AGE, FieldValue::Interval(Duration::from_secs(tile))),
                Field::new(ENTRIES, FieldValue::Count(tile)),
            ],
        );
        diagnostics.emit(
            Severity::Failure,
            REQUEST_ABANDONED,
            &[
                Field::new(KIND, FieldValue::Text("timed-out")),
                Field::new(REACHED_THE_SERVER, FieldValue::Truth(false)),
            ],
        );
    }
}

#[test]
fn with_no_sink_a_wall_of_events_allocates_nothing() {
    let clocks = FixedClocks;
    let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());

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
    let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Fault, a_salt());

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

/// The wall above with a sink listening to every event in it, which is where
/// 0071's own cost would show if the rule cost anything on an event it does not
/// touch.
///
/// The neighbour below is what makes this a measurement rather than a
/// restatement of the two cases above: with one field reduced instead of carried
/// whole, the same wall through the same sink allocates. The sink here keeps
/// nothing and allocates nothing itself, so what is counted is the facility.
#[test]
fn a_wall_of_carried_whole_events_allocates_nothing_with_a_sink_listening() {
    let clocks = FixedClocks;
    let sink = CountingSink {
        seen: AtomicUsize::new(0),
    };
    let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

    let warm_up = allocations_during(|| a_wall_of_events(&diagnostics));
    assert_eq!(warm_up, 0, "the first pass allocated");

    let measured = allocations_during(|| a_wall_of_events(&diagnostics));
    assert_eq!(
        measured, 0,
        "400 carried-whole events delivered to a listening sink allocated {measured} time(s)"
    );
    assert_eq!(
        sink.seen.load(Ordering::Relaxed),
        800,
        "every event emitted should have arrived once"
    );
}

/// The same wall with one field reduced instead of carried whole, which is what
/// 0071 costs where it has something to do.
///
/// It is here rather than in the crate because the answer is a number of
/// allocations, and that needs the counting allocator this binary installs. What
/// it says is a floor on the cost and never a bound on it: with no sink nothing
/// is built at all, so the reduction is free exactly where 0100 requires it to
/// be, and with a sink listening the reduction allocates. Nothing here bounds
/// how much, and no time is measured; that is #65 and #66.
#[test]
fn a_reduced_field_costs_nothing_with_no_sink_and_allocates_with_one() {
    let clocks = FixedClocks;

    let silent = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
    let warm_up = allocations_during(|| a_wall_of_reduced_events(&silent));
    assert_eq!(warm_up, 0, "the first pass allocated");
    let measured = allocations_during(|| a_wall_of_reduced_events(&silent));
    assert_eq!(
        measured, 0,
        "200 events with a reduced field and no sink allocated {measured} time(s)"
    );

    let sink = CountingSink {
        seen: AtomicUsize::new(0),
    };
    let listening = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());
    let with_a_sink = allocations_during(|| a_wall_of_reduced_events(&listening));
    assert!(
        with_a_sink > 0,
        "the reduction allocated nothing with a sink listening, so nothing was reduced"
    );
    assert_eq!(
        sink.seen.load(Ordering::Relaxed),
        200,
        "every event emitted should have arrived once"
    );
}

/// Two hundred events each carrying one reduced field beside one carried whole,
/// so that the pass 0071 adds is the difference between this wall and the one
/// above it.
fn a_wall_of_reduced_events(diagnostics: &Diagnostics<'_>) {
    for tile in 0..200_u64 {
        diagnostics.emit(
            Severity::Detail,
            ENTRY_SERVED_STALE,
            &[
                Field::new(ITEM, FieldValue::Count(tile)),
                Field::new(ENTRIES, FieldValue::Count(tile)),
            ],
        );
    }
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
    let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

    a_wall_of_events(&diagnostics);

    assert_eq!(
        sink.seen.load(Ordering::Relaxed),
        400,
        "every event emitted should have arrived once"
    );
}
