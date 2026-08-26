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
//! Two more modules are here and neither is one of the six. [`failure`] holds
//! the error vocabulary the other six map onto, which 0003 places inside
//! "reaching a server" and which every one of them uses; splitting it out is a
//! layout choice rather than a boundary claim. [`diagnostics`] holds the sink a
//! client supplies, because
//! `docs/decisions/0009-the-concurrency-model.md` states a thread rule for that
//! sink and the rule has to be attached to a name a reader meets.
//!
//! # What is deliberately not here
//!
//! Behaviour. Every type below is a name with the statement 0009 makes about its
//! kind, and nothing else. The interfaces, the fields and what any of it does
//! belong to the issues named beside each one, and a layout that decided them
//! would be deciding them in the file that was supposed to hold them.
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
//! What that bound is worth today is stated rather than implied: the types below
//! hold nothing, so no assertion can fail on the bytes in this tree. It bites on
//! the first field, which is the change these assertions exist for.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod artwork;
pub mod cache;
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
    any_thread::<server::QueryResult>();
    any_thread::<artwork::DecodedImage>();
};
