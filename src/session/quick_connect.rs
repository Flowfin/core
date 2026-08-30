//! The cadence a Quick Connect exchange is asked about on, and its four endings.
//!
//! `docs/decisions/0031-the-quick-connect-route.md` is the record and #31 is the
//! issue. The record decides one route and, of it, three things that are settled
//! without a socket: the interval between one question and the next, what a call
//! that started an exchange may end in, and which of the two values the server
//! issued crosses to the client.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of 0031 that two readings of one clock and a
//! construction settle: when the next question about an exchange is due, that
//! the interval does not move however many questions have been asked, what a
//! poll that failed does to the schedule, which of the four endings is a failure
//! and which three are answers, and which value a client is handed.
//!
//! WHAT IS NOT HERE IS THE POLL. Nothing in this tree opens a connection, for
//! the reason [`crate::server::transport`] gives about itself, so nothing asks a
//! server anything, nothing starts an exchange and nothing receives a code.
//! This module holds the schedule such a thing would run on and the shape of
//! what it would return. #31's three conditions are about requests made against
//! the fake server, and none of them is met by anything here.
//!
//! WHAT IS ALSO NOT HERE IS THE ROUTE'S OWN DETECTION. 0031 says the core asks
//! the configured server and never guesses, and that where the server states
//! nothing or states that the route is off the call is `capability-absent`
//! carrying [`crate::failure::Capability::QuickConnect`]. That value is built at
//! the one mapping point 0037 fixes, from an answer this tree cannot receive, so
//! this module names the capability and constructs nothing.
//!
//! # The number here is chosen and not measured
//!
//! 0031 says so of its own interval, in the same words 0045 uses for its
//! schedule: it is chosen against how long a person is willing to watch a
//! television not respond, and there was no code in this repository to measure
//! anything with. #65 is the harness that would replace a choice with a number,
//! and until it exists a reader should take the constant below as an argument
//! rather than as a measurement.

use core::time::Duration;

use crate::clock::SteadyInstant;
use crate::failure::{Capability, Failure};

/// The five seconds between one question about an exchange and the next.
///
/// From 0031, and it is the one number that record owns. Fixed rather than
/// doubling and not drawn from a range, so the delay between somebody approving
/// on a second device and the screen in front of them moving on is at most one
/// interval whether they approved at the first minute or at the ninth.
///
/// The capability this interval belongs to is
/// [`Capability::QuickConnect`], which is the name a call refused before an
/// exchange is started carries.
pub const THE_NEXT_QUESTION_IS_DUE_AFTER: Duration = Duration::from_secs(5);

/// Where one Quick Connect exchange stands between two questions.
///
/// It is per exchange. 0031 starts one exchange per call, so two calls are two
/// of these, and this type holds one of them rather than a set.
///
/// Every interval it measures is on the STEADY clock. 0102 puts an interval
/// between two events inside one run there, and 0031 names that clock for this
/// cadence in so many words. A device that suspended mid-wait has no exchange in
/// flight to be late for; what it has is a question that goes out when it wakes,
/// which is what the steady clock gives and what the elapsed clock would not.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhileWaiting {
    last_heard_from_the_server_at: SteadyInstant,
    questions_asked: u32,
}

impl WhileWaiting {
    /// An exchange the server has just issued a code for, with no question asked
    /// about it yet.
    ///
    /// `at` is the moment that answer arrived, and the first question is due one
    /// interval after it.
    ///
    /// THAT THE CADENCE RUNS FROM THE ANSWER IS A READING RATHER THAN A
    /// QUOTATION. 0031 says the core asks about the exchange every five seconds
    /// and does not say whether the first question follows the code immediately
    /// or after one interval. The reading taken here is that the call which
    /// issued the code was itself an exchange with the server about this
    /// attempt, so the interval runs from it like every other. The alternative
    /// asks the same server the same question twice inside one round trip, for
    /// an approval that could not have happened in between.
    #[must_use]
    pub const fn the_code_was_issued_at(at: SteadyInstant) -> Self {
        Self {
            last_heard_from_the_server_at: at,
            questions_asked: 0,
        }
    }

    /// The moment the server was last heard from about this exchange.
    #[must_use]
    pub const fn last_heard_from_the_server_at(self) -> SteadyInstant {
        self.last_heard_from_the_server_at
    }

    /// How many questions have been asked about this exchange.
    ///
    /// It is here so that a caller reporting what the core is doing has the
    /// count, and NOT so that the schedule can read it. Nothing below branches
    /// on this number, which is the whole of 0031's refusal of a backoff
    /// expressed where somebody would otherwise add one.
    #[must_use]
    pub const fn questions_asked(self) -> u32 {
        self.questions_asked
    }

    /// How long until the next question about this exchange is due.
    ///
    /// Zero once it is due, rather than a negative interval, which is the floor
    /// [`SteadyInstant::interval_since`] already takes for the same reason.
    #[must_use]
    pub const fn until_the_next_question(self, now: SteadyInstant) -> Duration {
        let waited = now.interval_since(self.last_heard_from_the_server_at);
        if waited.as_nanos() >= THE_NEXT_QUESTION_IS_DUE_AFTER.as_nanos() {
            Duration::ZERO
        } else {
            THE_NEXT_QUESTION_IS_DUE_AFTER.saturating_sub(waited)
        }
    }

    /// Whether the next question about this exchange is due.
    ///
    /// True from [`THE_NEXT_QUESTION_IS_DUE_AFTER`] onwards, and it is the same
    /// answer at the first question and at the thousandth.
    #[must_use]
    pub const fn a_question_is_due(self, now: SteadyInstant) -> bool {
        now.interval_since(self.last_heard_from_the_server_at)
            .as_nanos()
            >= THE_NEXT_QUESTION_IS_DUE_AFTER.as_nanos()
    }

    /// The state after one more question was asked and the server answered that
    /// the exchange is still pending.
    ///
    /// `at` is the moment of that answer. The interval to the next question is
    /// [`THE_NEXT_QUESTION_IS_DUE_AFTER`] again, which is 0031's decision not to
    /// back off: a server saying an exchange is pending is a server answering
    /// normally, and stretching the interval makes the person who approved at
    /// the ninth minute wait longer than the person who approved at the first
    /// for a reason neither of them could discover.
    #[must_use]
    pub const fn after_a_question_answered_at(self, at: SteadyInstant) -> Self {
        Self {
            last_heard_from_the_server_at: at,
            questions_asked: self.questions_asked.saturating_add(1),
        }
    }

    /// The state after a question that failed rather than answering.
    ///
    /// It is deliberately the same state
    /// [`WhileWaiting::after_a_question_answered_at`] produces, and it has its
    /// own name so that the decision is visible where somebody would otherwise
    /// write a different one. 0031: a poll that fails is not an ending, the
    /// attempt stays open and the next question goes out on the ordinary
    /// cadence. A person holding a phone in a corridor with one bar should not
    /// be signed out of a sign-in because one request timed out.
    ///
    /// The failure is not lost by being unhandled here. It leaves as a
    /// diagnostic event under 0100, which is 0005's rule that a client hears
    /// nothing on this route except through diagnostics.
    #[must_use]
    pub const fn after_a_question_that_failed_at(self, at: SteadyInstant) -> Self {
        self.after_a_question_answered_at(at)
    }
}

/// How a call that started a Quick Connect exchange ended.
///
/// 0031 fixes exactly four and no fifth: a session, a denial, an expiry, or the
/// caller's own cancellation. A core stopping under 0115 ends an open call the
/// same way the caller cancelling it does.
///
/// THREE OF THE FOUR ARE ANSWERS RATHER THAN FAILURES, and that is the decision
/// this type carries rather than a shape it happens to have.
/// [`HowTheCallEnded::what_the_caller_is_failed_with`] is where it bites: it is
/// `None` for a denial and for an expiry. Somebody refusing a sign-in on their
/// phone has not met a failure of the core, of the network or of the server, and
/// the shortest code that compiles maps a denial onto whichever failure is
/// nearest because the call already has a failure path and does not yet have a
/// three-state answer. That reads as correct and passes a test asserting the
/// call did not succeed.
///
/// The alternative 0031 priced is two more kinds in 0004, a sixteenth and a
/// seventeenth for two conditions that are not failures, which is a change to
/// that record and to every client.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowTheCallEnded {
    /// Somebody approved the exchange, and a session was established.
    ///
    /// 0005 fixes what that is for all three routes, and nothing after this
    /// point knows which route produced the token.
    Approved,
    /// Somebody refused the exchange.
    Denied,
    /// The code stopped being approvable because the server's own limit passed.
    ///
    /// That limit is the server's. 0031 neither reads it nor sets one beside it,
    /// and the core imposes no total limit of its own on this route.
    Expired,
    /// The caller stopped the call, or the core stopped under 0115.
    Cancelled,
}

impl HowTheCallEnded {
    /// The value of 0004's vocabulary a caller is failed with, where there is
    /// one.
    ///
    /// `None` for the three endings that are answers. `Some` for exactly one of
    /// the four, and it is [`Failure::cancelled`] rather than anything about
    /// this route: 0009 separates a cancelled call from every failure, and a
    /// caller holding an outcome should not be able to tell from its shape which
    /// part of the core built it.
    #[must_use]
    pub const fn what_the_caller_is_failed_with(self) -> Option<Failure> {
        match self {
            Self::Approved | Self::Denied | Self::Expired => None,
            Self::Cancelled => Some(Failure::cancelled()),
        }
    }

    /// Whether a client showing three different things has to distinguish this
    /// ending from the other two answers.
    ///
    /// True for the three answers and false for the cancellation, because a
    /// caller that cancelled already knows it did. This is #31's "a client shows
    /// three different things" written where a client author meets it.
    #[must_use]
    pub const fn a_client_shows_it(self) -> bool {
        match self {
            Self::Approved | Self::Denied | Self::Expired => true,
            Self::Cancelled => false,
        }
    }
}

/// The capability a call refused before an exchange is started names.
///
/// 0031: where the configured server states nothing about this route, or states
/// that it is off, the call is `capability-absent` from 0004 carrying the
/// capability name out of the set #10 fixes. A client that asked is told before
/// a person is shown a code that nothing will ever approve.
///
/// This is the name rather than the value. Building the failure is the mapping
/// point 0037 fixes, from an answer this tree cannot receive.
pub const THE_CAPABILITY_A_SERVER_MAY_NOT_OFFER: Capability = Capability::QuickConnect;

/// The two values the server issues when an exchange begins, and the boundary
/// between them.
///
/// 0031 decides that the code crosses to the client and that a second value the
/// server issues alongside it - the one the core presents when it asks about
/// that exchange - stays inside the core. It is held for the length of the call,
/// it is never written through the store in 0033 because it does not outlive the
/// process, and it never reaches the cache.
///
/// THE BOUNDARY IS THE CONSTRUCTION RATHER THAN A RULE SOMEBODY FOLLOWS. There
/// is no accessor outside this crate for the presented value, so a client cannot
/// be handed it by a caller who found one more field convenient. 0031 names that
/// as the third of the three properties that are wrong in a way nothing reports,
/// and says why it is not recoverable afterwards: once one client has the value,
/// changing the core does not take it back.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Clone, PartialEq, Eq)]
pub struct IssuedExchange {
    code: String,
    presented: String,
}

impl IssuedExchange {
    /// The pair as the server issued it.
    #[must_use]
    pub fn as_the_server_issued_it(code: String, presented: String) -> Self {
        Self { code, presented }
    }

    /// The code a person reads off the screen and repeats somewhere else.
    ///
    /// This is the whole of what crosses the boundary while an exchange is open.
    #[must_use]
    pub fn the_code_a_client_shows(&self) -> &str {
        &self.code
    }

    /// The value the core presents when it asks about this exchange.
    ///
    /// Reachable inside this crate and nowhere else, which is 0031's decision
    /// rather than a visibility somebody chose. The request that would carry it
    /// is #27's and is not built, so nothing outside the cases below calls this
    /// today.
    ///
    /// THE EXPECTATION IS WHAT KEEPS THAT SENTENCE TRUE IN BOTH DIRECTIONS. It
    /// is `expect` rather than `allow`: the day the request in #27 calls this,
    /// the lint stops firing, the unfulfilled expectation becomes a warning, and
    /// `-D warnings` in the build check turns it red. So the attribute has to be
    /// removed by whoever wires the value up, and the sentence above cannot go
    /// stale in silence.
    ///
    /// It is conditional because the cases below DO call it, so the expectation
    /// is unfulfilled in the test build and fulfilled in the shipped one. A
    /// plain `expect` reddens the test build for the same reason it is supposed
    /// to redden the other, which would report a test reaching the seam as
    /// though it were the request arriving.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the request that presents this value is #27 and is not built"
        )
    )]
    #[must_use]
    pub(crate) fn the_value_the_core_presents(&self) -> &str {
        &self.presented
    }
}

/// Neither value is written out.
///
/// 0031 excludes both from a diagnostic event, on 0071's default for a field
/// nobody classified. The reason for excluding the code is not confidentiality -
/// it is on a screen in somebody's living room - but that an event carrying it
/// pairs one person's pending sign-in with the moment they made it, on a route
/// whose whole shape is that the core cannot see where the approving happens.
///
/// A derived implementation here would write both into any event, assertion
/// message or panic that formatted one, which is a route out of the core that
/// 0071's treatment never sees.
impl core::fmt::Debug for IssuedExchange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IssuedExchange").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    //! 0031's cadence, its four endings and its boundary, asked of the values.
    //!
    //! What these cannot ask is any of #31's three conditions. Each of those is
    //! an exchange with the fake server, and nothing in this tree sends a
    //! request.

    use super::{
        HowTheCallEnded, IssuedExchange, THE_CAPABILITY_A_SERVER_MAY_NOT_OFFER,
        THE_NEXT_QUESTION_IS_DUE_AFTER, WhileWaiting,
    };
    use crate::clock::SteadyInstant;
    use crate::failure::{Capability, Kind};
    use core::time::Duration;

    const NANOS_IN_A_SECOND: u64 = 1_000_000_000;

    fn at(seconds: u64) -> SteadyInstant {
        SteadyInstant::from_nanos(seconds * NANOS_IN_A_SECOND)
    }

    fn after(questions: u32) -> WhileWaiting {
        let mut waiting = WhileWaiting::the_code_was_issued_at(at(0));
        for _ in 0..questions {
            waiting = waiting.after_a_question_answered_at(waiting.last_heard_from_the_server_at());
        }
        waiting
    }

    /// The interval, and the boundary itself rather than a value either side of
    /// it.
    #[test]
    fn a_question_is_due_five_seconds_after_the_server_was_last_heard_from() {
        let waiting = WhileWaiting::the_code_was_issued_at(at(0));

        assert!(!waiting.a_question_is_due(at(4)));
        assert!(waiting.a_question_is_due(at(5)));
        assert!(waiting.a_question_is_due(at(600)));
        assert_eq!(THE_NEXT_QUESTION_IS_DUE_AFTER.as_secs(), 5);
    }

    /// The cadence runs from the last answer rather than from the start of the
    /// exchange, so a question asked on time does not make the next one due at
    /// once.
    #[test]
    fn each_answer_moves_the_cadence_on() {
        let waiting =
            WhileWaiting::the_code_was_issued_at(at(0)).after_a_question_answered_at(at(5));

        assert_eq!(waiting.last_heard_from_the_server_at(), at(5));
        assert!(!waiting.a_question_is_due(at(9)));
        assert!(waiting.a_question_is_due(at(10)));
        assert_eq!(waiting.questions_asked(), 1);
    }

    /// 0031's refusal of a backoff, which is the property most likely to be
    /// changed by somebody being careful and is invisible in a test that
    /// approves the exchange at once.
    #[test]
    fn the_interval_does_not_move_however_many_questions_have_been_asked() {
        for questions in [0_u32, 1, 2, 7, 8, 63, 64, 1000] {
            let waiting = after(questions);

            assert_eq!(
                waiting.until_the_next_question(at(0)),
                THE_NEXT_QUESTION_IS_DUE_AFTER,
                "the interval moved after {questions} question(s)"
            );
            assert!(
                !waiting.a_question_is_due(at(4)),
                "a question became due early after {questions} question(s)"
            );
            assert!(
                waiting.a_question_is_due(at(5)),
                "a question was not due on time after {questions} question(s)"
            );
        }
    }

    /// The wait remaining, and that it floors at zero rather than running
    /// negative once the question is overdue.
    #[test]
    fn the_wait_remaining_counts_down_and_stops_at_zero() {
        let waiting = WhileWaiting::the_code_was_issued_at(at(0));

        assert_eq!(
            waiting.until_the_next_question(at(0)),
            Duration::from_secs(5)
        );
        assert_eq!(
            waiting.until_the_next_question(at(4)),
            Duration::from_secs(1)
        );
        assert_eq!(waiting.until_the_next_question(at(5)), Duration::ZERO);
        assert_eq!(waiting.until_the_next_question(at(600)), Duration::ZERO);
    }

    /// A poll that failed is not an ending: it moves the cadence on exactly as
    /// an answer does, and the attempt stays open.
    #[test]
    fn a_question_that_failed_leaves_the_exchange_where_an_answer_does() {
        let answered =
            WhileWaiting::the_code_was_issued_at(at(0)).after_a_question_answered_at(at(5));
        let failed =
            WhileWaiting::the_code_was_issued_at(at(0)).after_a_question_that_failed_at(at(5));

        assert_eq!(answered, failed);
        assert!(!failed.a_question_is_due(at(9)));
        assert!(failed.a_question_is_due(at(10)));
    }

    /// The count saturates rather than wrapping, so a very long wait cannot make
    /// the schedule read as a fresh exchange. Nothing branches on the count, so
    /// this is a statement about the reported number alone.
    #[test]
    fn the_question_count_saturates() {
        let waiting = WhileWaiting {
            last_heard_from_the_server_at: at(0),
            questions_asked: u32::MAX,
        };

        assert_eq!(
            waiting
                .after_a_question_answered_at(at(5))
                .questions_asked(),
            u32::MAX
        );
    }

    /// Denied and expired are answers rather than failures, which is the ending
    /// the shortest code that compiles gets wrong.
    #[test]
    fn only_the_cancellation_of_the_four_endings_fails_the_caller() {
        assert!(
            HowTheCallEnded::Approved
                .what_the_caller_is_failed_with()
                .is_none()
        );
        assert!(
            HowTheCallEnded::Denied
                .what_the_caller_is_failed_with()
                .is_none()
        );
        assert!(
            HowTheCallEnded::Expired
                .what_the_caller_is_failed_with()
                .is_none()
        );

        let cancelled = HowTheCallEnded::Cancelled
            .what_the_caller_is_failed_with()
            .expect("the cancellation is the one ending that fails the caller");
        assert_eq!(cancelled.kind(), Kind::Cancelled);
    }

    /// Three different things, which is what #31 asks a client to show.
    #[test]
    fn the_three_answers_are_what_a_client_shows() {
        assert!(HowTheCallEnded::Approved.a_client_shows_it());
        assert!(HowTheCallEnded::Denied.a_client_shows_it());
        assert!(HowTheCallEnded::Expired.a_client_shows_it());
        assert!(!HowTheCallEnded::Cancelled.a_client_shows_it());
    }

    /// The capability a call refused in front of an exchange names, which is the
    /// one #10 fixes for this route rather than a name invented here.
    #[test]
    fn the_route_names_the_capability_from_the_declared_set() {
        assert_eq!(
            THE_CAPABILITY_A_SERVER_MAY_NOT_OFFER,
            Capability::QuickConnect
        );
    }

    /// The code crosses and the presented value does not, which is the boundary
    /// 0031 draws.
    #[test]
    fn the_code_is_what_a_client_is_handed() {
        let issued = IssuedExchange::as_the_server_issued_it(
            "123456".to_owned(),
            "a-value-the-core-presents".to_owned(),
        );

        assert_eq!(issued.the_code_a_client_shows(), "123456");
        assert_eq!(
            issued.the_value_the_core_presents(),
            "a-value-the-core-presents"
        );
    }

    /// Neither value is written out by a formatting call, which is the route out
    /// of the core that 0071's treatment never sees.
    #[test]
    fn neither_value_reaches_a_formatted_line() {
        let issued = IssuedExchange::as_the_server_issued_it(
            "123456".to_owned(),
            "a-value-the-core-presents".to_owned(),
        );

        let written = format!("{issued:?}");

        assert!(!written.contains("123456"), "the code reached {written}");
        assert!(
            !written.contains("a-value-the-core-presents"),
            "the presented value reached {written}"
        );
    }
}
