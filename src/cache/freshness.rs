//! What a cache read answers with: the value, one of three states, and the age
//! that produced the state.
//!
//! 0043 is the record and #43 is the issue. 0006 fixes the three states and 0102
//! fixes what the age is measured against; this module holds the arithmetic of
//! the second and the table the first is read against.
//!
//! # Why the thresholds are here and not at a call site
//!
//! 0043's own argument is that the alternative is not five wrong numbers, it is
//! five places to look when the sixth caller wants to know what the rule is. So
//! the table lives once, with the comparison that uses it, and a caller asks a
//! question rather than doing arithmetic against a number it chose.
//!
//! The same holds for the two rules around the table, which 0043 says get
//! decided by accident and in opposite directions otherwise. A server may
//! shorten a threshold and may never lengthen one, which is
//! [`EntryKind::threshold`]. Age alone never withholds an entry, which is why
//! [`Answer`] has no variant for an entry too old to hand back: what produces
//! [`Answer::Absent`] is eviction under #42 or invalidation, never age.
//!
//! # What is not here
//!
//! Nothing calls any of this. The read path that would fetch bytes out of
//! [`crate::cache::ByteStore`], hand them here and answer a client sits above
//! both, and the calls a client makes are 0009's asynchronous surface, which
//! #115 builds.
//!
//! Neither is the demand for freshness. 0043 fixes what it returns when the
//! server cannot be reached, which is the transport's own kind from 0004, never
//! a stale entry and never a cache-specific failure, and [`crate::failure`]
//! holds no type to return, because the one mapping point 0037 requires is #37.
//! A demand written here today would have to invent a failure value at the call
//! site, which is the exact thing that record exists to prevent.
//!
//! Nothing stamps an entry with a version or tells a complete one from a
//! truncated one, so every [`WrittenAt`] below describes an entry this version
//! wrote completely. That is 0105 and #105, and 0043 already names it as the
//! condition its whole table is written under.

use crate::clock::WallMoment;
use core::time::Duration;

/// Nanoseconds in one second, as the width the moment arithmetic below is done
/// in.
///
/// A device that believes it is before the epoch is a moment this core carries
/// rather than saturates, which is [`WallMoment`]'s own decision, and the
/// difference of two such moments is signed for the same reason. The width is
/// wider than the product of the two parts so that no reading either type can
/// carry reaches its end.
const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// A moment on the device's own clock, as signed nanoseconds from the epoch.
///
/// This is arithmetic on one clock rather than a length of time measured on it.
/// 0102 forbids the second and [`WallMoment`] carries no subtraction for that
/// reason; what this module does instead is subtract two readings and then
/// correct the result by the skew, which is the whole of what makes the answer
/// survive a device clock that moved.
fn nanos_from_the_epoch(moment: WallMoment) -> i128 {
    i128::from(moment.seconds_from_the_epoch()) * NANOS_PER_SECOND + i128::from(moment.nanos())
}

/// A kind of cache entry, which is what fixes how long it stays fresh.
///
/// Five kinds, which is the set 0006 lists. A sixth is a change to that record
/// and to 0043 rather than a variant somebody adds here, because the threshold
/// beside each one is argued in the record and a variant with no row is a kind
/// with no rule.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EntryKind {
    /// The answer to a query over a library, which changes when anybody adds a
    /// file.
    LibraryQueryResults,
    /// What a server says about one item, which changes when a provider is
    /// re-run or somebody edits a field.
    ItemMetadata,
    /// What a server says it can do, which changes when the server is upgraded.
    ServerCapabilityAnswers,
    /// The bytes of an image, whose address is content-tagged, so a changed
    /// image is a different key rather than a stale entry.
    ArtworkBytes,
    /// The width and the height read out of those bytes, kept so that a layout
    /// can reserve space before the bytes arrive under #52.
    DecodedDimensions,
}

impl EntryKind {
    /// How long artwork bytes stay fresh, and the same for the dimensions read
    /// out of them.
    ///
    /// It is a named constant rather than two rows carrying one number, because
    /// 0043 states the second as "same as their bytes" and two numbers that are
    /// meant to agree are two numbers that will not. What the thirty days bound
    /// is not tracking is change: 0006 makes a changed image a different key, so
    /// this bounds the case where a server reuses a tag for different bytes,
    /// which is a server defect rather than a normal event.
    pub const ARTWORK_BYTES_STALE_AFTER: Duration = Duration::from_hours(30 * 24);

    /// How long an entry of this kind stays fresh, from 0043's table.
    ///
    /// None of these numbers is a measurement. They are chosen, the reasoning
    /// for each one is in the record rather than repeated here, and #65 is where
    /// a measured replacement would come from.
    #[must_use]
    pub const fn stale_after(self) -> Duration {
        match self {
            Self::LibraryQueryResults => Duration::from_mins(5),
            Self::ItemMetadata => Duration::from_hours(1),
            Self::ServerCapabilityAnswers => Duration::from_hours(24),
            Self::ArtworkBytes | Self::DecodedDimensions => Self::ARTWORK_BYTES_STALE_AFTER,
        }
    }

    /// The threshold an entry of this kind is actually read against, once the
    /// server has had its say.
    ///
    /// A server may shorten a threshold and may never lengthen one. It knows
    /// things the core does not, most obviously that a library is being written
    /// to, so a shorter time is used. It knows nothing about the device, and the
    /// table is about a person in front of a screen, so a longer time is
    /// discarded and the table wins.
    ///
    /// The direction is the whole content of this call. Honouring a server's
    /// stated freshness in both directions is what an ordinary cache does and is
    /// what happens with nobody deciding it, because that statement is the
    /// nearest thing to hand when a read path is written. What it costs is the
    /// direction where a server, or whatever sits in front of one, decides how
    /// long somebody's device holds their library, which 0101 places outside
    /// what the operator's trust covers.
    ///
    /// A response the server said may not be kept at all is not part of this
    /// trade and is not expressed here. 0006 already fixes that such a response
    /// is not kept, whatever else would allow it, so it never becomes an entry
    /// with a threshold to read.
    #[must_use]
    pub fn threshold(self, the_server_said: Option<Duration>) -> Duration {
        let from_the_table = self.stale_after();
        the_server_said.map_or(from_the_table, |said| said.min(from_the_table))
    }
}

/// The difference between a server's own stated time and the device's reading of
/// the same instant.
///
/// 0102 anchors an entry's age on the server rather than on any device clock,
/// because an age has to survive a restart of the process and both monotonic
/// clocks reset at one. This is the quantity that anchoring is done with: kept
/// with the entry at write, measured again at read, and the difference between
/// the two is what removes a device clock that moved in between.
///
/// It carries no uncertainty bound, and that is deliberate rather than an
/// omission. 0102 requires a MEASURED skew to be a value with a bound, because a
/// bare subtraction is wrong by the round trip, and the round trip is a fact the
/// transport holds. That measurement is #27's and does not exist. What this type
/// is, is the subtraction an entry keeps, and the bound belongs beside the
/// measurement rather than beside the entry.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skew {
    /// The server's moment less the device's, in nanoseconds.
    nanos: i128,
}

impl Skew {
    /// The skew between a server's stated moment and the device's reading of the
    /// same instant.
    #[must_use]
    pub fn between(
        the_servers_stated_moment: WallMoment,
        the_devices_wall_moment: WallMoment,
    ) -> Self {
        Self {
            nanos: nanos_from_the_epoch(the_servers_stated_moment)
                - nanos_from_the_epoch(the_devices_wall_moment),
        }
    }
}

/// The two moments an entry stores when it is written.
///
/// 0102 requires both, and this type is why: an entry that kept only the
/// device's reading has no anchor to correct against, and an entry that kept
/// only the server's has nothing to subtract a later device reading from. The
/// skew between them is derived rather than stored a third time.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrittenAt {
    the_servers_stated_moment: WallMoment,
    the_devices_wall_moment: WallMoment,
}

impl WrittenAt {
    /// The two moments, read at the same instant.
    ///
    /// At the same instant is what the caller owes and what nothing here can
    /// check. A device reading taken a second after the response was parsed puts
    /// that second into the skew, and every later age carries it.
    #[must_use]
    pub const fn at(
        the_servers_stated_moment: WallMoment,
        the_devices_wall_moment: WallMoment,
    ) -> Self {
        Self {
            the_servers_stated_moment,
            the_devices_wall_moment,
        }
    }

    /// What the server said the time was when it answered.
    #[must_use]
    pub const fn the_servers_stated_moment(self) -> WallMoment {
        self.the_servers_stated_moment
    }

    /// What the device believed the time was at the same instant.
    #[must_use]
    pub const fn the_devices_wall_moment(self) -> WallMoment {
        self.the_devices_wall_moment
    }

    /// The skew at write, which is what a later reading is corrected against.
    #[must_use]
    pub fn skew_at_write(self) -> Skew {
        Skew::between(self.the_servers_stated_moment, self.the_devices_wall_moment)
    }
}

/// Why an age could not be read as a length of time.
///
/// Two reasons rather than one because 0102 names two devices, and they are not
/// the same device: one that came up believing it is 1970, and one that jumped
/// forward. Collapsed, a diagnostic under #100 could report that a clock is
/// wrong and never which way, which is the half that says whether anything can
/// be done about it.
///
/// This is not the reason field 0043 refuses. That refusal is about which
/// THRESHOLD a stale entry passed, on the grounds that a threshold is the core's
/// and a client acting on one would be reimplementing the table. Neither value
/// below is a threshold, and neither says anything about the table.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyTheAgeIsUnreadable {
    /// The corrected difference came out below zero, which is a device clock
    /// that moved backwards between the write and the read by more than the
    /// correction removed.
    ItComputedAsNegative,
    /// The corrected difference came out past the sanity bound, which is a
    /// device that jumped forward.
    ItPassedTheSanityBound,
}

/// How old an entry is, on the anchor 0102 fixes.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Age {
    /// The age, as a length of time. This is what a client saying how long ago
    /// something was updated reads, and 0043 carries it on a fresh entry as well
    /// as on a stale one so that no client learns to ask twice.
    Of(Duration),
    /// One of the two guards fired, so there is no length of time to give. The
    /// entry is past every threshold, which is what [`Age::is_past`] answers.
    Unreadable(WhyTheAgeIsUnreadable),
}

impl Age {
    /// The age past which a computed age is not believed.
    ///
    /// Chosen rather than measured, and the choice is bounded on both sides. It
    /// has to be above every threshold in 0043's table, whose longest is thirty
    /// days, by enough that an entry kept through a long absence is still read
    /// as what it is; and it has to be below the magnitudes 0102 names as a
    /// clock that was never set, which that record describes as a decade. A year
    /// is an order of magnitude above the first and an order below the second.
    ///
    /// What crossing it costs is the age reading and never the entry. An entry
    /// past this bound is marked stale, which is where it already was, since the
    /// bound is far beyond every threshold in the table; what a client loses is
    /// the number and not the value. That is the direction 0043 requires, and it
    /// is why this bound can be chosen at all without measuring anything.
    pub const SANITY_BOUND: Duration = Duration::from_hours(365 * 24);

    /// The age of an entry at the moment it is read.
    ///
    /// 0102 fixes the arithmetic: the device's reading now, less the entry's
    /// stored device moment, corrected by the difference between the skew now
    /// and the skew at write. A device clock that moved between the write and
    /// the read moved both device readings and moved neither server moment, so
    /// the correction is exactly that movement and removes it.
    ///
    /// `the_skew_now` is `None` for the offline case, where there is no current
    /// measurement to correct against and the age is the uncorrected difference.
    /// That is 0102's own answer rather than a fallback: a correction computed
    /// from a skew nothing measured is a number nothing measured.
    ///
    /// Both guards fail towards asking the server. A needless request costs a
    /// round trip; an entry that is permanently fresh costs somebody seeing
    /// something that is not there any more, with nothing in the system that
    /// will ever correct it.
    #[must_use]
    pub fn at_read(
        written_at: WrittenAt,
        the_devices_wall_now: WallMoment,
        the_skew_now: Option<Skew>,
    ) -> Self {
        let uncorrected = nanos_from_the_epoch(the_devices_wall_now)
            - nanos_from_the_epoch(written_at.the_devices_wall_moment());
        let corrected = the_skew_now.map_or(uncorrected, |now| {
            uncorrected + (now.nanos - written_at.skew_at_write().nanos)
        });

        if corrected < 0 {
            return Self::Unreadable(WhyTheAgeIsUnreadable::ItComputedAsNegative);
        }
        // A difference too wide to express as a length of time is past the bound
        // by many orders of magnitude, so this is the same answer arrived at one
        // step earlier. It is reachable rather than defensive: the moments this
        // core carries run far beyond where a length of time here ends, and the
        // test below drives it.
        let Ok(nanos) = u64::try_from(corrected) else {
            return Self::Unreadable(WhyTheAgeIsUnreadable::ItPassedTheSanityBound);
        };
        let age = Duration::from_nanos(nanos);
        if age > Self::SANITY_BOUND {
            return Self::Unreadable(WhyTheAgeIsUnreadable::ItPassedTheSanityBound);
        }
        Self::Of(age)
    }

    /// Whether this age has reached a threshold.
    ///
    /// An age exactly at its threshold has reached it. "Stale after five
    /// minutes" is read as the five-minute-old entry being the first stale one
    /// rather than the last fresh one, which is the direction that fails towards
    /// asking the server.
    ///
    /// An unreadable age is past every threshold, including the longest in the
    /// table. That is the one place the two guards act.
    #[must_use]
    pub const fn is_past(self, threshold: Duration) -> bool {
        match self {
            Self::Of(age) => age.as_nanos() >= threshold.as_nanos(),
            Self::Unreadable(_) => true,
        }
    }
}

/// An entry the cache holds, as the read path found it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    kind: EntryKind,
    value: Vec<u8>,
    written_at: WrittenAt,
    the_server_said: Option<Duration>,
}

impl Held {
    /// An entry, its kind, the two moments it was written at, and what the
    /// server said about how long it may be kept.
    ///
    /// `the_server_said` is the server's own statement where the response
    /// carried one, translated to a length of time by whatever parsed that
    /// response. It is not a threshold: it is one of the two inputs to
    /// [`EntryKind::threshold`], and it can only shorten.
    #[must_use]
    pub fn of(
        kind: EntryKind,
        value: Vec<u8>,
        written_at: WrittenAt,
        the_server_said: Option<Duration>,
    ) -> Self {
        Self {
            kind,
            value,
            written_at,
            the_server_said,
        }
    }
}

/// What one cache read answers with.
///
/// Three states, from 0006, and 0043 fixes what each one carries. There is no
/// fourth and there is no variant for an entry withheld for its age: age marks
/// and invalidation removes, which is what keeps the three at three.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The entry has not reached its threshold. It carries its age even so,
    /// because a client that wants to say how old something is needs the number
    /// in the case where nothing is wrong, and an interface supplying it only on
    /// the unhappy path teaches every client to ask twice.
    Fresh {
        /// The bytes the cache holds.
        value: Vec<u8>,
        /// How old they are.
        age: Age,
    },
    /// The entry has reached its threshold. It carries the value and the age and
    /// nothing else: no field says which threshold it passed, because a
    /// threshold is the core's and a client acting on one would be
    /// reimplementing the table.
    Stale {
        /// The bytes the cache holds.
        value: Vec<u8>,
        /// How old they are.
        age: Age,
    },
    /// The cache holds nothing usable under that key. An entry never written,
    /// one eviction removed under #42, and one invalidation removed are all this
    /// answer, and an invalidated entry is this rather than [`Answer::Stale`]
    /// because what was held is wrong rather than old.
    Absent,
}

/// Answers one cache read.
///
/// This is the one place the three states are decided, so that a caller reads a
/// state rather than comparing an age against a number it chose for itself.
/// `held` is `None` for every route to [`Answer::Absent`].
#[must_use]
pub fn answer(
    held: Option<Held>,
    the_devices_wall_now: WallMoment,
    the_skew_now: Option<Skew>,
) -> Answer {
    let Some(held) = held else {
        return Answer::Absent;
    };
    let age = Age::at_read(held.written_at, the_devices_wall_now, the_skew_now);
    if age.is_past(held.kind.threshold(held.the_server_said)) {
        Answer::Stale {
            value: held.value,
            age,
        }
    } else {
        Answer::Fresh {
            value: held.value,
            age,
        }
    }
}

#[cfg(test)]
mod tests {
    //! What these prove, and what they cannot.
    //!
    //! Every case below moves a clock rather than waiting on one, which is what
    //! 0102 requires of the suite and what makes a timing case here take
    //! microseconds. Nothing in this file reads a platform clock, and the
    //! `no-platform-clock` rule in `.github/invariants/rules` is what refuses
    //! one anywhere under `src/`.
    //!
    //! None of them proves a threshold is the right length. The numbers are
    //! chosen, 0043 says so above its own table, and what these assert is that
    //! the table is the one being read.

    use super::{Age, Answer, EntryKind, Held, Skew, WhyTheAgeIsUnreadable, WrittenAt, answer};
    use crate::clock::WallMoment;
    use core::time::Duration;

    /// A moment on either clock, from whole seconds.
    fn at(seconds: i64) -> WallMoment {
        WallMoment::from_epoch(seconds, 0)
    }

    /// An entry written with the device and the server agreeing.
    fn written_with_no_skew(at_second: i64) -> WrittenAt {
        WrittenAt::at(at(at_second), at(at_second))
    }

    fn some_bytes() -> Vec<u8> {
        b"a-library-list".to_vec()
    }

    #[test]
    fn every_kind_carries_the_threshold_the_table_states() {
        assert_eq!(
            EntryKind::LibraryQueryResults.stale_after(),
            Duration::from_mins(5)
        );
        assert_eq!(
            EntryKind::ItemMetadata.stale_after(),
            Duration::from_hours(1)
        );
        assert_eq!(
            EntryKind::ServerCapabilityAnswers.stale_after(),
            Duration::from_hours(24)
        );
        assert_eq!(
            EntryKind::ArtworkBytes.stale_after(),
            Duration::from_hours(30 * 24)
        );
    }

    /// 0043 states this row as "same as their bytes" rather than as a number, so
    /// the assertion is the relation and not a second copy of thirty days.
    #[test]
    fn decoded_dimensions_go_stale_with_the_bytes_they_describe() {
        assert_eq!(
            EntryKind::DecodedDimensions.stale_after(),
            EntryKind::ArtworkBytes.stale_after()
        );
    }

    #[test]
    fn a_server_that_says_nothing_leaves_the_table_alone() {
        assert_eq!(
            EntryKind::ItemMetadata.threshold(None),
            EntryKind::ItemMetadata.stale_after()
        );
    }

    #[test]
    fn a_server_may_shorten_a_threshold() {
        assert_eq!(
            EntryKind::ItemMetadata.threshold(Some(Duration::from_secs(60))),
            Duration::from_secs(60)
        );
    }

    /// The half of the rule that is given away by accident. A read path that
    /// honoured the server's statement in both directions would pass every other
    /// case in this file.
    #[test]
    fn a_server_may_never_lengthen_one() {
        assert_eq!(
            EntryKind::ItemMetadata.threshold(Some(Duration::from_hours(7 * 24))),
            EntryKind::ItemMetadata.stale_after()
        );
    }

    #[test]
    fn an_age_is_the_distance_between_the_write_and_the_read() {
        let age = Age::at_read(written_with_no_skew(1_000), at(1_060), None);
        assert_eq!(age, Age::Of(Duration::from_secs(60)));
    }

    #[test]
    fn the_skew_at_write_is_the_servers_moment_less_the_devices() {
        let written = WrittenAt::at(at(1_030), at(1_000));
        assert_eq!(written.skew_at_write(), Skew::between(at(1_030), at(1_000)));
        assert_eq!(written.the_servers_stated_moment(), at(1_030));
        assert_eq!(written.the_devices_wall_moment(), at(1_000));
    }

    /// The correction, in the direction that would otherwise make an entry look
    /// older than it is. The device clock jumps forward by an hour between the
    /// write and the read; the server's own time advances by the minute that
    /// actually passed, so the skew now is an hour below the skew at write and
    /// the age is still a minute.
    ///
    /// The device is thirty seconds behind the server at write rather than
    /// level with it, and that is the whole reason this case bites. With the two
    /// agreeing, the skew at write is zero, and a correction that forgot to
    /// subtract it would produce the same answer as one that did.
    #[test]
    fn a_device_clock_that_jumped_forward_does_not_age_an_entry() {
        let written = WrittenAt::at(at(1_030), at(1_000));
        let the_skew_now = Skew::between(at(1_090), at(4_660));
        let age = Age::at_read(written, at(4_660), Some(the_skew_now));
        assert_eq!(age, Age::Of(Duration::from_secs(60)));
    }

    /// The same correction in the other direction, with the same thirty seconds
    /// of skew at write. The device clock moves back by an hour, which without
    /// the correction is an age below zero and one of the two guards, and the
    /// device reading lands before the epoch on the way.
    #[test]
    fn a_device_clock_that_moved_backwards_does_not_freshen_an_entry() {
        let written = WrittenAt::at(at(1_030), at(1_000));
        let the_skew_now = Skew::between(at(1_090), at(-2_540));
        let age = Age::at_read(written, at(-2_540), Some(the_skew_now));
        assert_eq!(age, Age::Of(Duration::from_secs(60)));
    }

    /// 0102's offline case. There is no current skew to correct against, so the
    /// answer is the uncorrected difference and not a correction computed from
    /// the skew the entry happens to carry.
    #[test]
    fn offline_the_age_is_the_uncorrected_difference() {
        let written = WrittenAt::at(at(1_030), at(1_000));
        assert_eq!(
            Age::at_read(written, at(1_060), None),
            Age::Of(Duration::from_secs(60))
        );
    }

    #[test]
    fn an_age_that_computes_as_negative_is_unreadable() {
        let age = Age::at_read(written_with_no_skew(10_000), at(9_000), None);
        assert_eq!(
            age,
            Age::Unreadable(WhyTheAgeIsUnreadable::ItComputedAsNegative)
        );
    }

    #[test]
    fn an_age_past_the_sanity_bound_is_unreadable() {
        let two_years = 2 * 365 * 24 * 60 * 60;
        let age = Age::at_read(written_with_no_skew(0), at(two_years), None);
        assert_eq!(
            age,
            Age::Unreadable(WhyTheAgeIsUnreadable::ItPassedTheSanityBound)
        );
    }

    /// An age exactly at the bound is not past it, which is the one-second
    /// neighbour of the case above.
    #[test]
    fn an_age_exactly_at_the_sanity_bound_is_still_a_length_of_time() {
        let bound = 365 * 24 * 60 * 60;
        assert_eq!(
            Age::at_read(written_with_no_skew(0), at(bound), None),
            Age::Of(Age::SANITY_BOUND)
        );
    }

    /// A difference too wide to express as a length of time at all. It is the
    /// same answer as the bound, arrived at one step earlier, and this is what
    /// drives that step.
    #[test]
    fn an_age_too_wide_to_express_is_past_the_sanity_bound() {
        let far = 1_000_000_000_000;
        let age = Age::at_read(written_with_no_skew(0), at(far), None);
        assert_eq!(
            age,
            Age::Unreadable(WhyTheAgeIsUnreadable::ItPassedTheSanityBound)
        );
    }

    #[test]
    fn an_unreadable_age_is_past_every_threshold_in_the_table() {
        let unreadable = Age::Unreadable(WhyTheAgeIsUnreadable::ItComputedAsNegative);
        assert!(unreadable.is_past(EntryKind::LibraryQueryResults.stale_after()));
        assert!(unreadable.is_past(EntryKind::ARTWORK_BYTES_STALE_AFTER));
    }

    #[test]
    fn an_age_exactly_at_its_threshold_has_reached_it() {
        assert!(Age::Of(Duration::from_mins(5)).is_past(Duration::from_mins(5)));
    }

    /// The one-nanosecond neighbour of the case above, which is what says the
    /// comparison is at the boundary rather than somewhere near it.
    #[test]
    fn an_age_one_nanosecond_short_of_its_threshold_has_not() {
        let just_short = Duration::from_mins(5) - Duration::from_nanos(1);
        assert!(!Age::Of(just_short).is_past(Duration::from_mins(5)));
    }

    #[test]
    fn a_key_the_cache_holds_nothing_under_is_absent() {
        assert_eq!(answer(None, at(1_000), None), Answer::Absent);
    }

    #[test]
    fn an_entry_inside_its_threshold_is_fresh_and_carries_its_age() {
        let held = Held::of(
            EntryKind::LibraryQueryResults,
            some_bytes(),
            written_with_no_skew(1_000),
            None,
        );
        assert_eq!(
            answer(Some(held), at(1_060), None),
            Answer::Fresh {
                value: some_bytes(),
                age: Age::Of(Duration::from_secs(60)),
            }
        );
    }

    #[test]
    fn an_entry_past_its_threshold_is_stale_and_still_carries_its_value() {
        let held = Held::of(
            EntryKind::LibraryQueryResults,
            some_bytes(),
            written_with_no_skew(1_000),
            None,
        );
        assert_eq!(
            answer(Some(held), at(1_400), None),
            Answer::Stale {
                value: some_bytes(),
                age: Age::Of(Duration::from_secs(400)),
            }
        );
    }

    /// 0043's rule that age never withholds. An entry a month past a five-minute
    /// threshold is served, marked stale, with its age, and there is no variant
    /// for refusing it.
    #[test]
    fn an_entry_far_past_its_threshold_is_still_served() {
        let month = 30 * 24 * 60 * 60;
        let held = Held::of(
            EntryKind::LibraryQueryResults,
            some_bytes(),
            written_with_no_skew(1_000),
            None,
        );
        assert_eq!(
            answer(Some(held), at(1_000 + month), None),
            Answer::Stale {
                value: some_bytes(),
                age: Age::Of(Duration::from_hours(30 * 24)),
            }
        );
    }

    /// The shortening rule reaching an answer rather than only a threshold. The
    /// same entry at the same moment is fresh under the table and stale under
    /// what the server said.
    #[test]
    fn a_server_shortening_a_threshold_turns_a_fresh_entry_stale() {
        let under_the_table = Held::of(
            EntryKind::ItemMetadata,
            some_bytes(),
            written_with_no_skew(1_000),
            None,
        );
        let under_what_the_server_said = Held::of(
            EntryKind::ItemMetadata,
            some_bytes(),
            written_with_no_skew(1_000),
            Some(Duration::from_secs(30)),
        );
        assert_eq!(
            answer(Some(under_the_table), at(1_060), None),
            Answer::Fresh {
                value: some_bytes(),
                age: Age::Of(Duration::from_secs(60)),
            }
        );
        assert_eq!(
            answer(Some(under_what_the_server_said), at(1_060), None),
            Answer::Stale {
                value: some_bytes(),
                age: Age::Of(Duration::from_secs(60)),
            }
        );
    }

    /// A wrong clock makes an entry stale early rather than fresh forever, which
    /// is the direction both guards exist to fix.
    #[test]
    fn an_entry_whose_age_is_unreadable_is_stale_rather_than_fresh() {
        let held = Held::of(
            EntryKind::ArtworkBytes,
            some_bytes(),
            written_with_no_skew(10_000),
            None,
        );
        assert_eq!(
            answer(Some(held), at(9_000), None),
            Answer::Stale {
                value: some_bytes(),
                age: Age::Unreadable(WhyTheAgeIsUnreadable::ItComputedAsNegative),
            }
        );
    }
}
