//! The decoded bytes the core holds at once, and what a decode does at the bound.
//!
//! `docs/decisions/0050-the-decoded-bytes-budget.md` decides all of it: sixty
//! four mebibytes of pixel buffer held at any moment, counted at four bytes a
//! pixel over the decoded dimensions, enforced by not starting a decode that
//! would exceed it rather than by failing one, with waiting decodes started in
//! the order they were asked for and a floor a client may not set the budget
//! below.
//!
//! # This is a working set and never a store
//!
//! Most of 0050's value is in that sentence, because the phrase reads as a cache
//! and is not one. 0009 hands a decoded image to the caller and never reads it
//! again, and 0006 caches artwork bytes and their declared dimensions rather
//! than pixels, so there is no decoded image anywhere in the core that somebody
//! is not waiting for. What is counted here is the pixel buffers of decodes that
//! are running plus those whose outcome has not yet reached the caller, and
//! nothing else. Read without that, the natural build is a cache of decoded
//! images, which is a second store with a second eviction policy for data 0006
//! deliberately does not keep, and the memory it occupies is the memory this
//! budget exists to bound.
//!
//! # Nothing fails at the bound
//!
//! A decode that would take the total past the budget does not start; it waits
//! until enough has been handed over. There is no sixteenth kind in 0004 for
//! this and none is asked for: nothing has gone wrong, and a failure here would
//! be one whose occurrence depends on what other callers were doing, which is
//! the least reproducible thing a client can be handed. So the answer type here
//! is [`WhatTheAskDoes`] and not a `Result`.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is the admission rule: the budget, the floor, the arithmetic
//! that turns declared dimensions into a buffer size, the running total, the
//! order waiting decodes are released in, and when a waiting decode has waited
//! long enough to be reported.
//!
//! WHAT IS NOT HERE IS THE DECODE. Nothing in this tree turns admitted bytes
//! into pixels, which [`super`] says of itself, so nothing here allocates a
//! buffer, runs on a lane or hands anything over. This module holds the rule
//! such a decoder would be admitted by, and the sizes it is told are the sizes a
//! decoder would allocate.
//!
//! WHAT IS ALSO NOT HERE IS THE LANE. 0050 declines to add a second bound on how
//! many decodes run at once, because 0009 already sizes the processing lane and
//! a second number for one question is how two answers get written down. Where
//! the lane comes from is what creating a core means, which is #115. The two
//! bounds interact rather than duplicate: this one also counts finished decodes
//! waiting to be handed over, which is the larger number on a client that
//! consumes slowly.
//!
//! WHAT IS ALSO NOT HERE IS THE REPORT. 0050 says the core reports how much is
//! held and how many decodes are waiting through 0100 after five seconds with a
//! decode waiting. [`DecodedBytesHeld::a_wait_worth_reporting`] answers whether
//! that moment has arrived; emitting the event is the facility in
//! [`crate::diagnostics`] and is not done here, so nothing in this tree reports
//! one.
//!
//! # Two figures in 0050 do not follow from its own arithmetic
//!
//! Both are recorded on #50 rather than repaired, because 0001 admits neither
//! correction as an edit to a landed record. They are named here because this
//! module is where an implementation written from those sentences would go
//! wrong.
//!
//! The record says the budget and the per-image bound are within four per cent
//! of each other. They are within 4.63 per cent of the budget and 4.86 per cent
//! of the bound, so five is the number true in both readings, and the rounding
//! goes the one way that makes the two bounds look closer than they are.
//!
//! The one that costs more to read wrongly: the record says that on a
//! four-processor television three maximal images cannot be decoded at once and
//! the THIRD waits. Two maximal images are 128000000 bytes against a budget of
//! 67108864, so it is the SECOND that waits and one maximal image fits with
//! nothing else beside it. [`DecodedBytesHeld`] is written to the arithmetic
//! rather than to that sentence, and
//! [`tests::one_maximal_image_fits_and_the_second_waits`] is what would redden if
//! somebody wrote the admission to the sentence instead.

use core::time::Duration;

use super::format::{DeclaredDimensions, PIXEL_COUNT_BOUND};
use crate::clock::SteadyInstant;

/// The bytes one pixel of a decoded image occupies.
///
/// From 0050. It is the buffer the core allocated at the decoded dimensions, and
/// not the encoded length, which
/// [`super::format::ENCODED_LENGTH_BOUND`] bounds separately during the
/// transfer, and not what the client does with the image afterwards, which is
/// the client's memory from the moment it is handed over.
pub const BYTES_A_PIXEL: u64 = 4;

/// The sixty four mebibytes the core holds at once unless a client sets
/// otherwise.
///
/// From 0050, chosen rather than measured, and #65 is where a measured
/// replacement would come from. A poster on a tile wall is taken as three
/// hundred by four hundred and fifty pixels, which is 540000 bytes decoded, so
/// this holds about 124 of them and a wall of two hundred at 108000000 bytes
/// does not fit. That is the point of the number rather than an accident of it:
/// what is on a screen at any moment is a fraction of two hundred, #53 cancels a
/// tile that scrolled off, and a budget large enough for every tile somebody
/// scrolled past holds a library in pixels.
pub const THE_BUDGET_AT_CREATION: u64 = 64 * 1024 * 1024;

/// The smallest budget a client may set.
///
/// Derived from 0055 rather than chosen here, which is 0050's own construction:
/// it is [`PIXEL_COUNT_BOUND`] at [`BYTES_A_PIXEL`]. Below it, an image 0055
/// admits could never be decoded, and the refusal a person would see would
/// depend on a memory setting rather than on the image, which is exactly the
/// client-dependent accept set 0055 exists against.
///
/// A client wanting a smaller ceiling is asking for 0055's dimension bound to be
/// lowered. That is a change to 0055 with its own argument about what an
/// attacker can make the core allocate, and it is not a number a client sets.
pub const THE_FLOOR_A_CLIENT_MAY_SET: u64 = PIXEL_COUNT_BOUND * BYTES_A_PIXEL;

/// How long a decode waits for room before the core says so.
///
/// From 0050. Not as an error on any call, because nothing failed, and not as a
/// sentence, because 0004 and 0100 both fix that the core writes none. It is an
/// interval between two events inside one run, so 0102 puts it on the steady
/// clock.
pub const A_WAITING_DECODE_IS_REPORTED_AFTER: Duration = Duration::from_secs(5);

/// Why a budget a client offered is not usable.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetNotUsable {
    /// Below [`THE_FLOOR_A_CLIENT_MAY_SET`].
    ///
    /// It carries both numbers, because the client's own value is the thing it
    /// has to change and a refusal that names neither sends somebody to read
    /// this file.
    BelowThePerImageBound {
        /// What the client offered, in bytes.
        offered: u64,
        /// The floor it has to reach, in bytes.
        floor: u64,
    },
}

/// The bound on decoded bytes held at once.
///
/// There is no ceiling on what a client may set it to, which is 0050's decision
/// rather than an omission here: a desktop with memory to spare is not a device
/// this record is protecting.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Budget {
    bytes: u64,
}

impl Budget {
    /// The budget a core has where the client set none.
    #[must_use]
    pub const fn at_creation() -> Self {
        Self {
            bytes: THE_BUDGET_AT_CREATION,
        }
    }

    /// A budget a client set, refused below the floor.
    ///
    /// # Errors
    ///
    /// [`BudgetNotUsable::BelowThePerImageBound`] where the value is under
    /// [`THE_FLOOR_A_CLIENT_MAY_SET`].
    pub const fn of(bytes: u64) -> Result<Self, BudgetNotUsable> {
        if bytes < THE_FLOOR_A_CLIENT_MAY_SET {
            return Err(BudgetNotUsable::BelowThePerImageBound {
                offered: bytes,
                floor: THE_FLOOR_A_CLIENT_MAY_SET,
            });
        }
        Ok(Self { bytes })
    }

    /// The bound, in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.bytes
    }
}

/// The pixel buffer one decode occupies.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecodedBytes {
    bytes: u64,
}

impl DecodedBytes {
    /// The buffer a decode at these dimensions would allocate.
    ///
    /// `None` where the dimensions are outside the bounds 0055 fixes. That is
    /// what makes the budget total rather than nearly total: every dimension
    /// this answers for produces a buffer no larger than
    /// [`THE_FLOOR_A_CLIENT_MAY_SET`], so no admissible image can be larger than
    /// the smallest budget a client may set, and nothing can wait for room that
    /// will never exist.
    #[must_use]
    pub const fn of(dimensions: DeclaredDimensions) -> Option<Self> {
        if !dimensions.are_inside_their_bounds() {
            return None;
        }
        Some(Self {
            bytes: dimensions.pixel_count() * BYTES_A_PIXEL,
        })
    }

    /// The buffer, in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.bytes
    }
}

/// What asking for a decode does under the budget.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheAskDoes {
    /// The buffer fits and the decode starts now.
    StartsNow,
    /// It waits for room, at the back of the queue.
    ///
    /// It is not a failure and there is no kind in 0004 for it. It carries its
    /// position so a caller reporting through 0100 can say where it stands
    /// without asking a second question.
    WaitsForRoom {
        /// How many decodes are waiting ahead of this one.
        behind: usize,
    },
}

/// The running total, and the queue of decodes waiting for room.
///
/// Thread safety, from 0009: this is bookkeeping the core mutates, so it carries
/// no interior mutability and is `Send + Sync` as a plain value. Whoever holds it
/// serialises access to it, the way the cache bookkeeping is held.
#[derive(Debug, Clone)]
pub struct DecodedBytesHeld {
    budget: Budget,
    held: u64,
    waiting: Vec<Waiting>,
}

/// One decode waiting for room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Waiting {
    bytes: DecodedBytes,
    since: SteadyInstant,
}

impl DecodedBytesHeld {
    /// Nothing decoded and nothing waiting, under this budget.
    #[must_use]
    pub const fn under(budget: Budget) -> Self {
        Self {
            budget,
            held: 0,
            waiting: Vec::new(),
        }
    }

    /// The bytes of pixel buffer the core holds right now.
    #[must_use]
    pub const fn held(&self) -> u64 {
        self.held
    }

    /// How many decodes are waiting for room.
    #[must_use]
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }

    /// The budget this is counted against.
    #[must_use]
    pub const fn budget(&self) -> Budget {
        self.budget
    }

    /// Asks for a decode of this size.
    ///
    /// A decode starts only where nothing is already waiting AND the buffer
    /// fits. The first half is what keeps 0050's order: without it a small
    /// decode asked for later would step past a large one that has been waiting,
    /// which is the starvation the record refuses when it declines to order the
    /// queue by size.
    pub fn ask(&mut self, bytes: DecodedBytes, now: SteadyInstant) -> WhatTheAskDoes {
        if self.waiting.is_empty() && self.fits(bytes) {
            self.held += bytes.bytes();
            return WhatTheAskDoes::StartsNow;
        }
        self.waiting.push(Waiting { bytes, since: now });
        WhatTheAskDoes::WaitsForRoom {
            behind: self.waiting.len() - 1,
        }
    }

    /// Releases a decode's buffer and starts whatever now fits, in order.
    ///
    /// One method for a completion handed to the caller and for a cancellation,
    /// because 0050 gives them the same effect on this total: 0009 bounds a
    /// cancelled decode to releasing its buffer at the end of the step it is
    /// inside, and the room then goes to whatever is waiting. What differs
    /// between them is what reaches the caller, which is not this module's.
    ///
    /// It returns the buffers that started, in the order they were asked for, so
    /// a caller does not have to ask the queue a second question to learn what
    /// it may now run.
    pub fn released(&mut self, bytes: DecodedBytes) -> Vec<DecodedBytes> {
        self.held = self.held.saturating_sub(bytes.bytes());
        let mut started = Vec::new();
        while let Some(first) = self.waiting.first().copied() {
            if !self.fits(first.bytes) {
                break;
            }
            self.waiting.remove(0);
            self.held += first.bytes.bytes();
            started.push(first.bytes);
        }
        started
    }

    /// Takes a decode out of the queue before it ever started.
    ///
    /// #53 cancels a tile that scrolled off, and one that scrolled off while
    /// waiting never allocated anything, so there is nothing to release. It
    /// answers whether a waiting decode of that size was found, because a caller
    /// withdrawing something the queue does not hold is a bookkeeping defect
    /// rather than an ordinary outcome.
    pub fn withdrawn_while_waiting(&mut self, bytes: DecodedBytes) -> bool {
        if let Some(at) = self.waiting.iter().position(|held| held.bytes == bytes) {
            self.waiting.remove(at);
            return true;
        }
        false
    }

    /// Whether a decode has waited long enough for the core to say so.
    ///
    /// The oldest wait is what decides it, so one long wait is reported even
    /// where later asks keep arriving. `None` is nothing waiting, which is a
    /// different answer from a wait that has not yet reached the interval.
    #[must_use]
    pub fn a_wait_worth_reporting(&self, now: SteadyInstant) -> Option<Duration> {
        let longest = self
            .waiting
            .iter()
            .map(|held| now.interval_since(held.since))
            .max()?;
        if longest >= A_WAITING_DECODE_IS_REPORTED_AFTER {
            Some(longest)
        } else {
            None
        }
    }

    /// Whether this buffer fits in what is left of the budget.
    const fn fits(&self, bytes: DecodedBytes) -> bool {
        self.budget.bytes() - self.held >= bytes.bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        A_WAITING_DECODE_IS_REPORTED_AFTER, BYTES_A_PIXEL, Budget, BudgetNotUsable, DecodedBytes,
        DecodedBytesHeld, THE_BUDGET_AT_CREATION, THE_FLOOR_A_CLIENT_MAY_SET, WhatTheAskDoes,
    };
    use crate::artwork::format::{AXIS_BOUND, DeclaredDimensions, PIXEL_COUNT_BOUND};
    use crate::clock::SteadyInstant;
    use core::time::Duration;

    fn at(millis: u64) -> SteadyInstant {
        SteadyInstant::from_nanos(millis * 1_000_000)
    }

    /// The poster 0050 does its arithmetic on: what #49 asks the server for
    /// rather than a full-resolution original.
    fn a_poster() -> DecodedBytes {
        DecodedBytes::of(DeclaredDimensions::of(300, 450))
            .expect("0055 admits three hundred by four hundred and fifty")
    }

    /// An image at exactly the per-image bound 0055 fixes, which is what fixes
    /// the floor here. Five thousand by three thousand two hundred is sixteen
    /// million pixels with both axes well inside [`AXIS_BOUND`], so it is the
    /// bound reached through the pixel count rather than through an axis.
    fn a_maximal_image() -> DecodedBytes {
        let dimensions = DeclaredDimensions::of(5_000, 3_200);
        assert_eq!(dimensions.pixel_count(), PIXEL_COUNT_BOUND);
        DecodedBytes::of(dimensions).expect("the largest image 0055 admits is admitted")
    }

    /// Fills whatever room is left with posters, asking only where the next one
    /// fits, so nothing reaches the queue. Every test below that needs a full
    /// budget builds it this way rather than assuming a count.
    fn fill_with_posters(held: &mut DecodedBytesHeld) -> usize {
        let mut started = 0;
        while held.held() + a_poster().bytes() <= held.budget().bytes() {
            assert_eq!(held.ask(a_poster(), at(0)), WhatTheAskDoes::StartsNow);
            started += 1;
        }
        started
    }

    /// 0050's own worked arithmetic, which the record states and this reproduces
    /// rather than restating. A poster is 540000 bytes and about 124 of them fit.
    #[test]
    fn the_arithmetic_0050_states_reproduces() {
        assert_eq!(a_poster().bytes(), 540_000);
        assert_eq!(THE_BUDGET_AT_CREATION, 67_108_864);
        assert_eq!(THE_BUDGET_AT_CREATION / a_poster().bytes(), 124);
    }

    /// The line 0050 calls the point of the number rather than an accident of
    /// it: a wall of two hundred tiles does not fit, and is not supposed to.
    #[test]
    fn a_wall_of_two_hundred_tiles_does_not_fit() {
        assert_eq!(200 * a_poster().bytes(), 108_000_000);
        assert!(200 * a_poster().bytes() > THE_BUDGET_AT_CREATION);
    }

    /// The figure 0050 states the other way round, recorded on #50 and written
    /// here to the arithmetic rather than to the sentence. Two maximal images are
    /// 128000000 bytes against a budget of 67108864, so the SECOND waits.
    #[test]
    fn one_maximal_image_fits_and_the_second_waits() {
        let maximal = a_maximal_image();
        assert_eq!(maximal.bytes(), PIXEL_COUNT_BOUND * BYTES_A_PIXEL);
        assert_eq!(maximal.bytes(), 64_000_000);
        assert!(2 * maximal.bytes() > THE_BUDGET_AT_CREATION);

        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        assert_eq!(held.ask(maximal, at(0)), WhatTheAskDoes::StartsNow);
        assert_eq!(
            held.ask(maximal, at(1)),
            WhatTheAskDoes::WaitsForRoom { behind: 0 }
        );
    }

    /// The floor is derived from 0055 rather than chosen, which is 0050's own
    /// construction, and it is what makes the budget total: no image 0055 admits
    /// can be larger than the smallest budget a client may set.
    #[test]
    fn no_admissible_image_is_larger_than_the_smallest_budget_a_client_may_set() {
        assert_eq!(THE_FLOOR_A_CLIENT_MAY_SET, 64_000_000);
        assert!(a_maximal_image().bytes() <= THE_FLOOR_A_CLIENT_MAY_SET);
        let smallest = Budget::of(THE_FLOOR_A_CLIENT_MAY_SET).expect("the floor is admitted");
        let mut held = DecodedBytesHeld::under(smallest);
        assert_eq!(
            held.ask(a_maximal_image(), at(0)),
            WhatTheAskDoes::StartsNow
        );
    }

    #[test]
    fn a_budget_below_the_per_image_bound_is_refused_with_both_numbers() {
        assert_eq!(
            Budget::of(THE_FLOOR_A_CLIENT_MAY_SET - 1),
            Err(BudgetNotUsable::BelowThePerImageBound {
                offered: THE_FLOOR_A_CLIENT_MAY_SET - 1,
                floor: THE_FLOOR_A_CLIENT_MAY_SET,
            })
        );
        assert!(Budget::of(THE_FLOOR_A_CLIENT_MAY_SET).is_ok());
    }

    /// 0050 sets no ceiling, and that is a decision rather than an omission.
    #[test]
    fn there_is_no_ceiling_on_what_a_client_may_set() {
        assert!(Budget::of(u64::MAX).is_ok());
    }

    /// Dimensions outside 0055's bounds produce no buffer size at all, which is
    /// what stops something waiting for room that will never exist.
    #[test]
    fn dimensions_0055_refuses_have_no_buffer_size_here() {
        assert!(DecodedBytes::of(DeclaredDimensions::of(AXIS_BOUND + 1, 1)).is_none());
        assert!(DecodedBytes::of(DeclaredDimensions::of(AXIS_BOUND, AXIS_BOUND)).is_none());
        assert!(DecodedBytes::of(DeclaredDimensions::of(1, 1)).is_some());
    }

    /// Nothing fails at the bound. The answer is a wait, and 0050 argues for that
    /// rather than for a sixteenth kind in 0004.
    #[test]
    fn a_decode_at_the_bound_waits_rather_than_failing() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        let mut admitted = 0;
        while held.ask(a_poster(), at(0)) == WhatTheAskDoes::StartsNow {
            admitted += 1;
        }
        assert_eq!(admitted, 124);
        assert_eq!(held.waiting(), 1);
        assert!(held.held() + a_poster().bytes() > Budget::at_creation().bytes());
    }

    /// Waiting decodes start in the order they were asked for, and that is
    /// visible only where the two answers differ: a small decode that WOULD fit
    /// at this instant still waits behind a large one that does not. Ordering the
    /// queue by size is what 0050 refuses when it says the large ones would
    /// starve, and the assertion in the middle is what makes this a test of the
    /// order rather than of the arithmetic.
    #[test]
    fn a_later_small_ask_does_not_step_past_an_earlier_large_one() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        let large = a_maximal_image();
        let small = a_poster();
        assert_eq!(held.ask(large, at(0)), WhatTheAskDoes::StartsNow);
        assert_eq!(
            held.ask(large, at(1)),
            WhatTheAskDoes::WaitsForRoom { behind: 0 }
        );

        assert!(
            held.budget().bytes() - held.held() >= small.bytes(),
            "the poster fits in what is left, so what follows is the order and not the room"
        );
        assert_eq!(
            held.ask(small, at(2)),
            WhatTheAskDoes::WaitsForRoom { behind: 1 }
        );
        assert_eq!(held.held(), large.bytes());

        // Both start when the room arrives, and the large one first.
        let started = held.released(large);
        assert_eq!(started, vec![large, small]);
        assert_eq!(held.waiting(), 0);
    }

    /// Releasing room starts everything that now fits, in one answer, so a caller
    /// does not have to ask the queue a second question.
    #[test]
    fn releasing_room_starts_everything_that_now_fits_in_order() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        let large = a_maximal_image();
        let small = a_poster();
        assert_eq!(held.ask(large, at(0)), WhatTheAskDoes::StartsNow);
        let filled = fill_with_posters(&mut held);
        for asked in 0..3 {
            assert_eq!(
                held.ask(small, at(asked + 1)),
                WhatTheAskDoes::WaitsForRoom {
                    behind: usize::try_from(asked).expect("three fits")
                }
            );
        }
        let started = held.released(large);
        assert_eq!(started, vec![small, small, small]);
        assert_eq!(held.waiting(), 0);
        assert_eq!(
            held.held(),
            u64::try_from(filled + 3).expect("eight fits") * small.bytes()
        );
    }

    /// A cancelled decode releases its buffer and the room goes to whatever is
    /// waiting, which is 0009's bound on cancellation rather than a new one here.
    /// It is the same method as a completion because 0050 gives them the same
    /// effect on this total.
    #[test]
    fn a_cancelled_decode_gives_its_room_to_whatever_was_waiting() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        let large = a_maximal_image();
        assert_eq!(held.ask(large, at(0)), WhatTheAskDoes::StartsNow);
        assert_eq!(
            held.ask(large, at(1)),
            WhatTheAskDoes::WaitsForRoom { behind: 0 }
        );
        assert_eq!(held.released(large), vec![large]);
        assert_eq!(held.held(), large.bytes());
        assert_eq!(held.waiting(), 0);
    }

    /// A tile that scrolled off while waiting never allocated anything, so there
    /// is nothing to release and it simply leaves the queue.
    #[test]
    fn a_decode_withdrawn_before_it_started_leaves_the_queue_and_releases_nothing() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        let large = a_maximal_image();
        let small = a_poster();
        assert_eq!(held.ask(large, at(0)), WhatTheAskDoes::StartsNow);
        let filled = fill_with_posters(&mut held);
        let before = held.held();

        assert_eq!(
            held.ask(small, at(1)),
            WhatTheAskDoes::WaitsForRoom { behind: 0 }
        );
        assert!(held.withdrawn_while_waiting(small));
        assert_eq!(held.waiting(), 0);
        assert_eq!(held.held(), before);
        assert_eq!(
            before,
            large.bytes() + u64::try_from(filled).expect("five fits") * small.bytes()
        );
        assert!(!held.withdrawn_while_waiting(small));
    }

    /// After five seconds with a decode waiting, the core says how long. Nothing
    /// waiting is a different answer from a wait that has not reached the
    /// interval, and the OLDEST wait decides, so later asks cannot hide one.
    #[test]
    fn a_wait_is_worth_reporting_only_after_the_interval() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        let large = a_maximal_image();
        assert_eq!(held.ask(large, at(0)), WhatTheAskDoes::StartsNow);
        assert_eq!(held.a_wait_worth_reporting(at(10_000)), None);

        assert_eq!(
            held.ask(large, at(1_000)),
            WhatTheAskDoes::WaitsForRoom { behind: 0 }
        );
        assert_eq!(held.a_wait_worth_reporting(at(5_999)), None);
        assert_eq!(
            held.a_wait_worth_reporting(at(6_000)),
            Some(A_WAITING_DECODE_IS_REPORTED_AFTER)
        );

        assert_eq!(
            held.ask(large, at(6_000)),
            WhatTheAskDoes::WaitsForRoom { behind: 1 }
        );
        assert_eq!(
            held.a_wait_worth_reporting(at(6_500)),
            Some(Duration::from_millis(5_500))
        );
    }

    /// The client's own backlog, which 0050 says is correct rather than a defect:
    /// a client that consumes none of its completions stalls only its own further
    /// decodes, and the core cannot tell it apart from one consuming slowly.
    #[test]
    fn a_client_that_consumes_nothing_stalls_only_itself() {
        let mut held = DecodedBytesHeld::under(Budget::at_creation());
        for _ in 0..124 {
            assert_eq!(held.ask(a_poster(), at(0)), WhatTheAskDoes::StartsNow);
        }
        for asked in 0..26 {
            assert_eq!(
                held.ask(a_poster(), at(0)),
                WhatTheAskDoes::WaitsForRoom { behind: asked }
            );
        }
        assert_eq!(held.waiting(), 26);
        assert_eq!(held.held(), 124 * a_poster().bytes());
        assert!(held.held() <= Budget::at_creation().bytes());

        let started = held.released(a_poster());
        assert_eq!(started, vec![a_poster()]);
        assert_eq!(held.waiting(), 25);
    }
}
