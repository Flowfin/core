//! The bound on what the cache holds, and what is evicted when it is reached.
//!
//! `docs/decisions/0042-the-cache-bound-and-what-is-evicted.md` is the record
//! and #42 is the issue. Four things come from it and are here: the bound is
//! counted on bytes the core itself counted as it handed them to the store, the
//! entry evicted at the bound is the least recently used one, eviction waits for
//! an outstanding read of the key it chose and never cancels one, and a store
//! that refuses a write is not evidence that the bound was wrong, so writing is
//! suspended for a stated interval rather than the core evicting its own entries
//! to make room for whatever filled the device.
//!
//! # What one of these is, and what #54 adds beside it
//!
//! [`BoundedCache`] is ONE bounded, ordered accounting unit. 0042 says the entry
//! evicted is the least recently used one WITHIN ITS OWN TIER, and 0006 refuses
//! one order across the whole cache. Neither is contradicted by there being one
//! of these today: the bound and the order live inside the unit rather than
//! across the cache, so #54 adds a second beside this one with its own bound and
//! its own order, and the property 0006 asks for is held by the shape rather
//! than by a comparator somebody has to remember at every call site. What #54
//! decides is which tiers exist, how the default is split between them, and that
//! artwork gives way to metadata under pressure. None of those is decided here.
//!
//! # What this does not hold, and what that costs
//!
//! THE INDEX DOES NOT SURVIVE A RESTART, AND NOTHING HERE WRITES IT THROUGH THE
//! STORE. 0042 puts it under a reserved key inside the envelope 0105 defines,
//! written no more often than once every ten seconds and once more at stop, and
//! #105 and #115 are where both of those arrive. So the bound in this tree is
//! enforced against what THIS run wrote, and entries an earlier run left in the
//! store are unaccounted for.
//!
//! THE START-UP RECONCILIATION IS NOT HERE EITHER, for the same reason and not
//! as an oversight. 0042 reads [`ByteStore::held_bytes`] once at start and
//! reduces the budget by whatever the index cannot account for. With no index to
//! restore, every start would find the whole store orphaned and would reduce the
//! budget to nothing, which is the unusable-cache outcome that record names as
//! its worst case rather than its ordinary one. It arrives with the persisted
//! index and not before it.

use core::num::NonZeroU64;
use core::time::Duration;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{ByteStore, EntryKey};
use crate::clock::{Clocks, ElapsedInstant};
use crate::diagnostics::{Diagnostics, EventName, Field, FieldValue, Severity};

/// One mebibyte, so that the arithmetic below reads as the record writes it.
const MEBIBYTE: u64 = 1024 * 1024;

/// The artwork share of the default, from the tier arithmetic in 0042.
const ARTWORK_SHARE: u64 = 224 * MEBIBYTE;

/// The metadata share of the default, from the same arithmetic.
const METADATA_SHARE: u64 = 32 * MEBIBYTE;

/// How long writing is suspended after a run of refusals, from 0042.
///
/// Five minutes on the `elapsed` clock: long enough that a full device is not
/// asked hundreds of times, short enough that somebody who deleted something
/// gets their cache back inside one sitting.
const SUSPENSION: Duration = Duration::from_mins(5);

/// How many consecutive refused writes it takes to suspend writing, from 0042.
///
/// Three, because one refusal is a transient and a run of them is a condition.
const REFUSALS_THAT_SUSPEND: u32 = 3;

/// Reported when a run of refused writes suspends writing.
///
/// Declared here rather than in a central set, which is 0100's rule: an identity
/// belongs with the thing that emits it.
const WRITING_SUSPENDED: EventName = EventName::declared("cache.writing-suspended");

/// How much the cache may hold, in bytes the core counted.
///
/// Counted on what the core handed to the store rather than on what the store
/// reports it occupies. 0042 gives the reason: a store over a filesystem answers
/// with block-rounded numbers, so ten thousand two-kibibyte entries occupy eight
/// times what the core wrote on a device with sixteen-kibibyte blocks, and a
/// bound enforced against that number evicts entries the core never accounted
/// for, differently on every platform. It counts payload bytes as the core
/// produced them, before the envelope 0105 wraps every entry in, because that
/// overhead is fixed per entry rather than a proportion of one.
///
/// THERE IS NO WAY TO EXPRESS A BOUND OF ZERO, and the type is what refuses one
/// rather than a check. 0042 says a client that wants nothing kept supplies no
/// store, which is [`super::CacheStorage::ForTheLifeOfTheProcess`], and that the
/// difference between the two is that the second still holds what it can for the
/// life of the process. A floor below which a bound is refused is #54's, with
/// the floor named at the refusal, and nothing here invents one.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheBound(NonZeroU64);

impl CacheBound {
    /// Two hundred and fifty six mebibytes.
    ///
    /// Chosen rather than measured, like every number on this board that is not
    /// accompanied by a command, and #65 is where a measured replacement would
    /// come from. It is the sum of the two tier bounds in #54 rather than a
    /// third setting that has to be kept in agreement with them, and the
    /// arithmetic behind it rests on two more chosen numbers: an artwork entry
    /// fetched at the size that will actually be drawn is taken as forty
    /// kibibytes, and an item's metadata as two kibibytes.
    ///
    /// ```text
    /// 224 MiB / 40 KiB  =  5734 artwork entries
    ///  32 MiB /  2 KiB  = 16384 metadata entries
    /// ```
    ///
    /// Too small and there is nothing to serve before the first network answer,
    /// which is what the cache exists for. Too large and the core holds a
    /// person's library in a directory somebody else's platform may reclaim
    /// without telling it, on a device whose storage the application does not
    /// own.
    pub const DEFAULT: Self = Self(match NonZeroU64::new(ARTWORK_SHARE + METADATA_SHARE) {
        Some(bytes) => bytes,
        None => panic!("the two shares are constants and their sum is not zero"),
    });

    /// A bound a client chose.
    #[must_use]
    pub const fn of(bytes: NonZeroU64) -> Self {
        Self(bytes)
    }

    /// The bound, in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.0.get()
    }
}

/// What became of bytes offered to the cache.
///
/// A FAILED WRITE NEVER FAILS THE CALL THAT CAUSED IT, which is 0040's sentence
/// and 0042 does not change it. This is not an error type and there is no error
/// type here: somebody asking for a library list gets the library list, and a
/// device that has run out of room is not a reason to answer an empty screen in
/// front of a working server and a valid session. What a caller may do with this
/// is decide whether to offer the same bytes again, and nothing else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cached {
    /// The bytes are in the store and the core is accounting for them.
    Kept,
    /// They are not. The store refused, or writing is suspended, or no entry
    /// could be evicted to make room. The cache is an accelerator and this is
    /// the accelerator declining, rather than anything failing.
    NotKept,
}

/// One entry, as the index knows it.
///
/// The length the core counted and the entry's position in the use order.
/// Nothing else, and in particular no part of the entry's value, because an
/// index that held values would be a second cache with no bound of its own.
#[derive(Debug, Clone)]
struct Held {
    counted: u64,
    used_at: u64,
}

/// Everything one lock protects.
struct Bookkeeping {
    /// What is held, by key.
    entries: BTreeMap<EntryKey, Held>,
    /// The use order, oldest first. Every key here is a key in `entries`.
    order: BTreeMap<u64, EntryKey>,
    /// The sum of the counted lengths in `entries`, held rather than summed so
    /// that a write does not walk the index to find out whether it fits.
    counted: u64,
    /// The next position in the use order.
    next_use: u64,
    /// Keys with a read outstanding, and how many. A key is here for exactly as
    /// long as a call into a client's store is in flight for it.
    reading: BTreeMap<EntryKey, u32>,
    /// How many writes the store has refused in a row.
    refusals_in_a_row: u32,
    /// When writing was suspended, where it is.
    suspended_at: Option<ElapsedInstant>,
}

impl core::fmt::Debug for Bookkeeping {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Bookkeeping")
            .field("entries", &self.entries.len())
            .field("counted", &self.counted)
            .field("reading", &self.reading.len())
            .field("refusals_in_a_row", &self.refusals_in_a_row)
            .field("suspended", &self.suspended_at.is_some())
            .finish_non_exhaustive()
    }
}

/// The cache's own bookkeeping over a store a client supplied.
///
/// It holds no bytes. What it holds is the accounting 0040 pays for by giving
/// the store four operations and no listing: which keys are there, how long each
/// one is as the core counted it, and in what order they were last used.
///
/// Thread safety, from 0009: safe from any thread, and it is called from both
/// lanes. The lock below is over the index and NEVER over a call into a client's
/// store, which is 0040's promise that a slow store is a slow store rather than
/// a stopped core. That is also what makes the read window in 0042 real rather
/// than theoretical: a read is in flight, outside every lock, while another lane
/// is choosing what to evict.
pub struct BoundedCache<'a> {
    store: &'a dyn ByteStore,
    clocks: &'a dyn Clocks,
    diagnostics: &'a Diagnostics<'a>,
    bound: CacheBound,
    bookkeeping: Mutex<Bookkeeping>,
}

/// Written out rather than derived, for the reason [`crate::diagnostics`] gives:
/// neither the store nor the clock source is a type this crate can require
/// `Debug` of, because both are supplied by a client.
impl core::fmt::Debug for BoundedCache<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BoundedCache")
            .field("bound", &self.bound.bytes())
            .field("counted", &self.counted_bytes())
            .field("entries", &self.entries_held())
            .finish_non_exhaustive()
    }
}

impl<'a> BoundedCache<'a> {
    /// The bookkeeping, over the store a client supplied.
    ///
    /// The index starts empty, which is what it means for this tree to hold no
    /// persisted one. See this module's own documentation for what that costs
    /// and for which issue pays it.
    #[must_use]
    pub fn new(
        store: &'a dyn ByteStore,
        clocks: &'a dyn Clocks,
        diagnostics: &'a Diagnostics<'a>,
        bound: CacheBound,
    ) -> Self {
        Self {
            store,
            clocks,
            diagnostics,
            bound,
            bookkeeping: Mutex::new(Bookkeeping {
                entries: BTreeMap::new(),
                order: BTreeMap::new(),
                counted: 0,
                next_use: 0,
                reading: BTreeMap::new(),
                refusals_in_a_row: 0,
                suspended_at: None,
            }),
        }
    }

    /// The bound this cache is held to.
    #[must_use]
    pub const fn bound(&self) -> CacheBound {
        self.bound
    }

    /// How much the core is accounting for, in bytes it counted itself.
    ///
    /// Never the store's own number, for the reason [`CacheBound`] carries.
    #[must_use]
    pub fn counted_bytes(&self) -> u64 {
        self.locked().counted
    }

    /// How many entries the index knows about.
    #[must_use]
    pub fn entries_held(&self) -> usize {
        self.locked().entries.len()
    }

    /// Reads an entry, and moves it to the end of the use order.
    ///
    /// `None` is an absence and is not a failure, which is 0040: a first run, an
    /// entry never written, an entry already evicted and a store that could not
    /// be read all produce it, and the network answers instead. The two are told
    /// apart at the interface a client implements rather than here, because what
    /// a caller does about either is the same thing.
    ///
    /// FOR AS LONG AS THE CALL INTO THE STORE IS IN FLIGHT, THIS KEY CANNOT BE
    /// EVICTED. That is 0042's rule and it is held here rather than at the
    /// eviction: eviction picks the next entry in the order instead and comes
    /// back to this key once the read has finished.
    pub fn read(&self, key: &EntryKey) -> Option<Vec<u8>> {
        let answer = {
            let _in_flight = ReadInFlight::started(self, key);
            self.store.read(key)
        };

        let mut bookkeeping = self.locked();
        match answer {
            Ok(Some(bytes)) => {
                touch(&mut bookkeeping, key);
                Some(bytes)
            }
            Ok(None) => {
                // The store does not have it and the index believed it did. That
                // is the accounting healing itself rather than an error: an
                // entry restored to the index after a refused write and an entry
                // a store lost both arrive here, and either way the budget was
                // wrong in the direction that evicts too early.
                drop(take(&mut bookkeeping, key));
                None
            }
            Err(_) => None,
        }
    }

    /// Offers bytes to the cache, evicting first where they would not fit.
    ///
    /// Eviction happens on the write that would exceed the bound, BEFORE that
    /// write, and it removes entries until the write fits. Not on a timer and
    /// not on a sweep, because a sweep is a thing that runs when nothing else is
    /// happening, which on a television is never.
    ///
    /// Three cases end in [`Cached::NotKept`] without the store being asked at
    /// all, and each is a reading of 0042 rather than a rule beside it.
    ///
    /// Writing is suspended, which is the device-is-full state below.
    ///
    /// The bytes are larger than the whole bound. 0042 says eviction removes
    /// entries until the write fits, and for these there is no number of
    /// evictions that makes them fit, so the loop would empty the cache and
    /// still not store them. Nothing is evicted and nothing is stored.
    ///
    /// Every entry that would have to go has a read outstanding. There is no
    /// next entry in the order to pick, so the write does not fit today and no
    /// read is cancelled to make it fit, which is the half of 0042's rule that
    /// is easy to lose.
    pub fn write(&self, key: &EntryKey, bytes: &[u8]) -> Cached {
        let now = self.clocks.elapsed();
        let incoming = counted_length(bytes);

        let plan = {
            let mut bookkeeping = self.locked();
            if Self::writing_is_suspended(&mut bookkeeping, now) {
                return Cached::NotKept;
            }
            match self.plan_room_for(&mut bookkeeping, key, incoming) {
                Some(plan) => plan,
                None => return Cached::NotKept,
            }
        };

        for at in 0..plan.evicting.len() {
            if self.store.remove(&plan.evicting[at].0).is_err() {
                // The bytes may still be there, so the index goes back to
                // accounting for them. A remove that could not be made is the
                // store being unreachable rather than a write being refused, so
                // it does not count towards the run that suspends writing.
                self.restore_from(&plan, at);
                return Cached::NotKept;
            }
        }

        if self.store.write(key, bytes).is_err() {
            self.refused(&plan, now);
            return Cached::NotKept;
        }

        let mut bookkeeping = self.locked();
        insert(&mut bookkeeping, key, incoming);
        bookkeeping.refusals_in_a_row = 0;
        Cached::Kept
    }

    /// Whether writing is suspended at this moment, clearing a suspension that
    /// has run out.
    ///
    /// 0042 has the core attempt one write again when the interval is up. The
    /// run of refusals is deliberately NOT reset here: a refusal on that attempt
    /// is the fourth consecutive one and suspends writing again, which is what
    /// stops a full device being asked hundreds of times. The run is reset by a
    /// write the store accepted and by nothing else.
    fn writing_is_suspended(bookkeeping: &mut Bookkeeping, now: ElapsedInstant) -> bool {
        let Some(since) = bookkeeping.suspended_at else {
            return false;
        };
        if now.interval_since(since) < SUSPENSION {
            return true;
        }
        bookkeeping.suspended_at = None;
        false
    }

    /// Chooses what has to go for `incoming` bytes to fit, and takes it out of
    /// the index before the lock is dropped.
    ///
    /// Taking it out under the lock is what stops two lanes choosing the same
    /// victim and each removing it once. Nothing is asked of the store here:
    /// every call into a client's store is made with no lock held.
    fn plan_room_for(
        &self,
        bookkeeping: &mut Bookkeeping,
        key: &EntryKey,
        incoming: u64,
    ) -> Option<EvictionPlan> {
        if incoming > self.bound.bytes() {
            return None;
        }

        // A write replaces whatever was there, so the copy being replaced is not
        // in the way of its own replacement.
        let replacing = take(bookkeeping, key);

        let mut evicting: Vec<(EntryKey, Held)> = Vec::new();
        while bookkeeping.counted + incoming > self.bound.bytes() {
            let victim = least_recently_used_not_being_read(bookkeeping)
                .and_then(|victim| take(bookkeeping, &victim));
            let Some(victim) = victim else {
                // Nothing may be evicted. Everything the plan took out goes back
                // exactly where it was, the entry being replaced included.
                put_back(bookkeeping, evicting);
                put_back(bookkeeping, replacing);
                return None;
            };
            evicting.push(victim);
        }

        Some(EvictionPlan {
            replacing: replacing.into_iter().collect(),
            evicting,
        })
    }

    /// Puts back everything a plan took out that is still in the store.
    ///
    /// `stopped_at` is the position in the plan whose removal the store refused.
    /// That entry and every one after it is still there; every one before it is
    /// not, and there is nothing to put back for those.
    fn restore_from(&self, plan: &EvictionPlan, stopped_at: usize) {
        let mut bookkeeping = self.locked();
        put_back(&mut bookkeeping, plan.evicting[stopped_at..].to_vec());
        put_back(&mut bookkeeping, plan.replacing.clone());
    }

    /// Counts a refused write, and suspends writing where that was the third in
    /// a row.
    ///
    /// The entry being replaced goes back into the index because the store may
    /// still hold it. Where it does not, the index over-counts by one entry's
    /// length until something reads that key, which errs towards evicting too
    /// early rather than towards exceeding the bound, and the read path is where
    /// it is corrected.
    ///
    /// What was already evicted stays evicted. Eviction happens before the write
    /// because 0042 says it does, so a write the store then refuses has spent
    /// those entries, and there is nothing to put back with: the core handed
    /// their bytes away and does not hold a copy.
    fn refused(&self, plan: &EvictionPlan, now: ElapsedInstant) {
        let report = {
            let mut bookkeeping = self.locked();
            put_back(&mut bookkeeping, plan.replacing.clone());
            bookkeeping.refusals_in_a_row += 1;
            if bookkeeping.refusals_in_a_row >= REFUSALS_THAT_SUSPEND
                && bookkeeping.suspended_at.is_none()
            {
                bookkeeping.suspended_at = Some(now);
                Some(bookkeeping.refusals_in_a_row)
            } else {
                None
            }
        };

        // Emitted with no lock held. 0100 forbids a sink from calling back into
        // the core and cannot enforce it; holding the index lock across that
        // call would turn the rule into a deadlock rather than a rule.
        if let Some(refusals) = report {
            self.diagnostics.emit(
                Severity::Notice,
                WRITING_SUSPENDED,
                &[
                    Field::new(
                        "consecutive-refusals",
                        FieldValue::Count(u64::from(refusals)),
                    ),
                    Field::new("suspended-for", FieldValue::Interval(SUSPENSION)),
                ],
            );
        }
    }

    /// The index, with a poisoned lock taken rather than propagated.
    ///
    /// Nothing in this module panics while the lock is held: what runs under it
    /// is arithmetic and map operations over values this crate owns. So a
    /// poisoned lock means a defect elsewhere, and a cache that answered nothing
    /// ever again would be a worse answer to it than one that carries on with
    /// the accounting it has.
    fn locked(&self) -> MutexGuard<'_, Bookkeeping> {
        self.bookkeeping
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// What one write took out of the index before it asked the store for anything.
#[derive(Debug)]
struct EvictionPlan {
    /// The copy this write replaces, where there was one. Empty or one entry.
    replacing: Vec<(EntryKey, Held)>,
    /// What has to leave the store, oldest first.
    evicting: Vec<(EntryKey, Held)>,
}

/// A read that is in flight, for as long as this value is alive.
///
/// A value with a destructor rather than a pair of calls, so that the key is
/// released whether the store answered, failed, or unwound. A key left marked as
/// being read is a key eviction could never choose again.
struct ReadInFlight<'c, 'a> {
    cache: &'c BoundedCache<'a>,
    key: EntryKey,
}

impl<'c, 'a> ReadInFlight<'c, 'a> {
    fn started(cache: &'c BoundedCache<'a>, key: &EntryKey) -> Self {
        *cache.locked().reading.entry(key.clone()).or_insert(0) += 1;
        Self {
            cache,
            key: key.clone(),
        }
    }
}

impl Drop for ReadInFlight<'_, '_> {
    fn drop(&mut self) {
        let mut bookkeeping = self.cache.locked();
        if let Some(outstanding) = bookkeeping.reading.get_mut(&self.key) {
            *outstanding -= 1;
            if *outstanding == 0 {
                bookkeeping.reading.remove(&self.key);
            }
        }
    }
}

/// The length the core counts for these bytes.
///
/// `u64` rather than the platform's own width, so that the bound means the same
/// number on a thirty-two-bit television as on anything else.
fn counted_length(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

/// The oldest entry in the use order that has no read in flight.
///
/// `None` where every entry there has one, which is the case 0042 answers by not
/// evicting rather than by cancelling a read.
fn least_recently_used_not_being_read(bookkeeping: &Bookkeeping) -> Option<EntryKey> {
    bookkeeping
        .order
        .values()
        .find(|key| !bookkeeping.reading.contains_key(*key))
        .cloned()
}

/// Takes an entry out of the index, with what it was.
fn take(bookkeeping: &mut Bookkeeping, key: &EntryKey) -> Option<(EntryKey, Held)> {
    let held = bookkeeping.entries.remove(key)?;
    bookkeeping.order.remove(&held.used_at);
    bookkeeping.counted -= held.counted;
    Some((key.clone(), held))
}

/// Puts entries back exactly where they were, position in the order included.
fn put_back<I>(bookkeeping: &mut Bookkeeping, entries: I)
where
    I: IntoIterator<Item = (EntryKey, Held)>,
{
    for (key, held) in entries {
        bookkeeping.counted += held.counted;
        bookkeeping.order.insert(held.used_at, key.clone());
        bookkeeping.entries.insert(key, held);
    }
}

/// Records an entry the store accepted, at the end of the use order.
fn insert(bookkeeping: &mut Bookkeeping, key: &EntryKey, counted: u64) {
    let used_at = bookkeeping.next_use;
    bookkeeping.next_use += 1;
    bookkeeping.counted += counted;
    bookkeeping.order.insert(used_at, key.clone());
    bookkeeping
        .entries
        .insert(key.clone(), Held { counted, used_at });
}

/// Moves an entry to the end of the use order.
///
/// 0042: used means read out of the cache or written into it, and both move the
/// entry to the end of the order.
fn touch(bookkeeping: &mut Bookkeeping, key: &EntryKey) {
    let Some(held) = bookkeeping.entries.get_mut(key) else {
        return;
    };
    let was = held.used_at;
    let used_at = bookkeeping.next_use;
    held.used_at = used_at;
    bookkeeping.next_use += 1;
    bookkeeping.order.remove(&was);
    bookkeeping.order.insert(used_at, key.clone());
}

#[cfg(test)]
mod tests {
    //! What the bound, the eviction and the refusal are proven with.
    //!
    //! The store double here is not the one in the sibling module and the
    //! difference is deliberate. That one answers normally or is unavailable for
    //! everything at once, which is what 0040's own rules need. These conditions
    //! need three more things: a store that refuses writes while still answering
    //! reads, because 0042's device-is-full state has reads continuing
    //! throughout; a count of how many times the store was actually asked to
    //! write, because "stops attempting cache writes" is a statement about calls
    //! that were not made; and a read that can be held open, because the rule
    //! that eviction never reaches an entry with a read outstanding cannot be
    //! observed at all unless the window is held open from outside.
    //!
    //! The clock is supplied rather than read, which is 0102 and the
    //! `no-platform-clock` rule in `.github/invariants/rules`. Five minutes of
    //! suspension is therefore five minutes of arithmetic and not five minutes
    //! of waiting.

    use super::{BoundedCache, CacheBound, Cached, MEBIBYTE, SUSPENSION};
    use crate::cache::{ByteStore, EntryKey, StorageUnavailable};
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use crate::diagnostics::{Diagnostics, DiagnosticsSink, Event, Severity};
    use core::num::NonZeroU64;
    use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
    use std::collections::BTreeMap;
    use std::sync::{Condvar, Mutex};

    /// A read held open from outside, so that the window 0042's rule is about
    /// exists for as long as the test needs it.
    ///
    /// It is armed for one key and fires once. A second read of the same key
    /// passes straight through, so releasing it cannot deadlock a later
    /// assertion that reads the same entry.
    #[derive(Debug, Default)]
    struct Gate {
        state: Mutex<GateState>,
        moved: Condvar,
    }

    #[derive(Debug, Default)]
    struct GateState {
        armed: Option<String>,
        inside: bool,
        released: bool,
    }

    impl Gate {
        fn arm(&self, key: &EntryKey) {
            let mut state = self
                .state
                .lock()
                .expect("the fixture holds no poisoned lock");
            state.armed = Some(key.as_str().to_owned());
            state.inside = false;
            state.released = false;
        }

        /// Called from inside the store's `read`, with no lock of the core's
        /// held. Returns once the test releases it.
        fn pass(&self, key: &EntryKey) {
            let mut state = self
                .state
                .lock()
                .expect("the fixture holds no poisoned lock");
            if state.armed.as_deref() != Some(key.as_str()) {
                return;
            }
            state.armed = None;
            state.inside = true;
            self.moved.notify_all();
            while !state.released {
                state = self
                    .moved
                    .wait(state)
                    .expect("the fixture holds no poisoned lock");
            }
        }

        fn wait_until_inside(&self) {
            let mut state = self
                .state
                .lock()
                .expect("the fixture holds no poisoned lock");
            while !state.inside {
                state = self
                    .moved
                    .wait(state)
                    .expect("the fixture holds no poisoned lock");
            }
        }

        fn release(&self) {
            let mut state = self
                .state
                .lock()
                .expect("the fixture holds no poisoned lock");
            state.released = true;
            self.moved.notify_all();
        }
    }

    /// A byte store that keeps entries in memory, can be made to refuse writes
    /// or removes on their own, counts what it was asked to do, and can hold one
    /// read open.
    #[derive(Debug, Default)]
    struct Store {
        held: Mutex<BTreeMap<String, Vec<u8>>>,
        refuse_reads: AtomicBool,
        refuse_writes: AtomicBool,
        refuse_removes: AtomicBool,
        write_attempts: AtomicU32,
        gate: Gate,
    }

    impl Store {
        fn entries(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Vec<u8>>> {
            self.held
                .lock()
                .expect("the fixture holds no poisoned lock")
        }

        fn refusing_writes() -> Self {
            let store = Self::default();
            store.refuse_writes.store(true, Ordering::Relaxed);
            store
        }

        fn attempts(&self) -> u32 {
            self.write_attempts.load(Ordering::Relaxed)
        }

        fn holds(&self, key: &EntryKey) -> bool {
            self.entries().contains_key(key.as_str())
        }
    }

    impl ByteStore for Store {
        fn read(&self, key: &EntryKey) -> Result<Option<Vec<u8>>, StorageUnavailable> {
            self.gate.pass(key);
            if self.refuse_reads.load(Ordering::Relaxed) {
                return Err(StorageUnavailable);
            }
            Ok(self.entries().get(key.as_str()).cloned())
        }

        fn write(&self, key: &EntryKey, bytes: &[u8]) -> Result<(), StorageUnavailable> {
            self.write_attempts.fetch_add(1, Ordering::Relaxed);
            if self.refuse_writes.load(Ordering::Relaxed) {
                return Err(StorageUnavailable);
            }
            self.entries()
                .insert(key.as_str().to_owned(), bytes.to_vec());
            Ok(())
        }

        fn remove(&self, key: &EntryKey) -> Result<(), StorageUnavailable> {
            if self.refuse_removes.load(Ordering::Relaxed) {
                return Err(StorageUnavailable);
            }
            self.entries().remove(key.as_str());
            Ok(())
        }

        fn held_bytes(&self) -> Result<u64, StorageUnavailable> {
            Ok(self
                .entries()
                .values()
                .map(|bytes| u64::try_from(bytes.len()).unwrap_or(u64::MAX))
                .sum())
        }
    }

    /// The clock source 0102 requires, with the one reading these conditions
    /// move.
    #[derive(Debug, Default)]
    struct Moving {
        elapsed_nanos: AtomicU64,
    }

    impl Moving {
        fn advance(&self, by: core::time::Duration) {
            let nanos = u64::try_from(by.as_nanos()).unwrap_or(u64::MAX);
            self.elapsed_nanos.fetch_add(nanos, Ordering::Relaxed);
        }
    }

    impl Clocks for Moving {
        fn steady(&self) -> SteadyInstant {
            SteadyInstant::from_nanos(0)
        }

        fn elapsed(&self) -> ElapsedInstant {
            ElapsedInstant::from_nanos(self.elapsed_nanos.load(Ordering::Relaxed))
        }

        fn wall(&self) -> WallMoment {
            WallMoment::from_epoch(0, 0)
        }
    }

    /// A sink that keeps the identity of every event it was handed.
    ///
    /// The identity is copied out rather than the event kept, because an
    /// [`Event`] borrows its fields for the call.
    #[derive(Debug, Default)]
    struct Collector {
        names: Mutex<Vec<&'static str>>,
    }

    impl Collector {
        fn named(&self, name: &str) -> usize {
            self.names
                .lock()
                .expect("the fixture holds no poisoned lock")
                .iter()
                .filter(|reported| **reported == name)
                .count()
        }
    }

    impl DiagnosticsSink for Collector {
        fn event(&self, event: &Event<'_>) {
            self.names
                .lock()
                .expect("the fixture holds no poisoned lock")
                .push(event.name().as_str());
        }
    }

    fn key(name: &str) -> EntryKey {
        EntryKey::from_derived_key(name.to_owned())
    }

    fn bound_of(bytes: u64) -> CacheBound {
        CacheBound::of(NonZeroU64::new(bytes).expect("the fixture names a bound above zero"))
    }

    /// A hundred bytes, which is the unit every fixture below counts in.
    fn a_hundred_bytes() -> Vec<u8> {
        vec![b'x'; 100]
    }

    /// The default is the sum of the two tier bounds rather than a third
    /// setting, which is the one thing about it a test can hold. That it is the
    /// right number is not something any run here judges: 0042 says it is chosen
    /// rather than measured and names #65 as where a measured replacement comes
    /// from.
    #[test]
    fn the_default_bound_is_the_sum_of_the_two_shares_and_not_a_third_number() {
        assert_eq!(CacheBound::DEFAULT.bytes(), 224 * MEBIBYTE + 32 * MEBIBYTE);
        assert_eq!(CacheBound::DEFAULT.bytes(), 256 * MEBIBYTE);
    }

    /// The first condition on #42: fill past the bound and prove the bound
    /// holds.
    ///
    /// Twenty entries of a hundred bytes into a bound of five hundred. What the
    /// assertion is on is both numbers, the core's own count and the store's,
    /// because a bound the core believes it is holding while the device fills up
    /// behind it is the failure this is for.
    #[test]
    fn filling_past_the_bound_leaves_the_bound_held() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(500));

        for at in 0..20 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::Kept
            );
            assert!(
                cache.counted_bytes() <= 500,
                "the core counted {} against a bound of 500 after {} writes",
                cache.counted_bytes(),
                at + 1
            );
            assert!(store.held_bytes().expect("the store answers") <= 500);
        }

        assert_eq!(cache.counted_bytes(), 500);
        assert_eq!(cache.entries_held(), 5);
    }

    /// What goes is the oldest by use, and 0042 refuses the two alternatives it
    /// names. This is the half that separates least recently used from age
    /// order: the entry written first is the one still there, because it was
    /// read since.
    #[test]
    fn the_entry_evicted_is_the_least_recently_used_one_and_not_the_oldest() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(200));

        assert_eq!(cache.write(&key("first"), &a_hundred_bytes()), Cached::Kept);
        assert_eq!(
            cache.write(&key("second"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert!(cache.read(&key("first")).is_some());

        assert_eq!(cache.write(&key("third"), &a_hundred_bytes()), Cached::Kept);

        assert!(
            store.holds(&key("first")),
            "reading it moved it out of the way"
        );
        assert!(
            !store.holds(&key("second")),
            "it was the least recently used"
        );
        assert!(store.holds(&key("third")));
        assert_eq!(cache.counted_bytes(), 200);
    }

    /// The second condition on #42, and the one 0042 says will be forgotten
    /// because the window is the length of one call into client code.
    ///
    /// The gate holds a read of the oldest entry open on a second thread while
    /// this one writes an entry that forces exactly one eviction. Without the
    /// rule the oldest entry is what goes, because it is the oldest; with it,
    /// eviction picks the next entry in the order instead and the read is not
    /// cancelled.
    #[test]
    fn an_entry_with_a_read_in_flight_is_not_the_one_evicted() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(200));

        assert_eq!(
            cache.write(&key("oldest"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert_eq!(cache.write(&key("newer"), &a_hundred_bytes()), Cached::Kept);

        store.gate.arm(&key("oldest"));
        // Nothing is asserted inside the scope. A failed assertion there would
        // leave the reading thread parked on a gate nobody releases, and the
        // run would hang instead of going red, which is the shape a guard's
        // deletion proof has to be able to observe.
        let (offered, read) = std::thread::scope(|threads| {
            let reader = threads.spawn(|| cache.read(&key("oldest")));

            store.gate.wait_until_inside();
            let offered = cache.write(&key("arriving"), &a_hundred_bytes());
            store.gate.release();

            (
                offered,
                reader.join().expect("the reading thread did not panic"),
            )
        });

        assert_eq!(offered, Cached::Kept);
        assert!(
            read.is_some(),
            "the read was answered rather than cancelled"
        );

        assert!(store.holds(&key("oldest")), "a read was in flight for it");
        assert!(
            !store.holds(&key("newer")),
            "it was the next entry in the order"
        );
        assert!(store.holds(&key("arriving")));
        assert_eq!(cache.counted_bytes(), 200);
    }

    /// Where every entry that could go has a read in flight there is no next
    /// entry to pick, and 0042's rule is that the read is not cancelled. So the
    /// write does not happen, and nothing about the call that caused it fails.
    #[test]
    fn a_write_that_could_only_fit_by_cancelling_a_read_does_not_happen() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(100));

        assert_eq!(cache.write(&key("only"), &a_hundred_bytes()), Cached::Kept);

        store.gate.arm(&key("only"));
        // Asserted after the scope, for the reason the fixture above states.
        let (offered, read) = std::thread::scope(|threads| {
            let reader = threads.spawn(|| cache.read(&key("only")));

            store.gate.wait_until_inside();
            let offered = cache.write(&key("arriving"), &a_hundred_bytes());
            store.gate.release();

            (
                offered,
                reader.join().expect("the reading thread did not panic"),
            )
        });

        assert_eq!(offered, Cached::NotKept);
        assert!(read.is_some());

        assert!(store.holds(&key("only")));
        assert!(!store.holds(&key("arriving")));
        assert_eq!(cache.counted_bytes(), 100);
    }

    /// The third condition on #42: a store that refuses writes, and a core that
    /// degrades to working without a cache rather than failing.
    ///
    /// Nothing here returns an error, because there is no error to return: a
    /// failed write never fails the call that caused it. What the caller can
    /// read is that the bytes were not kept, and reads keep being answered
    /// throughout, which is 0042's own sentence about a device with no room.
    #[test]
    fn a_store_that_refuses_every_write_leaves_a_core_that_works() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, CacheBound::DEFAULT);

        for at in 0..5 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }

        assert_eq!(cache.counted_bytes(), 0);
        assert_eq!(cache.entries_held(), 0);
        assert_eq!(
            cache.read(&key("0000")),
            None,
            "an absence rather than a failure"
        );

        store.refuse_writes.store(false, Ordering::Relaxed);
        clocks.advance(SUSPENSION);
        assert_eq!(cache.write(&key("later"), &a_hundred_bytes()), Cached::Kept);
        assert_eq!(cache.read(&key("later")), Some(a_hundred_bytes()));
    }

    /// After three consecutive refusals the store is not asked again, and the
    /// suspension is reported once rather than once per write.
    #[test]
    fn three_refusals_in_a_row_stop_the_core_asking_and_are_reported_once() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, CacheBound::DEFAULT);

        for at in 0..3 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }
        assert_eq!(store.attempts(), 3);
        assert_eq!(collector.named("cache.writing-suspended"), 1);

        for at in 3..20 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }
        assert_eq!(
            store.attempts(),
            3,
            "the store was not asked while writing was suspended"
        );
        assert_eq!(collector.named("cache.writing-suspended"), 1);
    }

    /// Reads continue throughout a suspension, because a device with no room can
    /// still be read and what is already held is what matters most at exactly
    /// that moment.
    #[test]
    fn reads_are_answered_while_writing_is_suspended() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, CacheBound::DEFAULT);

        assert_eq!(cache.write(&key("held"), &a_hundred_bytes()), Cached::Kept);
        store.refuse_writes.store(true, Ordering::Relaxed);
        for at in 0..3 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }

        assert_eq!(cache.read(&key("held")), Some(a_hundred_bytes()));
    }

    /// When the interval is up the core attempts one write again. The run of
    /// refusals is not reset by the interval running out, so a refusal on that
    /// attempt is the fourth consecutive one and suspends writing again.
    #[test]
    fn writing_returns_when_the_interval_is_up_and_one_more_refusal_suspends_it_again() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, CacheBound::DEFAULT);

        for at in 0..3 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }
        assert_eq!(store.attempts(), 3);

        clocks.advance(SUSPENSION);
        assert_eq!(
            cache.write(&key("probe"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(
            store.attempts(),
            4,
            "the interval was up, so one write was attempted"
        );
        assert_eq!(collector.named("cache.writing-suspended"), 2);

        assert_eq!(
            cache.write(&key("again"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(store.attempts(), 4, "and writing is suspended again");
    }

    /// A write the store accepts is what ends a run of refusals, and nothing
    /// else. Two refusals followed by a success leave the next two refusals
    /// short of the threshold.
    #[test]
    fn a_write_the_store_accepted_is_what_ends_a_run_of_refusals() {
        let store = Store::default();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, CacheBound::DEFAULT);

        store.refuse_writes.store(true, Ordering::Relaxed);
        assert_eq!(cache.write(&key("a"), &a_hundred_bytes()), Cached::NotKept);
        assert_eq!(cache.write(&key("b"), &a_hundred_bytes()), Cached::NotKept);

        store.refuse_writes.store(false, Ordering::Relaxed);
        assert_eq!(cache.write(&key("c"), &a_hundred_bytes()), Cached::Kept);

        store.refuse_writes.store(true, Ordering::Relaxed);
        assert_eq!(cache.write(&key("d"), &a_hundred_bytes()), Cached::NotKept);
        assert_eq!(cache.write(&key("e"), &a_hundred_bytes()), Cached::NotKept);
        assert_eq!(collector.named("cache.writing-suspended"), 0);
    }

    /// Bytes no number of evictions could make room for empty nothing. 0042 has
    /// eviction remove entries until the write fits, and where it never fits the
    /// loop would spend the whole cache and still not store them.
    #[test]
    fn bytes_larger_than_the_whole_bound_evict_nothing() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(150));

        assert_eq!(cache.write(&key("held"), &a_hundred_bytes()), Cached::Kept);
        assert_eq!(cache.write(&key("enormous"), &[b'x'; 151]), Cached::NotKept);

        assert!(
            store.holds(&key("held")),
            "nothing was spent on a write that could not fit"
        );
        assert!(!store.holds(&key("enormous")));
        assert_eq!(cache.counted_bytes(), 100);
    }

    /// A write replaces whatever was there, so the copy being replaced is not in
    /// the way of its own replacement and it is counted once rather than twice.
    #[test]
    fn a_write_that_replaces_an_entry_is_counted_once() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(100));

        assert_eq!(cache.write(&key("one"), &a_hundred_bytes()), Cached::Kept);
        assert_eq!(cache.write(&key("one"), &[b'y'; 100]), Cached::Kept);

        assert_eq!(cache.counted_bytes(), 100);
        assert_eq!(cache.entries_held(), 1);
        assert_eq!(cache.read(&key("one")), Some(vec![b'y'; 100]));
    }

    /// A remove the store could not make leaves the entry accounted for, because
    /// its bytes may still be there. Nothing is stored, and this is not a refused
    /// write, so it does not count towards the run that suspends writing.
    #[test]
    fn a_remove_the_store_refused_leaves_the_entry_accounted_for() {
        let store = Store::default();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(100));

        assert_eq!(cache.write(&key("held"), &a_hundred_bytes()), Cached::Kept);
        store.refuse_removes.store(true, Ordering::Relaxed);

        for _ in 0..5 {
            assert_eq!(
                cache.write(&key("arriving"), &a_hundred_bytes()),
                Cached::NotKept
            );
        }

        assert_eq!(cache.counted_bytes(), 100);
        assert_eq!(cache.entries_held(), 1);
        assert!(store.holds(&key("held")));
        assert_eq!(collector.named("cache.writing-suspended"), 0);
    }

    /// The index believing in an entry the store does not have is corrected by
    /// the read that finds it absent, rather than being carried until the cache
    /// has no budget left.
    #[test]
    fn an_entry_the_store_no_longer_has_leaves_the_index_on_the_next_read() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(100));

        assert_eq!(cache.write(&key("held"), &a_hundred_bytes()), Cached::Kept);
        store.entries().remove("held");

        assert_eq!(cache.read(&key("held")), None);
        assert_eq!(cache.counted_bytes(), 0);
        assert_eq!(cache.entries_held(), 0);
    }

    /// A store that could not be read is an absence to the caller and leaves the
    /// accounting where it was, because nothing was learned about what the store
    /// holds. That is the opposite direction from the entry the store no longer
    /// has above, and collapsing the two would empty the index every time a
    /// device was locked in the background.
    #[test]
    fn a_store_that_could_not_be_read_leaves_the_accounting_alone() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(100));

        assert_eq!(cache.write(&key("held"), &a_hundred_bytes()), Cached::Kept);
        store.refuse_reads.store(true, Ordering::Relaxed);

        assert_eq!(cache.read(&key("held")), None);
        assert_eq!(cache.counted_bytes(), 100);
        assert_eq!(cache.entries_held(), 1);
    }

    /// The bound a client chose is the one enforced, rather than the default
    /// quietly standing behind it.
    #[test]
    fn the_bound_a_client_chose_is_the_one_enforced() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = BoundedCache::new(&store, &clocks, &diagnostics, bound_of(250));

        assert_eq!(cache.bound().bytes(), 250);
        for at in 0..10 {
            assert_eq!(
                cache.write(&key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::Kept
            );
        }
        assert_eq!(cache.counted_bytes(), 200);
        assert_eq!(cache.entries_held(), 2);
    }
}
