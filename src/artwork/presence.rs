//! What an item has for one image kind, and the two things an absence is not.
//!
//! `docs/decisions/0049-the-artwork-address-and-the-size-asked-for.md` stops one
//! step short of this and says so: an item whose metadata carries no tag for a
//! kind has no address at all, which is a value a caller has to handle rather
//! than an empty string that produces a request, and what that value is called
//! is #51. This module is that value.
//!
//! # Why an absence is not an error
//!
//! `docs/decisions/0055-the-image-formats-the-core-decodes.md` already separates
//! the two and gives the reason: one says the server has nothing and the other
//! says the server sent something wrong, and a client that cannot tell them
//! apart shows a failure sentence over a library that is fine. A personal
//! library holds whatever the metadata provider happened to have, so an item
//! with no poster is the ordinary case rather than the broken one.
//!
//! So the absence is an answer on the call and never a kind in
//! `docs/decisions/0004-the-error-vocabulary.md`. That vocabulary is closed at
//! fifteen and this adds nothing to it. The kind a hurried answer would reach
//! for is [`crate::failure::Kind::NotFound`], and the reason it is wrong is not
//! a matter of taste: `NotFound` is what
//! [`crate::failure::Failure::from_status`] builds from a status a server
//! returned, and nothing here sends a request for a server to answer. An item
//! with no image of a kind is known to have none before anything is asked.
//!
//! # The three answers, and the collapse between two of them
//!
//! A caller holding what an item's metadata carried for one kind has three
//! cases rather than two, and the third is the one that goes wrong quietly.
//!
//! The metadata carried a usable tag. There is an image and there is a request
//! for it.
//!
//! The metadata carried no tag. There is no image of that kind, and 0049's
//! sentence is that no address exists.
//!
//! The metadata carried a tag whose bytes may not be written into a request.
//! That is a server sending something the core refuses under 0101, and it is a
//! different statement from the item having no image. [`WhatTheItemHas::of_kind`]
//! takes the raw value rather than an [`ImageTag`] so that the refusal cannot be
//! turned into an absence by a caller writing `.ok()`, which is the one line
//! that makes a hostile or broken server indistinguishable from an item nobody
//! photographed.
//!
//! # What is deliberately not here
//!
//! THE ABSENCE IS NOT KEPT, AND THIS PARAGRAPH SAID #51'S THIRD CONDITION WAS
//! ABOUT KEEPING IT. That condition read that a second request for a known-absent
//! image makes no network call, which was read here as meaning the first answer
//! had been kept. `docs/decisions/0006-the-cache-contract.md` lists what may be
//! cached and an absence is none of the five entries, and
//! `docs/decisions/0043-a-stale-answer-and-the-freshness-rule-per-kind.md`
//! closes that list by saying a sixth kind is a change to both records. Keeping
//! one is still an edit to two landed records rather than a line written here,
//! and nothing in this module writes to a store.
//!
//! What moved is the condition rather than the records. It was narrowed on
//! 2026-08-30 to what this core can promise: a second ask for a known-absent
//! image within one screenful makes no network call BECAUSE THE ANSWER CARRIES
//! NO ADDRESS FOR ONE TO BE MADE FROM, and nothing about the absence outlives
//! the ask that produced it. That is a property of
//! [`WhatTheItemHas::NoImageOfThisKind`] rather than of a store: a fetch in this
//! core is started from an [`ArtworkRequest`], the request is what
//! [`crate::artwork::address::ArtworkRequest::entry_key`] turns into the key
//! 0053 coalesces on, and an absence produces neither. The test beside this type
//! asks exactly that, with the same two asks for an image that does exist beside
//! it, because an assertion that nothing was fetched proves nothing on its own
//! in a tree that fetches nothing.
//!
//! There is also no request made for the answer that carries one. The transport
//! is #27 and nothing in this tree opens a connection, so a caller receiving
//! [`WhatTheItemHas::AnImage`] receives an address and nothing has been fetched.

use super::address::{ArtworkRequest, DrawnSize, ImageKind, ImageTag, ItemId, NotUsableInARequest};

/// What an item's metadata says it has for one image kind.
///
/// Three values and no fourth, and the set is closed for 0004's reason: a caller
/// matching on it is told by the compiler when a case appears rather than
/// falling into a branch somebody wrote for something else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheItemHas {
    /// The metadata carried a usable tag, and this is the request for the image.
    ///
    /// Nothing has been fetched. The request is an address and a key, and the
    /// thing that would go and get it is #27.
    AnImage(ArtworkRequest),
    /// The metadata carried no tag for this kind, so the item has no image of
    /// it.
    ///
    /// It carries nothing, and that is the decision in this variant rather than
    /// an omission. 0068 places an item identifier in the personal data list, so
    /// an absence carrying the item it is about would carry a personal field
    /// into wherever the answer is reported. Two absences are the same value,
    /// which is what the test beside this type asserts.
    NoImageOfThisKind,
    /// The metadata carried a tag whose bytes may not be written into a request.
    ///
    /// A statement about the server rather than about the item. 0101 treats
    /// every byte from a server as untrusted, and this one would have chosen
    /// part of a request the core was about to send.
    ATagThatCannotBeUsed(NotUsableInARequest),
}

impl WhatTheItemHas {
    /// Answers for one kind of one item, from the tag its metadata carried.
    ///
    /// `tag` is [`None`] where the metadata held no tag for this kind, and
    /// [`Some`] with the value exactly as the server sent it otherwise. An empty
    /// string is [`Some`] rather than [`None`] and is refused as such: 0041
    /// requires an absent value and a present empty one to be different things,
    /// and a server that sends an empty tag has sent something wrong rather than
    /// said the item has no image.
    #[must_use]
    pub fn of_kind(item: &ItemId, kind: ImageKind, tag: Option<&str>, size: DrawnSize) -> Self {
        let Some(tag) = tag else {
            return Self::NoImageOfThisKind;
        };
        match ImageTag::from_server(tag) {
            Ok(tag) => Self::AnImage(ArtworkRequest::for_item(item, kind, &tag, size)),
            Err(why) => Self::ATagThatCannotBeUsed(why),
        }
    }

    /// The request, where this answer carries one.
    ///
    /// [`None`] for both of the other two answers, and the two are not merged
    /// here: a caller that wants to tell them apart matches on the value, and a
    /// caller that only wants the request does not have to.
    #[must_use]
    pub const fn request(&self) -> Option<&ArtworkRequest> {
        match self {
            Self::AnImage(request) => Some(request),
            Self::NoImageOfThisKind | Self::ATagThatCannotBeUsed(_) => None,
        }
    }

    /// Whether this answer says the item has no image of that kind.
    ///
    /// True for exactly one of the three. A tag the core refused is not an
    /// absence, which is the whole reason this is a method rather than a
    /// caller's `request().is_none()`.
    #[must_use]
    pub const fn is_absent(&self) -> bool {
        matches!(self, Self::NoImageOfThisKind)
    }
}

#[cfg(test)]
mod tests {
    //! 0049's sentence about an item with no tag, asked of the three answers.
    //!
    //! THE LAST TWO ASK #51'S THIRD CONDITION AS IT WAS NARROWED, and this
    //! comment said no test here could ask it at all. What it could not ask is
    //! the condition as first written, which needed an absence to have been kept
    //! and a request to have been made. The narrowed one is about the answer
    //! carrying no address, which is here.
    //!
    //! WHAT THEY STILL CANNOT ASK IS ANYTHING ABOUT A WIRE. Nothing in this tree
    //! opens a connection, so "no network call" is asserted as "no request and
    //! therefore no key to start a fetch from" rather than as a socket that was
    //! not opened. The second of the two is what keeps that from being vacuous:
    //! the identical loop over an image that does exist starts one fetch and
    //! joins it on the second ask.

    use super::WhatTheItemHas;
    use crate::artwork::address::{DrawnSize, ImageKind, ItemId, NotUsableInARequest};
    use crate::artwork::announced::{SharedFetches, WhatTheHoldDid};
    use crate::cache::key::{KeySpace, ServerPart};

    fn item(id: &str) -> ItemId {
        ItemId::from_server(id).expect("the fixture identifier is inside 0049's admitted set")
    }

    fn size() -> DrawnSize {
        DrawnSize::asked_for(300, 450).expect("300 by 450 is 0050's poster and two rungs")
    }

    fn space<'a>() -> KeySpace<'a> {
        KeySpace {
            server: ServerPart::Reported("server-7a1b"),
            account: "account-3c9d",
            device: "device-9f2c",
        }
    }

    /// Asks for one image twice, holding a fetch for every answer that carries a
    /// request, and answers how many fetches that started.
    ///
    /// One helper for both tests below so that the only difference between them
    /// is the tag, which is what makes the second one a near miss of the first
    /// rather than a differently written test that happens to pass.
    fn two_asks_for(tag: Option<&str>) -> (SharedFetches, Vec<WhatTheHoldDid>) {
        let space = space();
        let mut fetches = SharedFetches::none();
        let mut held = Vec::new();

        for _ in 0..2 {
            let answer =
                WhatTheItemHas::of_kind(&item("item-one"), ImageKind::Primary, tag, size());
            if let Some(request) = answer.request() {
                held.push(fetches.hold(request.entry_key(&space)));
            }
        }

        (fetches, held)
    }

    /// An item whose metadata carried no tag for a kind produces no request at
    /// all, which is 0049's sentence and the whole of #51's first condition.
    #[test]
    fn no_tag_for_a_kind_produces_no_request() {
        let answer = WhatTheItemHas::of_kind(&item("item-one"), ImageKind::Primary, None, size());

        assert_eq!(answer, WhatTheItemHas::NoImageOfThisKind);
        assert!(answer.is_absent());
        assert!(answer.request().is_none());
    }

    /// A tag the metadata did carry produces a request for every kind, so the
    /// absence is never the answer where there is something to fetch.
    #[test]
    fn a_tag_produces_a_request_for_every_kind_and_never_the_absence() {
        for kind in ImageKind::ALL {
            let answer = WhatTheItemHas::of_kind(&item("item-one"), kind, Some("abc123"), size());

            assert!(!answer.is_absent(), "{kind:?} answered with the absence");
            let request = answer
                .request()
                .expect("a usable tag is an image and an image has a request");
            assert_eq!(
                request.path(),
                format!("/Items/item-one/Images/{}", kind.as_str())
            );
        }
    }

    /// A tag whose bytes the core refuses is not an absence.
    ///
    /// This is the collapse the signature exists against: a caller holding an
    /// `ImageTag` would have had to turn the refusal into something, and `.ok()`
    /// turns it into an item nobody photographed.
    #[test]
    fn a_refused_tag_is_not_an_absence() {
        let slash = WhatTheItemHas::of_kind(
            &item("item-one"),
            ImageKind::Primary,
            Some("abc/123"),
            size(),
        );

        assert_eq!(
            slash,
            WhatTheItemHas::ATagThatCannotBeUsed(NotUsableInARequest::ByteAt(3))
        );
        assert!(!slash.is_absent());
        assert!(slash.request().is_none());
    }

    /// An empty tag is a server sending something wrong rather than an item with
    /// no image, which is 0041's rule about an absent value and a present empty
    /// one arriving here.
    #[test]
    fn an_empty_tag_is_not_the_same_answer_as_no_tag() {
        let empty =
            WhatTheItemHas::of_kind(&item("item-one"), ImageKind::Primary, Some(""), size());
        let absent = WhatTheItemHas::of_kind(&item("item-one"), ImageKind::Primary, None, size());

        assert_eq!(
            empty,
            WhatTheItemHas::ATagThatCannotBeUsed(NotUsableInARequest::Empty)
        );
        assert_ne!(empty, absent);
        assert!(!empty.is_absent());
    }

    /// Two absences are one value, so nothing about the item leaves through the
    /// answer.
    ///
    /// 0068 places an item identifier in the personal data list. An absence that
    /// carried the item it is about would carry that field into wherever the
    /// answer is reported, one layer before 0071 applies a treatment to it.
    #[test]
    fn two_absences_are_indistinguishable() {
        let one = WhatTheItemHas::of_kind(&item("item-one"), ImageKind::Primary, None, size());
        let another =
            WhatTheItemHas::of_kind(&item("a-different-item"), ImageKind::Backdrop, None, size());

        assert_eq!(one, another);
    }

    /// #51's third condition as it was narrowed on 2026-08-30. A second ask for
    /// a known-absent image within one screenful makes no network call, because
    /// the answer carries no address for one to be made from.
    ///
    /// Nothing is kept between the two asks: the loop holds no state of its own
    /// and this module writes to no store, so the second answer is derived again
    /// rather than remembered. What it is derived from is the item's metadata,
    /// which 0006 does list as a cached kind, so the absence follows from
    /// something the cache already holds without becoming a sixth kind in it.
    #[test]
    fn a_second_ask_for_a_known_absent_image_starts_no_fetch() {
        let (fetches, held) = two_asks_for(None);

        assert!(held.is_empty(), "an absence produced a request to hold");
        assert_eq!(fetches.in_flight(), 0);
    }

    /// The same two asks for an image that does exist, which is what keeps the
    /// assertion above from being about a loop that fetches nothing whatever it
    /// is given.
    ///
    /// One fetch is started and the second ask joins it rather than starting a
    /// second, which is 0053 coalescing on the entry key. Change `of_kind` to
    /// answer with a request where the metadata carried no tag and the test
    /// above goes red while this one does not move.
    #[test]
    fn the_same_two_asks_for_an_image_that_exists_share_one_fetch() {
        let (fetches, held) = two_asks_for(Some("abc123"));

        assert_eq!(
            held,
            [
                WhatTheHoldDid::StartedTheFetch,
                WhatTheHoldDid::JoinedTheFetch
            ]
        );
        assert_eq!(fetches.in_flight(), 1);
    }

    /// The answer is per kind, over the whole set the type declares.
    ///
    /// WHAT THIS DOES NOT ASSERT IS THAT AN ABSENCE CANNOT WIDEN, and the name
    /// it first carried said it did. A widening needs something that holds
    /// answers for an item across kinds, and this is a function of one kind with
    /// no state to widen through, so no one-change neighbour of it could make
    /// this fail that way. What it does hold is that neither answer is a
    /// property of which kind was asked: an absent tag is the absence for all
    /// five and a usable tag is never the absence for any of them. It walks
    /// [`ImageKind::ALL`] rather than a list typed beside it, for the reason
    /// that constant exists.
    #[test]
    fn the_answer_is_per_kind_for_every_kind_the_type_declares() {
        let id = item("item-one");
        for absent in ImageKind::ALL {
            let answer = WhatTheItemHas::of_kind(&id, absent, None, size());
            assert!(answer.is_absent());

            for present in ImageKind::ALL.iter().filter(|kind| **kind != absent) {
                let other = WhatTheItemHas::of_kind(&id, *present, Some("abc123"), size());
                assert!(
                    !other.is_absent(),
                    "{absent:?} being absent answered for {present:?}"
                );
            }
        }
    }
}
