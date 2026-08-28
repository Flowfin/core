//! The envelope every cache entry is written inside, and what a read does with
//! one that does not check out.
//!
//! 0105 is the record and #105 is the issue. Everything the core writes through
//! the store in 0040 is wrapped in an envelope of the core's own carrying a
//! format version, the kind of the payload, the payload's length and a digest
//! over it. A read parses the envelope, checks the version, checks the kind,
//! checks the length and checks the digest before any part of the payload is
//! looked at, and an entry failing any of them is removed where it was found and
//! answered as absent.
//!
//! # Why the store is not asked instead
//!
//! 0040 gives the store four operations and no fifth, and none of them says
//! whether a write finished. The store is the client's, it may be a directory, a
//! database or a platform facility, and the atomicity of a write differs across
//! all three. So whether the bytes that came back are the bytes that went in is
//! answered by the bytes themselves. 0040 named a fifth operation as its own
//! reversal condition, and that condition has not happened here.
//!
//! # The order of the checks, and the one the record does not number
//!
//! 0105 names four checks and gives their order: parse, version, length, digest.
//! The kind is a fifth reading and it is here because that record's own sentence
//! about the field requires it - a payload is never handed to a reader for a
//! different kind - and a field nothing compares is a field that does nothing.
//! It sits immediately after the version, because both are questions about what
//! wrote the entry rather than about whether the bytes survived, and it is
//! counted under its own name so a reader can tell the two apart.
//!
//! # What the digest does not buy
//!
//! It is not authentication. Anything that can write the store can write a
//! matching digest, which is 0105's own sentence and 0101's reason for treating
//! every byte read back out of the store as untrusted. The envelope detects an
//! entry that was damaged, never one that was chosen, and parsing a payload
//! whose digest matched is still parsing untrusted input.
//!
//! # What is not here
//!
//! The queue in 0047. That record's entries are things a person did rather than
//! copies of what a server holds, and 0105 gives them the same envelope with
//! three different answers: a drop is reported at `failure` rather than
//! `notice`, a drain steps over a bad entry instead of stopping at it, and a
//! counter that fails its own envelope is rebuilt rather than emptying the
//! queue. Nothing in this tree holds a queue, so none of the three is written
//! here, and #47 is where they belong.

use core::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::EntryKey;
use super::bound::{Cached, Tier, TieredCache};
use super::freshness::EntryKind;
use crate::diagnostics::redaction::FieldName;
use crate::diagnostics::{Diagnostics, EventName, Field, FieldValue, Severity};

/// The version of the envelope and of the payload shape inside it.
///
/// One number for both, which is 0105: the envelope's own shape and the shape of
/// what it wraps move together, because a build that could read the second
/// without the first would be guessing about the part it cannot see.
///
/// A version that is not this one is discarded in both directions, including one
/// a newer build wrote. Downgrading a client is ordinary, and a build reading a
/// shape defined after it was written would be guessing in the other direction.
pub const FORMAT_VERSION: u32 = 1;

/// How many bytes of envelope sit in front of a payload.
///
/// Four for the version, one for the kind, eight for the length and thirty-two
/// for the digest. It is written out here so that a reader of the sizes a store
/// holds can account for it, and so that the two places below that index into a
/// header cannot disagree about where it ends.
pub const HEADER_WIDTH: usize = VERSION_WIDTH + KIND_WIDTH + LENGTH_WIDTH + DIGEST_WIDTH;

const VERSION_WIDTH: usize = size_of::<u32>();
const KIND_WIDTH: usize = 1;
const LENGTH_WIDTH: usize = size_of::<u64>();
/// Thirty-two, which is the width of the digest below. It is a constant here
/// rather than read off the digest type because the header layout has to be a
/// compile-time number, and the condition beside the conditions below is what
/// refuses a disagreement between the two.
const DIGEST_WIDTH: usize = 32;

/// Which of the readings a dropped entry failed.
///
/// It is a plain tag rather than a value carrying what was found, because its
/// other job is to be counted: 0105 asks for a standing count separated by which
/// check failed, and a tag carrying a number would make two drops of the same
/// kind two different counters. What was found is put on the event instead,
/// where 0105 asks for it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhichCheckFailed {
    /// The bytes are not an envelope at all: too short to hold a header, or a
    /// kind byte this build does not know.
    Malformed,
    /// The envelope parsed and its version is not the one this build writes.
    Version,
    /// The envelope parsed and names a different kind from the one the reader
    /// asked for.
    Kind,
    /// The payload is not the length the envelope states, which is ordinary
    /// truncation and is caught before anything is allocated for the payload.
    Length,
    /// The payload is the stated length and is not the payload the digest was
    /// taken over. This is the case the length misses: a write replaced in place
    /// by a shorter one, leaving the tail of the previous entry behind it, so
    /// the bytes are the stated length and are two entries end to end.
    Digest,
}

impl WhichCheckFailed {
    /// Every reading, so that a caller reads the set out of the crate rather
    /// than keeping a copy of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Malformed,
            Self::Version,
            Self::Kind,
            Self::Length,
            Self::Digest,
        ]
    }

    /// The reading as it is reported.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::Version => "version",
            Self::Kind => "kind",
            Self::Length => "length",
            Self::Digest => "digest",
        }
    }

    /// Where this reading's count is held.
    const fn at(self) -> usize {
        match self {
            Self::Malformed => 0,
            Self::Version => 1,
            Self::Kind => 2,
            Self::Length => 3,
            Self::Digest => 4,
        }
    }
}

/// The identity of a dropped entry, declared once beside the fields it carries.
///
/// 0105 puts this at `notice` rather than at `failure`, because nothing the
/// caller asked for failed: what the caller gets is absence, which 0006 already
/// gives it as one of three states, and the fetch that follows is an ordinary
/// fetch.
const ENTRY_DROPPED: EventName = EventName::declared("cache.entry-dropped");

/// The key, under the treatment 0071 gives it.
///
/// Reduced rather than carried whole, and 0105 asks for exactly that: the key is
/// not a field, and what identifies the entry is the correlator. A key is
/// derived from an address, an account and a device under 0041, so two people
/// running the same build against the same server do not hold the same one,
/// which is 0068's question answered against this field.
const ENTRY: FieldName = FieldName::reduced("entry");

/// What was expected, which check failed, and the version that was there.
///
/// All three are carried whole. A kind is one of five values this build
/// declares, a reading is one of five names, and the version is a number this
/// build or another build of this core wrote; none of them can differ between
/// two people running the same build against the same server.
const ENTRY_KIND: FieldName = FieldName::carried_whole("entry-kind");
const CHECK: FieldName = FieldName::carried_whole("check");
const VERSION_FOUND: FieldName = FieldName::carried_whole("version-found");

/// How many entries this run dropped, separated by which reading failed.
///
/// 0105 asks for this beside the event and gives the reason both are owed. An
/// event reaches a client that was listening at that moment; a count reaches one
/// that was not. A cache that empties itself on every start presents as a slow
/// network and can stay that way for a long time, and the difference between one
/// drop after a power cut and four hundred drops on every start is the whole
/// diagnosis.
///
/// Thread safety, from 0009: safe from any thread. The counts are atomics rather
/// than a lock, because reading one is a call that cannot wait in that record's
/// terms and a lock here would be a lock on the read path of the cache.
#[derive(Debug, Default)]
pub struct Drops {
    counted: [AtomicU64; 5],
}

const _: () = assert!(WhichCheckFailed::all().len() == 5);

impl Drops {
    /// A fresh set of counts, all zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many entries this run dropped for one reading.
    #[must_use]
    pub fn dropped(&self, which: WhichCheckFailed) -> u64 {
        self.counted[which.at()].load(Ordering::Relaxed)
    }

    /// How many entries this run dropped for any reading.
    #[must_use]
    pub fn total(&self) -> u64 {
        WhichCheckFailed::all()
            .iter()
            .map(|which| self.dropped(*which))
            .sum()
    }

    fn count(&self, which: WhichCheckFailed) {
        self.counted[which.at()].fetch_add(1, Ordering::Relaxed);
    }
}

/// Wraps a payload in the envelope this build writes.
///
/// The digest is taken over the payload alone rather than over the header. What
/// the header holds is either checked directly, as the version, the kind and the
/// length are, or is the digest itself, so covering it would add nothing that is
/// not already compared.
#[must_use]
pub fn seal(kind: EntryKind, payload: &[u8]) -> Vec<u8> {
    let mut sealed = Vec::with_capacity(HEADER_WIDTH + payload.len());
    sealed.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
    sealed.push(tag_of(kind));
    // A payload longer than a `u64` can count does not exist on any target this
    // core is built for, and the saturation is here so that the seal has no
    // panic in it rather than because the case is reachable.
    let stated = u64::try_from(payload.len()).unwrap_or(u64::MAX);
    sealed.extend_from_slice(&stated.to_be_bytes());
    sealed.extend_from_slice(&Sha256::digest(payload));
    sealed.extend_from_slice(payload);
    sealed
}

/// Reads an envelope back, in the order 0105 fixes.
///
/// The payload is returned as a slice of what was read rather than copied, so a
/// caller that drops the entry has allocated nothing for a payload it is not
/// going to look at.
///
/// # Errors
///
/// Names which reading failed. Every one of them means the same thing to a
/// caller, which is absence, and they are told apart so that the count 0105 asks
/// for can be.
pub fn open(expected: EntryKind, bytes: &[u8]) -> Result<&[u8], WhichCheckFailed> {
    // The header is taken apart as fixed-width chunks rather than by indexing,
    // so that bytes too short to hold one produce an answer here instead of a
    // panic, and so no offset is written down twice.
    let Some((version_at, rest)) = bytes.split_first_chunk::<VERSION_WIDTH>() else {
        return Err(WhichCheckFailed::Malformed);
    };
    let Some((kind_at, rest)) = rest.split_first_chunk::<KIND_WIDTH>() else {
        return Err(WhichCheckFailed::Malformed);
    };
    let Some((length_at, rest)) = rest.split_first_chunk::<LENGTH_WIDTH>() else {
        return Err(WhichCheckFailed::Malformed);
    };
    let Some((digest_at, payload)) = rest.split_first_chunk::<DIGEST_WIDTH>() else {
        return Err(WhichCheckFailed::Malformed);
    };

    // A kind byte this build does not know is not a kind mismatch. Nothing is
    // being compared: the envelope names something that is not one of the five
    // 0006 lists, so what was read is not an envelope this build can read at
    // all.
    let Some(kind) = kind_of(kind_at[0]) else {
        return Err(WhichCheckFailed::Malformed);
    };

    if u32::from_be_bytes(*version_at) != FORMAT_VERSION {
        return Err(WhichCheckFailed::Version);
    }
    if kind != expected {
        return Err(WhichCheckFailed::Kind);
    }
    if usize::try_from(u64::from_be_bytes(*length_at)) != Ok(payload.len()) {
        return Err(WhichCheckFailed::Length);
    }
    if Sha256::digest(payload).as_slice() != digest_at {
        return Err(WhichCheckFailed::Digest);
    }

    Ok(payload)
}

/// The version an envelope states, for an event about one that failed on it.
///
/// It reads the four bytes and judges nothing, because what it is for is the
/// field 0105 asks the event to carry: the version that was found where a
/// version was the thing that failed.
#[must_use]
pub fn version_found(bytes: &[u8]) -> Option<u32> {
    bytes
        .first_chunk::<VERSION_WIDTH>()
        .copied()
        .map(u32::from_be_bytes)
}

/// The cache as entries rather than as bytes.
///
/// This is the layer 0105 puts between the core and the store: it seals on the
/// way in, opens on the way out, and removes an entry that failed a reading
/// before answering absent. The bound and the eviction underneath it are 0042's
/// and are unchanged - what is counted against a tier is what is stored, which
/// is the sealed entry, so an envelope is paid for in the accounting rather than
/// hidden from it.
///
/// The kind rather than the tier is what a caller names, because the kind is
/// what an entry is and the tier is 0054's accounting for it. Which of the two
/// tiers a kind belongs to is [`Tier::of`].
///
/// Thread safety, from 0009: safe from any thread, for the reason everything it
/// holds is.
#[derive(Debug)]
pub struct Entries<'a> {
    cache: &'a TieredCache<'a>,
    diagnostics: &'a Diagnostics<'a>,
    drops: Drops,
}

impl<'a> Entries<'a> {
    /// Takes the cache the entries are kept in and the facility a drop is
    /// reported through.
    #[must_use]
    pub fn new(cache: &'a TieredCache<'a>, diagnostics: &'a Diagnostics<'a>) -> Self {
        Self {
            cache,
            diagnostics,
            drops: Drops::new(),
        }
    }

    /// Offers one entry to the cache, sealed.
    ///
    /// What comes back is 0042's answer about the bound and says nothing about
    /// the envelope, because nothing about an envelope can fail on the way in.
    pub fn write(&self, kind: EntryKind, key: &EntryKey, payload: &[u8]) -> Cached {
        self.cache.write(Tier::of(kind), key, &seal(kind, payload))
    }

    /// Reads one entry back, or answers absent.
    ///
    /// An entry that fails a reading is removed through the store's own remove
    /// operation and nothing else is examined, nothing else is removed, and no
    /// scan of the store is started. Clearing the cache on a bad entry is what
    /// 0105 refuses by name: it turns one truncated file into the cold start #46
    /// exists against, on the device that just lost power, and it is
    /// self-concealing, because the evidence goes with the cache.
    #[must_use]
    pub fn read(&self, kind: EntryKind, key: &EntryKey) -> Option<Vec<u8>> {
        let held = self.cache.read(Tier::of(kind), key)?;
        match open(kind, &held) {
            Ok(payload) => Some(payload.to_vec()),
            Err(which) => {
                self.cache.forget(Tier::of(kind), key);
                self.drops.count(which);
                self.report(kind, key, which, &held);
                None
            }
        }
    }

    /// How many entries this run dropped, separated by which reading failed.
    ///
    /// A call that cannot wait in the terms of 0009: it reads counters and asks
    /// nothing of the store.
    #[must_use]
    pub fn drops(&self) -> &Drops {
        &self.drops
    }

    fn report(&self, kind: EntryKind, key: &EntryKey, which: WhichCheckFailed, held: &[u8]) {
        let entry = Field::new(ENTRY, FieldValue::Text(key.as_str()));
        let entry_kind = Field::new(ENTRY_KIND, FieldValue::Text(kind.as_str()));
        let check = Field::new(CHECK, FieldValue::Text(which.as_str()));

        // The version is on the event only where a version was the thing that
        // failed, which is 0105's own wording. On any other reading the number
        // is either this build's own or unreadable, and a field carrying one of
        // those two would read as evidence about the drop.
        let found = match which {
            WhichCheckFailed::Version => version_found(held),
            WhichCheckFailed::Malformed
            | WhichCheckFailed::Kind
            | WhichCheckFailed::Length
            | WhichCheckFailed::Digest => None,
        };

        if let Some(found) = found {
            self.diagnostics.emit(
                Severity::Notice,
                ENTRY_DROPPED,
                &[
                    entry,
                    entry_kind,
                    check,
                    Field::new(VERSION_FOUND, FieldValue::Count(u64::from(found))),
                ],
            );
        } else {
            self.diagnostics
                .emit(Severity::Notice, ENTRY_DROPPED, &[entry, entry_kind, check]);
        }
    }
}

/// The byte each kind is written as.
///
/// The numbers are what is on the disk, so they are fixed here and never derived
/// from the order of the enum: reordering the variants of a type is an ordinary
/// edit and would otherwise silently re-label every entry a device already
/// holds.
const fn tag_of(kind: EntryKind) -> u8 {
    match kind {
        EntryKind::LibraryQueryResults => 1,
        EntryKind::ItemMetadata => 2,
        EntryKind::ServerCapabilityAnswers => 3,
        EntryKind::ArtworkBytes => 4,
        EntryKind::DecodedDimensions => 5,
    }
}

/// The kind a byte names, or nothing where this build does not know it.
const fn kind_of(tag: u8) -> Option<EntryKind> {
    match tag {
        1 => Some(EntryKind::LibraryQueryResults),
        2 => Some(EntryKind::ItemMetadata),
        3 => Some(EntryKind::ServerCapabilityAnswers),
        4 => Some(EntryKind::ArtworkBytes),
        5 => Some(EntryKind::DecodedDimensions),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    //! What 0105's four readings are proven with, and against what.
    //!
    //! The store double here answers normally and lets a condition reach in and
    //! damage one entry the way a device losing power would: the bytes are
    //! there, they are not what they claim to be, and nothing about the store
    //! itself failed. That is the whole shape this record is about, and it
    //! cannot be reached through the cache's own write path, which never writes
    //! a bad entry.
    //!
    //! The clock is supplied rather than read, which is 0102 and the
    //! `no-platform-clock` rule in `.github/invariants/rules`.

    use super::{
        Drops, Entries, FORMAT_VERSION, HEADER_WIDTH, WhichCheckFailed, kind_of, open, seal,
        tag_of, version_found,
    };
    use crate::cache::bound::{CacheBounds, Tier, TieredCache};
    use crate::cache::freshness::EntryKind;
    use crate::cache::{ByteStore, EntryKey, StorageUnavailable};
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use crate::diagnostics::redaction::CorrelatorSalt;
    use crate::diagnostics::{Diagnostics, DiagnosticsSink, Event, Severity};
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    /// A store that answers normally and can be reached into.
    #[derive(Debug, Default)]
    struct Store {
        held: Mutex<BTreeMap<String, Vec<u8>>>,
    }

    impl Store {
        fn holds(&self, key: &EntryKey) -> Option<Vec<u8>> {
            self.locked().get(key.as_str()).cloned()
        }

        /// Replaces what is under a key without going through the cache, which
        /// is what a truncated write or a half-replaced entry leaves behind.
        fn damage(&self, key: &EntryKey, bytes: Vec<u8>) {
            self.locked().insert(key.as_str().to_owned(), bytes);
        }

        fn locked(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Vec<u8>>> {
            self.held
                .lock()
                .expect("the fixture holds no poisoned lock")
        }
    }

    impl ByteStore for Store {
        fn read(&self, key: &EntryKey) -> Result<Option<Vec<u8>>, StorageUnavailable> {
            Ok(self.locked().get(key.as_str()).cloned())
        }

        fn write(&self, key: &EntryKey, bytes: &[u8]) -> Result<(), StorageUnavailable> {
            self.locked()
                .insert(key.as_str().to_owned(), bytes.to_vec());
            Ok(())
        }

        fn remove(&self, key: &EntryKey) -> Result<(), StorageUnavailable> {
            self.locked().remove(key.as_str());
            Ok(())
        }

        fn held_bytes(&self) -> Result<u64, StorageUnavailable> {
            Ok(self.locked().values().map(|held| held.len() as u64).sum())
        }
    }

    /// A clock source that answers the same three moments every time.
    #[derive(Debug)]
    struct Fixed;

    impl Clocks for Fixed {
        fn steady(&self) -> SteadyInstant {
            SteadyInstant::from_nanos(7)
        }

        fn elapsed(&self) -> ElapsedInstant {
            ElapsedInstant::from_nanos(7)
        }

        fn wall(&self) -> WallMoment {
            WallMoment::from_epoch(1_700_000_000, 0)
        }
    }

    /// What one event looked like to the sink, copied out of the borrowed value.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Collected {
        severity: Severity,
        name: &'static str,
        fields: Vec<(&'static str, String)>,
    }

    #[derive(Debug, Default)]
    struct Collector {
        seen: Mutex<Vec<Collected>>,
    }

    impl Collector {
        fn collected(&self) -> Vec<Collected> {
            self.seen
                .lock()
                .expect("the fixture holds no poisoned lock")
                .clone()
        }
    }

    impl DiagnosticsSink for Collector {
        fn event(&self, event: &Event<'_>) {
            self.seen
                .lock()
                .expect("the fixture holds no poisoned lock")
                .push(Collected {
                    severity: event.severity(),
                    name: event.name().as_str(),
                    fields: event
                        .fields()
                        .iter()
                        .map(|field| (field.name().as_str(), format!("{:?}", field.value())))
                        .collect(),
                });
        }
    }

    fn a_salt() -> CorrelatorSalt {
        CorrelatorSalt::from_bytes([0x5a; CorrelatorSalt::WIDTH])
    }

    fn key(name: &str) -> EntryKey {
        EntryKey::from_derived_key(name.to_owned())
    }

    const A_PAYLOAD: &[u8] = b"what a server answered, at a length nothing here depends on";
    const A_NEIGHBOUR: &[u8] = b"a second entry, written the same way and left alone";

    /// An entry written and read back through the envelope is the entry that
    /// went in, and nothing was dropped.
    #[test]
    fn what_was_written_is_what_comes_back() {
        let store = Store::default();
        let clocks = Fixed;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);

        assert_eq!(
            entries
                .read(EntryKind::ItemMetadata, &key("one"))
                .as_deref(),
            Some(A_PAYLOAD)
        );
        assert_eq!(entries.drops().total(), 0);
        assert!(collector.collected().is_empty());
    }

    /// What the store holds is the payload inside an envelope, and the bound is
    /// counted against the whole of it rather than against the payload alone.
    #[test]
    fn the_envelope_is_stored_and_is_counted() {
        let store = Store::default();
        let clocks = Fixed;
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);

        let held = store.holds(&key("one")).expect("the store kept it");
        assert_eq!(held.len(), HEADER_WIDTH + A_PAYLOAD.len());
        assert_eq!(
            cache.counted_bytes(Tier::Metadata),
            (HEADER_WIDTH + A_PAYLOAD.len()) as u64
        );
        assert_ne!(&held[..HEADER_WIDTH], A_PAYLOAD);
    }

    /// 0105's first condition: an entry carrying a version this build does not
    /// write is discarded rather than read, in both directions.
    #[test]
    fn an_entry_another_version_wrote_is_dropped_in_both_directions() {
        for stamp in [FORMAT_VERSION - 1, FORMAT_VERSION + 1] {
            let store = Store::default();
            let clocks = Fixed;
            let collector = Collector::default();
            let diagnostics =
                Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
            let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
            let entries = Entries::new(&cache, &diagnostics);

            entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);
            let mut stamped = store.holds(&key("one")).expect("the store kept it");
            stamped[..4].copy_from_slice(&stamp.to_be_bytes());
            store.damage(&key("one"), stamped);

            assert_eq!(entries.read(EntryKind::ItemMetadata, &key("one")), None);
            assert_eq!(entries.drops().dropped(WhichCheckFailed::Version), 1);
            assert_eq!(store.holds(&key("one")), None, "the entry was left behind");

            let seen = collector.collected();
            assert_eq!(seen.len(), 1);
            assert_eq!(seen[0].name, "cache.entry-dropped");
            assert_eq!(seen[0].severity, Severity::Notice);
            assert_eq!(
                seen[0]
                    .fields
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>(),
                vec!["entry", "entry-kind", "check", "version-found"]
            );
            assert_eq!(
                value_of(&seen[0], "version-found"),
                format!("Count({stamp})")
            );
            assert_eq!(value_of(&seen[0], "check"), "Text(\"version\")");
        }
    }

    /// 0105's second condition: an entry that was not finished is dropped rather
    /// than read, and the caller is left with the absence it would have had if
    /// the entry had never been written.
    #[test]
    fn a_truncated_entry_is_dropped_and_answered_as_absent() {
        let store = Store::default();
        let clocks = Fixed;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);
        let mut cut = store.holds(&key("one")).expect("the store kept it");
        cut.truncate(cut.len() - 1);
        store.damage(&key("one"), cut);

        assert_eq!(entries.read(EntryKind::ItemMetadata, &key("one")), None);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Length), 1);
        assert_eq!(store.holds(&key("one")), None, "the entry was left behind");

        // The absence is the ordinary one, so what a caller does next is write
        // what it fetched. That succeeds and the entry reads back.
        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);
        assert_eq!(
            entries
                .read(EntryKind::ItemMetadata, &key("one"))
                .as_deref(),
            Some(A_PAYLOAD)
        );
    }

    /// The case the length misses: a write replaced in place by a shorter one,
    /// leaving the tail of the previous entry behind, so the bytes are the
    /// stated length and are two entries end to end.
    #[test]
    fn an_entry_of_the_stated_length_that_is_two_entries_is_dropped() {
        let store = Store::default();
        let clocks = Fixed;
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);
        let mut mixed = store.holds(&key("one")).expect("the store kept it");
        let last = mixed.len() - 1;
        mixed[last] ^= 0xFF;
        store.damage(&key("one"), mixed);

        assert_eq!(entries.read(EntryKind::ItemMetadata, &key("one")), None);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Digest), 1);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Length), 0);
    }

    /// 0105's third condition: a drop takes one entry and nothing else.
    #[test]
    fn one_bad_entry_does_not_take_a_good_neighbour_with_it() {
        let store = Store::default();
        let clocks = Fixed;
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("bad"), A_PAYLOAD);
        entries.write(EntryKind::ItemMetadata, &key("good"), A_NEIGHBOUR);
        entries.write(EntryKind::ArtworkBytes, &key("art"), A_NEIGHBOUR);
        store.damage(&key("bad"), b"not an envelope".to_vec());

        assert_eq!(entries.read(EntryKind::ItemMetadata, &key("bad")), None);

        assert_eq!(
            entries
                .read(EntryKind::ItemMetadata, &key("good"))
                .as_deref(),
            Some(A_NEIGHBOUR),
            "the neighbour in the same tier went with it"
        );
        assert_eq!(
            entries
                .read(EntryKind::ArtworkBytes, &key("art"))
                .as_deref(),
            Some(A_NEIGHBOUR),
            "the neighbour in the other tier went with it"
        );
        assert_eq!(entries.drops().total(), 1);
    }

    /// A payload is never handed to a reader for a different kind, which is what
    /// the kind in the envelope is for.
    #[test]
    fn an_entry_is_not_handed_to_a_reader_for_another_kind() {
        let store = Store::default();
        let clocks = Fixed;
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);
        // Read under a kind on the same tier, so the tier is not what answers.
        assert_eq!(
            entries.read(EntryKind::LibraryQueryResults, &key("one")),
            None
        );
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Kind), 1);
    }

    /// Bytes that are not an envelope at all, and bytes naming a kind this build
    /// does not know, are the same answer and are counted apart from a version.
    #[test]
    fn bytes_that_are_not_an_envelope_are_malformed_rather_than_anything_else() {
        assert_eq!(
            open(EntryKind::ItemMetadata, b"short"),
            Err(WhichCheckFailed::Malformed)
        );

        let mut unknown = seal(EntryKind::ItemMetadata, A_PAYLOAD);
        unknown[4] = 0xEE;
        assert_eq!(
            open(EntryKind::ItemMetadata, &unknown),
            Err(WhichCheckFailed::Malformed)
        );
        assert_eq!(version_found(&unknown), Some(FORMAT_VERSION));
    }

    /// Every kind this build knows is written as a byte and read back as the
    /// same kind, and no two share one byte.
    #[test]
    fn every_kind_survives_the_byte_it_is_written_as() {
        for kind in EntryKind::all() {
            assert_eq!(kind_of(tag_of(*kind)), Some(*kind), "{}", kind.as_str());
        }

        let mut tags: Vec<u8> = EntryKind::all().iter().map(|kind| tag_of(*kind)).collect();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), EntryKind::all().len());
    }

    /// The counts are separated by which reading failed, which is what makes one
    /// drop after a power cut and four hundred drops on every start different
    /// readings rather than one number.
    #[test]
    fn the_counts_are_separated_by_which_reading_failed() {
        let drops = Drops::new();
        assert_eq!(drops.total(), 0);
        for which in WhichCheckFailed::all() {
            assert_eq!(drops.dropped(*which), 0, "{}", which.as_str());
        }

        let store = Store::default();
        let clocks = Fixed;
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(EntryKind::ItemMetadata, &key("one"), A_PAYLOAD);
        store.damage(&key("one"), b"short".to_vec());
        assert_eq!(entries.read(EntryKind::ItemMetadata, &key("one")), None);

        entries.write(EntryKind::ItemMetadata, &key("two"), A_PAYLOAD);
        let mut cut = store.holds(&key("two")).expect("the store kept it");
        cut.truncate(cut.len() - 2);
        store.damage(&key("two"), cut);
        assert_eq!(entries.read(EntryKind::ItemMetadata, &key("two")), None);

        assert_eq!(entries.drops().dropped(WhichCheckFailed::Malformed), 1);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Length), 1);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Version), 0);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Kind), 0);
        assert_eq!(entries.drops().dropped(WhichCheckFailed::Digest), 0);
        assert_eq!(entries.drops().total(), 2);
    }

    /// An entry that was never written is absent and is not a drop. Without this
    /// the counts above would hold for a read that counted every absence.
    #[test]
    fn an_entry_that_was_never_written_is_absent_and_is_not_a_drop() {
        let store = Store::default();
        let clocks = Fixed;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        assert_eq!(entries.read(EntryKind::ItemMetadata, &key("never")), None);
        assert_eq!(entries.drops().total(), 0);
        assert!(collector.collected().is_empty());
    }

    /// The key is not a field. What identifies the entry on the event is the
    /// correlator 0071 defines, which is what the facility puts there in place
    /// of the value it was handed.
    #[test]
    fn the_event_carries_a_correlator_and_never_the_key() {
        let store = Store::default();
        let clocks = Fixed;
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail, a_salt());
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);
        let entries = Entries::new(&cache, &diagnostics);

        entries.write(
            EntryKind::ItemMetadata,
            &key("a-key-nobody-else-holds"),
            A_PAYLOAD,
        );
        store.damage(&key("a-key-nobody-else-holds"), b"short".to_vec());
        assert_eq!(
            entries.read(EntryKind::ItemMetadata, &key("a-key-nobody-else-holds")),
            None
        );

        let seen = collector.collected();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].name, "cache.entry-dropped");
        // 0105 puts a dropped cache entry at notice because nothing the caller
        // asked for failed. The severity is asserted here as well as on the
        // version drop above, because the two take different branches of the
        // report and one assertion would leave the other unwatched.
        assert_eq!(seen[0].severity, Severity::Notice);
        let written_out = format!("{seen:?}");
        assert!(
            !written_out.contains("a-key-nobody-else-holds"),
            "the key was in what the sink was handed: {written_out}"
        );
        assert_eq!(value_of(&seen[0], "entry-kind"), "Text(\"item-metadata\")");
    }

    /// The debug text of one field of one collected event.
    fn value_of<'a>(collected: &'a Collected, name: &str) -> &'a str {
        collected
            .fields
            .iter()
            .find(|(field, _)| *field == name)
            .map_or_else(
                || panic!("no field named {name}"),
                |(_, value)| value.as_str(),
            )
    }
}
