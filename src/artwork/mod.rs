//! Fetching and decoding artwork.
//!
//! 0003 keeps image decoding inside the core and stops at the bitmap: turning
//! bytes into pixels is a parse of untrusted input that arrived over a network,
//! and putting that bitmap on a surface is the client's. The records are 0050,
//! 0053, 0054 and 0055, and the issues are #49 through #55.

/// Pixels the core produced from bytes a server sent.
///
/// Thread safety, from 0009: a decoded image is immutable, and its bytes belong
/// to the caller from the moment they are handed over. The core does not read
/// them again.
///
/// What formats are decoded at all, and how the rest are refused by name, is
/// 0055 and #55. The bound checked before a decode is 0050 and #50.
#[derive(Debug)]
pub struct DecodedImage {
    _private: (),
}
