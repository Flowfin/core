//! The password's life inside the core, and the closure a sign-in answer passes.
//!
//! `docs/decisions/0030-the-password-route.md` is the record and #30 is the
//! issue. The record decides one route and three things about it: what the
//! password is allowed to touch and for how long, that the route adds no error
//! vocabulary of its own, and that there is no branch in it that returns a
//! session with a field missing.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is the part of that a type and a closure over three values
//! settle: that the password cannot be copied, cannot be read twice, cannot be
//! printed and cannot outlive the call that presents it; that the account name
//! is kept exactly as it was typed; and which answers yield the facts 0005 says
//! a session holds and which yield one of 0004's kinds instead. None of it
//! reads a clock, a socket or a store.
//!
//! The third thing 0030 decides at this door is the payload on a refused
//! credential, and it is not here either, because 0037 fixes one mapping point
//! for every value of the failure vocabulary.
//! [`crate::failure::Failure::from_status_with_no_token_presented`] is where it
//! lives: this route has nothing to present, so a 401 it receives is
//! `not-authenticated` saying there was no token, which is the opposite payload
//! to the rejection 0034 acts on and the difference #34 and #35 branch on.
//!
//! WHAT IS NOT HERE IS THE ROUTE. Presenting a credential to a server is a
//! request, and the transport is #27, which is not built, so nothing in this
//! module sends or receives a byte. #30's own condition is a sign-in against the
//! fake server in #21, and that condition is untouched by everything below. #30
//! is where that is written against the issue rather than only here.
//!
//! # The lifetime is the property, so consumption is the mechanism
//!
//! 0030 names three shapes that each keep a password alive past the request, and
//! says none of them looks wrong in review because the wrongness is the lifetime
//! rather than the line. A credentials object the caller holds so a retry is
//! easy. A field on the session so a renewal can re-authenticate. A parameter
//! threaded through a helper so a test can sign in twice.
//!
//! [`Password`] refuses all three by construction rather than by a sentence.
//! There is no accessor: [`Password::present`] takes the value by move and hands
//! a borrow to one closure, so a second presentation does not compile, a copy
//! does not compile, and the value is dropped when that closure returns. There
//! is no credentials type in this module to hold, and nothing here builds a
//! session, so there is no field for one to sit on.
//!
//! WHAT THAT DOES NOT DO IS ERASE THE BYTES. 0030 takes the password as a plain
//! string, states that what the runtime leaves behind after the reference is
//! dropped is a real cost and an unmeasurable one here, and refuses to claim a
//! scrub it cannot perform. Nothing below performs one either, and nothing below
//! reaches the platform, so a heap page, a core dump and whatever a host writes
//! when it suspends a process are outside what any of this holds.
//!
//! # Why the password has no field name
//!
//! 0071 excludes a credential from a diagnostic event outright, with no severity
//! that admits it, and [`crate::diagnostics::redaction`] makes a field
//! unwritable unless somebody chose a treatment for its name. This module
//! declares no [`crate::diagnostics::redaction::FieldName`] for the password and
//! there is no conversion from [`Password`] into a field value, so the exclusion
//! is the absence of a name rather than a rule applied to one.
//!
//! The account name is the opposite case and is treated differently on purpose.
//! 0068 places it on the personal data list, which is a statement about an event
//! rather than about a Rust value, so [`AccountName`] is ordinary data here and
//! whatever event carries it names it where that event is written.

use crate::failure::{Expected, Failure, ReadingSite};

/// A password on its way to one request, and nowhere else.
///
/// The type carries no accessor, no copy and no printable form. What it carries
/// instead is [`Password::present`], which takes the value by move, so the
/// compiler is what refuses a second presentation rather than a review. See the
/// module documentation for the three shapes 0030 names and for what this does
/// not do.
///
/// Thread safety, from 0009: a plain value, safe from any thread. It is not
/// shared between threads by anything here, and the bound says only that
/// handing it to one is defined.
pub struct Password {
    secret: String,
}

/// Carries no byte of the secret, at any formatting width.
///
/// The derived shape would print the field, and a derived `Debug` is what a
/// panic message, a test failure and an event assembled by a client all reach
/// for. 0030 puts a password in none of those, so the value is not printable in
/// the clear from anywhere, including from this crate.
impl core::fmt::Debug for Password {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Password(..)")
    }
}

impl Password {
    /// Takes what the person typed.
    ///
    /// It takes a `String` rather than something the caller can clear, which is
    /// 0030's own choice and its stated residual: a string is what every client
    /// already has, and asking for anything else puts work on eleven client
    /// authors for a property the runtime may not offer.
    ///
    /// Nothing is judged here. An empty password is a password the server will
    /// refuse, and a core that refused it first would be answering for a server
    /// whose rules it does not know.
    #[must_use]
    pub const fn supplied(secret: String) -> Self {
        Self { secret }
    }

    /// Hands the secret to whatever writes the request, once.
    ///
    /// The value is consumed, so this is the whole of its readable life: the
    /// closure sees the bytes, and the string is dropped when this call returns.
    /// A caller that needs to sign in a second time is handed a second password
    /// by the person, which is 0030's rule for a rejection rather than an
    /// inconvenience of this signature.
    ///
    /// The closure's own product is returned unchanged. Nothing here inspects it,
    /// because what a request body looks like is #27's and not this module's.
    #[must_use]
    pub fn present<T>(self, into_the_request: impl FnOnce(&str) -> T) -> T {
        into_the_request(&self.secret)
    }
}

/// The account name, as the person typed it.
///
/// 0030 takes it unrepaired, for the reason 0028 refuses to repair an address:
/// a core that trimmed, folded or otherwise corrected a name would be deciding
/// what a server accepts, and the only place that is known is the server. So
/// nothing here trims a space, changes a case or normalises anything, and a name
/// the server refuses comes back as the same refusal a wrong password does.
///
/// It is ordinary data rather than a secret. 0068 places it on the personal data
/// list, which decides what an event may carry rather than what this value is,
/// and 0006 already keeps it out of the cache key in favour of the identifier
/// the server gave back.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountName {
    typed: String,
}

impl AccountName {
    /// Keeps what was typed, whole.
    #[must_use]
    pub const fn as_typed(typed: String) -> Self {
        Self { typed }
    }

    /// The name, byte for byte as it arrived.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.typed
    }
}

/// What a reader took out of an answer this route received.
///
/// Every member is optional because that is the question 0004's closure rule
/// asks: a body that parsed and omitted one of the three is the case the rule is
/// about, and a reader that could not express the omission would have nothing to
/// hand [`what_the_answer_yields`].
///
/// NO FIELD NAME OF ANY SERVER APPEARS HERE. What a body calls these three is
/// read where the body is read, which is #27, and a name written into this tree
/// today would be a claim about an interface nobody has read - the same reason
/// `tests/fake_server/surface.rs` gives for the bodies it answers with.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnswerRead<'a> {
    /// The token, if the body carried one.
    pub token: Option<&'a str>,
    /// The account identifier the server gave back, if the body carried one.
    pub account_identifier: Option<&'a str>,
    /// What the server said about the token's validity, if the body carried it.
    ///
    /// 0005 holds "whatever the server said" rather than a duration this core
    /// invented, so this is the statement as it arrived and is not parsed here.
    pub validity: Option<&'a str>,
    /// How far into the body reading had got when it finished.
    ///
    /// 0004's fourth rule carries where reading stopped, and for a body that
    /// parsed whole and omitted a field that is the end of the body rather than
    /// the position of anything. The reader supplies it because the reader is
    /// the only thing that knows it.
    pub read_to: usize,
}

/// The four facts 0005 says a session holds that this route has to obtain.
///
/// Three of them are here. The fourth, the device identity, is the caller's
/// already and is not read out of an answer, and the resolved address and the
/// capability answers are the same case.
///
/// THE TYPE IS THE PROPERTY. 0030 says there is no branch in this route that
/// returns a session with a field missing, and every field below is present
/// rather than optional, so a branch that wanted to return an incomplete one has
/// nothing to return it in. [`what_the_answer_yields`] is the only thing that
/// builds one, which is the same construction [`crate::failure::Constructed`]
/// uses and for the same reason.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FactsASessionNeeds<'a> {
    token: &'a str,
    account_identifier: &'a str,
    validity: &'a str,
}

/// Carries no byte of the token.
///
/// 0005 makes the token the only secret and 0071 excludes it from an event
/// outright, so this value is no more printable than [`Password`] is. The other
/// two members are ordinary data and are shown, because a value that hid them
/// would make every failure here unreadable to buy nothing.
impl core::fmt::Debug for FactsASessionNeeds<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FactsASessionNeeds")
            .field("token", &"..")
            .field("account_identifier", &self.account_identifier)
            .field("validity", &self.validity)
            .finish()
    }
}

impl<'a> FactsASessionNeeds<'a> {
    /// The token, which 0005 makes the only secret a session holds.
    #[must_use]
    pub const fn token(&self) -> &'a str {
        self.token
    }

    /// The account identifier the server gave back.
    #[must_use]
    pub const fn account_identifier(&self) -> &'a str {
        self.account_identifier
    }

    /// What the server said about the token's validity, unparsed.
    #[must_use]
    pub const fn validity(&self) -> &'a str {
        self.validity
    }
}

/// Which of the three facts an answer did not carry.
///
/// It is this module's own value rather than one of 0004's kinds, on the shape
/// [`crate::session::device::PartNotUsable`] already takes: 0037 requires every
/// value of the failure vocabulary to be built at one mapping point, and this
/// says which fact was absent so that a reader of a test failure has something
/// to name. It is the core's own word for the fact rather than a field name out
/// of a body, for the reason [`AnswerRead`] carries none.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FactNotCarried {
    /// No token, so nothing to present on the next call.
    Token,
    /// No account identifier, so nothing 0005 identifies the session by and
    /// nothing 0041 keys a cache entry with.
    AccountIdentifier,
    /// No statement about validity, so nothing 0034 schedules a renewal against.
    Validity,
}

impl FactNotCarried {
    /// The name this absence is written as.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::Token => "token",
            Self::AccountIdentifier => "account-identifier",
            Self::Validity => "validity",
        }
    }
}

/// An answer that yielded no session, and the two statements about it.
///
/// The kind is 0004's and is built at 0037's mapping point. The fact is this
/// module's word for what was absent, and it is here rather than inside the kind
/// because 0004 fixes what each of the fifteen carries and no row of that list
/// grows a field for this.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSession {
    failure: Failure,
    missing: FactNotCarried,
}

impl NoSession {
    /// 0004's kind for an answer the core could not read.
    #[must_use]
    pub const fn failure(&self) -> &Failure {
        &self.failure
    }

    /// Which of the three facts the body did not carry.
    #[must_use]
    pub const fn missing(&self) -> FactNotCarried {
        self.missing
    }
}

/// Turns what a reader took out of an answer into the facts a session needs, or
/// into 0004's kind for an answer the core could not read.
///
/// This is 0004's closure rule applied at this route's door: a 2xx whose body
/// parses but omits the token, the account identifier or the validity statement
/// is `answer-not-understood` rather than a session with a hole in it. The route
/// adds no kind of its own, which is 0030's decision and the reason it gives -
/// a sign-in reporting failures in its own words would be the first of eleven
/// clients' worth of drift, inside the call every client makes first.
///
/// AN EMPTY VALUE IS TREATED AS AN ABSENT ONE, AND THAT IS THIS MODULE'S READING
/// RATHER THAN A SENTENCE IN 0030. The record says the rule is about a body that
/// omits a field. A body carrying an empty token omits the token in every sense
/// that matters downstream: the next call would present nothing, 0034 would read
/// a rejection carrying "a token was presented" against a session that presented
/// none, and the answer would be a session 0005 cannot identify. Admitting it
/// would put that discovery three calls away from the answer that caused it. The
/// cost is that a server which genuinely means something by an empty string is
/// refused here, and no supported server line is known to.
///
/// The first absence in the order the members are declared is the one reported.
/// A body missing two fields is one unreadable answer rather than two, and 0004
/// carries one reason.
///
/// # Errors
///
/// [`NoSession`], carrying [`Failure::answer_not_understood`] at
/// [`ReadingSite::AnswerBody`] expecting [`Expected::AFieldTheCoreNeeds`], where
/// one of the three was absent or empty.
pub fn what_the_answer_yields<'a>(
    read: &AnswerRead<'a>,
) -> Result<FactsASessionNeeds<'a>, NoSession> {
    let absent = |missing| NoSession {
        failure: Failure::answer_not_understood(
            ReadingSite::AnswerBody,
            Expected::AFieldTheCoreNeeds,
            read.read_to,
        ),
        missing,
    };

    let token = carried(read.token).ok_or_else(|| absent(FactNotCarried::Token))?;
    let account_identifier = carried(read.account_identifier)
        .ok_or_else(|| absent(FactNotCarried::AccountIdentifier))?;
    let validity = carried(read.validity).ok_or_else(|| absent(FactNotCarried::Validity))?;

    Ok(FactsASessionNeeds {
        token,
        account_identifier,
        validity,
    })
}

/// A fact that arrived with nothing in it is a fact that did not arrive.
fn carried(value: Option<&str>) -> Option<&str> {
    value.filter(|found| !found.is_empty())
}

#[cfg(test)]
mod tests {
    //! 0030's password and its answer, asked of the values.
    //!
    //! What these cannot ask is #30's own condition. It is a sign-in against the
    //! fake server, and nothing in this tree opens a connection to drive one
    //! over.

    use super::{
        AccountName, AnswerRead, FactNotCarried, FactsASessionNeeds, Password,
        what_the_answer_yields,
    };
    use crate::failure::{Expected, Failure, Kind, ReadingSite};

    fn whole(read_to: usize) -> AnswerRead<'static> {
        AnswerRead {
            token: Some("a-token"),
            account_identifier: Some("an-account"),
            validity: Some("a-statement"),
            read_to,
        }
    }

    /// The bytes reach the one writer and the value is spent.
    ///
    /// THE NEAR MISS HERE IS A COMPILE FAILURE RATHER THAN A RED LINE, and that
    /// is worth stating plainly. `Password` is neither `Copy` nor `Clone` and
    /// `present` takes `self`, so a second presentation - which is the retry
    /// 0030 refuses, and the credentials object it names - is refused by the
    /// compiler rather than by anything this run evaluates. Adding the line
    /// below to this case is the deliberate violation, and it reddens the build:
    ///
    ///     password.present(str::to_owned);
    #[test]
    fn a_password_is_readable_once_and_the_reading_consumes_it() {
        let password = Password::supplied(String::from("hunter2"));
        let written = password.present(str::to_owned);
        assert_eq!(written, "hunter2");
    }

    /// The closure's own product crosses out, so a writer can hand back whatever
    /// it built. Nothing about the secret crosses with it.
    #[test]
    fn what_the_writer_returns_is_what_the_presentation_answers_with() {
        let password = Password::supplied(String::from("a-secret"));
        assert_eq!(password.present(str::len), 8);
    }

    /// The near miss is the derived shape, which prints the field. This is the
    /// one thing between a password and a panic message, so it is asked at both
    /// widths a formatter offers.
    #[test]
    fn the_debug_shape_carries_no_byte_of_the_password() {
        let password = Password::supplied(String::from("a-very-distinctive-secret"));
        let plain = format!("{password:?}");
        let alternate = format!("{password:#?}");

        assert_eq!(plain, "Password(..)");
        assert!(!plain.contains("distinctive"));
        assert!(!alternate.contains("distinctive"));
    }

    /// 0030 takes the name as it was typed. The near miss is the trim somebody
    /// adds because a person pasted a space, which changes what is sent to a
    /// server whose rules this core does not know.
    #[test]
    fn an_account_name_is_kept_exactly_as_it_was_typed() {
        let padded = AccountName::as_typed(String::from("  Ada  "));
        assert_eq!(padded.as_str(), "  Ada  ");

        let cased = AccountName::as_typed(String::from("ADA"));
        assert_ne!(cased, AccountName::as_typed(String::from("ada")));
    }

    /// The answer that yields a session yields all three facts.
    #[test]
    fn an_answer_carrying_the_three_facts_yields_them() {
        let facts = what_the_answer_yields(&whole(41)).expect("all three arrived");

        assert_eq!(facts.token(), "a-token");
        assert_eq!(facts.account_identifier(), "an-account");
        assert_eq!(facts.validity(), "a-statement");
    }

    /// Each of the three, absent on its own, is 0004's fourth rule and names
    /// itself. The near miss is a route that returns a session with the missing
    /// field left empty, which is what the type above cannot express.
    #[test]
    fn a_body_that_omits_one_of_the_three_is_an_answer_the_core_cannot_read() {
        for (missing, read) in [
            (
                FactNotCarried::Token,
                AnswerRead {
                    token: None,
                    ..whole(17)
                },
            ),
            (
                FactNotCarried::AccountIdentifier,
                AnswerRead {
                    account_identifier: None,
                    ..whole(17)
                },
            ),
            (
                FactNotCarried::Validity,
                AnswerRead {
                    validity: None,
                    ..whole(17)
                },
            ),
        ] {
            let no_session =
                what_the_answer_yields(&read).expect_err("one of the three was absent");

            assert_eq!(no_session.missing(), missing);
            assert_eq!(no_session.failure().kind(), Kind::AnswerNotUnderstood);

            let &Failure::AnswerNotUnderstood {
                site,
                expected,
                stopped_at,
                ..
            } = no_session.failure()
            else {
                panic!("the closure rule mapped onto something else");
            };
            assert_eq!(site, ReadingSite::AnswerBody);
            assert_eq!(expected, Expected::AFieldTheCoreNeeds);
            assert_eq!(stopped_at, 17);
        }
    }

    /// An empty value is the near miss for the absent one, and this module reads
    /// the two the same way. The reason is on `what_the_answer_yields`.
    #[test]
    fn a_field_that_arrived_empty_is_read_as_one_that_did_not_arrive() {
        let empty_token = AnswerRead {
            token: Some(""),
            ..whole(3)
        };
        assert_eq!(
            what_the_answer_yields(&empty_token)
                .expect_err("an empty token")
                .missing(),
            FactNotCarried::Token
        );

        let empty_identifier = AnswerRead {
            account_identifier: Some(""),
            ..whole(3)
        };
        assert_eq!(
            what_the_answer_yields(&empty_identifier)
                .expect_err("an empty identifier")
                .missing(),
            FactNotCarried::AccountIdentifier
        );

        let empty_validity = AnswerRead {
            validity: Some(""),
            ..whole(3)
        };
        assert_eq!(
            what_the_answer_yields(&empty_validity)
                .expect_err("an empty statement")
                .missing(),
            FactNotCarried::Validity
        );
    }

    /// Two absences are one unreadable answer, and the reported one is the first
    /// in declaration order.
    #[test]
    fn a_body_missing_two_fields_reports_the_first_of_them() {
        let read = AnswerRead {
            token: None,
            account_identifier: None,
            ..whole(5)
        };
        assert_eq!(
            what_the_answer_yields(&read)
                .expect_err("two were absent")
                .missing(),
            FactNotCarried::Token
        );
    }

    /// The token is the only secret 0005 names, so the facts do not print it
    /// either. The two ordinary members are shown on purpose.
    #[test]
    fn the_facts_debug_shape_carries_no_byte_of_the_token() {
        let facts = what_the_answer_yields(&AnswerRead {
            token: Some("a-distinctive-token"),
            ..whole(9)
        })
        .expect("all three arrived");

        let shown = format!("{facts:?}");
        assert!(!shown.contains("distinctive"));
        assert!(shown.contains("an-account"));
        assert!(shown.contains("a-statement"));
    }

    /// The names are what a report groups by, so they are asked for rather than
    /// assumed from the variant.
    #[test]
    fn each_absent_fact_has_its_own_declared_name() {
        assert_eq!(FactNotCarried::Token.declared_name(), "token");
        assert_eq!(
            FactNotCarried::AccountIdentifier.declared_name(),
            "account-identifier"
        );
        assert_eq!(FactNotCarried::Validity.declared_name(), "validity");
    }

    /// The facts are borrowed from the answer rather than copied out of it, so
    /// nothing here holds a token past the body it came from.
    #[test]
    fn the_facts_borrow_from_the_answer_they_were_read_out_of() {
        let body = String::from("a-token-inside-a-body");
        let read = AnswerRead {
            token: Some(&body[..7]),
            ..whole(21)
        };
        let facts: FactsASessionNeeds<'_> = what_the_answer_yields(&read).expect("all three");
        assert_eq!(facts.token(), "a-token");
    }
}
