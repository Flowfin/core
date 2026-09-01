//! The shape an image has before its bytes, and the rectangle a client reserves.
//!
//! `docs/decisions/0052-the-shape-reserved-before-the-bytes.md` is the record and
//! #52 is the issue. The record decides four things: that the shape is an aspect
//! ratio and never a pair of pixel dimensions, that it arrives as the decimal
//! text an answer carried rather than as a number something else already made of
//! it, that both supported server lines state one for [`ImageKind::Primary`] and
//! for no other kind, and what a client reserves in each of the answers.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that a byte string and two whole-number
//! divisions settle: which stated ratios are usable, which are refused and under
//! which name, what an unstated kind answers, and the rectangle each answer
//! reserves inside a box [`DrawnSize`] already fixed. None of it reads a clock, a
//! socket or a store.
//!
//! WHAT IS NOT HERE IS THE READ. The ratio is a field of an item's metadata,
//! getting that metadata is a request, and the transport is #27, so nothing in
//! this module sends or receives a byte. #52's condition that the answer be
//! available from a cached item with no network call is untouched by everything
//! below, and #52 is where that is written against the issue rather than only
//! here.
//!
//! Neither is the ask. 0052 says the core names the ratio field on every read
//! whose answers a client may reserve space for, because a read that does not
//! name it is answered with the field unset and that is indistinguishable here
//! from an item with no primary image. Naming a field is part of building a
//! query, which is #39, and nothing in this module builds one.
//!
//! # Why the seam is text
//!
//! The value on the wire is a decimal number inside an answer body, and turning
//! one into a machine number decides how many digits are kept, what an exponent
//! does and what a magnitude no image has does. 0052 keeps that decision here
//! rather than in whatever decodes the answer, on the same shape
//! [`super::address::ImageTag`] already takes for a value a server sent: the
//! bytes as they arrived, and the rules in this module are what admit them.
//!
//! It also makes every refusal below provable with no decoder, no socket and no
//! server, which is what lets these rules land while #27 is open.
//!
//! # Where the bound comes from
//!
//! [`super::address::LADDER`] and nothing else. A box is built from two rungs, so
//! the widest shape any box can have is the top rung over the bottom one and the
//! narrowest is the bottom over the top. Both numbers are computed from that
//! array here, so a ladder that changes carries them with it rather than leaving
//! a second copy to be found later.

use super::address::{DrawnSize, ImageKind, LADDER};

/// The unit a ratio is kept in: ten-thousandths of a width per unit of height.
///
/// Whole numbers rather than a floating point value, so the arithmetic that
/// reserves a rectangle has no rounding anybody has to reason about. Four
/// fraction digits is finer than a rung can express: at the top rung of the
/// ladder the fifth digit moves an edge by less than half a pixel.
const SCALE: u32 = 10_000;

/// How many digits after the point are read. The rest are consumed and dropped.
const FRACTION_DIGITS_READ: usize = 4;

/// The narrowest shape a box built from the ladder can have.
///
/// The lowest rung over the highest one, in [`SCALE`]'s unit.
const NARROWEST: u32 = LADDER[0] * SCALE / LADDER[LADDER.len() - 1];

/// The widest shape a box built from the ladder can have.
///
/// The highest rung over the lowest one, in [`SCALE`]'s unit.
const WIDEST: u32 = LADDER[LADDER.len() - 1] * SCALE / LADDER[0];

/// Why a ratio offered for an image kind could not be used.
///
/// The set is closed and exhaustive for 0004's reason: a caller matching on it
/// is told by the compiler when a case appears rather than falling into a branch
/// somebody wrote for something else. It is a local answer rather than a value
/// of the failure vocabulary, on the shape
/// [`super::address::SizeNotUsable`] already takes: 0037 requires every value of
/// that vocabulary to be built at one mapping point, and none of these is an
/// answer being read.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioNotUsable {
    /// The text is not digits, optionally a point and more digits.
    ///
    /// A leading sign and an exponent are both shapes a number may legally take
    /// in an answer body and both land here, unparsed. Inside the range an
    /// aspect ratio can occupy nothing is written either way, so this costs a
    /// well behaved server nothing and takes a negative, a zero spelled with a
    /// sign, and a magnitude no image has out before any arithmetic sees them.
    NotADecimalNumber,
    /// Below the lowest shape any box built from the ladder can have.
    ///
    /// Zero arrives here rather than under a name of its own. A ratio of zero is
    /// a server describing something no image is, which is what this variant
    /// says, and a second name for one value of a bound is a second thing to
    /// keep in step.
    NarrowerThanAnyBoxTheLadderBuilds,
    /// Above the widest shape any box built from the ladder can have.
    ///
    /// A number longer than anything here can hold arrives at this rather than
    /// wrapping: the digits past the point where the bound is already exceeded
    /// are consumed and not accumulated.
    WiderThanAnyBoxTheLadderBuilds,
    /// A ratio was offered for a kind neither supported line states one for.
    ///
    /// 0052 reads both lines and finds one stated shape, for
    /// [`ImageKind::Primary`]. A value offered for any other kind came from
    /// somewhere else on the same item, and the item type carries two fields
    /// that are about the item's own media rather than about an image of it. So
    /// this is a caller having mapped the wrong one, and it is refused over a
    /// runtime value because the loop that walks an item's five kinds holds the
    /// kind in a variable, which is where that mistake is made.
    StatedForAKindNoSupportedLineStatesOneFor(ImageKind),
}

/// A shape an image has, as a width per unit of height.
///
/// There is no way to build one outside the bound the ladder fixes, and no way
/// to build one from anything but the text a server sent, so a value of this
/// type is a ratio that can be drawn inside some box this core would ask for.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectRatio {
    ten_thousandths: u32,
}

impl AspectRatio {
    /// Reads the ratio out of the text an answer carried.
    ///
    /// `text` is the value exactly as the server sent it, before anything has
    /// turned it into a machine number. Digits, optionally a point and more
    /// digits; digits past the fourth after the point are consumed and dropped.
    ///
    /// # Errors
    ///
    /// [`RatioNotUsable`] where the text is not that shape, or where the value
    /// is outside the widest and narrowest shapes a box built from
    /// [`super::address::LADDER`] can have.
    pub fn from_server(text: &str) -> Result<Self, RatioNotUsable> {
        Ok(Self {
            ten_thousandths: ten_thousandths_of(text)?,
        })
    }

    /// The ratio in [`SCALE`]'s unit: the width per ten thousand of the height.
    #[must_use]
    pub const fn ten_thousandths(self) -> u32 {
        self.ten_thousandths
    }
}

/// What is known about an image's shape before any of its bytes exist.
///
/// Three values and no fourth, on the shape [`super::presence::WhatTheItemHas`]
/// already takes for a neighbouring question, and for the same reason: a caller
/// matching on it is told by the compiler when a case appears.
///
/// The last two reserve the same rectangle and are deliberately not merged. A
/// server that stated nothing and a server that stated something the core
/// refused are different statements, and collapsing them would make a server
/// sending nonsense look like a server being quiet.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatShapeIsKnown {
    /// The metadata stated a usable ratio for this kind.
    Stated(AspectRatio),
    /// The metadata stated nothing about this kind's shape.
    ///
    /// Every kind but [`ImageKind::Primary`] on both supported lines, a
    /// `Primary` the server left unset, and an item with no image of the kind at
    /// all. It carries nothing: 0068 places an item identifier in the personal
    /// data list, and an answer about an absence carrying the item it is about
    /// would take a personal field wherever the answer is reported.
    NothingStated,
    /// The metadata stated something that could not be used as a shape.
    ///
    /// A statement about the server, or about a caller that mapped the wrong
    /// field, rather than about the item.
    ARatioThatCannotBeUsed(RatioNotUsable),
}

impl WhatShapeIsKnown {
    /// Answers for one image kind, from the text the item's metadata carried.
    ///
    /// `stated` is [`None`] where the metadata held no ratio for this kind, and
    /// [`Some`] with the value exactly as the server sent it otherwise. An empty
    /// string is [`Some`] rather than [`None`] and is refused as such: a server
    /// that sent an empty value has sent something wrong rather than said
    /// nothing.
    #[must_use]
    pub fn of_kind(kind: ImageKind, stated: Option<&str>) -> Self {
        let Some(text) = stated else {
            return Self::NothingStated;
        };
        if !matches!(kind, ImageKind::Primary) {
            return Self::ARatioThatCannotBeUsed(
                RatioNotUsable::StatedForAKindNoSupportedLineStatesOneFor(kind),
            );
        }
        match AspectRatio::from_server(text) {
            Ok(ratio) => Self::Stated(ratio),
            Err(why) => Self::ARatioThatCannotBeUsed(why),
        }
    }

    /// The ratio, where this answer carries one.
    ///
    /// [`None`] for both of the other two answers, and the two are not merged
    /// here: a caller that wants to tell them apart matches on the value, and a
    /// caller that only wants the ratio does not have to.
    #[must_use]
    pub const fn ratio(&self) -> Option<AspectRatio> {
        match self {
            Self::Stated(ratio) => Some(*ratio),
            Self::NothingStated | Self::ARatioThatCannotBeUsed(_) => None,
        }
    }

    /// The rectangle a client reserves inside the box it asked to draw in.
    ///
    /// Where a ratio is stated, the largest rectangle of that shape fitting
    /// inside the box, computed in whole numbers that round down, so it never
    /// exceeds the box on either edge. No edge is zero, and what delivers that
    /// is the bound the ladder fixes rather than anything in this method: see
    /// [`fit`]. Where
    /// nothing is stated, and where a stated ratio was refused, the whole box:
    /// the client reserves the space it already asked to draw in, whatever
    /// arrives is drawn inside it, and no layout moves.
    #[must_use]
    pub fn rectangle_to_reserve(&self, within: DrawnSize) -> ReservedRectangle {
        let Self::Stated(ratio) = self else {
            return ReservedRectangle {
                width: within.width(),
                height: within.height(),
            };
        };
        let width = u64::from(within.width());
        let height = u64::from(within.height());
        let scaled = u64::from(ratio.ten_thousandths());
        let width_at_the_full_height = height * scaled / u64::from(SCALE);
        if width_at_the_full_height <= width {
            ReservedRectangle::inside(within, width_at_the_full_height, height)
        } else {
            ReservedRectangle::inside(within, width, width * u64::from(SCALE) / scaled)
        }
    }
}

/// The rectangle a client holds for an image that has not arrived.
///
/// Neither edge exceeds the box it was computed inside, and neither is zero.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReservedRectangle {
    width: u32,
    height: u32,
}

impl ReservedRectangle {
    /// The width to reserve.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// The height to reserve.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Holds both edges inside the box and off zero.
    fn inside(within: DrawnSize, width: u64, height: u64) -> Self {
        Self {
            width: fit(width, within.width()),
            height: fit(height, within.height()),
        }
    }
}

/// One edge, held inside the box's own edge.
///
/// THERE IS NO FLOOR HERE AND THAT IS DELIBERATE. A rectangle of no width is not
/// one a layout can hold, and what keeps both edges off zero is the bound rather
/// than a clamp: the narrowest ratio the ladder admits, inside the smallest box
/// the ladder builds, is 90 * 234 / 10000, which is 2. A floor was written here
/// first, and removing it reddened nothing, because nothing can reach it. It is
/// gone rather than kept, so that a bound widened past what the ladder justifies
/// reddens the case beside this rather than being rounded up to one pixel and
/// passing.
fn fit(value: u64, edge: u32) -> u32 {
    let bounded = value.min(u64::from(edge));
    u32::try_from(bounded).unwrap_or(edge)
}

/// Reads 0052's grammar and applies 0052's bound.
fn ten_thousandths_of(text: &str) -> Result<u32, RatioNotUsable> {
    let bytes = text.as_bytes();
    let ceiling = u64::from(WIDEST) / u64::from(SCALE) + 1;
    let mut index = 0;
    let mut whole: u64 = 0;
    let mut whole_digits = 0_usize;

    while index < bytes.len() && bytes[index].is_ascii_digit() {
        if whole <= ceiling {
            whole = whole * 10 + u64::from(bytes[index] - b'0');
        }
        index += 1;
        whole_digits += 1;
    }
    if whole_digits == 0 {
        return Err(RatioNotUsable::NotADecimalNumber);
    }

    let mut value = whole * u64::from(SCALE);
    if index < bytes.len() {
        if bytes[index] != b'.' {
            return Err(RatioNotUsable::NotADecimalNumber);
        }
        index += 1;
        let first_fraction_digit = index;
        let mut place = u64::from(SCALE) / 10;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            if index - first_fraction_digit < FRACTION_DIGITS_READ {
                value += u64::from(bytes[index] - b'0') * place;
                place /= 10;
            }
            index += 1;
        }
        if index == first_fraction_digit || index != bytes.len() {
            return Err(RatioNotUsable::NotADecimalNumber);
        }
    }

    if value < u64::from(NARROWEST) {
        return Err(RatioNotUsable::NarrowerThanAnyBoxTheLadderBuilds);
    }
    if value > u64::from(WIDEST) {
        return Err(RatioNotUsable::WiderThanAnyBoxTheLadderBuilds);
    }
    // Both bounds above are inside `u32`, so the conversion cannot fail. It is
    // written as a total expression rather than as an unwrap, and the fallback
    // is a value the bound already admits, so an unreachable branch is not an
    // uncovered one either.
    Ok(u32::try_from(value).unwrap_or(WIDEST))
}

#[cfg(test)]
mod tests {
    //! 0052's grammar, its bound and its two rectangles, asked of the answers.
    //!
    //! WHAT NONE OF THESE ASKS IS ANYTHING ABOUT A SERVER. Every ratio below is
    //! a byte string written here, because the read that would bring one is #27,
    //! and what is being proven is what this core does with a value once it has
    //! it rather than that a server sends that value.

    use super::{AspectRatio, RatioNotUsable, ReservedRectangle, SCALE, WhatShapeIsKnown};
    use crate::artwork::address::{DrawnSize, ImageKind, LADDER};
    use crate::artwork::presence::WhatTheItemHas;

    fn box_of(width: u32, height: u32) -> DrawnSize {
        DrawnSize::asked_for(width, height).expect("the fixture edges are rungs of the ladder")
    }

    fn poster() -> DrawnSize {
        box_of(300, 450)
    }

    /// A scaled ratio written back out in the grammar 0052 admits.
    fn text_of(ten_thousandths: u32) -> String {
        format!("{}.{:04}", ten_thousandths / SCALE, ten_thousandths % SCALE)
    }

    fn refusal(text: &str) -> RatioNotUsable {
        match WhatShapeIsKnown::of_kind(ImageKind::Primary, Some(text)) {
            WhatShapeIsKnown::ARatioThatCannotBeUsed(why) => why,
            other => panic!("{text:?} was admitted as {other:?}"),
        }
    }

    /// #52's first condition: every image kind can be asked for its shape, and
    /// asking costs no bytes because there are none to fetch.
    #[test]
    fn every_kind_answers_without_anything_being_fetched() {
        for kind in ImageKind::ALL {
            let known = WhatShapeIsKnown::of_kind(kind, None);
            assert_eq!(known, WhatShapeIsKnown::NothingStated, "{kind:?}");
            assert_eq!(known.ratio(), None, "{kind:?}");
            let reserved = known.rectangle_to_reserve(poster());
            assert_eq!(reserved.width(), poster().width(), "{kind:?}");
            assert_eq!(reserved.height(), poster().height(), "{kind:?}");
        }
    }

    /// 0052: one kind is stated on both lines and the other four are not, so a
    /// ratio offered for one of the four came from another field of the item.
    #[test]
    fn a_ratio_offered_for_a_kind_no_line_states_one_for_is_refused() {
        for kind in ImageKind::ALL {
            let known = WhatShapeIsKnown::of_kind(kind, Some("1.7778"));
            if matches!(kind, ImageKind::Primary) {
                assert!(matches!(known, WhatShapeIsKnown::Stated(_)), "{kind:?}");
            } else {
                assert_eq!(
                    known,
                    WhatShapeIsKnown::ARatioThatCannotBeUsed(
                        RatioNotUsable::StatedForAKindNoSupportedLineStatesOneFor(kind)
                    ),
                    "{kind:?}"
                );
            }
        }
    }

    /// The bound is the ladder's two ends rather than a number typed beside it.
    #[test]
    fn the_bound_is_the_widest_and_narrowest_box_the_ladder_builds() {
        let lowest = LADDER[0];
        let highest = LADDER[LADDER.len() - 1];
        assert_eq!((lowest, highest), (90, 3840));

        // 3840 over 90 is 42.6666..., and 90 over 3840 is 0.0234375.
        assert!(AspectRatio::from_server("42.6666").is_ok());
        assert_eq!(
            refusal("42.6667"),
            RatioNotUsable::WiderThanAnyBoxTheLadderBuilds
        );
        assert!(AspectRatio::from_server("0.0234").is_ok());
        assert_eq!(
            refusal("0.0233"),
            RatioNotUsable::NarrowerThanAnyBoxTheLadderBuilds
        );
    }

    /// Zero is the bound rather than a name of its own, and a number too long to
    /// hold is the bound rather than a value that wrapped into it.
    #[test]
    fn zero_is_narrower_and_a_very_long_number_is_wider() {
        assert_eq!(
            refusal("0"),
            RatioNotUsable::NarrowerThanAnyBoxTheLadderBuilds
        );
        assert_eq!(
            refusal("0.0000"),
            RatioNotUsable::NarrowerThanAnyBoxTheLadderBuilds
        );
        assert_eq!(
            refusal("99999999999999999999999999"),
            RatioNotUsable::WiderThanAnyBoxTheLadderBuilds
        );
    }

    /// The grammar, and the four shapes a number may legally take that it
    /// refuses. The near miss is the one that differs by a single byte.
    #[test]
    fn the_grammar_admits_digits_and_a_point_and_nothing_else() {
        for text in ["", ".", "1.", ".5", "1.7.7", "1,7778", " 1.7778", "1.7778 "] {
            assert_eq!(refusal(text), RatioNotUsable::NotADecimalNumber, "{text:?}");
        }
        for text in ["-1.5", "+1.5", "-0.0", "1e300", "1E3", "1.5e-2"] {
            assert_eq!(refusal(text), RatioNotUsable::NotADecimalNumber, "{text:?}");
        }
        assert!(AspectRatio::from_server("1.7778").is_ok());
        assert!(AspectRatio::from_server("1").is_ok());
    }

    /// Digits past the fourth are dropped rather than refused or accumulated.
    #[test]
    fn digits_past_the_fourth_are_not_read() {
        let short = AspectRatio::from_server("1.7777").expect("inside the bound");
        let long = AspectRatio::from_server("1.77779999999").expect("inside the bound");
        assert_eq!(short, long);
        assert_eq!(short.ten_thousandths(), 17_777);
    }

    /// The reserved rectangle is inside the box on both edges, for every pair of
    /// rungs and every ratio the bound admits at its ends and in the middle.
    #[test]
    fn a_reserved_rectangle_never_leaves_the_box_and_never_has_an_edge_of_zero() {
        // The two ends come from the bound rather than being typed, because the
        // bound is what keeps an edge off zero. A bound widened past what the
        // ladder justifies is then exercised at its own new ends here.
        let ends = [
            text_of(super::NARROWEST),
            text_of(super::WIDEST),
            String::from("0.6667"),
            String::from("1"),
            String::from("1.7778"),
            String::from("5.4054"),
        ];
        for text in &ends {
            let known = WhatShapeIsKnown::of_kind(ImageKind::Primary, Some(text));
            for width in LADDER {
                for height in LADDER {
                    let within = box_of(width, height);
                    let reserved = known.rectangle_to_reserve(within);
                    assert!(
                        reserved.width() <= within.width() && reserved.height() <= within.height(),
                        "{text} in {width}x{height} left the box as {reserved:?}"
                    );
                    assert!(
                        reserved.width() >= 1 && reserved.height() >= 1,
                        "{text} in {width}x{height} reserved nothing"
                    );
                }
            }
        }
    }

    /// The reserved rectangle has the stated shape, to within the pixel the
    /// rounding down costs, and it touches the box on the edge it is bounded by.
    #[test]
    fn a_reserved_rectangle_carries_the_stated_shape() {
        // 2:3 is the poster shape 0050's arithmetic is written against.
        let known = WhatShapeIsKnown::of_kind(ImageKind::Primary, Some("0.6667"));
        let reserved = known.rectangle_to_reserve(box_of(300, 450));
        assert_eq!((reserved.width(), reserved.height()), (300, 450));

        // 16:9 inside the same box is bounded by the width.
        let known = WhatShapeIsKnown::of_kind(ImageKind::Primary, Some("1.7778"));
        let reserved = known.rectangle_to_reserve(box_of(300, 450));
        assert_eq!((reserved.width(), reserved.height()), (300, 168));

        // 9:16 inside the same box is bounded by the height.
        let known = WhatShapeIsKnown::of_kind(ImageKind::Primary, Some("0.5625"));
        let reserved = known.rectangle_to_reserve(box_of(300, 450));
        assert_eq!((reserved.width(), reserved.height()), (253, 450));
    }

    /// Rounding down is the direction, so a reserved edge is never larger than
    /// the exact one and is never more than a pixel below it.
    #[test]
    fn rounding_goes_down_and_stops_within_a_pixel() {
        for text in ["0.6667", "1.7778", "0.5625", "2.3333"] {
            let scaled = u64::from(
                AspectRatio::from_server(text)
                    .expect("inside the bound")
                    .ten_thousandths(),
            );
            let known = WhatShapeIsKnown::of_kind(ImageKind::Primary, Some(text));
            for height in LADDER {
                let within = box_of(3840, height);
                let reserved = known.rectangle_to_reserve(within);
                let exact = u64::from(height) * scaled / u64::from(SCALE);
                if exact <= u64::from(within.width()) {
                    assert_eq!(u64::from(reserved.width()), exact.max(1), "{text} {height}");
                }
            }
        }
    }

    /// A refused ratio reserves the whole box and is still not an absence. The
    /// two answers agree on the rectangle and disagree on what happened.
    #[test]
    fn a_refused_ratio_reserves_the_box_and_is_not_the_same_answer_as_silence() {
        let refused = WhatShapeIsKnown::of_kind(ImageKind::Primary, Some("1e300"));
        let silent = WhatShapeIsKnown::of_kind(ImageKind::Primary, None);
        assert_eq!(
            refused.rectangle_to_reserve(poster()),
            silent.rectangle_to_reserve(poster())
        );
        assert_ne!(refused, silent);
        assert_eq!(refused.ratio(), None);
    }

    /// #52's third produce item, against #51's answer: an item with no image of
    /// a kind reserves the same rectangle as one whose shape is unstated,
    /// because an empty tile occupies a rectangle exactly as a full one does.
    #[test]
    fn an_item_with_no_image_of_a_kind_reserves_the_same_rectangle() {
        let item = crate::artwork::address::ItemId::from_server("item-4b2e")
            .expect("the fixture identifier is inside 0049's admitted set");

        let absent = WhatTheItemHas::of_kind(&item, ImageKind::Backdrop, None, poster());
        assert!(absent.is_absent());
        let has_one = WhatTheItemHas::of_kind(&item, ImageKind::Backdrop, Some("tag-1"), poster());
        assert!(!has_one.is_absent());

        let shape = WhatShapeIsKnown::of_kind(ImageKind::Backdrop, None);
        let reserved = shape.rectangle_to_reserve(poster());
        assert_eq!(
            reserved,
            ReservedRectangle {
                width: poster().width(),
                height: poster().height(),
            }
        );
    }

    /// The two numbers the bound is, pinned beside the two rungs they come from,
    /// so that a ladder whose ends move reddens here and whoever moved them
    /// reads 0052 rather than finding out later.
    #[test]
    fn the_two_bounds_are_the_ladder_ends_and_move_with_them() {
        assert_eq!((LADDER[0], LADDER[LADDER.len() - 1]), (90, 3840));
        assert_eq!(super::NARROWEST, 234);
        assert_eq!(super::WIDEST, 426_666);
        assert_eq!(SCALE, 10_000);
    }
}
