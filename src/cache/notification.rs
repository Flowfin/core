//! What a change the server reported does to what the cache holds.
//!
//! `docs/decisions/0116-learning-that-something-cached-has-changed.md` is the
//! record and #116 is the issue. 0006 names three ways an entry stops being
//! trusted - an explicit invalidation, a change the server reports, and the
//! passage of time - and this is the middle one. 0043's table is the third and
//! is in [`crate::cache::freshness`].
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of 0116 a comparison of identifiers and a match on
//! a kind settle: which entries a notification invalidates, which it shortens to
//! zero, which it leaves exactly where 0043 put them, and what a listener's own
//! state does to a threshold, which is nothing in every state it can be in.
//!
//! WHAT IS NOT HERE IS THE LISTENER. A held connection is a request that never
//! finishes, the transport is #27, and nothing in this module opens, reads,
//! closes or counts one. [`ListenerState`] is what a read path will be handed,
//! and it exists here so that the prohibition below has a subject rather than
//! because anything produces one.
//!
//! There is no reconnection schedule here either, and that is 0116's decision
//! rather than an omission: 0038 bounds the attempts of a request and 0045 is
//! the recovery schedule for a server that is gone, and a third timetable would
//! be a third answer to when the core tries again.
//!
//! # The prohibition is a type with one variant
//!
//! 0116's degradation rule is written as a prohibition because the tempting
//! change is the other direction: a core being told about every change could
//! hold a library list for a day instead of five minutes, and the hit rate would
//! improve visibly. It would also mean a listener that is connected and silent
//! for a reason nobody noticed produces a cache that is confidently wrong for a
//! day, and silent is exactly what a listener is when something has gone wrong
//! with it.
//!
//! So [`WhatAListenerDoesToAThreshold`] has one variant and no second one to
//! reach for. A change that wanted to lengthen a window while a connection is up
//! has to add a variant, which is a change to 0116 rather than a line in a read
//! path, and that is the whole mechanism. [`what_a_listener_does_to_a_threshold`]
//! is total over [`ListenerState::all`] and answers the same way for every
//! member.
//!
//! # The bound this cannot state
//!
//! #116 asks that a cached read reflect a change within a stated bound, and 0116
//! answers that the core cannot state one as a number: ahead of the core sits
//! the server's own batching, which is an operator's setting. What is here is
//! the core's own half - the moment a notification is applied, the entries it
//! names are absent or shortened - and no interval of the core's is added
//! anywhere in this module. There is no delay type here because there is no
//! delay.

use super::freshness::EntryKind;

/// A change the server reported, as the core reads it.
///
/// Two message kinds and no third, which is 0116 reading the server's own set
/// rather than a vocabulary invented here. The identifier lists are borrowed
/// from whatever read the message; nothing here parses one.
///
/// THE NAMES ARE QUOTED FROM 0116 AND FROM NOWHERE ELSE. That record reads them
/// out of the public server repository at a named commit, with the command that
/// produced each one, so they are a landed reading rather than a claim about an
/// interface nobody has looked at - which is the line
/// `tests/fake_server/surface.rs` draws for its own bodies.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notification<'a> {
    /// Something in a library moved.
    ///
    /// The three identifier lists are the ones 0116 quotes as carrying items:
    /// items added, items removed and items updated. The folder and collection
    /// lists the same message carries name nothing this cache holds under a kind
    /// of its own, so they are not read here and are not carried.
    LibraryChanged {
        /// Items the server says are new. 0116: these name nothing the cache
        /// holds, because an entry for an item nobody has asked about yet does
        /// not exist.
        items_added: &'a [&'a str],
        /// Items the server says are gone.
        items_removed: &'a [&'a str],
        /// Items the server says changed.
        items_updated: &'a [&'a str],
    },
    /// A position, a watched mark or another per-account value moved.
    ///
    /// It carries the new values as well as the identifiers, and 0116 refuses to
    /// write them into the cache: what arrives is one part of an item rather
    /// than an item, and merging a part into a cached whole produces an entry
    /// that is half of one moment and half of another, which is the state
    /// nothing afterwards can tell from a correct one. So only the identifiers
    /// are read, and the values are not carried here at all.
    UserDataChanged {
        /// The account the message is about. 0041 keys the cache per account, so
        /// a message about another account signed in on the same device reaches
        /// none of this session's entries.
        account: &'a str,
        /// The items whose per-account values moved.
        items: &'a [&'a str],
    },
}

/// One entry the cache holds, as this rule needs to see it.
///
/// It is what a read path already knows about an entry rather than anything read
/// out of a store: the kind fixes which rule applies, the item is what a
/// targeted invalidation is matched against, and the account is what 0041 keyed
/// it under.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedEntry<'a> {
    /// Which of 0006's five kinds this entry is.
    pub kind: EntryKind,
    /// The item this entry is about, where it is about one.
    ///
    /// A library query result is about no single item, and that is the fact the
    /// shortening rule exists for: the entry is keyed by 0041's digest over the
    /// request, and a digest cannot be asked whether the answer under it
    /// contained a given item.
    pub item: Option<&'a str>,
    /// The account 0041 keyed this entry under.
    pub account: &'a str,
}

/// What one notification does to one entry.
///
/// Three answers, and each is 0043's own vocabulary reached sooner rather than a
/// fourth state beside it. There is no answer meaning "kept longer", which is
/// the prohibition [`WhatAListenerDoesToAThreshold`] carries from the other
/// side.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhatTheNotificationDoes {
    /// The entry is removed. 0043 answers a later read with `Absent`, because
    /// what was held is wrong rather than old.
    Invalidated,
    /// The entry's threshold becomes zero. 0043 answers a later read with
    /// `Stale`, carrying the value and its age, so the screen fills at once and
    /// corrects itself.
    ///
    /// This is the half 0116 says would otherwise be got wrong in the expensive
    /// direction. Invalidating every query result instead is the more obviously
    /// correct reading of "the answer may have changed", and it empties the tile
    /// wall on every notification, which during a library scan is once per the
    /// operator's batching interval.
    ShortenedToZero,
    /// Nothing moves. The entry stays exactly where 0043's table put it.
    Untouched,
}

impl WhatTheNotificationDoes {
    /// The name this answer is written as.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::Invalidated => "invalidated",
            Self::ShortenedToZero => "shortened-to-zero",
            Self::Untouched => "untouched",
        }
    }
}

/// What a listener is doing.
///
/// Four states, and 0116 names all four in one sentence as the ones that leave
/// every threshold where the table put it: a connection that was refused, never
/// established, dropped, or that the server stopped talking on. `Connected` is
/// the fifth and is the one the prohibition is about, because it is the only one
/// anybody would be tempted to reward.
///
/// Nothing in this tree produces one of these. See the module documentation.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ListenerState {
    /// The upgrade was refused, which on this connection is a 401 because it is
    /// a capability of a session rather than of a server.
    Refused,
    /// Nothing has been attempted, or nothing reached the server.
    NeverEstablished,
    /// The connection is up and messages are arriving.
    Connected,
    /// The connection was up and is not any more.
    Dropped,
    /// The connection is up and the server has said nothing for longer than the
    /// timeout it states for itself. This is the failure mode #116 names as the
    /// one that looks exactly like a quiet library.
    SilentPastTheServersTimeout,
}

impl ListenerState {
    /// Every state, so a caller reads the set out of the crate rather than
    /// keeping a copy, and so a condition applies a rule to the whole of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Refused,
            Self::NeverEstablished,
            Self::Connected,
            Self::Dropped,
            Self::SilentPastTheServersTimeout,
        ]
    }

    /// The name this state is written as.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::Refused => "refused",
            Self::NeverEstablished => "never-established",
            Self::Connected => "connected",
            Self::Dropped => "dropped",
            Self::SilentPastTheServersTimeout => "silent-past-the-servers-timeout",
        }
    }
}

/// What a listener's state does to a freshness threshold.
///
/// ONE VARIANT, AND THE ABSENCE OF A SECOND IS THE RULE. See the module
/// documentation for why this is written as a prohibition and what a change in
/// the other direction would cost.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhatAListenerDoesToAThreshold {
    /// Nothing. Every entry lives under 0043's table whether or not anything is
    /// listening, and no state anywhere records that the cache is being kept
    /// current by a listener.
    Nothing,
}

/// Answers the prohibition for any state a listener can be in.
///
/// It reads its argument for nothing, which is the point: a caller cannot obtain
/// a different threshold by being connected, because there is no path through
/// this function that produces one.
#[must_use]
pub const fn what_a_listener_does_to_a_threshold(
    _state: ListenerState,
) -> WhatAListenerDoesToAThreshold {
    WhatAListenerDoesToAThreshold::Nothing
}

/// What one notification does to one entry, from 0116's rule per kind.
///
/// `the_sessions_account` is the account this cache belongs to, which the caller
/// holds and the notification does not decide.
///
/// # The rules, in the order they are applied
///
/// A message about another account reaches nothing. 0041 keys per account, so an
/// entry of a second account signed in on the same device is not this message's
/// to move. A `LibraryChanged` carries no account and is about the server rather
/// than about a person, so this test applies to `UserDataChanged` alone.
///
/// Artwork bytes, decoded dimensions and capability answers are untouched by
/// either message. 0006 makes a changed image a different key because the
/// address is content-tagged, so there is nothing for a notification to
/// invalidate, and what changes a capability answer is a server upgrade, which
/// neither message reports.
///
/// Item metadata for an item the message names as removed or updated is
/// invalidated, and `UserDataChanged` invalidates the items it names in exactly
/// the same way. An item named only as added moves nothing, because an entry for
/// an item nobody has asked about does not exist and one that does exist was not
/// said to have changed.
///
/// Library query results are shortened to zero on a `LibraryChanged` whether or
/// not the message named any item, because the core cannot tell which queries
/// contained which item and a library change means some may have moved.
///
/// A LIBRARY QUERY RESULT IS UNTOUCHED BY `UserDataChanged`, AND THAT IS A
/// READING RATHER THAN A SENTENCE IN 0116. The record states the shortening
/// under the library message and says of the other only that it invalidates the
/// item metadata entries it names. Shortening on this message too would be
/// defensible on the same argument about digests, and it costs the thing the
/// shortening rule exists to avoid arriving by a second route: a position report
/// is a per-account change, 0057 fixes that one is sent on a cadence during
/// playback, and a rule that shortened every query result on each of them would
/// leave the tile wall permanently stale while anything is playing. The bounded
/// cost of the reading taken instead is a query result that stays fresh for the
/// rest of 0043's five minutes after a watched mark moves. What would reverse it
/// is a measurement that somebody sees that window.
#[must_use]
pub fn what_a_notification_does(
    notification: &Notification<'_>,
    entry: &CachedEntry<'_>,
    the_sessions_account: &str,
) -> WhatTheNotificationDoes {
    match *notification {
        Notification::LibraryChanged {
            items_removed,
            items_updated,
            items_added: _,
        } => match entry.kind {
            EntryKind::LibraryQueryResults => WhatTheNotificationDoes::ShortenedToZero,
            EntryKind::ItemMetadata => {
                if names(entry.item, items_removed) || names(entry.item, items_updated) {
                    WhatTheNotificationDoes::Invalidated
                } else {
                    WhatTheNotificationDoes::Untouched
                }
            }
            EntryKind::ServerCapabilityAnswers
            | EntryKind::ArtworkBytes
            | EntryKind::DecodedDimensions => WhatTheNotificationDoes::Untouched,
        },
        Notification::UserDataChanged { account, items } => {
            if account != the_sessions_account {
                return WhatTheNotificationDoes::Untouched;
            }
            if entry.account != the_sessions_account {
                return WhatTheNotificationDoes::Untouched;
            }
            if entry.kind == EntryKind::ItemMetadata && names(entry.item, items) {
                WhatTheNotificationDoes::Invalidated
            } else {
                WhatTheNotificationDoes::Untouched
            }
        }
    }
}

/// Whether a list names the item an entry is about.
///
/// An entry about no item is named by no list, which is the library query result
/// arriving at a targeted rule and leaving it unmoved.
fn names(item: Option<&str>, listed: &[&str]) -> bool {
    item.is_some_and(|about| listed.contains(&about))
}

#[cfg(test)]
mod tests {
    //! 0116's rule per kind and its prohibition, asked of the values.
    //!
    //! What these cannot ask is #116's own two conditions. Both drive the fake
    //! server - one makes a change on it and reads the cache afterwards, the
    //! other breaks a connection without closing it cleanly - and nothing in
    //! this tree opens a connection to break.

    use super::{
        CachedEntry, ListenerState, Notification, WhatAListenerDoesToAThreshold,
        WhatTheNotificationDoes, what_a_listener_does_to_a_threshold, what_a_notification_does,
    };
    use crate::cache::freshness::EntryKind;

    const OURS: &str = "the-account-signed-in-here";
    const THEIRS: &str = "somebody-else-on-this-device";

    fn entry(kind: EntryKind, item: Option<&'static str>) -> CachedEntry<'static> {
        CachedEntry {
            kind,
            item,
            account: OURS,
        }
    }

    fn library_changed(
        added: &'static [&'static str],
        removed: &'static [&'static str],
        updated: &'static [&'static str],
    ) -> Notification<'static> {
        Notification::LibraryChanged {
            items_added: added,
            items_removed: removed,
            items_updated: updated,
        }
    }

    /// The targeted half. An updated item and a removed one are both the entry
    /// being wrong rather than old, which 0043 answers as `Absent`.
    #[test]
    fn an_item_the_message_updated_or_removed_is_invalidated() {
        for message in [
            library_changed(&[], &[], &["a-film"]),
            library_changed(&[], &["a-film"], &[]),
        ] {
            assert_eq!(
                what_a_notification_does(
                    &message,
                    &entry(EntryKind::ItemMetadata, Some("a-film")),
                    OURS
                ),
                WhatTheNotificationDoes::Invalidated
            );
        }
    }

    /// An item named only as added moves nothing. The near miss is a rule that
    /// treats every identifier in the message alike, which would invalidate a
    /// perfectly good entry every time a scan adds a file beside it.
    #[test]
    fn an_item_the_message_only_added_moves_nothing() {
        assert_eq!(
            what_a_notification_does(
                &library_changed(&["a-film"], &[], &[]),
                &entry(EntryKind::ItemMetadata, Some("a-film")),
                OURS
            ),
            WhatTheNotificationDoes::Untouched
        );
    }

    /// An item the message did not name at all is untouched. The near miss is a
    /// library change that invalidates every item entry rather than the named
    /// ones.
    #[test]
    fn an_item_the_message_did_not_name_is_untouched() {
        assert_eq!(
            what_a_notification_does(
                &library_changed(&[], &["another-film"], &["a-third-film"]),
                &entry(EntryKind::ItemMetadata, Some("a-film")),
                OURS
            ),
            WhatTheNotificationDoes::Untouched
        );
    }

    /// The shortening half, and it does not depend on the message naming
    /// anything: a digest over a request cannot be asked which items its answer
    /// contained, so a library change means some queries may have moved.
    #[test]
    fn a_library_change_shortens_every_query_result_named_or_not() {
        for message in [
            library_changed(&[], &[], &[]),
            library_changed(&["a-film"], &[], &[]),
            library_changed(&[], &[], &["another-film"]),
        ] {
            assert_eq!(
                what_a_notification_does(
                    &message,
                    &entry(EntryKind::LibraryQueryResults, None),
                    OURS
                ),
                WhatTheNotificationDoes::ShortenedToZero,
                "a query result was invalidated or left alone where 0116 shortens it"
            );
        }
    }

    /// The three kinds 0116 gives no rule to, under both messages. The near miss
    /// is an invalidation of artwork on a library change, which throws away the
    /// most expensive bytes in the cache for a change that cannot have moved
    /// them.
    #[test]
    fn artwork_dimensions_and_capability_answers_have_no_rule_under_either_message() {
        for kind in [
            EntryKind::ArtworkBytes,
            EntryKind::DecodedDimensions,
            EntryKind::ServerCapabilityAnswers,
        ] {
            for message in [
                library_changed(&["a-film"], &["a-film"], &["a-film"]),
                Notification::UserDataChanged {
                    account: OURS,
                    items: &["a-film"],
                },
            ] {
                assert_eq!(
                    what_a_notification_does(&message, &entry(kind, Some("a-film")), OURS),
                    WhatTheNotificationDoes::Untouched,
                    "{} was moved by a notification 0116 gives it no rule for",
                    kind.as_str()
                );
            }
        }
    }

    /// A per-account change invalidates the item entries it names, exactly as an
    /// update does, and it does not write the values it carried.
    #[test]
    fn a_per_account_change_invalidates_the_items_it_names() {
        assert_eq!(
            what_a_notification_does(
                &Notification::UserDataChanged {
                    account: OURS,
                    items: &["a-film"],
                },
                &entry(EntryKind::ItemMetadata, Some("a-film")),
                OURS
            ),
            WhatTheNotificationDoes::Invalidated
        );
    }

    /// The account test. The near miss is a rule that reads the identifiers and
    /// not the account, which lets one person on a shared device empty another
    /// person's cache entries.
    #[test]
    fn a_per_account_change_about_another_account_reaches_nothing() {
        let theirs = Notification::UserDataChanged {
            account: THEIRS,
            items: &["a-film"],
        };
        assert_eq!(
            what_a_notification_does(
                &theirs,
                &entry(EntryKind::ItemMetadata, Some("a-film")),
                OURS
            ),
            WhatTheNotificationDoes::Untouched
        );

        let ours_but_their_entry = CachedEntry {
            account: THEIRS,
            ..entry(EntryKind::ItemMetadata, Some("a-film"))
        };
        assert_eq!(
            what_a_notification_does(
                &Notification::UserDataChanged {
                    account: OURS,
                    items: &["a-film"],
                },
                &ours_but_their_entry,
                OURS
            ),
            WhatTheNotificationDoes::Untouched
        );
    }

    /// The reading written on `what_a_notification_does` rather than in 0116,
    /// asked here so that changing it changes a case rather than passing
    /// unnoticed.
    #[test]
    fn a_per_account_change_leaves_a_query_result_where_the_table_put_it() {
        assert_eq!(
            what_a_notification_does(
                &Notification::UserDataChanged {
                    account: OURS,
                    items: &["a-film"],
                },
                &entry(EntryKind::LibraryQueryResults, None),
                OURS
            ),
            WhatTheNotificationDoes::Untouched
        );
    }

    /// A library change never reaches a query result through the targeted rule,
    /// whatever the entry claims to be about.
    #[test]
    fn a_query_result_carrying_an_item_is_still_shortened_and_never_invalidated() {
        assert_eq!(
            what_a_notification_does(
                &library_changed(&[], &["a-film"], &[]),
                &entry(EntryKind::LibraryQueryResults, Some("a-film")),
                OURS
            ),
            WhatTheNotificationDoes::ShortenedToZero
        );
    }

    /// The prohibition, over every state a listener can be in. The near miss is
    /// the one state anybody would reward, and it is in the set.
    #[test]
    fn no_state_of_a_listener_changes_a_threshold() {
        assert_eq!(ListenerState::all().len(), 5);
        assert!(ListenerState::all().contains(&ListenerState::Connected));

        for state in ListenerState::all() {
            assert_eq!(
                what_a_listener_does_to_a_threshold(*state),
                WhatAListenerDoesToAThreshold::Nothing,
                "{} moved a threshold, and 0116 says no state does",
                state.declared_name()
            );
        }
    }

    /// The thresholds a listener does not move are 0043's, read out of the table
    /// rather than restated here. This is the same prohibition asked from the
    /// other side: the number an entry lives under is the table's in every
    /// state.
    #[test]
    fn every_kind_keeps_the_table_threshold_in_every_listener_state() {
        for kind in EntryKind::all() {
            for state in ListenerState::all() {
                assert_eq!(
                    what_a_listener_does_to_a_threshold(*state),
                    WhatAListenerDoesToAThreshold::Nothing
                );
                assert_eq!(kind.threshold(None), kind.stale_after());
            }
        }
    }

    /// The names are what a report groups by, so they are asked for rather than
    /// assumed from the variant.
    #[test]
    fn every_answer_and_every_listener_state_has_its_own_declared_name() {
        let answers = [
            WhatTheNotificationDoes::Invalidated,
            WhatTheNotificationDoes::ShortenedToZero,
            WhatTheNotificationDoes::Untouched,
        ];
        let mut names: Vec<&str> = answers.iter().map(|a| a.declared_name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), answers.len());

        let mut states: Vec<&str> = ListenerState::all()
            .iter()
            .map(|s| s.declared_name())
            .collect();
        states.sort_unstable();
        states.dedup();
        assert_eq!(states.len(), ListenerState::all().len());
    }
}
