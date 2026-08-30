//! The generation a rejection is answered against, and when a renewal is due.
//!
//! `docs/decisions/0034-renewal-and-the-token-generation.md` is the record and
//! #34 is the issue. The record decides how many renewals happen when twenty
//! calls are rejected in the same instant, what happens to each of those calls,
//! what a renewal that could not be attempted at all does to the session, and
//! when a renewal ahead of a rejection is scheduled.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that a counter and one clock reading settle:
//! which rejection starts a renewal and which joins the one already running,
//! which call is retried and which fails without renewing, what each of the
//! three renewal outcomes does to the session, and the moment a scheduled
//! renewal becomes due. Each is decided many times in a session's life, each is
//! provable in microseconds against the controlled clock 0102 requires, and each
//! is wrong in a way that looks like something else.
//!
//! WHAT IS NOT HERE IS THE RENEWAL. Sending one is a request, the rejection it
//! answers arrives from a server, and 0038's attempts happen underneath it. All
//! of that is the transport, which is #27 and is not built, so nothing in this
//! module sends or receives a byte. Neither does it hold a session: what it
//! answers with is what the session then does, so signing out lives with #114
//! and 0114 rather than being performed here.
//!
//! # The one thing this module refuses to be able to say
//!
//! 0102 fixes that a token is never refused on the device's own reading of a
//! clock and that a stated expiry is a hint used only for scheduling. So
//! [`RenewalSchedule`] answers when a renewal is DUE and there is deliberately
//! no operation here that answers whether a token has expired. A token whose
//! stated expiry passed an hour ago is still sent, and the server still decides.

use core::time::Duration;

use crate::clock::ElapsedInstant;

/// How long before a stated expiry a renewal is scheduled.
///
/// Five minutes, from 0034, and it is chosen rather than measured. The reason it
/// is this number is that it is longer than any single call the core makes,
/// which 0007 bounds at five seconds, so a renewal that fires on schedule is not
/// competing with a call a person is waiting on.
pub const A_RENEWAL_IS_DUE_THIS_LONG_BEFORE_THE_STATED_EXPIRY: Duration = Duration::from_mins(5);

/// Which token a call went out under.
///
/// 0034: it starts at one when the session is acquired and increases by one
/// every time a token replaces another, and every request records the generation
/// of the token it went out under. A rejection is then answered against the
/// generation it names rather than against a flag saying a renewal is running,
/// because the question that matters is whether the token this call was rejected
/// under has already been replaced, and a flag answers a different one.
///
/// # Where it stops counting
///
/// The count saturates at [`u64::MAX`] rather than wrapping, and saturation
/// would make two generations compare equal, which is the failure this type
/// exists against. It is stated rather than guarded: reaching it needs one
/// renewal per nanosecond for longer than the age of the universe, and a guard
/// against it would be a branch no test could reach honestly.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Generation {
    number: u64,
}

impl Generation {
    /// The generation a session is acquired at, which 0034 fixes as one.
    #[must_use]
    pub const fn first() -> Self {
        Self { number: 1 }
    }

    /// The generation after this one, which a token replacing another takes.
    #[must_use]
    pub const fn next(self) -> Self {
        Self {
            number: self.number.saturating_add(1),
        }
    }

    /// The number itself, for a caller that records it against a request.
    #[must_use]
    pub const fn number(self) -> u64 {
        self.number
    }
}

/// Whether the server offers a way to exchange a live token for a fresh one.
///
/// 0034 is written for both of #10's answers rather than waiting for one, and
/// 0034 says where this comes from: the capability answers 0005 already holds on
/// the session, so it is one answer per server rather than a discovery at every
/// rejection. Nothing in this module asks a server anything.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenewalRoute {
    /// The server states a route, so everything 0034 decides is what happens.
    Offered,
    /// The server states none. The renewal attempt is not made and the first
    /// rejection ends the session, which 0034 fixes rather than leaving to the
    /// caller.
    NotOffered,
}

/// A call the server rejected with a token presented.
///
/// 0034 says which condition starts a renewal and which two do not, and the
/// distinction is carried by 0004's kind and payload rather than by a status
/// code: a wrong password is the same kind with no token presented, and a server
/// that did not answer is a different kind entirely. So this type is only ever
/// built for the one condition, and nothing in this module reads a status.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rejection {
    /// The generation of the token the rejected call went out under.
    pub went_out_under: Generation,
    /// Whether this call is the retry 0034 allows a rejected call.
    ///
    /// A retry rejected in turn starts nothing further. A token issued seconds
    /// ago and immediately refused is not a token a third one would fix, and the
    /// loop that refuses is the shape that gets a device's address blocked at an
    /// authentication endpoint.
    pub is_the_retry: bool,
}

/// What a rejected call does, which is one of five things and never nothing.
///
/// 0034 requires every call outstanding when the token died to end in an
/// outcome, and says plainly that nothing in this path returns a result with no
/// items in it. This enumeration is that requirement written as the answers a
/// caller can get.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatARejectedCallDoes {
    /// Start the one renewal this session may have, and wait for its outcome.
    StartTheRenewal,
    /// A renewal against this same token is already running. Wait for it.
    ///
    /// This is the nineteen of twenty. They are not a second renewal and they
    /// are not a failure yet.
    WaitForTheRenewalAlreadyRunning,
    /// The token this call went out under has already been replaced.
    ///
    /// No renewal is started, because the thing that would be renewed is gone.
    /// The call takes the one retry 0034 allows it, against the token the
    /// session holds now.
    RetryAgainstTheCurrentToken,
    /// This call was already the retry. It fails, and starts nothing.
    FailAndStartNothing,
    /// The server offers no renewal route, so the session ends here.
    ///
    /// 0034 refuses the alternative by name: re-running a sign-in route on a
    /// person's behalf needs a password, a second device or a browser, and doing
    /// any of it unasked is a sign-in the person did not make.
    SignTheSessionOut,
}

/// How a renewal that was attempted ended.
///
/// The two failing members are the split 0034 says is decided wrongly by reflex,
/// because both are a renewal that produced no token.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowTheRenewalEnded {
    /// A fresh token came back.
    AFreshToken,
    /// The server answered, and the answer was that this session is over.
    TheServerRefusedIt,
    /// Nothing answered, or it timed out.
    ///
    /// The core learned nothing about the token. 0034 states what getting this
    /// wrong in the other direction costs: not the failed call, but a token that
    /// is gone, so the recovery is a sign-in on an on-screen keyboard rather
    /// than a retry.
    NothingAnswered,
}

/// What a renewal's outcome does to the session and to the calls waiting on it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheOutcomeDoes {
    /// The generation has moved. Every waiting call retries once, against the
    /// new token, and its caller is not told a renewal happened.
    RetryTheWaitingCallsOnce,
    /// The session is signed out and the token discarded. Waiting calls fail
    /// with `not-authenticated`, which 0005's sequence is what performs.
    SignTheSessionOut,
    /// Nothing about the session moved. The token is not discarded, the
    /// generation does not move, and waiting calls fail with the transport's own
    /// kind rather than with `not-authenticated`.
    LeaveTheSessionExactlyAsItWas,
}

/// One session's renewals: the generation it holds and whether one is running.
///
/// # This is per session and never ambient
///
/// 0034 puts the counter on the session, and two sessions renewing at once are
/// two renewals. They are different servers, different accounts or different
/// devices, and 0005 refuses an ambient current session precisely so that one
/// cannot be answered with the other's token. So this type holds one session's
/// state and there is no register of them here.
///
/// Thread safety, from 0009: a plain value, safe from any thread. Who may mutate
/// one is 0009's rule for the core's own state; nothing here takes a lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Renewals {
    generation: Generation,
    route: RenewalRoute,
    running_against: Option<Generation>,
}

impl Renewals {
    /// The state a session is acquired in: generation one, nothing running.
    #[must_use]
    pub const fn acquired(route: RenewalRoute) -> Self {
        Self {
            generation: Generation::first(),
            route,
            running_against: None,
        }
    }

    /// The generation of the token this session holds now.
    #[must_use]
    pub const fn generation(self) -> Generation {
        self.generation
    }

    /// Whether a renewal is running, and against which token.
    ///
    /// The generation rather than a boolean, because what a caller needs to know
    /// about a running renewal is which token it is replacing.
    #[must_use]
    pub const fn running_against(self) -> Option<Generation> {
        self.running_against
    }

    /// Answers a rejection, and starts at most one renewal for this session.
    ///
    /// The order of the readings is 0034's and it is not interchangeable. A
    /// server with no renewal route ends the session at the first rejection
    /// whatever the generation says. A retry that was rejected in turn fails
    /// without renewing. A rejection naming a generation the session no longer
    /// holds takes its retry. Only a rejection naming the current generation can
    /// start one, and only where none is already running against it.
    pub fn rejected(&mut self, rejection: Rejection) -> WhatARejectedCallDoes {
        if self.route == RenewalRoute::NotOffered {
            return WhatARejectedCallDoes::SignTheSessionOut;
        }
        if rejection.is_the_retry {
            return WhatARejectedCallDoes::FailAndStartNothing;
        }
        if rejection.went_out_under != self.generation {
            return WhatARejectedCallDoes::RetryAgainstTheCurrentToken;
        }
        if self.running_against == Some(self.generation) {
            return WhatARejectedCallDoes::WaitForTheRenewalAlreadyRunning;
        }
        self.running_against = Some(self.generation);
        WhatARejectedCallDoes::StartTheRenewal
    }

    /// Applies the outcome of the renewal that was running.
    ///
    /// A fresh token moves the generation on, which is what makes every
    /// rejection that arrives afterwards naming the old one take its retry
    /// instead of starting a second renewal. A refusal ends the session. A
    /// silence moves nothing at all, and that is the half 0034 says the natural
    /// code gets wrong: the session stays exactly as it was, so the next call
    /// tries the same token again, because it may well still be valid and the
    /// only thing that was wrong was the network.
    ///
    /// Applying an outcome when nothing was running leaves the session alone and
    /// answers as a silence does. That is not a case 0034 describes; it is the
    /// safe reading of one, because the alternative would let a stray outcome
    /// discard a token no renewal was made against.
    pub fn ended(&mut self, how: HowTheRenewalEnded) -> WhatTheOutcomeDoes {
        if self.running_against.is_none() {
            return WhatTheOutcomeDoes::LeaveTheSessionExactlyAsItWas;
        }
        match how {
            HowTheRenewalEnded::AFreshToken => {
                self.generation = self.generation.next();
                self.running_against = None;
                WhatTheOutcomeDoes::RetryTheWaitingCallsOnce
            }
            HowTheRenewalEnded::TheServerRefusedIt => {
                self.running_against = None;
                WhatTheOutcomeDoes::SignTheSessionOut
            }
            HowTheRenewalEnded::NothingAnswered => {
                self.running_against = None;
                WhatTheOutcomeDoes::LeaveTheSessionExactlyAsItWas
            }
        }
    }
}

/// When a renewal ahead of a rejection is due.
///
/// 0034: the later of the stated expiry less five minutes and the halfway point
/// of the token's stated lifetime. The halfway rule exists for the
/// short-lifetime case: a token stated to last two minutes would otherwise be
/// scheduled for renewal three minutes before it was issued, and a schedule in
/// the past is a renewal on every call.
///
/// # It is an interval on the elapsed clock
///
/// 0102 puts it there, so a device that was asleep past the moment owes ONE
/// renewal when it wakes. What holds that here is the shape rather than a
/// sentence: [`RenewalSchedule::is_due`] answers yes or no, and there is no
/// operation returning how many moments passed, so there is nothing for a caller
/// to multiply.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenewalSchedule {
    issued: ElapsedInstant,
    stated_lifetime: Duration,
}

impl RenewalSchedule {
    /// The schedule for a token issued at a moment with a stated lifetime.
    ///
    /// The lifetime is what the server said, and 0034 and 0102 both treat it as
    /// a hint for scheduling. Nothing here refuses anything on it.
    #[must_use]
    pub const fn stated(issued: ElapsedInstant, stated_lifetime: Duration) -> Self {
        Self {
            issued,
            stated_lifetime,
        }
    }

    /// How long after the token was issued the renewal is due.
    ///
    /// The five-minutes-before-expiry offset is floored at zero rather than
    /// running backwards, and the halfway point is what wins whenever it does.
    /// The two are equal at a stated lifetime of ten minutes, which is the
    /// boundary between the two rules.
    #[must_use]
    pub fn after_the_token_was_issued(self) -> Duration {
        let before_the_expiry = self
            .stated_lifetime
            .saturating_sub(A_RENEWAL_IS_DUE_THIS_LONG_BEFORE_THE_STATED_EXPIRY);
        let halfway = self.stated_lifetime / 2;
        if before_the_expiry > halfway {
            before_the_expiry
        } else {
            halfway
        }
    }

    /// Whether the renewal is due at this reading of the elapsed clock.
    ///
    /// A device that slept through the moment reads yes once, the same as a
    /// device that was awake for it, which is 0102's rule about this clock
    /// arriving as an answer rather than as a sentence.
    ///
    /// THIS IS NOT AN EXPIRY QUESTION AND CANNOT BE USED AS ONE. It says a
    /// renewal is worth attempting. 0102 fixes that a token is never refused on
    /// the device's own clock, and there is no operation in this module that
    /// answers whether a token is still good.
    #[must_use]
    pub fn is_due(self, now: ElapsedInstant) -> bool {
        now.interval_since(self.issued) >= self.after_the_token_was_issued()
    }
}

#[cfg(test)]
mod tests {
    //! 0034's counter and its schedule, asked of the values.
    //!
    //! What these cannot ask is #34's own condition. It drives the fake server to
    //! reject a token mid-run with many calls failing at once, and nothing the
    //! core runs starts a thread for a second call to be in flight on.

    use super::{
        A_RENEWAL_IS_DUE_THIS_LONG_BEFORE_THE_STATED_EXPIRY, Generation, HowTheRenewalEnded,
        Rejection, RenewalRoute, RenewalSchedule, Renewals, WhatARejectedCallDoes,
        WhatTheOutcomeDoes,
    };
    use crate::clock::ElapsedInstant;
    use core::time::Duration;

    const NANOS_IN_A_SECOND: u64 = 1_000_000_000;

    fn at(seconds: u64) -> ElapsedInstant {
        ElapsedInstant::from_nanos(seconds * NANOS_IN_A_SECOND)
    }

    fn rejected_under(generation: Generation) -> Rejection {
        Rejection {
            went_out_under: generation,
            is_the_retry: false,
        }
    }

    /// The property #34 asks for, at the number 0034 uses to state it. Twenty
    /// rejections naming one token produce one renewal and nineteen waits.
    #[test]
    fn twenty_calls_rejected_under_one_token_start_exactly_one_renewal() {
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let held = renewals.generation();

        let mut started = 0;
        let mut waited = 0;
        for _ in 0..20 {
            match renewals.rejected(rejected_under(held)) {
                WhatARejectedCallDoes::StartTheRenewal => started += 1,
                WhatARejectedCallDoes::WaitForTheRenewalAlreadyRunning => waited += 1,
                other => panic!("a rejection under the held token answered {other:?}"),
            }
        }

        assert_eq!(started, 1);
        assert_eq!(waited, 19);
        assert_eq!(renewals.running_against(), Some(held));
    }

    /// The reason it is a counter and not a flag, which is the case a flag gets
    /// wrong: a rejection that arrives AFTER the renewal has already finished.
    /// A flag is clear by then and would start a second renewal.
    #[test]
    fn a_rejection_arriving_after_the_renewal_finished_starts_no_second_one() {
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let old = renewals.generation();

        assert_eq!(
            renewals.rejected(rejected_under(old)),
            WhatARejectedCallDoes::StartTheRenewal
        );
        assert_eq!(
            renewals.ended(HowTheRenewalEnded::AFreshToken),
            WhatTheOutcomeDoes::RetryTheWaitingCallsOnce
        );
        assert_ne!(renewals.generation(), old);
        assert_eq!(renewals.running_against(), None);

        assert_eq!(
            renewals.rejected(rejected_under(old)),
            WhatARejectedCallDoes::RetryAgainstTheCurrentToken
        );
        assert_eq!(renewals.running_against(), None);
    }

    /// A rejection naming the token the session now holds starts a renewal
    /// again, which is what stops the rule above turning into a session that can
    /// never renew twice.
    #[test]
    fn a_rejection_under_the_new_token_may_start_the_next_renewal() {
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        renewals.rejected(rejected_under(renewals.generation()));
        renewals.ended(HowTheRenewalEnded::AFreshToken);

        let now_held = renewals.generation();
        assert_eq!(now_held.number(), 2);
        assert_eq!(
            renewals.rejected(rejected_under(now_held)),
            WhatARejectedCallDoes::StartTheRenewal
        );
    }

    /// The retry gets one attempt and no more. A token issued seconds ago and
    /// immediately refused is not one a third would fix.
    #[test]
    fn a_rejected_retry_fails_and_starts_nothing() {
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let held = renewals.generation();

        let retry = Rejection {
            went_out_under: held,
            is_the_retry: true,
        };

        assert_eq!(
            renewals.rejected(retry),
            WhatARejectedCallDoes::FailAndStartNothing
        );
        assert_eq!(renewals.running_against(), None);
    }

    /// A server offering no renewal route ends the session at the first
    /// rejection, and the generation still exists underneath it.
    #[test]
    fn a_server_with_no_renewal_route_signs_out_at_the_first_rejection() {
        let mut renewals = Renewals::acquired(RenewalRoute::NotOffered);
        let held = renewals.generation();

        assert_eq!(
            renewals.rejected(rejected_under(held)),
            WhatARejectedCallDoes::SignTheSessionOut
        );
        assert_eq!(renewals.running_against(), None);
        assert_eq!(renewals.generation(), held);
    }

    /// The split 0034 says is decided wrongly by reflex. A refusal ends the
    /// session; a silence moves nothing, and the generation stays where it was
    /// so the same token is tried again.
    #[test]
    fn a_refusal_ends_the_session_and_a_silence_changes_nothing() {
        let mut refused = Renewals::acquired(RenewalRoute::Offered);
        let held = refused.generation();
        refused.rejected(rejected_under(held));
        assert_eq!(
            refused.ended(HowTheRenewalEnded::TheServerRefusedIt),
            WhatTheOutcomeDoes::SignTheSessionOut
        );
        assert_eq!(refused.generation(), held);

        let mut silent = Renewals::acquired(RenewalRoute::Offered);
        silent.rejected(rejected_under(held));
        assert_eq!(
            silent.ended(HowTheRenewalEnded::NothingAnswered),
            WhatTheOutcomeDoes::LeaveTheSessionExactlyAsItWas
        );
        assert_eq!(silent.generation(), held);
        assert_eq!(
            silent.rejected(rejected_under(held)),
            WhatARejectedCallDoes::StartTheRenewal
        );
    }

    /// An outcome for a renewal nobody started leaves the session alone rather
    /// than discarding a token no renewal was made against.
    #[test]
    fn an_outcome_with_nothing_running_moves_nothing() {
        let mut renewals = Renewals::acquired(RenewalRoute::Offered);
        let held = renewals.generation();

        assert_eq!(
            renewals.ended(HowTheRenewalEnded::TheServerRefusedIt),
            WhatTheOutcomeDoes::LeaveTheSessionExactlyAsItWas
        );
        assert_eq!(renewals.generation(), held);
        assert_eq!(
            renewals.ended(HowTheRenewalEnded::AFreshToken),
            WhatTheOutcomeDoes::LeaveTheSessionExactlyAsItWas
        );
        assert_eq!(renewals.generation(), held);
    }

    /// The generation starts at one and counts by one.
    #[test]
    fn the_generation_starts_at_one_and_counts_by_one() {
        assert_eq!(Generation::first().number(), 1);
        assert_eq!(Generation::first().next().number(), 2);
        assert_eq!(Generation::first().next().next().number(), 3);
    }

    /// The long-lifetime rule: five minutes before the stated expiry.
    #[test]
    fn a_long_lived_token_is_renewed_five_minutes_before_its_stated_expiry() {
        let hour = RenewalSchedule::stated(at(0), Duration::from_secs(3600));

        assert_eq!(hour.after_the_token_was_issued(), Duration::from_mins(55));
        assert!(!hour.is_due(at(3299)));
        assert!(hour.is_due(at(3300)));
        assert_eq!(
            A_RENEWAL_IS_DUE_THIS_LONG_BEFORE_THE_STATED_EXPIRY.as_secs(),
            300
        );
    }

    /// The short-lifetime rule, and the case 0034 names: a two-minute token
    /// would otherwise be scheduled three minutes before it was issued, which is
    /// a renewal on every call.
    #[test]
    fn a_two_minute_token_is_renewed_at_its_halfway_point_and_never_in_the_past() {
        let two_minutes = RenewalSchedule::stated(at(0), Duration::from_secs(120));

        assert_eq!(
            two_minutes.after_the_token_was_issued(),
            Duration::from_secs(60)
        );
        assert!(!two_minutes.is_due(at(59)));
        assert!(two_minutes.is_due(at(60)));
    }

    /// The boundary between the two rules, at the lifetime where they agree, and
    /// one second either side of it so the choice is visible.
    #[test]
    fn the_two_rules_meet_at_a_ten_minute_lifetime() {
        let ten = RenewalSchedule::stated(at(0), Duration::from_secs(600));
        assert_eq!(ten.after_the_token_was_issued(), Duration::from_secs(300));

        let under = RenewalSchedule::stated(at(0), Duration::from_secs(599));
        assert_eq!(
            under.after_the_token_was_issued(),
            Duration::from_millis(299_500),
            "the halfway rule is what holds below the meeting point"
        );

        let over = RenewalSchedule::stated(at(0), Duration::from_secs(601));
        assert_eq!(
            over.after_the_token_was_issued(),
            Duration::from_secs(601 - 300),
            "the before-the-expiry rule is what holds above it"
        );
    }

    /// Every stated lifetime from nothing to an hour produces a moment at or
    /// after the issue and at or before the stated expiry, which is the pair of
    /// properties a schedule in the past would break.
    #[test]
    fn no_stated_lifetime_schedules_a_renewal_outside_the_token_it_is_for() {
        for seconds in 0..3600 {
            let lifetime = Duration::from_secs(seconds);
            let due = RenewalSchedule::stated(at(0), lifetime).after_the_token_was_issued();

            assert!(
                due <= lifetime,
                "a lifetime of {seconds}s scheduled a renewal after its own expiry"
            );
            assert!(
                due >= lifetime / 2,
                "a lifetime of {seconds}s scheduled a renewal before its halfway point"
            );
        }
    }

    /// A device that slept past the moment owes one renewal on waking, and the
    /// answer is the same yes it would have been at the moment itself. There is
    /// no operation here that could turn a week of sleep into a count.
    #[test]
    fn a_device_that_slept_past_the_moment_reads_the_same_yes() {
        let schedule = RenewalSchedule::stated(at(10), Duration::from_secs(3600));

        assert!(!schedule.is_due(at(10 + 3299)));
        assert!(schedule.is_due(at(10 + 3300)));
        assert!(schedule.is_due(at(10 + 3300 + 7 * 24 * 3600)));
    }
}
