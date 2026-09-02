//! The set of destinations the core may contact, and what a redirect does.
//!
//! `docs/decisions/0069-every-host-the-core-may-contact.md` decides both. The
//! set is exactly the origins of the servers the operator configured, one row
//! per configured server and no row the core adds on its own; a redirect is
//! followed only where it stays inside the origin the request was sent to, and
//! one leaving it is refused before a request is sent to the new location.
//!
//! # Why the set is a value here rather than a list somewhere
//!
//! 0069 derives its table rather than writing it out, and the number of rows in
//! a running core is the number of servers the operator added. So the authority
//! for the set is a register the core holds, and this is it. It starts empty,
//! which `docs/decisions/0068-the-data-locality-position.md` fixes and which is
//! what makes the check in #70 unambiguous: any host at all in a run that
//! configured none is the defect, with nothing to reason about.
//!
//! # Where a name may be handed to a resolver
//!
//! 0069 puts name resolution inside the set rather than in front of it, because
//! a lookup is itself a request that carries the name being looked up, and a
//! core that resolved first and compared afterwards has already told a third
//! party which server the person uses. [`AdmittedOrigin`] is that sentence as a
//! type: it is produced by [`Destinations::admit`] and by nothing else, so a
//! call that takes one has had the comparison made for it, and one that takes an
//! [`Origin`] has not.
//!
//! WHAT THAT IS WORTH TODAY IS AVAILABLE RATHER THAN HELD, and the difference is
//! the whole of it. Nothing in this tree connects, resolves or requests, so no
//! call site consumes an [`AdmittedOrigin`] and nothing is kept to the order by
//! it. What it buys is that the day the transport in #27 lands, a connect
//! written against [`Origin`] is a connect that skipped this comparison, and the
//! reviewer of that change is reading a signature rather than searching for a
//! call. [`Origin`] itself still hands out its host to anybody holding one; this
//! module does not narrow that, because the parse in
//! [`crate::server::address`] is 0028's rather than 0069's.
//!
//! # What this module does not do
//!
//! IT REFUSES NOTHING THAT IS HAPPENING. The refusal 0069 wants is a request not
//! being sent, and no request is sent from anywhere in this crate. What is here
//! answers correctly about an origin and produces the failure value a caller
//! would hand back; whether any caller asks is #27 for the transport and #70 for
//! the test that fails when the core reaches a host nobody configured.

use crate::failure::{Expected, Failure, ReadingSite};
use crate::server::address::Origin;
use std::sync::{Mutex, MutexGuard};

/// Where a refused redirect says reading stopped.
///
/// 0004 fixes the payload of `answer-not-understood` as the site, what was
/// expected there, and an offset, and it forbids the bytes at that offset
/// because an answer holds library contents and may hold a token. A refused
/// redirect consumed none of the location it was handed - it read the whole of
/// it and declined it - so the offset is the start of it rather than a position
/// inside something that was partly parsed.
pub const A_REFUSED_REDIRECT_CONSUMED_NOTHING: usize = 0;

/// An origin the configured set admits, and therefore one a name may be resolved
/// for.
///
/// There is no constructor. [`Destinations::admit`] and [`Destinations::follow`]
/// are the only two things that make one, and both make it out of a comparison
/// against the set rather than out of an address, so a value of this type is
/// evidence that the comparison happened.
///
/// It carries the origin rather than borrowing the register, because 0069's
/// answer is about the origin, and a value that held the register would make
/// every request the core made a reader of it for as long as the request lasted.
///
/// Thread safety, from 0009: immutable once made, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedOrigin {
    origin: Origin,
}

impl AdmittedOrigin {
    /// The origin this admits, whole.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// The host a resolver may be given.
    ///
    /// This accessor exists so that the one place a name leaves the core towards
    /// a resolver is reached through a value that has been compared against the
    /// set. It is the same string [`Origin::host`] answers with; what is
    /// different is what a caller had to hold to get here.
    #[must_use]
    pub fn host(&self) -> &str {
        self.origin.host()
    }
}

/// What adding a configured server did to the set.
///
/// The set is closed and exhaustive, for 0004's reason applied to a different
/// subject: a caller matching on it is told by the compiler when a case appears
/// rather than falling into a branch somebody wrote for something else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatConfiguringDid {
    /// The origin was not in the set and is now. This is a row arriving.
    AddedARow,
    /// The origin was already in the set, so nothing moved.
    ///
    /// 0069 fixes one row per configured server, so a second sign-in against a
    /// server the operator already added widens nothing and duplicates nothing.
    LeftTheSetAsItWas,
}

/// What the core does with a redirect a server answered with.
///
/// Thread safety, from 0009: a value handed back to a caller, safe from any
/// thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatARedirectDoes {
    /// It stays inside the origin the request went to, so it is followed.
    ///
    /// This is the ordinary case 0069 names: a server correcting a path, a
    /// trailing separator or a version prefix.
    Followed(AdmittedOrigin),
    /// It leaves that origin, so nothing is sent to it.
    ///
    /// The failure is built here rather than by the caller, so that a refusal
    /// cannot be turned into some other kind on the way back.
    Refused(Failure),
}

/// Every origin the operator configured, and nothing else.
///
/// Thread safety, from 0009: safe from any thread. Every request would consult
/// it, and a register that were not would make each of them a synchronisation
/// point.
#[derive(Debug)]
pub struct Destinations {
    held: Mutex<Vec<Origin>>,
}

impl Default for Destinations {
    fn default() -> Self {
        Self::new()
    }
}

impl Destinations {
    /// A set with nothing in it.
    ///
    /// This is the whole of "the set is empty before anything is configured".
    /// There is no other constructor, nothing is added by a request succeeding,
    /// and no host is here because the core needed one: the only way a row
    /// arrives is [`Destinations::configured`], which a caller supplies the
    /// origin to.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            held: Mutex::new(Vec::new()),
        }
    }

    /// Records that the operator configured a server at this origin.
    ///
    /// 0069 names two acts that reach here and no third. The first is the
    /// sign-in that adds the first server, which is #30. The second is the
    /// deliberate per-server act
    /// `docs/decisions/0072-federation-is-a-deliberate-per-server-act.md`
    /// defines, performed the same way by the same person, which adds a row
    /// rather than widening one.
    ///
    /// It takes an [`Origin`] rather than an address, because 0069 compares the
    /// origin as it was resolved on the way in rather than the address as it was
    /// typed, and [`crate::server::address::BaseAddress::origin`] is where that
    /// resolution has already happened.
    pub fn configured(&self, origin: Origin) -> WhatConfiguringDid {
        let mut held = self.held();
        if held.contains(&origin) {
            return WhatConfiguringDid::LeftTheSetAsItWas;
        }
        held.push(origin);
        WhatConfiguringDid::AddedARow
    }

    /// Removes one origin from the set, and answers whether it was in it.
    ///
    /// It reaches one row. A set holding several servers loses the named one and
    /// keeps the rest, because a person removing one server is not saying
    /// anything about another, and the shortest implementation empties the set.
    pub fn forgotten(&self, origin: &Origin) -> bool {
        let mut held = self.held();
        let before = held.len();
        held.retain(|kept| kept != origin);
        held.len() != before
    }

    /// How many servers the operator configured.
    ///
    /// 0069 says the number of rows in a running core is this number, so it is
    /// read out of the register rather than stated anywhere.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.held().len()
    }

    /// Every origin in the set.
    ///
    /// A person can be shown where their core may go, which is the half of
    /// 0068's position a client can act on.
    #[must_use]
    pub fn all(&self) -> Vec<Origin> {
        self.held().clone()
    }

    /// Whether this origin is one the operator configured, and a name may
    /// therefore be resolved for it.
    ///
    /// An empty set admits nobody, which is 0068's sentence and #70's
    /// unambiguous failure.
    ///
    /// It compares the whole origin. 0069 refuses the host alone by name: a
    /// server behind a reverse proxy shares its machine with whatever else is on
    /// another port, and a typed `http://` is honoured rather than upgraded, so
    /// the scheme is part of what the operator chose. An absent port is not the
    /// scheme's default filled in either, which is [`Origin`]'s own rule and
    /// reaches this comparison unchanged.
    #[must_use]
    pub fn admit(&self, origin: &Origin) -> Option<AdmittedOrigin> {
        self.held()
            .iter()
            .find(|held| *held == origin)
            .map(|held| AdmittedOrigin {
                origin: held.clone(),
            })
    }

    /// What a redirect answered by a request sent to `sent_to` does.
    ///
    /// IT COMPARES AGAINST THE ORIGIN THE REQUEST WENT TO AND NEVER AGAINST THE
    /// WHOLE SET, and that is 0069 read exactly rather than read conveniently.
    /// That record follows a redirect only where it stays inside the origin that
    /// was already in the set and refuses one going anywhere else, so a server
    /// the operator configured redirecting to a SECOND server the operator also
    /// configured is refused here. The convenient reading admits it, and what it
    /// admits is one configured server deciding that another one answers for it,
    /// which is a decision
    /// `docs/decisions/0101-what-the-core-trusts.md` places with the operator:
    /// the person is trusted for which servers the core may talk to, and a
    /// server is not trusted to move a request onto one of them.
    ///
    /// Nothing is sent to a refused location: not a credential, not the device
    /// identity, not the query. The refusal is the answer, and there is no
    /// variant carrying a location a caller could still reach.
    #[must_use]
    pub fn follow(&self, sent_to: &AdmittedOrigin, location: &Origin) -> WhatARedirectDoes {
        if location == sent_to.origin() {
            return WhatARedirectDoes::Followed(sent_to.clone());
        }
        WhatARedirectDoes::Refused(Failure::answer_not_understood(
            ReadingSite::CrossOriginRedirectRefused,
            Expected::AnOriginTheCoreMayContact,
            A_REFUSED_REDIRECT_CONSUMED_NOTHING,
        ))
    }

    fn held(&self) -> MutexGuard<'_, Vec<Origin>> {
        self.held.lock().expect("the set holds no poisoned lock")
    }
}

#[cfg(test)]
mod tests {
    //! Every origin below is produced by parsing an address, rather than built
    //! from parts here, because 0069 compares the origin as it was resolved on
    //! the way in and a value assembled in this file would prove the comparison
    //! against something no address could produce.

    use super::{
        A_REFUSED_REDIRECT_CONSUMED_NOTHING, Destinations, WhatARedirectDoes, WhatConfiguringDid,
    };
    use crate::failure::{Expected, Failure, Kind, ReadingSite};
    use crate::server::address::{BaseAddress, Origin};

    fn origin(typed: &str) -> Origin {
        BaseAddress::parse(typed).expect("usable").origin()
    }

    fn configured(typed: &str) -> Destinations {
        let set = Destinations::new();
        set.configured(origin(typed));
        set
    }

    /// 0068 fixes that the set is empty before anything is configured, and 0069
    /// names that as what makes #70's failure unambiguous.
    #[test]
    fn a_set_nobody_configured_admits_nobody() {
        let set = Destinations::new();

        assert_eq!(set.rows(), 0);
        assert!(set.admit(&origin("https://films.example")).is_none());
        assert!(set.admit(&origin("http://127.0.0.1:8096")).is_none());
    }

    /// The one row 0069's table has, arriving the way that record says it does.
    #[test]
    fn the_origin_the_operator_configured_is_the_one_that_is_admitted() {
        let set = configured("https://films.example");

        assert_eq!(set.rows(), 1);
        assert!(set.admit(&origin("https://films.example")).is_some());
        assert!(set.admit(&origin("https://music.example")).is_none());
    }

    /// 0069 compares the origin as it was resolved on the way in rather than the
    /// address as it was typed, so two spellings of one server are one row.
    #[test]
    fn what_was_typed_is_compared_after_the_parse_rather_than_before_it() {
        let set = configured("HTTPS://Films.Example/jellyfin/");

        assert!(set.admit(&origin("https://films.example")).is_some());
        assert_eq!(
            set.configured(origin("https://FILMS.example/other")),
            WhatConfiguringDid::LeftTheSetAsItWas,
        );
        assert_eq!(set.rows(), 1);
    }

    /// 0069 refuses the host alone by name, because a typed `http://` is
    /// honoured rather than upgraded and the scheme is part of what the operator
    /// chose. The cost of the shorter comparison is in that record: an `https`
    /// origin redirected to `http` on the same host sends the session token in
    /// the clear on a rule written to protect the address.
    #[test]
    fn the_scheme_is_part_of_the_comparison() {
        let set = configured("https://films.example");

        assert!(set.admit(&origin("http://films.example")).is_none());
    }

    /// 0028 keeps what was typed and supplies no default port, so an origin that
    /// named one and an origin that did not are two.
    #[test]
    fn a_port_that_was_typed_and_a_port_that_was_not_are_two_origins() {
        let set = configured("https://films.example");

        assert!(set.admit(&origin("https://films.example:443")).is_none());
        assert!(set.admit(&origin("https://films.example:8920")).is_none());
    }

    /// A path is not part of an origin, which is 0069's own sentence and the
    /// half a reader gets wrong.
    #[test]
    fn a_server_at_a_sub_path_is_the_same_origin_as_one_at_the_root() {
        let set = configured("https://films.example/jellyfin");

        assert!(set.admit(&origin("https://films.example")).is_some());
        assert!(
            set.admit(&origin("https://films.example/anything"))
                .is_some()
        );
    }

    /// 0069 adds a row per configured server rather than widening one, and
    /// 0072's act is the second way one arrives.
    #[test]
    fn a_second_configured_server_adds_a_row_and_widens_nothing() {
        let set = configured("https://films.example");

        assert_eq!(
            set.configured(origin("https://music.example")),
            WhatConfiguringDid::AddedARow,
        );
        assert_eq!(set.rows(), 2);
        assert!(set.admit(&origin("https://films.example")).is_some());
        assert!(set.admit(&origin("https://music.example")).is_some());
        assert!(set.admit(&origin("https://photos.example")).is_none());
    }

    /// Removing one server says nothing about another, and the shortest
    /// implementation of it empties the set.
    #[test]
    fn forgetting_one_server_leaves_every_other_row_standing() {
        let set = configured("https://films.example");
        set.configured(origin("https://music.example"));

        assert!(set.forgotten(&origin("https://films.example")));
        assert!(!set.forgotten(&origin("https://films.example")));
        assert_eq!(set.rows(), 1);
        assert!(set.admit(&origin("https://music.example")).is_some());
    }

    /// The ordinary case 0069 names: a server correcting a path, a trailing
    /// separator or a version prefix.
    #[test]
    fn a_redirect_inside_the_origin_it_was_sent_to_is_followed() {
        let set = configured("https://films.example");
        let sent_to = set
            .admit(&origin("https://films.example"))
            .expect("in the set");

        let followed = set.follow(&sent_to, &origin("https://films.example/Items/1"));

        assert_eq!(followed, WhatARedirectDoes::Followed(sent_to));
    }

    /// The failure this board came within one line of configuration of, which is
    /// 0069's own reason for being written before the transport: every HTTP
    /// client anybody would reach for follows a redirect anywhere by default.
    #[test]
    fn a_redirect_leaving_that_origin_is_refused_with_the_kind_0069_takes() {
        let set = configured("https://films.example");
        let sent_to = set
            .admit(&origin("https://films.example"))
            .expect("in the set");

        let refused = set.follow(&sent_to, &origin("https://images.cdn.example/poster.jpg"));

        let WhatARedirectDoes::Refused(failure) = refused else {
            panic!("0069 refuses a redirect leaving the origin the request was sent to");
        };
        assert_eq!(failure.kind(), Kind::AnswerNotUnderstood);
        let Failure::AnswerNotUnderstood {
            site,
            expected,
            stopped_at,
            ..
        } = failure
        else {
            panic!("the refusal mapped onto something other than the catch-all");
        };
        assert_eq!(site, ReadingSite::CrossOriginRedirectRefused);
        assert_eq!(expected, Expected::AnOriginTheCoreMayContact);
        assert_eq!(stopped_at, A_REFUSED_REDIRECT_CONSUMED_NOTHING);
    }

    /// The refusal is against the origin the request went to and never against
    /// the whole set. A configured server is not trusted to move a request onto
    /// another configured server, because 0101 places that decision with the
    /// operator.
    #[test]
    fn a_redirect_to_a_second_configured_server_is_refused_too() {
        let set = configured("https://films.example");
        set.configured(origin("https://music.example"));
        let sent_to = set
            .admit(&origin("https://films.example"))
            .expect("in the set");

        let refused = set.follow(&sent_to, &origin("https://music.example/Audio/1"));

        assert!(matches!(refused, WhatARedirectDoes::Refused(_)));
        assert!(set.admit(&origin("https://music.example")).is_some());
    }

    /// The near miss for the comparison the redirect rule makes: one port apart
    /// on the same host and scheme is a different origin, and 0069's alternative
    /// that compares hosts is what admits it.
    #[test]
    fn a_redirect_to_another_port_on_the_same_host_is_refused() {
        let set = configured("https://films.example");
        let sent_to = set
            .admit(&origin("https://films.example"))
            .expect("in the set");

        let refused = set.follow(&sent_to, &origin("https://films.example:8920/Items/1"));

        assert!(matches!(refused, WhatARedirectDoes::Refused(_)));
    }

    /// A downgrade to the clear is the case 0069 prices for the shorter
    /// comparison, and it is a redirect rather than only a configuration.
    #[test]
    fn a_redirect_downgrading_the_scheme_on_the_same_host_is_refused() {
        let set = configured("https://films.example");
        let sent_to = set
            .admit(&origin("https://films.example"))
            .expect("in the set");

        let refused = set.follow(&sent_to, &origin("http://films.example/Items/1"));

        assert!(matches!(refused, WhatARedirectDoes::Refused(_)));
    }

    /// The host a resolver may be given is reached through the admission and not
    /// beside it, which is 0069 putting resolution inside the set.
    #[test]
    fn the_name_a_resolver_is_given_comes_off_an_admitted_origin() {
        let set = configured("HTTPS://Films.Example");
        let admitted = set
            .admit(&origin("https://films.example"))
            .expect("in the set");

        assert_eq!(admitted.host(), "films.example");
        assert_eq!(admitted.origin(), &origin("https://films.example"));
    }
}
