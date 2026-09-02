//! The four states a request ages through, and when a server itself is gone.
//!
//! `docs/decisions/0007-a-slow-server-and-a-server-that-is-gone.md` is the
//! record and #44 is the issue. 0007 decides that slow and absent are separate
//! conditions with separate recoveries, so the core reports four states rather
//! than one pending state, reports them progressively as a request ages rather
//! than once at the end, and never makes a caller wait for the network to learn
//! what the cache could already have given it.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that two readings of one clock and a counter
//! settle: when a request has been outstanding long enough to be called late,
//! when it is abandoned, that each of those is said once and never unsaid, which
//! transport outcomes are evidence the server is absent, and how many
//! abandonments with no success between them make a server unreachable rather
//! than a request unlucky.
//!
//! WHAT IS NOT HERE IS THE REQUEST. Nothing in this tree opens a connection, for
//! the reason [`super::transport`] gives about itself, so nothing here starts a
//! request, waits on one, cancels one or hears an answer. This module holds the
//! states such a loop would report from. #44's own condition drives the fake
//! server at several delays and asserts the sequence, and none of it is met by
//! anything here.
//!
//! WHAT IS ALSO NOT HERE IS THE CACHED ANSWER EACH REPORT CARRIES. 0007 has the
//! report at nought carry whether a cached answer exists and its age, and the
//! `late` and `abandoned` reports carry the same. That age is 0043's, the read
//! that produces it is #43, and a field here holding a number nothing can
//! compute would be a shape rather than a fact. The states are what a report is
//! built around and the payload is the caller's to attach.
//!
//! WHAT IS ALSO NOT HERE IS THE RECOVERY. What happens after a server is
//! declared unreachable is 0045 and is in [`super::recovery`], which is the
//! schedule this module's last state hands over to. Nothing here probes, and
//! nothing here reports a server answering again.
//!
//! # Why the abandonment count is here rather than beside the retry policy
//!
//! [`super::retry`] says of itself that the count is not there: 0038 decides
//! that a call spending all three attempts is one abandonment rather than three,
//! because 0007 declares a server unreachable after two consecutive ones, and
//! that module records the decision and enforces nothing, so a loop charging
//! three would pass every check in it. The count is 0007's rather than 0038's -
//! it is about the server rather than about a call - and
//! [`ConsecutiveAbandonments`] is where charging it wrongly now fails a case.
//!
//! What that still does not reach is the caller. Nothing in this tree runs a
//! call, so nothing charges this counter, and a loop that charged three per call
//! would fail nothing here either. What has moved is that the rule has a subject
//! and a near miss instead of only a paragraph.
//!
//! # Every number here is chosen and none is measured
//!
//! 0007 says so of its own thresholds, in the same words 0038 uses for its waits
//! and 0045 for its three. #65 is the harness that would replace a choice with a
//! number and #62 is where the budget the 400 ms is divided out of is
//! established, so until both exist a reader should take each constant below as
//! an argument rather than as a measurement.

use core::time::Duration;

use super::transport::A_CALL_IS_ABANDONED_AFTER;
use crate::clock::SteadyInstant;
use crate::failure::TransportOutcome;

/// How long a request may be outstanding before it is late.
///
/// From 0007, and derived rather than chosen for its shape. #62 publishes 1.2
/// seconds from a cold start to the first usable tile, and a client that is
/// going to commit to what the cache already gave it needs that decision made
/// with enough of the budget left to read, decode and draw. The core cannot know
/// how long a draw takes on a television, so it takes one third of the budget
/// for its own attempt and leaves two thirds, and one third of 1.2 seconds is
/// 400 ms.
///
/// THE DERIVATION IS THE PART THAT MATTERS AND IT IS NOT VISIBLE FROM THE
/// NUMBER. 400 ms is defensible only while the budget it was divided out of is
/// 1.2 seconds; a record says so where a constant cannot, and this is the
/// sentence that carries it to the constant.
pub const A_REQUEST_IS_LATE_AFTER: Duration = Duration::from_millis(400);

/// How many abandonments with no success between them make the server
/// unreachable rather than the request unlucky.
///
/// From 0007. One abandonment is a fact about a request and two consecutive ones
/// are a fact about the server. At five seconds each that is ten seconds before
/// the core says what somebody watching the screen worked out earlier, and a
/// third would make it fifteen: two is where the accounting stops being useful
/// and starts being ceremony.
pub const ABANDONMENTS_BEFORE_A_SERVER_IS_GONE: u32 = 2;

/// One of 0007's four states.
///
/// Four and no fifth. A fifth is a change to that record rather than a variant
/// added here, because each of these is a different thing for a client to draw
/// and the record argues each one against the two beside it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum State {
    /// An answer arrived from the server and is current.
    Fresh,
    /// A request is still outstanding and has passed the point at which waiting
    /// for it can still meet the published budget. THE REQUEST HAS NOT FAILED,
    /// and a client that draws this as an error shows a person a spinner and
    /// then an error for a request that was about to succeed.
    Late,
    /// The core stopped waiting for that request. It says nothing about the
    /// server beyond that one request, and a client that reads it as a server
    /// being gone tells a person their server is down because one endpoint was
    /// slow.
    Abandoned,
    /// The server rather than one request. Nothing is attempted against it until
    /// 0045's schedule says otherwise.
    Unreachable,
}

impl State {
    /// Every state, so a caller reads the set out of the crate rather than
    /// keeping a copy of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Fresh, Self::Late, Self::Abandoned, Self::Unreachable]
    }

    /// The name this state is reported as.
    ///
    /// It is what a report carries rather than the text a debug printing would
    /// produce, for the reason 0100 gives: a name that changed when somebody
    /// renamed a variant would change what every client's report says.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Late => "late",
            Self::Abandoned => "abandoned",
            Self::Unreachable => "unreachable",
        }
    }
}

/// What a reading of the clock says a caller should report now.
///
/// It is the CHANGE rather than the state, which is what "progressively" means:
/// a loop that asked twice inside the same stage would report twice, and a
/// client that received `late` a second time has no way to tell it from a second
/// request going late.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhatToReport {
    /// Nothing has changed since the last reading.
    Nothing,
    /// The request has passed 400 ms and is still outstanding.
    Late,
    /// The request has passed the call deadline and the core has stopped waiting.
    Abandoned,
}

/// One outstanding request, as it ages.
///
/// It holds the moment the request began and what has already been said about
/// it, and answers what a reading of the clock has newly made true. It performs
/// nothing: abandoning a request is the caller's act, and this says when 0007
/// requires it.
///
/// The clock is the steady one, from 0102's table by way of
/// [`super::transport::CallDeadline`], which measures the same call against the
/// same reading. An interval inside one call is exactly what that clock is for,
/// and a wall clock here would make a request late because somebody corrected
/// the time.
///
/// Thread safety, from 0009: a plain value, safe from any thread. It is not
/// shared between threads by anything here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgingRequest {
    began: SteadyInstant,
    said: WhatHasBeenSaid,
}

/// How far through 0007's sequence one request has been reported.
///
/// It is private and ordered, and both are the mechanism: the only transition a
/// reading can make is forward, so a state cannot be unsaid and cannot be said
/// twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WhatHasBeenSaid {
    OnlyThatItStarted,
    ThatItIsLate,
    ThatItWasAbandoned,
}

impl AgingRequest {
    /// A request that has just been made.
    ///
    /// 0007's report at nought - which session it belongs to, whether a cached
    /// answer exists and its age - is the caller's and is made before any byte
    /// leaves the machine. This type begins after it, holding only the moment.
    #[must_use]
    pub const fn began_at(began: SteadyInstant) -> Self {
        Self {
            began,
            said: WhatHasBeenSaid::OnlyThatItStarted,
        }
    }

    /// How long the request has been outstanding.
    #[must_use]
    pub fn age_at(self, now: SteadyInstant) -> Duration {
        now.interval_since(self.began)
    }

    /// What this reading has newly made true, recording that it was said.
    ///
    /// The thresholds are reached at rather than passed: a reading exactly at
    /// 400 ms is late, and one exactly at the call deadline is abandoned, which
    /// is the boundary [`super::transport::CallDeadline::passed_at`] takes for
    /// the same deadline.
    ///
    /// A READING PAST THE DEADLINE THAT NEVER SAW 400 MS REPORTS THE
    /// ABANDONMENT ALONE, and that is a decision rather than an oversight. The
    /// `late` report exists so a client can commit to a cached answer with
    /// enough budget left to draw it; five seconds later that budget is spent,
    /// and emitting it then would tell a client something is about to be true
    /// which stopped being true four and a half seconds ago. What it costs is a
    /// client that counts reports rather than reading them, and 0007 already
    /// says none of these is a sentence for a person.
    pub fn what_to_report_at(&mut self, now: SteadyInstant) -> WhatToReport {
        let age = self.age_at(now);

        if age >= A_CALL_IS_ABANDONED_AFTER {
            if self.said < WhatHasBeenSaid::ThatItWasAbandoned {
                self.said = WhatHasBeenSaid::ThatItWasAbandoned;
                return WhatToReport::Abandoned;
            }
            return WhatToReport::Nothing;
        }

        if age >= A_REQUEST_IS_LATE_AFTER && self.said < WhatHasBeenSaid::ThatItIsLate {
            self.said = WhatHasBeenSaid::ThatItIsLate;
            return WhatToReport::Late;
        }

        WhatToReport::Nothing
    }
}

/// What one transport outcome says about the server, before any counting.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhatAnOutcomeSaysAboutTheServer {
    /// The server is not there, and 0007 reports it at once with no threshold
    /// because none of the three outcomes that reach this involved waiting.
    ItIsNotThere,
    /// Nothing. Whatever went wrong is about this request, or about a peer that
    /// answered.
    NothingAboutTheServer,
}

/// Whether an outcome is evidence the server is absent.
///
/// Three outcomes are, and none of them needs a threshold: the name did not
/// resolve, the connection was refused, or the network could not be reached.
///
/// A CERTIFICATE THE CORE WILL NOT ACCEPT IS NOT ONE OF THEM, and that is the
/// near miss 0007 names. It is a server that answered, it is 0029's outcome with
/// its own identity in 0004, and reporting it as unreachable sends a person
/// looking for a network problem that does not exist. A body that dropped or a
/// deadline that expired are not evidence either: the first is a server that was
/// answering and the second is what the count below is for.
#[must_use]
pub const fn what_an_outcome_says(
    outcome: &TransportOutcome<'_>,
) -> WhatAnOutcomeSaysAboutTheServer {
    match *outcome {
        TransportOutcome::NameDidNotResolve
        | TransportOutcome::ConnectionRefused
        | TransportOutcome::NetworkUnreachable => WhatAnOutcomeSaysAboutTheServer::ItIsNotThere,
        TransportOutcome::ConnectionDroppedMidBody
        | TransportOutcome::DeadlineReached { .. }
        | TransportOutcome::AnswerStalledMidBody { .. }
        | TransportOutcome::PeerNotTrusted { .. } => {
            WhatAnOutcomeSaysAboutTheServer::NothingAboutTheServer
        }
    }
}

/// Abandonments against one server with no success between them.
///
/// One counter per server, held by whatever holds a server. Nothing in this tree
/// holds one, which is the module documentation's point about what this does not
/// reach.
///
/// Thread safety, from 0009: a plain value, safe from any thread. It is not
/// shared between threads by anything here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConsecutiveAbandonments {
    counted: u32,
}

impl ConsecutiveAbandonments {
    /// A server nothing has been abandoned against.
    #[must_use]
    pub const fn none() -> Self {
        Self { counted: 0 }
    }

    /// How many are standing.
    #[must_use]
    pub const fn counted(self) -> u32 {
        self.counted
    }

    /// Charges one abandonment and answers whether the server is now gone.
    ///
    /// ONE CALL IS ONE ABANDONMENT HOWEVER MANY ATTEMPTS IT SPENT. 0038 decides
    /// that, and this signature is where it is expressed: there is no argument
    /// for a number of attempts, so a caller cannot charge three for a call that
    /// used its three.
    ///
    /// The count saturates rather than wrapping. A third and a tenth are the
    /// same state as the second, and an unreachable server that keeps being
    /// asked is 0045's schedule rather than this counter's problem.
    pub const fn abandoned(&mut self) -> State {
        self.counted = self.counted.saturating_add(1);
        if self.counted >= ABANDONMENTS_BEFORE_A_SERVER_IS_GONE {
            State::Unreachable
        } else {
            State::Abandoned
        }
    }

    /// Records that the server answered, which resets the count.
    ///
    /// A server that answered is not gone, and 0007 says so as the reason rather
    /// than as a consequence: the count is of CONSECUTIVE abandonments, so one
    /// success anywhere in the sequence ends the run.
    pub const fn answered(&mut self) {
        self.counted = 0;
    }

    /// Records evidence the server is absent, which is unreachable at once.
    ///
    /// The count is left where it is rather than being raised to the threshold.
    /// What made the server unreachable is the evidence and not the accounting,
    /// and a counter moved by something that did not wait would make the next
    /// abandonment after a recovery the second of a pair that never happened.
    #[must_use]
    pub const fn evidence_of_absence(self) -> State {
        State::Unreachable
    }
}

#[cfg(test)]
mod tests {
    //! 0007's thresholds and its counter, asked of the values.
    //!
    //! Every case moves a clock rather than waiting on one, which is what 0102
    //! requires of the suite and what makes a five second threshold cost
    //! microseconds. What these cannot ask is #44's own condition, which drives
    //! the fake server at several delays; nothing in this tree makes a request
    //! for a delay to answer.

    use super::{
        A_REQUEST_IS_LATE_AFTER, ABANDONMENTS_BEFORE_A_SERVER_IS_GONE, AgingRequest,
        ConsecutiveAbandonments, State, WhatAnOutcomeSaysAboutTheServer, WhatToReport,
        what_an_outcome_says,
    };
    use crate::clock::SteadyInstant;
    use crate::failure::{CertificateReason, Deadline, TransportOutcome};
    use crate::server::transport::A_CALL_IS_ABANDONED_AFTER;
    use core::time::Duration;

    /// A reading of the steady clock, from whole milliseconds.
    fn at(millis: u64) -> SteadyInstant {
        SteadyInstant::from_nanos(millis * 1_000_000)
    }

    fn began() -> AgingRequest {
        AgingRequest::began_at(at(0))
    }

    /// The two thresholds are the record's, and the second is read out of the
    /// transport rather than written twice: two numbers that are meant to agree
    /// are two numbers that will not.
    #[test]
    fn the_thresholds_are_the_ones_0007_states() {
        assert_eq!(A_REQUEST_IS_LATE_AFTER, Duration::from_millis(400));
        assert_eq!(A_CALL_IS_ABANDONED_AFTER, Duration::from_secs(5));
        assert_eq!(ABANDONMENTS_BEFORE_A_SERVER_IS_GONE, 2);
    }

    /// The boundary itself rather than a value either side of it. The near miss
    /// is a strict comparison, which leaves a request that is exactly at the
    /// threshold reported as though nothing had happened.
    #[test]
    fn a_request_is_late_at_four_hundred_milliseconds_and_not_before() {
        let mut request = began();
        assert_eq!(request.what_to_report_at(at(399)), WhatToReport::Nothing);
        assert_eq!(request.what_to_report_at(at(400)), WhatToReport::Late);
    }

    /// Said once. The near miss is a loop polling every frame, which would
    /// report `late` sixty times a second and give a client no way to tell one
    /// slow request from sixty.
    #[test]
    fn late_is_reported_once_however_often_the_clock_is_read() {
        let mut request = began();
        assert_eq!(request.what_to_report_at(at(400)), WhatToReport::Late);
        for reading in [401, 500, 1_200, 4_999] {
            assert_eq!(
                request.what_to_report_at(at(reading)),
                WhatToReport::Nothing,
                "late was reported a second time at {reading} ms"
            );
        }
    }

    /// The whole sequence for a request that is watched throughout, which is
    /// what #44 calls the exact sequence of stages.
    #[test]
    fn a_watched_request_reports_late_then_abandoned_and_then_nothing() {
        let mut request = began();
        let reported: Vec<WhatToReport> = [0, 399, 400, 1_200, 4_999, 5_000, 5_001, 60_000]
            .into_iter()
            .map(|millis| request.what_to_report_at(at(millis)))
            .collect();

        assert_eq!(
            reported,
            vec![
                WhatToReport::Nothing,
                WhatToReport::Nothing,
                WhatToReport::Late,
                WhatToReport::Nothing,
                WhatToReport::Nothing,
                WhatToReport::Abandoned,
                WhatToReport::Nothing,
                WhatToReport::Nothing,
            ]
        );
    }

    /// A request nobody looked at until the deadline had passed reports the
    /// abandonment alone. The reason is on `what_to_report_at`.
    #[test]
    fn a_request_first_read_past_the_deadline_does_not_report_a_late_that_is_over() {
        let mut request = began();
        assert_eq!(
            request.what_to_report_at(at(5_000)),
            WhatToReport::Abandoned
        );
        assert_eq!(request.what_to_report_at(at(5_400)), WhatToReport::Nothing);
    }

    /// Never backwards. A clock that answered with an earlier reading than the
    /// last one - which 0102 says the steady clock does not do, and which a
    /// caller can still hand in - unsays nothing.
    #[test]
    fn a_reading_that_went_backwards_unsays_nothing() {
        let mut request = began();
        assert_eq!(
            request.what_to_report_at(at(5_000)),
            WhatToReport::Abandoned
        );
        assert_eq!(request.what_to_report_at(at(10)), WhatToReport::Nothing);
    }

    /// The age is the interval since the request began, which is what every
    /// report 0007 describes carries beside its state.
    #[test]
    fn the_age_is_the_interval_since_the_request_began() {
        let request = AgingRequest::began_at(at(1_000));
        assert_eq!(request.age_at(at(1_450)), Duration::from_millis(450));
        assert_eq!(request.age_at(at(999)), Duration::ZERO);
    }

    /// The three outcomes that are evidence, and the four that are not. The near
    /// miss is the certificate, which 0007 names because it is a server that
    /// answered.
    #[test]
    fn only_the_three_outcomes_that_never_reached_a_server_are_evidence_of_absence() {
        for outcome in [
            TransportOutcome::NameDidNotResolve,
            TransportOutcome::ConnectionRefused,
            TransportOutcome::NetworkUnreachable,
        ] {
            assert_eq!(
                what_an_outcome_says(&outcome),
                WhatAnOutcomeSaysAboutTheServer::ItIsNotThere,
                "{outcome:?} is one of 0007's three and was not read as one"
            );
        }

        for outcome in [
            TransportOutcome::ConnectionDroppedMidBody,
            TransportOutcome::DeadlineReached {
                deadline: Deadline::WholeRequest,
                elapsed: Duration::from_secs(5),
            },
            TransportOutcome::AnswerStalledMidBody {
                deadline: Deadline::WholeRequest,
                elapsed: Duration::from_secs(5),
            },
            TransportOutcome::PeerNotTrusted {
                reason: CertificateReason::SelfSigned,
                fingerprint: "a-fingerprint",
            },
        ] {
            assert_eq!(
                what_an_outcome_says(&outcome),
                WhatAnOutcomeSaysAboutTheServer::NothingAboutTheServer,
                "{outcome:?} was read as evidence that the server is absent"
            );
        }
    }

    /// One abandonment is about the request and the second is about the server.
    #[test]
    fn the_second_consecutive_abandonment_is_the_server_and_the_first_is_not() {
        let mut against = ConsecutiveAbandonments::none();
        assert_eq!(against.abandoned(), State::Abandoned);
        assert_eq!(against.counted(), 1);
        assert_eq!(against.abandoned(), State::Unreachable);
        assert_eq!(against.counted(), 2);
    }

    /// A success anywhere in the run ends it. The near miss is a counter that
    /// only ever rises, which declares a server gone on two abandonments a week
    /// apart with a thousand successful calls in between.
    #[test]
    fn a_success_resets_the_run() {
        let mut against = ConsecutiveAbandonments::none();
        assert_eq!(against.abandoned(), State::Abandoned);
        against.answered();
        assert_eq!(against.counted(), 0);
        assert_eq!(
            against.abandoned(),
            State::Abandoned,
            "an abandonment after a success was charged as the second of a pair"
        );
    }

    /// Past the threshold the answer does not change and the count does not
    /// wrap.
    #[test]
    fn a_server_already_gone_stays_gone_and_the_count_saturates() {
        let mut against = ConsecutiveAbandonments::none();
        for _ in 0..4 {
            let _ = against.abandoned();
        }
        assert_eq!(against.counted(), 4);
        assert_eq!(against.abandoned(), State::Unreachable);

        let mut high = ConsecutiveAbandonments::none();
        for _ in 0..3 {
            let _ = high.abandoned();
        }
        assert!(high.counted() >= ABANDONMENTS_BEFORE_A_SERVER_IS_GONE);
    }

    /// Evidence of absence needs no count and moves none. The near miss is a
    /// counter raised to the threshold by something that did not wait, which
    /// makes the first abandonment after a recovery the second of a pair that
    /// never happened.
    #[test]
    fn evidence_of_absence_is_unreachable_at_once_and_leaves_the_count_alone() {
        let against = ConsecutiveAbandonments::none();
        assert_eq!(against.evidence_of_absence(), State::Unreachable);
        assert_eq!(against.counted(), 0);
    }

    /// The names are what a report carries, so they are asked for rather than
    /// assumed from the variant.
    #[test]
    fn every_state_has_its_own_declared_name() {
        let mut names: Vec<&str> = State::all().iter().map(|s| s.declared_name()).collect();
        assert_eq!(names, vec!["fresh", "late", "abandoned", "unreachable"]);
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), State::all().len());
    }
}
