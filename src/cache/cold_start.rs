//! What a start serves before a session is restored, and what it does not.
//!
//! `docs/decisions/0046-what-is-served-before-a-session-is-restored.md` is the
//! record and #46 is the issue. 0046 decides that a cache read is answerable
//! from the moment the core is created, because 0033 makes a session's identity
//! the client's to hold and 0041 builds a key out of that identity alone; that
//! what can be served that way is every kind 0006 caches, under 0043's states;
//! and that what cannot be served is a list of calls rather than a list of
//! entries.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that a match on a kind and a match on a call
//! settle: that no cache read waits on the secret read, which entries are
//! servable that early, which calls are not, and that what is served says
//! nothing about whether anybody is signed in.
//!
//! WHAT IS NOT HERE IS THE START PATH. Creating the core is #115 and
//! [`crate::Core`] carries no method for it, so nothing here starts anything,
//! names a session, or reads either store. The two stores this orders between
//! are traits a client implements - [`crate::cache::ByteStore`] and
//! [`crate::session::SecretStore`] - and nothing below holds one.
//!
//! WHAT IS ALSO NOT HERE IS A MEASUREMENT POINT. 0046 refuses to name a second
//! pair: 0008 already opens the core's interval at the first library query after
//! creation and closes it at the first decoded artwork bitmap in that answer, and
//! separates an empty-cache variant from a warm-cache one. This path is that
//! warm-cache variant. Two names for one interval is how two numbers for one
//! question get published.
//!
//! # The order is one direction and it is the whole of the rule
//!
//! 0046 says the cache read must not be sequenced behind the secret read, and
//! 0033 names the case that makes it concrete: a device locked at the moment of
//! a background start fails the secret read rather than answering it, so a start
//! ordered the other way shows nothing at all while a complete answer sits in the
//! store - on the one device where the person is also most likely to have no
//! network.
//!
//! [`what_a_start_serves`] takes how the secret read went and cannot let it reach
//! the answer: every path through it is decided by what the cache held. The near
//! miss is the sequencing that returns nothing when the store could not answer,
//! and it is a case in this module rather than a sentence.
//!
//! # A full screen is not a signed-in state
//!
//! A cache read never moves a session's state, and nothing this module answers
//! with carries one. A session whose secret has not been read is not restored, a
//! session whose token the server has since ended is not restored, and neither
//! becomes restored by a library query being answered out of the store.
//!
//! The core also does not decide who is holding the device. Naming a session is
//! the act that exposes that session's cache and that act is the client's. 0041
//! keeps two accounts on one device out of each other's entries; it does not and
//! cannot keep one account's entries from whoever picked the device up, and a
//! client that wants somebody to prove who they are does it before naming the
//! session.

use super::freshness::{Answer, EntryKind};

/// How the read of the session's secret went, or that it has not been made.
///
/// It is an input to [`what_a_start_serves`] that the function is required not
/// to act on, which is why it is a declared set rather than a boolean: the four
/// states include the two that tempt a start path into waiting, and a case can
/// then name them.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HowTheSecretReadWent {
    /// It has not been made yet, or it is still outstanding. This is the state a
    /// start is in for most of the interval #62 measures.
    StillOutstanding,
    /// A token came back and the session can be restored.
    ATokenCameBack,
    /// The store answered and held nothing under that name, which 0033 makes an
    /// absence rather than a failure and which means sign in again.
    NothingWasKeptUnderThatName,
    /// The store could not answer at all, which 0004 maps to
    /// `storage-unavailable`. A DEVICE LOCKED AT A BACKGROUND START IS THIS ONE,
    /// and it is the case the ordering rule exists for.
    TheStoreCouldNotAnswer,
}

impl HowTheSecretReadWent {
    /// Every state, so a condition applies a rule to the whole of it rather than
    /// to whichever member somebody remembered.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::StillOutstanding,
            Self::ATokenCameBack,
            Self::NothingWasKeptUnderThatName,
            Self::TheStoreCouldNotAnswer,
        ]
    }
}

/// What a start hands back for one cache read.
///
/// IT CARRIES NO SESSION STATE, and the absence is the decision rather than an
/// unfinished type. A client that wants to know whether it is signed in asks for
/// that on its own, which 0009 makes a call that cannot wait, and does not infer
/// it from having received data.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatAStartServes {
    /// What the cache held, under one of 0043's three states with its age.
    FromTheCache(Answer),
    /// The read has not answered yet. Nothing is being withheld: this is the
    /// store not having come back rather than a decision about what may be
    /// shown.
    TheCacheHasNotAnsweredYet,
}

/// What a start serves for one cache read, given how the secret read went.
///
/// `the_cache_answered` is `None` where the byte store has not come back. Every
/// other case is 0043's own answer, unchanged: a cold start carves no exception
/// into the freshness table, so an entry past its threshold is `Stale` here
/// exactly as it is at any other moment.
///
/// THE SECOND ARGUMENT IS READ FOR NOTHING AND THAT IS THE PROPERTY. It is taken
/// so that a caller cannot hold this function and a sequencing rule in two
/// places, and so that the case list below has the locked device to name. See the
/// module documentation for the failure it is against.
#[must_use]
pub fn what_a_start_serves(
    the_cache_answered: Option<Answer>,
    _the_secret_read: HowTheSecretReadWent,
) -> WhatAStartServes {
    match the_cache_answered {
        Some(answer) => WhatAStartServes::FromTheCache(answer),
        None => WhatAStartServes::TheCacheHasNotAnsweredYet,
    }
}

/// Whether an entry of this kind may be served before a session is restored.
///
/// Every kind, with no exception carved for this path. 0046's table says yes
/// five times, and the reason it is a total function rather than a list is that
/// a sixth kind added to 0006 arrives here as a compile error rather than as an
/// entry silently outside the rule.
///
/// THE CASE #46 NAMES AS THE ONE TO BE CAREFUL ABOUT DOES NOT APPEAR HERE, and
/// its absence is worth stating. A cached playback authorisation is what that
/// issue asks be withheld this early; 0006 does not cache one, because it is
/// derived from the token and the token is the only secret, so there is no entry
/// to withhold and no rule here that has to remember to. That came from the
/// cache contract rather than from anything on this path, and a later change
/// that started caching something derived from a token would break this sentence
/// without touching it.
#[must_use]
pub const fn is_servable_before_a_session_is_restored(kind: EntryKind) -> bool {
    match kind {
        EntryKind::LibraryQueryResults
        | EntryKind::ItemMetadata
        | EntryKind::ServerCapabilityAnswers
        | EntryKind::ArtworkBytes
        | EntryKind::DecodedDimensions => true,
    }
}

/// A call a client can make while a session is still being restored.
///
/// 0046's second list is of CALLS rather than of entries, and that distinction is
/// the useful half of it: what a start cannot do is everything that needs a
/// token, and naming them here is what stops each one being discovered at its own
/// call site.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CallAtStart {
    /// A read of something the cache may hold.
    AReadOfSomethingCached,
    /// A playback handover, which is #111 and is a call to the server rather
    /// than a lookup.
    APlaybackHandover,
    /// Any report or write toward the server, a position report included. Those
    /// go onto 0047's queue, which a start does not drain until there is a token
    /// to drain it with.
    AWriteTowardTheServer,
    /// A read that demands freshness, which 0006 fixes as returning a fresh
    /// answer or a named failure and never a stale one.
    AReadThatDemandsFreshness,
    /// Signing in, on any of 0005's three routes.
    ASignIn,
}

impl CallAtStart {
    /// Every call in 0046's list, so a condition applies a rule to the whole of
    /// it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::AReadOfSomethingCached,
            Self::APlaybackHandover,
            Self::AWriteTowardTheServer,
            Self::AReadThatDemandsFreshness,
            Self::ASignIn,
        ]
    }

    /// The name this call is written as.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::AReadOfSomethingCached => "a-read-of-something-cached",
            Self::APlaybackHandover => "a-playback-handover",
            Self::AWriteTowardTheServer => "a-write-toward-the-server",
            Self::AReadThatDemandsFreshness => "a-read-that-demands-freshness",
            Self::ASignIn => "a-sign-in",
        }
    }
}

/// Whether a call can be answered before the session is restored.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhetherItCanBeAnsweredYet {
    /// Yes, out of the byte store, with no token and no network call.
    OutOfTheCache,
    /// No. It needs a token, and what it answers with before there is one is the
    /// named failure for its own kind rather than a stale answer or a wait.
    NotUntilThereIsAToken,
}

/// Which calls a start can answer, from 0046's second list.
///
/// A READ THAT DEMANDS FRESHNESS IS IN THE SECOND LIST AND AN ORDINARY READ IS
/// IN THE FIRST, which is the pair a start path collapses if nobody wrote it
/// down. 0006 fixes that a demand for freshness returns a fresh answer or a named
/// failure and never a stale one, so before there is a token the failure is what
/// it returns - and answering it out of the cache instead would be the one place
/// where a cold start turned into a promise the cache contract refuses.
#[must_use]
pub const fn before_a_session_is_restored(call: CallAtStart) -> WhetherItCanBeAnsweredYet {
    match call {
        CallAtStart::AReadOfSomethingCached => WhetherItCanBeAnsweredYet::OutOfTheCache,
        CallAtStart::APlaybackHandover
        | CallAtStart::AWriteTowardTheServer
        | CallAtStart::AReadThatDemandsFreshness
        | CallAtStart::ASignIn => WhetherItCanBeAnsweredYet::NotUntilThereIsAToken,
    }
}

#[cfg(test)]
mod tests {
    //! 0046's ordering and its two lists, asked of the values.
    //!
    //! What these cannot ask is #46's own condition, which starts the core with a
    //! populated cache and an unreachable server. Creating a core is #115 and
    //! there is nothing here to start.

    use super::{
        CallAtStart, HowTheSecretReadWent, WhatAStartServes, WhetherItCanBeAnsweredYet,
        before_a_session_is_restored, is_servable_before_a_session_is_restored,
        what_a_start_serves,
    };
    use crate::cache::freshness::{Age, Answer, EntryKind};
    use core::time::Duration;

    fn a_library_list() -> Answer {
        Answer::Fresh {
            value: b"a-library-list".to_vec(),
            age: Age::Of(Duration::from_secs(30)),
        }
    }

    /// The ordering, asked in every state the secret read can be in. The near
    /// miss is the state that tempts a start path into waiting: a device locked
    /// at a background start, where the store could not answer at all.
    #[test]
    fn what_is_served_does_not_move_with_how_the_secret_read_went() {
        for went in HowTheSecretReadWent::all() {
            assert_eq!(
                what_a_start_serves(Some(a_library_list()), *went),
                WhatAStartServes::FromTheCache(a_library_list()),
                "a complete answer in the store was withheld while the secret read was {went:?}"
            );
        }
    }

    /// A locked device is the case 0033 names and 0046 orders against, so it is
    /// asked on its own as well as inside the sweep above.
    #[test]
    fn a_store_that_could_not_answer_does_not_empty_the_screen() {
        assert_eq!(
            what_a_start_serves(
                Some(a_library_list()),
                HowTheSecretReadWent::TheStoreCouldNotAnswer
            ),
            WhatAStartServes::FromTheCache(a_library_list())
        );
    }

    /// A cache that has not come back is that and nothing else, in every state.
    /// The near miss is an empty answer reported as a decision about what may be
    /// shown.
    #[test]
    fn a_cache_that_has_not_answered_is_not_a_refusal() {
        for went in HowTheSecretReadWent::all() {
            assert_eq!(
                what_a_start_serves(None, *went),
                WhatAStartServes::TheCacheHasNotAnsweredYet
            );
        }
    }

    /// 0043's answer arrives unchanged. A cold start carves no exception into the
    /// freshness table, so a stale entry is stale here too and says so.
    #[test]
    fn a_stale_entry_is_served_as_stale_rather_than_withheld() {
        let stale = Answer::Stale {
            value: b"an-old-library-list".to_vec(),
            age: Age::Of(Duration::from_secs(3600)),
        };
        assert_eq!(
            what_a_start_serves(Some(stale.clone()), HowTheSecretReadWent::StillOutstanding),
            WhatAStartServes::FromTheCache(stale)
        );
    }

    /// Every kind 0006 caches is servable this early. The near miss is a rule
    /// that withholds one kind on this path, which reads as caution and produces
    /// a cold start that is empty for exactly the entries it was built to serve.
    #[test]
    fn every_kind_the_cache_holds_is_servable_before_a_session_is_restored() {
        for kind in EntryKind::all() {
            assert!(
                is_servable_before_a_session_is_restored(*kind),
                "{} was withheld before the session was restored",
                kind.as_str()
            );
        }
        assert_eq!(EntryKind::all().len(), 5);
    }

    /// The second list is of calls, and only the ordinary read is answerable.
    #[test]
    fn only_a_read_of_something_cached_is_answerable_before_there_is_a_token() {
        assert_eq!(
            before_a_session_is_restored(CallAtStart::AReadOfSomethingCached),
            WhetherItCanBeAnsweredYet::OutOfTheCache
        );

        for call in [
            CallAtStart::APlaybackHandover,
            CallAtStart::AWriteTowardTheServer,
            CallAtStart::AReadThatDemandsFreshness,
            CallAtStart::ASignIn,
        ] {
            assert_eq!(
                before_a_session_is_restored(call),
                WhetherItCanBeAnsweredYet::NotUntilThereIsAToken,
                "{} was answered before there was a token to answer it with",
                call.declared_name()
            );
        }
    }

    /// The pair a start path collapses. A read that demands freshness and an
    /// ordinary read differ here, and 0006 is why: a demand for freshness returns
    /// a fresh answer or a named failure and never a stale one.
    #[test]
    fn a_demand_for_freshness_is_not_the_read_beside_it() {
        assert_ne!(
            before_a_session_is_restored(CallAtStart::AReadThatDemandsFreshness),
            before_a_session_is_restored(CallAtStart::AReadOfSomethingCached)
        );
    }

    /// The names are what a report groups by, so they are asked for rather than
    /// assumed from the variant.
    #[test]
    fn every_call_has_its_own_declared_name() {
        let mut names: Vec<&str> = CallAtStart::all()
            .iter()
            .map(|c| c.declared_name())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), CallAtStart::all().len());
    }
}
