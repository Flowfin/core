//! The page a library read answers with, the total the server stated, and the
//! read that is not paged.
//!
//! `docs/decisions/0039-the-page-the-item-and-what-next-up-is-not.md` is the
//! record and #39 is the issue. It decides one answer shape and four properties
//! of it: that a page is asked for by AN OFFSET AND A COUNT, because that is
//! what the two routes 0010 names accept and there is no cursor to hold; that
//! the answer carries the items, the offset it begins at and the total the
//! server stated, with whether ANOTHER PAGE EXISTS derived from those three
//! rather than stored beside them; that the core NEVER TURNS COUNTING OFF,
//! because the field a server returns with counting off is the page's own
//! length arriving where a real total would; and that the view read is ANSWERED
//! WHOLE IN ONE PAGE, with a paging request against it refused rather than
//! turned into nothing on the wire.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of 0039 that a list and a count settle: the two
//! numbers a paged read is asked for by, the three numbers an answer carries,
//! the fourth derived from them, and which of 0010's library reads accepts a
//! page request at all.
//!
//! WHAT IS NOT HERE IS A READ. Nothing in this tree makes a request, for the
//! reason [`crate::server::transport`] gives about itself, so nothing here asks
//! a server anything and nothing here receives a page from one. This module
//! holds the shape such an answer is handed back in. #39's three conditions are
//! a test per call against a recorded fixture, paging proven across a boundary,
//! and one item type across the calls, and none of them is met by anything here.
//!
//! WHAT IS NOT HERE IS THE ITEM'S FIELDS, and that absence is 0039's own
//! sentence rather than an omission. The record fixes that every read answers
//! with ONE item type and says in the same paragraph that which fields a given
//! read populates depends on what the core asks for in the server's `fields`
//! parameter, which is a request-shaping decision that belongs with the code
//! that makes the request. So [`Page`] carries the item as a parameter: this
//! module holds the page while the type inside it is still #39's to shape, and
//! the difference between two reads shows up as an absent field on one type
//! rather than as a second type.
//!
//! WHAT IS NOT HERE IS NEXT UP. 0039 refuses it rather than deciding it: the
//! route exists on both server lines and answers with the same type, and it is
//! in none of 0010's capabilities, so a core reaching it would be growing an
//! enumerated surface in the record that describes the reads rather than in the
//! record that fixes them. [`LibraryRead`] holds the reads 0010 carries and no
//! others.
//!
//! # The trap this module is written against
//!
//! Both paged routes take a flag that turns counting off, it defaults to on, and
//! with it off the server fills the total in from the page it is returning. The
//! number then arrives in the same field, with the same type, as a real total. A
//! caller that pages until the offset plus the page length reaches the total
//! stops after one page and shows a library with one screenful in it, and
//! nothing anywhere reports an error. [`THE_TOTAL_IS_ALWAYS_ASKED_FOR`] is where
//! the core's side of that is written down.

/// Whether the core asks a server to count the whole set on a paged read.
///
/// Always, and this is 0039's decision rather than a default nobody chose. The
/// flag exists on both paged routes and defaults to on, so a core that never
/// sends it gets a real total; a core that sent it as false would receive the
/// length of the page it was already holding, where a caller reads the size of
/// the library.
///
/// It is not a performance option this core declines to use. It is a field that
/// becomes WRONG rather than absent, which is the one shape 0004's vocabulary
/// has no way to express and 0101 says to expect from a server.
pub const THE_TOTAL_IS_ALWAYS_ASKED_FOR: bool = true;

/// What a client asks a paged read for.
///
/// An offset and a count, because that is what the server has. There is no
/// cursor, no continuation token and no link header on either supported line, so
/// a cursor in the core's own interface would be a value the core invented over
/// an offset it still had to send.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    offset: usize,
    count: usize,
}

impl PageRequest {
    /// A request for `count` items beginning at `offset`.
    #[must_use]
    pub const fn beginning_at(offset: usize, count: usize) -> Self {
        Self { offset, count }
    }

    /// The offset the page is asked to begin at.
    #[must_use]
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// How many items are asked for.
    #[must_use]
    pub const fn count(self) -> usize {
        self.count
    }
}

/// What a library read answers with.
///
/// Three shapes rather than two, because 0039 separates a read that is answered
/// whole from a read that answers one item. Both refuse a page request and they
/// refuse it for different reasons, and one "not paged" would tell a caller the
/// same thing about two different surfaces.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheReadAnswers {
    /// A page asked for by an offset and a count.
    APageAskedFor,
    /// The whole set, in one page, always.
    ///
    /// The route takes neither parameter on either supported line, so there is
    /// nothing for an offset to be sent as.
    OnePageHoldingEverything,
    /// One item, on its own.
    OneItem,
}

/// The library reads 0010's capability table carries.
///
/// These four and no others. A fifth entry here would be an enumerated surface
/// growing in the record that describes the reads, which is what 0039 refuses
/// next up on.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryRead {
    /// The top of the library, from `library-query`. Answered whole.
    Views,
    /// The items in a view, from `library-query`. Paged.
    Items,
    /// What playback resumes into, from `resume-list`. Paged.
    Resume,
    /// One item in full, from `item-detail`.
    ItemDetail,
}

impl LibraryRead {
    /// What this read answers with.
    #[must_use]
    pub const fn answers(self) -> WhatTheReadAnswers {
        match self {
            Self::Views => WhatTheReadAnswers::OnePageHoldingEverything,
            Self::Items | Self::Resume => WhatTheReadAnswers::APageAskedFor,
            Self::ItemDetail => WhatTheReadAnswers::OneItem,
        }
    }

    /// Ask this read for a page.
    ///
    /// A READ THAT TAKES NO PAGING PARAMETERS REFUSES THE REQUEST RATHER THAN
    /// DROPPING IT, which is 0039's decision and the second half of the failure
    /// the total is. `GET /UserViews` accepts neither an offset nor a count on
    /// either supported line, so a core that carried a page request this far
    /// would send nothing extra on the wire and hand back the first answer as
    /// though it were the page that was asked for. A caller asking for the
    /// second hundred views and receiving the first hundred cannot tell a
    /// request that was not sent from one that was answered in full.
    #[must_use]
    pub const fn ask_for(self, request: PageRequest) -> WhatAskingForAPageDid {
        match self.answers() {
            WhatTheReadAnswers::APageAskedFor => WhatAskingForAPageDid::SendsIt(request),
            WhatTheReadAnswers::OnePageHoldingEverything => {
                WhatAskingForAPageDid::RefusedIt(NotAPagedRead::TheWholeAnswerIsOnePage)
            }
            WhatTheReadAnswers::OneItem => {
                WhatAskingForAPageDid::RefusedIt(NotAPagedRead::TheReadAnswersOneItem)
            }
        }
    }
}

/// Why a read took no page request.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPagedRead {
    /// The route takes no offset and no count, and the whole set comes back in
    /// one answer.
    TheWholeAnswerIsOnePage,
    /// The route answers one item rather than a set of them.
    TheReadAnswersOneItem,
}

/// What asking a read for a page did.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAskingForAPageDid {
    /// The read takes both parameters, so the request is what goes out.
    SendsIt(PageRequest),
    /// The read takes neither, so the request is refused here and never sent.
    RefusedIt(NotAPagedRead),
}

/// One answer from a library read.
///
/// The three fields are the server's own: the items, the offset the page begins
/// at, and the total it stated. The core adds no fourth. Whether there is
/// another page is DERIVED from those three by [`Page::has_another_page`],
/// because a stored flag is a fourth field that can disagree with the three it
/// came from.
///
/// The item is a parameter rather than a type this module fixes, for the reason
/// the module documentation gives: 0039 fixes that there is one item type across
/// every read and leaves which fields a read populates to the code that shapes
/// the request.
///
/// Thread safety, from 0009: a query result is immutable once it has been handed
/// back. Safe from any thread where the item is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Page<I> {
    items: Vec<I>,
    offset: usize,
    total: usize,
}

impl<I> Page<I> {
    /// The answer a paged read gave, as the server stated it.
    ///
    /// THE TOTAL IS NOT REVALIDATED, which is 0039 saying the core states the
    /// server's number and nothing more. A client comparing it against what it
    /// has drawn is comparing two things the server said at two moments, and a
    /// core that repaired the number here would be hiding the one case the
    /// record is written against rather than reporting it.
    #[must_use]
    pub const fn from_server(offset: usize, total: usize, items: Vec<I>) -> Self {
        Self {
            items,
            offset,
            total,
        }
    }

    /// The answer a read that takes no paging parameters gave.
    ///
    /// One page, beginning at zero, with a total equal to the number of items,
    /// so a caller written against a page works for this read too and is told
    /// there is no second page, which is true. That is the honest shape rather
    /// than a second answer type: a separate type for this read costs #39's last
    /// condition, which is that a client written against one read can display
    /// the result of another.
    #[must_use]
    pub const fn answered_whole(items: Vec<I>) -> Self {
        Self {
            offset: 0,
            total: items.len(),
            items,
        }
    }

    /// The items in this page.
    #[must_use]
    pub fn items(&self) -> &[I] {
        &self.items
    }

    /// How many items this page holds.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether this page holds nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The offset this page begins at, as the server stated it.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// The total the server stated for the whole set.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Whether the server's three numbers leave another page after this one.
    ///
    /// DERIVED FROM THE THREE AND NEVER STORED: the offset plus the number of
    /// items returned, against the total.
    ///
    /// A server asked not to count answers with the page's own length where the
    /// total goes, and this then reads as no further page after the first one.
    /// That is the arithmetic being right about the numbers it was given rather
    /// than a defect here, and it is why [`THE_TOTAL_IS_ALWAYS_ASKED_FOR`] is a
    /// decision rather than a default.
    #[must_use]
    pub const fn has_another_page(&self) -> bool {
        self.offset + self.items.len() < self.total
    }

    /// Where the next page begins, or nothing where the three numbers leave
    /// none.
    ///
    /// Derived the same way and from the same three, so a caller cannot reach a
    /// state where this answers an offset and [`Page::has_another_page`] says
    /// there is nothing after this one.
    #[must_use]
    pub const fn next_page_beginning_at(&self, count: usize) -> Option<PageRequest> {
        if self.has_another_page() {
            Some(PageRequest::beginning_at(
                self.offset + self.items.len(),
                count,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    //! 0039's page, its derived fourth number and its unpaged read, asked of the
    //! values.
    //!
    //! What these cannot ask is any of #39's three conditions. Each of those
    //! needs a request to have been made and a recording of one to exist, and
    //! nothing in this tree makes a request.

    use super::{
        LibraryRead, NotAPagedRead, Page, PageRequest, THE_TOTAL_IS_ALWAYS_ASKED_FOR,
        WhatAskingForAPageDid, WhatTheReadAnswers,
    };

    fn items(count: usize) -> Vec<usize> {
        (0..count).collect()
    }

    /// The three fields come back as the server stated them, and the core adds
    /// nothing to them.
    #[test]
    fn a_page_states_the_three_numbers_the_server_sent() {
        let page = Page::from_server(40, 913, items(20));

        assert_eq!(page.offset(), 40);
        assert_eq!(page.total(), 913);
        assert_eq!(page.len(), 20);
        assert_eq!(page.items()[0], 0);
        assert!(!page.is_empty());
    }

    /// The fourth number is derived, at the boundary rather than either side of
    /// it: one item short of the total leaves a page, and reaching it does not.
    #[test]
    fn another_page_is_derived_from_the_three_at_the_boundary() {
        assert!(Page::from_server(0, 101, items(100)).has_another_page());
        assert!(!Page::from_server(0, 100, items(100)).has_another_page());
        assert!(Page::from_server(100, 201, items(100)).has_another_page());
        assert!(!Page::from_server(100, 200, items(100)).has_another_page());
    }

    /// The offset the next page begins at is the offset plus what came back, and
    /// it is absent exactly where there is no further page.
    #[test]
    fn the_next_page_begins_where_this_one_ended() {
        let page = Page::from_server(100, 250, items(100));

        assert_eq!(
            page.next_page_beginning_at(100),
            Some(PageRequest::beginning_at(200, 100))
        );

        let last = Page::from_server(200, 250, items(50));
        assert_eq!(last.next_page_beginning_at(100), None);
        assert!(!last.has_another_page());
    }

    /// A server that returned fewer items than were asked for moves the next
    /// offset by WHAT CAME BACK and not by what was asked for, which are the
    /// same number on every page a full server answers and different on the
    /// first one it does not.
    #[test]
    fn a_short_page_moves_the_offset_by_what_came_back() {
        let short = Page::from_server(0, 250, items(40));

        assert!(short.has_another_page());
        assert_eq!(
            short.next_page_beginning_at(100),
            Some(PageRequest::beginning_at(40, 100))
        );
    }

    /// A server that was asked not to count answers with the page's own length
    /// where the total goes, and the derivation is right about the numbers it
    /// was given. The core's side of that is the constant rather than a repair
    /// here.
    #[test]
    fn a_total_that_is_the_page_length_leaves_no_further_page() {
        let counting_off = Page::from_server(0, 100, items(100));

        assert!(!counting_off.has_another_page());
        assert_eq!(counting_off.next_page_beginning_at(100), None);
        const { assert!(THE_TOTAL_IS_ALWAYS_ASKED_FOR) };
    }

    /// The view read is answered whole, in the page type the other reads use,
    /// and it says there is no second page because there is not.
    #[test]
    fn the_unpaged_read_is_one_page_holding_everything() {
        let views = Page::answered_whole(items(7));

        assert_eq!(views.offset(), 0);
        assert_eq!(views.total(), 7);
        assert_eq!(views.len(), 7);
        assert!(!views.has_another_page());
        assert_eq!(views.next_page_beginning_at(7), None);
    }

    /// Which of 0010's four library reads takes an offset and a count.
    #[test]
    fn the_two_paged_reads_are_the_two_the_server_pages() {
        assert_eq!(
            LibraryRead::Items.answers(),
            WhatTheReadAnswers::APageAskedFor
        );
        assert_eq!(
            LibraryRead::Resume.answers(),
            WhatTheReadAnswers::APageAskedFor
        );
        assert_eq!(
            LibraryRead::Views.answers(),
            WhatTheReadAnswers::OnePageHoldingEverything
        );
        assert_eq!(
            LibraryRead::ItemDetail.answers(),
            WhatTheReadAnswers::OneItem
        );
    }

    /// A page request against a read that takes no paging parameters is refused
    /// here rather than sent as nothing and answered as though it had been.
    #[test]
    fn asking_an_unpaged_read_for_a_page_is_refused_and_not_dropped() {
        let asked = PageRequest::beginning_at(100, 100);

        assert_eq!(
            LibraryRead::Views.ask_for(asked),
            WhatAskingForAPageDid::RefusedIt(NotAPagedRead::TheWholeAnswerIsOnePage)
        );
        assert_eq!(
            LibraryRead::ItemDetail.ask_for(asked),
            WhatAskingForAPageDid::RefusedIt(NotAPagedRead::TheReadAnswersOneItem)
        );
        assert_eq!(
            LibraryRead::Items.ask_for(asked),
            WhatAskingForAPageDid::SendsIt(asked)
        );
        assert_eq!(
            LibraryRead::Resume.ask_for(asked),
            WhatAskingForAPageDid::SendsIt(asked)
        );
    }

    /// The request carries both numbers back out unchanged.
    #[test]
    fn a_page_request_carries_the_two_numbers_the_server_takes() {
        let asked = PageRequest::beginning_at(300, 50);

        assert_eq!(asked.offset(), 300);
        assert_eq!(asked.count(), 50);
    }
}
