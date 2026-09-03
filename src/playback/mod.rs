//! Tracking playback position.
//!
//! 0003 puts the unit, the precision, the reporting cadence, what happens to a
//! position recorded while the server was gone, and what counts as watched
//! inside the core. The records are 0056, 0057, 0058, 0060 and 0111, and the
//! issues are #56 through #60 and #111.
//!
//! Video decoding is outside the core, for the reason 0112 records. The core
//! stops at the handover in #111.
//!
//! 0056 fixes the unit a position is expressed in against the server rather than
//! against whatever duration type the runtime offers, and 0011 measures what that
//! type actually is on the chosen toolchain: unsigned, and in nanoseconds. The
//! conversion at the boundary is 0056's, and nothing here adopts the runtime type
//! as the wire unit.
//!
//! # What is here now
//!
//! [`Ticks`] is 0056's type and [`AdmittedPosition`] is the one act that applies
//! 0056's two bounds. [`resume`] carries 0058's three thresholds and its rule
//! for whose position wins, expressed in that unit, and [`watched`] carries
//! 0060's completion rule, which takes [`resume`]'s boundary rather than
//! stating a second one. [`cadence`] carries 0057's interval, the five events
//! that do not wait for it, what each does to it, and the constraint 0057 puts
//! on [`resume`]'s rewind. [`report`] is the report itself: the one act that
//! puts a position on the queue in 0047, on each of those five events and when
//! the interval says one is due, and the place #57's three conditions are asked.
//!
//! # Why the type is named for the unit
//!
//! 0056 decides one type for a position and a duration, because they are the
//! same quantity from the same origin and every rule that uses one uses the
//! other. A name meaning "position" would then be wrong every time the value is a
//! duration, and the other way round. The unit is the one thing true of both, and
//! this issue's own condition is that the type exists with its unit stated, so
//! the unit is in the name and each conversion says which unit it speaks.
//!
//! # Where the bounds are applied
//!
//! 0056 says both bounds are applied where a value enters the core, from a
//! caller or from a server, and never at each use. [`AdmittedPosition::of`] is
//! that act. THE SITES THAT CALL IT DO NOT EXIST IN THIS TREE. Nothing here
//! reaches a server and nothing here holds an item, so no value enters the core
//! today and this module is the rule waiting for its callers: the reads are #39
//! and the handover is #111. The reports are [`report`], and it is not one of
//! those sites on purpose: it takes an [`AdmittedPosition`] rather than a
//! number, so the act happens at whichever boundary hands it one, which is the
//! client-facing call #115's creation owes rather than anything here.
//!
//! # What this module does not report
//!
//! 0056 says a clamp beyond one second is reported through 0100, once per item.
//! [`AdmittedPosition`] computes the overshoot and says whether it passed that
//! tolerance; it emits nothing. Once per item needs an item to remember it
//! against, and there is no item in this tree, so a value here that emitted would
//! emit once per call instead, which is a different rule wearing the same
//! sentence.

pub mod cadence;
pub mod report;
pub mod resume;
pub mod watched;

/// A whole number of ticks of one hundred nanoseconds, which is what a playback
/// position and a stated duration are both expressed in.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// # The unit
///
/// 0056 takes the server's unit rather than a friendlier one, and the reason is
/// not that conversion is expensive. A tick count from the server is not a whole
/// number of milliseconds, so a value converted in and back out is not the value
/// that arrived, and 0058's disagreement rule then compares two numbers that were
/// never in the same unit. [`Ticks::PER_SECOND`] carries the count.
///
/// # The width, and why nothing here overflows
///
/// A signed sixty four bit integer is what the server declares for all three of
/// its fields, so no value the server can state is unrepresentable here. The
/// largest such value is about twenty nine thousand years at this unit, which no
/// duration a server states and no sum of positions inside an item comes near.
/// The arithmetic below saturates anyway, because a bound nothing reaches is
/// cheaper to hold than to argue about.
///
/// # Never negative
///
/// Below zero is not a value this type holds. A caller cannot construct one and
/// the arithmetic here saturates at zero rather than producing one, so the
/// failure #56 names, a signed value going negative on a seek to the start,
/// cannot be built at all rather than being caught at each seek.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ticks {
    /// Never negative. Every constructor below floors at zero and every operation
    /// saturates there, which is what makes that a property of the type rather
    /// than a rule at a call site.
    ticks: i64,
}

impl Ticks {
    /// The beginning of an item, and the value a subtraction saturates to.
    pub const ZERO: Self = Self { ticks: 0 };

    /// Ticks in one second, from 0056. The server states the same count.
    pub const PER_SECOND: i64 = 10_000_000;

    /// Ticks in one millisecond, from the same count.
    pub const PER_MILLISECOND: i64 = 10_000;

    /// A count of ticks, floored at the beginning.
    ///
    /// This is the floor of 0056's first edge, applied here so that a negative
    /// number arriving from a caller or from a wire becomes the beginning rather
    /// than a position no rule below is written for.
    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        Self {
            ticks: if ticks < 0 { 0 } else { ticks },
        }
    }

    /// A count of whole seconds, exactly.
    ///
    /// Exact because a second is a whole number of ticks. Saturating at the top
    /// for the reason the type's own note gives: the bound is not reachable from
    /// anything a server states, and a caller handing this the largest integer
    /// there is gets the largest position there is rather than a wrap.
    #[must_use]
    pub const fn from_seconds(seconds: i64) -> Self {
        Self::from_ticks(seconds.saturating_mul(Self::PER_SECOND))
    }

    /// A count of whole milliseconds, exactly, on the same terms.
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self::from_ticks(millis.saturating_mul(Self::PER_MILLISECOND))
    }

    /// The tick count, which is what goes on the wire.
    ///
    /// 0056 keeps this reachable through a name that says the unit, because
    /// something has to write the number. What that record refuses is a number of
    /// unstated unit where a position is meant, which is a different thing.
    #[must_use]
    pub const fn as_ticks(self) -> i64 {
        self.ticks
    }

    /// Whole seconds, truncated toward zero.
    ///
    /// Truncated and never rounded to nearest. Rounding up is what a reader
    /// expects and it is wrong at exactly one place: the last tick of an item
    /// rounds to a whole second past the stated duration, and 0058's finished
    /// test then fires on an item one tick short of its end. Truncation is wrong
    /// by less than a unit at the other end, where nothing tests a boundary.
    #[must_use]
    pub const fn as_seconds(self) -> i64 {
        self.ticks / Self::PER_SECOND
    }

    /// Whole milliseconds, truncated toward zero, for the same reason.
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.ticks / Self::PER_MILLISECOND
    }

    /// This less `other`, saturating at the beginning.
    ///
    /// This is 0058's rewind rather than a general arithmetic. Saturation is what
    /// makes the rewind correct with nothing to remember at the call site: three
    /// seconds into an item less a ten second rewind is the beginning, and no
    /// caller has to check for it.
    ///
    /// THE FLOOR HERE IS THIS TYPE'S AND NOT THE WIDTH'S, and the difference is
    /// the whole of what this line has to get right. The integer operation of the
    /// same name saturates at the bottom of a signed sixty four bit integer, so
    /// written as one it produces a large negative number for exactly the case
    /// this method exists to serve, and every value below is built through
    /// [`Ticks::from_ticks`] for that reason.
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self::from_ticks(self.ticks.saturating_sub(other.ticks))
    }

    /// This plus `other`, saturating at the top of the width.
    ///
    /// Both sides are already at or above the beginning, so the sum is too and
    /// the floor cannot be reached from here. It goes through the same
    /// constructor anyway, so the floor has one home rather than two.
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self::from_ticks(self.ticks.saturating_add(other.ticks))
    }
}

/// A position that has been admitted into the core, with what admitting it cost.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// # What this is for
///
/// 0056 applies both of its bounds where a value enters the core and never at
/// each use, because a bound applied at each use is a rule somebody has to
/// remember at every call site and the sites that forget are the ones nobody
/// reaches in a test. [`AdmittedPosition::of`] is that one act, and it is the
/// only place in this module where a stated duration is consulted.
///
/// # Why the overshoot is carried rather than discarded
///
/// The clamp against a stated duration is silent within one second and reported
/// beyond it. A caller that received only the clamped value could not tell the
/// two apart, so it would either report every clamp or none. The overshoot is
/// here so that a caller reports 0056's case rather than a case of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedPosition {
    position: Ticks,
    overshoot: Ticks,
}

impl AdmittedPosition {
    /// How far past a stated duration a position may be before it is worth saying
    /// so.
    ///
    /// One second, which is the server's own tolerance for the same drift: it
    /// treats a position within one second of the end as the end. A stated
    /// duration comes from a probe of the file and the stream can run slightly
    /// past it, so a clamp of a few ticks is an ordinary library rather than news.
    pub const TOLERATED_OVERSHOOT: Ticks = Ticks::from_seconds(1);

    /// Admit a tick count that arrived from a caller or from a server.
    ///
    /// Both of 0056's bounds are applied here. Below the beginning becomes the
    /// beginning. Past a stated duration is clamped to that duration rather than
    /// refused, because refusing would throw away a real position over a metadata
    /// inaccuracy that is ordinary, and because nothing is allocated or indexed
    /// against a position: a wrong one costs a person the wrong place in a film,
    /// and refusing the answer that carried it would cost them the library. That
    /// is a departure from the shape 0101 uses for a declared length, and 0056
    /// records it as one.
    ///
    /// `stated_duration` of `None` is an item whose duration the server does not
    /// know, which the server produces rather than this record inventing it. There
    /// is then no upper bound and only the floor applies.
    #[must_use]
    pub const fn of(ticks: i64, stated_duration: Option<Ticks>) -> Self {
        let floored = Ticks::from_ticks(ticks);
        match stated_duration {
            None => Self {
                position: floored,
                overshoot: Ticks::ZERO,
            },
            Some(duration) => Self {
                position: if floored.as_ticks() > duration.as_ticks() {
                    duration
                } else {
                    floored
                },
                overshoot: floored.saturating_sub(duration),
            },
        }
    }

    /// The position, inside both bounds.
    #[must_use]
    pub const fn position(self) -> Ticks {
        self.position
    }

    /// How far past the stated duration the value was, or the beginning where it
    /// was not past it and where no duration was stated.
    #[must_use]
    pub const fn overshoot(self) -> Ticks {
        self.overshoot
    }

    /// Whether the clamp is the one 0056 reports through 0100 rather than the one
    /// it passes over in silence.
    ///
    /// Strictly beyond the tolerance, so a position exactly one second past a
    /// stated duration is the silent case. That is the side the server's own
    /// comparison falls on, and a boundary that disagreed with it would report an
    /// item the server treats as finished.
    #[must_use]
    pub const fn is_worth_reporting(self) -> bool {
        self.overshoot.as_ticks() > Self::TOLERATED_OVERSHOOT.as_ticks()
    }
}

#[cfg(test)]
mod tests {
    use super::{AdmittedPosition, Ticks};

    #[test]
    fn a_second_is_ten_million_ticks_and_a_millisecond_is_ten_thousand() {
        assert_eq!(Ticks::PER_SECOND, 10_000_000);
        assert_eq!(Ticks::PER_MILLISECOND, 10_000);
        assert_eq!(Ticks::from_seconds(1).as_ticks(), 10_000_000);
        assert_eq!(Ticks::from_millis(1).as_ticks(), 10_000);
    }

    /// The first of 0056's three edges. A seek to the start that went one tick the
    /// wrong way is the beginning, and the type holds nothing below it.
    #[test]
    fn a_position_before_the_beginning_is_the_beginning() {
        assert_eq!(Ticks::from_ticks(-1), Ticks::ZERO);
        assert_eq!(Ticks::from_ticks(i64::MIN), Ticks::ZERO);
        assert_eq!(Ticks::from_seconds(-30), Ticks::ZERO);
        assert_eq!(Ticks::from_millis(-1), Ticks::ZERO);
        assert_eq!(AdmittedPosition::of(-1, None).position(), Ticks::ZERO);
        assert_eq!(
            AdmittedPosition::of(-1, Some(Ticks::from_seconds(60))).position(),
            Ticks::ZERO
        );
    }

    /// The second edge. Clamped rather than refused, so the answer that carried
    /// the position survives.
    #[test]
    fn a_position_past_a_stated_duration_is_the_stated_duration() {
        let duration = Ticks::from_seconds(90);
        let admitted = AdmittedPosition::of(Ticks::from_seconds(95).as_ticks(), Some(duration));
        assert_eq!(admitted.position(), duration);
        assert_eq!(admitted.overshoot(), Ticks::from_seconds(5));
    }

    /// The third edge. The duration field is nullable on the server, so this is a
    /// case the server produces rather than one the record invents.
    #[test]
    fn an_item_with_no_stated_duration_has_no_upper_bound() {
        let ten_hours = Ticks::from_seconds(36_000);
        let admitted = AdmittedPosition::of(ten_hours.as_ticks(), None);
        assert_eq!(admitted.position(), ten_hours);
        assert_eq!(admitted.overshoot(), Ticks::ZERO);
        assert!(!admitted.is_worth_reporting());
    }

    /// A stream running a few ticks past a probed duration is an ordinary library.
    /// A stream running a minute past it is something an operator has to hear
    /// about.
    #[test]
    fn a_clamp_is_silent_within_one_second_and_worth_reporting_beyond_it() {
        let duration = Ticks::from_seconds(90);

        let a_few_ticks = AdmittedPosition::of(duration.as_ticks() + 3, Some(duration));
        assert!(!a_few_ticks.is_worth_reporting());
        assert_eq!(a_few_ticks.overshoot().as_ticks(), 3);

        let exactly_one_second =
            AdmittedPosition::of(duration.as_ticks() + Ticks::PER_SECOND, Some(duration));
        assert!(!exactly_one_second.is_worth_reporting());

        let one_tick_more =
            AdmittedPosition::of(duration.as_ticks() + Ticks::PER_SECOND + 1, Some(duration));
        assert!(one_tick_more.is_worth_reporting());

        let a_minute = AdmittedPosition::of(
            duration.as_ticks() + Ticks::from_seconds(60).as_ticks(),
            Some(duration),
        );
        assert!(a_minute.is_worth_reporting());
        assert_eq!(a_minute.overshoot(), Ticks::from_seconds(60));
    }

    #[test]
    fn a_position_inside_both_bounds_is_admitted_unchanged_and_costs_nothing() {
        let duration = Ticks::from_seconds(90);
        let admitted = AdmittedPosition::of(Ticks::from_seconds(30).as_ticks(), Some(duration));
        assert_eq!(admitted.position(), Ticks::from_seconds(30));
        assert_eq!(admitted.overshoot(), Ticks::ZERO);
        assert!(!admitted.is_worth_reporting());
    }

    /// A position exactly at the stated duration is not past it, so it is not a
    /// clamp and there is nothing to report.
    #[test]
    fn a_position_exactly_at_the_stated_duration_is_not_a_clamp() {
        let duration = Ticks::from_seconds(90);
        let admitted = AdmittedPosition::of(duration.as_ticks(), Some(duration));
        assert_eq!(admitted.position(), duration);
        assert_eq!(admitted.overshoot(), Ticks::ZERO);
        assert!(!admitted.is_worth_reporting());
    }

    /// Into the type exactly, out of it truncating. Both directions are 0056's and
    /// the record names the loss each one takes.
    #[test]
    fn conversions_in_are_exact_and_conversions_out_truncate_toward_zero() {
        assert_eq!(Ticks::from_seconds(90).as_seconds(), 90);
        assert_eq!(Ticks::from_millis(1_500).as_millis(), 1_500);
        assert_eq!(Ticks::from_millis(1_500).as_seconds(), 1);

        let not_a_whole_second = Ticks::from_ticks(19_999_999);
        assert_eq!(not_a_whole_second.as_seconds(), 1);
        assert_eq!(not_a_whole_second.as_millis(), 1_999);
    }

    /// The one place rounding to nearest would be wrong. An item one tick short of
    /// its stated end must not read as a whole second past it, because 0058's
    /// finished test is written against that number.
    #[test]
    fn the_last_tick_of_an_item_does_not_round_up_past_its_stated_duration() {
        let duration = Ticks::from_seconds(90);
        let last_tick = Ticks::from_ticks(duration.as_ticks() - 1);
        assert_eq!(last_tick.as_seconds(), 89);
        assert!(last_tick.as_seconds() < duration.as_seconds());
    }

    /// 0058's rewind, which is why the subtraction saturates.
    #[test]
    fn a_rewind_past_the_beginning_is_the_beginning() {
        let three_seconds_in = Ticks::from_seconds(3);
        let rewind = Ticks::from_seconds(10);
        assert_eq!(three_seconds_in.saturating_sub(rewind), Ticks::ZERO);
        assert_eq!(
            Ticks::from_seconds(30).saturating_sub(rewind),
            Ticks::from_seconds(20)
        );
    }

    /// The width is the server's, and the arithmetic saturates rather than
    /// wrapping. A wrap here would arrive as a plausible position rather than as a
    /// failure, which is the shape the floor above exists against.
    #[test]
    fn arithmetic_saturates_at_the_top_of_the_width_rather_than_wrapping() {
        let largest = Ticks::from_ticks(i64::MAX);
        assert_eq!(largest.as_ticks(), i64::MAX);
        assert_eq!(largest.saturating_add(Ticks::from_seconds(1)), largest);
        assert_eq!(Ticks::from_seconds(i64::MAX), largest);
        assert_eq!(Ticks::from_millis(i64::MAX), largest);
    }

    /// The bound 0056 states rather than asks for a check on: the largest value
    /// this width holds is about twenty nine thousand years at this unit, so no
    /// duration a server states comes near it.
    #[test]
    fn the_largest_representable_position_is_about_twenty_nine_thousand_years() {
        let years = Ticks::from_ticks(i64::MAX).as_seconds() / (86_400 * 365);
        assert_eq!(years, 29_247);
    }

    /// One type for both, which is 0056's decision. A duration and a position
    /// subtract without a conversion between them, and that is the arithmetic the
    /// record exists to remove from call sites.
    #[test]
    fn a_position_and_a_duration_are_one_type_and_subtract_directly() {
        let duration = Ticks::from_seconds(90);
        let position = Ticks::from_seconds(75);
        assert_eq!(duration.saturating_sub(position), Ticks::from_seconds(15));
        assert_eq!(position.saturating_sub(duration), Ticks::ZERO);
    }

    /// Ordering is the type's, so 0058 and 0060 compare two of these rather than
    /// two numbers that were never in the same unit.
    #[test]
    fn two_of_these_compare_directly() {
        assert!(Ticks::from_seconds(10) < Ticks::from_seconds(11));
        assert_eq!(Ticks::from_millis(1_000), Ticks::from_seconds(1));
        assert!(Ticks::ZERO <= Ticks::from_ticks(0));
    }
}
