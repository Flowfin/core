//! Caching what was fetched.
//!
//! 0003 puts the keys, the bound, the eviction, the age of an entry and whether
//! a stale entry may be served inside the core, and puts the location of storage
//! outside it. The records are 0006, 0040, 0041, 0042, 0043, 0046, 0047 and
//! 0105, and the issues are #40 through #48 and #105.
//!
//! 0041 requires a cryptographic digest for a cache key, 0011 measures that the
//! toolchain offers none, and 0103 is the rule that decides whether one may be
//! taken as a dependency. Nothing here is written against a digest that does not
//! exist yet.

/// The place a client lets the core put bytes.
///
/// The core is told where to write and never asks the platform, which is 0003's
/// sentence and 0040's record. #40 is the issue that decides what this asks of a
/// client.
///
/// Thread safety, from 0009: may be called from either lane and concurrently,
/// including for two entries at once. A client that assumed single-threaded
/// access would corrupt its own storage rather than producing a failure the core
/// could report, which is why the statement is here rather than left implied.
pub trait ByteStore: Send + Sync {}
