//! Reaching a server.
//!
//! 0003 puts address parsing, the transport, timeouts, certificate validation,
//! retry and the mapping of every failure onto one error vocabulary inside the
//! core, and this is where they live. The records that decide them are 0027,
//! 0028, 0029, 0037, 0038 and 0069, and the issues that build them are #27
//! through #39.
//!
//! The mapping point 0037 requires is one place, and the type it produces lives
//! in [`crate::failure`] rather than here, so that a value of the failure set
//! cannot be built anywhere else.
//!
//! [`address`] holds the first of them. 0028's rules are applied where an address
//! enters the core and nowhere else, and every request path is appended to the
//! result by the one routine that module carries.
//!
//! [`transport`] holds 0027's bounds: the two per-attempt deadlines inside the
//! call deadline 0007 sets, how many requests may be outstanding and against
//! whom, how long an idle connection is kept, and how far a cancelled response
//! is read before the connection is closed instead. It holds no socket, and its
//! own documentation says why that is a decision rather than an omission.
//!
//! [`certificate`] holds 0029's one exception: which certificate an operator
//! pinned for which server, what a pin never vouches for, and what a client is
//! handed after a refusal so it can show one. It holds no validation, and its
//! own documentation says why that is the platform's rather than an omission.
//!
//! [`recovery`] holds 0045's schedule for a server that is gone: how long until
//! the next probe is due, where the doubling stops, the hour after which the
//! core stops asking, and what a client's attempt-now does to both. It holds no
//! probe, for the same reason [`transport`] holds no socket.
//!
//! [`retry`] holds 0038's one policy for every request: which kinds are retried
//! inside a call and which are handed to a renewal or to [`recovery`]'s
//! schedule, how many attempts a call may spend, the interval each wait is drawn
//! over, and the seam that draw enters through. It holds no loop, for the same
//! reason [`transport`] holds no socket, and it is where both spreads in this
//! module take their draw.
//!
//! [`federation`] holds what 0072 decides: a second host becomes reachable only
//! through an act a person performed, against one server, naming what it shares,
//! and revocable without the network. Which hosts may be contacted at all is
//! 0069 and #69, and this is the register that would add one to that list.

pub mod address;
pub mod certificate;
pub mod federation;
pub mod recovery;
pub mod retry;
pub mod transport;
pub mod write_queue;

/// An answer the core has already received and handed back.
///
/// Thread safety, from 0009: a query result is immutable once it has been handed
/// back. There is no shared mutable state to protect, and the core keeps no
/// reference through which it could change one.
///
/// What a client can ask for, and what comes back, is #39.
#[derive(Debug)]
pub struct QueryResult {
    _private: (),
}
