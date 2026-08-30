//! The attempt a delegated sign-in is tied to, and the order it is matched in.
//!
//! `docs/decisions/0032-a-server-that-delegates-sign-in.md` is the record and
//! #32 is the issue. The record decides one route and three properties of it:
//! that the value tying an attempt to its answer is generated per attempt from
//! unpredictable bytes, that comparison of it is over the whole value, and that
//! an answer is matched against a started attempt BEFORE it is used and accepted
//! at most once.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that a comparison of two byte strings and a
//! membership question settle: which answers name an attempt this process
//! started, which of them are refused, what a second answer for one attempt
//! finds, and what a caller is given only once a match has happened. None of it
//! reads a clock, a socket or a store, and each is wrong in a way nothing
//! downstream would report.
//!
//! WHAT IS NOT HERE IS THE ROUTE. Knowing that a server delegates is an answer
//! from the configured server, the address handed to the client is built from
//! the origin 0028 resolved, and the exchange at the end is an ordinary request.
//! All three need the transport, which is #27 and is not built, so nothing in
//! this module sends or receives a byte. #32 is where that is written against
//! the issue rather than only here.
//!
//! # The order is a type rather than a rule
//!
//! 0032 says the order is the property that gets written correctly by accident
//! and then lost, because checking after using reads identically to checking
//! before. So the answer a caller hands in is not returned to it in a form
//! anything can relay until it has matched: [`Relayable`] is produced by
//! [`OpenAttempts::answer`] and by nothing else. That is the same construction
//! [`crate::failure::Constructed`] uses, for the same reason.
//!
//! NOTHING IN THIS TREE CONSUMES A [`Relayable`] YET, because the exchange it
//! would be relayed in is #27. The type is the seam that exchange will take, and
//! until it lands this constrains an order at compile time and proves nothing
//! about a request.
//!
//! # Where the bytes come from
//!
//! 0011 measured the toolchain and found no source of unpredictable bytes on a
//! stable build, so the seam 0032 itself named is what is used and the client
//! supplies them. 0036 already pays for that once for the device identity, and
//! [`crate::session::device::LEAST_UNPREDICTABLE_BYTES`] is the width both take;
//! this module reads that constant rather than declaring a second copy of one
//! number.

use super::device::LEAST_UNPREDICTABLE_BYTES;

/// Why a value offered for an attempt was refused.
///
/// A local answer rather than a value of the failure vocabulary, on the shape
/// [`crate::session::device::PartNotUsable`] already takes: 0037 requires every
/// value of that vocabulary to be built at one mapping point, and a client
/// handing the core too few bytes is a caller's mistake rather than an answer
/// being read.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueNotUsable {
    /// Fewer unpredictable bytes than [`LEAST_UNPREDICTABLE_BYTES`].
    ///
    /// Carries what was supplied, because the caller's own count is the thing it
    /// has to change, and a refusal that does not say the number sends somebody
    /// to read this file.
    FewerBytesThanTheWidth {
        /// How many bytes reached [`TieValue::from_unpredictable_bytes`].
        supplied: usize,
    },
}

/// A second attempt was started under a value one open attempt already carries.
///
/// This module's own refusal rather than anything 0032 asks for, and it is worth
/// saying which. 0032 fixes that the value is drawn per attempt from a source of
/// unpredictable bytes; two open attempts sharing one value is that sentence not
/// holding, and the cost of letting it through is silent: the second answer to
/// arrive would finish whichever attempt the scan reached first, and accepted at
/// most once would still look true from outside. Refusing at the start is where
/// the condition is visible.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueAlreadyOpen;

/// An answer named no attempt this process started and has not already finished.
///
/// IT CARRIES NOTHING, AND THE ABSENCE IS THE DECISION. 0032 maps this onto
/// `not-authenticated` from 0004 with the payload saying there was no token to
/// present, and refuses to distinguish a client that lost its own attempt from
/// an answer somebody injected: the two are not separable to anybody outside the
/// core in a way that would be safe to publish, because telling them apart in
/// the answer would be telling the second one which of the two it was. A field
/// here naming which of the three conditions was met - no such value, a
/// different return address, or an attempt already finished - would be that
/// distinction arriving through a type instead of through a message.
///
/// THE MAPPING ONTO THE VOCABULARY IS NOT IN THIS TREE. It belongs where the
/// route is built, which is #32 and needs #27, and [`crate::failure::Failure`]
/// carries no constructor for the payload 0032 names. This type is what that
/// mapping point will be handed.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoAttemptMatched;

/// The value that ties one attempt to its answer.
///
/// 0032 fixes what it is: generated by the core for each attempt, at least 128
/// bits, and never a clock, a counter, an identifier the core already holds, or
/// anything derived from the session being established. `no-platform-clock` in
/// `.github/invariants/rules` refuses a reading of the machine anywhere under
/// `src/`, so the first of those four has a machine behind it, for 0102's reason
/// rather than for this record's. The other three are properties of the bytes a
/// client supplied, are not decidable here, and no test below claims they are.
/// That obligation is the client's and the conformance suite in #76 is where a
/// client is asked about it.
///
/// # It is a secret while its attempt is open
///
/// 0032 puts it in the same class as the token for as long as the attempt is
/// open: excluded from a diagnostic event by 0071's rule for anything derived
/// from a credential, never written to the cache, and never written through
/// 0033, because it does not outlive the process. There is no accessor here that
/// hands the bytes back, and the debug shape below is written out by hand, on
/// [`crate::diagnostics::redaction::CorrelatorSalt`]'s pattern, so the value
/// cannot reach an output through the trait every type in this crate carries.
///
/// WHAT IS NOT DONE IS THE ERASURE. The bytes are dropped with the value and
/// nothing here overwrites them first. `src/lib.rs` forbids unsafe code, a write
/// the compiler is free to remove is not an erasure, and a dependency that does
/// it would need arguing against 0103. So a copy may remain in freed memory
/// until it is reused, which is a residual rather than something this module
/// prevents.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Clone)]
pub struct TieValue {
    bytes: Vec<u8>,
}

/// Written out rather than derived, so the value this type exists to keep off
/// every output cannot reach one through the trait.
impl core::fmt::Debug for TieValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TieValue").finish_non_exhaustive()
    }
}

/// Comparison over the whole value, which 0032 requires in so many words.
///
/// The loop reads to the end of the shorter string whatever it has already seen,
/// and a difference in length is folded in before it starts rather than answered
/// by returning early. A derived implementation would stop at the first byte
/// that differs and at a length mismatch, which is the thing the record's
/// sentence is about.
///
/// WHAT THAT SENTENCE BUYS IS BOUNDED BY WHAT A SOURCE CAN SAY. This is a
/// property of the code written here; nothing in this repository asks what the
/// compiler emitted, and no test below measures a duration. A reader who takes
/// the loop for a timing guarantee is taking more than it offers.
impl PartialEq for TieValue {
    fn eq(&self, other: &Self) -> bool {
        let mut differing = u8::from(self.bytes.len() != other.bytes.len());
        for (mine, theirs) in self.bytes.iter().zip(&other.bytes) {
            differing |= mine ^ theirs;
        }
        differing == 0
    }
}

impl Eq for TieValue {}

impl TieValue {
    /// Takes the unpredictable bytes a client supplied, and refuses too few.
    ///
    /// The bytes are held as they arrived. A digest here would be free to add
    /// and would say something untrue: it would read as though the core had made
    /// the value harder to guess, when what it would do is fix the width of a
    /// value whose unpredictability is entirely the client's doing. 0036 took
    /// the same position for the device identity and wrote the reason down
    /// there.
    ///
    /// There is no upper width. 0032 states a floor and no ceiling, and a
    /// ceiling invented here would refuse a client that supplied more entropy
    /// than the record asks for.
    ///
    /// # Errors
    ///
    /// [`ValueNotUsable::FewerBytesThanTheWidth`] where fewer than
    /// [`LEAST_UNPREDICTABLE_BYTES`] arrived, carrying how many did.
    pub fn from_unpredictable_bytes(bytes: &[u8]) -> Result<Self, ValueNotUsable> {
        if bytes.len() < LEAST_UNPREDICTABLE_BYTES {
            return Err(ValueNotUsable::FewerBytesThanTheWidth {
                supplied: bytes.len(),
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// How many bytes the value is.
    ///
    /// The width and never the value. A caller that wants to check it supplied
    /// what it meant to can ask this; nothing here hands the bytes back.
    #[must_use]
    pub fn width(&self) -> usize {
        self.bytes.len()
    }
}

/// What the client handed back, after it has matched an attempt.
///
/// This is the seam 0032's order property is held by. It is produced by
/// [`OpenAttempts::answer`] and by nothing else, so a caller cannot hold the
/// answer in the shape an exchange takes until the match has already happened.
/// 0032 states the failure that construction is against: a core that exchanged
/// the answer with the server and then checked which attempt it belonged to has
/// already sent an attacker's value to the operator's server.
///
/// # It is untrusted and it is not bounded here
///
/// 0101 reaches this value: it left the process and is untrusted whatever it
/// claims about itself. 0032 says it is validated for shape and bounded in
/// length before anything is done with it, AND STATES NEITHER A SHAPE NOR A
/// NUMBER. Neither is invented here. What this type carries is the ordering
/// alone, the bound is owed where the shape it is bounded against is decided,
/// and a reader who takes a `Relayable` for a validated value is taking more
/// than it offers.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Clone)]
pub struct Relayable {
    answer: String,
}

/// Written out by hand for the reason [`TieValue`]'s is: what a provider handed
/// back is exchanged for a token, so 0071's rule for anything derived from a
/// credential reaches it, and a derived shape would put it in every report that
/// prints the value it travels inside.
impl core::fmt::Debug for Relayable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Relayable").finish_non_exhaustive()
    }
}

impl Relayable {
    /// The answer, for the exchange to carry.
    ///
    /// Public because the request that relays it is written outside this module,
    /// and crate-private would move the seam rather than remove it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.answer
    }
}

/// One attempt this process started and has not finished.
struct Attempt {
    value: TieValue,
    return_address: Option<String>,
}

/// Written out by hand so an attempt cannot carry its value into an output
/// through the field, which is the leak [`TieValue`]'s own shape is against.
impl core::fmt::Debug for Attempt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Attempt").finish_non_exhaustive()
    }
}

/// The attempts this process has started and not finished.
///
/// # There is no expiry here and there is not going to be one
///
/// 0032 says an attempt ends when it is answered, when the caller cancels it, or
/// when the core stops under 0115, and in exactly one of those. There is no
/// number for how long a person may take, which is 0005's treatment of the wait
/// in #31 arriving unchanged: a person authenticating somewhere else is doing
/// something the core cannot see and has no business timing. So this type reads
/// no clock, and the set is bounded by the calls the caller is holding rather
/// than by a rule of the core's, which is the same bound every other outstanding
/// call in the core has.
///
/// # Why a scan rather than a map
///
/// A hash lookup compares a hash before it compares a value, and a hash is a
/// reduction that answers before the whole value has been read. 0032's
/// comparison sentence is about the value, so the lookup is the place it would
/// be lost, and the set is bounded by the caller's own outstanding calls, which
/// is small. [`TieValue`] carries no `Hash` for the same reason: a type that
/// cannot be hashed cannot be put in the container that would lose the property.
///
/// Thread safety, from 0009: this holds the state of calls in flight. It is
/// safe from any thread as a value, and nothing here takes a lock; who may
/// mutate one is 0009's rule for the core's own state rather than this module's.
#[derive(Debug, Default)]
pub struct OpenAttempts {
    open: Vec<Attempt>,
}

impl OpenAttempts {
    /// No attempt started.
    #[must_use]
    pub const fn none() -> Self {
        Self { open: Vec::new() }
    }

    /// Starts an attempt under a value, with the return address it is tied to.
    ///
    /// The return address is the client's, supplied at the time of the call and
    /// carried through unchanged, because only the client knows what its
    /// platform can receive. 0032 makes it part of what the attempt is tied to,
    /// so an answer arriving for a different return address is an answer for a
    /// different attempt, and [`OpenAttempts::answer`] refuses it as one.
    ///
    /// `None` is the route that needs no return address, and it is a value of
    /// the tie rather than a wildcard: an answer that names one does not match
    /// an attempt started without one, and the other way round.
    ///
    /// # Errors
    ///
    /// [`ValueAlreadyOpen`] where an open attempt already carries this value.
    /// See that type for why this is refused rather than allowed.
    pub fn start(
        &mut self,
        value: TieValue,
        return_address: Option<&str>,
    ) -> Result<(), ValueAlreadyOpen> {
        if self.open.iter().any(|attempt| attempt.value == value) {
            return Err(ValueAlreadyOpen);
        }
        self.open.push(Attempt {
            value,
            return_address: return_address.map(ToOwned::to_owned),
        });
        Ok(())
    }

    /// Matches an answer against a started attempt, and finishes it.
    ///
    /// The attempt is removed before anything is returned, so an answer is
    /// accepted at most once and a second answer naming the same value finds
    /// nothing started. That is 0032's sentence held by the removal rather than
    /// by a flag somebody has to read.
    ///
    /// # Errors
    ///
    /// [`NoAttemptMatched`], carrying nothing, where no open attempt has this
    /// value and this return address. See that type for why the three ways to
    /// reach it are not told apart.
    pub fn answer(
        &mut self,
        value: &TieValue,
        return_address: Option<&str>,
        answer: &str,
    ) -> Result<Relayable, NoAttemptMatched> {
        let found = self.open.iter().position(|attempt| {
            attempt.value == *value && attempt.return_address.as_deref() == return_address
        });
        let Some(index) = found else {
            return Err(NoAttemptMatched);
        };
        self.open.remove(index);
        Ok(Relayable {
            answer: answer.to_owned(),
        })
    }

    /// Ends an attempt because the caller cancelled it.
    ///
    /// The return address is not asked for. A cancellation comes from the caller
    /// that started the attempt and is not a value arriving from outside the
    /// process, so the tie has nothing to answer here; what the tie is for is
    /// telling an answer apart from an answer for something else.
    ///
    /// # Errors
    ///
    /// [`NoAttemptMatched`] where no open attempt carries this value, which is
    /// what a second cancellation and a cancellation after an answer both reach.
    pub fn cancel(&mut self, value: &TieValue) -> Result<(), NoAttemptMatched> {
        let found = self.open.iter().position(|attempt| attempt.value == *value);
        let Some(index) = found else {
            return Err(NoAttemptMatched);
        };
        self.open.remove(index);
        Ok(())
    }

    /// Ends every open attempt, which is what the core stopping does to them.
    ///
    /// 0115 is what stopping means and it is not decided, so this is the effect
    /// on this set alone and says nothing about what else stopping does. After
    /// it, every value that was open reaches [`NoAttemptMatched`], which is the
    /// same answer an unstarted one reaches.
    pub fn stop(&mut self) {
        self.open.clear();
    }

    /// How many attempts are open.
    #[must_use]
    pub fn open(&self) -> usize {
        self.open.len()
    }
}

#[cfg(test)]
mod tests {
    //! 0032's value and its order, asked of the values.
    //!
    //! What these cannot ask is #32's own condition. Each of its three tests
    //! drives the fake server through the route, and nothing in this tree opens
    //! a connection to drive one over.

    use super::{NoAttemptMatched, OpenAttempts, TieValue, ValueAlreadyOpen, ValueNotUsable};
    use crate::session::device::LEAST_UNPREDICTABLE_BYTES;

    fn value(seed: u8) -> TieValue {
        let mut bytes = [seed; LEAST_UNPREDICTABLE_BYTES];
        bytes[0] = seed ^ 0x5a;
        TieValue::from_unpredictable_bytes(&bytes).expect("the width above is the width admitted")
    }

    /// The floor is 0032's 128 bits, and the boundary itself rather than a value
    /// either side of it. The near miss is one byte short.
    #[test]
    fn a_value_narrower_than_a_hundred_and_twenty_eight_bits_is_refused() {
        assert_eq!(LEAST_UNPREDICTABLE_BYTES, 16);

        let short = [7_u8; LEAST_UNPREDICTABLE_BYTES - 1];
        assert_eq!(
            TieValue::from_unpredictable_bytes(&short),
            Err(ValueNotUsable::FewerBytesThanTheWidth {
                supplied: LEAST_UNPREDICTABLE_BYTES - 1
            })
        );

        let exact = [7_u8; LEAST_UNPREDICTABLE_BYTES];
        assert_eq!(
            TieValue::from_unpredictable_bytes(&exact)
                .expect("the width itself is admitted")
                .width(),
            LEAST_UNPREDICTABLE_BYTES
        );

        assert_eq!(
            TieValue::from_unpredictable_bytes(&[]),
            Err(ValueNotUsable::FewerBytesThanTheWidth { supplied: 0 })
        );
    }

    /// A wider value is admitted. 0032 states a floor and no ceiling.
    #[test]
    fn a_wider_value_is_admitted_and_keeps_its_width() {
        let wide = [3_u8; LEAST_UNPREDICTABLE_BYTES * 4];
        assert_eq!(
            TieValue::from_unpredictable_bytes(&wide)
                .expect("wider than the floor is admitted")
                .width(),
            LEAST_UNPREDICTABLE_BYTES * 4
        );
    }

    /// Comparison reaches the last byte. The near miss is two values agreeing
    /// everywhere except the final one, which a comparison reading a prefix
    /// would call equal.
    #[test]
    fn two_values_differing_only_in_the_last_byte_are_not_equal() {
        let mine = [1_u8; LEAST_UNPREDICTABLE_BYTES];
        let mut theirs = mine;
        theirs[LEAST_UNPREDICTABLE_BYTES - 1] ^= 0x01;

        let mine = TieValue::from_unpredictable_bytes(&mine).expect("admitted");
        let theirs = TieValue::from_unpredictable_bytes(&theirs).expect("admitted");

        assert_ne!(mine, theirs);
        assert_eq!(mine, mine.clone());
    }

    /// A value that is a prefix of another is not that other, which is the
    /// length half of the same comparison.
    #[test]
    fn a_prefix_is_not_the_value_it_is_a_prefix_of() {
        let short = [4_u8; LEAST_UNPREDICTABLE_BYTES];
        let long = [4_u8; LEAST_UNPREDICTABLE_BYTES + 1];

        let short = TieValue::from_unpredictable_bytes(&short).expect("admitted");
        let long = TieValue::from_unpredictable_bytes(&long).expect("admitted");

        assert_ne!(short, long);
        assert_ne!(long, short);
    }

    /// The value is not written out by the trait every type in this crate
    /// carries. A derived shape would put the bytes in whatever printed the
    /// structure the value travels inside.
    #[test]
    fn the_debug_shape_carries_no_byte_of_the_value() {
        let bytes = [0xab_u8; LEAST_UNPREDICTABLE_BYTES];
        let value = TieValue::from_unpredictable_bytes(&bytes).expect("admitted");

        let written = format!("{value:?}");

        assert!(!written.contains("171"), "{written}");
        assert!(!written.contains("ab"), "{written}");
        assert!(written.contains("TieValue"), "{written}");
    }

    /// An answer for an attempt nobody started is refused, which is the
    /// condition 0032 maps onto `not-authenticated` with nothing presented.
    #[test]
    fn an_answer_naming_no_started_attempt_is_refused() {
        let mut attempts = OpenAttempts::none();

        assert_eq!(
            attempts.answer(&value(1), None, "code").err(),
            Some(NoAttemptMatched)
        );
        assert_eq!(attempts.open(), 0);
    }

    /// The first answer finishes the attempt and the second finds nothing
    /// started, which is 0032's at-most-once held by the removal.
    #[test]
    fn a_second_answer_for_one_attempt_finds_nothing_started() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(2), Some("app://back")).expect("fresh");
        assert_eq!(attempts.open(), 1);

        let relayable = attempts
            .answer(&value(2), Some("app://back"), "the-code")
            .expect("the attempt was started");
        assert_eq!(relayable.as_str(), "the-code");
        assert_eq!(attempts.open(), 0);

        assert_eq!(
            attempts
                .answer(&value(2), Some("app://back"), "the-code")
                .err(),
            Some(NoAttemptMatched)
        );
    }

    /// The return address is part of the tie. The near miss is the right value
    /// with a return address one character off, which is an answer for a
    /// different attempt.
    #[test]
    fn an_answer_for_a_different_return_address_is_a_different_attempt() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(3), Some("app://back")).expect("fresh");

        assert_eq!(
            attempts
                .answer(&value(3), Some("app://backx"), "code")
                .err(),
            Some(NoAttemptMatched)
        );
        assert_eq!(
            attempts.answer(&value(3), None, "code").err(),
            Some(NoAttemptMatched)
        );
        assert_eq!(attempts.open(), 1);

        attempts
            .answer(&value(3), Some("app://back"), "code")
            .expect("the address it was started with matches");
        assert_eq!(attempts.open(), 0);
    }

    /// An attempt started without a return address is not matched by an answer
    /// naming one, which is the other direction of the same tie.
    #[test]
    fn an_attempt_with_no_return_address_is_not_matched_by_an_answer_carrying_one() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(4), None).expect("fresh");

        assert_eq!(
            attempts.answer(&value(4), Some("app://back"), "code").err(),
            Some(NoAttemptMatched)
        );

        attempts
            .answer(&value(4), None, "code")
            .expect("started without one and answered without one");
    }

    /// Two attempts are open at once and each answer finishes its own, which is
    /// what the value is for.
    #[test]
    fn one_answer_finishes_its_own_attempt_and_leaves_the_other() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(5), Some("a")).expect("fresh");
        attempts.start(value(6), Some("b")).expect("fresh");
        assert_eq!(attempts.open(), 2);

        attempts
            .answer(&value(5), Some("a"), "code")
            .expect("started");

        assert_eq!(attempts.open(), 1);
        assert_eq!(
            attempts.answer(&value(5), Some("a"), "code").err(),
            Some(NoAttemptMatched)
        );
        attempts
            .answer(&value(6), Some("b"), "code")
            .expect("the other one is untouched");
    }

    /// A repeated value is refused at the start rather than at the answer.
    #[test]
    fn a_second_attempt_under_one_open_value_is_refused() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(7), Some("a")).expect("fresh");

        assert_eq!(attempts.start(value(7), Some("b")), Err(ValueAlreadyOpen));
        assert_eq!(attempts.open(), 1);
    }

    /// A value is free again once its attempt has ended, because what is refused
    /// is two OPEN attempts sharing one value.
    #[test]
    fn a_value_may_be_started_again_once_its_attempt_has_ended() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(8), None).expect("fresh");
        attempts.cancel(&value(8)).expect("started");

        attempts.start(value(8), None).expect("no longer open");
        assert_eq!(attempts.open(), 1);
    }

    /// Cancelling ends the attempt, and a cancellation of something that is not
    /// open reaches the same answer an answer for it would.
    #[test]
    fn a_cancelled_attempt_is_ended_and_cancelling_twice_finds_nothing() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(9), Some("a")).expect("fresh");

        attempts.cancel(&value(9)).expect("started");
        assert_eq!(attempts.open(), 0);

        assert_eq!(attempts.cancel(&value(9)).err(), Some(NoAttemptMatched));
        assert_eq!(
            attempts.answer(&value(9), Some("a"), "code").err(),
            Some(NoAttemptMatched)
        );
    }

    /// Stopping ends every open attempt, and what was open afterwards answers
    /// the same as what was never started.
    #[test]
    fn stopping_ends_every_open_attempt() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(10), Some("a")).expect("fresh");
        attempts.start(value(11), None).expect("fresh");
        assert_eq!(attempts.open(), 2);

        attempts.stop();

        assert_eq!(attempts.open(), 0);
        assert_eq!(
            attempts.answer(&value(10), Some("a"), "code").err(),
            Some(NoAttemptMatched)
        );
        assert_eq!(attempts.cancel(&value(11)).err(), Some(NoAttemptMatched));
    }

    /// What the client handed back is not written out by the trait either, for
    /// 0071's rule about anything derived from a credential.
    #[test]
    fn the_relayable_debug_shape_carries_no_part_of_the_answer() {
        let mut attempts = OpenAttempts::none();
        attempts.start(value(12), None).expect("fresh");

        let relayable = attempts
            .answer(&value(12), None, "a-provider-answer")
            .expect("started");

        let written = format!("{relayable:?}");
        assert!(!written.contains("provider"), "{written}");
        assert!(written.contains("Relayable"), "{written}");
    }
}
