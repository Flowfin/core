//! The clocks every deadline is measured against.
//!
//! This is not one of the six things 0003 names. It is here for the reason
//! [`crate::diagnostics`] is: 0102 states a rule per clock, all three reach the
//! core through one injected source, and a rule with no name to attach to is a
//! rule a reader meets nowhere.
//!
//! The source is supplied from outside rather than read here, and that is a
//! consequence of the same record rather than a convenience. 0102 says nothing
//! in the core reads a platform clock directly, and the `no-platform-clock` rule
//! in `.github/invariants/rules` refuses a platform reading anywhere under
//! `src/`, so the implementation cannot live in this tree at all.
//!
//! All three clocks are on this one source although the spans in 0061 read only
//! [`Clocks::steady`]. A source offering the one clock its first caller needed
//! is a source the second caller adds a second of, which is the split 0102
//! decided against: the record's own sentence is that the core reads three
//! clocks and never a fourth, through one source. Which of the three each
//! deadline is on is 0102's table, and the issues that build those deadlines are
//! #27, #38, #45, #47 and #57.

use core::time::Duration;

/// A reading of the steady clock.
///
/// From 0102: moves forward only, at a rate nothing corrects, from an origin
/// with no meaning. It measures an interval between two events inside one run,
/// it is not comparable across runs or between devices, and on several platforms
/// it stops while the device is suspended.
///
/// The value is nanoseconds from that origin. Nothing may be read out of it
/// except the interval between two readings, which is why there is no accessor
/// for the number itself: a moment on this clock names nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SteadyInstant {
    nanos_from_the_origin: u64,
}

impl SteadyInstant {
    /// A reading, as nanoseconds from the origin this clock counts from.
    #[must_use]
    pub const fn from_nanos(nanos_from_the_origin: u64) -> Self {
        Self {
            nanos_from_the_origin,
        }
    }

    /// The interval from an earlier reading to this one.
    ///
    /// A reading that is not later than `earlier` produces a zero interval
    /// rather than a negative one. 0102 forbids this clock from moving backwards
    /// and the controlled source in the suite refuses to move it backwards, so
    /// the shipping code carries no branch for a case it is not built to meet;
    /// what it carries instead is a floor, and a zero interval is the honest
    /// reading of two endpoints that did not advance.
    #[must_use]
    pub const fn interval_since(self, earlier: Self) -> Duration {
        Duration::from_nanos(
            self.nanos_from_the_origin
                .saturating_sub(earlier.nanos_from_the_origin),
        )
    }
}

/// A reading of the elapsed clock.
///
/// From 0102: the same properties as [`SteadyInstant`], and it keeps counting
/// while the device is suspended. It measures an interval that has to survive a
/// device going to sleep, which is the queued action in #47 rather than a
/// request timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElapsedInstant {
    nanos_from_the_origin: u64,
}

impl ElapsedInstant {
    /// A reading, as nanoseconds from the origin this clock counts from.
    #[must_use]
    pub const fn from_nanos(nanos_from_the_origin: u64) -> Self {
        Self {
            nanos_from_the_origin,
        }
    }

    /// The interval from an earlier reading to this one, floored at zero for the
    /// reason [`SteadyInstant::interval_since`] gives.
    #[must_use]
    pub const fn interval_since(self, earlier: Self) -> Duration {
        Duration::from_nanos(
            self.nanos_from_the_origin
                .saturating_sub(earlier.nanos_from_the_origin),
        )
    }
}

/// What the device believes the time is.
///
/// From 0102: it moves in both directions, by a correction, by a person setting
/// it, and by a television coming up from a power cut believing it is 1970. It
/// is the only clock that names a moment two machines can talk about, and it is
/// read only where something outside the device also has an opinion about that
/// moment.
///
/// A duration is never measured on it. There is deliberately no operation here
/// that subtracts one moment from another, because that operation is the rule
/// 0102 states and the place it would be broken.
///
/// The seconds are signed so that a device believing it is before 1970 is a
/// value this type can carry rather than one it saturates. A clock that wrong is
/// exactly the case the record names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WallMoment {
    seconds_from_the_epoch: i64,
    nanos: u32,
}

impl WallMoment {
    /// A moment, as whole seconds from the epoch and nanoseconds after them.
    #[must_use]
    pub const fn from_epoch(seconds_from_the_epoch: i64, nanos: u32) -> Self {
        Self {
            seconds_from_the_epoch,
            nanos,
        }
    }

    /// The whole seconds from the epoch.
    #[must_use]
    pub const fn seconds_from_the_epoch(self) -> i64 {
        self.seconds_from_the_epoch
    }

    /// The nanoseconds after the whole second.
    #[must_use]
    pub const fn nanos(self) -> u32 {
        self.nanos
    }
}

/// The one source every clock in the core is read through.
///
/// 0102 fixes both halves of this. The core reads three clocks and never a
/// fourth, and it reads all three here rather than from a platform: a deadline
/// measured against a clock no test can move is a timeout test that takes
/// seconds and answers differently on a loaded machine.
///
/// Thread safety, from 0009: safe from any thread. The core reads it on both
/// lanes, and a source that were not would make every deadline in the core a
/// synchronisation point.
///
/// What an implementation owes, beyond answering: neither monotonic reading may
/// ever go backwards. The core carries no branch for one that does, so an
/// implementation that lets it happen produces intervals nothing here will
/// report as wrong.
pub trait Clocks: Send + Sync {
    /// The steady clock now.
    fn steady(&self) -> SteadyInstant;

    /// The elapsed clock now.
    fn elapsed(&self) -> ElapsedInstant;

    /// What the device believes the time is.
    fn wall(&self) -> WallMoment;
}

#[cfg(test)]
mod tests {
    use super::{ElapsedInstant, SteadyInstant, WallMoment};

    #[test]
    fn an_interval_is_the_distance_between_two_readings() {
        let started = SteadyInstant::from_nanos(1_000);
        let ended = SteadyInstant::from_nanos(4_500);
        assert_eq!(ended.interval_since(started).as_nanos(), 3_500);
    }

    #[test]
    fn two_readings_that_did_not_advance_are_a_zero_interval() {
        let at = SteadyInstant::from_nanos(1_000);
        assert_eq!(at.interval_since(at).as_nanos(), 0);
    }

    /// The floor rather than a wrap. A subtraction that went the other way in an
    /// unsigned type is the largest interval this core could report, and it
    /// would arrive as a plausible number rather than as a failure.
    #[test]
    fn a_reading_earlier_than_the_one_before_it_is_a_zero_interval_and_not_a_huge_one() {
        let later = SteadyInstant::from_nanos(1_000);
        let earlier = SteadyInstant::from_nanos(4_500);
        assert_eq!(later.interval_since(earlier).as_nanos(), 0);
    }

    #[test]
    fn the_elapsed_clock_measures_an_interval_the_same_way() {
        let started = ElapsedInstant::from_nanos(7);
        let ended = ElapsedInstant::from_nanos(11);
        assert_eq!(ended.interval_since(started).as_nanos(), 4);
        assert_eq!(started.interval_since(ended).as_nanos(), 0);
    }

    #[test]
    fn a_wall_moment_carries_a_time_before_the_epoch() {
        let power_cut = WallMoment::from_epoch(-86_400, 5);
        assert_eq!(power_cut.seconds_from_the_epoch(), -86_400);
        assert_eq!(power_cut.nanos(), 5);
    }
}
