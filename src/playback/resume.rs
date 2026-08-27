//! Where playback resumes, the two ends of an item, and whose position wins.
//!
//! 0058 is the record and #58 is the issue. Three thresholds and one rule, all
//! expressed in [`Ticks`], which is 0056's unit and this module's own reason for
//! sitting beside it.
//!
//! # Why the rules are here and not at a call site
//!
//! 0058's own argument is that a number that is not decided is not absent: it is
//! present in whatever the first caller wrote, and the first caller writes the
//! one that makes the case in front of them behave. So the numbers live once,
//! with the comparisons that use them, and a caller asks a question rather than
//! doing arithmetic.
//!
//! # What is not here
//!
//! Nothing calls any of this. The queue an undelivered position sits on is
//! 0047's and #47 builds it, the reports that record a position are #57, and the
//! reads that supply the server's are #39. What this module holds is the rule,
//! and every input it takes is supplied by something that does not exist yet.
//!
//! Marking an item watched is 0060 and #60, which takes the boundary below
//! rather than choosing one of its own.

use super::Ticks;

/// What a resume does with a recorded position.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Three answers rather than two, because the two that offer no resume are not
/// the same answer. 0058 sends a finished item to 0060 to be marked watched, and
/// it keeps an item below the first threshold out of whatever a client builds
/// out of items with a position. A caller collapsing them would either mark a
/// glance watched or fill that list with items nobody intends to return to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resume {
    /// The recorded position is inside the last ninety seconds or the last five
    /// per cent, whichever is shorter, so the item is finished. No resume is
    /// offered and 0060 marks it watched at this same boundary.
    ItemIsFinished,
    /// The recorded position is below the first sixty seconds or the first five
    /// per cent, whichever is shorter, so no position is kept. The item is
    /// offered from the start next time and appears in no list of things a
    /// person is part way through.
    NoPositionIsKept,
    /// Playback resumes here, which is the recorded position less the rewind.
    At(Ticks),
}

/// Which of the two positions a resume is computed from.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// 0058 decides this by delivery order and never by magnitude, and the two
/// answers below say which of the two was consulted rather than only which value
/// came out, because the whole content of the rule is which one was asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionInForce {
    /// The device holds a position for this item that it has not yet delivered,
    /// so that one wins and the server's is not consulted at all.
    TheDevices(Ticks),
    /// The device holds nothing undelivered, so the server's position is taken
    /// as it stands and no comparison is made.
    TheServers(Ticks),
    /// Neither holds a position for this item.
    Neither,
}

impl Resume {
    /// How far back a resume starts from the recorded position.
    ///
    /// Ten seconds, which 0057's reporting interval fixes rather than a feeling
    /// about how much of a sentence a person wants back: that record states the
    /// constraint in the direction that can be checked, the interval may not
    /// exceed the rewind, and ten satisfies it at the boundary. Every second
    /// beyond the boundary is paid on every ordinary resume, including the
    /// overwhelming majority where nothing was lost, and buys against a loss
    /// 0057 has already bounded.
    pub const REWIND: Ticks = Ticks::from_seconds(10);

    /// The duration half of the boundary past which an item is finished.
    ///
    /// Ninety rather than sixty because closing credits run about a minute and
    /// longer with a song over them. Sixty marks an item finished while the last
    /// of it is still playing, and the person that fails is the one who was
    /// still watching.
    pub const FINISHED_INSIDE: Ticks = Ticks::from_seconds(90);

    /// The duration half of the boundary below which no position is kept.
    ///
    /// Sixty seconds, sized to what a person does before deciding rather than to
    /// anything about the item. It has to clear the rewind to have any effect at
    /// all, since a recorded position under ten seconds already resumes at the
    /// start, and sixty is where a person has seen an opening rather than a
    /// moment of one.
    pub const KEPT_FROM: Ticks = Ticks::from_seconds(60);

    /// The proportion half of both boundaries, as a percentage of the stated
    /// duration.
    ///
    /// Five per cent at each end. Neither half of either pair works alone, which
    /// is why each is a pair: a fixed duration is wrong at the short end, since
    /// ninety seconds into a three minute item is half of it, and a proportion
    /// is wrong at the long end, since five per cent of a two hour film is six
    /// minutes and somebody who stopped there stopped in the middle of
    /// something. The shorter of the two is taken, so the crossing point is
    /// where the two rules agree rather than a third number.
    ///
    /// [`Resume::a_proportion_of`] is what computes it, and it divides rather
    /// than multiplying, for the reason written there.
    pub const PROPORTION_PER_CENT: i64 = 5;

    /// [`Resume::PROPORTION_PER_CENT`] of a stated duration.
    ///
    /// A division by twenty rather than a multiplication by five and a division
    /// by a hundred. The two agree on every value either can compute, and the
    /// multiplication overflows the width for a duration above a fifth of it,
    /// which is a number this type can hold and would then answer wrongly for
    /// rather than refusing. It truncates toward zero, so the band is at most
    /// one tick narrower than the exact proportion at each end, which is a
    /// hundred nanoseconds on a boundary whose other half is measured in
    /// minutes.
    #[must_use]
    pub const fn a_proportion_of(stated_duration: Ticks) -> Ticks {
        Ticks::from_ticks(stated_duration.as_ticks() / 20)
    }

    /// The shorter of a duration and the proportion, which is what both
    /// boundaries are made of.
    #[must_use]
    const fn the_shorter_of(fixed: Ticks, stated_duration: Ticks) -> Ticks {
        let proportion = Self::a_proportion_of(stated_duration);
        if proportion.as_ticks() < fixed.as_ticks() {
            proportion
        } else {
            fixed
        }
    }

    /// Where playback resumes for a recorded position, or why it does not.
    ///
    /// `stated_duration` of `None` is an item whose duration the server does not
    /// know. Neither proportion exists for it, so 0058 says plainly that a
    /// position is kept from the first moment, the item is never treated as
    /// finished on its own, and only the rewind still applies, because the
    /// rewind needs nothing but the position. 0060 answers the same absence the
    /// same way.
    ///
    /// A STATED DURATION OF ZERO IS NOT A CASE 0058 NAMES, and this applies the
    /// rule to it rather than inventing an answer: every position is inside the
    /// last nothing of it, so the item is finished. The server's own guard on
    /// the same arithmetic asks whether the duration is above zero before it
    /// applies any of its thresholds, so a server that states zero is one this
    /// rule and that one disagree about. Whether a stated zero should instead be
    /// read as no duration at all is a question about the server surface, which
    /// is #10.
    #[must_use]
    pub const fn of(recorded: Ticks, stated_duration: Option<Ticks>) -> Self {
        let Some(duration) = stated_duration else {
            return Self::At(recorded.saturating_sub(Self::REWIND));
        };

        let finished_from =
            duration.saturating_sub(Self::the_shorter_of(Self::FINISHED_INSIDE, duration));
        if recorded.as_ticks() >= finished_from.as_ticks() {
            return Self::ItemIsFinished;
        }

        if recorded.as_ticks() < Self::the_shorter_of(Self::KEPT_FROM, duration).as_ticks() {
            return Self::NoPositionIsKept;
        }

        Self::At(recorded.saturating_sub(Self::REWIND))
    }
}

/// Which position a resume is computed from when a device and a server both may
/// hold one.
///
/// 0058 decides this by delivery order and never by magnitude. Where the device
/// holds a position for that item that it has not yet delivered, on 0047's
/// queue, the device's wins and the server's is not read. Where it holds none,
/// the server's is taken as it stands and no comparison is made at all.
///
/// # Why not the larger number
///
/// Comparing two moments would mean comparing two devices' readings of the wall
/// clock 0102 places a moment on, and a rule that resolves a person's viewing
/// history by trusting whichever device's clock is further ahead fails in a way
/// nobody can see. Comparing two positions instead means the larger wins, and
/// the larger is wrong in the case that matters most: a person who finished
/// something on a phone and deliberately started it again on a television would
/// be sent back to the end of it.
///
/// # The case it gets wrong
///
/// A device that queued a position, then did not reach a server for a long time
/// while the person watched the same item somewhere else, comes back holding a
/// statement older than the server's and still wins. What bounds that window is
/// 0047's bound on the queue and 0045's recovery schedule that drains it, and
/// neither is a bound on how long a person may leave a device switched off.
#[must_use]
pub const fn which_wins(
    undelivered_on_this_device: Option<Ticks>,
    reported_by_the_server: Option<Ticks>,
) -> PositionInForce {
    match undelivered_on_this_device {
        Some(ours) => PositionInForce::TheDevices(ours),
        None => match reported_by_the_server {
            Some(theirs) => PositionInForce::TheServers(theirs),
            None => PositionInForce::Neither,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{PositionInForce, Resume, which_wins};
    use crate::playback::Ticks;

    /// A two hour film, which is the length 0058 argues both proportions against.
    const TWO_HOURS: Ticks = Ticks::from_seconds(7_200);

    /// Under thirty minutes the proportion is the shorter half at both ends,
    /// which is the crossing point 0058 names.
    const THREE_MINUTES: Ticks = Ticks::from_seconds(180);

    #[test]
    fn the_four_numbers_are_the_ones_the_record_states() {
        assert_eq!(Resume::REWIND, Ticks::from_seconds(10));
        assert_eq!(Resume::FINISHED_INSIDE, Ticks::from_seconds(90));
        assert_eq!(Resume::KEPT_FROM, Ticks::from_seconds(60));
        assert_eq!(Resume::PROPORTION_PER_CENT, 5);
    }

    /// The divisor and the percentage are the same number, wherever the
    /// multiplication is safe to compute.
    #[test]
    fn the_proportion_is_five_per_cent() {
        for seconds in [1_i64, 60, 180, 1_800, 7_200, 86_400] {
            let duration = Ticks::from_seconds(seconds);
            let by_percentage = duration.as_ticks() * Resume::PROPORTION_PER_CENT / 100;
            assert_eq!(Resume::a_proportion_of(duration).as_ticks(), by_percentage);
        }
    }

    /// The multiplication the divisor exists to avoid, on a duration this type
    /// holds. Five per cent of it is a real number and the product is not.
    #[test]
    fn the_proportion_of_the_largest_position_does_not_overflow() {
        let largest = Ticks::from_ticks(i64::MAX);
        assert_eq!(Resume::a_proportion_of(largest).as_ticks(), i64::MAX / 20);
        assert!(
            largest
                .as_ticks()
                .checked_mul(Resume::PROPORTION_PER_CENT)
                .is_none()
        );
    }

    // ---- the rewind, at both sides of its boundary ----

    #[test]
    fn a_resume_starts_ten_seconds_before_the_recorded_position() {
        let recorded = Ticks::from_seconds(600);
        assert_eq!(
            Resume::of(recorded, Some(TWO_HOURS)),
            Resume::At(Ticks::from_seconds(590))
        );
    }

    /// 0058: the rewind is not applied where the recorded position is below it,
    /// because a resume to a negative position is not a case to handle, it is a
    /// start.
    #[test]
    fn a_recorded_position_below_the_rewind_resumes_at_the_start() {
        assert_eq!(
            Resume::of(Ticks::from_seconds(4), None),
            Resume::At(Ticks::ZERO)
        );
        assert_eq!(Resume::of(Resume::REWIND, None), Resume::At(Ticks::ZERO));
        assert_eq!(
            Resume::of(Ticks::from_ticks(Resume::REWIND.as_ticks() + 1), None),
            Resume::At(Ticks::from_ticks(1))
        );
    }

    // ---- the end boundary, at both sides ----

    #[test]
    fn an_item_is_finished_inside_the_last_ninety_seconds_and_not_a_tick_before() {
        let boundary = TWO_HOURS.saturating_sub(Resume::FINISHED_INSIDE);
        assert_eq!(
            Resume::of(boundary, Some(TWO_HOURS)),
            Resume::ItemIsFinished
        );
        assert_eq!(
            Resume::of(Ticks::from_ticks(boundary.as_ticks() - 1), Some(TWO_HOURS)),
            Resume::At(Ticks::from_ticks(
                boundary.as_ticks() - 1 - Resume::REWIND.as_ticks()
            ))
        );
    }

    /// Below thirty minutes the proportion is the shorter half, so the boundary
    /// moves off ninety seconds. Nine seconds is five per cent of three minutes.
    #[test]
    fn on_a_short_item_the_end_boundary_is_the_proportion_and_not_the_duration() {
        let nine_seconds = Ticks::from_seconds(9);
        assert_eq!(Resume::a_proportion_of(THREE_MINUTES), nine_seconds);

        let boundary = THREE_MINUTES.saturating_sub(nine_seconds);
        assert_eq!(
            Resume::of(boundary, Some(THREE_MINUTES)),
            Resume::ItemIsFinished
        );
        assert_ne!(
            Resume::of(
                Ticks::from_ticks(boundary.as_ticks() - 1),
                Some(THREE_MINUTES)
            ),
            Resume::ItemIsFinished
        );
    }

    // ---- the start boundary, at both sides ----

    #[test]
    fn no_position_is_kept_below_the_first_sixty_seconds_and_one_is_kept_at_it() {
        assert_eq!(
            Resume::of(
                Ticks::from_ticks(Resume::KEPT_FROM.as_ticks() - 1),
                Some(TWO_HOURS)
            ),
            Resume::NoPositionIsKept
        );
        assert_eq!(
            Resume::of(Resume::KEPT_FROM, Some(TWO_HOURS)),
            Resume::At(Ticks::from_seconds(50))
        );
    }

    /// The same pair at the short end, where the proportion is again the shorter
    /// half: nine seconds rather than sixty.
    #[test]
    fn on_a_short_item_the_start_boundary_is_the_proportion_and_not_the_duration() {
        let nine_seconds = Ticks::from_seconds(9);
        assert_eq!(
            Resume::of(
                Ticks::from_ticks(nine_seconds.as_ticks() - 1),
                Some(THREE_MINUTES)
            ),
            Resume::NoPositionIsKept
        );
        assert_eq!(
            Resume::of(nine_seconds, Some(THREE_MINUTES)),
            Resume::At(Ticks::ZERO)
        );
    }

    /// The crossing points, which are two lengths rather than one, because the
    /// two ends pair the same proportion with different durations.
    ///
    /// 0058 names the end one: taking the shorter of the two gives ninety
    /// seconds to anything above thirty minutes and a proportion to everything
    /// below it. The start boundary is sixty seconds against the same five per
    /// cent, so it crosses at twenty minutes and not at thirty, and the record
    /// does not state that number. A reader who takes thirty minutes for both
    /// has the start boundary wrong for every item between twenty and thirty
    /// minutes, where the proportion is already the longer half.
    #[test]
    fn the_two_ends_cross_at_thirty_minutes_and_at_twenty() {
        let thirty_minutes = Ticks::from_seconds(1_800);
        assert_eq!(
            Resume::a_proportion_of(thirty_minutes),
            Resume::FINISHED_INSIDE
        );
        assert!(
            Resume::a_proportion_of(Ticks::from_seconds(1_799)).as_ticks()
                < Resume::FINISHED_INSIDE.as_ticks()
        );
        assert!(
            Resume::a_proportion_of(Ticks::from_seconds(1_801)).as_ticks()
                > Resume::FINISHED_INSIDE.as_ticks()
        );

        let twenty_minutes = Ticks::from_seconds(1_200);
        assert_eq!(Resume::a_proportion_of(twenty_minutes), Resume::KEPT_FROM);
        assert!(
            Resume::a_proportion_of(Ticks::from_seconds(1_199)).as_ticks()
                < Resume::KEPT_FROM.as_ticks()
        );
        assert!(
            Resume::a_proportion_of(Ticks::from_seconds(1_201)).as_ticks()
                > Resume::KEPT_FROM.as_ticks()
        );
    }

    /// The band between the two crossing points, where one end takes the fixed
    /// number and the other takes the proportion, on the same item.
    ///
    /// A twenty five minute item has a proportion of seventy five seconds, which
    /// is longer than the sixty at the start and shorter than the ninety at the
    /// end. So the start boundary is the fixed sixty and the end boundary is the
    /// proportion, and an implementation that took one half for both would be
    /// wrong at one end of every item in this band.
    #[test]
    fn between_the_two_crossings_each_end_takes_a_different_half() {
        let twenty_five_minutes = Ticks::from_seconds(1_500);
        let proportion = Resume::a_proportion_of(twenty_five_minutes);
        assert_eq!(proportion, Ticks::from_seconds(75));
        assert!(proportion.as_ticks() > Resume::KEPT_FROM.as_ticks());
        assert!(proportion.as_ticks() < Resume::FINISHED_INSIDE.as_ticks());

        assert_eq!(
            Resume::of(
                Ticks::from_ticks(Resume::KEPT_FROM.as_ticks() - 1),
                Some(twenty_five_minutes)
            ),
            Resume::NoPositionIsKept
        );
        assert_eq!(
            Resume::of(Resume::KEPT_FROM, Some(twenty_five_minutes)),
            Resume::At(Ticks::from_seconds(50))
        );

        let finished_from = twenty_five_minutes.saturating_sub(proportion);
        assert_eq!(finished_from, Ticks::from_seconds(1_425));
        assert_eq!(
            Resume::of(finished_from, Some(twenty_five_minutes)),
            Resume::ItemIsFinished
        );
        assert_ne!(
            Resume::of(
                Ticks::from_ticks(finished_from.as_ticks() - 1),
                Some(twenty_five_minutes)
            ),
            Resume::ItemIsFinished
        );
    }

    // ---- an item whose duration the server does not state ----

    /// 0058: a position is kept from the first moment, the item is never
    /// finished on its own, and only the rewind applies.
    #[test]
    fn an_item_with_no_stated_duration_keeps_a_position_from_the_first_moment() {
        assert_eq!(Resume::of(Ticks::ZERO, None), Resume::At(Ticks::ZERO));
        assert_eq!(
            Resume::of(Ticks::from_seconds(1), None),
            Resume::At(Ticks::ZERO)
        );
        assert_eq!(
            Resume::of(Ticks::from_seconds(30), None),
            Resume::At(Ticks::from_seconds(20))
        );
    }

    #[test]
    fn an_item_with_no_stated_duration_is_never_finished_on_its_own() {
        for seconds in [0_i64, 59, 60, 7_199, 7_200, 86_400] {
            assert_ne!(
                Resume::of(Ticks::from_seconds(seconds), None),
                Resume::ItemIsFinished
            );
            assert_ne!(
                Resume::of(Ticks::from_seconds(seconds), None),
                Resume::NoPositionIsKept
            );
        }
    }

    /// The case 0058 does not name, applied rather than invented, and pinned here
    /// so a later reading of it is a change somebody has to make on purpose.
    #[test]
    fn a_stated_duration_of_zero_makes_every_position_finished() {
        assert_eq!(
            Resume::of(Ticks::ZERO, Some(Ticks::ZERO)),
            Resume::ItemIsFinished
        );
        assert_eq!(
            Resume::of(Ticks::from_seconds(5), Some(Ticks::ZERO)),
            Resume::ItemIsFinished
        );
    }

    // ---- whose position wins ----

    /// The position ahead on the device's side. The device wins, which is also
    /// what magnitude would have said, so this half proves nothing on its own and
    /// is here to be read beside the one below it.
    #[test]
    fn the_device_wins_with_the_position_ahead_on_its_side() {
        assert_eq!(
            which_wins(
                Some(Ticks::from_seconds(600)),
                Some(Ticks::from_seconds(60))
            ),
            PositionInForce::TheDevices(Ticks::from_seconds(600))
        );
    }

    /// The position ahead on the server's side, which is the half that separates
    /// delivery order from magnitude. The device still wins.
    #[test]
    fn the_device_wins_with_the_position_ahead_on_the_servers_side() {
        assert_eq!(
            which_wins(
                Some(Ticks::from_seconds(60)),
                Some(Ticks::from_seconds(600))
            ),
            PositionInForce::TheDevices(Ticks::from_seconds(60))
        );
    }

    /// The server's value is not consulted at all when the device holds one, so
    /// moving it through every shape it can take moves nothing.
    #[test]
    fn the_servers_position_changes_nothing_while_the_device_holds_one() {
        let ours = Ticks::from_seconds(60);
        let expected = PositionInForce::TheDevices(ours);
        assert_eq!(which_wins(Some(ours), None), expected);
        for seconds in [0_i64, 1, 59, 60, 61, 7_200, 86_400] {
            assert_eq!(
                which_wins(Some(ours), Some(Ticks::from_seconds(seconds))),
                expected
            );
        }
        assert_eq!(
            which_wins(Some(ours), Some(Ticks::from_ticks(i64::MAX))),
            expected
        );
    }

    #[test]
    fn the_servers_position_is_taken_as_it_stands_where_the_device_holds_none() {
        assert_eq!(
            which_wins(None, Some(Ticks::from_seconds(600))),
            PositionInForce::TheServers(Ticks::from_seconds(600))
        );
        assert_eq!(
            which_wins(None, Some(Ticks::ZERO)),
            PositionInForce::TheServers(Ticks::ZERO)
        );
    }

    #[test]
    fn neither_holding_a_position_is_its_own_answer() {
        assert_eq!(which_wins(None, None), PositionInForce::Neither);
    }

    /// A device position of zero is a position rather than an absence, which is
    /// 0056's rule about the type carrying no absent value read from this side.
    #[test]
    fn a_device_position_at_the_beginning_still_wins() {
        assert_eq!(
            which_wins(Some(Ticks::ZERO), Some(Ticks::from_seconds(600))),
            PositionInForce::TheDevices(Ticks::ZERO)
        );
    }
}
