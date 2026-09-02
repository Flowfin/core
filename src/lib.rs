//! The shared core every Flowfin client uses.
//!
//! # What is here, and why it is arranged this way
//!
//! `docs/decisions/0003-what-the-core-does-not-do.md` draws the boundary: the
//! core owns everything between a server address and a decoded byte, and it owns
//! nothing a person can see. That record names six things the core owns, and
//! this crate has one module for each of them, so the boundary is visible in the
//! tree rather than only in a document.
//!
//! | module            | the thing 0003 says the core owns             |
//! |-------------------|-----------------------------------------------|
//! | [`server`]        | reaching a server                             |
//! | [`session`]       | holding a session                             |
//! | [`cache`]         | caching what was fetched                      |
//! | [`artwork`]       | fetching and decoding artwork                 |
//! | [`playback`]      | tracking playback position                    |
//! | [`measurement`]   | producing measurements                        |
//!
//! Three more modules are here and none of them is one of the six. [`failure`] holds
//! the error vocabulary the other six map onto, which 0003 places inside
//! "reaching a server" and which every one of them uses; splitting it out is a
//! layout choice rather than a boundary claim. [`diagnostics`] holds the sink a
//! client supplies, because
//! `docs/decisions/0009-the-concurrency-model.md` states a thread rule for that
//! sink and the rule has to be attached to a name a reader meets. [`clock`] holds
//! the one source all three clocks reach the core through, for the same reason:
//! `docs/decisions/0102-the-clocks-every-deadline-is-measured-against.md` states
//! a rule per clock, and a reading taken anywhere else would be a deadline no
//! test can move.
//!
//! # What is deliberately not here
//!
//! THIS SECTION SAID THAT EVERY TYPE BELOW IS A NAME WITH THE STATEMENT 0009
//! MAKES ABOUT ITS KIND AND NOTHING ELSE, AND THAT HAS STOPPED BEING TRUE. It
//! was written when the tree held no behaviour at all. `session` carries the
//! interface #33 defines, `measurement` carries the span facility #61 does, and
//! `playback` carries the unit and the two bounds 0056 fixes for #56, and each
//! was landed by the issue that owned it rather than by a layout deciding
//! anything. It was found while adding the second of the three, by reading this
//! file to place a module rather than by anything reporting it, and nothing here
//! reads it.
//!
//! What the sentence was for still holds where nothing has landed yet. A type
//! below whose issue has not been worked is a name with 0009's statement on it
//! and nothing else, and what it does belongs to the issue named beside it: a
//! layout that decided them would be deciding them in the file that was supposed
//! to hold them.
//!
//! # The thread statements are checked rather than written
//!
//! 0009 states its reentrancy and thread-safety rules per kind of object and
//! says each type carries the statement for its kind where a reader will meet
//! it. A doc comment is where a reader meets it; the assertions at the bottom of
//! this file are what refuses a change that breaks it. "Safe from any thread" is
//! `Send + Sync` here, and a field that is not thread-safe stops the crate
//! compiling rather than being caught in review.
//!
//! What that bound is worth is stated rather than implied, and it has moved. THIS
//! PARAGRAPH SAID THE TYPES BELOW HOLD NOTHING, SO NO ASSERTION COULD FAIL ON THE
//! BYTES IN THIS TREE. One of them holds something now: `measurement::Measurement`
//! carries a reference to a client's clock source, a reference to a client's
//! subscriber and a counter, and the assertion on it is what refuses a facility
//! that stopped being safe from any thread.
//!
//! THE SENTENCE AFTER IT SAID THE OTHER FOUR STILL HOLD NOTHING, AND IT IS NOT
//! REPLACED BY A NEW COUNT. It was written against a list of five and the list
//! below is longer than that, so a reader checking the four could not tell which
//! four were meant. A count written here goes stale on the next landing for the
//! same reason this one did, and the list is one command away:
//!
//! ```text
//! git grep -c 'any_thread::<' -- src/lib.rs
//! ```
//!
//! What the sentence was for is unchanged and does not need the number: an
//! assertion over a type holding nothing cannot fail on the bytes in this tree,
//! and it is there so that the day the type holds something is the day the
//! compiler starts judging it.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod artwork;
pub mod cache;
pub mod clock;
pub mod diagnostics;
pub mod failure;
pub mod measurement;
pub mod playback;
pub mod server;
pub mod session;

/// A running core.
///
/// Thread safety, from 0009: safe from any thread, always, including while it is
/// being stopped. It is the only object with no conditions on it.
///
/// What creating and stopping one means, and what happens to a host that
/// suspends it, is #115. This type carries no method for either.
#[derive(Debug)]
pub struct Core {
    _private: (),
}

/// Asserts at compile time that a type is safe to use from any thread.
///
/// This is the whole mechanism behind every "safe from any thread" statement in
/// this crate. It is a function rather than a comment so that the compiler is
/// what refuses the violation.
const fn any_thread<T: Send + Sync>() {}

const _: () = {
    any_thread::<Core>();
    any_thread::<session::Session>();
    any_thread::<session::device::DeviceIdentity>();
    any_thread::<session::device::Capabilities>();
    any_thread::<session::device::PartNotUsable>();
    any_thread::<session::delegated::TieValue>();
    any_thread::<session::delegated::Relayable>();
    any_thread::<session::delegated::OpenAttempts>();
    any_thread::<session::delegated::ValueNotUsable>();
    any_thread::<session::delegated::ValueAlreadyOpen>();
    any_thread::<session::delegated::NoAttemptMatched>();
    any_thread::<session::quick_connect::WhileWaiting>();
    any_thread::<session::quick_connect::HowTheCallEnded>();
    any_thread::<session::quick_connect::IssuedExchange>();
    any_thread::<session::renewal::Generation>();
    any_thread::<session::renewal::RenewalRoute>();
    any_thread::<session::renewal::Rejection>();
    any_thread::<session::renewal::WhatARejectedCallDoes>();
    any_thread::<session::renewal::HowTheRenewalEnded>();
    any_thread::<session::renewal::WhatTheOutcomeDoes>();
    any_thread::<session::renewal::Renewals>();
    any_thread::<session::renewal::RenewalSchedule>();
    any_thread::<measurement::Measurement<'static>>();
    any_thread::<diagnostics::Diagnostics<'static>>();
    any_thread::<diagnostics::redaction::Treatment>();
    any_thread::<diagnostics::redaction::FieldName>();
    any_thread::<diagnostics::redaction::CorrelatorSalt>();
    any_thread::<diagnostics::redaction::Correlator>();
    any_thread::<server::federation::Federation<'static>>();
    any_thread::<server::destinations::Destinations>();
    any_thread::<server::destinations::AdmittedOrigin>();
    any_thread::<server::destinations::WhatConfiguringDid>();
    any_thread::<server::destinations::WhatARedirectDoes>();
    any_thread::<server::library::PageRequest>();
    any_thread::<server::library::WhatTheReadAnswers>();
    any_thread::<server::library::LibraryRead>();
    any_thread::<server::library::NotAPagedRead>();
    any_thread::<server::library::WhatAskingForAPageDid>();
    any_thread::<server::library::Page<()>>();
    any_thread::<server::certificate::Fingerprint>();
    any_thread::<server::certificate::PresentedChain<'static>>();
    any_thread::<server::certificate::Refused<'static>>();
    any_thread::<server::certificate::PinnedServer>();
    any_thread::<server::certificate::Pin>();
    any_thread::<server::certificate::Pins>();
    any_thread::<server::recovery::WhileUnreachable>();
    any_thread::<server::address::BaseAddress>();
    any_thread::<server::address::Origin>();
    any_thread::<server::transport::CallDeadline>();
    any_thread::<server::transport::AttemptBound>();
    any_thread::<server::transport::Outstanding>();
    any_thread::<server::transport::Waits>();
    any_thread::<server::transport::EndsAConnection>();
    any_thread::<server::transport::ACancelledBody>();
    any_thread::<server::transport::IdleConnections<()>>();
    any_thread::<server::write_queue::WhatIsAsserted>();
    any_thread::<server::write_queue::Target>();
    any_thread::<server::write_queue::Entry<()>>();
    any_thread::<server::write_queue::Dropped>();
    any_thread::<server::write_queue::WhatTheEnqueueDid>();
    any_thread::<server::write_queue::WriteQueue<()>>();
    any_thread::<server::address::AddressNotUsable>();
    any_thread::<server::retry::Attempts>();
    any_thread::<server::retry::WhatTheRequestDoes>();
    any_thread::<server::retry::WhatAFailureDoes>();
    any_thread::<server::retry::TheWait>();
    any_thread::<server::retry::WhyTheCallStopped>();
    any_thread::<server::retry::WhatTheCallDoesNext>();
    any_thread::<artwork::DecodedImage>();
    any_thread::<artwork::address::ImageKind>();
    any_thread::<artwork::address::Edge>();
    any_thread::<artwork::address::SizeNotUsable>();
    any_thread::<artwork::address::DrawnSize>();
    any_thread::<artwork::address::NotUsableInARequest>();
    any_thread::<artwork::address::ItemId>();
    any_thread::<artwork::address::ImageTag>();
    any_thread::<artwork::address::ArtworkRequest>();
    any_thread::<artwork::announced::WhatTheAnnouncementDid>();
    any_thread::<artwork::announced::AnnouncedWindow>();
    any_thread::<artwork::announced::WhatTheHoldDid>();
    any_thread::<artwork::announced::WhatTheWithdrawalDid>();
    any_thread::<artwork::announced::SharedFetches>();
    any_thread::<artwork::format::Accepted>();
    any_thread::<artwork::format::DeclaredDimensions>();
    any_thread::<artwork::format::Refused>();
    any_thread::<artwork::format::Admitted>();
    any_thread::<artwork::presence::WhatTheItemHas>();
    any_thread::<artwork::budget::Budget>();
    any_thread::<artwork::budget::BudgetNotUsable>();
    any_thread::<artwork::budget::DecodedBytes>();
    any_thread::<artwork::budget::WhatTheAskDoes>();
    any_thread::<artwork::budget::DecodedBytesHeld>();
    any_thread::<artwork::shape::RatioNotUsable>();
    any_thread::<artwork::shape::AspectRatio>();
    any_thread::<artwork::shape::WhatShapeIsKnown>();
    any_thread::<artwork::shape::ReservedRectangle>();
    any_thread::<playback::Ticks>();
    any_thread::<playback::AdmittedPosition>();
    any_thread::<playback::cadence::ReportsWithoutWaiting>();
    any_thread::<playback::cadence::WhatItDoesToTheInterval>();
    any_thread::<playback::cadence::TheInterval>();
    any_thread::<playback::resume::Resume>();
    any_thread::<playback::resume::PositionInForce>();
    any_thread::<playback::watched::Marked>();
    any_thread::<playback::watched::MarkedBy>();
    any_thread::<cache::bound::TieredCache<'static>>();
    any_thread::<cache::bound::CacheBounds>();
    any_thread::<cache::bound::Tier>();
    any_thread::<cache::envelope::Entries<'static>>();
    any_thread::<cache::envelope::Drops>();
    any_thread::<cache::envelope::WhichCheckFailed>();
    any_thread::<cache::freshness::EntryKind>();
    any_thread::<cache::freshness::Skew>();
    any_thread::<cache::freshness::WrittenAt>();
    any_thread::<cache::freshness::WhyTheAgeIsUnreadable>();
    any_thread::<cache::freshness::Age>();
    any_thread::<cache::freshness::Held>();
    any_thread::<cache::freshness::Answer>();
};
