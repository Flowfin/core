//! Caching what was fetched.
//!
//! 0003 puts the keys, the bound, the eviction, the age of an entry and whether
//! a stale entry may be served inside the core, and puts the location of storage
//! outside it. The records are 0006, 0040, 0041, 0042, 0043, 0046, 0047 and
//! 0105, and the issues are #40 through #48 and #105.
//!
//! THIS PARAGRAPH SAID NOTHING HERE WAS WRITTEN AGAINST A DIGEST THAT DID NOT
//! EXIST YET. One does. 0041 requires a cryptographic digest for a cache key,
//! 0011 measured that the toolchain offers none, and 0103's clause for a
//! requirement a landed record already states is what admitted the one in
//! `Cargo.toml`, with the clause and what would retire it written beside the
//! entry. The construction is in [`key`].
//!
//! # What is here today
//!
//! The store interface 0040 fixes, the one failure it may report, the key type
//! it is asked about, and the capability a client asks when it wants to know
//! whether anything survives the process.
//!
//! THIS SECTION SAID NOTHING IN THIS TREE CACHES ANYTHING AND THAT THE BOUND AND
//! THE EVICTION WERE #42. The bounds and the eviction are in [`bound`], which is
//! #42 and #54 landed rather than pending, and a client that supplies a store
//! now gets bookkeeping over it: two tiers with their own bounds and their own
//! use orders so that neither can evict the other, bounds counted on bytes the
//! core counted, eviction of the least recently used entry in a tier before a
//! write that would exceed that tier's bound, a read in flight that eviction may
//! not reach, writing suspended rather than the core evicting its own entries
//! when the device is full, and artwork released so that a refused metadata
//! write can be attempted once more.
//!
//! What is still absent is the rest of the sentence and it is unchanged. WHAT is
//! cached at all is 0006 and #43, the cold-start path is #46, and the index that
//! survives a restart is #105. Nothing here decides any of those.

pub mod bound;
pub mod key;

/// The name one cache entry is kept under.
///
/// 0040 asks a store to be given an opaque key and to learn nothing from it: no
/// parsing, no structure, no path layout. What a key is MADE of is 0041, and
/// with it the guarantee that two servers and two people on one device cannot
/// read each other's entries.
///
/// THIS PARAGRAPH SAID THE DERIVATION WAS NOT IN THIS TREE. It is, in [`key`],
/// and [`EntryKey::derive`] is the call that performs it.
/// [`EntryKey::from_derived_key`] stays beside it and still checks nothing about
/// what it is handed, exactly as [`crate::session::SecretName`] does for the
/// other store: a client's own test double and the bookkeeping in [`bound`] both
/// need a key that stands for one, and asking either to run a derivation would
/// make a key a thing only the core can name.
///
/// The two spaces must not collide. 0033 requires a secret store name and a
/// cache key to be distinguishable, so whatever tag 0041 puts at the front of a
/// key is what separates them. [`key`] carries the cache space's tag and a test
/// that the two spaces differ; the secret store's own tag arrives with #33's
/// naming and is not invented there.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryKey {
    key: String,
}

impl EntryKey {
    /// Takes a key a derivation produced.
    ///
    /// This does not derive anything and does not judge what it is handed. See
    /// the type's own documentation for where the derivation lives and for what
    /// that means for this tree today.
    #[must_use]
    pub const fn from_derived_key(key: String) -> Self {
        Self { key }
    }

    /// The key, as the store will use it.
    ///
    /// A client's implementation needs the bytes it will key its own platform
    /// facility on, so this is public rather than crate-private.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.key
    }
}

/// The one failure a byte store may report.
///
/// 0040 admits exactly one: a full device, a store the platform closed
/// underneath the core, a permission withdrawn while the application was in the
/// background. The core does not tell those apart, because its answer is the
/// same for all of them, and what that answer is belongs to #42.
///
/// WHAT THIS CARRIES AND WHAT IT DELIBERATELY DOES NOT. 0004's
/// `storage-unavailable` carries which store it was and whether the failure was
/// a read or a write. Both are facts the core already holds at the moment it
/// calls, so asking a client's implementation to supply them would be asking it
/// to repeat what the caller knows, and 0037 requires the value of the failure
/// vocabulary to be built at one mapping point and nowhere else. That mapping
/// point does not exist yet; it is #37, and [`crate::failure`] holds no type
/// today.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageUnavailable;

/// The place a client lets the core put bytes.
///
/// The core is told where to write and never asks the platform, which is 0003's
/// sentence and 0040's record. #40 is the issue that decides what this asks of a
/// client.
///
/// Thread safety, from 0009: may be called from either lane and concurrently,
/// including for two entries at once. A client that assumed single-threaded
/// access would corrupt its own storage rather than producing a failure the core
/// could report, which is why the statement is here rather than left implied.
/// This is the deliberate opposite of [`crate::session::SecretStore`], and 0040
/// gives the reason: a cache is read and written constantly and by both lanes,
/// while a keychain call is rare.
///
/// # Four operations and no fifth
///
/// There is no listing, no iteration, no prefix scan, no transaction and no
/// expiry. 0040 says what pays for the absence of the first: eviction in #42
/// needs to know what is there and in what order, and with no way to ask the
/// store that is the core's own bookkeeping. It is a real cost and it lands on
/// one issue rather than on every client. The alternative spreads it - a store
/// that enumerates cheaply over a directory, expensively over a platform
/// key-value facility, and in a different order on each - so the eviction rule
/// 0006 promises is one rule becomes a rule per client.
///
/// # What an implementation has to be
///
/// A place bytes survive, whose location the client chose. A directory, a
/// database, a platform key-value facility. 0040 asks nothing of it that only
/// some of those can do, and it is never asked to understand a key.
///
/// A client with nowhere to put bytes supplies no store at all, which is
/// [`CacheStorage::ForTheLifeOfTheProcess`], rather than inventing a location on
/// the core's behalf.
///
/// # What the store never sees
///
/// No secret. The token and anything else that authenticates go through the
/// separate interface in #33, and 0006 states the reason the two are separate
/// rather than one store with a convention: with two interfaces the proof is
/// that the cache store never receives the token, which is a thing a test can
/// watch, and #48 is where it is watched. Nothing in this tree watches it today,
/// because nothing in this tree holds a token.
///
/// # Blocking, and why these are ordinary calls
///
/// An implementation MAY BLOCK, and the core holds no lock of its own across the
/// call, so a slow store is a slow store rather than a stopped core. 0009 makes
/// every call that can wait asynchronous *at the interface a client calls*, and
/// this is the interface the core calls, on the waiting lane and never on the
/// caller's thread.
///
/// THE RECORD SAYS BOTH THINGS AND THIS IS THE READING TAKEN. 0040's opening
/// sentence calls the four operations asynchronous; its `## Threads` section
/// says an implementation may block and that a store over a filesystem does not
/// need to find an asynchronous file interface on every platform to be correct.
/// The second is the one that describes THIS interface: what the first sentence
/// is about is the cache call a client makes, which is asynchronous because 0009
/// says so, and which is not built here. A shape that returned a handle from
/// these four would be promising something the same record gives away two
/// sections later. [`crate::session::SecretStore`] resolved the identical
/// tension the same way.
pub trait ByteStore: Send + Sync {
    /// Reads the entry kept under a key.
    ///
    /// `Ok(None)` is an absence and is not a failure. A first run, an entry
    /// never written and an entry already removed all produce it, and 0006
    /// already gives a caller three states for that reason: fresh, stale and
    /// absent. A store that reported absence as a failure would make the
    /// cold-start path in #46 look like something going wrong.
    ///
    /// # Errors
    ///
    /// [`StorageUnavailable`] where the platform facility could not be read.
    /// 0040 fixes what the core does with that: the entry is absent, the network
    /// answers, and the call that wanted it does not fail.
    fn read(&self, key: &EntryKey) -> Result<Option<Vec<u8>>, StorageUnavailable>;

    /// Writes an entry, replacing whatever was there.
    ///
    /// # Errors
    ///
    /// [`StorageUnavailable`] where the platform facility could not be written.
    /// A FAILED WRITE NEVER FAILS THE CALL THAT CAUSED IT. Somebody asking for a
    /// library list gets the library list; the cache is an accelerator, and a
    /// device that has run out of room is not a reason to show an empty screen
    /// in front of a working server and a valid session.
    fn write(&self, key: &EntryKey, bytes: &[u8]) -> Result<(), StorageUnavailable>;

    /// Removes the entry kept under a key.
    ///
    /// Removing one that is not there succeeds, for 0040's reason: a caller that
    /// has to ask first has a race to lose.
    ///
    /// # Errors
    ///
    /// [`StorageUnavailable`] where the platform facility could not be reached.
    fn remove(&self, key: &EntryKey) -> Result<(), StorageUnavailable>;

    /// How much is held, in bytes, as the store counts them.
    ///
    /// As the store counts them rather than as the core does. A directory
    /// implementation answers with what the platform says the files occupy,
    /// which is not the sum of the byte counts written, and 0040 asks for the
    /// store's own number because that is the one that fills a device.
    ///
    /// # Errors
    ///
    /// [`StorageUnavailable`] where the platform facility could not be asked.
    fn held_bytes(&self) -> Result<u64, StorageUnavailable>;
}

/// What a client is told about where cached bytes rest.
///
/// 0040 requires the answer to be askable rather than implied. A core that
/// quietly behaved as though it had a cache would let a client measure the cold
/// start in #46 against a number that never happens on a real device, and would
/// leave the operator documentation in #74 unable to answer what is stored on
/// the device, which in this configuration is nothing.
///
/// It is a call that cannot wait in the terms of 0009, so it is an ordinary
/// answer computed from state the core already holds. The public call that
/// returns it from a running core arrives with #115; [`CacheStorage::of`] is the
/// whole of the answer today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStorage {
    /// A store the client supplied holds the entries, so what was cached
    /// survives the process.
    KeptByTheClient,
    /// No store was supplied. The core works, nothing refuses to run, no call
    /// starts failing, and 0006's guarantee that an answer says whether it is
    /// fresh, stale or absent holds unchanged. What changes is that everything
    /// is absent again after the process ends.
    ///
    /// What is held in the meantime is what the core can hold in memory, under
    /// the same bound and the same eviction as a store-backed cache, which is
    /// #42 and is not built. THAT IS NOT THE SAME THING AS AN IN-MEMORY STORE A
    /// CLIENT SUPPLIED, and 0040 keeps them apart because they look identical
    /// from outside: a supplied store is under test as a store and can be made
    /// to fail, to be full, or to answer slowly, while this is the absence of
    /// one.
    ForTheLifeOfTheProcess,
}

impl CacheStorage {
    /// The answer for a core holding this store, or holding none.
    #[must_use]
    pub const fn of(store: Option<&dyn ByteStore>) -> Self {
        match store {
            Some(_) => Self::KeptByTheClient,
            None => Self::ForTheLifeOfTheProcess,
        }
    }
}

#[cfg(test)]
mod tests {
    //! The in-memory store 0040 asks the suite for, and what it is used to
    //! prove.
    //!
    //! It keeps what it is given in a vector and has no other member, so it
    //! cannot reach a disk. That is a property of the type rather than something
    //! a test observes, and what refuses a filesystem route anywhere under
    //! `src/` is the `no-filesystem-access` rule in `.github/invariants/rules`,
    //! which is a check rather than a test. Both are stated here so that neither
    //! is read as the other.
    //!
    //! It is a store under test AS A STORE, which 0040 separates from a core
    //! with no store: it can be made to fail and it can be asked what it holds.
    //! The absence of a store is [`CacheStorage::of`] with `None`, and the two
    //! have their own tests below.
    //!
    //! It is behind `#[cfg(test)]` rather than published, so nothing a client
    //! links can reach it. The conformance suite in #76 asks the same questions
    //! from a client's side and is not built.

    use super::{ByteStore, CacheStorage, EntryKey, StorageUnavailable};
    use std::sync::Mutex;

    /// What the double does when it is asked.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Answering {
        /// The platform facility is there.
        Normally,
        /// The platform facility could not be reached, which is the only failure
        /// 0040 admits.
        Unavailable,
    }

    /// A byte store that keeps entries in memory and nowhere else.
    struct InMemory {
        /// Held under a lock because 0009 has this interface called from either
        /// lane and concurrently, which is the opposite of what it says about
        /// the secret store. Here the lock is what the record requires rather
        /// than what the bound costs.
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

    impl ByteStore for InMemory {
        fn read(&self, key: &EntryKey) -> Result<Option<Vec<u8>>, StorageUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(StorageUnavailable);
            }
            Ok(self
                .entries()
                .iter()
                .find(|(kept, _)| kept == key.as_str())
                .map(|(_, bytes)| bytes.clone()))
        }

        fn write(&self, key: &EntryKey, bytes: &[u8]) -> Result<(), StorageUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(StorageUnavailable);
            }
            let mut held = self.entries();
            held.retain(|(kept, _)| kept != key.as_str());
            held.push((key.as_str().to_owned(), bytes.to_vec()));
            Ok(())
        }

        fn remove(&self, key: &EntryKey) -> Result<(), StorageUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(StorageUnavailable);
            }
            self.entries().retain(|(kept, _)| kept != key.as_str());
            Ok(())
        }

        fn held_bytes(&self) -> Result<u64, StorageUnavailable> {
            if self.answering == Answering::Unavailable {
                return Err(StorageUnavailable);
            }
            Ok(self
                .entries()
                .iter()
                .map(|(_, bytes)| bytes.len() as u64)
                .sum())
        }
    }

    fn a_key() -> EntryKey {
        EntryKey::from_derived_key("0000000000000000".to_owned())
    }

    fn another_key() -> EntryKey {
        EntryKey::from_derived_key("1111111111111111".to_owned())
    }

    #[test]
    fn an_entry_written_under_a_key_comes_back_under_that_key() {
        let store = InMemory::new(Answering::Normally);
        store
            .write(&a_key(), b"a-library-list")
            .expect("the store answers");
        assert_eq!(
            store.read(&a_key()).expect("the store answers"),
            Some(b"a-library-list".to_vec())
        );
    }

    #[test]
    fn writing_under_a_key_replaces_whatever_was_there() {
        let store = InMemory::new(Answering::Normally);
        store
            .write(&a_key(), b"the-first")
            .expect("the store answers");
        store
            .write(&a_key(), b"the-second")
            .expect("the store answers");
        assert_eq!(
            store.read(&a_key()).expect("the store answers"),
            Some(b"the-second".to_vec())
        );
        assert_eq!(store.held_bytes().expect("the store answers"), 10);
    }

    #[test]
    fn a_key_nothing_was_written_under_answers_with_an_absence() {
        let store = InMemory::new(Answering::Normally);
        store
            .write(&a_key(), b"a-library-list")
            .expect("the store answers");
        assert_eq!(store.read(&another_key()).expect("the store answers"), None);
    }

    #[test]
    fn removing_a_key_nothing_was_written_under_succeeds() {
        let store = InMemory::new(Answering::Normally);
        store.remove(&a_key()).expect("the store answers");
    }

    #[test]
    fn what_was_removed_is_an_absence_rather_than_an_empty_entry() {
        let store = InMemory::new(Answering::Normally);
        store
            .write(&a_key(), b"a-library-list")
            .expect("the store answers");
        store.remove(&a_key()).expect("the store answers");
        assert_eq!(store.read(&a_key()).expect("the store answers"), None);
        assert_eq!(store.held_bytes().expect("the store answers"), 0);
    }

    /// The rule 0040 states most explicitly, at the one place a type can carry
    /// it: a store that could not answer and a store that has nothing are two
    /// different values, so a caller cannot reach the second by ignoring the
    /// first. Collapsed, a device locked in the background reads as a cold cache
    /// and the core refetches everything.
    #[test]
    fn a_store_that_could_not_answer_is_not_an_absence() {
        let unavailable = InMemory::new(Answering::Unavailable);
        assert_eq!(unavailable.read(&a_key()), Err(StorageUnavailable));

        let empty = InMemory::new(Answering::Normally);
        assert_eq!(empty.read(&a_key()), Ok(None));

        assert_ne!(unavailable.read(&a_key()), empty.read(&a_key()));
    }

    /// Every operation may report the one failure, and none of them may report
    /// anything else. The compiler holds the second half; this holds the first,
    /// because an implementation that could only fail on read would pass every
    /// other test in this file.
    #[test]
    fn every_operation_can_report_the_one_failure_the_record_admits() {
        let store = InMemory::new(Answering::Unavailable);
        assert_eq!(store.read(&a_key()), Err(StorageUnavailable));
        assert_eq!(store.write(&a_key(), b"bytes"), Err(StorageUnavailable));
        assert_eq!(store.remove(&a_key()), Err(StorageUnavailable));
        assert_eq!(store.held_bytes(), Err(StorageUnavailable));
    }

    #[test]
    fn what_is_held_is_counted_as_the_store_counts_it() {
        let store = InMemory::new(Answering::Normally);
        assert_eq!(store.held_bytes().expect("the store answers"), 0);
        store
            .write(&a_key(), b"1234567890")
            .expect("the store answers");
        store
            .write(&another_key(), b"12345")
            .expect("the store answers");
        assert_eq!(store.held_bytes().expect("the store answers"), 15);
    }

    #[test]
    fn a_core_handed_no_store_says_the_cache_lives_for_the_process() {
        assert_eq!(CacheStorage::of(None), CacheStorage::ForTheLifeOfTheProcess);
    }

    #[test]
    fn a_core_handed_a_store_says_the_client_keeps_it() {
        let store = InMemory::new(Answering::Normally);
        assert_eq!(
            CacheStorage::of(Some(&store)),
            CacheStorage::KeptByTheClient
        );
    }

    /// A supplied store that keeps its bytes in memory and a core with no store
    /// look identical from outside, and 0040 keeps them apart. The capability is
    /// where the difference is visible, and this is the assertion that would go
    /// red if the two were collapsed.
    #[test]
    fn an_in_memory_store_is_not_the_absence_of_a_store() {
        let store = InMemory::new(Answering::Normally);
        assert_ne!(CacheStorage::of(Some(&store)), CacheStorage::of(None));
    }
}
