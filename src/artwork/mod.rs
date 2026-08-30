//! Fetching and decoding artwork.
//!
//! 0003 keeps image decoding inside the core and stops at the bitmap: turning
//! bytes into pixels is a parse of untrusted input that arrived over a network,
//! and putting that bitmap on a surface is the client's. The records are 0050,
//! 0053, 0054 and 0055, and the issues are #49 through #55.
//!
//! # What is here today
//!
//! Everything 0055 puts before a decoder, in [`format`]: which three formats are
//! accepted, the signature match that decides which one a response is from the
//! bytes rather than from what the server declared, the bound on encoded length,
//! and the bound on the dimensions a header declares, read before any buffer for
//! pixels exists.
//!
//! Everything 0049 puts before a request, in [`address`]: the five image kinds,
//! the ladder a requested size is rounded onto so that two nearby tiles share
//! one entry, the content tag 0006 depends on, and the refusal of an identifier
//! or a tag whose bytes would let a server choose part of the request.
//!
//! Everything #51 puts between the two, in [`presence`]: what an item has for
//! one image kind, where an item with no tag for a kind stops being a request
//! that was never built and becomes an answer a client can show, and why a tag
//! the core refused is neither that answer nor the same thing as one.
//!
//! Everything 0053 puts in front of the fetch, in [`announced`]: the ordered
//! window a client announces, what a window longer than the bound costs and how
//! a client is told, and which callers are sharing one entry's fetch so that the
//! last withdrawal is the one that abandons it.
//!
//! What is absent is the decoder itself. [`DecodedImage`] is still a name, the
//! decode and its memory budget are #50, and nothing in this tree turns admitted
//! bytes into pixels. What is also absent is the fetch: [`address`] builds the
//! address and derives the key, and the transport that would go and get it is
//! #27.

pub mod address;
pub mod announced;
pub mod format;
pub mod presence;

/// Pixels the core produced from bytes a server sent.
///
/// Thread safety, from 0009: a decoded image is immutable, and its bytes belong
/// to the caller from the moment they are handed over. The core does not read
/// them again.
///
/// THIS PARAGRAPH SENT A READER TO #55 FOR WHICH FORMATS ARE DECODED AT ALL AND
/// THAT HALF IS NOW IN [`format`]. What is still absent here is the decode: the
/// bound on decoded bytes held at once is 0050 and #50, and it is a different
/// quantity from the per-image bound [`format`] enforces.
#[derive(Debug)]
pub struct DecodedImage {
    _private: (),
}
