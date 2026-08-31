//! The two acts, the order a sign-out takes, and what each one removes.
//!
//! `docs/decisions/0114-signing-out-and-forgetting-a-server.md` is the record
//! and #114 is the issue. 0114 decides that signing out and forgetting a server
//! are two acts rather than one, that the local half of a sign-out completes
//! whatever the network is doing, that four kinds of work in flight each end in
//! a named state, and that the queue survives a sign-out because an undelivered
//! action is the one thing somebody did that has no copy anywhere else.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that a type, a table and a pair of counts
//! settle: that the server half cannot be reached before the local half is done,
//! which act removes what, how each kind of work in flight ends, and what a
//! removal that could not be completed reports.
//!
//! WHAT IS NOT HERE IS THE SESSION. [`crate::session::Session`] holds nothing,
//! there is no token in this tree to drop from memory, and no store is called by
//! anything below: the secret store and the byte store are traits a client
//! implements and nothing here holds one. So this module says what a sign-out
//! does and performs none of it.
//!
//! WHAT IS ALSO NOT HERE IS THE SET A FORGET REMOVES. 0114 makes it the entries
//! whose first three parts are that server, that account and that device
//! identity, and 0041's keys are digests over exactly those, so naming the set
//! is a question for the index 0042 holds. [`Removal`] counts what an act
//! reached and what it could not, which is the half 0114 requires be reported,
//! and it computes no set.
//!
//! # The order is a type rather than a rule
//!
//! 0114 states the order as the thing this act exists against: reversing it, so
//! that the local half waits on the server half, produces a person handing the
//! device to somebody else believing they signed out while the token is still in
//! memory because a request timed out.
//!
//! So [`TellingTheServer`] is produced by [`LocalHalf::what_is_left_for_the_server`]
//! and by nothing else. A caller cannot reach the request before the local half
//! is a value it holds, which is the same construction
//! [`crate::failure::Constructed`] and [`crate::session::delegated::Relayable`]
//! use, for the same reason.
//!
//! NOTHING IN THIS TREE CONSUMES A [`TellingTheServer`], because the request it
//! stands for needs the transport in #27. The type is the seam that request will
//! take, and until it lands this constrains an order at compile time and proves
//! nothing about a network.

use super::renewal::HowTheRenewalEnded;

/// Which of the two acts a client asked for.
///
/// Two and no third. 0114's whole first section is that a client which was not
/// told they differ collapses them into one button, and which of the two that
/// button does is then decided by whoever wrote it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Act {
    /// The next person to pick this device up must not be me. Ordinary, and it
    /// happens on a television in a house every evening.
    SignOut,
    /// This device should stop holding anything about that server at all. What
    /// somebody does before they sell the device, or when they have finished
    /// with a server they were visiting.
    ForgetTheServer,
}

impl Act {
    /// Both acts, so a caller reads the set out of the crate rather than keeping
    /// a copy of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::SignOut, Self::ForgetTheServer]
    }

    /// The name this act is reported under.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::SignOut => "sign-out",
            Self::ForgetTheServer => "forget-the-server",
        }
    }
}

/// One thing an act takes away.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhatIsTakenAway {
    /// The token leaves memory. Nothing further is sent under it by this core.
    TheTokenFromMemory,
    /// The secret is forgotten through 0033's interface. Forgetting one that is
    /// not there succeeds, so a session whose secret was never written signs out
    /// without a failure.
    TheSecretFromItsStore,
    /// Every cache entry under the key space for that server and that account,
    /// in both tiers of 0054.
    TheCacheEntries,
    /// The queue 0047 holds for that session, including its standing count of
    /// what it dropped.
    TheQueue,
    /// The rows the eviction index 0042 holds for the entries removed. 0040
    /// gives the store no listing and 0041's keys are digests, so that index is
    /// the only route to the set, and a removal that left it holding rows for
    /// entries that are gone would leave the bound counting bytes that are gone.
    TheIndexRowsForThoseEntries,
}

/// What an act takes away, in 0114's own list.
///
/// A SIGN-OUT REMOVES NEITHER THE CACHE ENTRIES NOR THE QUEUE, and both are the
/// near miss rather than an omission. #114's own list of the parts of a sign-out
/// puts cache removal inside it; 0006 and 0068 landed the other direction with
/// their reasons and 0114 follows them, because the entries are keyed per
/// server, per account and per device, so leaving them costs nobody else's
/// privacy and signing back in on a device somebody already used is the case the
/// cache exists for. The queue is stronger still: its contents are what somebody
/// did that has no copy anywhere else, and a sign-out that discarded it would
/// lose a fortnight of positions to an ordinary evening act.
///
/// Forgetting a server is strictly a sign-out plus removal, which is why the
/// second list opens with the whole of the first.
#[must_use]
pub const fn what_it_takes_away(act: Act) -> &'static [WhatIsTakenAway] {
    match act {
        Act::SignOut => &[
            WhatIsTakenAway::TheTokenFromMemory,
            WhatIsTakenAway::TheSecretFromItsStore,
        ],
        Act::ForgetTheServer => &[
            WhatIsTakenAway::TheTokenFromMemory,
            WhatIsTakenAway::TheSecretFromItsStore,
            WhatIsTakenAway::TheCacheEntries,
            WhatIsTakenAway::TheQueue,
            WhatIsTakenAway::TheIndexRowsForThoseEntries,
        ],
    }
}

/// A kind of work that can be running when a sign-out arrives.
///
/// Four, from 0114, and each ends in a named state rather than in silence, which
/// is 0009's refusal to report a cancelled or undelivered thing as something it
/// is not applied to one moment.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorkInFlight {
    /// A request already sent on that session.
    ARequestAlreadySent,
    /// A decode running for that session.
    ADecodeRunning,
    /// A read or a write already begun through the byte store.
    ACallInsideTheClientsStore,
    /// An action 0047's queue is holding for that session.
    AQueuedAction,
}

impl WorkInFlight {
    /// Every kind, so a condition applies a rule to the whole of it rather than
    /// to whichever member somebody remembered.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ARequestAlreadySent,
            Self::ADecodeRunning,
            Self::ACallInsideTheClientsStore,
            Self::AQueuedAction,
        ]
    }
}

/// How one kind of work in flight ends when a sign-out arrives.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HowItEnds {
    /// Cancelled, ending as `cancelled` from 0004. AN ANSWER THAT ARRIVES
    /// AFTERWARDS IS DISCARDED RATHER THAN DELIVERED and is not written to the
    /// cache, because it was fetched under a session that has ended.
    CancelledAndALateAnswerDiscarded,
    /// Ended at the next step boundary 0009 and 0115 fix, because a decoder is
    /// not interruptible at an arbitrary instruction. Bytes already handed to a
    /// caller are the caller's and are not reached, which is the boundary 0042
    /// draws for eviction.
    EndedAtTheNextStepBoundary,
    /// Runs to completion. That is 0115's rule and it holds here without
    /// exception: a sign-out does not abandon a call inside somebody else's
    /// implementation.
    CompletesInsideTheClientsCode,
    /// Stays where it is. A later sign-in to the same server, account and device
    /// finds the queue in its own order and drains it before anything else is
    /// sent.
    StaysQueued,
}

/// How each kind of work in flight ends, from 0114.
///
/// CANCELLING IS BY SESSION AND NEVER BY LANE, and 0114 names that as the part
/// that will be got wrong. 0009 gives the core two lanes shared by every
/// session, so the unit of work on a lane carries the session it belongs to. A
/// sign-out that cancelled a lane's work would stop the other account's tile
/// wall on a television with two people signed in, and the person who lost their
/// screen did nothing. Nothing in this signature can express a lane, which is
/// the whole of what it can do about that.
#[must_use]
pub const fn how_it_ends(work: WorkInFlight) -> HowItEnds {
    match work {
        WorkInFlight::ARequestAlreadySent => HowItEnds::CancelledAndALateAnswerDiscarded,
        WorkInFlight::ADecodeRunning => HowItEnds::EndedAtTheNextStepBoundary,
        WorkInFlight::ACallInsideTheClientsStore => HowItEnds::CompletesInsideTheClientsCode,
        WorkInFlight::AQueuedAction => HowItEnds::StaysQueued,
    }
}

/// Why the sign-out is happening.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhySigningOut {
    /// A client asked for one of the two acts.
    TheClientAskedFor(Act),
    /// 0034's renewal ended in a refusal, which signs the session out. The
    /// server half is skipped, because the server has already refused that
    /// token, and 0005 requires the answer to be the same whether the sign-out
    /// was asked for or forced.
    ARenewalWasRefused,
}

/// The local half of a sign-out, once it is done.
///
/// It cannot fail, and the absence of a failure is the decision: 0114 makes the
/// server half the only one that may, so a sign-out on a train is a sign-out and
/// reports that the server was not told rather than refusing to complete.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalHalf {
    act: Act,
    tell_the_server: bool,
    changed_anything: bool,
}

/// What is left to do at the server, once the local half is done.
///
/// It is produced by [`LocalHalf::what_is_left_for_the_server`] and by nothing
/// else, which is how the order in 0114 is held. See the module documentation.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TellingTheServer {
    _produced_by_the_local_half: (),
}

impl LocalHalf {
    /// Completes the local half.
    ///
    /// `already_signed_out` is the caller's reading of the session it named. A
    /// sign-out of a session that is already signed out succeeds and changes
    /// nothing, which is 0114's own sentence and is why this is not a failure.
    ///
    /// A forced sign-out is always the plain act rather than a forget: 0034
    /// refusing a renewal says nothing about whether somebody wants their cache
    /// removed, and a refusal that emptied a person's library would be the
    /// stronger act arriving without anybody asking for it.
    #[must_use]
    pub const fn completed(why: WhySigningOut, already_signed_out: bool) -> Self {
        let act = match why {
            WhySigningOut::TheClientAskedFor(act) => act,
            WhySigningOut::ARenewalWasRefused => Act::SignOut,
        };
        let forced = matches!(why, WhySigningOut::ARenewalWasRefused);
        Self {
            act,
            tell_the_server: !forced && !already_signed_out,
            changed_anything: !already_signed_out,
        }
    }

    /// The same, for a renewal that ended in a refusal.
    ///
    /// It is a second constructor rather than a caller writing the variant,
    /// because [`HowTheRenewalEnded`] is where the other side of this decision
    /// lives and a caller reading a refusal out of that enum should not have to
    /// know which of 0114's cases it maps onto. Any other ending signs nothing
    /// out and answers `None`.
    #[must_use]
    pub const fn after_a_renewal(
        ended: HowTheRenewalEnded,
        already_signed_out: bool,
    ) -> Option<Self> {
        match ended {
            HowTheRenewalEnded::TheServerRefusedIt => Some(Self::completed(
                WhySigningOut::ARenewalWasRefused,
                already_signed_out,
            )),
            _ => None,
        }
    }

    /// Which act this was.
    #[must_use]
    pub const fn act(self) -> Act {
        self.act
    }

    /// Whether anything moved. A sign-out of a session already signed out
    /// succeeds and changes nothing.
    #[must_use]
    pub const fn changed_anything(self) -> bool {
        self.changed_anything
    }

    /// The request that is left, where there is one.
    ///
    /// `None` where the server has already refused the token, and where the
    /// session was already signed out and there is nothing to end.
    #[must_use]
    pub const fn what_is_left_for_the_server(self) -> Option<TellingTheServer> {
        if self.tell_the_server {
            Some(TellingTheServer {
                _produced_by_the_local_half: (),
            })
        } else {
            None
        }
    }
}

/// What a client is told about the server half.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhatTheClientIsTold {
    /// The server was told and the token is finished there.
    TheServerWasTold,
    /// There was nothing to tell it: the token was already refused, or the
    /// session was already signed out.
    ThereWasNothingToTell,
    /// The attempt did not succeed, AND THE TOKEN MAY STILL BE LIVE AT THE
    /// SERVER. That is a thing an operator can act on from the server's own side
    /// and cannot act on if nobody said it, which is why it is a state of its
    /// own rather than a silence beside a completed sign-out.
    TheServerWasNotToldAndTheTokenMayStillBeLive,
}

/// What to report once the server half has been attempted or skipped.
///
/// `attempt_succeeded` is `None` where no attempt was made. The local half has
/// completed in every case here, which is what having a [`LocalHalf`] to pass
/// means.
#[must_use]
pub const fn what_the_client_is_told(
    local: LocalHalf,
    attempt_succeeded: Option<bool>,
) -> WhatTheClientIsTold {
    match (local.tell_the_server, attempt_succeeded) {
        (false, _) | (true, None) => WhatTheClientIsTold::ThereWasNothingToTell,
        (true, Some(true)) => WhatTheClientIsTold::TheServerWasTold,
        (true, Some(false)) => WhatTheClientIsTold::TheServerWasNotToldAndTheTokenMayStillBeLive,
    }
}

/// How far a removal reached.
///
/// THERE IS NO SUCCESS VALUE HERE, and that is 0114's decision rather than an
/// unfinished type. Where the index is absent or was refused by 0105 the set is
/// not reachable, and the core reports that it removed what it could and names
/// how many entries it could not reach; it does not report a removal it did not
/// make. An operator who asked for their data to be removed and was told it was
/// gone has no reason ever to ask again, so an honest count is worth more than a
/// success value.
///
/// Repeating a removal is safe: removing an entry that is not there succeeds
/// under 0040's rule, and the index says what is left.
///
/// Thread safety, from 0009: a plain value, safe from any thread. It is not
/// shared between threads by anything here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Removal {
    removed: u32,
    could_not_be_reached: u32,
}

impl Removal {
    /// A removal that has not begun.
    #[must_use]
    pub const fn nothing_yet() -> Self {
        Self {
            removed: 0,
            could_not_be_reached: 0,
        }
    }

    /// Records one entry removed through 0040's remove operation.
    ///
    /// One at a time, because that is what the store interface offers: the core
    /// removes what it wrote, does not reach past the interface, and does not
    /// ask a client to delete a directory.
    pub const fn removed_one(&mut self) {
        self.removed = self.removed.saturating_add(1);
    }

    /// Records entries the act could not reach, because the index was absent or
    /// because the store stopped answering part way.
    pub const fn could_not_reach(&mut self, how_many: u32) {
        self.could_not_be_reached = self.could_not_be_reached.saturating_add(how_many);
    }

    /// How many were removed.
    #[must_use]
    pub const fn removed(self) -> u32 {
        self.removed
    }

    /// How many were not.
    #[must_use]
    pub const fn could_not_be_reached(self) -> u32 {
        self.could_not_be_reached
    }

    /// Whether everything the act named was reached.
    ///
    /// It is a question about this report rather than a promise about the
    /// device: 0068 refuses to promise what an uninstall leaves behind and this
    /// record does not promise it either.
    #[must_use]
    pub const fn reached_everything_it_named(self) -> bool {
        self.could_not_be_reached == 0
    }
}

#[cfg(test)]
mod tests {
    //! 0114's two acts and the order one of them takes, asked of the values.
    //!
    //! What these cannot ask is #114's own three conditions. Each signs in
    //! against a server, and nothing in this tree signs in.

    use super::{
        Act, HowItEnds, LocalHalf, Removal, WhatIsTakenAway, WhatTheClientIsTold, WhySigningOut,
        WorkInFlight, how_it_ends, what_it_takes_away, what_the_client_is_told,
    };
    use crate::session::renewal::HowTheRenewalEnded;

    fn asked_for(act: Act) -> LocalHalf {
        LocalHalf::completed(WhySigningOut::TheClientAskedFor(act), false)
    }

    /// The two acts differ in exactly the way 0114 says they do, and the near
    /// miss is the one button: a sign-out that removed the cache entries, which
    /// refetches a whole library over whatever connection somebody has the next
    /// time they sign in.
    #[test]
    fn a_sign_out_leaves_the_cache_and_the_queue_and_a_forget_does_not() {
        let signing_out = what_it_takes_away(Act::SignOut);
        assert!(!signing_out.contains(&WhatIsTakenAway::TheCacheEntries));
        assert!(!signing_out.contains(&WhatIsTakenAway::TheQueue));

        let forgetting = what_it_takes_away(Act::ForgetTheServer);
        assert!(forgetting.contains(&WhatIsTakenAway::TheCacheEntries));
        assert!(forgetting.contains(&WhatIsTakenAway::TheQueue));
        assert!(forgetting.contains(&WhatIsTakenAway::TheIndexRowsForThoseEntries));
    }

    /// Forgetting a server is strictly a sign-out plus removal, which is the
    /// sentence that keeps the two from drifting apart one field at a time.
    #[test]
    fn forgetting_a_server_is_everything_a_sign_out_does_and_then_removal() {
        let forgetting = what_it_takes_away(Act::ForgetTheServer);
        for part in what_it_takes_away(Act::SignOut) {
            assert!(
                forgetting.contains(part),
                "{part:?} is part of a sign-out and not part of a forget"
            );
        }
        assert!(what_it_takes_away(Act::SignOut).len() < forgetting.len());
    }

    /// Both acts drop the token and forget the secret, which is the half that
    /// makes either of them a sign-out at all.
    #[test]
    fn every_act_drops_the_token_and_forgets_the_secret() {
        for act in Act::all() {
            let parts = what_it_takes_away(*act);
            assert!(parts.contains(&WhatIsTakenAway::TheTokenFromMemory));
            assert!(parts.contains(&WhatIsTakenAway::TheSecretFromItsStore));
        }
    }

    /// The four kinds of work and their four endings, each different from the
    /// others. The near miss is a sign-out that abandons a call inside the
    /// client's own store, which 0115 refuses without exception.
    #[test]
    fn each_kind_of_work_in_flight_ends_in_its_own_named_state() {
        assert_eq!(
            how_it_ends(WorkInFlight::ARequestAlreadySent),
            HowItEnds::CancelledAndALateAnswerDiscarded
        );
        assert_eq!(
            how_it_ends(WorkInFlight::ADecodeRunning),
            HowItEnds::EndedAtTheNextStepBoundary
        );
        assert_eq!(
            how_it_ends(WorkInFlight::ACallInsideTheClientsStore),
            HowItEnds::CompletesInsideTheClientsCode
        );
        assert_eq!(
            how_it_ends(WorkInFlight::AQueuedAction),
            HowItEnds::StaysQueued
        );

        let mut endings: Vec<HowItEnds> = WorkInFlight::all()
            .iter()
            .map(|w| how_it_ends(*w))
            .collect();
        endings.sort_unstable();
        endings.dedup();
        assert_eq!(
            endings.len(),
            WorkInFlight::all().len(),
            "two kinds of work share an ending, which is the silence 0009 refuses"
        );
    }

    /// The order. The local half is a value before there is anything to send,
    /// and the request cannot be reached without it.
    ///
    /// THE NEAR MISS HERE IS A COMPILE FAILURE RATHER THAN A RED LINE:
    /// `TellingTheServer` has no public constructor and no public field, so a
    /// caller that wanted to send before completing the local half has nothing
    /// to build.
    #[test]
    fn the_request_to_the_server_comes_out_of_the_local_half_and_from_nowhere_else() {
        let local = asked_for(Act::SignOut);
        assert!(local.changed_anything());
        assert!(local.what_is_left_for_the_server().is_some());
    }

    /// A sign-out on a train is a sign-out. It reports that the server was not
    /// told, with the fact an operator can act on, rather than refusing to
    /// complete.
    #[test]
    fn a_failed_attempt_reports_that_the_token_may_still_be_live() {
        let local = asked_for(Act::SignOut);
        assert_eq!(
            what_the_client_is_told(local, Some(false)),
            WhatTheClientIsTold::TheServerWasNotToldAndTheTokenMayStillBeLive
        );
        assert_eq!(
            what_the_client_is_told(local, Some(true)),
            WhatTheClientIsTold::TheServerWasTold
        );
        assert_eq!(
            what_the_client_is_told(local, None),
            WhatTheClientIsTold::ThereWasNothingToTell
        );
    }

    /// A sign-out forced by a refused renewal reaches the same state with the
    /// server half skipped, which is what 0005 requires when it says the answer
    /// must be the same whether the sign-out was asked for or forced.
    #[test]
    fn a_refused_renewal_signs_out_without_telling_the_server() {
        let forced = LocalHalf::completed(WhySigningOut::ARenewalWasRefused, false);
        assert_eq!(forced.act(), Act::SignOut);
        assert!(forced.changed_anything());
        assert!(forced.what_is_left_for_the_server().is_none());
        assert_eq!(
            what_the_client_is_told(forced, None),
            WhatTheClientIsTold::ThereWasNothingToTell
        );
    }

    /// Only a refusal signs the session out, and it never turns into the
    /// stronger act. The near miss is a refused renewal that emptied somebody's
    /// library, which is a forget arriving without anybody asking for one.
    #[test]
    fn only_a_refusal_signs_out_and_it_is_never_the_stronger_act() {
        let refused = LocalHalf::after_a_renewal(HowTheRenewalEnded::TheServerRefusedIt, false)
            .expect("a refusal signs the session out");
        assert_eq!(refused.act(), Act::SignOut);
        assert!(!what_it_takes_away(refused.act()).contains(&WhatIsTakenAway::TheCacheEntries));

        for ending in [
            HowTheRenewalEnded::AFreshToken,
            HowTheRenewalEnded::NothingAnswered,
        ] {
            assert!(
                LocalHalf::after_a_renewal(ending, false).is_none(),
                "{ending:?} signed a session out"
            );
        }
    }

    /// Signing out twice succeeds and changes nothing, and there is nothing left
    /// to tell the server the second time.
    #[test]
    fn a_session_already_signed_out_succeeds_and_moves_nothing() {
        let again = LocalHalf::completed(WhySigningOut::TheClientAskedFor(Act::SignOut), true);
        assert!(!again.changed_anything());
        assert!(again.what_is_left_for_the_server().is_none());
        assert_eq!(
            what_the_client_is_told(again, None),
            WhatTheClientIsTold::ThereWasNothingToTell
        );
    }

    /// A removal that reached nothing says so. The near miss is a success value,
    /// which tells an operator their data is gone and gives them no reason ever
    /// to ask again.
    #[test]
    fn a_removal_that_could_not_reach_the_set_reports_what_it_did_not_do() {
        let mut removal = Removal::nothing_yet();
        removal.could_not_reach(42);
        assert_eq!(removal.removed(), 0);
        assert_eq!(removal.could_not_be_reached(), 42);
        assert!(!removal.reached_everything_it_named());
    }

    /// A removal that stopped part way keeps what it removed and says how far it
    /// reached.
    #[test]
    fn a_removal_that_stopped_part_way_keeps_both_numbers() {
        let mut removal = Removal::nothing_yet();
        for _ in 0..3 {
            removal.removed_one();
        }
        removal.could_not_reach(2);
        assert_eq!(removal.removed(), 3);
        assert_eq!(removal.could_not_be_reached(), 2);
        assert!(!removal.reached_everything_it_named());
    }

    /// A removal that reached everything it named is the only shape that answers
    /// so.
    #[test]
    fn only_a_removal_that_left_nothing_unreached_says_it_reached_everything() {
        let mut removal = Removal::nothing_yet();
        removal.removed_one();
        assert!(removal.reached_everything_it_named());
        removal.could_not_reach(1);
        assert!(!removal.reached_everything_it_named());
    }

    /// The names are what a report carries, so they are asked for rather than
    /// assumed from the variant.
    #[test]
    fn both_acts_have_their_own_declared_name() {
        assert_eq!(Act::SignOut.declared_name(), "sign-out");
        assert_eq!(Act::ForgetTheServer.declared_name(), "forget-the-server");
    }
}
