//! The address an artwork request is made at, and the size it asks for.
//!
//! `docs/decisions/0049-the-artwork-address-and-the-size-asked-for.md` is the
//! record. It fixes five things that are each easy to get wrong quietly, and all
//! five are properties of the types here rather than conditions a caller keeps:
//! which image kinds exist, that a size is required, that a size is rounded onto
//! a fixed ladder before it reaches a request, that the content tag 0006 depends
//! on is required, and that an identifier or a tag a server sent is refused on
//! its bytes before it is written into a request.
//!
//! # What is here and what is not
//!
//! Everything up to the request. [`ArtworkRequest`] holds the path and the three
//! parameters, hands out the address a transport would fetch, and derives the
//! cache key 0041 builds. Nothing here fetches anything: the transport is #27
//! and does not exist, so no call in this module reaches a network.
//!
//! What an item with no image of a kind answers with is #51 and is deliberately
//! absent. The rule here is narrower and is the half that belongs to addressing:
//! a tag is a required input, so an item carrying none for a kind produces no
//! address at all rather than an address with an empty tag in it.

use crate::cache::EntryKey;
use crate::cache::key::{KeySpace, Parameter, RequestKey};
use crate::server::address::BaseAddress;

/// The five image kinds the core builds an address for.
///
/// 0049 fixes the set and names each one. It is closed the way
/// [`crate::server::address::Scheme`] and the accepted formats in
/// [`crate::artwork::format`] are closed: a kind outside it cannot be
/// constructed, so there is no call site at which one could be caught.
///
/// Which kinds a given server actually holds is a claim in 0049 rather than a
/// measurement. Nothing in this tree contacts a server, and #104 is the route by
/// which that claim becomes measured or is contradicted.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageKind {
    /// The cover a tile wall is made of, and the one 0050's arithmetic is
    /// written against.
    Primary,
    /// The full-bleed still behind a detail screen, and the largest of the five.
    Backdrop,
    /// The wide still an episode row is drawn from.
    Thumb,
    /// The title treatment drawn over a backdrop, which is usually the one with
    /// transparency.
    Logo,
    /// The wide strip a series row uses where a client draws one.
    Banner,
}

impl ImageKind {
    /// Every kind, for a caller that walks the set rather than naming members.
    ///
    /// It is here so that a test over "every image kind" is a test over the set
    /// the type declares rather than over a list somebody typed beside it, which
    /// is the list that stops matching when a sixth kind is added.
    pub const ALL: [Self; 5] = [
        Self::Primary,
        Self::Backdrop,
        Self::Thumb,
        Self::Logo,
        Self::Banner,
    ];

    /// The kind as it is written into the path 0010 fixes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Backdrop => "Backdrop",
            Self::Thumb => "Thumb",
            Self::Logo => "Logo",
            Self::Banner => "Banner",
        }
    }
}

/// The ladder a requested edge is rounded up onto.
///
/// 0049 fixes these fifteen values and the reason two of them are what they are:
/// 0050 takes a poster at three hundred by four hundred and fifty, so both of
/// those numbers are rungs and the record's arithmetic lands exactly rather than
/// nearly.
///
/// It is public because a client sizing a tile can land on a rung deliberately
/// instead of being rounded onto one, and because a reader checking 0050's
/// arithmetic should not have to read the source to find the set.
pub const LADDER: [u32; 15] = [
    90, 120, 180, 240, 300, 360, 450, 600, 720, 900, 1080, 1440, 1920, 2560, 3840,
];

/// Which edge of a requested size could not be used.
///
/// The set is closed and exhaustive for 0004's reason: a caller matching on it
/// is told by the compiler when a case appears rather than falling into a branch
/// somebody wrote for something else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// The width.
    Width,
    /// The height.
    Height,
}

/// Why a requested size could not be turned into one the core asks for.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeNotUsable {
    /// An edge of zero, which is not a size anything is drawn at.
    EdgeIsZero(Edge),
    /// An edge above the top of [`LADDER`].
    ///
    /// 0049 refuses rather than clamping. Clamping would answer a caller that
    /// asked for more than the ladder holds with a smaller image and no
    /// indication, which is the accidental full-resolution fetch turned inside
    /// out: the caller believes it asked for the size that will be drawn and did
    /// not get it.
    EdgeAboveTheLadder(Edge),
}

/// The size the core asks a server for, already on a rung of [`LADDER`].
///
/// There is no way to build one that is not on a rung, and no way to build one
/// at all without both edges. That is the whole of 0049's "size as a required
/// input rather than an optional one": a caller cannot accidentally request the
/// original, because no value of this type means the original.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrawnSize {
    width: u32,
    height: u32,
}

impl DrawnSize {
    /// Rounds the size a caller will draw at up onto the ladder.
    ///
    /// Each edge independently, to the smallest rung that is not below it. Two
    /// sizes that differ by a few pixels land on one rung and therefore on one
    /// address and one cache entry, which is what this issue exists for. Two
    /// that straddle a rung do not, and 0049 states that bound rather than
    /// leaving a reader to discover it.
    ///
    /// # Errors
    ///
    /// [`SizeNotUsable`] where an edge is zero or above the top of the ladder,
    /// naming which edge it was.
    pub const fn asked_for(width: u32, height: u32) -> Result<Self, SizeNotUsable> {
        let rounded_width = match rung(width, Edge::Width) {
            Ok(rounded) => rounded,
            Err(refused) => return Err(refused),
        };
        let rounded_height = match rung(height, Edge::Height) {
            Ok(rounded) => rounded,
            Err(refused) => return Err(refused),
        };
        Ok(Self {
            width: rounded_width,
            height: rounded_height,
        })
    }

    /// The width that will be asked for, which is a rung.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// The height that will be asked for, which is a rung.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

/// The smallest rung not below an edge.
const fn rung(edge: u32, which: Edge) -> Result<u32, SizeNotUsable> {
    if edge == 0 {
        return Err(SizeNotUsable::EdgeIsZero(which));
    }
    let mut index = 0;
    while index < LADDER.len() {
        if LADDER[index] >= edge {
            return Ok(LADDER[index]);
        }
        index += 1;
    }
    Err(SizeNotUsable::EdgeAboveTheLadder(which))
}

/// Why a value a server sent may not be written into a request.
///
/// THE POSITION AND NOT THE BYTE, WHICH IS THE ONE DECISION IN THIS TYPE. 0068
/// places an item identifier in the personal data list, so a refusal carrying
/// the offending character would carry a piece of the value into wherever the
/// refusal is reported, which is exactly what 0071 applies a treatment to one
/// layer further on. The position says as much as anybody debugging needs and is
/// not about a person.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotUsableInARequest {
    /// Nothing was there.
    Empty,
    /// A byte outside the admitted set, at this zero-based position.
    ByteAt(usize),
}

/// The identifier of one item, as a server gave it back.
///
/// 0049 refuses this on its bytes before it is written into the path 0010 fixes,
/// because 0101 treats every byte from a server as untrusted and this one
/// chooses part of a request the core is about to send.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId {
    value: String,
}

impl ItemId {
    /// Takes an identifier a server sent, refusing it on its bytes.
    ///
    /// # Errors
    ///
    /// [`NotUsableInARequest`] where it is empty or carries a byte outside the
    /// set 0049 admits, which is an ASCII letter, digit, hyphen or underscore.
    pub fn from_server(value: &str) -> Result<Self, NotUsableInARequest> {
        admitted(value)?;
        Ok(Self {
            value: value.to_owned(),
        })
    }

    /// The identifier, as it is written into a path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// The content tag an item's metadata carried for one image kind.
///
/// 0006 makes artwork the one cached kind that is never revalidated against the
/// server, on the ground that a changed image is a different address rather than
/// a stale entry. This is the value that makes that true, so it is required
/// rather than optional, and it is refused on its bytes for the same reason
/// [`ItemId`] is.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageTag {
    value: String,
}

impl ImageTag {
    /// Takes a tag a server sent, refusing it on its bytes.
    ///
    /// # Errors
    ///
    /// [`NotUsableInARequest`] where it is empty or carries a byte outside the
    /// set 0049 admits.
    pub fn from_server(value: &str) -> Result<Self, NotUsableInARequest> {
        admitted(value)?;
        Ok(Self {
            value: value.to_owned(),
        })
    }

    /// The tag, as it is written into a parameter.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Refuses a value a server sent that does not consist entirely of the bytes
/// 0049 admits.
///
/// The set is narrower than what a URL permits, which 0049 argues is the right
/// yardstick: it is judged against what these two fields are rather than against
/// what a path can hold.
fn admitted(value: &str) -> Result<(), NotUsableInARequest> {
    if value.is_empty() {
        return Err(NotUsableInARequest::Empty);
    }
    for (position, byte) in value.bytes().enumerate() {
        if !(byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') {
            return Err(NotUsableInARequest::ByteAt(position));
        }
    }
    Ok(())
}

/// The names of the three parameters, in the order they are written.
///
/// Sorted by name, which is the order `crate::cache::key` sorts them into before
/// it writes them into a key. Writing the address in the same order means there
/// is one order in this module rather than two that have to be kept in step.
const MAX_HEIGHT: &str = "maxHeight";
/// The maximum width parameter.
const MAX_WIDTH: &str = "maxWidth";
/// The content tag parameter.
const TAG: &str = "tag";

/// One artwork request: the path 0010 fixes and the three parameters 0049 puts
/// on it.
///
/// It owns its strings so that it can hand out both an address and a
/// [`RequestKey`], which borrows. Building it performs no allocation a caller
/// could avoid: the path and the two numbers have to become text to be written
/// into either one.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkRequest {
    path: String,
    tag: String,
    width: String,
    height: String,
}

impl ArtworkRequest {
    /// Builds the request for one image of one item at one size.
    ///
    /// Every input is required, which is 0049's decision expressed as a
    /// signature: there is no argument here that can be left out to mean the
    /// original, and no tag that can be absent to mean an unversioned address.
    #[must_use]
    pub fn for_item(item: &ItemId, kind: ImageKind, tag: &ImageTag, size: DrawnSize) -> Self {
        let mut path = String::from("/Items/");
        path.push_str(item.as_str());
        path.push_str("/Images/");
        path.push_str(kind.as_str());
        Self {
            path,
            tag: tag.as_str().to_owned(),
            width: size.width().to_string(),
            height: size.height().to_string(),
        }
    }

    /// The path this request is made at, which is 0010's template filled in.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The three parameters, sorted by name.
    ///
    /// Each is [`Some`], because all three are required. The type still carries
    /// the option because 0041 requires an absent parameter and a present empty
    /// one to be written differently, and that distinction belongs to the key
    /// rather than to this endpoint.
    #[must_use]
    pub fn parameters(&self) -> [Parameter<'_>; 3] {
        [
            (MAX_HEIGHT, Some(self.height.as_str())),
            (MAX_WIDTH, Some(self.width.as_str())),
            (TAG, Some(self.tag.as_str())),
        ]
    }

    /// The address a transport would fetch, joined onto a base address.
    ///
    /// The join is [`BaseAddress::join`] rather than a URL type's reference
    /// resolution, for the reason 0028 gives and `crate::server::address` holds:
    /// resolution drops an operator's sub-path and every request answers 404.
    ///
    /// No escaping is applied to anything written here, and that is 0049's
    /// decision rather than an omission. The two values that came from a server
    /// were refused on their bytes on the way into [`ItemId`] and [`ImageTag`],
    /// and the other two are numbers this module produced.
    #[must_use]
    pub fn address(&self, base: &BaseAddress) -> String {
        let mut address = base.join(&self.path);
        let mut separator = '?';
        for (name, value) in self.parameters() {
            if let Some(value) = value {
                address.push(separator);
                address.push_str(name);
                address.push('=');
                address.push_str(value);
                separator = '&';
            }
        }
        address
    }

    /// The cache entry this request's answer is kept under.
    ///
    /// 0053 coalesces announced tiles on this key rather than on the address,
    /// which is what makes that record survive whatever this one decides: two
    /// nearby sizes that round onto one rung are one key here, so they are one
    /// fetch there without 0053 needing a rule of its own.
    #[must_use]
    pub fn entry_key(&self, space: &KeySpace<'_>) -> EntryKey {
        let parameters = self.parameters();
        EntryKey::derive(
            space,
            &RequestKey {
                endpoint: &self.path,
                parameters: &parameters,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    //! What each test is for, where it is not obvious from its name.
    //!
    //! The table-driven case over every kind and several sizes is #49's first
    //! condition. It walks [`ImageKind::ALL`] rather than a list written beside
    //! it, so a sixth kind added without a case here fails to compile the
    //! expectation table rather than passing silently.
    //!
    //! The pair of nearby sizes resolving to one entry key is #49's second
    //! condition, and it is asserted on the key rather than on the address
    //! because the key is what 0053 coalesces on and what 0042 counts.

    use super::{
        ArtworkRequest, DrawnSize, Edge, ImageKind, ImageTag, ItemId, LADDER, NotUsableInARequest,
        SizeNotUsable,
    };
    use crate::cache::key::{KeySpace, ServerPart};
    use crate::server::address::BaseAddress;

    fn base() -> BaseAddress {
        BaseAddress::parse("https://films.example/jellyfin").expect("the address is usable")
    }

    fn item() -> ItemId {
        ItemId::from_server("a1b2c3d4e5f6").expect("the identifier is usable")
    }

    fn tag() -> ImageTag {
        ImageTag::from_server("9f2c4d").expect("the tag is usable")
    }

    fn space<'a>() -> KeySpace<'a> {
        KeySpace {
            server: ServerPart::Reported("server-7a1b"),
            account: "account-3c9d",
            device: "device-9f2c",
        }
    }

    fn poster() -> DrawnSize {
        DrawnSize::asked_for(300, 450).expect("the poster rung is on the ladder")
    }

    /// #49's first condition. Every kind, at three sizes, against the address
    /// each one produces.
    #[test]
    fn every_kind_at_several_sizes_produces_the_address_0010_fixes() {
        let base = base();
        let item = item();
        let tag = tag();

        let sizes = [(300, 450), (180, 180), (1920, 1080)];
        let expected_per_size = [("450", "300"), ("180", "180"), ("1080", "1920")];

        for (asked, (height, width)) in sizes.into_iter().zip(expected_per_size) {
            let size = DrawnSize::asked_for(asked.0, asked.1).expect("the size is on the ladder");
            for kind in ImageKind::ALL {
                let address = ArtworkRequest::for_item(&item, kind, &tag, size).address(&base);
                let expected = format!(
                    "https://films.example/jellyfin/Items/a1b2c3d4e5f6/Images/{}\
                     ?maxHeight={height}&maxWidth={width}&tag=9f2c4d",
                    kind.as_str()
                );
                assert_eq!(address, expected, "kind {kind:?} at {asked:?}");
            }
        }
    }

    /// The five kinds are five distinct addresses, so a table above that agreed
    /// with itself for the wrong reason is caught.
    #[test]
    fn the_five_kinds_are_five_different_addresses() {
        let base = base();
        let item = item();
        let tag = tag();
        let mut seen = Vec::new();
        for kind in ImageKind::ALL {
            seen.push(ArtworkRequest::for_item(&item, kind, &tag, poster()).address(&base));
        }
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), ImageKind::ALL.len());
    }

    /// #49's second condition, on the key rather than on the address.
    #[test]
    fn two_nearby_sizes_resolve_to_one_cache_entry() {
        let item = item();
        let tag = tag();
        let space = space();

        let asked = DrawnSize::asked_for(298, 447).expect("the size is on the ladder");
        let also_asked = DrawnSize::asked_for(300, 450).expect("the size is on the ladder");

        let first = ArtworkRequest::for_item(&item, ImageKind::Primary, &tag, asked);
        let second = ArtworkRequest::for_item(&item, ImageKind::Primary, &tag, also_asked);

        assert_eq!(
            first.entry_key(&space).as_str(),
            second.entry_key(&space).as_str()
        );
    }

    /// The bound 0049 states on the sentence above it, asserted so that nobody
    /// reads the previous test as a promise the rounding cannot keep.
    #[test]
    fn two_sizes_straddling_a_rung_are_two_cache_entries() {
        let item = item();
        let tag = tag();
        let space = space();

        let below = DrawnSize::asked_for(300, 450).expect("the size is on the ladder");
        let above = DrawnSize::asked_for(301, 450).expect("the size is on the ladder");
        assert_ne!(below.width(), above.width());

        let first = ArtworkRequest::for_item(&item, ImageKind::Primary, &tag, below);
        let second = ArtworkRequest::for_item(&item, ImageKind::Primary, &tag, above);

        assert_ne!(
            first.entry_key(&space).as_str(),
            second.entry_key(&space).as_str()
        );
    }

    /// 0006's reason for artwork never being revalidated: a changed image is a
    /// different key. Without the tag in the request part this assertion fails.
    #[test]
    fn a_changed_tag_is_a_different_cache_entry() {
        let item = item();
        let space = space();
        let first_tag = tag();
        let second_tag = ImageTag::from_server("0e5a11").expect("the tag is usable");

        let first = ArtworkRequest::for_item(&item, ImageKind::Primary, &first_tag, poster());
        let second = ArtworkRequest::for_item(&item, ImageKind::Primary, &second_tag, poster());

        assert_ne!(
            first.entry_key(&space).as_str(),
            second.entry_key(&space).as_str()
        );
    }

    /// Two kinds of one item are two entries, so a wall of tiles cannot show a
    /// backdrop where a poster belongs.
    #[test]
    fn two_kinds_of_one_item_are_two_cache_entries() {
        let item = item();
        let tag = tag();
        let space = space();

        let primary = ArtworkRequest::for_item(&item, ImageKind::Primary, &tag, poster());
        let backdrop = ArtworkRequest::for_item(&item, ImageKind::Backdrop, &tag, poster());

        assert_ne!(
            primary.entry_key(&space).as_str(),
            backdrop.entry_key(&space).as_str()
        );
    }

    #[test]
    fn every_rung_rounds_to_itself() {
        for rung in LADDER {
            let size = DrawnSize::asked_for(rung, rung).expect("a rung is on the ladder");
            assert_eq!((size.width(), size.height()), (rung, rung));
        }
    }

    #[test]
    fn an_edge_one_above_a_rung_takes_the_next_one() {
        for pair in LADDER.windows(2) {
            let (lower, upper) = (pair[0], pair[1]);
            let size = DrawnSize::asked_for(lower + 1, 90).expect("the size is on the ladder");
            assert_eq!(size.width(), upper, "one above {lower}");
        }
    }

    #[test]
    fn rounding_goes_up_and_never_down() {
        for asked in 1..=3840_u32 {
            let size = DrawnSize::asked_for(asked, 90).expect("the size is on the ladder");
            assert!(size.width() >= asked, "asked {asked}, got {}", size.width());
        }
    }

    #[test]
    fn the_poster_0050_computes_with_is_a_rung() {
        let size = DrawnSize::asked_for(300, 450).expect("the size is on the ladder");
        assert_eq!((size.width(), size.height()), (300, 450));
    }

    #[test]
    fn an_edge_of_zero_is_refused_and_says_which_edge() {
        assert_eq!(
            DrawnSize::asked_for(0, 450),
            Err(SizeNotUsable::EdgeIsZero(Edge::Width))
        );
        assert_eq!(
            DrawnSize::asked_for(300, 0),
            Err(SizeNotUsable::EdgeIsZero(Edge::Height))
        );
    }

    #[test]
    fn an_edge_above_the_ladder_is_refused_rather_than_clamped() {
        assert_eq!(
            DrawnSize::asked_for(3841, 450),
            Err(SizeNotUsable::EdgeAboveTheLadder(Edge::Width))
        );
        assert_eq!(
            DrawnSize::asked_for(300, 3841),
            Err(SizeNotUsable::EdgeAboveTheLadder(Edge::Height))
        );
    }

    /// The refusal 0101 asks for. Each of these is a server steering a request
    /// the core is about to send.
    #[test]
    fn an_identifier_that_would_choose_another_path_is_refused() {
        for steering in [
            "../Users/Me",
            "a1b2/../../Sessions",
            "a1b2?maxWidth=4000",
            "a1b2#fragment",
            "a1b2%2f",
            "a1b2 c3",
        ] {
            assert!(
                ItemId::from_server(steering).is_err(),
                "admitted {steering:?}"
            );
        }
    }

    #[test]
    fn a_tag_that_would_add_a_parameter_is_refused() {
        for steering in ["9f2c&maxWidth=4000", "9f2c=1", "9f2c#x", "9f2c/../"] {
            assert!(
                ImageTag::from_server(steering).is_err(),
                "admitted {steering:?}"
            );
        }
    }

    #[test]
    fn an_empty_identifier_and_an_empty_tag_are_refused() {
        assert_eq!(ItemId::from_server(""), Err(NotUsableInARequest::Empty));
        assert_eq!(ImageTag::from_server(""), Err(NotUsableInARequest::Empty));
    }

    /// The position rather than the byte, which is the decision
    /// [`NotUsableInARequest`] carries.
    #[test]
    fn a_refusal_names_the_position_and_carries_no_part_of_the_value() {
        assert_eq!(
            ItemId::from_server("a1b2/c3"),
            Err(NotUsableInARequest::ByteAt(4))
        );
    }

    #[test]
    fn the_ordinary_shapes_a_server_sends_are_admitted() {
        for ordinary in [
            "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
            "a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
            "item_9",
            "9",
        ] {
            assert!(
                ItemId::from_server(ordinary).is_ok(),
                "refused {ordinary:?}"
            );
        }
    }

    /// A sub-path an operator put a server behind survives, which is 0028's
    /// reason for [`BaseAddress::join`] existing at all.
    #[test]
    fn a_base_address_carrying_a_sub_path_keeps_it() {
        let address = ArtworkRequest::for_item(&item(), ImageKind::Primary, &tag(), poster())
            .address(&base());
        assert!(
            address.starts_with("https://films.example/jellyfin/Items/"),
            "{address}"
        );
    }
}
