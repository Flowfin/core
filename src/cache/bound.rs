//! The bounds on what the cache holds, and what is evicted when one is reached.
//!
//! Two records and two issues. 0042 and #42 fix the bound, the eviction rule,
//! the read eviction may not reach, and what a full device does. 0054 and #54
//! fix that there are two tiers rather than one, that neither can evict the
//! other, what the split between them is, and the single place where artwork
//! gives way to metadata.
//!
//! # The tiers, and why the split is structural rather than a preference
//!
//! Artwork in one tier and everything else in the other, each with its own bound
//! and its own use order. A comparator that prefers metadata would express the
//! same intention and would not hold it: a preference is a rule that lasts until
//! the metadata IS the least recently used thing in a cache full of artwork,
//! which after a long pass through a large library it is. With two orders
//! nothing ever chooses between them, because there is no order in which the two
//! appear together.
//!
//! Neither tier borrows from the other. An artwork tier that is full beside a
//! metadata tier that is half empty evicts artwork and leaves the free space
//! free. Borrowing reads as an improvement and is one bound again in slower
//! motion: the first sustained pass through a library takes the borrowed space
//! and nothing triggers giving it back.
//!
//! # What this does not hold, and what that costs
//!
//! THE INDEX DOES NOT SURVIVE A RESTART, AND NOTHING HERE WRITES IT THROUGH THE
//! STORE. 0042 puts it under a reserved key inside the envelope 0105 defines,
//! written no more often than once every ten seconds and once more at stop, and
//! #105 and #115 are where both of those arrive. So the bounds in this tree are
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
//!
//! WHICH ENTRIES ARE OF WHICH KIND IS NOT DECIDED HERE. 0054 places artwork
//! bytes in one tier and library query results, item metadata, capability
//! answers and decoded image dimensions in the other, and 0006 and #43 are where
//! the kinds themselves live. A caller says which tier it is asking about; there
//! is no kind in this module to derive it from, and inventing one would decide
//! #43 in the file that was supposed to hold #54.

use core::time::Duration;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{ByteStore, EntryKey};
use crate::clock::{Clocks, ElapsedInstant};
use crate::diagnostics::{Diagnostics, EventName, Field, FieldValue, Severity};

/// One mebibyte, so that the arithmetic below reads as the records write it.
const MEBIBYTE: u64 = 1024 * 1024;

/// The artwork tier's default bound, from the arithmetic in 0042 and 0054.
const ARTWORK_DEFAULT: u64 = 224 * MEBIBYTE;

/// The metadata tier's default bound, from the same arithmetic.
const METADATA_DEFAULT: u64 = 32 * MEBIBYTE;

/// The smallest metadata tier 0054 admits.
///
/// Four mebibytes, which is about two thousand items on the two-kibibyte figure
/// those records use. Below it the tier holds less than a large library's
/// listing, so one pass through that library evicts what the pass started from
/// and the cache costs more than it returns.
const METADATA_FLOOR: u64 = 4 * MEBIBYTE;

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

/// How much more than the refused write artwork gives way for, from 0054.
///
/// Eight times, because freeing exactly what was needed buys one write and then
/// the next metadata write does the same work again.
const GIVE_WAY_MULTIPLE: u64 = 8;

/// The floor on how much artwork gives way at once, from 0054.
///
/// One mebibyte, because a run of two-kibibyte writes would otherwise trigger an
/// eviction round each, and a round is a walk of the artwork order and a call
/// into the client's store.
const GIVE_WAY_FLOOR: u64 = MEBIBYTE;

/// Reported when a run of refused writes suspends writing.
///
/// Declared here rather than in a central set, which is 0100's rule: an identity
/// belongs with the thing that emits it.
const WRITING_SUSPENDED: EventName = EventName::declared("cache.writing-suspended");

/// Reported when artwork was released so that a refused metadata write could be
/// attempted again.
const ARTWORK_GAVE_WAY: EventName = EventName::declared("cache.artwork-gave-way");

/// Which of the two accountings an entry belongs to.
///
/// 0054 gives artwork its own tier because artwork is large, numerous and cheap
/// to lose, while metadata is small, few, and expensive to lose: it is what a
/// client builds a first screen out of, and losing it costs somebody a blank
/// screen in front of a server that is not there.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Artwork bytes.
    Artwork,
    /// Everything else 0006 calls a cache entry: library query results, item
    /// metadata, a server's capability answers, and the decoded dimensions of an
    /// image.
    ///
    /// Dimensions are here rather than beside the bytes they describe, which
    /// 0054 says is worth stating because it reads as a mistake. They are tens
    /// of bytes each, they are what #52 uses to reserve room before an image
    /// arrives, and losing them costs a layout that moves under somebody rather
    /// than a picture that is briefly missing.
    Metadata,
}

impl Tier {
    /// Both tiers.
    ///
    /// Here so that a caller reads the set out of the crate rather than keeping
    /// a copy of it, and so the conditions below apply a rule to the whole of it
    /// rather than to whichever member somebody remembered.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Artwork, Self::Metadata]
    }

    /// The tier as it is reported.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Artwork => "artwork",
            Self::Metadata => "metadata",
        }
    }

    /// Where this tier's accounting is held.
    const fn at(self) -> usize {
        match self {
            Self::Artwork => 0,
            Self::Metadata => 1,
        }
    }
}

/// A bound a client asked for that is below the floor its tier has.
///
/// It names the floor, which is 0054's requirement: a bound below it is refused
/// rather than accepted and quietly raised, because a client silently given
/// something other than what it asked for has no way to find out.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundBelowTheFloor {
    tier: Tier,
    floor: u64,
    asked: u64,
}

impl BoundBelowTheFloor {
    /// The tier whose floor this is.
    #[must_use]
    pub const fn tier(self) -> Tier {
        self.tier
    }

    /// The smallest bound that tier admits, in bytes.
    #[must_use]
    pub const fn floor(self) -> u64 {
        self.floor
    }

    /// What was asked for, in bytes.
    #[must_use]
    pub const fn asked(self) -> u64 {
        self.asked
    }
}

/// How much each tier may hold, in bytes the core counted.
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
/// THE TOTAL IS NOT A SETTING. 0042 states two hundred and fifty six mebibytes
/// and 0054 states the two numbers that add to it, and [`CacheBounds::total`]
/// computes the sum rather than holding a third value. Three numbers where two
/// would do is three numbers that can disagree, and the disagreement would be
/// found by whichever of them the code happened to check first.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheBounds {
    artwork: u64,
    metadata: u64,
}

impl CacheBounds {
    /// Two hundred and twenty four mebibytes of artwork beside thirty two of
    /// metadata.
    ///
    /// Both chosen rather than measured, like every number on this board that is
    /// not accompanied by a command, and #65 is where measured replacements
    /// would come from. The arithmetic rests on two more chosen numbers: an
    /// artwork entry fetched at the size that will actually be drawn is taken as
    /// forty kibibytes, and an item's metadata as two kibibytes.
    ///
    /// ```text
    /// 224 MiB / 40 KiB  =  5734 artwork entries
    ///  32 MiB /  2 KiB  = 16384 metadata entries
    /// 224 + 32          =   256 MiB, which is the total in 0042
    /// ```
    ///
    /// Seven eighths to artwork and one eighth to metadata, and the RATIO rather
    /// than either number is what the reasoning is about. An artwork entry is
    /// about twenty times the size of a metadata one and a library has one or
    /// two images an item, so holding both for the same items in proportion
    /// would spend roughly ninety five per cent of the cache on pictures. The
    /// split gives metadata more than that on purpose, so that the tier which
    /// decides whether there is a screen at all runs out last.
    pub const DEFAULT: Self = Self {
        artwork: ARTWORK_DEFAULT,
        metadata: METADATA_DEFAULT,
    };

    /// The bounds a client chose.
    ///
    /// The artwork tier may be zero, and that is not the floor being zero. A
    /// client that draws no artwork exists, and the probe an operator runs
    /// against their own server in #92 is one. A client that holds no metadata
    /// does not, because there would be nothing left for the core to serve out
    /// of the cache at all, which is why the two tiers are asked different
    /// questions here.
    ///
    /// # Errors
    ///
    /// [`BoundBelowTheFloor`] where the metadata bound is under the floor 0054
    /// fixes, carrying that floor rather than only refusing.
    pub const fn of(artwork: u64, metadata: u64) -> Result<Self, BoundBelowTheFloor> {
        if metadata < METADATA_FLOOR {
            return Err(BoundBelowTheFloor {
                tier: Tier::Metadata,
                floor: METADATA_FLOOR,
                asked: metadata,
            });
        }
        Ok(Self { artwork, metadata })
    }

    /// The bound on one tier, in bytes.
    #[must_use]
    pub const fn of_tier(self, tier: Tier) -> u64 {
        match tier {
            Tier::Artwork => self.artwork,
            Tier::Metadata => self.metadata,
        }
    }

    /// The sum of the two, which is what 0042 calls the total.
    ///
    /// Computed rather than held, so that a client changing one tier changes the
    /// total and there is no third number to keep in agreement.
    #[must_use]
    pub const fn total(self) -> u64 {
        self.artwork + self.metadata
    }
}

/// What became of bytes offered to the cache.
///
/// A FAILED WRITE NEVER FAILS THE CALL THAT CAUSED IT, which is 0040's sentence
/// and neither 0042 nor 0054 changes it. This is not an error type and there is
/// no error type here: somebody asking for a library list gets the library list,
/// and a device that has run out of room is not a reason to answer an empty
/// screen in front of a working server and a valid session. What a caller may do
/// with this is decide whether to offer the same bytes again, and nothing else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cached {
    /// The bytes are in the store and the core is accounting for them.
    Kept,
    /// They are not. The store refused, or writing is suspended, or no entry in
    /// that tier could be evicted to make room. The cache is an accelerator and
    /// this is the accelerator declining, rather than anything failing.
    NotKept,
}

/// One entry, as the index knows it.
///
/// The length the core counted and the entry's position in its tier's use order.
/// Nothing else, and in particular no part of the entry's value, because an
/// index that held values would be a second cache with no bound of its own.
#[derive(Debug, Clone)]
struct Held {
    counted: u64,
    used_at: u64,
}

/// One tier's accounting.
struct Bookkeeping {
    /// What this tier holds, by key.
    entries: BTreeMap<EntryKey, Held>,
    /// This tier's use order, oldest first. Every key here is a key in
    /// `entries`, and no key of the other tier ever appears in it, which is what
    /// makes 0054's promise structural rather than a preference.
    order: BTreeMap<u64, EntryKey>,
    /// The sum of the counted lengths in `entries`, held rather than summed so
    /// that a write does not walk the index to find out whether it fits.
    counted: u64,
}

impl Bookkeeping {
    const fn empty() -> Self {
        Self {
            entries: BTreeMap::new(),
            order: BTreeMap::new(),
            counted: 0,
        }
    }
}

impl core::fmt::Debug for Bookkeeping {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Bookkeeping")
            .field("entries", &self.entries.len())
            .field("counted", &self.counted)
            .finish_non_exhaustive()
    }
}

/// Everything one lock protects.
///
/// The two tiers are separate accountings and the three fields under them are
/// deliberately not. A read in flight is a key, and a key names one entry in one
/// store; a run of refused writes and the suspension that follows are facts
/// about the device, which 0054 says applies to both tiers rather than to
/// whichever one met it.
struct State {
    tiers: [Bookkeeping; 2],
    /// The next position in either use order. One counter for both, so that no
    /// two entries anywhere share a position; the orders stay separate because
    /// they are separate maps rather than because the numbers are.
    next_use: u64,
    /// Keys with a read outstanding, and how many. A key is here for exactly as
    /// long as a call into a client's store is in flight for it.
    reading: BTreeMap<EntryKey, u32>,
    /// How many writes the store has refused in a row, across both tiers.
    refusals_in_a_row: u32,
    /// When writing was suspended, where it is.
    suspended_at: Option<ElapsedInstant>,
}

/// The cache's own bookkeeping over a store a client supplied.
///
/// It holds no bytes. What it holds is the accounting 0040 pays for by giving
/// the store four operations and no listing: which keys are there, how long each
/// one is as the core counted it, which tier it is in, and in what order that
/// tier last used it.
///
/// Thread safety, from 0009: safe from any thread, and it is called from both
/// lanes. The lock below is over the index and NEVER over a call into a client's
/// store, which is 0040's promise that a slow store is a slow store rather than
/// a stopped core. That is also what makes the read window in 0042 real rather
/// than theoretical: a read is in flight, outside every lock, while another lane
/// is choosing what to evict.
pub struct TieredCache<'a> {
    store: &'a dyn ByteStore,
    clocks: &'a dyn Clocks,
    diagnostics: &'a Diagnostics<'a>,
    bounds: CacheBounds,
    state: Mutex<State>,
}

/// Written out rather than derived, for the reason [`crate::diagnostics`] gives:
/// neither the store nor the clock source is a type this crate can require
/// `Debug` of, because both are supplied by a client.
impl core::fmt::Debug for TieredCache<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TieredCache")
            .field("bounds", &self.bounds)
            .field("artwork", &self.counted_bytes(Tier::Artwork))
            .field("metadata", &self.counted_bytes(Tier::Metadata))
            .finish_non_exhaustive()
    }
}

impl<'a> TieredCache<'a> {
    /// The bookkeeping, over the store a client supplied.
    ///
    /// The index starts empty, which is what it means for this tree to hold no
    /// persisted one. See this module's own documentation for what that costs
    /// and for which issues pay it.
    #[must_use]
    pub fn new(
        store: &'a dyn ByteStore,
        clocks: &'a dyn Clocks,
        diagnostics: &'a Diagnostics<'a>,
        bounds: CacheBounds,
    ) -> Self {
        Self {
            store,
            clocks,
            diagnostics,
            bounds,
            state: Mutex::new(State {
                tiers: [Bookkeeping::empty(), Bookkeeping::empty()],
                next_use: 0,
                reading: BTreeMap::new(),
                refusals_in_a_row: 0,
                suspended_at: None,
            }),
        }
    }

    /// The bounds this cache is held to.
    #[must_use]
    pub const fn bounds(&self) -> CacheBounds {
        self.bounds
    }

    /// How much one tier is accounting for, in bytes the core counted itself.
    ///
    /// Never the store's own number, for the reason [`CacheBounds`] carries.
    #[must_use]
    pub fn counted_bytes(&self, tier: Tier) -> u64 {
        self.locked().tiers[tier.at()].counted
    }

    /// How many entries one tier's index knows about.
    #[must_use]
    pub fn entries_held(&self, tier: Tier) -> usize {
        self.locked().tiers[tier.at()].entries.len()
    }

    /// Reads an entry, and moves it to the end of its tier's use order.
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
    pub fn read(&self, tier: Tier, key: &EntryKey) -> Option<Vec<u8>> {
        let answer = {
            let _in_flight = ReadInFlight::started(self, key);
            self.store.read(key)
        };

        let mut state = self.locked();
        match answer {
            Ok(Some(bytes)) => {
                touch(&mut state, tier, key);
                Some(bytes)
            }
            Ok(None) => {
                // The store does not have it and the index believed it did. That
                // is the accounting healing itself rather than an error: an
                // entry restored to the index after a refused write and an entry
                // a store lost both arrive here, and either way the budget was
                // wrong in the direction that evicts too early.
                drop(take(&mut state, tier, key));
                None
            }
            Err(_) => None,
        }
    }

    /// Offers bytes to one tier, evicting from that tier first where they would
    /// not fit.
    ///
    /// Eviction happens on the write that would exceed the bound, BEFORE that
    /// write, and it removes entries until the write fits. Not on a timer and
    /// not on a sweep, because a sweep is a thing that runs when nothing else is
    /// happening, which on a television is never. It considers only this tier,
    /// which is 0054: there is no order in which the two appear together, so
    /// nothing has to prefer one at eviction time.
    ///
    /// Three cases end in [`Cached::NotKept`] without the store being asked at
    /// all, and each is a reading of 0042 rather than a rule beside it.
    ///
    /// Writing is suspended, which is the device-is-full state below.
    ///
    /// The bytes are larger than the whole tier's bound. 0042 says eviction
    /// removes entries until the write fits, and for these there is no number of
    /// evictions that makes them fit, so the loop would empty the tier and still
    /// not store them. Nothing is evicted and nothing is stored, and a tier a
    /// client bounded at zero is that case for everything offered to it.
    ///
    /// Every entry that would have to go has a read outstanding. There is no
    /// next entry in the order to pick, so the write does not fit today and no
    /// read is cancelled to make it fit, which is the half of 0042's rule that
    /// is easy to lose.
    pub fn write(&self, tier: Tier, key: &EntryKey, bytes: &[u8]) -> Cached {
        let now = self.clocks.elapsed();
        let incoming = counted_length(bytes);

        let plan = {
            let mut state = self.locked();
            if writing_is_suspended(&mut state, now) {
                return Cached::NotKept;
            }
            match self.plan_room_for(&mut state, tier, key, incoming) {
                Some(plan) => plan,
                None => return Cached::NotKept,
            }
        };

        if !self.evict(&plan) {
            return Cached::NotKept;
        }

        if self.store.write(key, bytes).is_ok() {
            return self.kept(tier, key, incoming);
        }

        // 0054's one asymmetry, written at the only place in the design where it
        // has anything to bite on. A refused METADATA write releases artwork and
        // tries once more. A refused ARTWORK write never touches metadata, is
        // not retried, and the entry is simply not cached, which #51 already
        // makes a first-class answer rather than a failure. The other direction
        // would free metadata to make room for a picture, during exactly the
        // pass through a library where the metadata is needed next.
        self.count_refusal(now);
        if tier == Tier::Metadata && self.artwork_gives_way(incoming) {
            if self.store.write(key, bytes).is_ok() {
                return self.kept(tier, key, incoming);
            }
            // A second call the store said no to, and counted as one. Where
            // there was no artwork to release there was no second call, and
            // nothing is counted for it.
            self.count_refusal(now);
        }

        self.put_replaced_back(&plan);
        Cached::NotKept
    }

    /// Records an entry the store accepted, and ends the run of refusals.
    fn kept(&self, tier: Tier, key: &EntryKey, incoming: u64) -> Cached {
        let mut state = self.locked();
        insert(&mut state, tier, key, incoming);
        state.refusals_in_a_row = 0;
        Cached::Kept
    }

    /// Releases artwork so that a refused metadata write can be attempted again.
    ///
    /// At least eight times the refused write or one mebibyte, whichever is
    /// more, and as much as the artwork tier holds where that is less. Answers
    /// whether anything was actually released, because attempting the write
    /// again after freeing nothing is a second call into a store that has
    /// already said no.
    ///
    /// WHAT THIS SPENDS IS SPENT WHETHER OR NOT THE SECOND ATTEMPT SUCCEEDS. The
    /// core handed those bytes away and holds no copy, so an attempt the store
    /// refuses again has cost the artwork for nothing. 0054 states that as the
    /// price of the rule rather than as a defect in it, and names the case: two
    /// bounds that together exceed what the device can hold.
    fn artwork_gives_way(&self, refused: u64) -> bool {
        let target = refused
            .saturating_mul(GIVE_WAY_MULTIPLE)
            .max(GIVE_WAY_FLOOR);

        let plan = {
            let mut state = self.locked();
            release_from_artwork(&mut state, target)
        };
        if plan.evicting.is_empty() {
            return false;
        }

        let released: u64 = plan.evicting.iter().map(|(_, held)| held.counted).sum();
        let entries = u64::try_from(plan.evicting.len()).unwrap_or(u64::MAX);
        if !self.evict(&plan) {
            return false;
        }

        self.diagnostics.emit(
            Severity::Notice,
            ARTWORK_GAVE_WAY,
            &[
                Field::new("released-bytes", FieldValue::Count(released)),
                Field::new("released-entries", FieldValue::Count(entries)),
                Field::new("for-tier", FieldValue::Text(Tier::Metadata.as_str())),
            ],
        );
        true
    }

    /// Asks the store to remove everything a plan chose, oldest first.
    ///
    /// Answers whether all of them went. A remove the store refused puts that
    /// entry and everything after it back into the index, because their bytes
    /// may still be there; what was already removed stays removed, and there is
    /// nothing to put back with.
    fn evict(&self, plan: &EvictionPlan) -> bool {
        for at in 0..plan.evicting.len() {
            if self.store.remove(&plan.evicting[at].0).is_err() {
                // A remove that could not be made is the store being unreachable
                // rather than a write being refused, so it does not count
                // towards the run that suspends writing.
                let mut state = self.locked();
                put_back(&mut state, plan.tier, plan.evicting[at..].to_vec());
                put_back(&mut state, plan.tier, plan.replacing.clone());
                return false;
            }
        }
        true
    }

    /// Chooses what has to go for `incoming` bytes to fit in one tier, and takes
    /// it out of the index before the lock is dropped.
    ///
    /// Taking it out under the lock is what stops two lanes choosing the same
    /// victim and each removing it once. Nothing is asked of the store here:
    /// every call into a client's store is made with no lock held.
    fn plan_room_for(
        &self,
        state: &mut State,
        tier: Tier,
        key: &EntryKey,
        incoming: u64,
    ) -> Option<EvictionPlan> {
        let bound = self.bounds.of_tier(tier);
        if bound == 0 || incoming > bound {
            return None;
        }

        // A write replaces whatever was there, so the copy being replaced is not
        // in the way of its own replacement.
        let replacing = take(state, tier, key);

        let mut evicting: Vec<(EntryKey, Held)> = Vec::new();
        while state.tiers[tier.at()].counted + incoming > bound {
            let victim = least_recently_used_not_being_read(state, tier)
                .and_then(|victim| take(state, tier, &victim));
            let Some(victim) = victim else {
                // Nothing in this tier may be evicted. Everything the plan took
                // out goes back exactly where it was, the entry being replaced
                // included.
                put_back(state, tier, evicting);
                put_back(state, tier, replacing);
                return None;
            };
            evicting.push(victim);
        }

        Some(EvictionPlan {
            tier,
            replacing: replacing.into_iter().collect(),
            evicting,
        })
    }

    /// Puts back the entry a refused write was replacing.
    ///
    /// The store may still hold it. Where it does not, the index over-counts by
    /// one entry's length until something reads that key, which errs towards
    /// evicting too early rather than towards exceeding a bound, and the read
    /// path is where it is corrected.
    ///
    /// What was already evicted stays evicted. Eviction happens before the write
    /// because 0042 says it does, so a write the store then refuses has spent
    /// those entries, and there is nothing to put back with.
    fn put_replaced_back(&self, plan: &EvictionPlan) {
        let mut state = self.locked();
        put_back(&mut state, plan.tier, plan.replacing.clone());
    }

    /// Counts one refused write, and suspends writing where it was the third in
    /// a row.
    ///
    /// A REFUSAL IS ONE CALL THE STORE SAID NO TO, not one call a caller made.
    /// 0042 counts write refusals, and the second attempt 0054 adds is a second
    /// write, so a metadata write refused on both sides of an artwork release
    /// advances this by two. That reaches the suspension sooner on a device that
    /// is genuinely full, which is the conservative direction, and 0054's own
    /// sentence is that the suspension then applies to both tiers.
    fn count_refusal(&self, now: ElapsedInstant) {
        let report = {
            let mut state = self.locked();
            state.refusals_in_a_row += 1;
            if state.refusals_in_a_row >= REFUSALS_THAT_SUSPEND && state.suspended_at.is_none() {
                state.suspended_at = Some(now);
                Some(state.refusals_in_a_row)
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
    fn locked(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// What one write took out of the index before it asked the store for anything.
#[derive(Debug)]
struct EvictionPlan {
    /// The tier every entry below belongs to.
    tier: Tier,
    /// The copy this write replaces, where there was one. Empty or one entry.
    replacing: Vec<(EntryKey, Held)>,
    /// What has to leave the store, oldest first.
    evicting: Vec<(EntryKey, Held)>,
}

/// Whether writing is suspended at this moment, clearing a suspension that has
/// run out.
///
/// 0042 has the core attempt one write again when the interval is up. The run of
/// refusals is deliberately NOT reset here: a refusal on that attempt is the
/// next consecutive one and suspends writing again, which is what stops a full
/// device being asked hundreds of times. The run is reset by a write the store
/// accepted and by nothing else.
fn writing_is_suspended(state: &mut State, now: ElapsedInstant) -> bool {
    let Some(since) = state.suspended_at else {
        return false;
    };
    if now.interval_since(since) < SUSPENSION {
        return true;
    }
    state.suspended_at = None;
    false
}

/// Takes artwork out of the index, oldest first, until at least `target` bytes
/// have been released or there is nothing left that may go.
///
/// It releases what it can rather than refusing where the tier holds less than
/// the target, because the point of the rule is to make room for one metadata
/// write and a tier holding two hundred kibibytes has two hundred kibibytes to
/// give.
fn release_from_artwork(state: &mut State, target: u64) -> EvictionPlan {
    let mut evicting: Vec<(EntryKey, Held)> = Vec::new();
    let mut released = 0;
    while released < target {
        let victim = least_recently_used_not_being_read(state, Tier::Artwork)
            .and_then(|victim| take(state, Tier::Artwork, &victim));
        let Some(victim) = victim else {
            break;
        };
        released += victim.1.counted;
        evicting.push(victim);
    }
    EvictionPlan {
        tier: Tier::Artwork,
        replacing: Vec::new(),
        evicting,
    }
}

/// A read that is in flight, for as long as this value is alive.
///
/// A value with a destructor rather than a pair of calls, so that the key is
/// released whether the store answered, failed, or unwound. A key left marked as
/// being read is a key eviction could never choose again.
struct ReadInFlight<'c, 'a> {
    cache: &'c TieredCache<'a>,
    key: EntryKey,
}

impl<'c, 'a> ReadInFlight<'c, 'a> {
    fn started(cache: &'c TieredCache<'a>, key: &EntryKey) -> Self {
        *cache.locked().reading.entry(key.clone()).or_insert(0) += 1;
        Self {
            cache,
            key: key.clone(),
        }
    }
}

impl Drop for ReadInFlight<'_, '_> {
    fn drop(&mut self) {
        let mut state = self.cache.locked();
        if let Some(outstanding) = state.reading.get_mut(&self.key) {
            *outstanding -= 1;
            if *outstanding == 0 {
                state.reading.remove(&self.key);
            }
        }
    }
}

/// The length the core counts for these bytes.
///
/// `u64` rather than the platform's own width, so that a bound means the same
/// number on a thirty-two-bit television as on anything else.
fn counted_length(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

/// The oldest entry in one tier's use order that has no read in flight.
///
/// `None` where every entry there has one, which is the case 0042 answers by not
/// evicting rather than by cancelling a read.
fn least_recently_used_not_being_read(state: &State, tier: Tier) -> Option<EntryKey> {
    state.tiers[tier.at()]
        .order
        .values()
        .find(|key| !state.reading.contains_key(*key))
        .cloned()
}

/// Takes an entry out of one tier's index, with what it was.
fn take(state: &mut State, tier: Tier, key: &EntryKey) -> Option<(EntryKey, Held)> {
    let bookkeeping = &mut state.tiers[tier.at()];
    let held = bookkeeping.entries.remove(key)?;
    bookkeeping.order.remove(&held.used_at);
    bookkeeping.counted -= held.counted;
    Some((key.clone(), held))
}

/// Puts entries back into one tier exactly where they were, position in that
/// tier's order included.
fn put_back<I>(state: &mut State, tier: Tier, entries: I)
where
    I: IntoIterator<Item = (EntryKey, Held)>,
{
    let bookkeeping = &mut state.tiers[tier.at()];
    for (key, held) in entries {
        bookkeeping.counted += held.counted;
        bookkeeping.order.insert(held.used_at, key.clone());
        bookkeeping.entries.insert(key, held);
    }
}

/// Records an entry the store accepted, at the end of its tier's use order.
fn insert(state: &mut State, tier: Tier, key: &EntryKey, counted: u64) {
    let used_at = state.next_use;
    state.next_use += 1;
    let bookkeeping = &mut state.tiers[tier.at()];
    bookkeeping.counted += counted;
    bookkeeping.order.insert(used_at, key.clone());
    bookkeeping
        .entries
        .insert(key.clone(), Held { counted, used_at });
}

/// Moves an entry to the end of its tier's use order.
///
/// 0042: used means read out of the cache or written into it, and both move the
/// entry to the end of the order.
fn touch(state: &mut State, tier: Tier, key: &EntryKey) {
    let used_at = state.next_use;
    let bookkeeping = &mut state.tiers[tier.at()];
    let Some(held) = bookkeeping.entries.get_mut(key) else {
        return;
    };
    let was = held.used_at;
    held.used_at = used_at;
    bookkeeping.order.remove(&was);
    bookkeeping.order.insert(used_at, key.clone());
    state.next_use += 1;
}

#[cfg(test)]
mod tests {
    //! What the bounds, the eviction, the split and the giving-way rule are
    //! proven with.
    //!
    //! The store double here is not the one in the sibling module and the
    //! difference is deliberate. That one answers normally or is unavailable for
    //! everything at once, which is what 0040's own rules need. These conditions
    //! need four more things: a store that refuses writes while still answering
    //! reads, because 0042's device-is-full state has reads continuing
    //! throughout; a count of how many times the store was actually asked to
    //! write, because "stops attempting cache writes" is a statement about calls
    //! that were not made; a store that refuses only the first few writes, which
    //! is what a device with room freed under it looks like; and a read that can
    //! be held open, because the rule that eviction never reaches an entry with
    //! a read outstanding cannot be observed at all unless the window is held
    //! open from outside.
    //!
    //! The clock is supplied rather than read, which is 0102 and the
    //! `no-platform-clock` rule in `.github/invariants/rules`. Five minutes of
    //! suspension is therefore five minutes of arithmetic and not five minutes
    //! of waiting.

    use super::{
        BoundBelowTheFloor, CacheBounds, Cached, GIVE_WAY_FLOOR, MEBIBYTE, METADATA_FLOOR,
        SUSPENSION, Tier, TieredCache,
    };
    use crate::cache::{ByteStore, EntryKey, StorageUnavailable};
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use crate::diagnostics::{Diagnostics, DiagnosticsSink, Event, Severity};
    use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
    use std::collections::BTreeMap;
    use std::sync::{Condvar, Mutex};

    /// A read held open from outside, so that the window 0042's rule is about
    /// exists for as long as a condition needs it.
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
        /// held. Returns once the condition releases it.
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

    /// A byte store that keeps entries in memory, can be made to refuse reads,
    /// writes or removes on their own, counts what it was asked to do, and can
    /// hold one read open.
    #[derive(Debug, Default)]
    struct Store {
        held: Mutex<BTreeMap<String, Vec<u8>>>,
        refuse_reads: AtomicBool,
        refuse_writes: AtomicBool,
        refuse_removes: AtomicBool,
        /// How many more writes are refused before the store starts accepting
        /// them. Only read where `refuse_writes` is not set.
        refuse_the_next_writes: AtomicU32,
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

        /// A device that has no room until something frees some.
        fn refusing_the_next_writes(how_many: u32) -> Self {
            let store = Self::default();
            store
                .refuse_the_next_writes
                .store(how_many, Ordering::Relaxed);
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
            let left = self.refuse_the_next_writes.load(Ordering::Relaxed);
            if left > 0 {
                self.refuse_the_next_writes
                    .store(left - 1, Ordering::Relaxed);
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

    fn bounds_of(artwork: u64, metadata: u64) -> CacheBounds {
        CacheBounds::of(artwork, metadata)
            .expect("the fixture names a metadata bound above the floor")
    }

    /// Whole mebibytes of bytes, written out here so that no condition below
    /// has to convert a bound into a length and meet the cast that a
    /// thirty-two-bit target makes lossy.
    fn mebibytes(how_many: usize) -> Vec<u8> {
        vec![b'm'; how_many * 1024 * 1024]
    }

    /// One of them.
    fn a_mebibyte() -> Vec<u8> {
        mebibytes(1)
    }

    /// A hundred bytes, which is the unit most conditions below count in.
    fn a_hundred_bytes() -> Vec<u8> {
        vec![b'x'; 100]
    }

    // ---------------------------------------------------------------- bounds

    /// The default split is the two numbers 0054 states, and the total is their
    /// sum rather than a third value. That the numbers are the right ones is not
    /// something any run here judges: both records say they are chosen rather
    /// than measured and name #65 as where measured replacements come from.
    #[test]
    fn the_default_split_is_the_two_numbers_and_the_total_is_their_sum() {
        assert_eq!(CacheBounds::DEFAULT.of_tier(Tier::Artwork), 224 * MEBIBYTE);
        assert_eq!(CacheBounds::DEFAULT.of_tier(Tier::Metadata), 32 * MEBIBYTE);
        assert_eq!(CacheBounds::DEFAULT.total(), 256 * MEBIBYTE);
    }

    /// The total follows the tiers rather than being kept in agreement with
    /// them, which is what stops three numbers disagreeing.
    #[test]
    fn changing_a_tier_changes_the_total() {
        let bounds = bounds_of(10 * MEBIBYTE, 6 * MEBIBYTE);
        assert_eq!(bounds.total(), 16 * MEBIBYTE);
    }

    /// A metadata bound under the floor is refused, and the refusal names the
    /// floor rather than only saying no.
    #[test]
    fn a_metadata_bound_below_the_floor_is_refused_with_the_floor_named() {
        let refused = CacheBounds::of(MEBIBYTE, METADATA_FLOOR - 1)
            .expect_err("a bound below the floor is refused");
        assert_eq!(
            refused,
            BoundBelowTheFloor {
                tier: Tier::Metadata,
                floor: METADATA_FLOOR,
                asked: METADATA_FLOOR - 1,
            }
        );
        assert_eq!(refused.tier(), Tier::Metadata);
        assert_eq!(refused.floor(), 4 * MEBIBYTE);
        assert_eq!(refused.asked(), METADATA_FLOOR - 1);
    }

    /// Exactly the floor is admitted. The near miss is the one-byte difference
    /// beside it, which is the condition above.
    #[test]
    fn a_metadata_bound_exactly_at_the_floor_is_admitted() {
        let bounds = CacheBounds::of(0, METADATA_FLOOR).expect("the floor itself is admitted");
        assert_eq!(bounds.of_tier(Tier::Metadata), METADATA_FLOOR);
    }

    /// The artwork tier may be zero and the metadata tier may not, which is
    /// 0054's asymmetry rather than an oversight: a client that draws no artwork
    /// exists and a client that holds no metadata does not.
    #[test]
    fn the_artwork_tier_may_be_zero_and_the_metadata_tier_may_not() {
        assert!(CacheBounds::of(0, METADATA_FLOOR).is_ok());
        assert!(CacheBounds::of(0, 0).is_err());
    }

    /// A tier bounded at zero keeps nothing, rather than keeping one entry
    /// because nothing had to be evicted to make room for it.
    #[test]
    fn a_tier_bounded_at_zero_keeps_nothing() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, bounds_of(0, METADATA_FLOOR));

        assert_eq!(
            cache.write(Tier::Artwork, &key("a"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("empty"), &[]),
            Cached::NotKept
        );
        assert_eq!(cache.counted_bytes(Tier::Artwork), 0);
        assert_eq!(
            cache.write(Tier::Metadata, &key("m"), &a_hundred_bytes()),
            Cached::Kept
        );
    }

    // ------------------------------------------------------- the two orders

    /// The condition #54 states: fill the artwork tier past its bound and prove
    /// no metadata entry was evicted.
    ///
    /// The metadata entry is written first and never touched again, so it is the
    /// least recently used entry in the whole cache by a wide margin. Under one
    /// order it is the first thing to go.
    #[test]
    fn filling_the_artwork_tier_past_its_bound_evicts_no_metadata() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(500, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(
                Tier::Metadata,
                &key("the-library-listing"),
                &a_hundred_bytes()
            ),
            Cached::Kept
        );

        for at in 0..50 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("tile-{at:04}")),
                    &a_hundred_bytes()
                ),
                Cached::Kept
            );
            assert!(cache.counted_bytes(Tier::Artwork) <= 500);
        }

        assert_eq!(cache.counted_bytes(Tier::Artwork), 500);
        assert_eq!(cache.entries_held(Tier::Artwork), 5);
        assert_eq!(cache.counted_bytes(Tier::Metadata), 100);
        assert_eq!(cache.entries_held(Tier::Metadata), 1);
        assert!(
            store.holds(&key("the-library-listing")),
            "fifty artwork writes did not reach the other tier"
        );
    }

    /// And the other direction, which is the same property read the other way
    /// round: filling metadata past its bound evicts no artwork.
    #[test]
    fn filling_the_metadata_tier_past_its_bound_evicts_no_artwork() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(500, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("a-poster"), &a_hundred_bytes()),
            Cached::Kept
        );

        // A mebibyte an entry, so that the floor 0054 puts under this tier is
        // filled by a number of writes a condition can read rather than by
        // thousands.
        for at in 0..20 {
            assert_eq!(
                cache.write(
                    Tier::Metadata,
                    &key(&format!("item-{at:04}")),
                    &a_mebibyte()
                ),
                Cached::Kept
            );
        }

        assert_eq!(cache.counted_bytes(Tier::Metadata), METADATA_FLOOR);
        assert_eq!(cache.entries_held(Tier::Metadata), 4);
        assert_eq!(cache.counted_bytes(Tier::Artwork), 100);
        assert!(store.holds(&key("a-poster")));
    }

    /// Neither tier borrows from the other. An artwork tier at its bound evicts
    /// artwork while the metadata tier is empty, and the free space stays free.
    #[test]
    fn a_full_tier_does_not_borrow_from_an_empty_one() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(200, METADATA_FLOOR),
        );

        for at in 0..4 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("tile-{at:04}")),
                    &a_hundred_bytes()
                ),
                Cached::Kept
            );
        }

        assert_eq!(cache.counted_bytes(Tier::Artwork), 200);
        assert_eq!(cache.entries_held(Tier::Artwork), 2);
        assert_eq!(cache.counted_bytes(Tier::Metadata), 0);
    }

    /// Within a tier the entry evicted is the least recently used one, and 0042
    /// refuses the two alternatives it names. This is the half that separates
    /// least recently used from age order: the entry written first is the one
    /// still there, because it was read since.
    #[test]
    fn the_entry_evicted_is_the_least_recently_used_one_and_not_the_oldest() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(200, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("first"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("second"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert!(cache.read(Tier::Artwork, &key("first")).is_some());

        assert_eq!(
            cache.write(Tier::Artwork, &key("third"), &a_hundred_bytes()),
            Cached::Kept
        );

        assert!(
            store.holds(&key("first")),
            "reading it moved it out of the way"
        );
        assert!(
            !store.holds(&key("second")),
            "it was the least recently used"
        );
        assert!(store.holds(&key("third")));
    }

    /// Reading in one tier does not move anything in the other. The two orders
    /// share one counter and are still two orders.
    #[test]
    fn using_one_tier_does_not_move_the_order_of_the_other() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(200, METADATA_FLOOR),
        );
        let two_mebibytes = mebibytes(2);

        assert_eq!(
            cache.write(Tier::Metadata, &key("older-metadata"), &two_mebibytes),
            Cached::Kept
        );
        assert_eq!(
            cache.write(Tier::Metadata, &key("newer-metadata"), &two_mebibytes),
            Cached::Kept
        );
        for at in 0..10 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("tile-{at:04}")),
                    &a_hundred_bytes()
                ),
                Cached::Kept
            );
            assert!(
                cache
                    .read(Tier::Artwork, &key(&format!("tile-{at:04}")))
                    .is_some()
            );
        }

        assert_eq!(
            cache.write(Tier::Metadata, &key("arriving"), &two_mebibytes),
            Cached::Kept
        );
        assert!(
            !store.holds(&key("older-metadata")),
            "the oldest of the metadata order went, and no artwork decided that"
        );
        assert!(store.holds(&key("newer-metadata")));
    }

    // ------------------------------------------------ the read in flight

    /// 0042's rule, and the one it says will be forgotten because the window is
    /// the length of one call into client code.
    ///
    /// The gate holds a read of the oldest artwork entry open on a second thread
    /// while this one writes an entry that forces exactly one eviction.
    #[test]
    fn an_entry_with_a_read_in_flight_is_not_the_one_evicted() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(200, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("oldest"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("newer"), &a_hundred_bytes()),
            Cached::Kept
        );

        store.gate.arm(&key("oldest"));
        // Nothing is asserted inside the scope. A failed assertion there would
        // leave the reading thread parked on a gate nobody releases, and the run
        // would hang instead of going red, which is the shape a guard's deletion
        // proof has to be able to observe.
        let (offered, read) = std::thread::scope(|threads| {
            let reader = threads.spawn(|| cache.read(Tier::Artwork, &key("oldest")));

            store.gate.wait_until_inside();
            let offered = cache.write(Tier::Artwork, &key("arriving"), &a_hundred_bytes());
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
        assert!(!store.holds(&key("newer")), "it was the next in the order");
        assert!(store.holds(&key("arriving")));
    }

    /// Where every entry that could go has a read in flight there is no next
    /// entry to pick, and 0042's rule is that the read is not cancelled. So the
    /// write does not happen, and nothing about the call that caused it fails.
    #[test]
    fn a_write_that_could_only_fit_by_cancelling_a_read_does_not_happen() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(100, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("only"), &a_hundred_bytes()),
            Cached::Kept
        );

        store.gate.arm(&key("only"));
        // Asserted after the scope, for the reason the condition above states.
        let (offered, read) = std::thread::scope(|threads| {
            let reader = threads.spawn(|| cache.read(Tier::Artwork, &key("only")));

            store.gate.wait_until_inside();
            let offered = cache.write(Tier::Artwork, &key("arriving"), &a_hundred_bytes());
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
    }

    // ------------------------------------------------- the full device

    /// A store that refuses writes, and a core that degrades to working without
    /// a cache rather than failing.
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
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        for at in 0..5 {
            assert_eq!(
                cache.write(
                    Tier::Metadata,
                    &key(&format!("{at:04}")),
                    &a_hundred_bytes()
                ),
                Cached::NotKept
            );
        }

        assert_eq!(cache.counted_bytes(Tier::Metadata), 0);
        assert_eq!(
            cache.read(Tier::Metadata, &key("0000")),
            None,
            "an absence rather than a failure"
        );

        store.refuse_writes.store(false, Ordering::Relaxed);
        clocks.advance(SUSPENSION);
        assert_eq!(
            cache.write(Tier::Metadata, &key("later"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert_eq!(
            cache.read(Tier::Metadata, &key("later")),
            Some(a_hundred_bytes())
        );
    }

    /// After three consecutive refusals the store is not asked again, and the
    /// suspension is reported once rather than once per write. The tier that met
    /// the refusals has no artwork to release, so no second attempt is made and
    /// each write is one refusal.
    #[test]
    fn three_refusals_in_a_row_stop_the_core_asking_and_are_reported_once() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        for at in 0..3 {
            assert_eq!(
                cache.write(Tier::Artwork, &key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }
        assert_eq!(store.attempts(), 3);
        assert_eq!(collector.named("cache.writing-suspended"), 1);

        for at in 3..20 {
            assert_eq!(
                cache.write(Tier::Artwork, &key(&format!("{at:04}")), &a_hundred_bytes()),
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

    /// A suspension reached in one tier applies to the other, which is 0054's
    /// sentence about what happens after the giving-way rule has been tried.
    #[test]
    fn a_suspension_reached_on_artwork_applies_to_metadata_too() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        for at in 0..3 {
            assert_eq!(
                cache.write(Tier::Artwork, &key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }
        assert_eq!(store.attempts(), 3);

        assert_eq!(
            cache.write(Tier::Metadata, &key("metadata"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(store.attempts(), 3, "the other tier was not asked either");
    }

    /// Reads continue throughout a suspension, because a device with no room can
    /// still be read and what is already held is what matters most at exactly
    /// that moment.
    #[test]
    fn reads_are_answered_while_writing_is_suspended() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        assert_eq!(
            cache.write(Tier::Metadata, &key("held"), &a_hundred_bytes()),
            Cached::Kept
        );
        store.refuse_writes.store(true, Ordering::Relaxed);
        for at in 0..3 {
            assert_eq!(
                cache.write(Tier::Artwork, &key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }

        assert_eq!(
            cache.read(Tier::Metadata, &key("held")),
            Some(a_hundred_bytes())
        );
    }

    /// When the interval is up the core attempts one write again. The run of
    /// refusals is not reset by the interval running out, so a refusal on that
    /// attempt is the next consecutive one and suspends writing again.
    #[test]
    fn writing_returns_when_the_interval_is_up_and_one_more_refusal_suspends_it_again() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        for at in 0..3 {
            assert_eq!(
                cache.write(Tier::Artwork, &key(&format!("{at:04}")), &a_hundred_bytes()),
                Cached::NotKept
            );
        }
        assert_eq!(store.attempts(), 3);

        clocks.advance(SUSPENSION);
        assert_eq!(
            cache.write(Tier::Artwork, &key("probe"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(
            store.attempts(),
            4,
            "the interval was up, so one write was attempted"
        );
        assert_eq!(collector.named("cache.writing-suspended"), 2);

        assert_eq!(
            cache.write(Tier::Artwork, &key("again"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(store.attempts(), 4, "and writing is suspended again");
    }

    /// A write the store accepts is what ends a run of refusals, and nothing
    /// else.
    #[test]
    fn a_write_the_store_accepted_is_what_ends_a_run_of_refusals() {
        let store = Store::default();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        store.refuse_writes.store(true, Ordering::Relaxed);
        assert_eq!(
            cache.write(Tier::Artwork, &key("a"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("b"), &a_hundred_bytes()),
            Cached::NotKept
        );

        store.refuse_writes.store(false, Ordering::Relaxed);
        assert_eq!(
            cache.write(Tier::Artwork, &key("c"), &a_hundred_bytes()),
            Cached::Kept
        );

        store.refuse_writes.store(true, Ordering::Relaxed);
        assert_eq!(
            cache.write(Tier::Artwork, &key("d"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("e"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(collector.named("cache.writing-suspended"), 0);
    }

    // ------------------------------------------- where artwork gives way

    /// 0054's one asymmetry. A refused metadata write releases artwork and is
    /// attempted once more; the store accepts the second attempt because the
    /// device now has room.
    #[test]
    fn a_refused_metadata_write_releases_artwork_and_is_attempted_again() {
        let store = Store::refusing_the_next_writes(0);
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(4 * MEBIBYTE, METADATA_FLOOR),
        );

        // Two mebibytes of artwork, in entries large enough that one release
        // round takes a small number of them.
        for at in 0..8 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("tile-{at:04}")),
                    &vec![b'x'; 256 * 1024]
                ),
                Cached::Kept
            );
        }
        assert_eq!(cache.counted_bytes(Tier::Artwork), 2 * MEBIBYTE);

        // The device has no room for the next write and gets some back.
        store.refuse_the_next_writes.store(1, Ordering::Relaxed);
        let attempts_before = store.attempts();
        assert_eq!(
            cache.write(
                Tier::Metadata,
                &key("the-library-listing"),
                &a_hundred_bytes()
            ),
            Cached::Kept
        );

        assert_eq!(
            store.attempts() - attempts_before,
            2,
            "refused once, then attempted again after artwork was released"
        );
        assert!(
            cache.counted_bytes(Tier::Artwork) <= 2 * MEBIBYTE - GIVE_WAY_FLOOR,
            "at least the floor was released, and {} is what is left",
            cache.counted_bytes(Tier::Artwork)
        );
        assert_eq!(cache.counted_bytes(Tier::Metadata), 100);
        assert_eq!(collector.named("cache.artwork-gave-way"), 1);
    }

    /// It releases at least eight times the refused write, not exactly what was
    /// needed, because freeing exactly enough buys one write and then the next
    /// one does the same work again.
    #[test]
    fn what_is_released_is_at_least_eight_times_the_refused_write() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(8 * MEBIBYTE, METADATA_FLOOR),
        );

        // Sixteen entries of 256 KiB, so a release round can stop on a boundary
        // rather than having to take the whole tier.
        for at in 0..16 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("tile-{at:04}")),
                    &vec![b'x'; 256 * 1024]
                ),
                Cached::Kept
            );
        }
        let before = cache.counted_bytes(Tier::Artwork);
        assert_eq!(before, 4 * MEBIBYTE);

        // A refused write of 512 KiB asks for at least 4 MiB back.
        store.refuse_the_next_writes.store(1, Ordering::Relaxed);
        assert_eq!(
            cache.write(
                Tier::Metadata,
                &key("large-listing"),
                &vec![b'm'; 512 * 1024]
            ),
            Cached::Kept
        );

        let released = before - cache.counted_bytes(Tier::Artwork);
        assert!(
            released >= 8 * 512 * 1024,
            "eight times the refused write is {} and {released} was released",
            8 * 512 * 1024
        );
    }

    /// A refused ARTWORK write never releases metadata, is not attempted again,
    /// and the entry is simply not cached. This is the direction 0054 refuses,
    /// and written the natural way round it would free metadata to make room for
    /// a picture.
    #[test]
    fn a_refused_artwork_write_never_releases_metadata() {
        let store = Store::default();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(MEBIBYTE, METADATA_FLOOR),
        );

        for at in 0..4 {
            assert_eq!(
                cache.write(
                    Tier::Metadata,
                    &key(&format!("item-{at:04}")),
                    &a_hundred_bytes()
                ),
                Cached::Kept
            );
        }
        assert_eq!(cache.counted_bytes(Tier::Metadata), 400);

        store.refuse_the_next_writes.store(1, Ordering::Relaxed);
        let attempts_before = store.attempts();
        assert_eq!(
            cache.write(Tier::Artwork, &key("a-poster"), &a_hundred_bytes()),
            Cached::NotKept
        );

        assert_eq!(
            store.attempts() - attempts_before,
            1,
            "refused once and not attempted again"
        );
        assert_eq!(
            cache.counted_bytes(Tier::Metadata),
            400,
            "no metadata was released for a picture"
        );
        assert_eq!(cache.entries_held(Tier::Metadata), 4);
        assert_eq!(collector.named("cache.artwork-gave-way"), 0);
    }

    /// With no artwork to release there is no second attempt, because attempting
    /// the write again after freeing nothing is a second call into a store that
    /// has already said no.
    #[test]
    fn with_no_artwork_to_release_a_refused_metadata_write_is_not_attempted_again() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(&store, &clocks, &diagnostics, CacheBounds::DEFAULT);

        assert_eq!(
            cache.write(Tier::Metadata, &key("listing"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(store.attempts(), 1);
        assert_eq!(collector.named("cache.artwork-gave-way"), 0);
    }

    /// A metadata write the device refuses on both sides of a release counts two
    /// refusals, because a refusal is one call the store said no to. Two such
    /// writes reach the suspension.
    #[test]
    fn a_metadata_write_refused_on_both_sides_of_a_release_counts_twice() {
        let store = Store::refusing_writes();
        let clocks = Moving::default();
        let collector = Collector::default();
        let diagnostics = Diagnostics::new(&clocks, Some(&collector), Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(4 * MEBIBYTE, METADATA_FLOOR),
        );

        // Artwork put there while the store was answering, so there is something
        // to release once it stops.
        store.refuse_writes.store(false, Ordering::Relaxed);
        for at in 0..8 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("tile-{at:04}")),
                    &vec![b'x'; 256 * 1024]
                ),
                Cached::Kept
            );
        }
        store.refuse_writes.store(true, Ordering::Relaxed);

        assert_eq!(
            cache.write(Tier::Metadata, &key("one"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(collector.named("cache.writing-suspended"), 0);

        assert_eq!(
            cache.write(Tier::Metadata, &key("two"), &a_hundred_bytes()),
            Cached::NotKept
        );
        assert_eq!(
            collector.named("cache.writing-suspended"),
            1,
            "two writes, four refusals, and the suspension arrived on the third"
        );
    }

    // ------------------------------------------------ the smaller rules

    /// Bytes no number of evictions could make room for empty nothing.
    #[test]
    fn bytes_larger_than_a_tiers_whole_bound_evict_nothing() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(150, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("held"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("enormous"), &[b'x'; 151]),
            Cached::NotKept
        );

        assert!(
            store.holds(&key("held")),
            "nothing was spent on a write that could not fit"
        );
        assert!(!store.holds(&key("enormous")));
        assert_eq!(cache.counted_bytes(Tier::Artwork), 100);
    }

    /// A write replaces whatever was there, so the copy being replaced is not in
    /// the way of its own replacement and it is counted once rather than twice.
    #[test]
    fn a_write_that_replaces_an_entry_is_counted_once() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(100, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("one"), &a_hundred_bytes()),
            Cached::Kept
        );
        assert_eq!(
            cache.write(Tier::Artwork, &key("one"), &[b'y'; 100]),
            Cached::Kept
        );

        assert_eq!(cache.counted_bytes(Tier::Artwork), 100);
        assert_eq!(cache.entries_held(Tier::Artwork), 1);
        assert_eq!(
            cache.read(Tier::Artwork, &key("one")),
            Some(vec![b'y'; 100])
        );
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
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(100, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("held"), &a_hundred_bytes()),
            Cached::Kept
        );
        store.refuse_removes.store(true, Ordering::Relaxed);

        for _ in 0..5 {
            assert_eq!(
                cache.write(Tier::Artwork, &key("arriving"), &a_hundred_bytes()),
                Cached::NotKept
            );
        }

        assert_eq!(cache.counted_bytes(Tier::Artwork), 100);
        assert_eq!(cache.entries_held(Tier::Artwork), 1);
        assert!(store.holds(&key("held")));
        assert_eq!(collector.named("cache.writing-suspended"), 0);
    }

    /// The index believing in an entry the store does not have is corrected by
    /// the read that finds it absent, rather than being carried until the tier
    /// has no budget left.
    #[test]
    fn an_entry_the_store_no_longer_has_leaves_the_index_on_the_next_read() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(100, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("held"), &a_hundred_bytes()),
            Cached::Kept
        );
        store.entries().remove("held");

        assert_eq!(cache.read(Tier::Artwork, &key("held")), None);
        assert_eq!(cache.counted_bytes(Tier::Artwork), 0);
        assert_eq!(cache.entries_held(Tier::Artwork), 0);
    }

    /// A store that could not be read is an absence to the caller and leaves the
    /// accounting where it was, because nothing was learned about what the store
    /// holds. That is the opposite direction from the condition above, and
    /// collapsing the two would empty the index every time a device was locked
    /// in the background.
    #[test]
    fn a_store_that_could_not_be_read_leaves_the_accounting_alone() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(100, METADATA_FLOOR),
        );

        assert_eq!(
            cache.write(Tier::Artwork, &key("held"), &a_hundred_bytes()),
            Cached::Kept
        );
        store.refuse_reads.store(true, Ordering::Relaxed);

        assert_eq!(cache.read(Tier::Artwork, &key("held")), None);
        assert_eq!(cache.counted_bytes(Tier::Artwork), 100);
        assert_eq!(cache.entries_held(Tier::Artwork), 1);
    }

    /// The bounds a client chose are the ones enforced, rather than the defaults
    /// quietly standing behind them, and each tier answers for itself.
    #[test]
    fn the_bounds_a_client_chose_are_the_ones_enforced() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(250, METADATA_FLOOR),
        );

        assert_eq!(cache.bounds().of_tier(Tier::Artwork), 250);
        assert_eq!(cache.bounds().of_tier(Tier::Metadata), METADATA_FLOOR);
        assert_eq!(cache.bounds().total(), 250 + METADATA_FLOOR);

        for at in 0..10 {
            assert_eq!(
                cache.write(
                    Tier::Artwork,
                    &key(&format!("a{at:04}")),
                    &a_hundred_bytes()
                ),
                Cached::Kept
            );
            assert_eq!(
                cache.write(Tier::Metadata, &key(&format!("m{at:04}")), &a_mebibyte()),
                Cached::Kept
            );
        }
        assert_eq!(cache.counted_bytes(Tier::Artwork), 200);
        assert_eq!(cache.counted_bytes(Tier::Metadata), METADATA_FLOOR);
    }

    /// Every tier answers both accounting questions, applied to the whole set
    /// rather than to whichever member somebody remembered.
    #[test]
    fn every_tier_is_accounted_for_separately() {
        let store = Store::default();
        let clocks = Moving::default();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail);
        let cache = TieredCache::new(
            &store,
            &clocks,
            &diagnostics,
            bounds_of(MEBIBYTE, METADATA_FLOOR),
        );

        for tier in Tier::all() {
            assert_eq!(cache.counted_bytes(*tier), 0);
            assert_eq!(cache.entries_held(*tier), 0);
            assert_eq!(
                cache.write(*tier, &key(tier.as_str()), &a_hundred_bytes()),
                Cached::Kept
            );
            assert_eq!(cache.counted_bytes(*tier), 100);
            assert_eq!(cache.entries_held(*tier), 1);
        }
    }
}
