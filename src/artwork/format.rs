//! Which formats reach a decoder at all, and what refuses the rest.
//!
//! 0055 is the record and #55 is the issue. The core decodes JPEG, PNG and WebP
//! and nothing else; it decides which of the three a response is by reading the
//! bytes rather than by believing what the server declared; and it refuses
//! anything outside the set, and anything declaring dimensions past a bound,
//! before a decoder is reached and before pixels are allocated.
//!
//! # Why the refusal is a list and not a failed decode
//!
//! 0055's own argument. Handing the bytes to a decoder to find out what they are
//! means the parser has already been handed the bytes, which is the whole of what
//! was supposed to be prevented, and the surface then becomes whichever formats
//! the decoder happens to support rather than a set anybody chose.
//!
//! There is no wider detection step either. Nothing here identifies a format
//! outside the accepted three in order to name it in a refusal, because such a
//! step is a second parser over untrusted bytes whose only purpose is a better
//! diagnostic, and it grows every time somebody adds a row to its table.
//!
//! # What this reads, and how far into the bytes
//!
//! [`Accepted::of`] reads at most [`SIGNATURE_PREFIX`] bytes and compares them
//! against three fixed strings. Nothing in it branches on anything else.
//!
//! [`admitted`] then reads the matched format's own header for the dimensions it
//! declares. That is a parse of untrusted bytes and it is written as one: every
//! read is bounds-checked, every branch that cannot make sense of what it finds
//! answers that the header declared no dimensions, and the one loop advances by
//! at least two bytes on every pass so that it ends at the end of what it was
//! given.
//!
//! # What is not here
//!
//! No decoder. Turning admitted bytes into pixels is #50, and this module
//! deliberately stops at the point where a decoder would be handed something.
//!
//! No failure of 0004's vocabulary. 0055 fixes every refusal below as
//! `answer-not-understood`, and 0037 requires that value to be built at one
//! mapping point, which is #37 and does not exist. [`Refused`] is a local answer
//! that says which check refused, the way [`crate::cache::StorageUnavailable`]
//! is for its own store, and the several values here become one kind at that
//! mapping point rather than a sixteenth one.
//!
//! Nothing applies the encoded-length bound during a transfer, because there is
//! no transfer. 0055 requires it to be applied while the response is being read
//! rather than after it is complete, so a server sending without end is refused
//! during the transfer; [`encoded_length_is_still_inside_its_bound`] is the
//! judgement a reader makes on a running count, and the reading is #27's and
//! #49's.
//!
//! Nothing here decides whether an admitted image is trusted. It is not. It is
//! decoded under every other rule 0101 sets for an untrusted parse, and the
//! decoder is in #86's target set for the same reason.

/// The formats the core decodes.
///
/// Three, and a fourth is a supersession of 0055 rather than a variant somebody
/// adds, because each of the three carries its argument in that record and each
/// costs a parser on the untrusted side of 0101 and a target in #86.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Accepted {
    /// What a photographic still is stored and served as.
    Jpeg,
    /// What anything with flat colour or transparency is stored as.
    Png,
    /// What a server's own resizing produces where it has one, which #49 makes
    /// the ordinary answer on this board rather than the unusual one.
    WebP,
}

/// How many leading bytes the signature match reads, and no more.
///
/// Twelve, which is the longest of the three signatures: WebP's is a container
/// tag at the front and a format tag eight bytes later, with four bytes of
/// length between them that say nothing about the format.
pub const SIGNATURE_PREFIX: usize = 12;

impl Accepted {
    /// Which of the three these bytes are, read from the bytes alone.
    ///
    /// `None` is every other input, including a format this repository has a
    /// name and a reason for. 0055 refuses AVIF, HEIC, SVG, GIF, BMP, TIFF and
    /// ICO with an argument each, and refuses everything unlisted by the same
    /// rule; none of those arguments is a branch here, because the accepted set
    /// is the rule and the named refusals are the ones worth their reasons.
    ///
    /// What the server declared the content type to be is not an argument to
    /// this call and cannot be. 0101 puts that declaration on its untrusted list
    /// and 0055 takes the consequence: the bytes win, and a disagreement between
    /// the two is not an error in itself.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(Self::Jpeg);
        }
        if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Some(Self::Png);
        }
        if bytes.starts_with(b"RIFF") && bytes.get(8..SIGNATURE_PREFIX) == Some(&b"WEBP"[..]) {
            return Some(Self::WebP);
        }
        None
    }
}

/// The bound on how many encoded bytes the core will hold for one image.
///
/// Sixteen megabytes, read as sixteen mebibytes, and the reading is stated
/// because 0055 writes the number in words. The difference between the two
/// readings is five per cent of a number that is chosen rather than measured, and
/// a buffer bound written as a power of two is the one a reader of this constant
/// expects; #65 is where a measured replacement for either comes from.
///
/// What it is for is not how large a picture looks. It is what a sender can make
/// the device hold, and 0055 applies it during the transfer rather than after it,
/// so a server sending without end is refused while it is still sending.
pub const ENCODED_LENGTH_BOUND: usize = 16 * 1024 * 1024;

/// The bound on either axis of the dimensions a header declares.
///
/// Eight thousand one hundred and ninety two. It is enforced beside
/// [`PIXEL_COUNT_BOUND`] and not instead of it: an axis bound alone admits an
/// image inside both axes and enormous in area.
pub const AXIS_BOUND: u32 = 8_192;

/// The bound on the total the two axes multiply to.
///
/// Sixteen million. It is enforced beside [`AXIS_BOUND`] and not instead of it: a
/// total alone admits a single row long enough to overflow an index somewhere
/// downstream.
///
/// Sixteen million pixels at four bytes each is sixty four megabytes, which is
/// far beyond any tile a client draws and is at the edge of what a television
/// will give a single allocation before the platform kills the process.
pub const PIXEL_COUNT_BOUND: u64 = 16_000_000;

/// Whether a transfer that has read this many bytes so far may continue.
///
/// This is the encoded-length bound in the form 0055 asks for it, which is a
/// judgement on a running count rather than on a finished response. A reader that
/// asked only once the response was complete would have already held everything
/// the sender sent, which is the case the bound exists against.
#[must_use]
pub const fn encoded_length_is_still_inside_its_bound(read_so_far: usize) -> bool {
    read_so_far <= ENCODED_LENGTH_BOUND
}

/// What a header declared before anything was allocated for it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeclaredDimensions {
    width: u32,
    height: u32,
}

impl DeclaredDimensions {
    /// The two numbers a header declared.
    #[must_use]
    pub const fn of(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// The declared width.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// The declared height.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    /// What the two multiply to.
    ///
    /// Widened before the multiplication rather than after it. Two numbers this
    /// type can carry multiply to more than either can hold, and a product that
    /// wrapped would arrive as a small number that passes the bound, which is
    /// the shape the bound exists against.
    #[must_use]
    pub const fn pixel_count(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Whether both bounds hold.
    #[must_use]
    pub const fn are_inside_their_bounds(self) -> bool {
        self.width <= AXIS_BOUND
            && self.height <= AXIS_BOUND
            && self.pixel_count() <= PIXEL_COUNT_BOUND
    }
}

/// Why a response will not reach a decoder.
///
/// Every one of these is `answer-not-understood` from 0004, which is one kind
/// rather than four. 0055 says as much and says the fit is imperfect: a refused
/// format is a shape the core recognised and declined on purpose, and the word in
/// the kind says the opposite. What is here is which check refused, which is what
/// a diagnostic event under #100 would carry and what a reader of a refusal
/// wants; the mapping onto the kind is #37's and is not made anywhere yet.
///
/// A refusal is not the absent answer in #51. One says the server sent something
/// wrong and the other says the server has no image, and a client that cannot
/// tell them apart shows a failure sentence over a library that is fine.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// More encoded bytes than [`ENCODED_LENGTH_BOUND`] admits.
    TheEncodedLengthPassedItsBound,
    /// The leading bytes are none of the three accepted signatures. This is the
    /// refusal every unaccepted format arrives at, whether or not 0055 names it.
    TheSignatureMatchedNoAcceptedFormat,
    /// The signature matched and the header that follows it does not declare
    /// dimensions this reader can take out of it. A truncated header, a header
    /// whose own fields disagree with the format, and a header declaring zero in
    /// an axis are all this: each of the three formats forbids a zero axis, and
    /// in JPEG a declared height of zero means the number arrives later in the
    /// stream, which is a header that has not declared its dimensions.
    TheHeaderDeclaredNoDimensions,
    /// The header declared dimensions past [`AXIS_BOUND`] or
    /// [`PIXEL_COUNT_BOUND`]. A header declaring enormous dimensions costs the
    /// sender nothing and costs the device a buffer, which is why this is read
    /// before one is allocated rather than after.
    TheDeclaredDimensionsPassedTheirBound,
}

/// A response that has passed every check and may be handed to a decoder.
///
/// Passing them does not make it trusted. 0055 says so plainly: an image inside
/// both bounds is decoded under every other rule 0101 sets for an untrusted
/// parse.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Admitted {
    format: Accepted,
    dimensions: DeclaredDimensions,
}

impl Admitted {
    /// Which of the three the bytes are.
    #[must_use]
    pub const fn format(self) -> Accepted {
        self.format
    }

    /// What the header declared, already inside both bounds.
    #[must_use]
    pub const fn dimensions(self) -> DeclaredDimensions {
        self.dimensions
    }
}

/// Everything that runs before a decoder is reached.
///
/// The order is 0055's and it is not an implementation detail. The encoded length
/// is judged first, because it is the one bound that does not need the bytes to
/// mean anything. The signature is matched second, because a header is only read
/// once it is known which format's header it is. The declared dimensions are
/// judged last and still before any buffer exists for the pixels.
///
/// # Errors
///
/// [`Refused`], whose variants say which of the three checks refused.
pub fn admitted(bytes: &[u8]) -> Result<Admitted, Refused> {
    if !encoded_length_is_still_inside_its_bound(bytes.len()) {
        return Err(Refused::TheEncodedLengthPassedItsBound);
    }
    let format = Accepted::of(bytes).ok_or(Refused::TheSignatureMatchedNoAcceptedFormat)?;
    let dimensions =
        declared_dimensions(format, bytes).ok_or(Refused::TheHeaderDeclaredNoDimensions)?;
    if !dimensions.are_inside_their_bounds() {
        return Err(Refused::TheDeclaredDimensionsPassedTheirBound);
    }
    Ok(Admitted { format, dimensions })
}

/// The dimensions the matched format's own header declares.
///
/// `None` wherever the header cannot be read as one of that format's, which
/// includes a zero in either axis for the reason
/// [`Refused::TheHeaderDeclaredNoDimensions`] gives.
fn declared_dimensions(format: Accepted, bytes: &[u8]) -> Option<DeclaredDimensions> {
    let declared = match format {
        Accepted::Jpeg => jpeg_dimensions(bytes),
        Accepted::Png => png_dimensions(bytes),
        Accepted::WebP => webp_dimensions(bytes),
    }?;
    if declared.width() == 0 || declared.height() == 0 {
        return None;
    }
    Some(declared)
}

/// PNG declares its dimensions in the first chunk after the signature.
///
/// That chunk is required to be the header chunk, of a length the format fixes,
/// and this reader requires both rather than searching for the chunk by name.
/// Searching would be a walk over untrusted chunk lengths, which is a parser, to
/// find a chunk the format says is already there.
fn png_dimensions(bytes: &[u8]) -> Option<DeclaredDimensions> {
    if bytes.get(8..12)? != [0, 0, 0, 13] {
        return None;
    }
    if bytes.get(12..16)? != *b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    Some(DeclaredDimensions::of(width, height))
}

/// JPEG declares its dimensions in a start-of-frame segment, which is not at a
/// fixed offset.
///
/// The walk is the one parse in this module that has to be a loop, and it is
/// bounded in both directions: every read goes through a checked index, so the
/// end of the bytes ends the walk, and every pass advances by at least the two
/// bytes of the marker it just read, so no input makes it stand still.
///
/// Every spelling of a start-of-frame marker is a start of frame here. The
/// arithmetic-coded and the progressive families declare the same two numbers in
/// the same two places, and a reader that knew only the baseline one would walk
/// past the dimensions of a progressive image and answer that the header declared
/// none.
fn jpeg_dimensions(bytes: &[u8]) -> Option<DeclaredDimensions> {
    let mut at = 2;
    loop {
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        let mut marker = *bytes.get(at + 1)?;
        at += 2;
        // A run of 0xFF before a marker is padding the format allows.
        while marker == 0xFF {
            marker = *bytes.get(at)?;
            at += 1;
        }
        match marker {
            0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {
                let height = u16::from_be_bytes(bytes.get(at + 3..at + 5)?.try_into().ok()?);
                let width = u16::from_be_bytes(bytes.get(at + 5..at + 7)?.try_into().ok()?);
                return Some(DeclaredDimensions::of(u32::from(width), u32::from(height)));
            }
            // The markers that carry no length field at all.
            0x01 | 0xD0..=0xD9 => {}
            _ => {
                // A length field counts itself, so a well-formed one is at least
                // two. A LENGTH BELOW TWO IS NOT REFUSED HERE AND DOES NOT NEED
                // TO BE. The walk has already advanced past the marker before it
                // reads this field, so it cannot stand still whatever the field
                // says, and the byte a short length sends it back to is that
                // field's own high half, which is zero for anything below two and
                // therefore not the marker byte the next pass requires. A refusal
                // written here would be one nothing could be watched failing on,
                // which is what the first version of this line was.
                let length = u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?);
                at = at.checked_add(usize::from(length))?;
            }
        }
    }
}

/// WebP declares its dimensions in whichever of three chunks follows the
/// container tag, and the three encode them differently.
///
/// The chunk tag is read rather than guessed, and a tag that is none of the three
/// is a header this reader will not take dimensions out of. That is deliberately
/// not the same as refusing the container: it is one answer, and 0055's rule is
/// that a header nothing here can read is a header that declared nothing.
fn webp_dimensions(bytes: &[u8]) -> Option<DeclaredDimensions> {
    match bytes.get(12..16)? {
        // Lossy. The dimensions are fourteen bits each, inside a frame header
        // that begins with a fixed three-byte sequence.
        b"VP8 " => {
            if bytes.get(23..26)? != [0x9D, 0x01, 0x2A] {
                return None;
            }
            let width = u16::from_le_bytes(bytes.get(26..28)?.try_into().ok()?) & 0x3FFF;
            let height = u16::from_le_bytes(bytes.get(28..30)?.try_into().ok()?) & 0x3FFF;
            Some(DeclaredDimensions::of(u32::from(width), u32::from(height)))
        }
        // Lossless. Fourteen bits each again, packed one after the other, and
        // each stored one below the real number.
        b"VP8L" => {
            if *bytes.get(20)? != 0x2F {
                return None;
            }
            let packed = u32::from_le_bytes(bytes.get(21..25)?.try_into().ok()?);
            let width = (packed & 0x3FFF) + 1;
            let height = ((packed >> 14) & 0x3FFF) + 1;
            Some(DeclaredDimensions::of(width, height))
        }
        // Extended. Twenty-four bits each, little-endian, each stored one below
        // the real number, after a flags byte and three the format reserves.
        b"VP8X" => {
            let width = twenty_four_bits(bytes.get(24..27)?)? + 1;
            let height = twenty_four_bits(bytes.get(27..30)?)? + 1;
            Some(DeclaredDimensions::of(width, height))
        }
        _ => None,
    }
}

/// Three little-endian bytes as one number.
fn twenty_four_bits(three: &[u8]) -> Option<u32> {
    let [low, middle, high] = three.try_into().ok()?;
    Some(u32::from(low) | (u32::from(middle) << 8) | (u32::from(high) << 16))
}

#[cfg(test)]
mod tests {
    //! What these prove, and what they cannot.
    //!
    //! The bytes below are headers written by hand rather than images produced
    //! by an encoder. That is the whole subject: what runs before a decoder is
    //! reached reads a signature and a header and never the rest of a file, so a
    //! header and a real image are the same input to it. What no case here can
    //! say is whether a decoder would then succeed, because there is no decoder;
    //! that is #50, and #104 is where fixtures are held honest against a real
    //! server.
    //!
    //! Nothing here writes a file. The `no-filesystem-access` rule in
    //! `.github/invariants/rules` refuses a filesystem route anywhere under
    //! `src/`, and every input below is a slice of bytes in the source.

    use super::{
        AXIS_BOUND, Accepted, DeclaredDimensions, ENCODED_LENGTH_BOUND, PIXEL_COUNT_BOUND, Refused,
        admitted, encoded_length_is_still_inside_its_bound,
    };

    /// A PNG header declaring the two numbers it is given.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    /// A baseline JPEG header, with one segment before the frame so that the
    /// walk has something to step over rather than finding the answer first.
    fn jpeg(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xD8, 0xFF];
        // An application segment of four bytes, counting its own length field.
        bytes.extend_from_slice(&[0xE0, 0x00, 0x04, 0x00, 0x00]);
        bytes.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes
    }

    /// The same header with a progressive frame marker instead of a baseline
    /// one.
    fn progressive_jpeg(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = jpeg(width, height);
        bytes[9] = 0xC2;
        bytes
    }

    fn riff(chunk: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::from(*b"RIFF");
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"WEBP");
        bytes.extend_from_slice(chunk);
        bytes
    }

    /// A lossy WebP header. The two numbers are fourteen bits each.
    fn lossy_webp(width: u16, height: u16) -> Vec<u8> {
        let mut chunk = Vec::from(*b"VP8 ");
        chunk.extend_from_slice(&[0, 0, 0, 0]);
        chunk.extend_from_slice(&[0x00, 0x00, 0x00]);
        chunk.extend_from_slice(&[0x9D, 0x01, 0x2A]);
        chunk.extend_from_slice(&width.to_le_bytes());
        chunk.extend_from_slice(&height.to_le_bytes());
        riff(&chunk)
    }

    /// A lossless WebP header. Each number is stored one below the real one.
    fn lossless_webp(width: u32, height: u32) -> Vec<u8> {
        let packed = (width - 1) | ((height - 1) << 14);
        let mut chunk = Vec::from(*b"VP8L");
        chunk.extend_from_slice(&[0, 0, 0, 0]);
        chunk.push(0x2F);
        chunk.extend_from_slice(&packed.to_le_bytes());
        riff(&chunk)
    }

    /// An extended WebP header. Twenty-four bits each, one below the real one.
    fn extended_webp(width: u32, height: u32) -> Vec<u8> {
        let mut chunk = Vec::from(*b"VP8X");
        chunk.extend_from_slice(&[0, 0, 0, 0]);
        chunk.push(0x00);
        chunk.extend_from_slice(&[0, 0, 0]);
        chunk.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
        chunk.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
        riff(&chunk)
    }

    #[test]
    fn a_jpeg_is_admitted_with_what_its_frame_declared() {
        let it = admitted(&jpeg(640, 480)).expect("a jpeg header");
        assert_eq!(it.format(), Accepted::Jpeg);
        assert_eq!(it.dimensions(), DeclaredDimensions::of(640, 480));
    }

    /// A reader that knew only the baseline marker would walk past this frame
    /// and answer that the header declared nothing.
    #[test]
    fn a_progressive_jpeg_declares_its_dimensions_in_the_same_place() {
        let it = admitted(&progressive_jpeg(640, 480)).expect("a progressive jpeg header");
        assert_eq!(it.dimensions(), DeclaredDimensions::of(640, 480));
    }

    #[test]
    fn a_png_is_admitted_with_what_its_header_chunk_declared() {
        let it = admitted(&png(1920, 1080)).expect("a png header");
        assert_eq!(it.format(), Accepted::Png);
        assert_eq!(it.dimensions(), DeclaredDimensions::of(1920, 1080));
    }

    #[test]
    fn a_lossy_webp_is_admitted_with_what_its_frame_declared() {
        let it = admitted(&lossy_webp(300, 450)).expect("a lossy webp header");
        assert_eq!(it.format(), Accepted::WebP);
        assert_eq!(it.dimensions(), DeclaredDimensions::of(300, 450));
    }

    #[test]
    fn a_lossless_webp_is_admitted_with_what_its_header_declared() {
        let it = admitted(&lossless_webp(300, 450)).expect("a lossless webp header");
        assert_eq!(it.format(), Accepted::WebP);
        assert_eq!(it.dimensions(), DeclaredDimensions::of(300, 450));
    }

    #[test]
    fn an_extended_webp_is_admitted_with_what_its_header_declared() {
        let it = admitted(&extended_webp(300, 450)).expect("an extended webp header");
        assert_eq!(it.format(), Accepted::WebP);
        assert_eq!(it.dimensions(), DeclaredDimensions::of(300, 450));
    }

    /// The formats 0055 names a reason for, and one it does not. All of them
    /// arrive at the same refusal, which is the record's rule: the accepted set
    /// is what decides, and the named refusals are the ones worth their reasons
    /// rather than the whole of what is refused.
    #[test]
    fn every_format_outside_the_set_is_refused_before_a_decoder() {
        let outside: [(&str, &[u8]); 8] = [
            ("gif", b"GIF89a\x01\x00\x01\x00\x00\x00\x00"),
            ("bmp", b"BM\x36\x00\x00\x00\x00\x00\x00\x00\x36\x00"),
            ("tiff", b"II\x2A\x00\x08\x00\x00\x00\x00\x00\x00\x00"),
            ("ico", b"\x00\x00\x01\x00\x01\x00\x10\x10\x00\x00\x01\x00"),
            ("svg", b"<svg xmlns=\"h"),
            ("avif", b"\x00\x00\x00\x1cftypavif\x00\x00\x00\x00"),
            ("heic", b"\x00\x00\x00\x1cftypheic\x00\x00\x00\x00"),
            (
                "nothing anybody named",
                b"\x7FELF\x02\x01\x01\x00\x00\x00\x00\x00",
            ),
        ];
        for (what, bytes) in outside {
            assert_eq!(
                admitted(bytes),
                Err(Refused::TheSignatureMatchedNoAcceptedFormat),
                "{what} reached further than the signature match"
            );
            assert_eq!(Accepted::of(bytes), None, "{what} matched a signature");
        }
    }

    /// The one-byte neighbour of the JPEG signature, which is the mistake a
    /// prefix comparison written too short would make.
    #[test]
    fn a_prefix_one_byte_short_of_a_signature_matches_nothing() {
        assert_eq!(Accepted::of(&[0xFF, 0xD8]), None);
        assert_eq!(
            Accepted::of(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A]),
            None
        );
    }

    /// A container tag with something else inside it. WebP is the one signature
    /// of the three that is not contiguous, so a match on its first four bytes
    /// alone would admit every other thing carried in the same container.
    #[test]
    fn a_container_that_is_not_webp_matches_nothing() {
        let mut wave = Vec::from(*b"RIFF");
        wave.extend_from_slice(&[0, 0, 0, 0]);
        wave.extend_from_slice(b"WAVE");
        assert_eq!(Accepted::of(&wave), None);
        assert_eq!(
            admitted(&wave),
            Err(Refused::TheSignatureMatchedNoAcceptedFormat)
        );
    }

    /// 0055's own sentence: the bytes win. There is no argument to [`admitted`]
    /// for what a server declared, so a declaration cannot reach this decision
    /// at all, and what a case can assert is that the bytes decide.
    #[test]
    fn the_bytes_decide_what_a_thing_is_and_not_what_a_server_declared() {
        let served_as_a_jpeg_but_a_png = png(16, 16);
        let it = admitted(&served_as_a_jpeg_but_a_png).expect("a png header");
        assert_eq!(it.format(), Accepted::Png);
        assert_ne!(it.format(), Accepted::Jpeg);
    }

    #[test]
    fn dimensions_past_the_axis_bound_are_refused() {
        assert_eq!(
            admitted(&png(AXIS_BOUND + 1, 1)),
            Err(Refused::TheDeclaredDimensionsPassedTheirBound)
        );
        assert_eq!(
            admitted(&png(1, AXIS_BOUND + 1)),
            Err(Refused::TheDeclaredDimensionsPassedTheirBound)
        );
    }

    /// The image an axis bound alone admits: inside both axes and enormous in
    /// area.
    #[test]
    fn dimensions_inside_both_axes_and_past_the_total_are_refused() {
        let inside_both_axes = png(AXIS_BOUND, AXIS_BOUND);
        assert_eq!(
            DeclaredDimensions::of(AXIS_BOUND, AXIS_BOUND).pixel_count(),
            67_108_864
        );
        assert!(DeclaredDimensions::of(AXIS_BOUND, AXIS_BOUND).pixel_count() > PIXEL_COUNT_BOUND);
        assert_eq!(
            admitted(&inside_both_axes),
            Err(Refused::TheDeclaredDimensionsPassedTheirBound)
        );
    }

    /// The image a total alone admits: a single row long enough to overflow an
    /// index somewhere downstream.
    #[test]
    fn a_single_row_past_the_axis_bound_is_refused_although_its_total_is_small() {
        let one_row = png(100_000, 1);
        assert!(DeclaredDimensions::of(100_000, 1).pixel_count() < PIXEL_COUNT_BOUND);
        assert_eq!(
            admitted(&one_row),
            Err(Refused::TheDeclaredDimensionsPassedTheirBound)
        );
    }

    /// The two largest numbers a header can declare multiply to more than the
    /// type either is held in. Widened after the multiplication rather than
    /// before it, the product wraps to a small number and passes the bound.
    #[test]
    fn a_product_that_would_wrap_is_still_past_the_bound() {
        let enormous = DeclaredDimensions::of(u32::MAX, u32::MAX);
        assert_eq!(enormous.pixel_count(), 18_446_744_065_119_617_025);
        assert!(!enormous.are_inside_their_bounds());
    }

    #[test]
    fn dimensions_exactly_at_both_bounds_are_admitted() {
        let at_the_axis = DeclaredDimensions::of(AXIS_BOUND, 1);
        assert!(at_the_axis.are_inside_their_bounds());
        let at_the_total = DeclaredDimensions::of(4_000, 4_000);
        assert_eq!(at_the_total.pixel_count(), PIXEL_COUNT_BOUND);
        assert!(at_the_total.are_inside_their_bounds());
        assert!(admitted(&png(4_000, 4_000)).is_ok());
    }

    #[test]
    fn a_header_declaring_a_zero_axis_declares_no_dimensions() {
        assert_eq!(
            admitted(&png(0, 16)),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
        assert_eq!(
            admitted(&png(16, 0)),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    #[test]
    fn a_signature_with_no_header_behind_it_declares_no_dimensions() {
        let signature_only = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(
            admitted(&signature_only),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
        assert_eq!(
            admitted(&[0xFF, 0xD8, 0xFF]),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    /// A PNG whose first chunk is not the one the format requires there. The
    /// reader requires it rather than searching for it, so this is a header it
    /// will not take dimensions out of.
    #[test]
    fn a_png_whose_first_chunk_is_not_the_header_chunk_declares_no_dimensions() {
        let mut not_the_header_chunk = png(16, 16);
        not_the_header_chunk[12] = b'X';
        assert_eq!(
            admitted(&not_the_header_chunk),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    /// A segment declaring a length its own field cannot mean. This is a case
    /// rather than a guard proof: the walk ends on it with or without a refusal
    /// written for it, which is why no refusal is written for it.
    #[test]
    fn a_jpeg_segment_claiming_no_length_ends_the_walk() {
        let standing_still: [u8; 8] = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x00, 0xFF, 0xC0];
        assert_eq!(
            admitted(&standing_still),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    /// A segment length long enough to run past everything given. The walk ends
    /// at the end of the bytes rather than reading anything it was not handed.
    #[test]
    fn a_jpeg_segment_claiming_more_than_it_was_given_ends_the_walk() {
        let past_the_end: [u8; 8] = [0xFF, 0xD8, 0xFF, 0xE0, 0xFF, 0xFF, 0x00, 0x00];
        assert_eq!(
            admitted(&past_the_end),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    #[test]
    fn a_webp_chunk_none_of_the_three_declares_no_dimensions() {
        let mut chunk = Vec::from(*b"ANIM");
        chunk.extend_from_slice(&[0; 16]);
        assert_eq!(
            admitted(&riff(&chunk)),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    /// The fixed sequence inside a lossy frame header, one byte away from
    /// itself. Without the check, whatever two numbers follow are read as
    /// dimensions.
    #[test]
    fn a_lossy_webp_frame_without_its_fixed_sequence_declares_no_dimensions() {
        let mut broken = lossy_webp(300, 450);
        broken[23] = 0x9C;
        assert_eq!(
            admitted(&broken),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    #[test]
    fn a_lossless_webp_without_its_signature_byte_declares_no_dimensions() {
        let mut broken = lossless_webp(300, 450);
        broken[20] = 0x2E;
        assert_eq!(
            admitted(&broken),
            Err(Refused::TheHeaderDeclaredNoDimensions)
        );
    }

    #[test]
    fn more_encoded_bytes_than_the_bound_admits_are_refused() {
        let past_the_bound = vec![0; ENCODED_LENGTH_BOUND + 1];
        assert_eq!(
            admitted(&past_the_bound),
            Err(Refused::TheEncodedLengthPassedItsBound)
        );
    }

    /// The encoded-length bound is judged before the signature is matched, so a
    /// response too long to hold is refused for its length rather than for what
    /// it turns out to be.
    #[test]
    fn a_valid_image_past_the_encoded_length_bound_is_refused_for_its_length() {
        let mut too_long = png(16, 16);
        too_long.resize(ENCODED_LENGTH_BOUND + 1, 0);
        assert_eq!(
            admitted(&too_long),
            Err(Refused::TheEncodedLengthPassedItsBound)
        );
    }

    /// The bound in the form a transfer asks it, which is a running count rather
    /// than a finished response.
    #[test]
    fn a_transfer_is_refused_at_the_byte_that_passes_the_bound() {
        assert!(encoded_length_is_still_inside_its_bound(0));
        assert!(encoded_length_is_still_inside_its_bound(
            ENCODED_LENGTH_BOUND
        ));
        assert!(!encoded_length_is_still_inside_its_bound(
            ENCODED_LENGTH_BOUND + 1
        ));
    }

    #[test]
    fn nothing_at_all_matches_no_signature() {
        assert_eq!(
            admitted(&[]),
            Err(Refused::TheSignatureMatchedNoAcceptedFormat)
        );
    }
}
