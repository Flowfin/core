//! The schedule a server that is gone is probed on, and where it stops.
//!
//! `docs/decisions/0045-the-recovery-schedule.md` decides all of it: two seconds
//! before the first probe, doubling to a ceiling of five minutes, the wait drawn
//! over that interval with the spread 0038 uses, one hour of continuous
//! unreachability before the core stops probing, and a client's advisory
//! attempt-now which resets both.
//!
//! # Why the schedule is here before anything that probes
//!
//! Two landed records point at this number and neither can supply it. 0007 says
//! the core attempts the server again on its own bounded schedule and stops when
//! the bound is reached, without saying what the bound is, and 0038 says the
//! delay 0004's retry column refers to for `server-unreachable` is this
//! schedule. A reader following either arrives at a value, and until this module
//! there was none.
//!
//! 0045 also names what a schedule written from a call site loses. The bound is
//! the part left out, because nothing about an unbounded probe fails a test: the
//! suite runs for seconds and the defect is a phone that was warm in the
//! morning. [`WhileUnreachable::probing_has_stopped`] is that bound, and it is
//! proven here in microseconds against the clock 0102 requires.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything 0045 decides that two readings of one clock
//! settle: how long until the next probe is due, what that interval is after
//! each failure, when the ceiling has been reached, when the hour is up, and
//! what a client's attempt-now does to both.
//!
//! WHAT IS NOT HERE IS THE PROBE. Nothing in this tree opens a connection, for
//! the reason [`super::transport`] gives about itself, so nothing calls a
//! server, nothing declares one unreachable, and nothing reports a recovery.
//! This module holds the schedule such a thing would run on. #45's three
//! conditions are about a fake server being taken away and put back, and none of
//! them is met by anything here.
//!
//! WHAT IS ALSO NOT HERE IS THE DRAW. 0045 applies the spread 0038 defines, and
//! 0038 fixes it as a wait drawn uniformly at random from zero to the computed
//! value, per attempt per caller. Nothing in this tree supplies randomness: the
//! clocks reach the core through one injected source and there is no equivalent
//! for a draw, so where one enters is a question 0038 owns and #38 is open. What
//! this module carries is the interval the draw is taken OVER, which is the half
//! 0045 decides, and [`WhileUnreachable::interval_the_wait_is_drawn_over`] is
//! named for that rather than for a wait, so a caller cannot read it as one.
//!
//! # Every number here is chosen and none is measured
//!
//! 0045 says so of its own three, in the same words 0007 uses for its thresholds
//! and 0038 for its waits: there was no code in this repository to measure. #65
//! is the harness that would replace a choice with a number, and until it exists
//! a reader should take each constant below as an argument rather than as a
//! measurement.

use core::time::Duration;

use crate::clock::ElapsedInstant;

/// The two seconds before the first probe of a server just declared unreachable.
///
/// From 0045. Not zero, because the evidence for the declaration was either an
/// immediate refusal or two abandoned requests and neither becomes false within
/// a second. Not longer, because the commonest cause on a home network is a
/// router that dropped a connection for a moment, and somebody who walks back
/// into range wants their library.
pub const A_FIRST_PROBE_IS_DUE_AFTER: Duration = Duration::from_secs(2);

/// The five minutes the doubling stops at.
///
/// From 0045. Doubling without a ceiling reaches intervals at which the core is
/// no longer probing in any useful sense, and five minutes is short enough that
/// somebody who fixes their server sees the application notice on its own
/// without opening it.
pub const PROBES_ARE_SPACED_AT_MOST: Duration = Duration::from_mins(5);

/// The hour of continuous unreachability after which the core stops probing.
///
/// From 0045. There is a bound at all because a probe is a network call on a
/// device somebody is carrying, and a core that probes for as long as the
/// process lives keeps a radio busy on a phone in a bag overnight for a server
/// switched off the previous evening.
///
/// An hour rather than a count of attempts, because a count is a number whose
/// meaning changes every time the schedule does, and what is being bounded is
/// how long the core spends on a server that is not answering.
pub const PROBING_STOPS_AFTER: Duration = Duration::from_hours(1);

/// Where one server stands while it is not answering.
///
/// It is per server. 0045 says two configured servers are two states, two
/// schedules and two recoveries, because one being absent says nothing about the
/// other, and this type holds one of them rather than a set.
///
/// Every interval it measures is on the ELAPSED clock. 0102 puts a wait that has
/// to survive a suspension there, and 0045 says in as many words that a device
/// which slept through the wait is due a probe on waking rather than starting
/// the wait again. Reading it on `steady` would give that device an hour it
/// never spent.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhileUnreachable {
    since: ElapsedInstant,
    probes_made: u32,
}

impl WhileUnreachable {
    /// A server just declared unreachable, with no probe made yet.
    ///
    /// `at` is the moment of the declaration, which is what both the schedule
    /// and the bound run from.
    #[must_use]
    pub const fn declared_at(at: ElapsedInstant) -> Self {
        Self {
            since: at,
            probes_made: 0,
        }
    }

    /// The moment the declaration was made, or the moment a client last asked
    /// for an attempt.
    #[must_use]
    pub const fn since(self) -> ElapsedInstant {
        self.since
    }

    /// How many probes have been made under this schedule.
    #[must_use]
    pub const fn probes_made(self) -> u32 {
        self.probes_made
    }

    /// The interval the wait before the next probe is drawn over.
    ///
    /// Two seconds before the first probe, doubling after each failure, and
    /// never longer than [`PROBES_ARE_SPACED_AT_MOST`].
    ///
    /// IT IS NOT THE WAIT AND THE NAME SAYS SO. 0038 fixes the wait as a value
    /// drawn uniformly at random from zero to the computed one, and the module
    /// documentation says why the draw is not here. A caller that used this
    /// value as the wait would put every client on one household network back
    /// in step, which is the failure the spread exists against and the one 0045
    /// says matters more here than in 0038.
    ///
    /// THE DOUBLING STOPS AT THE CEILING RATHER THAN BEING CLAMPED AFTER IT, and
    /// the first version of this function did the other thing. It shifted the
    /// two seconds left once per probe and took the smaller of that and the
    /// ceiling, which is correct until the shift overflows: sixty-three probes
    /// gave a shifted value of zero, the comparison then read it as under the
    /// ceiling, and the interval came back as no wait at all. It was found by
    /// the test below walking every count up to sixty-four rather than checking
    /// the two either side of the ceiling. Stopping the doubling the moment it
    /// reaches the ceiling cannot overflow at all, because the value is under
    /// the ceiling every time it is doubled.
    #[must_use]
    pub const fn interval_the_wait_is_drawn_over(self) -> Duration {
        let ceiling = PROBES_ARE_SPACED_AT_MOST.as_secs();
        let mut seconds = A_FIRST_PROBE_IS_DUE_AFTER.as_secs();
        let mut doublings = 0;
        while doublings < self.probes_made && seconds < ceiling {
            seconds *= 2;
            doublings += 1;
        }
        if seconds >= ceiling {
            PROBES_ARE_SPACED_AT_MOST
        } else {
            Duration::from_secs(seconds)
        }
    }

    /// Whether the doubling has reached its ceiling.
    ///
    /// Here so that a caller reporting what the core is doing can say the
    /// schedule has settled without comparing two durations itself and getting
    /// the boundary wrong.
    #[must_use]
    pub const fn spacing_is_at_the_ceiling(self) -> bool {
        self.interval_the_wait_is_drawn_over().as_secs() >= PROBES_ARE_SPACED_AT_MOST.as_secs()
    }

    /// The state after one more probe has been made and did not reach the
    /// server.
    ///
    /// It moves the doubling on and leaves the moment the bound runs from where
    /// it was, because the bound is an hour of continuous unreachability rather
    /// than an hour since the last attempt. A probe that reset it would make the
    /// bound unreachable: every probe would extend the hour it is supposed to
    /// end.
    #[must_use]
    pub const fn after_a_probe_that_failed(self) -> Self {
        Self {
            since: self.since,
            probes_made: self.probes_made.saturating_add(1),
        }
    }

    /// Whether the core has stopped probing this server.
    ///
    /// True from [`PROBING_STOPS_AFTER`] of continuous unreachability onwards.
    ///
    /// Stopping is not giving up on the session, the queue or the cache. 0045 is
    /// explicit: nothing is discarded, the queue in 0047 keeps every entry, and
    /// the cached entries keep their ages under 0043. The core has stopped
    /// asking and that is all it has stopped doing, which is why this answers a
    /// question about probing and nothing else.
    #[must_use]
    pub const fn probing_has_stopped(self, now: ElapsedInstant) -> bool {
        now.interval_since(self.since).as_secs() >= PROBING_STOPS_AFTER.as_secs()
    }

    /// The state after a client told the core to attempt now.
    ///
    /// It resets the schedule and the bound, which is what 0045 says that call
    /// does. The call exists because the core cannot know what a client knows:
    /// 0003 refuses it platform knowledge, and whether the device just joined a
    /// network or came off aeroplane mode is exactly the fact a client is
    /// holding.
    ///
    /// It is advisory. It does not promise the server is there, it does not fail
    /// when it is not, and nothing here answers whether a probe succeeded.
    ///
    /// IT IS ALSO WHAT RESTARTS A STOPPED SCHEDULE, and there is no second call
    /// for that. 0045 says what restarts the core after the bound is any request
    /// a client makes, so a state that has stopped and one that has not take the
    /// same route back, and a caller cannot reach a state that is stopped
    /// forever.
    #[must_use]
    pub const fn attempt_now(self, at: ElapsedInstant) -> Self {
        Self::declared_at(at)
    }
}

#[cfg(test)]
mod tests {
    //! 0045's schedule and bound, asked of the values.
    //!
    //! What these cannot ask is any of #45's three conditions. Each of those
    //! takes the fake server away and puts it back, and nothing in this tree
    //! opens a connection to take away.

    use super::{
        A_FIRST_PROBE_IS_DUE_AFTER, PROBES_ARE_SPACED_AT_MOST, PROBING_STOPS_AFTER,
        WhileUnreachable,
    };
    use crate::clock::ElapsedInstant;
    use core::time::Duration;

    const NANOS_IN_A_SECOND: u64 = 1_000_000_000;

    fn at(seconds: u64) -> ElapsedInstant {
        ElapsedInstant::from_nanos(seconds * NANOS_IN_A_SECOND)
    }

    fn after(probes: u32) -> WhileUnreachable {
        let mut state = WhileUnreachable::declared_at(at(0));
        for _ in 0..probes {
            state = state.after_a_probe_that_failed();
        }
        state
    }

    /// The first probe is due over two seconds and the interval doubles after
    /// each failure, which is 0045's schedule up to its ceiling.
    #[test]
    fn the_interval_starts_at_two_seconds_and_doubles() {
        assert_eq!(
            after(0).interval_the_wait_is_drawn_over(),
            Duration::from_secs(2)
        );
        assert_eq!(
            after(1).interval_the_wait_is_drawn_over(),
            Duration::from_secs(4)
        );
        assert_eq!(
            after(2).interval_the_wait_is_drawn_over(),
            Duration::from_secs(8)
        );
        assert_eq!(
            after(3).interval_the_wait_is_drawn_over(),
            Duration::from_secs(16)
        );
        assert_eq!(A_FIRST_PROBE_IS_DUE_AFTER.as_secs(), 2);
    }

    /// The ceiling. 0045 refuses doubling without one, because it reaches
    /// intervals at which the core is not really probing and a server fixed at
    /// midday is noticed in the evening.
    #[test]
    fn the_doubling_stops_at_five_minutes_and_never_passes_it() {
        assert_eq!(
            after(7).interval_the_wait_is_drawn_over(),
            Duration::from_secs(256)
        );
        assert_eq!(
            after(8).interval_the_wait_is_drawn_over(),
            PROBES_ARE_SPACED_AT_MOST
        );
        assert!(!after(7).spacing_is_at_the_ceiling());
        assert!(after(8).spacing_is_at_the_ceiling());

        for probes in 8..64 {
            assert_eq!(
                after(probes).interval_the_wait_is_drawn_over(),
                PROBES_ARE_SPACED_AT_MOST,
                "the interval passed the ceiling after {probes} probes"
            );
        }
    }

    /// The bound, and the boundary itself rather than a value either side of it.
    #[test]
    fn probing_stops_at_an_hour_of_continuous_unreachability() {
        let state = WhileUnreachable::declared_at(at(0));

        assert!(!state.probing_has_stopped(at(3599)));
        assert!(state.probing_has_stopped(at(3600)));
        assert!(state.probing_has_stopped(at(7200)));
        assert_eq!(PROBING_STOPS_AFTER.as_secs(), 3600);
    }

    /// The bound is an hour of continuous unreachability rather than an hour
    /// since the last attempt, so probing does not push it away. Without this
    /// the bound could never be reached at all.
    #[test]
    fn a_probe_moves_the_schedule_on_and_does_not_move_the_bound() {
        let state = after(9);

        assert_eq!(state.since(), at(0));
        assert_eq!(state.probes_made(), 9);
        assert!(state.probing_has_stopped(at(3600)));
    }

    /// A client's attempt-now resets both, which is what makes it cheaper than
    /// any schedule.
    #[test]
    fn an_attempt_now_resets_the_schedule_and_the_bound() {
        let stopped = after(9);
        assert!(stopped.probing_has_stopped(at(3600)));

        let asked = stopped.attempt_now(at(3600));

        assert_eq!(asked.probes_made(), 0);
        assert_eq!(
            asked.interval_the_wait_is_drawn_over(),
            A_FIRST_PROBE_IS_DUE_AFTER
        );
        assert!(!asked.probing_has_stopped(at(7199)));
        assert!(asked.probing_has_stopped(at(7200)));
    }

    /// A device that slept through the wait has waited, because 0102 puts this
    /// interval on the clock that keeps counting through a suspension. The
    /// reading below is a suspension: nothing moved the state and the clock
    /// advanced by an hour.
    #[test]
    fn a_suspension_spends_the_hour_rather_than_pausing_it() {
        let state = WhileUnreachable::declared_at(at(10));

        assert!(!state.probing_has_stopped(at(11)));
        assert!(state.probing_has_stopped(at(10 + 3600)));
    }
}
