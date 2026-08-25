//! What the core tells a client about itself.
//!
//! This is not one of the six things 0003 names either. It is here because 0009
//! states a thread rule for the sink a client supplies, and a rule with no name
//! to attach to is a rule a reader meets nowhere. The record is 0100 and the
//! issue is #100. What may leave through an event is 0071 and #71.

/// The place a client receives the core's diagnostic events.
///
/// Thread safety, from 0009: may be called from any lane, at any time, and
/// concurrently. It must be safe for that, it must not block, and it must not
/// call back into the core. The last of the three is the deadlock, so the
/// interface forbids it rather than documenting it.
///
/// What an event carries, and which fields may appear in one at all, is 0071 and
/// #71. Nothing here decides either.
pub trait DiagnosticsSink: Send + Sync {}
