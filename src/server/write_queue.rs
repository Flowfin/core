//! The order, the coalescing and the bound every write to a server is held
//! under.
//!
//! `docs/decisions/0047-the-write-queue.md` is the record and #47 is the issue.
//! The record decides one queue and four properties of it: that the order is a
//! counter stored with the queue rather than any clock, that two actions
//! touching one target with one kind are one entry coalesced at the moment of
//! enqueue, that the queue is bounded and an overflow drops the OLDEST entry and
//! reports it, and that nothing here is ever expired by age.
//!
//! # Why it is here rather than beside the store
//!
//! 0047 opens with the sentence that decides this: the queue is not the offline
//! path, it is THE path, and every write the core makes to a server goes onto it
//! whether the server is answering or not. So it is part of reaching a server,
//! and it sits beside [`super::recovery`], which is what it drains on rather
//! than on a timer of its own.
//!
//! The alternative was `crate::cache`, on the strength of 0047 sending the bytes
//! to the store 0040 defines. That is where a queue is KEPT, and this module is
//! not where anything is kept: nothing below writes a byte anywhere. Placing it
//! by its storage would put the write path inside the module 0003 describes as
//! caching what was fetched.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of 0047 that a counter and an equality settle:
//! which entry a new action replaces, where a replacement sits in the order,
//! which entry an overflow drops, what the queue reports afterwards, and the
//! order a drain walks in.
//!
//! WHAT IS NOT HERE IS THE DELIVERY. Nothing in this tree opens a connection,
//! for the reason [`super::transport`] gives about itself, so nothing is sent,
//! nothing is acknowledged and no drain runs. This module holds the queue such a
//! drain would walk. #47's two conditions restart the core and restore a server,
//! and neither is met by anything here.
//!
//! WHAT IS ALSO NOT HERE IS DURABILITY. 0047 puts the bytes in the store 0040
//! defines, keyed under #41, and neither the store interface's caller nor that
//! keying is built. So this queue lives as long as the value does, and the
//! counter it carries is what a restore would restore rather than something that
//! survives one today.
//!
//! THIS PARAGRAPH SAID THE AGE WAS NOT HERE. It said 0047 stores two moments per
//! entry and computes an age the way 0043 computes a cache entry's, that the
//! reading belongs with the two guards [`crate::cache::freshness`] already
//! carries, and that no function below takes a clock reading at all. The first
//! half is built: an entry carries the pair 0047 names and answers
//! [`Entry::age_at`] from it. The second half is what building it had to keep,
//! and it is kept by borrowing rather than by discipline - the pair, the
//! correction and the two guards are [`crate::cache::freshness`]'s own types, so
//! there is no second arithmetic here to drift from the one 0043 fixed. The third
//! is unchanged: every moment below arrives as an argument, nothing here reads a
//! clock, and an age is computed only where somebody asks for one.
//!
//! WHAT THE AGE DOES IS NOTHING, AND THAT IS 0047'S RULE RATHER THAN AN
//! UNFINISHED HALF. It is carried for reporting: not a reason to drop an entry,
//! not an input to the bound, and not a threshold. The absence used to be held by
//! there being no age at all, which is a guarantee that ends the moment one
//! arrives, so what holds it now is a case: an entry whose age is unreadable and
//! an entry a year old are both answered by the head and both delivered.
//!
//! THE DROP REPORT IS MADE HERE AND THE STANDING COUNT IS BESIDE IT. 0047 asks
//! for both, because the two answer different people: an event reaches a client
//! that was listening at the moment it happened, and a count reaches one that
//! was not. The event carries the kind and the correlator 0071 defines for the
//! target, never the identifier, and [`Diagnostics`] is what turns the second
//! into the first, so the reduction is decided at the boundary rather than by
//! this module remembering to do it.
//!
//! # The number here is chosen and not measured
//!
//! 0047 says so of its bound, and says what makes a thousand defensible: with
//! coalescing at enqueue it is a thousand distinct items somebody touched rather
//! than a thousand actions taken. #65 is the harness that would replace it with
//! a measured number.

use crate::cache::freshness::{Age, Skew, WrittenAt};
use crate::clock::WallMoment;
use crate::diagnostics::redaction::FieldName;
use crate::diagnostics::{Diagnostics, EventName, Field, FieldValue, Severity};

/// The event 0047 owes at the moment an entry is dropped at the bound.
///
/// At `failure` rather than `notice`, which is 0105's sentence about a dropped
/// queue entry and its reason: a cache entry can be fetched again and a person's
/// own action cannot, so what did not happen here is something somebody did
/// reaching the server. A client filtering `notice` out is a client that would
/// stop seeing the one thing 0047 exists to prevent being silent about.
const AN_ENTRY_WAS_DROPPED: EventName = EventName::declared("write-queue.entry-dropped");

/// Which item the dropped entry was about.
///
/// Reduced, so it leaves as the correlator 0071 defines and never as the
/// identifier. That is 0047's own requirement for this report, and it is what
/// makes two drops for one item readable as one item rather than as two.
pub(crate) const FOR_TARGET: FieldName = FieldName::reduced("for-target");

/// Which statement about that item was dropped.
///
/// Carried whole: it is one of a fixed set this module declares, so two people
/// running the same build against the same server cannot hold different values
/// for it, which is 0068's question and the one 0071's first treatment is for.
pub(crate) const ASSERTED_ABOUT: FieldName = FieldName::carried_whole("asserted-about");

/// The entries one session's queue holds before an overflow drops something.
///
/// From 0047, chosen rather than measured. What defends it is the coalescing
/// rule rather than the number: a thousand entries is a thousand distinct
/// targets somebody touched while the server was away, which is far outside what
/// a month offline produces.
pub const A_SESSIONS_QUEUE_HOLDS_AT_MOST: usize = 1000;

/// Which statement about a target an entry carries.
///
/// 0047 coalesces per kind as well as per target, because a position report and
/// a watched mark are two different statements about one item and neither
/// replaces the other.
///
/// THE SET IS WHAT LANDED RECORDS NAME AND NOTHING HERE DECIDES IT. 0047 fixes
/// the rule and names no kinds. The two below are the two other records already
/// state - 0057 makes a position the kind a progress report carries, and 0060
/// fixes what a watched mark is - and a third arrives with the issue that adds
/// the action rather than with this module.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhatIsAsserted {
    /// Where playback reached, which 0057 reports on its cadence and 0058 reads
    /// when playback resumes.
    PlaybackPosition,
    /// That an item has been watched, which 0060 fixes the meaning of.
    Watched,
}

impl WhatIsAsserted {
    /// Every kind, so that a caller reads the set out of the crate rather than
    /// keeping a copy of it, and so a case applies a rule to the whole of it
    /// rather than to whichever member somebody remembered.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::PlaybackPosition, Self::Watched]
    }

    /// The kind as it is reported.
    ///
    /// This is what an event carries rather than the text a debug printing would
    /// produce, which 0100 requires: a field is data a client reads, and a name
    /// that changed when somebody renamed a variant would change what every
    /// client's report says.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlaybackPosition => "playback-position",
            Self::Watched => "watched",
        }
    }
}

/// The item an entry is about.
///
/// It is the identifier the server knows the item by, held whole because the
/// queue has to name the item to the server when it drains.
///
/// WHAT IT IS NOT IS SOMETHING THAT MAY BE REPORTED. 0047 says a drop is
/// reported carrying the correlator for its target under 0071 rather than the
/// identifier itself, so the identifier is written out nowhere. That is what the
/// hand-written formatting below is for, and a derived one would put it in every
/// report that formatted an entry.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Target {
    identifier: String,
}

/// Written out by hand for the reason 0047 gives about a drop report: the
/// correlator names a target and the identifier never does.
impl core::fmt::Debug for Target {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Target").finish_non_exhaustive()
    }
}

impl Target {
    /// The item this entry is about, as the server names it.
    #[must_use]
    pub const fn item(identifier: String) -> Self {
        Self { identifier }
    }

    /// The identifier, for the request that names the item.
    ///
    /// Public because the request that carries it is written outside this
    /// module, and crate-private would move the seam rather than remove it. What
    /// keeps it out of a report is 0071's treatment at the diagnostics boundary
    /// and the formatting above, neither of which this accessor goes through.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.identifier
    }
}

/// One thing somebody did, waiting to be told to the server.
///
/// `A` is the asserted state itself. It is a parameter rather than a fixed set
/// because 0047's rules are about the queue and not about what an action says,
/// and the actions arrive with the issues that add them: a progress report is
/// #57's and a watched mark is #60's. What 0047 does require of every one of
/// them is that it is an assertion of a desired state rather than a delta, so
/// that delivering it twice is the same as delivering it once. THAT IS A
/// PROPERTY OF THE VALUE A CALLER PUTS IN AND NOTHING HERE JUDGES IT.
///
/// Thread safety, from 0009: a plain value, safe from any thread where what it
/// carries is.
#[derive(Clone, PartialEq, Eq)]
pub struct Entry<A> {
    order: u64,
    target: Target,
    asserted_about: WhatIsAsserted,
    assertion: A,
    enqueued_at: WrittenAt,
}

/// Written out by hand so an entry cannot carry its target's identifier into an
/// output through the field, which is the leak [`Target`]'s own shape is
/// against.
impl<A> core::fmt::Debug for Entry<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entry")
            .field("order", &self.order)
            .field("asserted_about", &self.asserted_about)
            .finish_non_exhaustive()
    }
}

impl<A> Entry<A> {
    /// Where this entry stands in the order somebody's actions were taken.
    ///
    /// A counter rather than a moment on any of 0102's three clocks. A clock
    /// would reorder somebody's own actions when the device clock is corrected,
    /// when it suspends, or across a restart, and nothing in the result would
    /// say a clock was involved.
    #[must_use]
    pub const fn order(&self) -> u64 {
        self.order
    }

    /// The item this entry is about.
    #[must_use]
    pub const fn target(&self) -> &Target {
        &self.target
    }

    /// Which statement about that item this entry carries.
    #[must_use]
    pub const fn asserted_about(&self) -> WhatIsAsserted {
        self.asserted_about
    }

    /// The asserted state, for the request that would deliver it.
    #[must_use]
    pub const fn assertion(&self) -> &A {
        &self.assertion
    }

    /// The two moments this entry was enqueued with.
    ///
    /// 0047 names them: the server's own last stated time, and the device's wall
    /// reading at the instant the action was queued. They are what an age
    /// survives a restart on, and 0047 spends its argument on the alternative -
    /// a restored queue that treated every entry as freshly enqueued would keep
    /// its order and lose every age, so a client could say only that something is
    /// pending.
    ///
    /// It is [`WrittenAt`] rather than a pair of this module's own because 0047
    /// says the age is computed the way 0043 computes a cache entry's, and one
    /// type is how that stays true of a second arithmetic nobody wrote.
    #[must_use]
    pub const fn enqueued_at(&self) -> WrittenAt {
        self.enqueued_at
    }

    /// How long this entry has been waiting, on 0102's anchor.
    ///
    /// The whole of the computation is [`Age::at_read`]'s, with the same
    /// correction and the same two guards, which is what 0047 asks for by naming
    /// 0043 rather than describing an arithmetic of its own. `the_skew_now` is
    /// `None` where there is no current measurement to correct against, which is
    /// the ordinary case for a queue: the server the entry is waiting for is the
    /// server that has not been reachable.
    ///
    /// WHAT THIS ANSWER IS FOR IS A SENTENCE A CLIENT SAYS, AND NOTHING HERE
    /// READS IT. 0047 makes the age reporting and never a threshold: an action
    /// somebody took is not less true because their device was off, and expiring
    /// one would be the silent discard that record exists against arriving
    /// through a mechanism that looks like hygiene.
    #[must_use]
    pub fn age_at(&self, the_devices_wall_now: WallMoment, the_skew_now: Option<Skew>) -> Age {
        Age::at_read(self.enqueued_at, the_devices_wall_now, the_skew_now)
    }
}

/// What an entry that was dropped at the bound was about.
///
/// 0047 requires a drop to be reported at the moment it happens, through the
/// interface in 0100, carrying the kind of action and the correlator for its
/// target rather than the identifier. THE ENQUEUE MAKES THAT REPORT ITSELF AND
/// THIS PARAGRAPH SAID IT WAS HANDED BACK FOR SOMEBODY ELSE TO MAKE. Handing it
/// back left the record's "every drop" resting on every caller remembering, and
/// a caller that ignores the answer produces exactly the silent discard the
/// whole record exists against. What is still handed back is this value, because
/// a caller may want to say something of its own about what was lost.
///
/// IT CARRIES NO ASSERTION. What was dropped is gone, and holding the value
/// would put a person's own action into the type whose whole purpose is to say
/// that it was lost.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    /// The item the dropped entry was about.
    pub target: Target,
    /// Which statement about that item was dropped.
    pub asserted_about: WhatIsAsserted,
    /// Where the dropped entry stood in the order.
    pub order: u64,
}

/// What an enqueue did, which is one of three things and never nothing.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheEnqueueDid {
    /// A target and kind the queue was not already holding, added at the end of
    /// the order.
    Added,
    /// A target and kind the queue was already holding. The later assertion
    /// replaced the earlier one and kept its position in the order, which is
    /// what makes the replacement invisible to the order.
    ReplacedInPlace,
    /// The queue was at [`A_SESSIONS_QUEUE_HOLDS_AT_MOST`], so the oldest entry
    /// was dropped to make room.
    ///
    /// The drop is not a return value a caller may ignore quietly: 0047 requires
    /// it reported as it happens AND kept as a standing count, and
    /// [`WriteQueue::dropped`] is the second half.
    DroppedTheOldest(Dropped),
}

/// One session's queue of writes.
///
/// It is per session, which 0005 already fixes and which follows from the keying
/// in #41: an action belongs to the account that took it, on the server it was
/// taken against, and a second account signing in on that device has its own
/// queue and cannot drain somebody else's. This type holds one of them rather
/// than a set.
///
/// Thread safety, from 0009: safe from any thread where what it carries is. It
/// is a plain value with no interior mutability, so a caller sharing one across
/// threads gives it the same treatment as any other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteQueue<A> {
    entries: Vec<Entry<A>>,
    next_order: u64,
    dropped: u64,
}

impl<A> Default for WriteQueue<A> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<A> WriteQueue<A> {
    /// A queue holding nothing, with the counter at its first value.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
            next_order: 1,
            dropped: 0,
        }
    }

    /// How many entries the queue is holding.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the queue is holding nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many entries this session's queue has dropped at the bound.
    ///
    /// The standing count 0047 requires beside the event, so that a client can
    /// tell an operator something was lost without having been listening at the
    /// moment it was. It is per session and a drain does not clear it: what it
    /// counts is what was never delivered, and delivering something else says
    /// nothing about that.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Every entry, in the order somebody's actions were taken.
    #[must_use]
    pub fn entries(&self) -> &[Entry<A>] {
        &self.entries
    }

    /// Put one action on the queue.
    ///
    /// Coalescing happens HERE rather than at the drain, and 0047 calls that the
    /// decision rather than an implementation detail. At the drain the queue
    /// holds every one of the ninety positions somebody produced while scrubbing
    /// through a film, so the bound is reached by activity rather than by
    /// breadth and the person punished by it is the one who used the application
    /// most.
    ///
    /// A replacement keeps the earlier entry's position in the order. A person
    /// who marks an episode watched, plays something else, then changes their
    /// mind about the episode has told the server two things about the episode
    /// and one about the other item, and the last thing they said about the
    /// episode is what stands.
    ///
    /// AT THE BOUND THE OLDEST GOES AND NOT THE NEWEST. The alternative refuses
    /// to record what somebody just did while holding something from three weeks
    /// ago, which is the version of the failure they are in front of.
    ///
    /// A DROP IS REPORTED HERE AND NOT BY THE CALLER. 0047 says every drop is
    /// reported at the moment it happens, and the moment it happens is inside
    /// this call. The diagnostics facility is a parameter rather than something
    /// the queue holds, for the reason every clock reading is one: this module
    /// owns no facility of the client's and reaches for nothing.
    ///
    /// The order the dropped entry stood in is on the value handed back and not
    /// on the event. 0047 names two things the report carries, the kind and the
    /// correlator, and a counter meaningful only inside one queue is not one of
    /// them.
    ///
    /// A REPLACEMENT KEEPS THE EARLIER ENTRY'S TWO MOMENTS, as it keeps its
    /// position in the order, and 0047 says only the second of those. Taking the
    /// later action's moments is the shape that reads as obvious - the statement
    /// standing is the new one - and it reports every actively touched entry as
    /// freshly enqueued, so a queue undelivered for a month says seconds for the
    /// items somebody kept scrubbing. That is the failure 0047 names for a
    /// restored queue arriving through the coalescing door instead, and it lands
    /// on the person who used the application most, which is the same person
    /// coalescing at enqueue exists to protect.
    pub fn enqueue(
        &mut self,
        target: Target,
        asserted_about: WhatIsAsserted,
        assertion: A,
        enqueued_at: WrittenAt,
        diagnostics: &Diagnostics<'_>,
    ) -> WhatTheEnqueueDid {
        if let Some(held) = self
            .entries
            .iter_mut()
            .find(|entry| entry.target == target && entry.asserted_about == asserted_about)
        {
            held.assertion = assertion;
            return WhatTheEnqueueDid::ReplacedInPlace;
        }

        let mut what_it_did = WhatTheEnqueueDid::Added;
        if self.entries.len() >= A_SESSIONS_QUEUE_HOLDS_AT_MOST {
            let oldest = self.entries.remove(0);
            self.dropped = self.dropped.saturating_add(1);
            let dropped = Dropped {
                target: oldest.target,
                asserted_about: oldest.asserted_about,
                order: oldest.order,
            };
            diagnostics.emit(
                Severity::Failure,
                AN_ENTRY_WAS_DROPPED,
                &[
                    Field::new(FOR_TARGET, FieldValue::Text(dropped.target.as_str())),
                    Field::new(
                        ASSERTED_ABOUT,
                        FieldValue::Text(dropped.asserted_about.as_str()),
                    ),
                ],
            );
            what_it_did = WhatTheEnqueueDid::DroppedTheOldest(dropped);
        }

        self.entries.push(Entry {
            order: self.next_order,
            target,
            asserted_about,
            assertion,
            enqueued_at,
        });
        self.next_order = self.next_order.saturating_add(1);
        what_it_did
    }

    /// The entry a drain would try next, which is the earliest in the order.
    ///
    /// A drain walks in counter order and STOPS at the first entry it could not
    /// deliver, rather than skipping it. Continuing past it would deliver a
    /// later action for one target ahead of an earlier one for a different
    /// target, and the order somebody's actions arrive in is the only thing the
    /// server can use to reconstruct what they did. That is why there is a head
    /// and no iterator that removes as it goes: a caller that failed simply
    /// stops calling [`WriteQueue::after_it_was_delivered`].
    #[must_use]
    pub fn next_to_deliver(&self) -> Option<&Entry<A>> {
        self.entries.first()
    }

    /// Take the head off, after it reached the server.
    ///
    /// Answers the entry that was delivered, or `None` for an empty queue.
    ///
    /// It is separate from [`WriteQueue::next_to_deliver`] so that an entry is
    /// removed only where a delivery is claimed. A single call that answered and
    /// removed at once would lose an entry to a delivery that failed, which is
    /// the silent discard this whole record exists against.
    pub fn after_it_was_delivered(&mut self) -> Option<Entry<A>> {
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    //! 0047's order, coalescing and bound, asked of the values.
    //!
    //! What these cannot ask is either of #47's two conditions. Both restart the
    //! core and restore a server, and nothing in this tree does either.

    use super::{
        A_SESSIONS_QUEUE_HOLDS_AT_MOST, Target, WhatIsAsserted, WhatTheEnqueueDid, WriteQueue,
    };
    use crate::cache::freshness::{Age, Skew, WhyTheAgeIsUnreadable, WrittenAt};
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use crate::diagnostics::redaction::CorrelatorSalt;
    use crate::diagnostics::{Diagnostics, DiagnosticsSink, Event, FieldValue, Severity};
    use core::time::Duration;
    use std::sync::Mutex;

    /// A clock that does not move.
    ///
    /// 0100 stamps an event with the wall moment and nothing here reads it back,
    /// which that record says of the field itself: it is for lining an event up
    /// against a server's own log, and no core behaviour depends on it.
    #[derive(Debug, Default)]
    struct Still;

    impl Clocks for Still {
        fn steady(&self) -> SteadyInstant {
            SteadyInstant::from_nanos(0)
        }

        fn elapsed(&self) -> ElapsedInstant {
            ElapsedInstant::from_nanos(0)
        }

        fn wall(&self) -> WallMoment {
            WallMoment::from_epoch(0, 0)
        }
    }

    static STILL: Still = Still;

    fn a_salt() -> CorrelatorSalt {
        CorrelatorSalt::from_bytes([0x5a; CorrelatorSalt::WIDTH])
    }

    /// The facility with nobody listening, which is what most cases here want:
    /// they are about the order, the coalescing and the bound, and a report
    /// nobody receives changes none of the three.
    fn nobody_listening() -> Diagnostics<'static> {
        Diagnostics::new(&STILL, None, Severity::Detail, a_salt())
    }

    /// One event as a case reads it: how much attention it is worth, what it is
    /// called, and its fields as name and text.
    type Told = (Severity, &'static str, Vec<(&'static str, String)>);

    /// Keeps each event's severity, name and fields as text, so a case reads
    /// what the client received rather than what the queue was asked to send.
    #[derive(Debug, Default)]
    struct Collector {
        told: Mutex<Vec<Told>>,
    }

    impl Collector {
        fn told(&self) -> Vec<Told> {
            self.told
                .lock()
                .expect("the fixture holds no poisoned lock")
                .clone()
        }
    }

    impl DiagnosticsSink for Collector {
        fn event(&self, event: &Event<'_>) {
            let fields = event
                .fields()
                .iter()
                .map(|field| {
                    let value = match field.value() {
                        FieldValue::Text(text) => text.to_owned(),
                        FieldValue::Count(count) => count.to_string(),
                        FieldValue::Interval(interval) => format!("{interval:?}"),
                        FieldValue::Truth(truth) => truth.to_string(),
                    };
                    (field.name().as_str(), value)
                })
                .collect();
            self.told
                .lock()
                .expect("the fixture holds no poisoned lock")
                .push((event.severity(), event.name().as_str(), fields));
        }
    }

    fn listening(collector: &Collector) -> Diagnostics<'_> {
        Diagnostics::new(&STILL, Some(collector), Severity::Detail, a_salt())
    }

    /// Seconds in a day, for the moments below, so that a case saying "a year"
    /// says it in the units 0043's bound is written in rather than in a numeral
    /// nobody can read.
    const A_DAY: i64 = 24 * 60 * 60;

    fn item(identifier: &str) -> Target {
        Target::item(identifier.to_string())
    }

    fn a_queue() -> WriteQueue<String> {
        WriteQueue::empty()
    }

    /// The pair 0047 stores at enqueue, with the two moments agreeing.
    ///
    /// Both are handed in, because nothing in this module reads a clock. A
    /// server and a device that agree put the skew at write at zero, which is
    /// what lets the cases below read as the arithmetic they are about; the one
    /// case that is about a correction builds its own pair.
    fn moments(seconds: i64) -> WrittenAt {
        WrittenAt::at(
            WallMoment::from_epoch(seconds, 0),
            WallMoment::from_epoch(seconds, 0),
        )
    }

    fn put(queue: &mut WriteQueue<String>, id: &str, kind: WhatIsAsserted, said: &str) {
        queue.enqueue(
            item(id),
            kind,
            said.to_string(),
            moments(0),
            &nobody_listening(),
        );
    }

    /// The order is a counter increased once per entry, which is what a clock
    /// correction, a suspension and a restart cannot disturb.
    #[test]
    fn the_order_is_a_counter_and_starts_at_one() {
        let mut queue = a_queue();

        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");
        put(&mut queue, "b", WhatIsAsserted::Watched, "yes");
        put(&mut queue, "c", WhatIsAsserted::Watched, "yes");

        let orders: Vec<u64> = queue.entries().iter().map(super::Entry::order).collect();
        assert_eq!(orders, [1, 2, 3]);
    }

    /// Two actions touching one target with one kind are one entry, the later
    /// standing, and the replacement keeps the earlier one's position.
    #[test]
    fn a_later_action_replaces_an_earlier_one_in_place() {
        let mut queue = a_queue();

        put(&mut queue, "a", WhatIsAsserted::PlaybackPosition, "at 10");
        put(&mut queue, "b", WhatIsAsserted::PlaybackPosition, "at 20");
        let what_it_did = queue.enqueue(
            item("a"),
            WhatIsAsserted::PlaybackPosition,
            "at 30".to_string(),
            moments(0),
            &nobody_listening(),
        );

        assert_eq!(what_it_did, WhatTheEnqueueDid::ReplacedInPlace);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.entries()[0].order(), 1);
        assert_eq!(queue.entries()[0].assertion(), "at 30");
        assert_eq!(queue.entries()[1].order(), 2);
    }

    /// A scrub is one entry however many positions it produced, which is what
    /// coalescing at enqueue buys and what coalescing at drain would not.
    #[test]
    fn ninety_positions_for_one_item_are_one_entry() {
        let mut queue = a_queue();

        for position in 0..90 {
            queue.enqueue(
                item("a"),
                WhatIsAsserted::PlaybackPosition,
                format!("at {position}"),
                moments(0),
                &nobody_listening(),
            );
        }

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.entries()[0].assertion(), "at 89");
        assert_eq!(queue.dropped(), 0);
    }

    /// Coalescing is per kind as well as per target: a position and a watched
    /// mark are two statements about one item and neither replaces the other.
    #[test]
    fn one_target_holds_one_entry_per_kind() {
        let mut queue = a_queue();

        put(&mut queue, "a", WhatIsAsserted::PlaybackPosition, "at 10");
        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");

        assert_eq!(queue.len(), 2);
        for kind in WhatIsAsserted::all() {
            assert_eq!(
                queue
                    .entries()
                    .iter()
                    .filter(|entry| entry.asserted_about() == *kind)
                    .count(),
                1,
                "the queue did not hold exactly one {} entry",
                kind.as_str()
            );
        }
    }

    /// And per target as well as per kind, which is the other direction of the
    /// same rule.
    #[test]
    fn two_targets_of_one_kind_are_two_entries() {
        let mut queue = a_queue();

        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");
        put(&mut queue, "b", WhatIsAsserted::Watched, "yes");

        assert_eq!(queue.len(), 2);
    }

    /// The bound, at the boundary itself, and the drop it reports.
    #[test]
    fn the_thousand_and_first_target_displaces_the_oldest() {
        let mut queue = a_queue();

        for target in 0..A_SESSIONS_QUEUE_HOLDS_AT_MOST {
            queue.enqueue(
                item(&format!("item-{target}")),
                WhatIsAsserted::Watched,
                "yes".to_string(),
                moments(0),
                &nobody_listening(),
            );
        }
        assert_eq!(queue.len(), A_SESSIONS_QUEUE_HOLDS_AT_MOST);
        assert_eq!(queue.dropped(), 0);

        let what_it_did = queue.enqueue(
            item("one-too-many"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            moments(0),
            &nobody_listening(),
        );

        let WhatTheEnqueueDid::DroppedTheOldest(dropped) = what_it_did else {
            panic!("the entry past the bound did not report a drop: {what_it_did:?}");
        };
        assert_eq!(dropped.order, 1);
        assert_eq!(dropped.target.as_str(), "item-0");
        assert_eq!(queue.len(), A_SESSIONS_QUEUE_HOLDS_AT_MOST);
        assert_eq!(queue.dropped(), 1);
        assert_eq!(queue.entries()[0].target().as_str(), "item-1");
        assert_eq!(
            queue.entries()[A_SESSIONS_QUEUE_HOLDS_AT_MOST - 1]
                .target()
                .as_str(),
            "one-too-many"
        );
    }

    /// The bound is reached by breadth and never by activity, which is the
    /// sentence that defends the number. A queue at the bound takes any number
    /// of further actions for targets it already holds and drops nothing.
    #[test]
    fn a_queue_at_the_bound_drops_nothing_for_a_target_it_already_holds() {
        let mut queue = a_queue();

        for target in 0..A_SESSIONS_QUEUE_HOLDS_AT_MOST {
            queue.enqueue(
                item(&format!("item-{target}")),
                WhatIsAsserted::Watched,
                "yes".to_string(),
                moments(0),
                &nobody_listening(),
            );
        }

        for again in 0..500 {
            let what_it_did = queue.enqueue(
                item("item-7"),
                WhatIsAsserted::Watched,
                format!("still yes {again}"),
                moments(0),
                &nobody_listening(),
            );
            assert_eq!(what_it_did, WhatTheEnqueueDid::ReplacedInPlace);
        }

        assert_eq!(queue.dropped(), 0);
        assert_eq!(queue.len(), A_SESSIONS_QUEUE_HOLDS_AT_MOST);
    }

    /// A drain walks in counter order and stops at the first entry it could not
    /// deliver, rather than skipping it.
    #[test]
    fn a_drain_walks_in_order_and_stops_where_it_fails() {
        let mut queue = a_queue();
        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");
        put(&mut queue, "b", WhatIsAsserted::Watched, "yes");
        put(&mut queue, "c", WhatIsAsserted::Watched, "yes");

        let mut delivered = Vec::new();
        while let Some(next) = queue.next_to_deliver() {
            if next.target().as_str() == "b" {
                break;
            }
            let taken = queue
                .after_it_was_delivered()
                .expect("the head was there a moment ago");
            delivered.push(taken.target().as_str().to_string());
        }

        assert_eq!(delivered, ["a"]);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.next_to_deliver().map(super::Entry::order), Some(2));
    }

    /// A failed delivery loses nothing, because answering the head and removing
    /// it are two calls.
    #[test]
    fn looking_at_the_head_does_not_remove_it() {
        let mut queue = a_queue();
        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");

        assert_eq!(queue.next_to_deliver().map(super::Entry::order), Some(1));
        assert_eq!(queue.next_to_deliver().map(super::Entry::order), Some(1));
        assert_eq!(queue.len(), 1);
    }

    /// The standing count is per session and a drain does not clear it: what it
    /// counts is what was never delivered.
    #[test]
    fn draining_does_not_clear_the_drop_count() {
        let mut queue = a_queue();
        for target in 0..=A_SESSIONS_QUEUE_HOLDS_AT_MOST {
            queue.enqueue(
                item(&format!("item-{target}")),
                WhatIsAsserted::Watched,
                "yes".to_string(),
                moments(0),
                &nobody_listening(),
            );
        }
        assert_eq!(queue.dropped(), 1);

        while queue.after_it_was_delivered().is_some() {}

        assert!(queue.is_empty());
        assert_eq!(queue.dropped(), 1);
    }

    /// The identifier of a target does not reach a formatted line, because a
    /// drop is reported with a correlator under 0071 and never with the
    /// identifier.
    #[test]
    fn a_targets_identifier_does_not_reach_a_formatted_line() {
        let mut queue = a_queue();
        put(
            &mut queue,
            "an-item-nobody-should-read",
            WhatIsAsserted::Watched,
            "yes",
        );

        let entry = format!("{:?}", queue.entries()[0]);
        let target = format!("{:?}", queue.entries()[0].target());

        assert!(
            !entry.contains("an-item-nobody-should-read"),
            "the identifier reached {entry}"
        );
        assert!(
            !target.contains("an-item-nobody-should-read"),
            "the identifier reached {target}"
        );
        assert!(
            entry.contains("order"),
            "the order is what a report may carry: {entry}"
        );
    }

    /// The reported name of a kind is data rather than a variant's spelling.
    #[test]
    fn every_kind_reports_a_name_of_its_own() {
        let mut seen = Vec::new();
        for kind in WhatIsAsserted::all() {
            assert!(!kind.as_str().is_empty());
            assert!(!seen.contains(&kind.as_str()), "two kinds report one name");
            seen.push(kind.as_str());
        }
        assert_eq!(seen.len(), 2);
    }

    /// An empty queue answers rather than failing, which is what lets a drain be
    /// a loop with no length check in front of it.
    #[test]
    fn an_empty_queue_has_no_head_and_nothing_to_deliver() {
        let mut queue: WriteQueue<String> = WriteQueue::default();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(queue.next_to_deliver().is_none());
        assert!(queue.after_it_was_delivered().is_none());
        assert_eq!(queue.dropped(), 0);
    }

    /// The two moments 0047 stores are the two the entry hands back, so a restore
    /// has something to compute an age from rather than a queue that can say only
    /// that something is pending.
    #[test]
    fn an_entry_carries_the_two_moments_it_was_enqueued_with() {
        let mut queue = a_queue();
        queue.enqueue(
            item("a"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            WrittenAt::at(
                WallMoment::from_epoch(1_700_000_000, 0),
                WallMoment::from_epoch(1_700_000_040, 0),
            ),
            &nobody_listening(),
        );

        let enqueued_at = queue.entries()[0].enqueued_at();
        assert_eq!(
            enqueued_at
                .the_servers_stated_moment()
                .seconds_from_the_epoch(),
            1_700_000_000
        );
        assert_eq!(
            enqueued_at
                .the_devices_wall_moment()
                .seconds_from_the_epoch(),
            1_700_000_040
        );
    }

    /// The age is the device difference, and it is the whole of 0043's
    /// arithmetic rather than a second one: a minute on the device is a minute.
    #[test]
    fn an_entry_reports_how_long_it_has_been_waiting() {
        let mut queue = a_queue();
        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");

        assert_eq!(
            queue.entries()[0].age_at(WallMoment::from_epoch(60, 0), None),
            Age::Of(Duration::from_secs(60))
        );
    }

    /// A device clock that moved between the enqueue and the reading moved both
    /// device readings and neither server moment, and the correction removes
    /// exactly that movement. This is 0043's correction reaching the queue,
    /// which is what 0047 asks for by naming that record rather than describing
    /// an arithmetic of its own.
    #[test]
    fn a_device_clock_that_jumped_forward_is_corrected_out_of_the_age() {
        let mut queue = a_queue();
        queue.enqueue(
            item("a"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            moments(1000),
            &nobody_listening(),
        );

        // Sixty seconds passed and the device also jumped forty seconds ahead of
        // the server, so its own reading is a hundred seconds on.
        let age = queue.entries()[0].age_at(
            WallMoment::from_epoch(1100, 0),
            Some(Skew::between(
                WallMoment::from_epoch(1060, 0),
                WallMoment::from_epoch(1100, 0),
            )),
        );

        assert_eq!(age, Age::Of(Duration::from_secs(60)));
    }

    /// The first of 0043's two guards, reaching a queued action: a device that
    /// came up believing it is earlier than when the action was taken.
    #[test]
    fn a_device_clock_that_moved_backwards_leaves_the_age_unreadable() {
        let mut queue = a_queue();
        queue.enqueue(
            item("a"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            moments(1000),
            &nobody_listening(),
        );

        assert_eq!(
            queue.entries()[0].age_at(WallMoment::from_epoch(940, 0), None),
            Age::Unreadable(WhyTheAgeIsUnreadable::ItComputedAsNegative)
        );
    }

    /// The second guard: a device that jumped forward past the bound beyond
    /// which a computed age is not believed.
    #[test]
    fn an_age_past_the_sanity_bound_is_unreadable() {
        let mut queue = a_queue();
        put(&mut queue, "a", WhatIsAsserted::Watched, "yes");

        assert_eq!(
            queue.entries()[0].age_at(WallMoment::from_epoch(400 * A_DAY, 0), None),
            Age::Unreadable(WhyTheAgeIsUnreadable::ItPassedTheSanityBound)
        );
    }

    /// A replacement keeps the earlier entry's moments, as it keeps its position
    /// in the order. Taking the later action's moments reports every actively
    /// touched entry as freshly enqueued, which is 0047's restored-queue failure
    /// arriving through the coalescing door, and it lands on the person who used
    /// the application most.
    #[test]
    fn a_replacement_keeps_the_earlier_entrys_moments() {
        let mut queue = a_queue();
        queue.enqueue(
            item("a"),
            WhatIsAsserted::PlaybackPosition,
            "at 10".to_string(),
            moments(0),
            &nobody_listening(),
        );

        let what_it_did = queue.enqueue(
            item("a"),
            WhatIsAsserted::PlaybackPosition,
            "at 90".to_string(),
            moments(20 * A_DAY),
            &nobody_listening(),
        );

        assert_eq!(what_it_did, WhatTheEnqueueDid::ReplacedInPlace);
        assert_eq!(queue.entries()[0].assertion(), "at 90");
        assert_eq!(
            queue.entries()[0]
                .enqueued_at()
                .the_devices_wall_moment()
                .seconds_from_the_epoch(),
            0,
            "the replacement took the later action's moments"
        );
        assert_eq!(
            queue.entries()[0].age_at(WallMoment::from_epoch(21 * A_DAY, 0), None),
            Age::Of(Duration::from_hours(21 * 24)),
            "the entry reported the age of the last thing somebody said"
        );
    }

    /// 0047's report, at the moment it happens: the kind of action and the
    /// correlator for the target, at `failure` rather than `notice` because a
    /// person's own action cannot be fetched again.
    #[test]
    fn a_drop_is_reported_as_it_happens_with_the_kind_and_a_correlator() {
        let collector = Collector::default();
        let diagnostics = listening(&collector);
        let mut queue: WriteQueue<String> = WriteQueue::empty();

        for target in 0..A_SESSIONS_QUEUE_HOLDS_AT_MOST {
            queue.enqueue(
                item(&format!("item-{target}")),
                WhatIsAsserted::Watched,
                "yes".to_string(),
                moments(0),
                &diagnostics,
            );
        }
        assert!(
            collector.told().is_empty(),
            "a queue under its bound reported something"
        );

        queue.enqueue(
            item("one-too-many"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            moments(0),
            &diagnostics,
        );

        let told = collector.told();
        assert_eq!(told.len(), 1, "the drop was reported {} times", told.len());
        let (severity, name, fields) = &told[0];
        assert_eq!(*severity, Severity::Failure);
        assert_eq!(*name, "write-queue.entry-dropped");
        assert_eq!(fields.len(), 2, "the event carried {fields:?}");
        assert_eq!(fields[1], ("asserted-about", "watched".to_string()));
    }

    /// The half of that report 0047 states as a refusal: the correlator names
    /// the target and the identifier reaches nobody.
    #[test]
    fn a_drop_report_carries_a_correlator_and_never_the_identifier() {
        let collector = Collector::default();
        let diagnostics = listening(&collector);
        let mut queue: WriteQueue<String> = WriteQueue::empty();

        for target in 0..=A_SESSIONS_QUEUE_HOLDS_AT_MOST {
            queue.enqueue(
                item(&format!("an-item-nobody-should-read-{target}")),
                WhatIsAsserted::PlaybackPosition,
                "at 10".to_string(),
                moments(0),
                &diagnostics,
            );
        }

        let told = collector.told();
        assert_eq!(told.len(), 1);
        let (name, value) = &told[0].2[0];
        assert_eq!(*name, "for-target");
        assert!(
            !value.contains("an-item-nobody-should-read"),
            "the identifier reached the sink as {value}"
        );
        assert!(
            !value.is_empty() && value.chars().all(|character| character.is_ascii_hexdigit()),
            "the target left as something other than a correlator: {value}"
        );
    }

    /// 0047's rule that nothing is ever expired by age, held by a case rather
    /// than by there being no age to expire on. An entry a year old and an entry
    /// whose age is unreadable are both at the head in order and both delivered.
    #[test]
    fn an_entry_is_never_expired_by_age() {
        let mut queue = a_queue();
        queue.enqueue(
            item("a-year-ago"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            moments(0),
            &nobody_listening(),
        );
        queue.enqueue(
            item("after-a-clock-that-moved"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
            moments(500 * A_DAY),
            &nobody_listening(),
        );

        let now = WallMoment::from_epoch(366 * A_DAY, 0);
        assert!(matches!(
            queue.entries()[0].age_at(now, None),
            Age::Unreadable(WhyTheAgeIsUnreadable::ItPassedTheSanityBound)
        ));
        assert!(matches!(
            queue.entries()[1].age_at(now, None),
            Age::Unreadable(WhyTheAgeIsUnreadable::ItComputedAsNegative)
        ));

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.next_to_deliver().map(super::Entry::order), Some(1));

        let mut delivered = Vec::new();
        while let Some(taken) = queue.after_it_was_delivered() {
            delivered.push(taken.target().as_str().to_string());
        }

        assert_eq!(delivered, ["a-year-ago", "after-a-clock-that-moved"]);
        assert_eq!(queue.dropped(), 0);
    }
}
