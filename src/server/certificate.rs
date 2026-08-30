//! The one exception 0029 admits, and everything it never becomes.
//!
//! `docs/decisions/0029-certificate-validation-and-the-self-signed-server.md`
//! decides that validation is always on, that nothing the core reads on its own
//! weakens it, and that the single exception is one exact certificate an
//! operator pinned through a client. This module is the register that exception
//! lives in and the rule that reads it.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything 0029 decides that a comparison of two byte strings
//! settles: what a fingerprint is taken over, which certificate of a presented
//! chain it is taken of, which server a pin reaches, what a pin never vouches
//! for, and what a client can read back and remove. Each of those is decided
//! once per connection, each is provable in microseconds, and each is wrong in a
//! way nothing downstream would report.
//!
//! WHAT IS NOT HERE IS THE VALIDATION, and it is absent for a reason rather than
//! for want of time. 0029 puts the chain, the name and the validity window with
//! the platform in so many words - the core validates "using the platform's own
//! trust store and the platform's own path building" and "does not reimplement
//! it" - and nothing in this tree reaches a platform. There is no socket either,
//! for the reason [`super::transport`] gives about itself. So no value of
//! [`CertificateReason`] is produced here by judging a certificate; the six
//! classes are what a platform's refusal is mapped onto, and that mapping
//! arrives with the connection in #27 and with #29's second condition.
//!
//! What follows from that, said once so a later reading does not take it for
//! more: nothing below decides that a certificate is acceptable. It decides that
//! a certificate an operator already asserted is theirs is the one that
//! answered, which is the whole of the exception and none of the rule.
//!
//! # The pin does not survive the process
//!
//! 0029 puts a pin in the byte store of #40 under a key built the way #41 builds
//! one, keyed by the server and the device and never by the account, and says
//! that with no store supplied the pin lives as long as the process and an
//! operator pins once per run. That last sentence is the state of this tree: the
//! register below holds its pins in memory, so what a client gets today is the
//! no-store behaviour the record already decided rather than a shortcut taken
//! here. Nothing about the account reaches any call in this module, which is how
//! that half of the key is kept: there is no parameter to pass one in.
//!
//! # The digest is chosen here and 0029 does not name it
//!
//! The record fixes WHAT is hashed - the whole of a certificate's encoding
//! rather than its public key alone - and does not fix with what. This module
//! takes SHA-256, for the record's own reason: a pin an operator cannot check by
//! comparing two strings is a pin they will accept without checking, and what a
//! server prints when its operator asks what certificate it is serving is a
//! SHA-256 fingerprint. A record naming a second digest is what would move this,
//! and until one does, a reader should take the choice as an argument rather
//! than as a measurement.

use core::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::failure::{CertificateReason, TransportOutcome};
use crate::server::address::BaseAddress;
use std::sync::Mutex;

/// One certificate, identified the way an operator can check it.
///
/// 0029 fixes the identification as a fingerprint "over the whole of its
/// encoding rather than over its public key alone", and gives the reason: the
/// whole encoding is what a server prints when its operator asks it what
/// certificate it is serving, and a pin nobody can check by comparing two
/// strings is a pin taken on faith.
///
/// Held as the written-out form rather than as bytes, because every use of one
/// is a comparison against something a person read off their own machine.
///
/// Thread safety, from 0009: immutable once taken. There is no method that
/// changes one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint {
    written: String,
}

impl Fingerprint {
    /// Takes the fingerprint of one certificate, over the whole of its encoding.
    ///
    /// The argument is the certificate exactly as it arrived. Nothing here
    /// parses it, skips a field or normalises a byte: a fingerprint over
    /// anything but the bytes that arrived is one an operator cannot reproduce.
    #[must_use]
    pub fn of(encoding: &[u8]) -> Self {
        let mut written = String::with_capacity(Sha256::output_size() * 2);
        for byte in Sha256::digest(encoding) {
            let _ = write!(written, "{byte:02x}");
        }
        Self { written }
    }

    /// The fingerprint as it is compared and shown, lower-case and hexadecimal.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.written
    }
}

/// A chain as a server presented it.
///
/// The end-entity certificate first, then whatever the server sent with it.
///
/// WHICH MEMBER 0029 CALLS THE END OF THE CHAIN IS READ HERE RATHER THAN AT A
/// CALL SITE. That record asks for "the fingerprint of the certificate at the
/// end of the presented chain" and says in the same sentence that it is what a
/// person compares against their own server. Only one member is that: the
/// certificate the server proves itself with. It is the first member on the
/// wire, so the two orderings a reader might have in mind disagree about the
/// word and agree about the certificate, and [`PresentedChain::end_entity`] is
/// the one place the reading is made.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentedChain<'a> {
    certificates: &'a [&'a [u8]],
}

impl<'a> PresentedChain<'a> {
    /// A chain as it arrived, end-entity certificate first.
    #[must_use]
    pub const fn as_presented(certificates: &'a [&'a [u8]]) -> Self {
        Self { certificates }
    }

    /// The certificate the server proved itself with, where it sent one.
    ///
    /// A chain with no members answers [`None`]. A peer that presented nothing
    /// has nothing for an operator to have pinned, and treating an absent
    /// certificate as a value is what would let it compare equal to something.
    #[must_use]
    pub const fn end_entity(&self) -> Option<&'a [u8]> {
        self.certificates.first().copied()
    }

    /// Every member, in the order the server sent them.
    ///
    /// 0029 hands a client "the presented chain as it arrived" so it can show
    /// what was refused, and this is that.
    #[must_use]
    pub const fn members(&self) -> &'a [&'a [u8]] {
        self.certificates
    }
}

/// What the core hands a client after it refused a peer.
///
/// 0029 lists what a client needs in hand before it may offer a person a pin:
/// the presented chain as it arrived, the fingerprint of the certificate at its
/// end, the reason class, and the subject, the issuer and the validity window as
/// data.
///
/// THE LAST THREE ARE NOT HERE AND THIS IS THE NEGATIVE STATEMENT ABOUT IT. A
/// subject, an issuer and a validity window are fields inside a certificate, and
/// reading them is parsing one. 0029 leaves the reading of a certificate with
/// the platform and this tree reaches no platform, so a type carrying those
/// three today would carry whatever this module invented for them. They arrive
/// with the connection that produced the refusal, which is #27 and #29's second
/// condition, and nothing here should be read as covering them.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused<'a> {
    address: &'a str,
    chain: PresentedChain<'a>,
    fingerprint: Fingerprint,
    reason: CertificateReason,
}

impl<'a> Refused<'a> {
    /// Records what was presented and which of 0029's six classes refused it.
    ///
    /// The class is the platform's refusal mapped onto the closed set, so this
    /// call is made where a connection failed and nowhere else. It is public
    /// because the place that will make it is the transport in #27, which is a
    /// different module from this one.
    #[must_use]
    pub fn of(address: &'a str, chain: PresentedChain<'a>, reason: CertificateReason) -> Self {
        Self {
            address,
            chain,
            fingerprint: Fingerprint::of(chain.end_entity().unwrap_or_default()),
            reason,
        }
    }

    /// The address that was contacted.
    #[must_use]
    pub const fn address(&self) -> &'a str {
        self.address
    }

    /// The chain as it arrived.
    #[must_use]
    pub const fn chain(&self) -> PresentedChain<'a> {
        self.chain
    }

    /// The fingerprint of the certificate at the end of that chain.
    ///
    /// This is the value an operator compares against what their own server
    /// prints, and the value a client hands back to [`Pins::pin`] if they say it
    /// is theirs.
    #[must_use]
    pub const fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }

    /// Which of the six classes it was.
    #[must_use]
    pub const fn reason(&self) -> CertificateReason {
        self.reason
    }

    /// The refusal as the transport reports it, in 0004's vocabulary.
    ///
    /// The failure value is built at the one mapping point 0037 requires, and
    /// this is what that point is given: the class and the fingerprint, and
    /// never the chain, because 0004 fixes what `certificate-rejected` carries
    /// and a chain is not one of its fields.
    #[must_use]
    pub fn as_transport_outcome(&self) -> TransportOutcome<'_> {
        TransportOutcome::PeerNotTrusted {
            reason: self.reason,
            fingerprint: self.fingerprint.as_str(),
        }
    }
}

/// The server a pin belongs to.
///
/// 0029 fixes it as the resolved identity in 0006 and, where the core does not
/// have one yet because the pin is being taken during the first connection, as
/// the base address from 0028 with no other address inheriting it. Those are the
/// same two things 0041 makes the server part of a cache key out of.
///
/// IT IS AN OWNED VALUE RATHER THAN THAT TYPE, and the difference is not a
/// preference. [`crate::cache::key::ServerPart`] borrows for the length of one
/// derivation, and a register outlives every call made against it. Where a pin
/// is written to the store in #40, the value below is what a key is derived
/// from, so the two arms are the same two on purpose.
///
/// Thread safety, from 0009: immutable once made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedServer {
    /// The identifier the server reported about itself.
    Reported(String),
    /// The base address from 0028, where the server offered no identifier.
    ///
    /// Two addresses that reach one server are two entries here, which is the
    /// cost 0041 states for the same fallback and fails in the same safe
    /// direction: a pin taken at one address does not answer at another.
    Address(BaseAddress),
}

/// One pin, as a client recorded it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    server: PinnedServer,
    fingerprint: Fingerprint,
}

impl Pin {
    /// Which server it reaches, and no other.
    #[must_use]
    pub const fn server(&self) -> &PinnedServer {
        &self.server
    }

    /// The one certificate it accepts.
    #[must_use]
    pub const fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
}

/// Every certificate an operator pinned on this device.
///
/// Thread safety, from 0009: safe from any thread. Every connection would
/// consult it, and a register that were not would make every handshake a
/// synchronisation point.
#[derive(Debug)]
pub struct Pins {
    held: Mutex<Vec<Pin>>,
}

impl Default for Pins {
    fn default() -> Self {
        Self::new()
    }
}

impl Pins {
    /// A register with no pins in it.
    ///
    /// This is the whole of "no trust on first use": there is no other
    /// constructor, nothing is recorded by answering a connection, and a refusal
    /// is never remembered as an acceptance. A register nobody has called
    /// [`Pins::pin`] on accepts nothing from anybody.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            held: Mutex::new(Vec::new()),
        }
    }

    /// Records that one exact certificate, for one server, is acceptable.
    ///
    /// The core does not decide that it is. A client asks the operator, having
    /// shown them what [`Refused`] carries, and tells the core what they
    /// answered.
    ///
    /// A second call against a server that already carries a pin replaces it,
    /// because a server presents one certificate and two pins against it would
    /// be two answers to one question with nothing to say which the operator
    /// meant. The replaced fingerprint is answered back, so a client can show
    /// what an operator has just overridden.
    ///
    /// IT TAKES A FINGERPRINT RATHER THAN A [`Refused`], AND THAT IS DELIBERATE.
    /// Requiring the refusal would read as the stronger interface and would
    /// refuse the case 0029 depends on: a pin lives in the store in #40 and is
    /// read back at the start of the next run, where no refusal has happened
    /// yet. What keeps a client to the record is the record and the review
    /// rather than this signature.
    pub fn pin(&self, server: PinnedServer, fingerprint: Fingerprint) -> Option<Fingerprint> {
        let mut held = self.held();
        match held.iter_mut().find(|pin| pin.server == server) {
            Some(pin) => Some(core::mem::replace(&mut pin.fingerprint, fingerprint)),
            None => {
                held.push(Pin {
                    server,
                    fingerprint,
                });
                None
            }
        }
    }

    /// Whether this chain is the certificate an operator pinned for this server.
    ///
    /// This is the question a connection asks, and it is asked per connection
    /// rather than per request, because 0029 reuses a pinned connection like any
    /// other.
    ///
    /// It answers `true` only where the END-ENTITY certificate's fingerprint is
    /// the pinned one. THIS IS THE WHOLE OF WHAT A PIN NEVER BECOMES. 0029
    /// refuses a certificate signed by the pinned certificate and refuses a
    /// chain that merely contains it, and both fall out of comparing one member:
    /// a chain whose end entity is something else does not match, whoever
    /// vouched for it and whatever else the server sent along. The convenient
    /// version searches the chain, and that version turns one server's key into
    /// something that can answer for any name at all, including the ones the
    /// person did not type.
    ///
    /// A server nobody pinned answers `false`, which is validation answering for
    /// every server nobody made an exception for.
    #[must_use]
    pub fn admits(&self, server: &PinnedServer, chain: &PresentedChain<'_>) -> bool {
        let Some(end_entity) = chain.end_entity() else {
            return false;
        };
        let presented = Fingerprint::of(end_entity);
        self.held()
            .iter()
            .any(|pin| &pin.server == server && pin.fingerprint == presented)
    }

    /// What is pinned for one server, where anything is.
    #[must_use]
    pub fn pinned(&self, server: &PinnedServer) -> Option<Fingerprint> {
        self.held()
            .iter()
            .find(|pin| &pin.server == server)
            .map(|pin| pin.fingerprint.clone())
    }

    /// Every pin this device holds.
    ///
    /// 0029 requires a pin to be visible: a client can read back which servers
    /// carry one and what fingerprint each holds, which is what makes it a
    /// decision an operator can revisit rather than one taken once in the dark.
    #[must_use]
    pub fn all(&self) -> Vec<Pin> {
        self.held().clone()
    }

    /// Removes the pin against one server, and answers whether there was one.
    ///
    /// It reaches one entry. A register with several pins in it loses the named
    /// one and keeps the rest, because removing an exception for one server is
    /// not a statement about any other, and the shortest implementation empties
    /// the register.
    pub fn remove(&self, server: &PinnedServer) -> bool {
        let mut held = self.held();
        let before = held.len();
        held.retain(|pin| &pin.server != server);
        held.len() != before
    }

    fn held(&self) -> std::sync::MutexGuard<'_, Vec<Pin>> {
        self.held
            .lock()
            .expect("the register holds no poisoned lock")
    }
}

#[cfg(test)]
mod tests {
    //! What 0029 decides that a comparison of two byte strings settles.
    //!
    //! What these cannot ask is whether any connection consults the register,
    //! because there is no connection: nothing in this tree opens a socket or
    //! validates a certificate. That is #29's second condition, and it is
    //! written into that issue rather than implied by a passing run here.
    //!
    //! No certificate below is a certificate. They are byte strings, because
    //! every property under test is a property of a digest and of a comparison,
    //! and a real encoding would prove the same things while suggesting that
    //! something here had parsed one.

    use super::{Fingerprint, Pin, PinnedServer, Pins, PresentedChain, Refused};
    use crate::failure::{CertificateReason, TransportOutcome};
    use crate::server::address::BaseAddress;

    /// The operator's own certificate, as it arrives.
    const OPERATORS: &[u8] = b"the certificate the operator issued for their own server";

    /// One byte different from it, which is the near miss a digest has to
    /// separate.
    const ALMOST: &[u8] = b"the certificate the operator issued for their own servet";

    /// Something else entirely, standing for a certificate the pinned one
    /// vouched for.
    const VOUCHED_FOR: &[u8] = b"a certificate the operator's certificate signed";

    fn a_server() -> PinnedServer {
        PinnedServer::Address(BaseAddress::parse("films.example").expect("usable"))
    }

    fn another_server() -> PinnedServer {
        PinnedServer::Address(BaseAddress::parse("music.example").expect("usable"))
    }

    fn pinned_to(server: PinnedServer, certificate: &[u8]) -> Pins {
        let pins = Pins::new();
        pins.pin(server, Fingerprint::of(certificate));
        pins
    }

    /// The written form is checked against a value produced outside this
    /// repository, so that the digest is under test rather than this file
    /// agreeing with itself.
    ///
    ///     printf '' | sha256sum
    ///     e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  -
    #[test]
    fn the_fingerprint_is_a_sha_256_over_the_bytes_that_arrived() {
        assert_eq!(
            Fingerprint::of(b"").as_str(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// 0029 takes the fingerprint over the whole encoding rather than over the
    /// public key alone, so two certificates sharing everything but one byte are
    /// two pins.
    #[test]
    fn one_byte_of_difference_is_a_different_certificate() {
        assert_ne!(Fingerprint::of(OPERATORS), Fingerprint::of(ALMOST));
    }

    /// The first of 0029's absent switches: a register nobody pinned anything on
    /// accepts nothing. There is no constructor that starts with a pin in it and
    /// nothing records one by answering a connection.
    #[test]
    fn nothing_is_admitted_until_a_client_pins_something() {
        let pins = Pins::new();

        assert!(!pins.admits(&a_server(), &PresentedChain::as_presented(&[OPERATORS])));
        assert_eq!(pins.pinned(&a_server()), None);
        assert!(pins.all().is_empty());
    }

    /// #29's third condition. A pin is scoped to one server and does not widen
    /// to a second, so the same certificate presented by another server is
    /// refused.
    #[test]
    fn a_pin_reaches_one_server_and_no_other() {
        let pins = pinned_to(a_server(), OPERATORS);
        let presented = PresentedChain::as_presented(&[OPERATORS]);

        assert!(pins.admits(&a_server(), &presented));
        assert!(!pins.admits(&another_server(), &presented));
    }

    /// What a pin never becomes, first direction: a certificate the pinned one
    /// signed is not accepted, even though the chain carries the pinned
    /// certificate as its issuer.
    #[test]
    fn a_certificate_the_pinned_one_vouched_for_is_not_admitted() {
        let pins = pinned_to(a_server(), OPERATORS);

        assert!(!pins.admits(
            &a_server(),
            &PresentedChain::as_presented(&[VOUCHED_FOR, OPERATORS])
        ));
    }

    /// What a pin never becomes, second direction: a chain that merely contains
    /// the pinned certificate somewhere other than at its end is not accepted.
    #[test]
    fn a_chain_that_merely_contains_the_pinned_certificate_is_not_admitted() {
        let pins = pinned_to(a_server(), OPERATORS);

        assert!(!pins.admits(
            &a_server(),
            &PresentedChain::as_presented(&[ALMOST, OPERATORS, VOUCHED_FOR])
        ));
    }

    /// A peer that presented nothing has nothing an operator could have pinned.
    #[test]
    fn a_chain_with_no_members_is_not_admitted() {
        let pins = pinned_to(a_server(), OPERATORS);

        assert!(!pins.admits(&a_server(), &PresentedChain::as_presented(&[])));
    }

    /// 0029's no accept-once. A refusal carries everything a client needs to ask
    /// somebody, and carrying it changes nothing in the register: a retry after
    /// a refusal is refused again, and only the pin call moves anything.
    #[test]
    fn a_refusal_is_not_an_acceptance() {
        let pins = Pins::new();
        let presented = PresentedChain::as_presented(&[OPERATORS]);
        let refused = Refused::of(
            "https://films.example",
            presented,
            CertificateReason::SelfSigned,
        );

        assert_eq!(refused.fingerprint(), &Fingerprint::of(OPERATORS));
        assert!(!pins.admits(&a_server(), &presented));

        pins.pin(a_server(), refused.fingerprint().clone());

        assert!(pins.admits(&a_server(), &presented));
    }

    /// What a client is given to show a person, and what the mapping point in
    /// 0037 is handed.
    #[test]
    fn a_refusal_carries_the_chain_the_class_and_the_fingerprint() {
        let members: [&[u8]; 2] = [OPERATORS, VOUCHED_FOR];
        let refused = Refused::of(
            "https://films.example",
            PresentedChain::as_presented(&members),
            CertificateReason::NameMismatch,
        );

        assert_eq!(refused.address(), "https://films.example");
        assert_eq!(refused.chain().members(), &members);
        assert_eq!(refused.reason(), CertificateReason::NameMismatch);

        let TransportOutcome::PeerNotTrusted {
            reason,
            fingerprint,
        } = refused.as_transport_outcome()
        else {
            panic!("a refused peer is not trusted");
        };
        assert_eq!(reason, CertificateReason::NameMismatch);
        assert_eq!(fingerprint, Fingerprint::of(OPERATORS).as_str());
    }

    /// 0029 requires a pin to be visible and removable, and removing one is a
    /// statement about one server rather than about the register.
    #[test]
    fn a_pin_is_visible_and_removable_without_reaching_another_server() {
        let pins = pinned_to(a_server(), OPERATORS);
        pins.pin(another_server(), Fingerprint::of(ALMOST));

        assert_eq!(
            pins.all()
                .iter()
                .map(Pin::server)
                .cloned()
                .collect::<Vec<_>>(),
            vec![a_server(), another_server()]
        );
        assert_eq!(pins.pinned(&a_server()), Some(Fingerprint::of(OPERATORS)));

        assert!(pins.remove(&a_server()));
        assert!(!pins.remove(&a_server()));

        assert_eq!(pins.pinned(&a_server()), None);
        assert_eq!(
            pins.pinned(&another_server()),
            Some(Fingerprint::of(ALMOST))
        );
    }

    /// A server presents one certificate, so a second pin against it answers the
    /// same question a second time and replaces the first rather than standing
    /// beside it.
    #[test]
    fn a_second_pin_against_one_server_replaces_the_first() {
        let pins = pinned_to(a_server(), OPERATORS);

        assert_eq!(
            pins.pin(a_server(), Fingerprint::of(ALMOST)),
            Some(Fingerprint::of(OPERATORS))
        );

        assert_eq!(pins.all().len(), 1);
        assert!(!pins.admits(&a_server(), &PresentedChain::as_presented(&[OPERATORS])));
        assert!(pins.admits(&a_server(), &PresentedChain::as_presented(&[ALMOST])));
    }

    /// 0029 holds a pin against the base address where the server offered no
    /// identifier of its own, and says no other address inherits it. An
    /// identifier that reads the same is a different server part, which is the
    /// distinction 0041 already pays a written part to keep.
    #[test]
    fn an_address_pin_does_not_reach_an_identifier_that_reads_the_same() {
        let pins = pinned_to(a_server(), OPERATORS);
        let reported = PinnedServer::Reported("films.example".to_owned());

        assert!(!pins.admits(&reported, &PresentedChain::as_presented(&[OPERATORS])));
    }
}
