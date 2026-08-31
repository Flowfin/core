//! The one policy every request to a server is retried under.
//!
//! `docs/decisions/0038-retry-and-backoff.md` decides all of it: three kinds
//! from 0004 are retried inside a call and no other kind is, a call gets at most
//! three attempts inside the same five seconds 0007 already abandons it after,
//! the wait before each retry is drawn uniformly at random from zero to a
//! doubling interval starting at 250 ms, an attempt is not begun with less than
//! half a second of that deadline left, and every one of those quantities is a
//! duration on the steady clock.
//!
//! # Why the policy is here before anything that sends a request
//!
//! 0038's own last section says a retry policy is the clearest case of a
//! decision that gets made by accident: it is never designed, it appears the
//! first time one call site has to cope with a flaky endpoint, and the loop
//! written that afternoon is copied to the next site because it is already
//! there. The values in it were typed rather than argued. This module is that
//! decision written where a call site meets it, so the first loop has a policy
//! to call rather than a policy to invent.
//!
//! # The seam the draw enters through
//!
//! 0038 fixes the spacing as a draw and says nothing about where the bytes come
//! from, and 0011 measured that the pinned toolchain offers no source. The
//! answer taken for this record is a seam of jitter's own, supplied by the
//! client, in the shape 0032 and 0036 already use for a value they cannot
//! generate either: [`Jitter`].
//!
//! It is jitter's own seam rather than the one
//! [`crate::session::device::LEAST_UNPREDICTABLE_BYTES`] governs, and the
//! difference is a promise rather than a mechanism. That width buys collision
//! resistance for a value tying a sign-in to its answer. A retry wait needs a
//! uniform draw over a range and needs no unpredictability at all, so tying it
//! to that seam would import a stronger promise than this needs and make the
//! weaker one look like a security guarantee. [`Jitter`] says in its own
//! documentation that it is not a security primitive, which is the sentence that
//! keeps the two apart.
//!
//! A generator seeded once inside the core was the alternative and it answers
//! nothing: the seed is the same question one layer down, and it would be a
//! hidden global the suite cannot move. A client-supplied seam gives a
//! deterministic sequence in a test and the platform's own generator in
//! production, which is what the third condition on #38 needs to be provable at
//! all.
//!
//! `docs/decisions/0045-the-recovery-schedule.md` applies the same spread and
//! [`super::recovery::WhileUnreachable::interval_the_wait_is_drawn_over`] names
//! the interval it is taken over, so both spreads draw through this one seam
//! rather than through two.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything 0038 decides that a kind, a counter and two
//! readings of one clock settle: what a failure does, how many attempts are
//! left, the interval the wait before each one is drawn over, when the server's
//! own hint replaces that draw, and whether there is enough deadline left to
//! begin an attempt at all.
//!
//! WHAT IS NOT HERE IS THE REQUEST. Nothing in this tree opens a connection, for
//! the reason [`super::transport`] gives about itself, so nothing here sends an
//! attempt, waits, or observes a second one succeed. This module holds the
//! policy such a loop would run under.
//!
//! WHAT IS ALSO NOT HERE IS THE ABANDONMENT COUNT. 0038 decides that a call
//! spending all three attempts is one abandonment rather than three, because
//! 0007 declares a server unreachable after two consecutive ones. Nothing in
//! this tree counts abandonments per server, so that decision is recorded in
//! [`WhyTheCallStopped::NoAttemptsLeft`]'s documentation and is enforced by
//! nothing: a loop that charged three would pass every check here.
//!
//! WHICH REQUESTS CHANGE SERVER STATE IS NOT DECIDED HERE EITHER, and 0038 says
//! so in as many words: it is a property of the surface 0010 records, and a list
//! in this module would be that list in the wrong place. [`WhatTheRequestDoes`]
//! is a parameter this module is told rather than something it works out.
//!
//! # Every number here is chosen and none is measured
//!
//! 0038 says so of its own, in the same words 0007 uses for the thresholds these
//! are fitted around. #65 is the harness that would replace a choice with a
//! number, and until it exists a reader should take each constant below as an
//! argument rather than as a measurement.

use core::time::Duration;

use super::transport::CallDeadline;
use crate::clock::SteadyInstant;
use crate::failure::{Failure, Kind};

/// The most attempts one call may spend.
///
/// From 0038. One is no retry at all. Two gives a transient failure one chance
/// to have passed. Three covers the case the second misses, which is a retry
/// that lands inside the same brief server hiccup that failed the first
/// attempt, since the first wait is short by design. A fourth would be paid for
/// out of the same five seconds, and that budget is more usefully left as room
/// for an attempt actually in flight.
///
/// A count is needed alongside the deadline rather than instead of it: a
/// connection refused in a few milliseconds costs almost none of the deadline,
/// so a deadline on its own would permit hundreds of attempts against a server
/// that is failing fast, which is where retrying is least useful and most
/// damaging.
pub const ATTEMPTS_AT_MOST: u32 = 3;

/// The interval the wait before the second attempt is drawn over.
///
/// From 0038. Short enough that the retry is still inside the window a person
/// perceives as one action, and long enough that a server which dropped one
/// request is not handed the replacement in the same instant. It doubles per
/// attempt from here, because a second failure is evidence the condition is
/// lasting rather than momentary.
///
/// It is named for the interval and not for a wait, so that a caller cannot read
/// it as one. What is waited is a draw over it, which is [`Jitter`].
pub const THE_FIRST_WAIT_IS_DRAWN_OVER: Duration = Duration::from_millis(250);

/// The deadline that has to be left before an attempt is begun.
///
/// From 0038. An attempt begun with less than this cannot plausibly complete,
/// and it still takes a connection out of the limit
/// [`super::transport::REQUESTS_OUTSTANDING_TO_ONE_SERVER`] holds, which is the
/// resource 0007 abandons requests to protect. So the core stops early rather
/// than starting something it has already decided not to wait for.
pub const AN_ATTEMPT_NEEDS_THIS_MUCH_DEADLINE_LEFT: Duration = Duration::from_millis(500);

/// The seam a retry wait is drawn through.
///
/// 0038 fixes the wait as a draw taken uniformly at random from zero to a
/// computed interval, per attempt and per caller, and 0011 measured that the
/// pinned toolchain offers no source of unpredictable bytes. So the draw is
/// supplied from outside, the way [`crate::clock::Clocks`] is, and for the same
/// reason: a wait a test cannot move is a retry test that takes seconds and
/// answers differently on a loaded machine.
///
/// THIS IS NOT A SECURITY PRIMITIVE AND MUST NOT BE READ AS ONE. What the draw
/// is for is thinning a burst: requests that failed together were issued
/// together, and a fixed wait moves the whole wall to a later instant without
/// thinning it. Nothing here depends on a value being hard to guess, so an
/// implementation owes a uniform spread and owes no unpredictability, and this
/// seam is deliberately not the one
/// [`crate::session::device::LEAST_UNPREDICTABLE_BYTES`] governs, where the
/// width is the whole point.
///
/// Thread safety, from 0009: safe from any thread. The core draws on both lanes,
/// and a source that were not would make every retry in the core a
/// synchronisation point.
///
/// What an implementation owes, beyond answering: a draw spread over the whole
/// interval rather than clustered inside it, and a fresh draw per call rather
/// than one value repeated. A source returning a constant satisfies the type and
/// defeats the record, and nothing here can tell the difference.
pub trait Jitter: Send + Sync {
    /// Draws a wait from zero to `interval`, uniformly.
    ///
    /// Zero is a legitimate answer and the range starts there deliberately.
    /// 0038 spreads over the full interval rather than over its upper half,
    /// because spreading over the full range is what actually thins a burst, and
    /// the objection that an individual retry can then go out almost
    /// immediately does not bite: [`ATTEMPTS_AT_MOST`] and the call deadline
    /// bound what any one caller can do however the draw falls.
    fn draw_over(&self, interval: Duration) -> Duration;
}

/// Whether repeating a request is free.
///
/// 0038 does not decide which calls change server state, and neither does this
/// module: it is a property of the surface 0010 records, and a list here would
/// drift against it. This is what a call site is told rather than what it works
/// out.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheRequestDoes {
    /// A request that only reads. Repeating it is free.
    OnlyReads,
    /// A request that changes something on the server. Repeating it is not free
    /// where the first attempt may already have been acted on.
    ChangesTheServer,
}

/// What a failed attempt does under this policy.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAFailureDoes {
    /// Retried inside the call, subject to the attempts and the deadline.
    ///
    /// The three kinds 0038 names: `timed-out`, `server-busy` and
    /// `server-failed`. Each is a condition that can be different a moment later
    /// without anything else changing, which is what 0004's Retry column means.
    RetriedInsideTheCall,
    /// Reported to the caller without a retry.
    ///
    /// Repeating any of these produces the same answer, and the retry is then a
    /// second identical failure charged to the caller's deadline.
    ReportedWithoutARetry,
    /// Not retried here. The server is absent and 0045's schedule owns what
    /// happens next.
    ///
    /// This is the one place 0038 and 0004's Retry column read differently, and
    /// it is separated from [`Self::ReportedWithoutARetry`] so that the
    /// difference survives. 0004 marks `server-unreachable` retryable after a
    /// delay; 0007 reports the server absent at once and attempts nothing
    /// further against it until the bounded recovery says otherwise. The delay
    /// 0004 refers to is that schedule, which is
    /// [`super::recovery::WhileUnreachable`], and there is no retry of the
    /// request inside the call.
    HandedToTheRecoverySchedule,
    /// Not retried here. A rejected session becomes a valid one by being
    /// renewed, and 0034's route owns that.
    ///
    /// Separated from [`Self::ReportedWithoutARetry`] because
    /// `not-authenticated` is the kind that looks retryable. A renewal followed
    /// by the original call is a different sequence with a different proof, not
    /// a retry of a failure, and it is
    /// [`crate::session::renewal::Renewals`]. Sending the same rejected token
    /// again is the thing 0038 refuses.
    HandedToTheRenewal,
    /// Outside this policy altogether.
    ///
    /// `storage-unavailable` is not a request to a server. What happens when a
    /// store the client supplied fails belongs with 0040 and 0033, and deciding
    /// it here would be deciding it in the wrong record.
    OutsideThisPolicy,
}

/// What a call does after an attempt failed.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheCallDoesNext {
    /// Wait, then begin another attempt.
    ///
    /// The wait is [`TheWait`] rather than a duration, because a drawn wait and
    /// a wait the server asked for are different things and a caller that
    /// collapsed them would apply the draw to a hint.
    WaitsThenAttemptsAgain(TheWait),
    /// Report the failure to the caller now.
    ReportsTheFailure(WhyTheCallStopped),
}

/// The wait before the next attempt.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheWait {
    /// Drawn over this interval, through [`Jitter`].
    ///
    /// The interval doubles per attempt from [`THE_FIRST_WAIT_IS_DRAWN_OVER`],
    /// and 0038 leaves the doubling without a ceiling deliberately: at three
    /// attempts the computed intervals are 250 ms and 500 ms and a ceiling would
    /// never be reached, so one now would be a number with no argument behind
    /// it. The moment [`ATTEMPTS_AT_MOST`] rises, a ceiling is owed and 0038 is
    /// superseded rather than edited.
    DrawnOver(Duration),
    /// The server said when to come back, and 0004 says the retry waits for the
    /// hint rather than for the computed value.
    ///
    /// A server refusing load knows more about when it will stop than any
    /// schedule here does, so the hint is used whole and no draw is applied to
    /// it. A hint that is absent, unreadable or not a duration is not a hint and
    /// never reaches this arm; the computed interval is used instead, which is
    /// the given-or-assumed distinction 0004 already carries.
    TheServerSaid(Duration),
}

/// Why a call stopped rather than attempting again.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyTheCallStopped {
    /// The kind is not one of the three that are retried inside a call.
    ///
    /// It carries what the failure does instead, so a caller that has to hand it
    /// to a renewal or to the recovery schedule is told which, rather than being
    /// sent back to re-read the kind.
    TheKindIsNotRetried(WhatAFailureDoes),
    /// Every attempt has been spent.
    ///
    /// THIS IS ONE ABANDONMENT AND NOT THREE. 0007 declares a server unreachable
    /// after two consecutive abandonments, and counting attempts there instead
    /// of calls would declare a healthy server absent on the strength of one
    /// slow endpoint on one call. Nothing in this tree counts abandonments per
    /// server, so that sentence is a decision recorded here and refused by
    /// nothing.
    NoAttemptsLeft,
    /// Less than [`AN_ATTEMPT_NEEDS_THIS_MUCH_DEADLINE_LEFT`] of the call
    /// deadline remains.
    TooLittleDeadlineLeft,
    /// The server's hint outlasts what is left of the deadline.
    ///
    /// The call ends rather than parking the caller past the point 0007 says an
    /// answer is not coming. 0004's payload keeps the hint intact, so whatever
    /// decides what to do next has the same information the core had.
    TheHintOutlastsTheDeadline,
    /// A `timed-out` whose bytes may already have been acted on, for a request
    /// that changes something on the server.
    ///
    /// 0004 carries whether anything reached the server precisely so this can be
    /// decided. Where nothing reached it the call certainly did not happen and
    /// the retry is free; where something may have, the core does not know
    /// whether the server acted, and repeating it belongs to whatever asked -
    /// which for an action taken while the server was gone is
    /// [`super::write_queue`].
    TheRequestMayAlreadyHaveBeenActedOn,
}

/// How many attempts a call has spent.
///
/// A counter rather than a flag, for the same reason
/// [`crate::session::renewal::Generation`] is: the question a retry loop has to
/// answer is how many attempts this call has already paid for, and a flag
/// answers a different one.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attempts {
    made: u32,
}

impl Attempts {
    /// A call that has not attempted anything yet.
    #[must_use]
    pub const fn none() -> Self {
        Self { made: 0 }
    }

    /// How many attempts this call has made.
    #[must_use]
    pub const fn made(self) -> u32 {
        self.made
    }

    /// The same call, one attempt later.
    ///
    /// It saturates rather than wrapping. A count that wrapped would hand a call
    /// a fresh set of attempts, which is the one arithmetic mistake here that
    /// produces more requests rather than fewer.
    #[must_use]
    pub const fn after_an_attempt(self) -> Self {
        Self {
            made: self.made.saturating_add(1),
        }
    }

    /// Whether another attempt is left under [`ATTEMPTS_AT_MOST`].
    #[must_use]
    pub const fn another_is_left(self) -> bool {
        self.made < ATTEMPTS_AT_MOST
    }

    /// The interval the wait before the next attempt is drawn over.
    ///
    /// [`THE_FIRST_WAIT_IS_DRAWN_OVER`] doubled once per attempt already made
    /// beyond the first: 250 ms before the second attempt, 500 ms before the
    /// third. It saturates rather than overflowing, so a counter that somehow
    /// ran past [`ATTEMPTS_AT_MOST`] produces a long interval rather than a
    /// short one.
    #[must_use]
    pub const fn interval_the_wait_is_drawn_over(self) -> Duration {
        let doublings = self.made.saturating_sub(1);
        let mut interval = THE_FIRST_WAIT_IS_DRAWN_OVER;
        let mut doubled = 0;
        while doubled < doublings {
            interval = match interval.checked_mul(2) {
                Some(longer) => longer,
                None => return Duration::MAX,
            };
            doubled += 1;
        }
        interval
    }
}

/// Applies the client's seam to an interval, and holds it to the interval.
///
/// 0038 says the wait is drawn from zero to the computed value, so a draw longer
/// than the interval is not a legal answer. Unlike a clock reading, this one is
/// cheap to hold: the value is compared against the interval it was asked for
/// and the smaller is used, so a seam that answers with a minute cannot spend a
/// caller's deadline waiting. Nothing reports that it happened, and a seam that
/// answers with a constant inside the interval is indistinguishable here from
/// one that draws.
#[must_use]
pub fn wait_drawn_over(jitter: &dyn Jitter, interval: Duration) -> Duration {
    let drawn = jitter.draw_over(interval);
    if drawn > interval { interval } else { drawn }
}

/// What this policy does with one failure, before any deadline is read.
///
/// The three kinds 0038 retries, the two it hands elsewhere, the one it does not
/// reach, and the `timed-out` that changed something on the server.
#[must_use]
pub fn what_a_failure_does(failure: &Failure, request: WhatTheRequestDoes) -> WhatAFailureDoes {
    match failure {
        Failure::TimedOut {
            bytes_reached_the_server,
            ..
        } => {
            if *bytes_reached_the_server && request == WhatTheRequestDoes::ChangesTheServer {
                WhatAFailureDoes::ReportedWithoutARetry
            } else {
                WhatAFailureDoes::RetriedInsideTheCall
            }
        }
        Failure::ServerBusy { .. } | Failure::ServerFailed { .. } => {
            WhatAFailureDoes::RetriedInsideTheCall
        }
        Failure::ServerUnreachable { .. } => WhatAFailureDoes::HandedToTheRecoverySchedule,
        Failure::NotAuthenticated { .. } => WhatAFailureDoes::HandedToTheRenewal,
        Failure::StorageUnavailable { .. } => WhatAFailureDoes::OutsideThisPolicy,
        Failure::AddressNotUsable { .. }
        | Failure::CertificateRejected { .. }
        | Failure::NotPermitted { .. }
        | Failure::NotFound { .. }
        | Failure::RequestRefused { .. }
        | Failure::AnswerNotUnderstood { .. }
        | Failure::CapabilityAbsent { .. }
        | Failure::Cancelled { .. }
        | Failure::InternalFault { .. } => WhatAFailureDoes::ReportedWithoutARetry,
    }
}

/// What the call does after this attempt failed.
///
/// The deadline is the call's rather than the attempt's, which is 0038's whole
/// answer to the overall bound #38 asks for: every attempt and every wait sits
/// inside the five seconds
/// [`super::transport::A_CALL_IS_ABANDONED_AFTER`] measures from the moment the
/// caller made the call, so a retry cannot extend a caller's wait.
///
/// `made` is the attempts spent INCLUDING the one that just failed, so a call
/// whose first attempt failed passes `Attempts::none().after_an_attempt()`.
#[must_use]
pub fn what_the_call_does_next(
    failure: &Failure,
    request: WhatTheRequestDoes,
    made: Attempts,
    deadline: CallDeadline,
    now: SteadyInstant,
) -> WhatTheCallDoesNext {
    let does = what_a_failure_does(failure, request);
    if does != WhatAFailureDoes::RetriedInsideTheCall {
        let why = if failure.kind() == Kind::TimedOut {
            WhyTheCallStopped::TheRequestMayAlreadyHaveBeenActedOn
        } else {
            WhyTheCallStopped::TheKindIsNotRetried(does)
        };
        return WhatTheCallDoesNext::ReportsTheFailure(why);
    }

    if !made.another_is_left() {
        return WhatTheCallDoesNext::ReportsTheFailure(WhyTheCallStopped::NoAttemptsLeft);
    }

    let left = deadline.left_at(now);
    if left < AN_ATTEMPT_NEEDS_THIS_MUCH_DEADLINE_LEFT {
        return WhatTheCallDoesNext::ReportsTheFailure(WhyTheCallStopped::TooLittleDeadlineLeft);
    }

    if let Failure::ServerBusy {
        retry_after: Some(hint),
        ..
    } = failure
    {
        if *hint > left {
            return WhatTheCallDoesNext::ReportsTheFailure(
                WhyTheCallStopped::TheHintOutlastsTheDeadline,
            );
        }
        return WhatTheCallDoesNext::WaitsThenAttemptsAgain(TheWait::TheServerSaid(*hint));
    }

    WhatTheCallDoesNext::WaitsThenAttemptsAgain(TheWait::DrawnOver(
        made.interval_the_wait_is_drawn_over(),
    ))
}

/// Whether an attempt may be begun at all.
///
/// Read after the wait rather than before it, because the wait is drawn and the
/// draw is not known when [`what_the_call_does_next`] answers. 0038 puts the
/// floor on beginning an attempt, so this is where it is applied.
#[must_use]
pub fn an_attempt_may_begin(deadline: CallDeadline, now: SteadyInstant) -> bool {
    deadline.left_at(now) >= AN_ATTEMPT_NEEDS_THIS_MUCH_DEADLINE_LEFT
}

#[cfg(test)]
mod tests {
    use super::{
        AN_ATTEMPT_NEEDS_THIS_MUCH_DEADLINE_LEFT, ATTEMPTS_AT_MOST, Attempts, Jitter,
        THE_FIRST_WAIT_IS_DRAWN_OVER, TheWait, WhatAFailureDoes, WhatTheCallDoesNext,
        WhatTheRequestDoes, WhyTheCallStopped, an_attempt_may_begin, wait_drawn_over,
        what_a_failure_does, what_the_call_does_next,
    };
    use crate::clock::SteadyInstant;
    use crate::failure::{
        Answered, Attempt, Capability, CertificateReason, Deadline, Expected, Failure, FaultSite,
        Kind, Operation, ReadingSite, Store, TransportOutcome,
    };
    use crate::server::address::BaseAddress;
    use crate::server::transport::{A_CALL_IS_ABANDONED_AFTER, CallDeadline};
    use core::sync::atomic::{AtomicU32, Ordering};
    use core::time::Duration;

    const BEGAN: SteadyInstant = SteadyInstant::from_nanos(0);

    fn at(millis: u64) -> SteadyInstant {
        SteadyInstant::from_nanos(millis * 1_000_000)
    }

    /// A draw a test can predict, which is the whole reason the seam is supplied
    /// from outside rather than seeded inside the core. It answers with a fixed
    /// fraction of the interval, so a doubling interval produces a doubling wait,
    /// and two of them with different fractions are two callers.
    ///
    /// The count is atomic rather than a cell because [`Jitter`] requires `Sync`,
    /// which is 0009's statement about the seam. A fixture that had to loosen
    /// that bound would be proving something else.
    struct AFixedFraction {
        numerator: u32,
        denominator: u32,
        draws: AtomicU32,
    }

    impl AFixedFraction {
        fn new(numerator: u32, denominator: u32) -> Self {
            Self {
                numerator,
                denominator,
                draws: AtomicU32::new(0),
            }
        }

        fn draws(&self) -> u32 {
            self.draws.load(Ordering::Relaxed)
        }
    }

    impl Jitter for AFixedFraction {
        fn draw_over(&self, interval: Duration) -> Duration {
            self.draws.fetch_add(1, Ordering::Relaxed);
            interval * self.numerator / self.denominator
        }
    }

    #[test]
    fn the_three_kinds_0038_names_are_the_only_ones_retried_inside_a_call() {
        for failure in [timed_out(false), server_busy(None), server_failed()] {
            assert_eq!(
                what_a_failure_does(&failure, WhatTheRequestDoes::OnlyReads),
                WhatAFailureDoes::RetriedInsideTheCall,
                "{} is one of the three 0038 retries",
                failure.kind().declared_name()
            );
        }
    }

    #[test]
    fn no_other_kind_is_retried_inside_a_call() {
        for failure in every_other_kind() {
            assert_ne!(
                what_a_failure_does(&failure, WhatTheRequestDoes::OnlyReads),
                WhatAFailureDoes::RetriedInsideTheCall,
                "{} is not one of the three 0038 retries",
                failure.kind().declared_name()
            );
        }
    }

    /// The two that look retryable and are handed on rather than reported, which
    /// is the distinction 0038 spends two paragraphs on.
    #[test]
    fn the_absent_server_and_the_rejected_session_are_handed_on_rather_than_reported() {
        assert_eq!(
            what_a_failure_does(&server_unreachable(), WhatTheRequestDoes::OnlyReads),
            WhatAFailureDoes::HandedToTheRecoverySchedule
        );
        assert_eq!(
            what_a_failure_does(&not_authenticated(), WhatTheRequestDoes::OnlyReads),
            WhatAFailureDoes::HandedToTheRenewal
        );
    }

    #[test]
    fn a_failing_store_is_outside_this_policy_rather_than_a_kind_it_refuses() {
        assert_eq!(
            what_a_failure_does(&storage_unavailable(), WhatTheRequestDoes::OnlyReads),
            WhatAFailureDoes::OutsideThisPolicy
        );
    }

    /// The condition #38 words as "a never-retried kind is attempted exactly
    /// once". The first failure of such a kind ends the call, so the attempt that
    /// produced it is the only one, and this walks every kind that is not one of
    /// the three rather than sampling one of them.
    #[test]
    fn a_never_retried_kind_ends_the_call_on_its_first_attempt() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        for failure in every_other_kind() {
            let next = what_the_call_does_next(
                &failure,
                WhatTheRequestDoes::OnlyReads,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            );
            assert!(
                matches!(next, WhatTheCallDoesNext::ReportsTheFailure(_)),
                "{} may not be attempted a second time",
                failure.kind().declared_name()
            );
        }
    }

    #[test]
    fn a_timeout_that_may_already_have_been_acted_on_is_not_repeated_where_it_changes_the_server() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        assert_eq!(
            what_the_call_does_next(
                &timed_out(true),
                WhatTheRequestDoes::ChangesTheServer,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::ReportsTheFailure(
                WhyTheCallStopped::TheRequestMayAlreadyHaveBeenActedOn
            )
        );
    }

    #[test]
    fn the_same_timeout_is_repeated_where_nothing_reached_the_server() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        assert_eq!(
            what_the_call_does_next(
                &timed_out(false),
                WhatTheRequestDoes::ChangesTheServer,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::WaitsThenAttemptsAgain(TheWait::DrawnOver(
                THE_FIRST_WAIT_IS_DRAWN_OVER
            ))
        );
    }

    #[test]
    fn a_read_is_repeated_after_a_timeout_that_reached_the_server() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        assert!(matches!(
            what_the_call_does_next(
                &timed_out(true),
                WhatTheRequestDoes::OnlyReads,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::WaitsThenAttemptsAgain(_)
        ));
    }

    #[test]
    fn the_interval_doubles_and_the_third_attempt_is_the_last() {
        let first = Attempts::none().after_an_attempt();
        let second = first.after_an_attempt();
        let third = second.after_an_attempt();
        assert_eq!(
            first.interval_the_wait_is_drawn_over(),
            Duration::from_millis(250)
        );
        assert_eq!(
            second.interval_the_wait_is_drawn_over(),
            Duration::from_millis(500)
        );
        assert!(first.another_is_left());
        assert!(second.another_is_left());
        assert!(!third.another_is_left());
        assert_eq!(third.made(), ATTEMPTS_AT_MOST);
    }

    /// The bound #38 asks a test to prove, in the direction arithmetic settles:
    /// three attempts and the two waits between them, each drawn at the top of
    /// its interval, spend three quarters of a second of a five second deadline.
    #[test]
    fn every_wait_this_policy_can_impose_fits_inside_the_call_deadline() {
        let first = Attempts::none().after_an_attempt();
        let second = first.after_an_attempt();
        let longest =
            first.interval_the_wait_is_drawn_over() + second.interval_the_wait_is_drawn_over();
        assert_eq!(longest, Duration::from_millis(750));
        assert!(longest < A_CALL_IS_ABANDONED_AFTER);
    }

    /// The other half of the same bound, and the half that actually holds it. The
    /// arithmetic above is true of the waits alone; what stops a caller waiting
    /// past 0007's five seconds is that the policy refuses to begin an attempt
    /// once the deadline is nearly spent, however many attempts are left.
    #[test]
    fn the_deadline_ends_a_call_that_still_has_attempts_left() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        let nearly_spent = at(4_600);
        assert!(deadline.left_at(nearly_spent) < AN_ATTEMPT_NEEDS_THIS_MUCH_DEADLINE_LEFT);
        assert_eq!(
            what_the_call_does_next(
                &server_failed(),
                WhatTheRequestDoes::OnlyReads,
                Attempts::none().after_an_attempt(),
                deadline,
                nearly_spent,
            ),
            WhatTheCallDoesNext::ReportsTheFailure(WhyTheCallStopped::TooLittleDeadlineLeft)
        );
        assert!(!an_attempt_may_begin(deadline, nearly_spent));
        assert!(an_attempt_may_begin(deadline, at(4_500)));
    }

    #[test]
    fn a_call_that_spent_every_attempt_reports_rather_than_attempting_a_fourth() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        let spent = Attempts::none()
            .after_an_attempt()
            .after_an_attempt()
            .after_an_attempt();
        assert_eq!(
            what_the_call_does_next(
                &server_failed(),
                WhatTheRequestDoes::OnlyReads,
                spent,
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::ReportsTheFailure(WhyTheCallStopped::NoAttemptsLeft)
        );
    }

    #[test]
    fn the_servers_own_hint_replaces_the_draw() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        assert_eq!(
            what_the_call_does_next(
                &server_busy(Some(Duration::from_millis(900))),
                WhatTheRequestDoes::OnlyReads,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::WaitsThenAttemptsAgain(TheWait::TheServerSaid(
                Duration::from_millis(900)
            ))
        );
    }

    #[test]
    fn a_hint_longer_than_what_is_left_ends_the_call_rather_than_parking_the_caller() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        assert_eq!(
            what_the_call_does_next(
                &server_busy(Some(Duration::from_secs(30))),
                WhatTheRequestDoes::OnlyReads,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::ReportsTheFailure(WhyTheCallStopped::TheHintOutlastsTheDeadline)
        );
    }

    #[test]
    fn a_server_that_gave_no_hint_falls_back_to_the_computed_interval() {
        let deadline = CallDeadline::beginning_at(BEGAN);
        assert_eq!(
            what_the_call_does_next(
                &server_busy(None),
                WhatTheRequestDoes::OnlyReads,
                Attempts::none().after_an_attempt(),
                deadline,
                at(1),
            ),
            WhatTheCallDoesNext::WaitsThenAttemptsAgain(TheWait::DrawnOver(
                THE_FIRST_WAIT_IS_DRAWN_OVER
            ))
        );
    }

    /// The property #38 asks a test to prove. Two callers that failed at the same
    /// instant read the same interval, and the draw is the only thing that
    /// separates them, which is why 0038 makes it per attempt per caller rather
    /// than one value for the wall.
    #[test]
    fn two_callers_failing_at_the_same_moment_wait_different_lengths() {
        let interval = Attempts::none()
            .after_an_attempt()
            .interval_the_wait_is_drawn_over();
        let one = AFixedFraction::new(1, 8);
        let other = AFixedFraction::new(7, 8);
        let waited_by_one = wait_drawn_over(&one, interval);
        let waited_by_the_other = wait_drawn_over(&other, interval);
        assert_ne!(waited_by_one, waited_by_the_other);
        assert!(waited_by_one <= interval);
        assert!(waited_by_the_other <= interval);
    }

    /// A draw per attempt rather than once per call, which is what keeps two
    /// callers apart before the third attempt as well as before the second.
    #[test]
    fn a_draw_is_taken_for_every_attempt_and_not_once_for_the_call() {
        let source = AFixedFraction::new(1, 2);
        let first = Attempts::none().after_an_attempt();
        let second = first.after_an_attempt();
        let before_the_second = wait_drawn_over(&source, first.interval_the_wait_is_drawn_over());
        let before_the_third = wait_drawn_over(&source, second.interval_the_wait_is_drawn_over());
        assert_eq!(source.draws(), 2);
        assert_eq!(before_the_second, Duration::from_millis(125));
        assert_eq!(before_the_third, Duration::from_millis(250));
    }

    /// Zero is a legitimate draw, and 0038 argues for a range that starts there
    /// rather than at half the interval.
    #[test]
    fn zero_is_a_draw_this_policy_admits() {
        let never_waits = AFixedFraction::new(0, 1);
        assert_eq!(
            wait_drawn_over(&never_waits, THE_FIRST_WAIT_IS_DRAWN_OVER),
            Duration::ZERO
        );
    }

    /// A seam is a client's code and can answer with anything. 0038 says the draw
    /// is from zero to the computed value, so a longer answer is held to the
    /// interval rather than spending a caller's deadline on it.
    #[test]
    fn a_seam_that_answers_past_the_interval_is_held_to_it() {
        let far_too_long = AFixedFraction::new(1_000, 1);
        assert_eq!(
            wait_drawn_over(&far_too_long, THE_FIRST_WAIT_IS_DRAWN_OVER),
            THE_FIRST_WAIT_IS_DRAWN_OVER
        );
    }

    // ----------------------------------------------------------------------
    // Fixtures. Every failure is built through the mapping point 0037 requires
    // rather than by naming a variant, so a test cannot construct a value the
    // core could not.
    // ----------------------------------------------------------------------

    fn attempt(bytes_reached_the_server: bool) -> Attempt<'static> {
        Attempt {
            address: "https://server.example",
            bytes_reached_the_server,
        }
    }

    fn answered(status: u16, retry_after: Option<Duration>) -> Failure {
        Failure::from_status(
            status,
            &Answered {
                capability: Capability::LibraryQuery,
                identifier: Some("an-item"),
                retry_after,
                server_code: None,
            },
        )
    }

    fn timed_out(bytes_reached_the_server: bool) -> Failure {
        let failure = Failure::from_transport(
            &TransportOutcome::DeadlineReached {
                deadline: Deadline::WholeRequest,
                elapsed: Duration::from_secs(5),
            },
            &attempt(bytes_reached_the_server),
        );
        assert_eq!(failure.kind(), Kind::TimedOut);
        failure
    }

    fn server_busy(retry_after: Option<Duration>) -> Failure {
        let failure = answered(429, retry_after);
        assert_eq!(failure.kind(), Kind::ServerBusy);
        failure
    }

    fn server_failed() -> Failure {
        let failure = answered(500, None);
        assert_eq!(failure.kind(), Kind::ServerFailed);
        failure
    }

    fn not_authenticated() -> Failure {
        let failure = answered(401, None);
        assert_eq!(failure.kind(), Kind::NotAuthenticated);
        failure
    }

    fn server_unreachable() -> Failure {
        let failure =
            Failure::from_transport(&TransportOutcome::ConnectionRefused, &attempt(false));
        assert_eq!(failure.kind(), Kind::ServerUnreachable);
        failure
    }

    fn storage_unavailable() -> Failure {
        Failure::storage_unavailable(Store::Cache, Operation::Read)
    }

    /// Every kind that is not one of the three 0038 retries, one value each.
    ///
    /// Written out rather than derived from the enum, because a sixteenth kind
    /// added to 0004 has to be placed here by somebody deciding what it does
    /// rather than defaulted into whichever arm a loop happened to take. The
    /// assertion at the end is what stops this list quietly holding one of the
    /// three it is the complement of.
    fn every_other_kind() -> Vec<Failure> {
        let kinds = vec![
            Failure::address_not_usable(
                &BaseAddress::parse("ftp://server.example")
                    .expect_err("a scheme 0028 refuses is what this fixture needs"),
            ),
            server_unreachable(),
            Failure::from_transport(
                &TransportOutcome::PeerNotTrusted {
                    reason: CertificateReason::SelfSigned,
                    fingerprint: "00:11",
                },
                &attempt(false),
            ),
            not_authenticated(),
            answered(403, None),
            answered(404, None),
            answered(400, None),
            Failure::answer_not_understood(
                ReadingSite::AnswerBody,
                Expected::ABodyTheCoreCanRead,
                0,
            ),
            Failure::capability_absent(Capability::TokenRenewal),
            Failure::cancelled(),
            Failure::internal_fault(FaultSite::SuccessMappedAsFailure),
            storage_unavailable(),
        ];
        for failure in &kinds {
            assert!(
                !matches!(
                    failure.kind(),
                    Kind::TimedOut | Kind::ServerBusy | Kind::ServerFailed
                ),
                "{} belongs in the retried set, not here",
                failure.kind().declared_name()
            );
        }
        kinds
    }
}
