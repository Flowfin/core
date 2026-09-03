//! A token that dies mid-playback: 0005's sequence, as what the queue and the
//! session do at each step of it.
//!
//! `docs/decisions/0005-the-session-model.md` fixes the sequence under its
//! mid-playback section and names #35 as where it is implemented. Three landed
//! records already hold the values it is made of: 0034 fixes what one renewal
//! is and what each of its outcomes does, which is [`super::renewal`]; 0047
//! holds the queue every report is on, which is [`crate::server::write_queue`];
//! and 0057 makes the report, which is [`crate::playback::report`]. This module
//! is the join of the three, and it decides nothing any of them decides.
//!
//! # The guarantee, and how a type holds it rather than a sentence
//!
//! 0005: playback already in flight is not interrupted. The stream is read by
//! the platform's player against the address #111 hands over, the core has no
//! way to stop it and would be wrong to use one, so a background report failing
//! reaches nothing a person is watching. A case cannot observe a stream the
//! core never holds, and the comment of 2026-08-11 on #35 says why a case that
//! asserted one would be a guard that cannot fail. What the core can be asked
//! about is its own output while the token dies, and each of the four things
//! 0005 promises about that is held by a type rather than by a rule somebody
//! remembers:
//!
//! - No cancellation is delivered and no outcome arrives on a call the client
//!   did not make. [`WhatARejectedReportDoes`] and
//!   [`WhatTheOutcomeDoesToPlayback`] carry no value of 0004's vocabulary, so
//!   there is nothing here that could fail a caller with anything.
//! - Nothing already in the queue is discarded, because an authentication
//!   failure says nothing about whether the positions are correct.
//!   [`a_report_was_rejected`] takes the queue by shared reference and cannot
//!   remove an entry from it. The rejected report was the head, it stays the
//!   head, and the drain stops there, which is 0047's own rule for an entry
//!   that could not be delivered.
//! - On success the queue drains in order and the current position is
//!   reported. The report goes through the queue like every other, where 0047's
//!   coalescing replaces the held one in place, so a renewal never sends one
//!   report and queues a second for the same item.
//! - On failure the session is signed out, the queue is kept, and the client
//!   is told once, through 0100, that this session can no longer report and
//!   that positions are being held. Not as an error on any call, because it
//!   made none.
//!
//! # What is here, and what is deliberately not
//!
//! WHAT IS NOT HERE IS THE DRAIN AND THE RENEWAL. Both are requests, the
//! rejection this module answers arrives from a server, and the transport is
//! #27 and is not built. Nothing here sends or receives a byte, nothing here
//! stops a drain because nothing runs one, and the queue this tree holds is not
//! durable, which `crate::server::write_queue` says of itself. So #35's
//! condition, a run through a token death against the fake server, has no
//! subject in this tree, and what is proven below is what the values answer at
//! each step of the sequence.
//!
//! WHAT IS ALSO NOT HERE IS THE SESSION. [`Renewals`] is the session's counter
//! and is handed in rather than held; the signed-out state is
//! [`LocalHalf`], which is answered rather than performed; and there is no
//! token in this tree to drop from memory.
//!
//! WHAT IS ALSO NOT HERE IS THE POSITION REACHED AFTER THE TOKEN DIED. 0005
//! guarantees the last position observed before the rejection and nothing
//! after it, bounded by 0057's cadence, and says a core that claimed the exact
//! stopping point would be claiming something it never observed. The current
//! position handed to [`the_renewal_ended`] is the client's reading at that
//! moment, which is what 0057 makes every position, and this module claims
//! nothing about the gap.

use crate::diagnostics::redaction::FieldName;
use crate::diagnostics::{Diagnostics, EventName, Field, FieldValue, Severity};
use crate::playback::AdmittedPosition;
use crate::playback::report::{PositionReport, Reporting};
use crate::server::write_queue::{WhatIsAsserted, WhatTheEnqueueDid, WriteQueue};
use crate::session::renewal::{
    HowTheRenewalEnded, Rejection, Renewals, WhatARejectedCallDoes, WhatTheOutcomeDoes,
};
use crate::session::sign_out::{LocalHalf, WhySigningOut};

/// The event 0005 owes the client at the moment a renewal fails during
/// playback: this session can no longer report, and positions are being held.
///
/// At `failure` rather than `notice`, for the reason 0105 gives about a dropped
/// queue entry: the thing that did not happen is somebody's own position
/// reaching the server, and a client reporting what was lost reads this level.
/// It is the one thing the client is told, and it is told through the sink
/// rather than on a call, because it made none.
const REPORTING_SUSPENDED: EventName = EventName::declared("session.reporting-suspended");

/// How many position reports the queue is holding for the session at that
/// moment.
///
/// Carried whole: it is a count, which 0071 lists among the values that cannot
/// differ between two people running the same build against the same server.
/// The item each position is about is not on the event at all, in either
/// treatment, because 0047 reports a queue's contents by correlator and this
/// event is about the session rather than about any one entry.
pub(crate) const POSITIONS_HELD: FieldName = FieldName::carried_whole("positions-held");

/// What a position report the server rejected does.
///
/// Five answers and never nothing, one per answer 0034 gives a rejected call,
/// read for a report rather than for a call a client made. Every one of them
/// holds the report where it is: the queue is not touched on this path, which
/// the signature of [`a_report_was_rejected`] is what holds.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatARejectedReportDoes {
    /// The report stays at the head, the drain stops there, and the session's
    /// one renewal starts. This is the case 0005's sequence is written for.
    HeldAndTheRenewalStarts,
    /// The report stays at the head and a renewal against this same token is
    /// already running. The drain waits for its outcome, and starts nothing.
    HeldWhileTheRenewalAlreadyRuns,
    /// The report went out under a token the session has since replaced. The
    /// drain delivers the head again, once, under the token the session holds
    /// now, which is the one retry 0034 allows a rejected call.
    DeliveredAgainUnderTheCurrentToken,
    /// The report was already the retry and was rejected in turn. It stays at
    /// the head, the drain stops, and no second renewal is started, for 0034's
    /// reason: a token issued seconds ago and immediately refused is not a
    /// token a third one would fix.
    HeldAndNothingStarts,
    /// The server offers no renewal route, so the first rejection ends the
    /// session. Every report is kept, the client is told through the sink, and
    /// this is the local half of the sign-out 0114 fixes.
    HeldAndTheSessionSignsOut(LocalHalf),
}

/// What a renewal's outcome does to playback.
///
/// Three answers, one per outcome 0034 fixes, each read for what it does to
/// the queue and to the one report 0005 says the success branch makes.
///
/// Thread safety, from 0009: a plain value, safe from any thread. It is not
/// `Copy` because the queue's answer may carry what it dropped, and that holds
/// the dropped entry's target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheOutcomeDoesToPlayback {
    /// A fresh token came back. The current position was reported through the
    /// queue, this is what the queue did with it, and the drain resumes from
    /// the head in order. Nothing is told to the client, because nothing
    /// happened that a person needs to know about.
    CurrentPositionReportedAndTheDrainResumes(WhatTheEnqueueDid),
    /// The server refused the renewal. Every report is kept, the session is
    /// signed out, and the client was told once through the sink. The core
    /// stops attempting to report and does not retry on a schedule, because
    /// every attempt would fail for the same reason; what restarts reporting
    /// is a person signing in again to the same account on the same server.
    HeldAndTheSessionSignsOut(LocalHalf),
    /// Nothing answered the renewal, or nothing was running. The session is
    /// exactly as it was, the token is not discarded, every report is kept,
    /// and nothing is told to anybody: the core learned nothing about the
    /// token, and the drain resumes when 0045 next reports the server
    /// reachable.
    HeldAndTheSessionLeftAsItWas,
}

/// The number of position reports the queue holds.
fn positions_held(queue: &WriteQueue<PositionReport>) -> u64 {
    let held = queue
        .entries()
        .iter()
        .filter(|entry| entry.asserted_about() == WhatIsAsserted::PlaybackPosition)
        .count();
    u64::try_from(held).unwrap_or(u64::MAX)
}

/// Tells the client, once, that this session can no longer report.
fn say_positions_are_held(queue: &WriteQueue<PositionReport>, diagnostics: &Diagnostics<'_>) {
    diagnostics.emit(
        Severity::Failure,
        REPORTING_SUSPENDED,
        &[Field::new(
            POSITIONS_HELD,
            FieldValue::Count(positions_held(queue)),
        )],
    );
}

/// The sign-out a renewal that produced no token forces.
///
/// Forced rather than asked for, so it is always the plain act and never a
/// forget, which is [`LocalHalf::completed`]'s own reading of
/// [`WhySigningOut::ARenewalWasRefused`]. A session with a renewal running or a
/// rejection arriving is a session that was signed in, so the caller's reading
/// of it is that it was not already signed out.
///
/// 0114 names one reason for a forced sign-out and 0034 names two cases that
/// force one, a renewal refused and a renewal that was not possible. Both take
/// the same reason here, because 0114's answer is the same for both and a
/// second variant would carry a distinction nothing downstream reads.
const fn forced_sign_out() -> LocalHalf {
    LocalHalf::completed(WhySigningOut::ARenewalWasRefused, false)
}

/// Answers a position report the server rejected with a token presented.
///
/// This is 0034's [`Renewals::rejected`] read for the head of the queue rather
/// than for a call a client made, and the difference is the whole of what this
/// function adds: a call fails with a kind from 0004 and a report does not,
/// because nobody is holding a call for it to fail. The report stays where it
/// is in every case, which the shared reference to the queue is what holds.
/// The drain stops at it, because 0047 stops at the first entry that could not
/// be delivered, and what happens next is the renewal's outcome arriving at
/// [`the_renewal_ended`].
///
/// The one case that reaches the client is a server with no renewal route,
/// where 0034 ends the session at the first rejection. That is 0005's failure
/// branch arriving without a renewal having been attempted, and it is told the
/// same way.
pub fn a_report_was_rejected(
    renewals: &mut Renewals,
    rejection: Rejection,
    queue: &WriteQueue<PositionReport>,
    diagnostics: &Diagnostics<'_>,
) -> WhatARejectedReportDoes {
    match renewals.rejected(rejection) {
        WhatARejectedCallDoes::StartTheRenewal => WhatARejectedReportDoes::HeldAndTheRenewalStarts,
        WhatARejectedCallDoes::WaitForTheRenewalAlreadyRunning => {
            WhatARejectedReportDoes::HeldWhileTheRenewalAlreadyRuns
        }
        WhatARejectedCallDoes::RetryAgainstTheCurrentToken => {
            WhatARejectedReportDoes::DeliveredAgainUnderTheCurrentToken
        }
        WhatARejectedCallDoes::FailAndStartNothing => WhatARejectedReportDoes::HeldAndNothingStarts,
        WhatARejectedCallDoes::SignTheSessionOut => {
            say_positions_are_held(queue, diagnostics);
            WhatARejectedReportDoes::HeldAndTheSessionSignsOut(forced_sign_out())
        }
    }
}

/// Applies the outcome of the renewal a rejection during playback started.
///
/// 0034's [`Renewals::ended`] decides what the outcome does to the session,
/// and this reads each answer for what it does to playback. A fresh token
/// makes the one report 0005's success branch owes, the position now, through
/// the queue: the entry the rejection left at the head is replaced in place,
/// and the drain then resumes from that head in order, so the position a
/// person reached during the renewal arrives at the server once. A refusal
/// keeps every report, signs the session out and tells the client once. A
/// silence keeps every report and moves nothing.
///
/// The current position is the client's reading at this moment, taken the way
/// 0057 takes every position, and the report leaves the cadence interval where
/// it was.
pub fn the_renewal_ended(
    renewals: &mut Renewals,
    how: HowTheRenewalEnded,
    reporting: &mut Reporting,
    current: AdmittedPosition,
    queue: &mut WriteQueue<PositionReport>,
    diagnostics: &Diagnostics<'_>,
) -> WhatTheOutcomeDoesToPlayback {
    match renewals.ended(how) {
        WhatTheOutcomeDoes::RetryTheWaitingCallsOnce => {
            let what_the_queue_did = reporting.report_after_a_renewal(current, queue);
            WhatTheOutcomeDoesToPlayback::CurrentPositionReportedAndTheDrainResumes(
                what_the_queue_did,
            )
        }
        WhatTheOutcomeDoes::SignTheSessionOut => {
            say_positions_are_held(queue, diagnostics);
            WhatTheOutcomeDoesToPlayback::HeldAndTheSessionSignsOut(forced_sign_out())
        }
        WhatTheOutcomeDoes::LeaveTheSessionExactlyAsItWas => {
            WhatTheOutcomeDoesToPlayback::HeldAndTheSessionLeftAsItWas
        }
    }
}

#[cfg(test)]
mod tests {
    //! 0005's mid-playback sequence, asked of the values at each step.
    //!
    //! What these cannot ask is #35's condition: a run through a token death
    //! against the fake server, with the stream, the drain and the renewal
    //! request all in flight. Nothing here sends a byte, and the queue is not
    //! durable.

    use std::sync::Mutex;

    use super::{
        POSITIONS_HELD, WhatARejectedReportDoes, WhatTheOutcomeDoesToPlayback,
        a_report_was_rejected, the_renewal_ended,
    };
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use crate::diagnostics::redaction::CorrelatorSalt;
    use crate::diagnostics::{Diagnostics, DiagnosticsSink, Event, FieldValue, Severity};
    use crate::playback::cadence::ReportsWithoutWaiting;
    use crate::playback::report::{PositionReport, ReportedOn, Reporting};
    use crate::playback::{AdmittedPosition, Ticks};
    use crate::server::write_queue::{Target, WhatTheEnqueueDid, WriteQueue};
    use crate::session::renewal::{HowTheRenewalEnded, Rejection, RenewalRoute, Renewals};
    use crate::session::sign_out::{LocalHalf, WhySigningOut};

    /// A clock source that does not move. Nothing here reads a clock for a
    /// decision; the facility needs one for the moment it stamps on an event.
    #[derive(Debug, Default)]
    struct Still;

    impl Clocks for Still {
        fn steady(&self) -> SteadyInstant {
            SteadyInstant::from_nanos(0)
        }

        fn elapsed(&self) -> ElapsedInstant {
            ElapsedInstant::from_nanos(0)
        }

        fn wall(&self) -> WallMoment {
            WallMoment::from_epoch(0, 0)
        }
    }

    /// Keeps every event's name and the count it carried under
    /// `positions-held`, so a case can say how many times the client was told
    /// and what it was told.
    #[derive(Debug, Default)]
    struct Collector {
        told: Mutex<Vec<(&'static str, Option<u64>)>>,
    }

    impl Collector {
        fn told(&self) -> Vec<(&'static str, Option<u64>)> {
            self.told
                .lock()
                .expect("the fixture holds no poisoned lock")
                .clone()
        }
    }

    impl DiagnosticsSink for Collector {
        fn event(&self, event: &Event<'_>) {
            let held = event
                .fields()
                .iter()
                .find(|field| field.name() == POSITIONS_HELD)
                .and_then(|field| match field.value() {
                    FieldValue::Count(count) => Some(count),
                    _ => None,
                });
            self.told
                .lock()
                .expect("the fixture holds no poisoned lock")
                .push((event.name().as_str(), held));
        }
    }

    fn a_salt() -> CorrelatorSalt {
        CorrelatorSalt::from_bytes([0x5a; CorrelatorSalt::WIDTH])
    }

    fn at(seconds: u64) -> ElapsedInstant {
        ElapsedInstant::from_nanos(seconds * 1_000_000_000)
    }

    fn played_to(seconds: i64) -> AdmittedPosition {
        AdmittedPosition::of(Ticks::from_seconds(seconds).as_ticks(), None)
    }

    fn item(identifier: &str) -> Target {
        Target::item(identifier.to_string())
    }

    /// What a queue holds, in order, as something a case can compare before
    /// and after: the order number, the item, and the position.
    fn contents(queue: &WriteQueue<PositionReport>) -> Vec<(u64, String, Ticks)> {
        queue
            .entries()
            .iter()
            .map(|entry| {
                (
                    entry.order(),
                    entry.target().as_str().to_string(),
                    entry.assertion().position(),
                )
            })
            .collect()
    }

    /// The queue 0005's sequence is written against: the film at the head,
    /// with the report the rejection is about, and an episode behind it. Two
    /// items, two positions, and the film's reporting handed back so the
    /// success branch can report through it.
    fn a_queue_with_the_film_at_the_head() -> (WriteQueue<PositionReport>, Reporting) {
        let mut queue = WriteQueue::empty();
        let mut film = Reporting::for_item(item("the-film"), at(0));
        assert_eq!(
            film.report(
                ReportsWithoutWaiting::Started,
                played_to(0),
                at(0),
                &mut queue
            ),
            WhatTheEnqueueDid::Added
        );
        assert_eq!(
            film.report(
                ReportsWithoutWaiting::Seeked,
                played_to(10),
                at(1),
                &mut queue
            ),
            WhatTheEnqueueDid::ReplacedInPlace
        );
        let mut episode = Reporting::for_item(item("the-episode"), at(2));
        assert_eq!(
            episode.report(
                ReportsWithoutWaiting::Started,
                played_to(0),
                at(2),
                &mut queue
            ),
            WhatTheEnqueueDid::Added
        );
        assert_eq!(queue.len(), 2);
        (queue, film)
    }

    fn rejected_under(renewals: &Renewals) -> Rejection {
        Rejection {
            went_out_under: renewals.generation(),
            is_the_retry: false,
        }
    }

    /// The head of the queue was rejected under the token the session holds.
    fn reject_the_head(
        renewals: &mut Renewals,
        queue: &WriteQueue<PositionReport>,
        diagnostics: &Diagnostics<'_>,
    ) -> WhatARejectedReportDoes {
        let rejection = rejected_under(renewals);
        a_report_was_rejected(renewals, rejection, queue, diagnostics)
    }

    /// 0005's first step. The report that was due when the rejection arrived
    /// is not lost and is not retried immediately: it is at the head, it stays
    /// at the head, nothing behind it moves, the renewal starts, and the
    /// client hears nothing.
    #[test]
    fn a_rejected_report_stays_at_the_head_and_the_renewal_starts() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (queue, _) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);

        let did = reject_the_head(&mut renewals, &queue, &diagnostics);

        assert_eq!(did, WhatARejectedReportDoes::HeldAndTheRenewalStarts);
        assert_eq!(contents(&queue), before);
        assert_eq!(queue.dropped(), 0);
        assert_eq!(renewals.running_against(), Some(renewals.generation()));
        assert!(collector.told().is_empty(), "the client was told something");
    }

    /// The nineteen of twenty, for a report: a second rejection under the same
    /// token joins the renewal already running and holds.
    #[test]
    fn a_second_rejection_under_the_same_token_holds_and_waits() {
        let clocks = Still;
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
        let (queue, _) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let first = reject_the_head(&mut renewals, &queue, &diagnostics);
        assert_eq!(first, WhatARejectedReportDoes::HeldAndTheRenewalStarts);

        let second = reject_the_head(&mut renewals, &queue, &diagnostics);

        assert_eq!(
            second,
            WhatARejectedReportDoes::HeldWhileTheRenewalAlreadyRuns
        );
        assert_eq!(contents(&queue), before);
    }

    /// A report that went out under a token the session has since replaced
    /// takes its one retry, and a retry rejected in turn holds and starts no
    /// second renewal.
    #[test]
    fn a_report_under_a_replaced_token_is_delivered_again_once_and_no_more() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (mut queue, mut film) = a_queue_with_the_film_at_the_head();
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let old = renewals.generation();
        reject_the_head(&mut renewals, &queue, &diagnostics);
        the_renewal_ended(
            &mut renewals,
            HowTheRenewalEnded::AFreshToken,
            &mut film,
            played_to(12),
            &mut queue,
            &diagnostics,
        );
        assert_ne!(renewals.generation(), old);
        let before = contents(&queue);

        let again = a_report_was_rejected(
            &mut renewals,
            Rejection {
                went_out_under: old,
                is_the_retry: false,
            },
            &queue,
            &diagnostics,
        );
        let the_retry = Rejection {
            went_out_under: renewals.generation(),
            is_the_retry: true,
        };
        let and_again = a_report_was_rejected(&mut renewals, the_retry, &queue, &diagnostics);

        assert_eq!(
            again,
            WhatARejectedReportDoes::DeliveredAgainUnderTheCurrentToken
        );
        assert_eq!(and_again, WhatARejectedReportDoes::HeldAndNothingStarts);
        assert_eq!(renewals.running_against(), None);
        assert_eq!(contents(&queue), before);
        assert!(collector.told().is_empty(), "the client was told something");
    }

    /// 0034 ends the session at the first rejection where the server offers no
    /// renewal route. That is 0005's failure branch without a renewal having
    /// been attempted: every report is kept, the sign-out is the forced one,
    /// and the client is told once, with how many positions are held.
    #[test]
    fn a_server_with_no_renewal_route_signs_out_at_the_first_rejection_and_holds_every_report() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (queue, _) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::NotOffered);

        let did = reject_the_head(&mut renewals, &queue, &diagnostics);

        assert_eq!(
            did,
            WhatARejectedReportDoes::HeldAndTheSessionSignsOut(LocalHalf::completed(
                WhySigningOut::ARenewalWasRefused,
                false
            ))
        );
        assert_eq!(contents(&queue), before);
        assert_eq!(
            collector.told(),
            vec![("session.reporting-suspended", Some(2))]
        );
    }

    /// 0005's success branch. The current position is reported through the
    /// queue, so 0047's coalescing replaces the entry the rejection left at the
    /// head rather than adding a second one for the same item; the head keeps
    /// its place in the order, so the drain resumes exactly where it stopped;
    /// and the client hears nothing, because nothing happened that a person
    /// needs to know about.
    #[test]
    fn a_fresh_token_reports_the_current_position_in_place_and_the_drain_resumes() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (mut queue, mut film) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let old = renewals.generation();
        reject_the_head(&mut renewals, &queue, &diagnostics);

        let did = the_renewal_ended(
            &mut renewals,
            HowTheRenewalEnded::AFreshToken,
            &mut film,
            played_to(25),
            &mut queue,
            &diagnostics,
        );

        assert_eq!(
            did,
            WhatTheOutcomeDoesToPlayback::CurrentPositionReportedAndTheDrainResumes(
                WhatTheEnqueueDid::ReplacedInPlace
            )
        );
        let after = contents(&queue);
        assert_eq!(after.len(), before.len());
        assert_eq!(
            after[0].0, before[0].0,
            "the head lost its place in the order"
        );
        assert_eq!(after[0].1, "the-film");
        assert_eq!(after[0].2, Ticks::from_seconds(25));
        assert_eq!(after[1], before[1]);
        let head = queue.next_to_deliver().expect("the queue holds the film");
        assert_eq!(head.assertion().reported_on(), ReportedOn::AfterARenewal);
        assert_ne!(renewals.generation(), old);
        assert_eq!(renewals.running_against(), None);
        assert!(collector.told().is_empty(), "the client was told something");
    }

    /// 0005's failure branch. The queue is kept rather than dropped, because it
    /// belongs to the session and a person who signs in again gets those
    /// positions reported in order before anything else; the session is signed
    /// out by the forced act; and the client is told exactly once.
    #[test]
    fn a_refused_renewal_keeps_every_report_and_says_so_once() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (mut queue, mut film) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        reject_the_head(&mut renewals, &queue, &diagnostics);

        let did = the_renewal_ended(
            &mut renewals,
            HowTheRenewalEnded::TheServerRefusedIt,
            &mut film,
            played_to(25),
            &mut queue,
            &diagnostics,
        );

        assert_eq!(
            did,
            WhatTheOutcomeDoesToPlayback::HeldAndTheSessionSignsOut(LocalHalf::completed(
                WhySigningOut::ARenewalWasRefused,
                false
            ))
        );
        assert_eq!(contents(&queue), before);
        assert_eq!(queue.dropped(), 0);
        assert_eq!(renewals.running_against(), None);
        assert_eq!(
            collector.told(),
            vec![("session.reporting-suspended", Some(2))]
        );
    }

    /// 0034's silence. The core learned nothing about the token, so the
    /// session is exactly as it was, the generation has not moved, every
    /// report is kept, and the client is told nothing: a session emptied
    /// because a connection dropped is the failure that record names.
    #[test]
    fn a_silent_renewal_leaves_the_session_and_the_queue_exactly_as_they_were() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (mut queue, mut film) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        reject_the_head(&mut renewals, &queue, &diagnostics);
        let generation = renewals.generation();

        let did = the_renewal_ended(
            &mut renewals,
            HowTheRenewalEnded::NothingAnswered,
            &mut film,
            played_to(25),
            &mut queue,
            &diagnostics,
        );

        assert_eq!(
            did,
            WhatTheOutcomeDoesToPlayback::HeldAndTheSessionLeftAsItWas
        );
        assert_eq!(contents(&queue), before);
        assert_eq!(renewals.generation(), generation);
        assert_eq!(renewals.running_against(), None);
        assert!(collector.told().is_empty(), "the client was told something");
    }

    /// An outcome arriving when no renewal was running is 0034's safe reading
    /// of a case it does not describe: nothing moves, nothing is reported, and
    /// no position is put on the queue for a renewal that did not happen.
    #[test]
    fn an_outcome_when_nothing_was_running_changes_nothing() {
        let clocks = Still;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let (mut queue, mut film) = a_queue_with_the_film_at_the_head();
        let before = contents(&queue);
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);

        let did = the_renewal_ended(
            &mut renewals,
            HowTheRenewalEnded::AFreshToken,
            &mut film,
            played_to(25),
            &mut queue,
            &diagnostics,
        );

        assert_eq!(
            did,
            WhatTheOutcomeDoesToPlayback::HeldAndTheSessionLeftAsItWas
        );
        assert_eq!(contents(&queue), before);
        assert!(collector.told().is_empty(), "the client was told something");
    }
}
