//! Holding a session.
//!
//! 0003 puts acquiring a session, renewing it, holding more than one at a time,
//! and handing the secret to a store the client supplies inside the core. The
//! records are 0005, 0030, 0031, 0032, 0033, 0034, 0036 and 0114, and the issues
//! are #30 through #36 and #114.
//!
//! [`device`] holds what 0036 decides. It is here rather than beside the session
//! because 0005 fixes a session as one server, one account and one device
//! together, so the identity is part of what names a session rather than
//! something a session acquires. None of it is state the core holds: the client
//! keeps the identifier and the name, and the core owns the shape of the
//! capability description and fills none of it in.

pub mod device;

/// One signed-in session against one server.
///
/// Thread safety, from 0009: safe from any thread. Calling on a session while
/// another thread signs it out is defined rather than racing: the call either
/// goes out under a valid token or fails with the signed-out outcome, and never
/// goes out under a token that has been discarded.
///
/// Signing out and holding several at once is #114.
#[derive(Debug)]
pub struct Session {
    _private: (),
}

/// The name one session's secret is kept under.
///
/// 0033 fixes what a name identifies: one session, which 0005 fixes as the
/// server, the account and the device together. It is a derived label rather
/// than any of those three written out, because a keychain item is protected in
/// its value and frequently not in its label, and a label appears in the
/// platform's own listing, in a device backup, and in the view a person opens to
/// see what an application has stored. A label reading as an address and an
/// account is the leak 0072 describes, written by the part of the system that is
/// supposed to be the careful one.
///
/// THE DERIVATION IS NOT IN THIS TREE AND THIS TYPE DOES NOT PERFORM ONE. 0033
/// takes it from #41, which builds a cache key and is not built, and adds one
/// requirement of its own: a secret store name and a cache key are separate
/// spaces and must not be able to collide, so whatever tag #41 puts at the front
/// of a key distinguishes them. [`SecretName::from_derived_label`] takes a label
/// that derivation produced and checks nothing about it, so until #41 lands
/// nothing here produces a name meeting 0101's rule. That is a statement about
/// this tree rather than about the interface.
///
/// A name is ordinary data. It is the one thing about a session that a platform
/// will show a person, so it is not redacted here, and everything that makes it
/// safe to show is the derivation's doing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretName {
    label: String,
}

impl SecretName {
    /// Takes a label a derivation produced.
    ///
    /// This does not derive anything and does not judge what it is handed. See
    /// the type's own documentation for where the derivation lives and for what
    /// that means for this tree today.
    #[must_use]
    pub const fn from_derived_label(label: String) -> Self {
        Self { label }
    }

    /// The label, as the store will use it.
    ///
    /// A client's implementation needs the bytes it will key its own platform
    /// facility on, so this is public rather than crate-private.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.label
    }
}

/// The one failure a secret store may report.
///
/// 0033 admits exactly one: a locked device, a keychain the platform closed
/// while the application was in the background, a person who declined the
/// permission, an item the platform refused to write. The core does not tell
/// those apart, because its answer is the same for all of them.
///
/// WHAT THIS CARRIES AND WHAT IT DELIBERATELY DOES NOT. 0004's
/// `storage-unavailable` carries which store it was and whether the failure was
/// a read or a write. Both of those are facts the core already holds at the
/// moment it calls, so asking a client's implementation to supply them would be
/// asking it to repeat what the caller knows, and 0037 requires the value of the
/// failure vocabulary to be built at one mapping point and nowhere else. So a
/// store says only that it could not answer, and the mapping onto the vocabulary
/// is the core's. THIS SENTENCE SAID THAT MAPPING POINT DOES NOT EXIST YET. It
/// does: [`crate::failure::Failure::from_secret_store`] is where this value
/// becomes `storage-unavailable`, and it is the caller that says whether the
/// call was a read or a write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecretStoreUnavailable;

/// The place a client keeps a session secret.
///
/// The core never chooses where a secret is kept. 0033 is the record and #33 is
/// the issue that decides what this asks of a client.
///
/// # What an implementation has to be
///
/// The platform's own protected storage, which 0033 states as one property: the
/// facility whose contents are protected by the platform rather than by file
/// permissions the application chose. On Apple platforms the keychain. On
/// Android the platform keystore, reached directly or through the storage
/// facility backed by it. On Windows the credential manager. On desktop Linux
/// the freedesktop secret service, where a session provides one. On a television
/// the answer differs per vendor, and the honest position is that a client author
/// checks rather than assumes. Those facilities are named in 0033 as a claim
/// rather than as a measurement: nothing in this tree reads a platform and no
/// command in this repository produces that list.
///
/// A client with none of them supplies no store at all, which is
/// [`SecretStorage::ForTheLifeOfTheProcess`], rather than inventing one. There is
/// no fallback to a file and there will not be one: 0005 and 0101 both refuse a
/// file the core chose the location of, with or without obfuscation, because a
/// key the core manages lives on the same device as what it protects.
///
/// # Three operations and no fourth
///
/// There is no listing, no enumeration and no iteration, and 0033 says what
/// replaces it: a client that wants to restore a session at start already knows
/// which sessions it configured, because 0005 makes the whole of a session except
/// the token ordinary data that may be cached and shown. The keychain is asked
/// one question about one name and is never asked what it holds. What that costs
/// is an orphan, which 0033 states and prices.
///
/// # Threads
///
/// Thread safety, from 0009: called from the waiting lane only, and never
/// concurrently for one session, so a client may implement it without locking.
/// This is the deliberate opposite of [`crate::cache::ByteStore`], and the reason
/// is that a keychain call is rare and a platform keychain is the place a client
/// is most likely to write something naive.
///
/// The `Send + Sync` bound is here because the lane that calls it is not the
/// thread that supplied it. It is not a licence to call it concurrently, and
/// 0009's sentence above is the rule.
///
/// An implementation MAY BLOCK, and the core holds no lock of its own across the
/// call, so a keychain that puts up the platform's own authentication is a slow
/// store rather than a stopped core. That is why these are ordinary calls rather
/// than the completion-based shape 0009 fixes for the core's own public surface:
/// 0009 makes every call that can wait asynchronous *at the interface a client
/// calls*, and this is the interface the core calls, on a thread the core owns
/// and never on the caller's. A call that a record permits to block cannot also
/// be one that returns at once, so a shape that returned a handle here would be
/// promising something 0033's own thread section gives away in the next sentence.
/// The asynchronous surface that wraps these is #115 and #27 and is not built.
pub trait SecretStore: Send + Sync {
    /// Keeps a secret under a name, replacing whatever was there.
    ///
    /// # Errors
    ///
    /// [`SecretStoreUnavailable`] where the platform facility could not be
    /// written. 0033 fixes what the core does with that: a write that fails does
    /// not fail the sign-in that produced the token, and the session continues in
    /// memory as though no store had been supplied.
    fn keep(&self, name: &SecretName, secret: &[u8]) -> Result<(), SecretStoreUnavailable>;

    /// Reads the secret kept under a name.
    ///
    /// `Ok(None)` is an absence and is not a failure. A first run, a person who
    /// has never signed in, and a session whose secret was forgotten all produce
    /// it, and the answer is that there is no session to restore.
    ///
    /// # Errors
    ///
    /// [`SecretStoreUnavailable`] where the platform facility could not be read.
    /// THE DIFFERENCE BETWEEN THAT AND AN ABSENCE IS THE RULE 0033 STATES MOST
    /// EXPLICITLY, because the convenient handling of a failed read is to treat
    /// it as empty and carry on, and on a device locked at the moment of a
    /// background start that quietly replaces a working session's secret. An
    /// absence means sign in again. A failure means the secret may still be
    /// there, and a new one may not be written under that name until a read
    /// succeeds.
    fn read(&self, name: &SecretName) -> Result<Option<Vec<u8>>, SecretStoreUnavailable>;

    /// Forgets the secret kept under a name.
    ///
    /// Forgetting one that is not there succeeds, for 0040's reason: a caller
    /// that has to ask first has a race to lose.
    ///
    /// # Errors
    ///
    /// [`SecretStoreUnavailable`] where the platform facility could not be
    /// reached.
    fn forget(&self, name: &SecretName) -> Result<(), SecretStoreUnavailable>;
}

/// What a client is told about where a session secret rests.
///
/// 0033 requires the answer to be askable rather than implied: a client that
/// cannot ask cannot tell an operator why they sign in every morning, and it
/// cannot decide to prompt for sign-in at a moment of its own choosing rather
/// than at the moment somebody presses play.
///
/// It is a call that cannot wait in the terms of 0009, so it is an ordinary
/// answer computed from state the core already holds. The public call that
/// returns it from a running core arrives with #115, which is what decides what
/// creating a core means; [`SecretStorage::of`] is the whole of the answer today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStorage {
    /// A store the client supplied holds the secret, so a session outlives the
    /// process.
    KeptByTheClient,
    /// No store was supplied. The core works, sign-in succeeds, every call that
    /// needs a token has one, and nothing starts failing. What changes is that
    /// the session ends with the process and the person signs in again next
    /// time.
    ForTheLifeOfTheProcess,
}

impl SecretStorage {
    /// The answer for a core holding this store, or holding none.
    #[must_use]
    pub const fn of(store: Option<&dyn SecretStore>) -> Self {
        match store {
            Some(_) => Self::KeptByTheClient,
            None => Self::ForTheLifeOfTheProcess,
        }
    }
}

#[cfg(test)]
mod tests {
    //! The double 0033 asks the suite for, and what it is used to prove.
    //!
    //! It keeps what it is given in a vector and has no other member, so it
    //! cannot reach a disk. That is a property of the type rather than something
    //! a test observes, and what refuses a filesystem route anywhere under
    //! `src/` is the `no-filesystem-access` rule in `.github/invariants/rules`,
    //! which is a check rather than a test. Both are stated here so that neither
    //! is read as the other.
    //!
    //! It is behind `#[cfg(test)]` rather than published, so nothing a client
    //! links can reach it. The conformance suite in #76 asks the same questions
    //! from a client's side and is not built.

    use super::{SecretName, SecretStorage, SecretStore, SecretStoreUnavailable};
    use std::sync::Mutex;

    /// What the double does when it is asked.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Answering {
        /// The platform facility is there.
        Normally,
        /// The platform facility could not be reached, which is the only failure
        /// 0033 admits.
        Unavailable,
    }

    /// A secret store that keeps secrets in memory and nowhere else.
    struct InMemory {
        /// Held under a lock because the trait's bound requires the type to be
        /// safe from any thread. 0009 says the core calls this from one lane and
        /// never concurrently for one session, so the lock is what the bound
        /// costs rather than what the record requires.
        held: Mutex<Vec<(String, Vec<u8>)>>,
        answering: Answering,
    }

    impl InMemory {
        fn new(answering: Answering) -> Self {
            Self {
                held: Mutex::new(Vec::new()),
                answering,
            }
        }

        fn entries(&self) -> std::sync::MutexGuard<'_, Vec<(String, Vec<u8>)>> {
            self.held
                .lock()
                .expect("the fixture holds no poisoned lock")
        }
    }

    impl SecretStore for InMemory {
        fn keep(&self, name: &SecretName, secret: &[u8]) -> Result<(), SecretStoreUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(SecretStoreUnavailable);
            }
            let mut held = self.entries();
            held.retain(|(kept, _)| kept != name.as_str());
            held.push((name.as_str().to_owned(), secret.to_vec()));
            Ok(())
        }

        fn read(&self, name: &SecretName) -> Result<Option<Vec<u8>>, SecretStoreUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(SecretStoreUnavailable);
            }
            Ok(self
                .entries()
                .iter()
                .find(|(kept, _)| kept == name.as_str())
                .map(|(_, secret)| secret.clone()))
        }

        fn forget(&self, name: &SecretName) -> Result<(), SecretStoreUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(SecretStoreUnavailable);
            }
            self.entries().retain(|(kept, _)| kept != name.as_str());
            Ok(())
        }
    }

    fn a_name() -> SecretName {
        SecretName::from_derived_label("0000000000000000".to_owned())
    }

    fn another_name() -> SecretName {
        SecretName::from_derived_label("1111111111111111".to_owned())
    }

    #[test]
    fn a_secret_kept_under_a_name_comes_back_under_that_name() {
        let store = InMemory::new(Answering::Normally);
        store
            .keep(&a_name(), b"a-token")
            .expect("the store answers");
        assert_eq!(
            store.read(&a_name()).expect("the store answers"),
            Some(b"a-token".to_vec())
        );
    }

    #[test]
    fn keeping_under_a_name_replaces_whatever_was_there() {
        let store = InMemory::new(Answering::Normally);
        store
            .keep(&a_name(), b"the-first")
            .expect("the store answers");
        store
            .keep(&a_name(), b"the-second")
            .expect("the store answers");
        assert_eq!(
            store.read(&a_name()).expect("the store answers"),
            Some(b"the-second".to_vec())
        );
    }

    #[test]
    fn a_name_nothing_was_kept_under_answers_with_an_absence() {
        let store = InMemory::new(Answering::Normally);
        store
            .keep(&a_name(), b"a-token")
            .expect("the store answers");
        assert_eq!(
            store.read(&another_name()).expect("the store answers"),
            None
        );
    }

    #[test]
    fn forgetting_a_name_nothing_was_kept_under_succeeds() {
        let store = InMemory::new(Answering::Normally);
        store.forget(&a_name()).expect("the store answers");
    }

    #[test]
    fn what_was_forgotten_is_an_absence_rather_than_an_empty_secret() {
        let store = InMemory::new(Answering::Normally);
        store
            .keep(&a_name(), b"a-token")
            .expect("the store answers");
        store.forget(&a_name()).expect("the store answers");
        assert_eq!(store.read(&a_name()).expect("the store answers"), None);
    }

    /// The rule 0033 states most explicitly, at the one place a type can carry
    /// it: a store that could not answer and a store that has nothing are two
    /// different values, so a caller cannot reach the second by ignoring the
    /// first.
    #[test]
    fn a_store_that_could_not_answer_is_not_an_absence() {
        let unavailable = InMemory::new(Answering::Unavailable);
        assert_eq!(unavailable.read(&a_name()), Err(SecretStoreUnavailable));

        let empty = InMemory::new(Answering::Normally);
        assert_eq!(empty.read(&a_name()), Ok(None));

        assert_ne!(unavailable.read(&a_name()), empty.read(&a_name()));
    }

    #[test]
    fn a_core_handed_no_store_says_the_session_lives_for_the_process() {
        assert_eq!(
            SecretStorage::of(None),
            SecretStorage::ForTheLifeOfTheProcess
        );
    }

    #[test]
    fn a_core_handed_a_store_says_the_client_keeps_it() {
        let store = InMemory::new(Answering::Normally);
        assert_eq!(
            SecretStorage::of(Some(&store)),
            SecretStorage::KeptByTheClient
        );
    }
}
