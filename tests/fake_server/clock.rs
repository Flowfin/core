//! The clock the suite controls, in one place rather than one per test file.
//!
//! 0102 fixes what a controlled source may do and this is the source it
//! describes. #21 asks for it in one line - "a clock the test controls, so that a
//! timeout test takes microseconds rather than the timeout" - and three landed
//! records send a reader here for more than that line carries. What follows is
//! those obligations written as code.
//!
//! # What a test may do, and what it may not
//!
//! `steady` and `elapsed` move forward, by any amount, independently of each
//! other. A suspension is expressed by moving `elapsed` while `steady` stands
//! still, which is the only reason the two are separate clocks at all.
//!
//! `wall` is set freely, in both directions and by any amount. The television
//! coming up from a power cut believing it is 1970, and the person setting the
//! date forward, are cases 0102 requires a suite to be able to reach.
//!
//! Neither monotonic clock may be moved backwards. This source refuses it rather
//! than allowing it, and the two cases in the test target named for winding a
//! monotonic clock back are the proof of that refusal. The shipping code treats both
//! as never going backwards and carries no branch for it, so a source able to
//! move them backwards would be proving behaviour that nothing ships, and the
//! proof would look exactly like a real one.
//!
//! # Why it counts its own readings
//!
//! 0061 states the overhead bound of a span as a property rather than as a time,
//! and one half of that property is that nothing reads a clock when nobody is
//! listening. A source that cannot say how often it was asked turns that half
//! into a claim.

use flowfin_core::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The three readings, behind one lock.
///
/// One lock rather than three atomics because a test that moves two of them is
/// expressing one state of the world, and a reader that caught the pair half
/// moved would see a suspension nobody wrote.
struct Readings {
    steady_nanos: u64,
    elapsed_nanos: u64,
    wall: WallMoment,
}

/// A source of all three clocks that a test moves by hand.
///
/// Thread safety, from 0009: safe from any thread, which the trait requires and
/// which the fake server needs, since the core reads a deadline on one lane while
/// a test advances the clock on another.
pub struct ControlledClocks {
    readings: Mutex<Readings>,
    steady_reads: AtomicUsize,
    elapsed_reads: AtomicUsize,
    wall_reads: AtomicUsize,
}

impl ControlledClocks {
    /// A source standing at an origin a test can move away from.
    ///
    /// The monotonic clocks start above zero on purpose: a source starting at
    /// zero cannot tell a reading that was never taken from one taken at the
    /// origin, and the first thing anybody writes against a new clock is an
    /// assertion about an interval.
    #[must_use]
    pub fn started() -> Self {
        Self {
            readings: Mutex::new(Readings {
                steady_nanos: 1_000,
                elapsed_nanos: 1_000,
                // A moment inside the range every supported platform agrees
                // about, so that a fixture written against it is not a statement
                // about an epoch boundary.
                wall: WallMoment::from_epoch(1_700_000_000, 0),
            }),
            steady_reads: AtomicUsize::new(0),
            elapsed_reads: AtomicUsize::new(0),
            wall_reads: AtomicUsize::new(0),
        }
    }

    /// Moves the steady clock forward by `nanos`.
    ///
    /// # Panics
    ///
    /// Never for a forward move. The refusal below is on the other direction and
    /// this operation has no way to express one.
    pub fn advance_steady(&self, nanos: u64) {
        let mut readings = self.readings.lock().expect("the controlled clock lock");
        readings.steady_nanos = readings
            .steady_nanos
            .checked_add(nanos)
            .expect("a steady reading past the end of the clock's own range");
    }

    /// Moves the elapsed clock forward by `nanos`, independently of the steady
    /// one, which is how a suspension is written.
    ///
    /// # Panics
    ///
    /// Never for a forward move, for the reason above.
    pub fn advance_elapsed(&self, nanos: u64) {
        let mut readings = self.readings.lock().expect("the controlled clock lock");
        readings.elapsed_nanos = readings
            .elapsed_nanos
            .checked_add(nanos)
            .expect("an elapsed reading past the end of the clock's own range");
    }

    /// Sets what the device believes the time is, in either direction.
    pub fn set_wall(&self, moment: WallMoment) {
        let mut readings = self.readings.lock().expect("the controlled clock lock");
        readings.wall = moment;
    }

    /// Moves the steady clock to an absolute reading.
    ///
    /// # Panics
    ///
    /// When `nanos` is below the current reading. That is the refusal 0102
    /// requires of a controlled source, and it is a panic rather than a returned
    /// error because a test asking for it has written a case the shipping code
    /// has no branch for, and continuing would let that case look proven.
    pub fn set_steady(&self, nanos: u64) {
        let mut readings = self.readings.lock().expect("the controlled clock lock");
        assert!(
            nanos >= readings.steady_nanos,
            "the steady clock may not be wound back: 0102 forbids it, the core \
             carries no branch for it, and a test that moved it would be proving \
             behaviour nothing ships"
        );
        readings.steady_nanos = nanos;
    }

    /// Moves the elapsed clock to an absolute reading.
    ///
    /// # Panics
    ///
    /// When `nanos` is below the current reading, for the reason
    /// [`ControlledClocks::set_steady`] gives.
    pub fn set_elapsed(&self, nanos: u64) {
        let mut readings = self.readings.lock().expect("the controlled clock lock");
        assert!(
            nanos >= readings.elapsed_nanos,
            "the elapsed clock may not be wound back: 0102 forbids it, the core \
             carries no branch for it, and a test that moved it would be proving \
             behaviour nothing ships"
        );
        readings.elapsed_nanos = nanos;
    }

    /// How many times each clock has been read since this source was made.
    ///
    /// The order is steady, elapsed, wall.
    #[must_use]
    pub fn readings_taken(&self) -> (usize, usize, usize) {
        (
            self.steady_reads.load(Ordering::Relaxed),
            self.elapsed_reads.load(Ordering::Relaxed),
            self.wall_reads.load(Ordering::Relaxed),
        )
    }
}

impl Clocks for ControlledClocks {
    fn steady(&self) -> SteadyInstant {
        self.steady_reads.fetch_add(1, Ordering::Relaxed);
        let readings = self.readings.lock().expect("the controlled clock lock");
        SteadyInstant::from_nanos(readings.steady_nanos)
    }

    fn elapsed(&self) -> ElapsedInstant {
        self.elapsed_reads.fetch_add(1, Ordering::Relaxed);
        let readings = self.readings.lock().expect("the controlled clock lock");
        ElapsedInstant::from_nanos(readings.elapsed_nanos)
    }

    fn wall(&self) -> WallMoment {
        self.wall_reads.fetch_add(1, Ordering::Relaxed);
        let readings = self.readings.lock().expect("the controlled clock lock");
        readings.wall
    }
}
