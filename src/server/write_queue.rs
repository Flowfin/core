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
//! WHAT IS ALSO NOT HERE IS THE AGE. 0047 stores two moments per entry and
//! computes an age the way 0043 computes a cache entry's, for reporting and
//! never to act. That reading belongs with the two guards
//! [`crate::cache::freshness`] already carries, it acts on nothing here, and no
//! function below takes a clock reading at all - which is the same statement as
//! 0047's rule that an entry is never expired by age, in the form that cannot be
//! got wrong later.
//!
//! # The number here is chosen and not measured
//!
//! 0047 says so of its bound, and says what makes a thousand defensible: with
//! coalescing at enqueue it is a thousand distinct items somebody touched rather
//! than a thousand actions taken. #65 is the harness that would replace it with
//! a measured number.

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
}

/// What an entry that was dropped at the bound was about.
///
/// 0047 requires a drop to be reported at the moment it happens, through the
/// interface in 0100, carrying the kind of action and the correlator for its
/// target rather than the identifier. This is what the enqueue hands back so
/// that the report is made by whoever is holding a diagnostics facility, and it
/// carries the target so the correlator can be derived from it there.
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
    pub fn enqueue(
        &mut self,
        target: Target,
        asserted_about: WhatIsAsserted,
        assertion: A,
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
            what_it_did = WhatTheEnqueueDid::DroppedTheOldest(Dropped {
                target: oldest.target,
                asserted_about: oldest.asserted_about,
                order: oldest.order,
            });
        }

        self.entries.push(Entry {
            order: self.next_order,
            target,
            asserted_about,
            assertion,
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

    fn item(identifier: &str) -> Target {
        Target::item(identifier.to_string())
    }

    fn a_queue() -> WriteQueue<String> {
        WriteQueue::empty()
    }

    fn put(queue: &mut WriteQueue<String>, id: &str, kind: WhatIsAsserted, said: &str) {
        queue.enqueue(item(id), kind, said.to_string());
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
            );
        }
        assert_eq!(queue.len(), A_SESSIONS_QUEUE_HOLDS_AT_MOST);
        assert_eq!(queue.dropped(), 0);

        let what_it_did = queue.enqueue(
            item("one-too-many"),
            WhatIsAsserted::Watched,
            "yes".to_string(),
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
            );
        }

        for again in 0..500 {
            let what_it_did = queue.enqueue(
                item("item-7"),
                WhatIsAsserted::Watched,
                format!("still yes {again}"),
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
}
