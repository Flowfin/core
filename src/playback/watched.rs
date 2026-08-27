//! What counts as watched, and who said so.
//!
//! 0060 is the record and #60 is the issue.
//!
//! # There is one number and this module does not hold it
//!
//! 0060 states no boundary of its own. The core marks an item watched at exactly
//! the boundary 0058 stops offering a resume at, and this module asks
//! [`Resume::of`] rather than comparing anything, so the two cannot disagree
//! because there is only one comparison in the tree.
//!
//! That is the decision rather than a convenience. Two numbers with a rule that
//! they must agree is a rule somebody has to check, and the check passes on the
//! day it is written and fails to exist on the day one of the two is edited by
//! somebody who did not know the other was there. Completion and the resume
//! boundary are written in different places weeks apart by whoever needed each,
//! and the drift arrives when one is tuned in response to a report.
//!
//! It also changes what this issue's own condition can be, which 0060 says
//! rather than leaving to be found. A test that would fail if the two numbers
//! were changed independently cannot be written, because there is nothing to
//! change independently. What it becomes is a test that an item at the boundary
//! is both marked watched and offered no resume, and one a moment before it is
//! neither, which a change to the single number moves as one.
//!
//! # What is not here
//!
//! Nothing calls any of this. A mark rides 0047's queue exactly as a position
//! does and #47 builds it, the reads that would supply a server's mark are #39,
//! and what a sign-out does to a queue holding an undelivered mark is 0114.
//! Reconciling a mark against a server that moved while the device was away is
//! #59, under 0058's rule for the same disagreement.

use super::{Ticks, resume::Resume};

/// Who decided that an item is watched.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// A mark is a boolean on the wire and this distinction is the core's own. A
/// server states that an item was played and does not state who decided it, so
/// the distinction does not survive a round trip through one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkedBy {
    /// A person said so, deliberately.
    ThePerson,
    /// The core did, on 0058's boundary.
    TheCore,
}

/// Whether an item is watched, and on whose word.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marked {
    /// Nothing has marked this item watched.
    NotWatched,
    /// It is watched, and this is who said so.
    Watched(MarkedBy),
}

impl Marked {
    /// What the core makes of a recorded position.
    ///
    /// It asks [`Resume::of`] rather than comparing against a boundary of its
    /// own, which is 0060's whole decision: an item the resume rule calls
    /// finished is watched, and there is no second number that could drift from
    /// that one.
    ///
    /// An item whose duration the server does not state is never watched by the
    /// core, and that follows from the same call rather than from a branch here.
    /// [`Resume::of`] has no finished answer without a duration, because a
    /// proportion has no denominator, and 0060 refuses to invent a fixed rule
    /// for that case: a fixed rule applied to an item of unknown length is a
    /// fixed rule applied to something that may be three minutes long, which
    /// 0058 already refuses at both ends of the length range.
    #[must_use]
    pub const fn of(recorded: Ticks, stated_duration: Option<Ticks>) -> Self {
        match Resume::of(recorded, stated_duration) {
            Resume::ItemIsFinished => Self::Watched(MarkedBy::TheCore),
            Resume::NoPositionIsKept | Resume::At(_) => Self::NotWatched,
        }
    }

    /// A person saying so.
    ///
    /// This needs no duration and is not a computation, and it is the only way
    /// an item whose duration the server does not state ever becomes watched.
    #[must_use]
    pub const fn by_the_person() -> Self {
        Self::Watched(MarkedBy::ThePerson)
    }

    /// A mark that came back from a server with nothing local beside it.
    ///
    /// Treated as the person's, and this is chosen rather than left to the first
    /// process that starts with an empty memory. Treating it as the core's would
    /// let a fresh process withdraw a mark somebody set deliberately on another
    /// device, and that person would see something they had marked finished
    /// return to a list of things to continue with no act of theirs behind it.
    ///
    /// THE COST OF THE SAFE DIRECTION IS STATED RATHER THAN AVOIDED. A mark the
    /// core made on another device is never withdrawn on this one. That error
    /// leaves an item in the state it is already in, and the other one takes
    /// away a statement somebody made on purpose.
    #[must_use]
    pub const fn arriving_from_the_server() -> Self {
        Self::Watched(MarkedBy::ThePerson)
    }

    /// What a seek to `recorded` does to this mark.
    ///
    /// Only a mark the core made is ever reconsidered. A person who seeks back
    /// before the boundary on an item the core marked has said something the
    /// core should believe, so that mark is withdrawn. The same seek on an item
    /// the person marked themselves changes nothing, because the core would be
    /// undoing a statement somebody made on purpose, and being wrong in that
    /// direction is the version people notice and resent.
    ///
    /// A seek on an item carrying no mark is the ordinary rule in
    /// [`Marked::of`], which is why an unmarked item seeked past the boundary
    /// becomes watched here rather than needing a second call.
    #[must_use]
    pub const fn after_a_seek_to(self, recorded: Ticks, stated_duration: Option<Ticks>) -> Self {
        match self {
            Self::Watched(MarkedBy::ThePerson) => self,
            Self::Watched(MarkedBy::TheCore) | Self::NotWatched => {
                Self::of(recorded, stated_duration)
            }
        }
    }

    /// Whether the item is watched at all, whoever said so.
    #[must_use]
    pub const fn is_watched(self) -> bool {
        matches!(self, Self::Watched(_))
    }
}

#[cfg(test)]
mod tests {
    use super::{Marked, MarkedBy};
    use crate::playback::{Ticks, resume::Resume};

    const TWO_HOURS: Ticks = Ticks::from_seconds(7_200);
    const THREE_MINUTES: Ticks = Ticks::from_seconds(180);

    /// The condition this issue asks for, in the shape 0060 says it becomes. An
    /// item at the boundary is both marked watched and offered no resume; one a
    /// moment before it is neither. A change to the single number moves both
    /// halves together, because both halves are that number.
    #[test]
    fn at_the_boundary_an_item_is_watched_and_offered_no_resume() {
        let boundary = TWO_HOURS.saturating_sub(Resume::FINISHED_INSIDE);

        assert_eq!(
            Marked::of(boundary, Some(TWO_HOURS)),
            Marked::Watched(MarkedBy::TheCore)
        );
        assert_eq!(
            Resume::of(boundary, Some(TWO_HOURS)),
            Resume::ItemIsFinished
        );
    }

    #[test]
    fn a_moment_before_the_boundary_an_item_is_neither_watched_nor_finished() {
        let a_moment_before =
            Ticks::from_ticks(TWO_HOURS.saturating_sub(Resume::FINISHED_INSIDE).as_ticks() - 1);

        assert_eq!(
            Marked::of(a_moment_before, Some(TWO_HOURS)),
            Marked::NotWatched
        );
        assert_ne!(
            Resume::of(a_moment_before, Some(TWO_HOURS)),
            Resume::ItemIsFinished
        );
    }

    /// The strongest form of "the two cannot disagree": across every length and
    /// every position tried, the core's mark and the resume rule's finished
    /// answer are the same answer. A second number anywhere would show here.
    #[test]
    fn the_core_marks_exactly_what_the_resume_rule_calls_finished() {
        for duration_seconds in [1_i64, 60, 180, 1_200, 1_500, 1_800, 3_600, 7_200, 86_400] {
            let duration = Ticks::from_seconds(duration_seconds);

            // The band is recomputed here only to choose sample points either
            // side of the boundary. Nothing below asserts where the boundary is;
            // the assertion is that both rules answer the same at every point,
            // so this arithmetic being wrong would cost coverage and not a
            // verdict.
            let proportion = Resume::a_proportion_of(duration);
            let band = if proportion.as_ticks() < Resume::FINISHED_INSIDE.as_ticks() {
                proportion
            } else {
                Resume::FINISHED_INSIDE
            };
            let boundary = duration.saturating_sub(band);

            let mut points = vec![Ticks::ZERO, duration];
            for offset in [-2_i64, -1, 0, 1, 2] {
                points.push(Ticks::from_ticks(boundary.as_ticks() + offset));
            }
            for part in 1..=9_i64 {
                points.push(Ticks::from_ticks(duration.as_ticks() / 10 * part));
            }

            for at in points {
                let finished = Resume::of(at, Some(duration)) == Resume::ItemIsFinished;
                let watched = Marked::of(at, Some(duration)).is_watched();
                assert_eq!(
                    finished,
                    watched,
                    "duration {duration_seconds}s, position {} ticks",
                    at.as_ticks()
                );
            }
        }
    }

    // ---- an item whose duration the server does not state ----

    /// 0060: nothing is marked watched by the core, at any position, because
    /// there is no denominator and no fixed rule to fall back on.
    #[test]
    fn the_core_never_marks_an_item_of_unknown_duration() {
        for seconds in [0_i64, 1, 60, 180, 7_200, 86_400, 8_640_000] {
            assert_eq!(
                Marked::of(Ticks::from_seconds(seconds), None),
                Marked::NotWatched
            );
        }
        assert_eq!(
            Marked::of(Ticks::from_ticks(i64::MAX), None),
            Marked::NotWatched
        );
    }

    /// Such an item becomes watched only by a person saying so, which needs no
    /// duration and is not a computation.
    #[test]
    fn a_person_can_mark_an_item_of_unknown_duration() {
        let mark = Marked::by_the_person();
        assert!(mark.is_watched());
        assert_eq!(mark.after_a_seek_to(Ticks::ZERO, None), mark);
    }

    // ---- who said so, and what that changes ----

    #[test]
    fn a_seek_back_before_the_boundary_withdraws_the_cores_mark() {
        let boundary = TWO_HOURS.saturating_sub(Resume::FINISHED_INSIDE);
        let by_the_core = Marked::of(boundary, Some(TWO_HOURS));
        assert_eq!(by_the_core, Marked::Watched(MarkedBy::TheCore));

        assert_eq!(
            by_the_core.after_a_seek_to(Ticks::from_seconds(600), Some(TWO_HOURS)),
            Marked::NotWatched
        );
        assert_eq!(
            by_the_core
                .after_a_seek_to(Ticks::from_ticks(boundary.as_ticks() - 1), Some(TWO_HOURS)),
            Marked::NotWatched
        );
    }

    #[test]
    fn the_same_seek_leaves_a_mark_the_person_made_alone() {
        let by_the_person = Marked::by_the_person();
        assert_eq!(
            by_the_person.after_a_seek_to(Ticks::from_seconds(600), Some(TWO_HOURS)),
            by_the_person
        );
        assert_eq!(
            by_the_person.after_a_seek_to(Ticks::ZERO, Some(TWO_HOURS)),
            by_the_person
        );
        assert_eq!(
            by_the_person.after_a_seek_to(Ticks::ZERO, Some(THREE_MINUTES)),
            by_the_person
        );
    }

    /// A seek that stays past the boundary is not a withdrawal, so the core's
    /// mark stands.
    #[test]
    fn a_seek_that_stays_past_the_boundary_leaves_the_cores_mark_standing() {
        let boundary = TWO_HOURS.saturating_sub(Resume::FINISHED_INSIDE);
        let by_the_core = Marked::of(boundary, Some(TWO_HOURS));
        assert_eq!(
            by_the_core
                .after_a_seek_to(Ticks::from_ticks(boundary.as_ticks() + 1), Some(TWO_HOURS)),
            Marked::Watched(MarkedBy::TheCore)
        );
    }

    /// An unmarked item seeked past the boundary is the ordinary rule rather
    /// than a second call a caller has to remember.
    #[test]
    fn an_unmarked_item_seeked_past_the_boundary_becomes_the_cores() {
        let boundary = TWO_HOURS.saturating_sub(Resume::FINISHED_INSIDE);
        assert_eq!(
            Marked::NotWatched.after_a_seek_to(boundary, Some(TWO_HOURS)),
            Marked::Watched(MarkedBy::TheCore)
        );
    }

    /// The safe direction, and the cost of it. A mark from a server with nothing
    /// local beside it is the person's, so a fresh process cannot withdraw it.
    #[test]
    fn a_mark_from_the_server_is_the_persons_and_is_never_reconsidered() {
        let from_the_server = Marked::arriving_from_the_server();
        assert_eq!(from_the_server, Marked::Watched(MarkedBy::ThePerson));
        assert_eq!(
            from_the_server.after_a_seek_to(Ticks::ZERO, Some(TWO_HOURS)),
            from_the_server
        );
        assert_eq!(
            from_the_server.after_a_seek_to(Ticks::from_seconds(30), None),
            from_the_server
        );
    }

    /// An item below 0058's start boundary is not watched either, which is the
    /// other end of the same single number.
    #[test]
    fn an_item_below_the_start_boundary_is_not_watched() {
        assert_eq!(
            Resume::of(Ticks::from_seconds(30), Some(TWO_HOURS)),
            Resume::NoPositionIsKept
        );
        assert_eq!(
            Marked::of(Ticks::from_seconds(30), Some(TWO_HOURS)),
            Marked::NotWatched
        );
    }
}
