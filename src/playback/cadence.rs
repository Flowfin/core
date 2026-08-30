//! The interval a position is reported on, the five events that do not wait for
//! it, and what each of them does to it.
//!
//! `docs/decisions/0057-the-progress-reporting-cadence.md` is the record and #57
//! is the issue. It decides four things: ten seconds between reports while
//! something is playing, five events that report the moment they happen, an
//! interval that does not run while playback is paused, and every report going
//! onto the queue in 0047 rather than to a server.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of 0057 that two readings of one clock and a
//! mapping settle: when the next report is due, that nothing is ever due while
//! paused, which five events do not wait, and which of them starts, stops or
//! leaves the interval alone.
//!
//! WHAT IS NOT HERE IS THE REPORT. Nothing in this tree plays anything, holds a
//! session or reaches a server, so no position is produced, nothing is enqueued
//! and nothing is sent. This module holds the interval such a thing would run
//! on. #57's three open conditions are a scrub producing one report, each
//! immediate event reporting without waiting, and every report observed passing
//! through the queue, and none of them is met by anything here.
//!
//! WHAT IS ALSO NOT HERE IS A SECOND COALESCING RULE, and its absence is the
//! decision rather than an omission. #57's own condition that a scrub produces
//! one report is already answered by 0047 coalescing at the moment of enqueue,
//! and 0057 says in as many words that writing a second rule here is the thing
//! to avoid. [`crate::server::write_queue`] holds the only one.
//!
//! # The number here is chosen and not measured
//!
//! 0057 says so of its own interval. What it is chosen between is written in the
//! record as arithmetic rather than as a claim - one stream at ten seconds is
//! three hundred and sixty reports an hour, and four at once is one thousand
//! four hundred and forty writes an hour arriving at a machine an operator runs
//! at home - against the loss somebody takes when the report that would have
//! carried their position never happened. #65 is the harness a measured
//! replacement would come from.
//!
//! # One constraint here is on another record, and it is checked
//!
//! 0057 does not take the rewind's number, it puts a bound on it: the interval
//! may not exceed the rewind 0058 fixes, because a resume that rewinds by more
//! than the interval absorbs the whole loss and one that rewinds by less does
//! not. That is written in the direction that can be checked, and the case at
//! the bottom of this file is what checks it, so a change to either number on
//! its own turns the suite red instead of leaving two records quietly
//! disagreeing.

use core::time::Duration;

use crate::clock::ElapsedInstant;

/// The ten seconds between one position report and the next while something is
/// playing.
///
/// From 0057. It is the upper bound on how far back somebody is thrown when the
/// process was killed, the television lost power or the application was swiped
/// away, because none of those produces a stop event and the last interval
/// report is then the last thing the server heard.
pub const A_POSITION_IS_REPORTED_EVERY: Duration = Duration::from_secs(10);

/// One of the five events 0057 says reports the moment it happens.
///
/// They are events rather than intervals, so no clock moves them and 0102 has
/// nothing to say about them. Each is a moment where the position changed in a
/// way the interval would misrepresent if it were the only route.
///
/// STOPPING THE CORE IS NOT A SIXTH. 0115 already fixes that a stop neither
/// drains the queue nor discards it, so a report enqueued a moment before is
/// still there afterwards, and a sixth member would be a second mechanism for a
/// promise 0047 already keeps.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportsWithoutWaiting {
    /// Playback began.
    Started,
    /// Playback was paused, which is where a person will look when they come
    /// back.
    Paused,
    /// Playback was resumed.
    Resumed,
    /// The position was moved by an amount no interval can interpolate.
    Seeked,
    /// Playback ended.
    ///
    /// This is the report a person most expects to have landed, and the honest
    /// statement is that it is durable rather than delivered: 0047 and 0045
    /// decide whether it reaches the server, and 0057 promises no delivery it is
    /// not in a position to make.
    Stopped,
}

/// What one of those events does to the interval.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItDoesToTheInterval {
    /// The interval runs from this event.
    RunsFromHere,
    /// The interval stops, and nothing is due until something starts it again.
    Stops,
    /// The interval is untouched, because the position moved but playback did
    /// not.
    Untouched,
}

impl ReportsWithoutWaiting {
    /// Every event that does not wait for the interval, so that a caller reads
    /// the set out of the crate rather than keeping a copy of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Started,
            Self::Paused,
            Self::Resumed,
            Self::Seeked,
            Self::Stopped,
        ]
    }

    /// The event as it is reported.
    ///
    /// This is what a report carries rather than the text a debug printing would
    /// produce, for the reason 0100 gives: a field is data a client reads, and a
    /// name that changed when somebody renamed a variant would change what every
    /// client's report says.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Paused => "paused",
            Self::Resumed => "resumed",
            Self::Seeked => "seeked",
            Self::Stopped => "stopped",
        }
    }

    /// What this event does to the interval.
    ///
    /// A seek leaves it alone, and that is the member most likely to be got
    /// wrong. A seek moves the position without changing whether anything is
    /// playing, so restarting the interval on one would let somebody scrubbing
    /// steadily push the next interval report away indefinitely, and stopping it
    /// would leave a playing stream reporting nothing at all.
    #[must_use]
    pub const fn what_it_does_to_the_interval(self) -> WhatItDoesToTheInterval {
        match self {
            Self::Started | Self::Resumed => WhatItDoesToTheInterval::RunsFromHere,
            Self::Paused | Self::Stopped => WhatItDoesToTheInterval::Stops,
            Self::Seeked => WhatItDoesToTheInterval::Untouched,
        }
    }
}

/// Where the reporting interval for one item stands.
///
/// Every interval it measures is on the ELAPSED clock, which is what 0057 names.
/// 0102 puts a wait that has to survive a suspension there, and a device that
/// slept through three ticks owes one report on waking rather than three, which
/// is what a single reading of that clock gives.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheInterval {
    ran_from: ElapsedInstant,
    running: bool,
}

impl TheInterval {
    /// The interval running from the moment playback started.
    #[must_use]
    pub const fn running_from(at: ElapsedInstant) -> Self {
        Self {
            ran_from: at,
            running: true,
        }
    }

    /// The interval stopped, which is where a paused or ended stream leaves it.
    #[must_use]
    pub const fn stopped_at(at: ElapsedInstant) -> Self {
        Self {
            ran_from: at,
            running: false,
        }
    }

    /// Whether the interval is running.
    #[must_use]
    pub const fn is_running(self) -> bool {
        self.running
    }

    /// The moment the current interval runs from.
    #[must_use]
    pub const fn ran_from(self) -> ElapsedInstant {
        self.ran_from
    }

    /// Whether an interval report is due.
    ///
    /// FALSE WHILE PAUSED, ALWAYS, AND THAT IS THE DECISION RATHER THAN A GUARD.
    /// An interval that kept running would enqueue the same unmoving position
    /// repeatedly, 0047 would coalesce every one of them into the entry already
    /// there, and the only thing left would be the wake-up - which on a handheld
    /// is a wake-up per interval for as long as somebody leaves something
    /// paused, and that can be overnight.
    #[must_use]
    pub const fn a_report_is_due(self, now: ElapsedInstant) -> bool {
        self.running
            && now.interval_since(self.ran_from).as_nanos()
                >= A_POSITION_IS_REPORTED_EVERY.as_nanos()
    }

    /// How long until the next interval report is due, or `None` where the
    /// interval is not running.
    ///
    /// `None` rather than a very large duration, because a paused stream has no
    /// next report at all and a number here would be one a caller could wait on.
    #[must_use]
    pub const fn until_the_next_report(self, now: ElapsedInstant) -> Option<Duration> {
        if !self.running {
            return None;
        }
        let waited = now.interval_since(self.ran_from);
        if waited.as_nanos() >= A_POSITION_IS_REPORTED_EVERY.as_nanos() {
            Some(Duration::ZERO)
        } else {
            Some(A_POSITION_IS_REPORTED_EVERY.saturating_sub(waited))
        }
    }

    /// The interval after a report was made at `at`.
    #[must_use]
    pub const fn after_a_report_at(self, at: ElapsedInstant) -> Self {
        Self {
            ran_from: at,
            running: self.running,
        }
    }

    /// The interval after one of the five events, which reported at `at`
    /// whatever this answers.
    ///
    /// The report itself is not conditional on anything here: all five report
    /// the moment they happen, and this says only where the interval stands
    /// afterwards.
    #[must_use]
    pub const fn after(self, event: ReportsWithoutWaiting, at: ElapsedInstant) -> Self {
        match event.what_it_does_to_the_interval() {
            WhatItDoesToTheInterval::RunsFromHere => Self::running_from(at),
            WhatItDoesToTheInterval::Stops => Self::stopped_at(at),
            WhatItDoesToTheInterval::Untouched => self,
        }
    }
}

#[cfg(test)]
mod tests {
    //! 0057's interval, its five events and the constraint it puts on 0058,
    //! asked of the values.
    //!
    //! What these cannot ask is any of #57's three open conditions. Each is
    //! about a report being made and observed on a queue, and nothing in this
    //! tree plays anything.

    use super::{
        A_POSITION_IS_REPORTED_EVERY, ReportsWithoutWaiting, TheInterval, WhatItDoesToTheInterval,
    };
    use crate::clock::ElapsedInstant;
    use crate::playback::Ticks;
    use crate::playback::resume::Resume;
    use core::time::Duration;

    const NANOS_IN_A_SECOND: u64 = 1_000_000_000;

    fn at(seconds: u64) -> ElapsedInstant {
        ElapsedInstant::from_nanos(seconds * NANOS_IN_A_SECOND)
    }

    /// The interval, at the boundary itself rather than a value either side of
    /// it.
    #[test]
    fn a_report_is_due_ten_seconds_after_the_last_one() {
        let interval = TheInterval::running_from(at(0));

        assert!(!interval.a_report_is_due(at(9)));
        assert!(interval.a_report_is_due(at(10)));
        assert!(interval.a_report_is_due(at(600)));
        assert_eq!(A_POSITION_IS_REPORTED_EVERY.as_secs(), 10);
    }

    /// A report moves the interval on, so one report does not make the next due
    /// at once.
    #[test]
    fn a_report_starts_the_interval_again() {
        let interval = TheInterval::running_from(at(0)).after_a_report_at(at(10));

        assert_eq!(interval.ran_from(), at(10));
        assert!(!interval.a_report_is_due(at(19)));
        assert!(interval.a_report_is_due(at(20)));
    }

    /// Nothing is ever due while paused, which is what keeps a handheld from
    /// waking once per interval all night for a stream nobody is watching.
    #[test]
    fn nothing_is_due_while_paused_however_long_the_pause_is() {
        let paused = TheInterval::running_from(at(0)).after(ReportsWithoutWaiting::Paused, at(4));

        assert!(!paused.is_running());
        for now in [4_u64, 5, 14, 3600, 86_400] {
            assert!(
                !paused.a_report_is_due(at(now)),
                "a report became due {now} second(s) in while paused"
            );
            assert_eq!(paused.until_the_next_report(at(now)), None);
        }
    }

    /// Resuming starts the interval again from the resume, so the wait a person
    /// spent paused is not counted towards the next report.
    #[test]
    fn resuming_runs_the_interval_from_the_resume() {
        let interval = TheInterval::running_from(at(0))
            .after(ReportsWithoutWaiting::Paused, at(4))
            .after(ReportsWithoutWaiting::Resumed, at(3600));

        assert!(interval.is_running());
        assert!(!interval.a_report_is_due(at(3609)));
        assert!(interval.a_report_is_due(at(3610)));
    }

    /// What each of the five does to the interval, over the whole set rather
    /// than over whichever member somebody remembered.
    #[test]
    fn each_of_the_five_events_says_what_it_does_to_the_interval() {
        let effects: Vec<(&str, WhatItDoesToTheInterval)> = ReportsWithoutWaiting::all()
            .iter()
            .map(|event| (event.as_str(), event.what_it_does_to_the_interval()))
            .collect();

        assert_eq!(
            effects,
            [
                ("started", WhatItDoesToTheInterval::RunsFromHere),
                ("paused", WhatItDoesToTheInterval::Stops),
                ("resumed", WhatItDoesToTheInterval::RunsFromHere),
                ("seeked", WhatItDoesToTheInterval::Untouched),
                ("stopped", WhatItDoesToTheInterval::Stops),
            ]
        );
    }

    /// A seek leaves the interval where it was. Restarting it on every seek lets
    /// somebody scrubbing steadily push the next interval report away for as
    /// long as they keep scrubbing.
    #[test]
    fn a_seek_does_not_move_the_interval() {
        let mut interval = TheInterval::running_from(at(0));
        for scrub in 1..=9 {
            interval = interval.after(ReportsWithoutWaiting::Seeked, at(scrub));
        }

        assert_eq!(interval.ran_from(), at(0));
        assert!(interval.a_report_is_due(at(10)));
    }

    /// Five events and no sixth. Stopping the core is not one of them, for the
    /// reason 0115 gives.
    #[test]
    fn the_events_that_do_not_wait_are_five_and_each_reports_a_name_of_its_own() {
        let mut seen = Vec::new();
        for event in ReportsWithoutWaiting::all() {
            assert!(!event.as_str().is_empty());
            assert!(
                !seen.contains(&event.as_str()),
                "two events report one name"
            );
            seen.push(event.as_str());
        }
        assert_eq!(seen.len(), 5);
    }

    /// The wait remaining, and that it floors at zero rather than running
    /// negative once a report is overdue.
    #[test]
    fn the_wait_remaining_counts_down_and_stops_at_zero() {
        let interval = TheInterval::running_from(at(0));

        assert_eq!(
            interval.until_the_next_report(at(0)),
            Some(Duration::from_secs(10))
        );
        assert_eq!(
            interval.until_the_next_report(at(9)),
            Some(Duration::from_secs(1))
        );
        assert_eq!(interval.until_the_next_report(at(10)), Some(Duration::ZERO));
        assert_eq!(
            interval.until_the_next_report(at(600)),
            Some(Duration::ZERO)
        );
    }

    /// 0057's constraint on 0058, which is the one thing in that record that is
    /// about another record's number.
    ///
    /// The interval is the upper bound on how far back somebody is thrown, and a
    /// resume that rewinds by less than it does not absorb that loss. Neither
    /// number may be changed on its own: this is the disagreement made visible
    /// rather than latent, which is what 0057 says it wrote the constraint down
    /// for.
    #[test]
    fn the_reporting_interval_does_not_exceed_the_rewind() {
        let interval_in_seconds = i64::try_from(A_POSITION_IS_REPORTED_EVERY.as_secs())
            .expect("0057's interval is ten seconds and fits in the unit 0056 fixes");
        let interval = Ticks::from_seconds(interval_in_seconds);

        assert!(
            interval.as_ticks() <= Resume::REWIND.as_ticks(),
            "0057's interval of {} tick(s) exceeds 0058's rewind of {} tick(s), so one of the two records is wrong",
            interval.as_ticks(),
            Resume::REWIND.as_ticks()
        );
    }
}
