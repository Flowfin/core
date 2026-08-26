//! Federation, as the deliberate per-server act 0072 defines.
//!
//! `docs/decisions/0068-the-data-locality-position.md` allows exactly one route
//! for data to reach a second host and calls it deliberate.
//! `docs/decisions/0072-federation-is-a-deliberate-per-server-act.md` defines
//! that word as four properties, and this module is where an implementation
//! either has them or does not.
//!
//! # Why this sits under reaching a server
//!
//! A federation act is what makes a second host reachable at all, and reaching a
//! server is where 0003 places which hosts the core may contact. The list of
//! hosts itself is 0069 and #69; this is the register of acts that would add one
//! to it.
//!
//! # The four properties, and where each one is
//!
//! Off unless switched on: a [`Federation`] that nobody has called
//! [`Federation::add`] on shares nothing with anybody, and there is no
//! constructor that starts with a partner in it.
//!
//! Per second server: an act names one partner and [`Federation::may_share`]
//! answers per partner, so adding a second one cannot widen the first.
//!
//! Named before shared: an act carries the items it enumerated, from the closed
//! set in [`SharedItem`], and the enumeration is what the core is bound by
//! afterwards. An act naming nothing is refused rather than recorded, because a
//! partner that shares nothing is a permission with no enumeration behind it,
//! which is the shape the record is written against.
//!
//! Reversible: [`Federation::revoke`] takes one act, needs no network and asks
//! no second server, and leaves every other act untouched including a second act
//! against the same partner.
//!
//! # What this module cannot do, and does not pretend to
//!
//! NOTHING IN THIS TREE SHARES ANYTHING. There is no transport, no request and
//! no partner ever contacted, so what is proven here is that the register
//! answers correctly and not that some call site consulted it. The place a
//! request would be refused for reaching an unconfigured host is #69 and #70.
//!
//! The entries do not survive the process. 0072 says they are written down on
//! the device, through the store in 0040 where a client supplied one, keyed the
//! way everything else is under 0041. That key does not exist - 0041 requires a
//! cryptographic digest and 0011 measures that the toolchain offers none - so
//! this register holds its acts in memory, which is exactly what 0072 says
//! happens where no store was supplied, arrived at for a different reason. A
//! client is not told that yet, and the capability that would say it is
//! [`crate::cache::CacheStorage`] under #115.

use crate::clock::{Clocks, WallMoment};
use crate::server::address::BaseAddress;
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// One item of the personal data list in 0068, as a federation act may name it.
///
/// 0072 requires what an act will share to be enumerated in terms of that list
/// rather than described in general, because "your library will be shared" names
/// nothing a person can check afterwards and it is the sentence that makes every
/// later widening invisible.
///
/// The set is closed, so a client shows a person a sentence per item and the
/// compiler tells it when the set moves.
///
/// THE SESSION TOKEN IS ON 0068's LIST AND IS DELIBERATELY NOT A MEMBER HERE.
/// 0005 makes it the one secret and the credential that reaches everything else
/// on that list, and 0033 keeps it in a store the core hands to nobody. An act
/// able to name it would be an act able to share the thing that makes every
/// other item reachable, which is not a federation act but a handover of the
/// account. If that reading is wrong it is one variant in one file, and 0072 is
/// the record to argue with.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SharedItem {
    /// The server address as typed and the resolved identity behind it. A
    /// self-hosted address frequently says where the person lives.
    WhereTheLibraryLives,
    /// The account identifier and the account name.
    WhoTheAccountIs,
    /// The device identity and the device profile from #36. The profile says
    /// what hardware somebody has.
    WhatTheDeviceIs,
    /// Titles, identifiers, artwork and any metadata that came back about what
    /// is on that server.
    WhatIsInTheLibrary,
    /// What was played, when, how far it got, and whatever #60 decides counts as
    /// watched. 0068 places this closest to the sensitive kind, because a
    /// viewing history says a great deal about a person.
    WhatWasPlayed,
    /// The queue of actions taken while the server was gone, from #47, which is
    /// the same data with a delay on it.
    WhatWasQueuedWhileOffline,
    /// Diagnostic events and measurement spans, wherever their fields carry any
    /// of the above.
    WhatTheCoreReported,
}

impl SharedItem {
    /// Every item an act may name.
    ///
    /// Here so that a client building the sentence per item reads the set out of
    /// the crate rather than keeping a copy of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::WhereTheLibraryLives,
            Self::WhoTheAccountIs,
            Self::WhatTheDeviceIs,
            Self::WhatIsInTheLibrary,
            Self::WhatWasPlayed,
            Self::WhatWasQueuedWhileOffline,
            Self::WhatTheCoreReported,
        ]
    }

    /// The item as it is reported.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WhereTheLibraryLives => "where-the-library-lives",
            Self::WhoTheAccountIs => "who-the-account-is",
            Self::WhatTheDeviceIs => "what-the-device-is",
            Self::WhatIsInTheLibrary => "what-is-in-the-library",
            Self::WhatWasPlayed => "what-was-played",
            Self::WhatWasQueuedWhileOffline => "what-was-queued-while-offline",
            Self::WhatTheCoreReported => "what-the-core-reported",
        }
    }
}

/// Which act, among the acts on this device.
///
/// An act rather than a partner, because 0072 admits several acts against one
/// partner and revoking one has to leave the others alone. The shortest
/// implementation of revocation drops the partner, which is right only when the
/// partner has one act against it.
///
/// It is drawn in sequence from a counter this register owns, and it means
/// nothing outside one run, for the reason 0061 gives about span identifiers.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActId(u64);

impl ActId {
    /// The number, for a client that has to name one act back to the core.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// An act naming nothing, which is refused rather than recorded.
///
/// 0072 makes the enumeration made at the time of the act the boundary the core
/// is then bound by, so an act with an empty enumeration is a partner carrying a
/// permission and no boundary. It is the shape the record is written against and
/// it is refused at the one place it could be created.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnActMustNameWhatItShares;

/// One act, as the device recorded it.
///
/// Entries are appended and never rewritten, so a revoked act stays visible as
/// an act that was performed and then revoked. A list showing only what is
/// currently active would answer a different and less useful question, since the
/// person asking has usually just remembered something they did once.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    id: ActId,
    partner: BaseAddress,
    performed: WallMoment,
    shares: Vec<SharedItem>,
    revoked: Option<WallMoment>,
}

impl Act {
    /// Which act this is.
    #[must_use]
    pub const fn id(&self) -> ActId {
        self.id
    }

    /// The one server it names.
    #[must_use]
    pub const fn partner(&self) -> &BaseAddress {
        &self.partner
    }

    /// When it was performed, on `wall` from the single injected clock source.
    ///
    /// `wall` because this moment is shown to a person and compared with things
    /// outside the device, which is what 0102 puts on that clock. A device with a
    /// wrong clock therefore records a moment that is hard to place rather than
    /// an act that is wrong.
    #[must_use]
    pub const fn performed(&self) -> WallMoment {
        self.performed
    }

    /// What was enumerated at the moment it was performed.
    #[must_use]
    pub fn shares(&self) -> &[SharedItem] {
        &self.shares
    }

    /// When it was revoked, where it has been.
    #[must_use]
    pub const fn revoked(&self) -> Option<WallMoment> {
        self.revoked
    }

    /// Whether this act still permits anything.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.revoked.is_none()
    }
}

/// Every federation act this device has performed.
///
/// Thread safety, from 0009: safe from any thread. Both lanes would consult it,
/// and a register that were not would make every outbound request a
/// synchronisation point.
pub struct Federation<'a> {
    clocks: &'a dyn Clocks,
    acts: Mutex<Vec<Act>>,
    next_id: AtomicU64,
}

/// Written out rather than derived, for the reason [`crate::measurement`] gives:
/// the clock source is supplied by a client and this crate cannot require
/// `Debug` of a client's type. What is printed is what this register knows about
/// itself.
impl core::fmt::Debug for Federation<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Federation")
            .field("acts", &self.acts().len())
            .finish_non_exhaustive()
    }
}

impl<'a> Federation<'a> {
    /// A register with no acts in it.
    ///
    /// This is the whole of "off unless switched on": there is no other
    /// constructor, no default partner, nothing discovered on a network, and no
    /// server that becomes reachable because another server mentioned it.
    #[must_use]
    pub const fn new(clocks: &'a dyn Clocks) -> Self {
        Self {
            clocks,
            acts: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Records an act a person performed.
    ///
    /// The core does not decide that an act happened; a client asks a person and
    /// tells the core what they answered. What the core owns is that the
    /// enumeration is kept, that it binds afterwards, and that it is written
    /// down.
    ///
    /// WIDENING AN EXISTING ACT IS A NEW ACT AND THIS IS THE CALL THAT MAKES
    /// ONE. There is no call that adds an item to an act already recorded, so
    /// the register shows two entries rather than one entry that changed shape,
    /// which is 0072's rule expressed as an absent method.
    ///
    /// # Errors
    ///
    /// [`AnActMustNameWhatItShares`] where the enumeration is empty.
    pub fn add(
        &self,
        partner: BaseAddress,
        shares: &[SharedItem],
    ) -> Result<ActId, AnActMustNameWhatItShares> {
        if shares.is_empty() {
            return Err(AnActMustNameWhatItShares);
        }
        let id = ActId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let mut named = shares.to_vec();
        named.sort_unstable();
        named.dedup();
        self.held().push(Act {
            id,
            partner,
            performed: self.clocks.wall(),
            shares: named,
            revoked: None,
        });
        Ok(id)
    }

    /// Revokes one act.
    ///
    /// Needs no network, asks no second server, and leaves every other act
    /// untouched. Answers whether an act with that identifier was active; an
    /// identifier nothing matches and an act already revoked both answer
    /// `false`, and neither is a failure.
    ///
    /// What it cannot undo is what has already gone. The core does not know what
    /// the other host retained, cannot delete it, and offers nothing that sounds
    /// better: 0072 refuses an offer to ask the other server, because the
    /// difference between having asked and it being gone is invisible in an
    /// interface at the moment it matters most.
    pub fn revoke(&self, id: ActId) -> bool {
        let at = self.clocks.wall();
        let mut held = self.held();
        match held.iter_mut().find(|act| act.id == id && act.is_active()) {
            Some(act) => {
                act.revoked = Some(at);
                true
            }
            None => false,
        }
    }

    /// Whether anything permits sharing this item with this partner right now.
    ///
    /// This is the question every outbound request would ask, and it is answered
    /// from the acts rather than from a partner list: an act that has been
    /// revoked permits nothing from the moment the revocation was accepted,
    /// whether or not the partner can be reached and whether or not it was
    /// reached before.
    ///
    /// A partner nobody added answers `false`, which is the default
    /// configuration answering for every item and every server.
    #[must_use]
    pub fn may_share(&self, partner: &BaseAddress, item: SharedItem) -> bool {
        self.held()
            .iter()
            .any(|act| act.is_active() && &act.partner == partner && act.shares.contains(&item))
    }

    /// Every act, in the order it was performed, including revoked ones.
    ///
    /// The core exposes the entries rather than wording them, which is the same
    /// rule 0004 and 0100 already state: the sentence a person reads is the
    /// client's.
    #[must_use]
    pub fn acts(&self) -> Vec<Act> {
        self.held().clone()
    }

    /// Whether this device federates with anybody at all.
    ///
    /// Where the set is empty the feature is absent rather than idle, which is
    /// 0072's own sentence, and a client can say so rather than showing an empty
    /// list of partners.
    #[must_use]
    pub fn federates_with_anybody(&self) -> bool {
        self.held().iter().any(Act::is_active)
    }

    fn held(&self) -> std::sync::MutexGuard<'_, Vec<Act>> {
        self.acts
            .lock()
            .expect("the register holds no poisoned lock")
    }
}

#[cfg(test)]
mod tests {
    //! The four properties of 0072, each asked of the register.
    //!
    //! What these cannot ask is whether any call site consults the register,
    //! because there is no call site: nothing in this tree contacts a host at
    //! all. #69 holds the list of hosts the core may contact and #70 is the test
    //! that fails when it reaches one nobody configured.

    use super::{AnActMustNameWhatItShares, Federation, SharedItem};
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use crate::server::address::BaseAddress;
    use core::sync::atomic::{AtomicI64, Ordering};

    /// A wall clock a test moves, so that the moment on an act is a value this
    /// file chose rather than whatever the machine believes.
    struct ControlledClocks {
        seconds: AtomicI64,
    }

    impl ControlledClocks {
        const fn at(seconds: i64) -> Self {
            Self {
                seconds: AtomicI64::new(seconds),
            }
        }

        fn move_to(&self, seconds: i64) {
            self.seconds.store(seconds, Ordering::Relaxed);
        }
    }

    impl Clocks for ControlledClocks {
        fn steady(&self) -> SteadyInstant {
            SteadyInstant::from_nanos(0)
        }

        fn elapsed(&self) -> ElapsedInstant {
            ElapsedInstant::from_nanos(0)
        }

        fn wall(&self) -> WallMoment {
            WallMoment::from_epoch(self.seconds.load(Ordering::Relaxed), 0)
        }
    }

    fn a_partner() -> BaseAddress {
        BaseAddress::parse("films.example/jellyfin").expect("usable")
    }

    fn another_partner() -> BaseAddress {
        BaseAddress::parse("music.example").expect("usable")
    }

    /// Off unless switched on, asked of every item and of two servers nobody
    /// added.
    #[test]
    fn a_configuration_nobody_edited_federates_with_nothing() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        assert!(!federation.federates_with_anybody());
        assert!(federation.acts().is_empty());
        for item in SharedItem::all() {
            assert!(
                !federation.may_share(&a_partner(), *item),
                "{}",
                item.as_str()
            );
            assert!(
                !federation.may_share(&another_partner(), *item),
                "{}",
                item.as_str()
            );
        }
    }

    /// Per second server. This is the condition #72 states as enabling one
    /// server leaving another untouched.
    #[test]
    fn adding_one_partner_leaves_every_other_server_untouched() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");

        assert!(federation.may_share(&a_partner(), SharedItem::WhatIsInTheLibrary));
        for item in SharedItem::all() {
            assert!(
                !federation.may_share(&another_partner(), *item),
                "{}",
                item.as_str()
            );
        }
    }

    /// Named before shared. Federating for one purpose does not federate for
    /// another, and the enumeration is the boundary rather than the partner.
    #[test]
    fn an_act_permits_what_it_named_and_nothing_beside_it() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");

        assert!(federation.may_share(&a_partner(), SharedItem::WhatIsInTheLibrary));
        assert!(!federation.may_share(&a_partner(), SharedItem::WhatWasPlayed));
        assert!(!federation.may_share(&a_partner(), SharedItem::WhoTheAccountIs));
    }

    #[test]
    fn an_act_that_names_nothing_is_refused_rather_than_recorded() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        assert_eq!(
            federation.add(a_partner(), &[]),
            Err(AnActMustNameWhatItShares)
        );
        assert!(federation.acts().is_empty());
        assert!(!federation.federates_with_anybody());
    }

    /// Reversible, and this is the condition #72 states as revocation proven to
    /// stop further sharing.
    #[test]
    fn revoking_an_act_stops_everything_it_permitted() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        let act = federation
            .add(
                a_partner(),
                &[SharedItem::WhatIsInTheLibrary, SharedItem::WhatWasPlayed],
            )
            .expect("an act naming something");
        assert!(federation.may_share(&a_partner(), SharedItem::WhatWasPlayed));

        clocks.move_to(1_700_000_600);
        assert!(federation.revoke(act));

        assert!(!federation.may_share(&a_partner(), SharedItem::WhatIsInTheLibrary));
        assert!(!federation.may_share(&a_partner(), SharedItem::WhatWasPlayed));
        assert!(!federation.federates_with_anybody());
    }

    /// The shortest implementation of revocation drops the partner, which is
    /// right only when the partner has one act against it. This is the near-miss
    /// that catches it.
    #[test]
    fn revoking_one_act_leaves_a_second_act_against_the_same_partner_alone() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        let library = federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");
        federation
            .add(a_partner(), &[SharedItem::WhatWasPlayed])
            .expect("an act naming something");

        assert!(federation.revoke(library));

        assert!(!federation.may_share(&a_partner(), SharedItem::WhatIsInTheLibrary));
        assert!(federation.may_share(&a_partner(), SharedItem::WhatWasPlayed));
        assert!(federation.federates_with_anybody());
    }

    #[test]
    fn revoking_what_is_already_revoked_or_was_never_there_is_not_a_second_revocation() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        let act = federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");

        clocks.move_to(1_700_000_600);
        assert!(federation.revoke(act));
        clocks.move_to(1_700_009_999);
        assert!(!federation.revoke(act));

        let acts = federation.acts();
        assert_eq!(acts.len(), 1);
        assert_eq!(
            acts[0].revoked().map(WallMoment::seconds_from_the_epoch),
            Some(1_700_000_600)
        );
    }

    /// The device's own answer to what has been shared and with whom, which
    /// keeps a revoked act visible as an act that was performed and then
    /// revoked.
    #[test]
    fn the_register_is_appended_to_and_a_revoked_act_stays_in_it() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        let library = federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");
        clocks.move_to(1_700_000_060);
        federation
            .add(another_partner(), &[SharedItem::WhatWasPlayed])
            .expect("an act naming something");
        clocks.move_to(1_700_000_120);
        assert!(federation.revoke(library));

        let acts = federation.acts();
        assert_eq!(acts.len(), 2);
        assert_eq!(acts[0].id(), library);
        assert_eq!(acts[0].partner(), &a_partner());
        assert_eq!(acts[0].performed().seconds_from_the_epoch(), 1_700_000_000);
        assert_eq!(acts[0].shares(), &[SharedItem::WhatIsInTheLibrary]);
        assert!(!acts[0].is_active());
        assert_eq!(acts[1].performed().seconds_from_the_epoch(), 1_700_000_060);
        assert!(acts[1].is_active());
    }

    /// Widening is a new act rather than an act that changed shape, so the
    /// register shows both and the first one's enumeration is what it always
    /// was.
    #[test]
    fn widening_shows_as_two_entries_rather_than_one_that_changed() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");
        clocks.move_to(1_700_086_400);
        federation
            .add(a_partner(), &[SharedItem::WhatWasPlayed])
            .expect("an act naming something");

        let acts = federation.acts();
        assert_eq!(acts.len(), 2);
        assert_eq!(acts[0].shares(), &[SharedItem::WhatIsInTheLibrary]);
        assert_eq!(acts[1].shares(), &[SharedItem::WhatWasPlayed]);
        assert!(federation.may_share(&a_partner(), SharedItem::WhatIsInTheLibrary));
        assert!(federation.may_share(&a_partner(), SharedItem::WhatWasPlayed));
    }

    #[test]
    fn every_item_is_spelled_once_and_the_set_is_the_list_it_came_from() {
        let mut spellings: Vec<&str> = SharedItem::all().iter().map(|i| i.as_str()).collect();
        spellings.sort_unstable();
        let before = spellings.len();
        spellings.dedup();
        assert_eq!(spellings.len(), before);
        assert_eq!(SharedItem::all().len(), 7);
    }

    /// A partner is one server rather than one host, so a sub-path an operator
    /// typed is part of what was named. Two acts against the same host at
    /// different sub-paths are two partners, which falls out of the base address
    /// being the thing an act names.
    #[test]
    fn a_partner_is_the_base_address_that_was_named_and_not_merely_its_host() {
        let clocks = ControlledClocks::at(1_700_000_000);
        let federation = Federation::new(&clocks);

        federation
            .add(a_partner(), &[SharedItem::WhatIsInTheLibrary])
            .expect("an act naming something");

        let same_host_no_sub_path = BaseAddress::parse("films.example").expect("usable");
        assert!(!federation.may_share(&same_host_no_sub_path, SharedItem::WhatIsInTheLibrary));
    }
}
