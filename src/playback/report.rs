//! The report: the one act that puts a playback position on the queue.
//!
//! `docs/decisions/0057-the-progress-reporting-cadence.md` is the record and
//! #57 is the issue. [`super::cadence`] holds what that record decides about
//! WHEN a report is made; this module is the report itself, which is the part
//! #57's three conditions are about: a scrub producing one report, each of the
//! five events reporting without waiting for the interval, and every report
//! observed passing through the queue in 0047.
//!
//! # There is one route and it is the queue
//!
//! 0057 says every report is put on the queue in 0047 rather than sent, and
//! 0047 says why in its first section: a path that sends directly and queues
//! only when the server is away is two paths that agree until they do not, and
//! the disagreement is reachable only on a device whose connectivity changed
//! mid-playback, which is the phone on the train the queue was built for. So
//! every call below that makes a report takes the queue as a parameter and
//! hands back what the queue answered. There is no second way out of this
//! module, and the type is what holds that rather than a sentence: a report is
//! [`WhatTheEnqueueDid`], and nothing here can produce one without asking the
//! queue.
//!
//! # A report is an assertion, and it always states the position
//!
//! 0047 queues assertions of a desired state and never deltas, so that a
//! delivery repeated after a flaky reconnection has no second effect. A
//! [`PositionReport`] is one: where playback of this item is, now. 0056 adds
//! that the core always states the position on every report, because the server
//! reads an absent position as the whole duration, which is the item finished.
//! The type has no absent value for that reason, and a position at the
//! beginning is a position rather than an absence.
//!
//! # Coalescing is not here, and that is 0057's own instruction
//!
//! #57 asks that a person scrubbing through a film sends one report rather than
//! forty. That is true here because 0047 coalesces at the moment of enqueue,
//! per target and per kind, and 0057 says writing a second rule for it is the
//! thing to avoid. What this module adds is only what that record adds: the
//! target is the item, the kind is the position, and every report of either
//! occasion below is the same kind, which is why forty seeks leave one entry.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is the reporting: on which occasions a report is made, what it
//! carries, and that it goes through the queue and nowhere else.
//!
//! WHAT IS NOT HERE IS THE DELIVERY. Nothing in this tree opens a connection,
//! so no report leaves the device and no drain runs. The queue is 0047's, the
//! drain runs on 0045's recovery report, and which request carries a report is
//! #10's table met by #27's transport. 0057's honest statement about the report
//! a person most expects to have landed applies to every report here: durable
//! rather than delivered, and today the queue is not durable either, which
//! [`crate::server::write_queue`] says of itself.
//!
//! WHAT IS ALSO NOT HERE IS THE PLAYER. 0003 puts decoding and presenting
//! outside the core, so the core never sees a stream and never learns a position
//! by itself. A client tells it: the five events as they happen, and the
//! position as often as the platform's player reports one. What the core decides
//! is whether that observation becomes a report, which is the interval's
//! question and is answered by [`super::cadence`] rather than by the caller.
//!
//! WHAT IS ALSO NOT HERE IS THE ADMISSION. 0056 applies its two bounds where a
//! value enters the core, and [`Reporting`] takes an [`AdmittedPosition`]
//! rather than a number, so a position that reaches this module has already been
//! admitted and cannot have arrived any other way.

use super::AdmittedPosition;
use super::Ticks;
use super::cadence::{ReportsWithoutWaiting, TheInterval};
use crate::clock::ElapsedInstant;
use crate::server::write_queue::{Target, WhatIsAsserted, WhatTheEnqueueDid, WriteQueue};

/// What occasioned a report.
///
/// Two occasions and no third. 0057 fixes that a report is made on each of the
/// five events the moment it happens, and on the interval while something is
/// playing, and it names nothing else that produces one.
///
/// It is carried on the report rather than discarded so that whatever delivers
/// the report can tell a stop from a tick, which #10's table separates by path.
/// It does not change what the report asserts, which is the position, and it
/// does not change how the queue coalesces, which is per target and per kind.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportedOn {
    /// One of the five events that report without waiting for the interval.
    Event(ReportsWithoutWaiting),
    /// The interval said a report was due.
    TheInterval,
}

/// What one report asserts: where playback of one item is.
///
/// This is the assertion 0047 requires of everything it queues, and it is the
/// value a later report for the same item replaces. Forty seeks in five seconds
/// produce forty of these and leave one on the queue, holding the last
/// position rather than the first, which 0057 states as the consequence worth
/// stating.
///
/// The position is always present, for 0056's reason: a report carrying no
/// position is read by the server as the item finished.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionReport {
    position: Ticks,
    reported_on: ReportedOn,
}

impl PositionReport {
    /// Where playback is, in 0056's unit.
    #[must_use]
    pub const fn position(self) -> Ticks {
        self.position
    }

    /// What occasioned this report.
    #[must_use]
    pub const fn reported_on(self) -> ReportedOn {
        self.reported_on
    }
}

/// What observing a position did.
///
/// Three answers and never nothing, so a caller that hands the core every
/// position the player produces can see which of them became a report, and
/// a case can assert that the ones between reports did not.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatObservingDid {
    /// The interval said a report was due, one was made, and this is what the
    /// queue did with it.
    Reported(WhatTheEnqueueDid),
    /// Playback is running and the next report is not due yet. The position was
    /// read and nothing was enqueued, which is the difference between this
    /// module and reporting on every position change, the alternative 0057
    /// prices at a request per second per stream.
    NotDueYet,
    /// Nothing is playing: the item is paused, stopped or not yet started, and
    /// 0057 fixes that nothing is ever due then, however long it lasts.
    NothingIsPlaying,
}

/// The reporting for one item within one session.
///
/// It holds the item a report is about and where 0057's interval stands for
/// it, and nothing else. It holds no position, because the position is the
/// player's and arrives with every call; it holds no queue, because the queue is
/// the session's and outlives any one item's playback, which is why every call
/// that reports takes the queue rather than owning one.
///
/// Thread safety, from 0009: a plain value, safe from any thread. It has no
/// interior mutability, so a caller sharing one across threads gives it the
/// same treatment as any other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reporting {
    target: Target,
    interval: TheInterval,
}

impl Reporting {
    /// The reporting for an item nothing is playing yet.
    ///
    /// The interval starts stopped, so a position observed before the started
    /// event answers [`WhatObservingDid::NothingIsPlaying`] rather than becoming
    /// a report about something that is not happening. What starts it is
    /// [`Reporting::report`] with [`ReportsWithoutWaiting::Started`], which is
    /// itself the first report.
    #[must_use]
    pub const fn for_item(target: Target, at: ElapsedInstant) -> Self {
        Self {
            target,
            interval: TheInterval::stopped_at(at),
        }
    }

    /// The item this reporting is about.
    #[must_use]
    pub const fn target(&self) -> &Target {
        &self.target
    }

    /// Where 0057's interval stands for this item.
    #[must_use]
    pub const fn interval(&self) -> TheInterval {
        self.interval
    }

    /// One of the five events happened: report it now, and move the interval
    /// the way that event moves it.
    ///
    /// The report is made before the interval is consulted and whatever the
    /// interval says, which is 0057's sentence that the five events report the
    /// moment they happen. A seek one second after the last tick reports; a
    /// pause nine seconds in reports; and the interval is then left where
    /// [`TheInterval::after`] puts it, which for a seek is where it was.
    ///
    /// What comes back is what the queue did, and it is the only thing that
    /// comes back: a report that was not asked of the queue does not exist.
    pub fn report(
        &mut self,
        event: ReportsWithoutWaiting,
        position: AdmittedPosition,
        at: ElapsedInstant,
        queue: &mut WriteQueue<PositionReport>,
    ) -> WhatTheEnqueueDid {
        let what_the_queue_did = queue.enqueue(
            self.target.clone(),
            WhatIsAsserted::PlaybackPosition,
            PositionReport {
                position: position.position(),
                reported_on: ReportedOn::Event(event),
            },
        );
        self.interval = self.interval.after(event, at);
        what_the_queue_did
    }

    /// The player reported where it is: make a report if the interval says one
    /// is due, and otherwise make none.
    ///
    /// This is the call a client makes as often as its player produces a
    /// position, and the whole of 0057's cadence is that most of those calls
    /// enqueue nothing. Reporting on every one of them is the first alternative
    /// that record refuses, and it is the shape this method has the moment the
    /// question below is deleted.
    ///
    /// A report made here moves the interval on from `now`, so the next one is
    /// due ten seconds after this one rather than ten seconds after the last
    /// event.
    pub fn observe(
        &mut self,
        position: AdmittedPosition,
        now: ElapsedInstant,
        queue: &mut WriteQueue<PositionReport>,
    ) -> WhatObservingDid {
        if !self.interval.is_running() {
            return WhatObservingDid::NothingIsPlaying;
        }
        if !self.interval.a_report_is_due(now) {
            return WhatObservingDid::NotDueYet;
        }
        let what_the_queue_did = queue.enqueue(
            self.target.clone(),
            WhatIsAsserted::PlaybackPosition,
            PositionReport {
                position: position.position(),
                reported_on: ReportedOn::TheInterval,
            },
        );
        self.interval = self.interval.after_a_report_at(now);
        WhatObservingDid::Reported(what_the_queue_did)
    }
}

#[cfg(test)]
mod tests {
    //! #57's three conditions, asked of the report and the queue together.
    //!
    //! A scrub producing one report, each of the five events reporting without
    //! waiting for the interval, and every report observed passing through the
    //! queue. What these cannot ask is delivery: nothing here sends a byte, and
    //! the queue that holds every report is not durable.

    use super::{PositionReport, ReportedOn, Reporting, WhatObservingDid};
    use crate::clock::ElapsedInstant;
    use crate::playback::cadence::ReportsWithoutWaiting;
    use crate::playback::{AdmittedPosition, Ticks};
    use crate::server::write_queue::{
        Entry, Target, WhatIsAsserted, WhatTheEnqueueDid, WriteQueue,
    };

    const NANOS_IN_A_SECOND: u64 = 1_000_000_000;

    fn at(seconds: u64) -> ElapsedInstant {
        ElapsedInstant::from_nanos(seconds * NANOS_IN_A_SECOND)
    }

    fn played_to(seconds: i64) -> AdmittedPosition {
        AdmittedPosition::of(Ticks::from_seconds(seconds).as_ticks(), None)
    }

    fn item(identifier: &str) -> Target {
        Target::item(identifier.to_string())
    }

    /// An item started at second zero, with its one report already on the
    /// queue.
    fn playing(identifier: &str, queue: &mut WriteQueue<PositionReport>) -> Reporting {
        let mut reporting = Reporting::for_item(item(identifier), at(0));
        let did = reporting.report(ReportsWithoutWaiting::Started, played_to(0), at(0), queue);
        assert_eq!(did, WhatTheEnqueueDid::Added);
        reporting
    }

    /// The one entry the queue holds for an item, which is the last thing said
    /// about it.
    fn the_entry_for(identifier: &str, queue: &WriteQueue<PositionReport>) -> PositionReport {
        let entries: Vec<&PositionReport> = queue
            .entries()
            .iter()
            .filter(|entry| entry.target() == &item(identifier))
            .map(Entry::assertion)
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "the queue holds {} entries for one item",
            entries.len()
        );
        *entries[0]
    }

    /// #57's first condition. Forty seeks in five seconds are forty reports and
    /// one entry, holding the last position rather than the first, which is
    /// 0047's coalescing arriving through this module rather than a rule of its
    /// own.
    #[test]
    fn a_scrub_produces_one_report() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);

        let mut answers = Vec::new();
        for scrub in 1..=40_i64 {
            let position = played_to(scrub * 60);
            let moment = at(u64::try_from(scrub).expect("forty is a small number") / 8);
            answers.push(reporting.report(
                ReportsWithoutWaiting::Seeked,
                position,
                moment,
                &mut queue,
            ));
        }

        assert!(
            answers
                .iter()
                .all(|answer| *answer == WhatTheEnqueueDid::ReplacedInPlace),
            "a seek did something other than replace the entry in place: {answers:?}"
        );
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.dropped(), 0);
        let entry = the_entry_for("the-film", &queue);
        assert_eq!(entry.position(), Ticks::from_seconds(40 * 60));
        assert_eq!(
            entry.reported_on(),
            ReportedOn::Event(ReportsWithoutWaiting::Seeked)
        );
    }

    /// #57's second condition, over the whole set of five rather than the
    /// members somebody remembered. Each is reported one second into an
    /// interval that is not due, and an observation at the same moment shows
    /// the interval was indeed not due.
    #[test]
    fn each_immediate_event_reports_without_waiting_for_the_interval() {
        for event in ReportsWithoutWaiting::all() {
            let mut queue = WriteQueue::empty();
            let mut reporting = playing("the-film", &mut queue);
            let before = reporting.interval();
            assert!(
                !before.a_report_is_due(at(1)),
                "the interval was already due, so this case proves nothing"
            );

            let did = reporting.report(*event, played_to(1), at(1), &mut queue);

            assert_eq!(
                did,
                WhatTheEnqueueDid::ReplacedInPlace,
                "{} did not reach the queue",
                event.as_str()
            );
            let entry = the_entry_for("the-film", &queue);
            assert_eq!(entry.reported_on(), ReportedOn::Event(*event));
            assert_eq!(entry.position(), Ticks::from_seconds(1));
        }
    }

    /// The other half of the second condition: the interval is not what makes
    /// an event report, so an observation at the same moment an event would
    /// have reported makes none. This is the case that goes red when the
    /// question in [`Reporting::observe`] is deleted, which is 0057's first
    /// refused alternative.
    #[test]
    fn a_position_observed_before_the_interval_is_due_is_not_reported() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);

        for second in 1..=9_i64 {
            let did = reporting.observe(
                played_to(second),
                at(u64::try_from(second).expect("nine is a small number")),
                &mut queue,
            );
            assert_eq!(
                did,
                WhatObservingDid::NotDueYet,
                "a position {second} second(s) in became a report"
            );
        }

        assert_eq!(queue.len(), 1);
        assert_eq!(
            the_entry_for("the-film", &queue).reported_on(),
            ReportedOn::Event(ReportsWithoutWaiting::Started)
        );
    }

    /// #57's third condition. Every report a whole viewing produces comes back
    /// as what the queue did with it, and the queue afterwards holds exactly
    /// what coalescing those reports leaves. There is no other route: a report
    /// this module made and the queue did not see is not a value this module
    /// can produce.
    #[test]
    fn every_report_passes_through_the_queue() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);
        let mut reports = vec![WhatTheEnqueueDid::Added];

        let observed = |reporting: &mut Reporting,
                        queue: &mut WriteQueue<PositionReport>,
                        second: i64,
                        reports: &mut Vec<WhatTheEnqueueDid>| {
            match reporting.observe(
                played_to(second),
                at(u64::try_from(second).expect("a viewing fits in a day")),
                queue,
            ) {
                WhatObservingDid::Reported(did) => reports.push(did),
                WhatObservingDid::NotDueYet | WhatObservingDid::NothingIsPlaying => {}
            }
        };

        observed(&mut reporting, &mut queue, 5, &mut reports);
        observed(&mut reporting, &mut queue, 10, &mut reports);
        observed(&mut reporting, &mut queue, 20, &mut reports);
        reports.push(reporting.report(
            ReportsWithoutWaiting::Paused,
            played_to(25),
            at(25),
            &mut queue,
        ));
        observed(&mut reporting, &mut queue, 3600, &mut reports);
        reports.push(reporting.report(
            ReportsWithoutWaiting::Resumed,
            played_to(25),
            at(3600),
            &mut queue,
        ));
        reports.push(reporting.report(
            ReportsWithoutWaiting::Seeked,
            played_to(30),
            at(3602),
            &mut queue,
        ));
        observed(&mut reporting, &mut queue, 3610, &mut reports);
        reports.push(reporting.report(
            ReportsWithoutWaiting::Stopped,
            played_to(45),
            at(3620),
            &mut queue,
        ));

        assert_eq!(
            reports.len(),
            8,
            "the viewing made {} report(s)",
            reports.len()
        );
        assert_eq!(reports[0], WhatTheEnqueueDid::Added);
        assert!(
            reports[1..]
                .iter()
                .all(|did| *did == WhatTheEnqueueDid::ReplacedInPlace)
        );
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.dropped(), 0);
        assert_eq!(
            queue.entries()[0].asserted_about(),
            WhatIsAsserted::PlaybackPosition
        );
        let entry = the_entry_for("the-film", &queue);
        assert_eq!(entry.position(), Ticks::from_seconds(45));
        assert_eq!(
            entry.reported_on(),
            ReportedOn::Event(ReportsWithoutWaiting::Stopped)
        );
    }

    /// Two items are two entries, which is 0057's target being the item within
    /// the session: the person who started something else and came back has
    /// told the server about two things.
    #[test]
    fn reports_for_two_items_do_not_collapse_into_one() {
        let mut queue = WriteQueue::empty();
        let mut first = playing("the-film", &mut queue);
        let mut second = Reporting::for_item(item("the-episode"), at(0));

        let did = second.report(
            ReportsWithoutWaiting::Started,
            played_to(0),
            at(1),
            &mut queue,
        );
        assert_eq!(did, WhatTheEnqueueDid::Added);
        first.report(
            ReportsWithoutWaiting::Seeked,
            played_to(90),
            at(2),
            &mut queue,
        );
        second.report(
            ReportsWithoutWaiting::Seeked,
            played_to(15),
            at(3),
            &mut queue,
        );

        assert_eq!(queue.len(), 2);
        assert_eq!(
            the_entry_for("the-film", &queue).position(),
            Ticks::from_seconds(90)
        );
        assert_eq!(
            the_entry_for("the-episode", &queue).position(),
            Ticks::from_seconds(15)
        );
    }

    /// The interval, driven through the report rather than read on its own:
    /// due ten seconds after the last report and not nine, and a report made
    /// here moves it on. Deleting the move leaves the next report due at once.
    #[test]
    fn an_interval_report_is_made_ten_seconds_after_the_last_and_not_before() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);

        assert_eq!(
            reporting.observe(played_to(9), at(9), &mut queue),
            WhatObservingDid::NotDueYet
        );
        assert_eq!(
            reporting.observe(played_to(10), at(10), &mut queue),
            WhatObservingDid::Reported(WhatTheEnqueueDid::ReplacedInPlace)
        );
        assert_eq!(
            the_entry_for("the-film", &queue).reported_on(),
            ReportedOn::TheInterval
        );
        assert_eq!(
            reporting.observe(played_to(19), at(19), &mut queue),
            WhatObservingDid::NotDueYet
        );
        assert_eq!(
            reporting.observe(played_to(20), at(20), &mut queue),
            WhatObservingDid::Reported(WhatTheEnqueueDid::ReplacedInPlace)
        );
        assert_eq!(
            the_entry_for("the-film", &queue).position(),
            Ticks::from_seconds(20)
        );
    }

    /// A seek reports and leaves the interval alone, so a person scrubbing
    /// steadily does not push the next interval report away.
    #[test]
    fn a_seek_reports_and_does_not_move_the_interval() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);

        for second in 1..=9_i64 {
            reporting.report(
                ReportsWithoutWaiting::Seeked,
                played_to(second * 100),
                at(u64::try_from(second).expect("nine is a small number")),
                &mut queue,
            );
        }

        assert_eq!(
            reporting.observe(played_to(901), at(10), &mut queue),
            WhatObservingDid::Reported(WhatTheEnqueueDid::ReplacedInPlace)
        );
    }

    /// Nothing is reported while paused however long the pause is, which is
    /// the wake-up per interval overnight that 0057 wrote the rule against.
    /// The event itself reported, and that is the one entry.
    #[test]
    fn nothing_is_reported_while_paused_however_long_the_pause_is() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);
        reporting.report(
            ReportsWithoutWaiting::Paused,
            played_to(4),
            at(4),
            &mut queue,
        );

        for now in [4_u64, 5, 14, 3600, 86_400] {
            assert_eq!(
                reporting.observe(played_to(4), at(now), &mut queue),
                WhatObservingDid::NothingIsPlaying,
                "a report was made {now} second(s) in while paused"
            );
        }

        assert_eq!(queue.len(), 1);
        assert_eq!(
            the_entry_for("the-film", &queue).reported_on(),
            ReportedOn::Event(ReportsWithoutWaiting::Paused)
        );
    }

    /// Resuming reports, and the interval runs from the resume rather than from
    /// the pause, so the time spent paused is not counted towards the next
    /// report.
    #[test]
    fn resuming_reports_and_runs_the_interval_from_the_resume() {
        let mut queue = WriteQueue::empty();
        let mut reporting = playing("the-film", &mut queue);
        reporting.report(
            ReportsWithoutWaiting::Paused,
            played_to(4),
            at(4),
            &mut queue,
        );
        let did = reporting.report(
            ReportsWithoutWaiting::Resumed,
            played_to(4),
            at(3600),
            &mut queue,
        );

        assert_eq!(did, WhatTheEnqueueDid::ReplacedInPlace);
        assert_eq!(
            reporting.observe(played_to(13), at(3609), &mut queue),
            WhatObservingDid::NotDueYet
        );
        assert_eq!(
            reporting.observe(played_to(14), at(3610), &mut queue),
            WhatObservingDid::Reported(WhatTheEnqueueDid::ReplacedInPlace)
        );
    }

    /// A position observed before anything started is not a report about
    /// something that is not happening.
    #[test]
    fn nothing_is_reported_before_playback_started() {
        let mut queue = WriteQueue::empty();
        let mut reporting = Reporting::for_item(item("the-film"), at(0));

        assert_eq!(
            reporting.observe(played_to(0), at(30), &mut queue),
            WhatObservingDid::NothingIsPlaying
        );
        assert!(queue.is_empty());
        assert_eq!(reporting.target(), &item("the-film"));
    }

    /// 0056's rule that the core always states the position, on every report.
    /// A report at the beginning carries the beginning, which is a position and
    /// not an absence, and the type has no absent value to carry instead.
    #[test]
    fn a_report_always_states_the_position() {
        let mut queue = WriteQueue::empty();
        let mut reporting = Reporting::for_item(item("the-film"), at(0));
        reporting.report(
            ReportsWithoutWaiting::Started,
            played_to(0),
            at(0),
            &mut queue,
        );

        let entry = the_entry_for("the-film", &queue);
        assert_eq!(entry.position(), Ticks::ZERO);
        assert_eq!(
            entry.reported_on(),
            ReportedOn::Event(ReportsWithoutWaiting::Started)
        );
    }
}
