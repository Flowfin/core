//! Tracking playback position.
//!
//! 0003 puts the unit, the precision, the reporting cadence, what happens to a
//! position recorded while the server was gone, and what counts as watched
//! inside the core. The records are 0056, 0057, 0058, 0060 and 0111, and the
//! issues are #56 through #60 and #111.
//!
//! Video decoding is outside the core, for the reason 0112 records. The core
//! stops at the handover in #111.
//!
//! 0056 fixes the unit a position is expressed in against the server rather than
//! against whatever duration type the runtime offers, and 0011 measures what that
//! type actually is on the chosen toolchain: unsigned, and in nanoseconds. The
//! conversion at the boundary is 0056's, and nothing here adopts the runtime type
//! as the wire unit.
