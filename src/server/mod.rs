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
