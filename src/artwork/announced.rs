//! The window a client announces, what a longer one costs, and the fetch two
//! announcements share.
//!
//! `docs/decisions/0053-announced-tiles-and-the-shared-fetch.md` is the record
//! and #53 is the issue. It decides one surface and four properties of it: that
//! an announcement is an ORDERED WINDOW and the order is the whole of the
//! priority, that announcing again REPLACES the previous window rather than
//! adding to it, that a window longer than a stated bound is kept to its first
//! entries with the remainder REPORTED rather than dropped in silence, and that
//! announcements resolving to one entry share one fetch which is abandoned when
//! the LAST caller holding it has withdrawn and not before.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of 0053 that a list and a count settle: what a
//! window holds after one is announced, what a client is told when one was cut,
//! which announcements are sharing a fetch, and which withdrawal is the one that
//! abandons it.
//!
//! WHAT IS NOT HERE IS THE FETCH. Nothing in this tree opens a connection, for
//! the reason [`crate::server::transport`] gives about itself, so nothing is
//! fetched, nothing is decoded and nothing is abandoned. This module holds the
//! bookkeeping such a fetch would be started and stopped from. #53's three
//! conditions announce two hundred tiles against a running core, and none of
//! them is met by anything here.
//!
//! WHAT IS ALSO NOT HERE IS A CAP ON OUTSTANDING REQUESTS, and its absence is
//! 0053's own sentence rather than an omission. The cap #53 asks for is not a
//! number that record adds: 0027 holds at most six requests outstanding to one
//! server and names this wall as the reason for the figure. What 0053 fixes is
//! that the other announced entries sit in front of the transport as data in
//! announcement order rather than as waiters on the lane 0009 sizes once, which
//! is what this module holds.
//!
//! WHAT IS ALSO NOT HERE IS THE REPORT. 0053 says a cut window is reported
//! through 0100. [`AnnouncedWindow::announce`] answers what it did so that
//! whoever holds a diagnostics facility can report it, and nothing in this tree
//! holds one at a point where a window is announced.
//!
//! # The number here is chosen and not measured
//!
//! 0053 says so of its bound, with the arithmetic beside it: two hundred for the
//! wall #53 is named for, and fifty-six further entries, which is more than one
//! screen of any tile size on any supported target. #65 is the harness a
//! measured replacement would come from.

use crate::cache::EntryKey;

/// The entries one session may have announced at once.
///
/// From 0053, chosen rather than measured. The case it is against is not a
/// client that announces two hundred and one; it is a client that announces its
/// library, which is a queue the core holds proportional to somebody else's
/// data.
pub const ANNOUNCED_ENTRIES_PER_SESSION: usize = 256;

/// What announcing a window did to it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheAnnouncementDid {
    /// The whole window was taken.
    TookAllOfIt,
    /// The window was longer than [`ANNOUNCED_ENTRIES_PER_SESSION`], so it was
    /// kept to its first entries in the order it was given and the tail was not
    /// announced.
    ///
    /// A client whose windows are being cut is a client whose prefetch is doing
    /// nothing, and there is no other way for its author to find that out, which
    /// is why this carries the count rather than being a boolean.
    CutTheTail {
        /// How many entries were announced.
        announced: usize,
        /// How many were not.
        cut: usize,
    },
}

/// The window one session has announced, in the order it was given.
///
/// The order is the order the core starts the work in, and it is the whole of
/// the priority the core accepts: 0053 refuses a priority number beside a tile,
/// because 0050 already declines a client-supplied priority on the decode
/// admission queue and two orderings over one set of work disagree the first
/// time a tile announced second carries the higher number.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnnouncedWindow {
    entries: Vec<EntryKey>,
}

impl AnnouncedWindow {
    /// A session that has announced nothing.
    #[must_use]
    pub const fn nothing_announced() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The entries announced, in the order the client gave them.
    #[must_use]
    pub fn entries(&self) -> &[EntryKey] {
        &self.entries
    }

    /// How many entries are announced.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is announced.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Announce a window.
    ///
    /// IT REPLACES THE PREVIOUS WINDOW RATHER THAN ADDING TO IT, which is
    /// 0053's decision and not a convenience. A wall being moved produces a new
    /// window several times a second, and a surface that accumulated would need
    /// a second call to say what is no longer coming - so the first client that
    /// forgot to make it would hold the whole of everything it had ever
    /// announced.
    ///
    /// A window longer than [`ANNOUNCED_ENTRIES_PER_SESSION`] is kept to its
    /// first entries IN THE ORDER IT WAS GIVEN and the tail is not announced.
    /// The tail is the part furthest from the screen, and the entries that were
    /// cut are still fetched when they are asked for, because an announcement is
    /// advisory and never the way of being served.
    pub fn announce(&mut self, window: Vec<EntryKey>) -> WhatTheAnnouncementDid {
        let offered = window.len();
        self.entries = window;
        if offered > ANNOUNCED_ENTRIES_PER_SESSION {
            self.entries.truncate(ANNOUNCED_ENTRIES_PER_SESSION);
            WhatTheAnnouncementDid::CutTheTail {
                announced: ANNOUNCED_ENTRIES_PER_SESSION,
                cut: offered - ANNOUNCED_ENTRIES_PER_SESSION,
            }
        } else {
            WhatTheAnnouncementDid::TookAllOfIt
        }
    }
}

/// What holding an entry did.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheHoldDid {
    /// Nothing was holding this entry, so a fetch starts.
    StartedTheFetch,
    /// A fetch for this entry is already in flight and this caller joined it.
    ///
    /// 0053 compares on the cache key 0041 builds rather than on the address,
    /// which is what makes the rule survive #49: that issue requires two nearby
    /// requested sizes to resolve to one entry, and coalescing on the address
    /// would fetch those twice for exactly the case it exists to prevent.
    JoinedTheFetch,
}

/// What withdrawing did.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheWithdrawalDid {
    /// The last caller holding this entry withdrew, so the fetch is abandoned.
    AbandonedTheFetch,
    /// Another caller is still holding it, so nothing is abandoned.
    LeftItHeld {
        /// How many callers are still holding it.
        holders: usize,
    },
    /// Nothing was holding this entry.
    ///
    /// Answered rather than ignored, because a withdrawal for something nobody
    /// holds is a caller that has lost track of what it announced, and the
    /// convenient handling is to subtract from a count that is already zero.
    NothingHeldIt,
}

/// Which entries have a fetch in flight, and how many callers each is shared
/// with.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SharedFetches {
    held: Vec<(EntryKey, usize)>,
}

impl SharedFetches {
    /// A session sharing nothing.
    #[must_use]
    pub const fn none() -> Self {
        Self { held: Vec::new() }
    }

    /// How many entries have a fetch in flight.
    #[must_use]
    pub const fn in_flight(&self) -> usize {
        self.held.len()
    }

    /// How many callers are holding the fetch for one entry.
    #[must_use]
    pub fn holders(&self, entry: &EntryKey) -> usize {
        self.held
            .iter()
            .find(|(held, _)| held == entry)
            .map_or(0, |(_, holders)| *holders)
    }

    /// Take a hold on the fetch for one entry, starting it where nothing else
    /// is holding it.
    pub fn hold(&mut self, entry: EntryKey) -> WhatTheHoldDid {
        if let Some((_, holders)) = self.held.iter_mut().find(|(held, _)| *held == entry) {
            *holders = holders.saturating_add(1);
            return WhatTheHoldDid::JoinedTheFetch;
        }
        self.held.push((entry, 1));
        WhatTheHoldDid::StartedTheFetch
    }

    /// Give up one hold on the fetch for one entry.
    ///
    /// THE FETCH IS ABANDONED WHEN THE LAST HOLDER WITHDRAWS AND NOT BEFORE, and
    /// that is the sentence 0053 puts the whole shared-fetch section on. 0009's
    /// rule that a cancelled call produces nothing is about a CALL; it does not
    /// reach a fetch another caller is still holding, and a withdrawal that
    /// abandoned on the first hold given up would blank the tile of whoever was
    /// still waiting on it. Nothing reports that: the second caller sees an
    /// image that did not arrive, which is indistinguishable from a slow server.
    pub fn withdraw(&mut self, entry: &EntryKey) -> WhatTheWithdrawalDid {
        let Some(at) = self.held.iter().position(|(held, _)| held == entry) else {
            return WhatTheWithdrawalDid::NothingHeldIt;
        };
        let holders = self.held[at].1;
        if holders <= 1 {
            self.held.remove(at);
            return WhatTheWithdrawalDid::AbandonedTheFetch;
        }
        self.held[at].1 = holders - 1;
        WhatTheWithdrawalDid::LeftItHeld {
            holders: holders - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    //! 0053's window, its bound and its shared fetch, asked of the values.
    //!
    //! What these cannot ask is any of #53's three conditions. Each announces
    //! tiles against a running core and watches work stop, and nothing in this
    //! tree fetches or decodes anything.

    use super::{
        ANNOUNCED_ENTRIES_PER_SESSION, AnnouncedWindow, SharedFetches, WhatTheAnnouncementDid,
        WhatTheHoldDid, WhatTheWithdrawalDid,
    };
    use crate::cache::EntryKey;

    fn entry(name: &str) -> EntryKey {
        EntryKey::from_derived_key(name.to_string())
    }

    fn a_window_of(entries: usize) -> Vec<EntryKey> {
        (0..entries).map(|n| entry(&format!("entry-{n}"))).collect()
    }

    /// A window is kept in the order it was given, because that order is the
    /// whole of the priority the core accepts.
    #[test]
    fn a_window_is_held_in_the_order_it_was_given() {
        let mut window = AnnouncedWindow::nothing_announced();

        let what_it_did = window.announce(vec![entry("c"), entry("a"), entry("b")]);

        assert_eq!(what_it_did, WhatTheAnnouncementDid::TookAllOfIt);
        assert_eq!(window.entries(), [entry("c"), entry("a"), entry("b")]);
    }

    /// Announcing again replaces rather than adds, so a window several times a
    /// second does not accumulate into everything a client ever expected to
    /// draw.
    #[test]
    fn announcing_again_replaces_the_previous_window() {
        let mut window = AnnouncedWindow::nothing_announced();
        window.announce(a_window_of(40));

        window.announce(vec![entry("only-this")]);

        assert_eq!(window.len(), 1);
        assert_eq!(window.entries(), [entry("only-this")]);
    }

    /// The bound, at the boundary itself rather than a value either side of it,
    /// and the cut reported rather than taken in silence.
    #[test]
    fn a_window_past_the_bound_is_cut_at_its_tail_and_the_cut_is_reported() {
        let mut window = AnnouncedWindow::nothing_announced();

        assert_eq!(
            window.announce(a_window_of(ANNOUNCED_ENTRIES_PER_SESSION)),
            WhatTheAnnouncementDid::TookAllOfIt
        );
        assert_eq!(window.len(), ANNOUNCED_ENTRIES_PER_SESSION);

        let what_it_did = window.announce(a_window_of(20_000));

        assert_eq!(
            what_it_did,
            WhatTheAnnouncementDid::CutTheTail {
                announced: ANNOUNCED_ENTRIES_PER_SESSION,
                cut: 20_000 - ANNOUNCED_ENTRIES_PER_SESSION,
            }
        );
        assert_eq!(window.len(), ANNOUNCED_ENTRIES_PER_SESSION);
        assert_eq!(ANNOUNCED_ENTRIES_PER_SESSION, 256);
    }

    /// The tail is what goes, so what is kept is the part nearest the screen.
    #[test]
    fn what_survives_a_cut_is_the_head_of_the_window() {
        let mut window = AnnouncedWindow::nothing_announced();

        window.announce(a_window_of(ANNOUNCED_ENTRIES_PER_SESSION + 3));

        assert_eq!(window.entries()[0], entry("entry-0"));
        assert_eq!(
            window.entries()[ANNOUNCED_ENTRIES_PER_SESSION - 1],
            entry(&format!("entry-{}", ANNOUNCED_ENTRIES_PER_SESSION - 1))
        );
        assert!(
            !window
                .entries()
                .contains(&entry(&format!("entry-{ANNOUNCED_ENTRIES_PER_SESSION}")))
        );
    }

    /// Two announcements that resolve to one entry share one fetch.
    #[test]
    fn two_tiles_naming_one_entry_are_one_fetch() {
        let mut fetches = SharedFetches::none();

        assert_eq!(
            fetches.hold(entry("poster")),
            WhatTheHoldDid::StartedTheFetch
        );
        assert_eq!(
            fetches.hold(entry("poster")),
            WhatTheHoldDid::JoinedTheFetch
        );

        assert_eq!(fetches.in_flight(), 1);
        assert_eq!(fetches.holders(&entry("poster")), 2);
    }

    /// Two entries never join, however many callers each has.
    #[test]
    fn two_entries_are_two_fetches() {
        let mut fetches = SharedFetches::none();

        fetches.hold(entry("poster"));
        fetches.hold(entry("backdrop"));

        assert_eq!(fetches.in_flight(), 2);
        assert_eq!(fetches.holders(&entry("poster")), 1);
        assert_eq!(fetches.holders(&entry("backdrop")), 1);
    }

    /// The fetch is abandoned when the LAST holder withdraws and not before,
    /// which is the sentence 0053 puts the shared fetch on.
    #[test]
    fn a_fetch_survives_every_withdrawal_but_the_last() {
        let mut fetches = SharedFetches::none();
        for _ in 0..4 {
            fetches.hold(entry("poster"));
        }

        for still_held in [3_usize, 2, 1] {
            assert_eq!(
                fetches.withdraw(&entry("poster")),
                WhatTheWithdrawalDid::LeftItHeld {
                    holders: still_held
                }
            );
            assert_eq!(fetches.in_flight(), 1);
        }

        assert_eq!(
            fetches.withdraw(&entry("poster")),
            WhatTheWithdrawalDid::AbandonedTheFetch
        );
        assert_eq!(fetches.in_flight(), 0);
        assert_eq!(fetches.holders(&entry("poster")), 0);
    }

    /// A withdrawal for something nobody holds says so rather than subtracting
    /// from a count that is already zero.
    #[test]
    fn withdrawing_something_nobody_holds_is_answered() {
        let mut fetches = SharedFetches::none();

        assert_eq!(
            fetches.withdraw(&entry("never-announced")),
            WhatTheWithdrawalDid::NothingHeldIt
        );

        fetches.hold(entry("poster"));
        fetches.withdraw(&entry("poster"));
        assert_eq!(
            fetches.withdraw(&entry("poster")),
            WhatTheWithdrawalDid::NothingHeldIt
        );
        assert_eq!(fetches.in_flight(), 0);
    }
}
