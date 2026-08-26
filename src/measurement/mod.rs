//! Producing measurements.
//!
//! 0003 puts named spans, their values, the spread across repeated runs, and a
//! statement of what a run did not measure inside the core. The records are
//! 0008, 0061 and 0064, and the issues are #61 through #67.
//!
//! 0061 refused a tracing library and 0064 names the two numbers the core does
//! not report. Both are reasons this module exists as the core's own facility
//! rather than as a seam onto somebody else's.
//!
//! # What a span is, in one paragraph
//!
//! A name from the set below, an identifier meaningful only inside one run, a
//! parent handed over rather than inferred, an interval read from the single
//! injected [`crate::clock::Clocks`] source, and an outcome from three values.
//! It is delivered once, when it closes, to a subscriber supplied when the core
//! was created. Where no subscriber was supplied, opening a span reads no clock,
//! allocates nothing and takes no lock, which is the bound 0061 states in place
//! of a time nobody has measured.
//!
//! # What is deliberately not here
//!
//! A span carries no fields. Not an item identifier, not a server address, not a
//! byte count, not a status code. 0068 places an item identifier under its
//! personal data list, and 0100's rule that an event carries fields exists so
//! that #71 can redact by reading field names; a facility with no fields does not
//! need that rule to reach it. What is given up is attribution, which is what the
//! events in 0100 are for.

use crate::clock::{Clocks, SteadyInstant};
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;

/// Every span name this core emits.
///
/// 0061 requires every name to be declared in one place in the tree rather than
/// written as a literal where it is emitted, for two reasons. #67 has to publish
/// each measurement with the command that produced it, and a set derivable by
/// reading one file is a set a run can print rather than one somebody keeps a
/// list of. And a literal at an emit site is renamed by whoever is working in
/// that file, which turns a renamed span into a number that stops arriving with
/// nothing failing anywhere.
///
/// # What is here today, and what is not
///
/// The three intervals a build is gated on, from 0008. The six sub-intervals
/// that record names in prose - the cache read, the request, the wait for the
/// server, the parse, the artwork fetch and the artwork decode - are not here,
/// because 0061 places the identity of a sub-interval with the issue that builds
/// the subsystem emitting it, in the same way 0100 places the identity of a
/// diagnostic event. Adding one is a variant here and is not a change to any
/// record; renaming one of the three below is a change to 0008, because those
/// three are what a build is gated on and a rename detaches the gate from what it
/// was gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SpanName {
    /// First library query issued to first decoded artwork bitmap returned, with
    /// the store empty.
    FirstTileCoreCold,
    /// The same two endpoints, with the store holding a complete previous answer
    /// for the same query.
    FirstTileCoreWarm,
    /// What-to-play call entered to playable handover returned.
    PlayCore,
}

impl SpanName {
    /// Every name this core declares.
    ///
    /// Here so that #67 can print the set rather than keep a copy of it, and so
    /// that the rule below is applied to the whole of it rather than to whichever
    /// members somebody remembered.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::FirstTileCoreCold,
            Self::FirstTileCoreWarm,
            Self::PlayCore,
        ]
    }

    /// The name as it is reported.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FirstTileCoreCold => "first-tile.core.cold",
            Self::FirstTileCoreWarm => "first-tile.core.warm",
            Self::PlayCore => "play.core",
        }
    }
}

/// Whether a name obeys 0061's spelling rule.
///
/// Lower case and dotted. Written as a function rather than as a comment so that
/// the compiler is not the only thing between a new variant and a name nobody
/// can place, and so the rule can be proven against text it controls rather than
/// only against the set this tree happens to hold today.
///
/// What it cannot ask is whether a name means what it says. That is a judgement
/// and no reading of this file makes it.
#[must_use]
pub fn is_a_well_formed_span_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut segments = 0;
    for segment in name.split('.') {
        segments += 1;
        if segment.is_empty() {
            return false;
        }
        for byte in segment.bytes() {
            let allowed = byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
            if !allowed {
                return false;
            }
        }
        if segment.starts_with('-') || segment.ends_with('-') {
            return false;
        }
    }
    segments >= 1
}

/// Whether one name is a sub-interval of another, by 0061's nesting rule.
///
/// A sub-interval's name begins with the name of the interval it sits inside,
/// so the set reads as the nesting does and a reader holding a name knows which
/// of the three numbers it is inside without a table.
///
/// The separator is required rather than a plain prefix test, which is the
/// one-character mistake this function exists to not make: `play.core-warm` is
/// not inside `play.core`.
#[must_use]
pub fn is_inside(name: &str, outer: &str) -> bool {
    match name.strip_prefix(outer) {
        Some(rest) => rest.starts_with('.') && rest.len() > 1,
        None => false,
    }
}

/// The identifier a client's own half of a published number joins to.
///
/// From 0061: unique inside one run of the core and meaningless outside it. It
/// is allocated in sequence from a counter the core owns rather than drawn from a
/// space wide enough to be unique everywhere, because a globally unique
/// identifier is stable enough to correlate two reports from one device and 0068
/// places that kind of correlation outside what the core hands anybody. A counter
/// that starts again with the process cannot do it, and it does everything 0008
/// asks of the identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpanId(u64);

impl SpanId {
    /// The number, for a client that has to put it in the report 0008 describes.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// How a span ended.
///
/// Three values and no fourth. 0009 already requires a cancelled call to be
/// distinct from every failure on the way out of the core, and #53's wall of two
/// hundred tiles is where most spans are withdrawn, so two values would put the
/// ordinary case in the same bucket as a decode that went wrong.
///
/// A failed span carries no kind from 0004. The kind is already on the failure
/// the caller received and on the event 0100 emits for it, and a third copy is a
/// third thing to keep in step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanOutcome {
    /// The work the span measured finished.
    Completed,
    /// The work failed.
    Failed,
    /// The work was withdrawn before it finished.
    Cancelled,
}

/// A span that has closed, as it reaches a subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedSpan {
    name: SpanName,
    id: SpanId,
    parent: Option<SpanId>,
    interval: Duration,
    outcome: SpanOutcome,
}

impl ClosedSpan {
    /// Which interval this is.
    #[must_use]
    pub const fn name(&self) -> SpanName {
        self.name
    }

    /// The identifier a client's half joins to.
    #[must_use]
    pub const fn id(&self) -> SpanId {
        self.id
    }

    /// The span this one sat inside, where it sat inside one.
    #[must_use]
    pub const fn parent(&self) -> Option<SpanId> {
        self.parent
    }

    /// How long it took, on the steady clock.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// How it ended.
    #[must_use]
    pub const fn outcome(&self) -> SpanOutcome {
        self.outcome
    }
}

/// The place a client receives the core's measurements.
///
/// A second client-supplied interface beside [`crate::diagnostics::DiagnosticsSink`],
/// which 0061 decided rather than folding measurement into the event stream.
///
/// Thread safety, from 0009 through 0100, which states it once for both
/// facilities: it is called on the thread the work happened on, the core holds
/// no lock of its own across the call, nothing is retained, and a subscriber that
/// blocks blocks a lane.
///
/// It is handed a span once, when the span closes. A subscriber that saw opens
/// would have to hold every open span to pair it with its close, which is state
/// with a bound somebody has to choose; delivering once means the subscriber
/// holds nothing.
pub trait MeasurementSink: Send + Sync {
    /// A span has closed.
    fn span_closed(&self, span: &ClosedSpan);
}

/// The facility every subsystem opens a span on.
///
/// Thread safety, from 0009: safe from any thread. Every lane opens and closes
/// spans, and a facility that were not would make measurement a synchronisation
/// point in the code it is measuring.
///
/// Where the subscriber comes from is 0100's answer, which is where the core is
/// created in #115, and it is not changed afterwards. That is not a convenience:
/// because it cannot change, the answer to "is anybody listening" at open is the
/// answer at close, so there is no span opened without a clock reading and closed
/// with one.
pub struct Measurement<'a> {
    clocks: &'a dyn Clocks,
    subscriber: Option<&'a dyn MeasurementSink>,
    next_id: AtomicU64,
}

/// Written out rather than derived, because neither of the two references is to
/// a type this crate can require `Debug` of: both are supplied by a client, and
/// asking a client's keychain or metrics object to be printable would be this
/// facility deciding something about a client's own types. What is printed is
/// what this facility knows about itself.
impl core::fmt::Debug for Measurement<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Measurement")
            .field("is_measuring", &self.is_measuring())
            .finish_non_exhaustive()
    }
}

impl<'a> Measurement<'a> {
    /// The facility, with the source it reads and the subscriber it delivers to.
    ///
    /// `None` is the ordinary case rather than a degraded one: a client that
    /// supplied no subscriber gets a core that works and measures nothing, and
    /// the section below on what that costs is the whole of the difference.
    #[must_use]
    pub const fn new(clocks: &'a dyn Clocks, subscriber: Option<&'a dyn MeasurementSink>) -> Self {
        Self {
            clocks,
            subscriber,
            next_id: AtomicU64::new(1),
        }
    }

    /// Whether anything is listening.
    #[must_use]
    pub const fn is_measuring(&self) -> bool {
        self.subscriber.is_some()
    }

    /// Opens a span, inside `parent` where it has one.
    ///
    /// WITH NO SUBSCRIBER THIS TESTS ONE REFERENCE AND DOES NOTHING ELSE. No
    /// clock is read, nothing is allocated, and no identifier is drawn. That is
    /// 0061's bound, and it is the part that is easy to lose: the natural
    /// implementation opens the span, reads the clock, and decides at close
    /// whether to hand anything over, which still reads a clock twice per span,
    /// six sub-intervals deep, across two hundred tiles.
    ///
    /// The parent is handed over and never inferred. There is no ambient
    /// context, no thread-local carrier, and nothing worked out from where the
    /// code is running, because 0009 moves work between two lanes by design and a
    /// context not carried across one of those handovers is wrong at exactly the
    /// handovers the sub-intervals exist to measure. What that costs is that a
    /// subsystem which cannot reach the parent cannot open a child, which is the
    /// case worth meeting rather than hiding.
    pub fn open(&self, name: SpanName, parent: Option<SpanId>) -> Span<'_, 'a> {
        let started = if self.subscriber.is_some() {
            Some((
                SpanId(self.next_id.fetch_add(1, Ordering::Relaxed)),
                self.clocks.steady(),
            ))
        } else {
            None
        };
        Span {
            facility: self,
            name,
            parent,
            started,
        }
    }
}

/// A span that is open.
///
/// A span dropped without being closed delivers nothing, which is 0061's rule
/// for a span still open when the core has stopped: 0009 has the stop call cancel
/// every outstanding call first, so the ordinary path closes those spans as
/// cancelled and delivers them, and anything still open after that was not closed
/// by the work it was measuring. An interval ended by a stop is a measurement of
/// the stop.
pub struct Span<'facility, 'a> {
    facility: &'facility Measurement<'a>,
    name: SpanName,
    parent: Option<SpanId>,
    started: Option<(SpanId, SteadyInstant)>,
}

/// Written out for the reason [`Measurement`]'s is.
impl core::fmt::Debug for Span<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Span")
            .field("name", &self.name)
            .field("id", &self.id())
            .field("parent", &self.parent)
            .finish_non_exhaustive()
    }
}

impl Span<'_, '_> {
    /// The identifier, where one was drawn.
    ///
    /// `None` where no subscriber was supplied, because no identifier is drawn
    /// for a span nothing will receive. A client asking for the join in 0008 is a
    /// client that supplied a subscriber.
    #[must_use]
    pub const fn id(&self) -> Option<SpanId> {
        match self.started {
            Some((id, _)) => Some(id),
            None => None,
        }
    }

    /// Which interval this is.
    #[must_use]
    pub const fn name(&self) -> SpanName {
        self.name
    }

    /// Closes the span and delivers it, once.
    ///
    /// With no subscriber this reads no clock and delivers nothing, for the
    /// reason [`Measurement::open`] gives.
    pub fn close(self, outcome: SpanOutcome) {
        let (Some((id, started)), Some(subscriber)) = (self.started, self.facility.subscriber)
        else {
            return;
        };
        let closed = ClosedSpan {
            name: self.name,
            id,
            parent: self.parent,
            interval: self.facility.clocks.steady().interval_since(started),
            outcome,
        };
        subscriber.span_closed(&closed);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClosedSpan, Measurement, MeasurementSink, SpanName, SpanOutcome,
        is_a_well_formed_span_name, is_inside,
    };
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use std::sync::Mutex;

    /// The controlled source 0102 permits the suite: `steady` and `elapsed` may
    /// be advanced by any amount and independently of each other, `wall` may be
    /// set freely in both directions, and neither monotonic clock may be moved
    /// backwards.
    ///
    /// It counts its own readings, which is what makes the bound in 0061 an
    /// assertion rather than a claim.
    struct ControlledClocks {
        state: Mutex<ControlledState>,
    }

    struct ControlledState {
        steady_nanos: u64,
        elapsed_nanos: u64,
        wall_seconds: i64,
        readings: usize,
    }

    impl ControlledClocks {
        fn new() -> Self {
            Self {
                state: Mutex::new(ControlledState {
                    steady_nanos: 0,
                    elapsed_nanos: 0,
                    wall_seconds: 0,
                    readings: 0,
                }),
            }
        }

        fn advance_steady(&self, nanos: u64) {
            let mut state = self.held();
            state.steady_nanos += nanos;
        }

        /// The refusal 0102 asks the fake to make, in the only shape a fake can
        /// make it. The shipping code has no branch for a monotonic clock going
        /// backwards, so a suite able to move one would be proving behaviour that
        /// nothing ships.
        fn refuse_to_move_steady_backwards(&self, to_nanos: u64) -> bool {
            let state = self.held();
            to_nanos >= state.steady_nanos
        }

        fn set_wall_seconds(&self, seconds: i64) {
            let mut state = self.held();
            state.wall_seconds = seconds;
        }

        fn readings(&self) -> usize {
            self.held().readings
        }

        fn held(&self) -> std::sync::MutexGuard<'_, ControlledState> {
            self.state
                .lock()
                .expect("the fixture holds no poisoned lock")
        }
    }

    impl Clocks for ControlledClocks {
        fn steady(&self) -> SteadyInstant {
            let mut state = self.held();
            state.readings += 1;
            SteadyInstant::from_nanos(state.steady_nanos)
        }

        fn elapsed(&self) -> ElapsedInstant {
            let mut state = self.held();
            state.readings += 1;
            ElapsedInstant::from_nanos(state.elapsed_nanos)
        }

        fn wall(&self) -> WallMoment {
            let mut state = self.held();
            state.readings += 1;
            WallMoment::from_epoch(state.wall_seconds, 0)
        }
    }

    /// A subscriber that keeps what it was handed, so that a test can read what
    /// the core delivered rather than what it was asked to deliver.
    struct Collected {
        spans: Mutex<Vec<ClosedSpan>>,
    }

    impl Collected {
        fn new() -> Self {
            Self {
                spans: Mutex::new(Vec::new()),
            }
        }

        fn taken(&self) -> Vec<ClosedSpan> {
            self.spans
                .lock()
                .expect("the fixture holds no poisoned lock")
                .clone()
        }
    }

    impl MeasurementSink for Collected {
        fn span_closed(&self, span: &ClosedSpan) {
            self.spans
                .lock()
                .expect("the fixture holds no poisoned lock")
                .push(*span);
        }
    }

    #[test]
    fn every_declared_name_obeys_the_spelling_rule() {
        for name in SpanName::all() {
            assert!(
                is_a_well_formed_span_name(name.as_str()),
                "{} is not a well formed span name",
                name.as_str()
            );
        }
    }

    #[test]
    fn every_declared_name_is_declared_once() {
        let mut seen: Vec<&str> = SpanName::all().iter().map(|name| name.as_str()).collect();
        let declared = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), declared, "two variants report one name");
    }

    /// The rule judged against text it controls rather than only against the set
    /// this tree happens to hold, which would prove the state of the tree on the
    /// day it ran.
    #[test]
    fn the_spelling_rule_reads_a_name_and_bites_the_near_misses() {
        assert!(is_a_well_formed_span_name("play.core"));
        assert!(is_a_well_formed_span_name("first-tile.core.cold"));
        assert!(!is_a_well_formed_span_name("Play.core"), "upper case");
        assert!(!is_a_well_formed_span_name("play_core"), "an underscore");
        assert!(
            !is_a_well_formed_span_name("play..core"),
            "an empty segment"
        );
        assert!(!is_a_well_formed_span_name(".play"), "a leading separator");
        assert!(!is_a_well_formed_span_name("play."), "a trailing separator");
        assert!(!is_a_well_formed_span_name("play core"), "a space");
        assert!(!is_a_well_formed_span_name(""), "nothing at all");
    }

    /// The one-character mistake the nesting rule exists to not make.
    #[test]
    fn nesting_is_read_at_the_separator_and_not_at_the_prefix() {
        assert!(is_inside("play.core.parse", "play.core"));
        assert!(!is_inside("play.core-warm", "play.core"), "a plain prefix");
        assert!(
            !is_inside("play.core", "play.core"),
            "a name is not inside itself"
        );
        assert!(
            !is_inside("play.core.", "play.core"),
            "a separator and nothing after it"
        );
        assert!(!is_inside("first-tile.core.cold", "play.core"));
    }

    #[test]
    fn a_span_reports_the_interval_the_controlled_clock_was_moved_by() {
        let clocks = ControlledClocks::new();
        let collected = Collected::new();
        let measurement = Measurement::new(&clocks, Some(&collected));

        let span = measurement.open(SpanName::PlayCore, None);
        clocks.advance_steady(1_750_000);
        span.close(SpanOutcome::Completed);

        let taken = collected.taken();
        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].name(), SpanName::PlayCore);
        assert_eq!(taken[0].interval().as_nanos(), 1_750_000);
        assert_eq!(taken[0].outcome(), SpanOutcome::Completed);
        assert_eq!(taken[0].parent(), None);
    }

    #[test]
    fn a_child_carries_the_parent_it_was_handed() {
        let clocks = ControlledClocks::new();
        let collected = Collected::new();
        let measurement = Measurement::new(&clocks, Some(&collected));

        let outer = measurement.open(SpanName::FirstTileCoreCold, None);
        let outer_id = outer.id().expect("a subscriber was supplied");
        let inner = measurement.open(SpanName::FirstTileCoreWarm, Some(outer_id));
        clocks.advance_steady(5);
        inner.close(SpanOutcome::Cancelled);
        clocks.advance_steady(5);
        outer.close(SpanOutcome::Completed);

        let taken = collected.taken();
        assert_eq!(taken.len(), 2);
        assert_eq!(taken[0].parent(), Some(outer_id));
        assert_eq!(taken[0].outcome(), SpanOutcome::Cancelled);
        assert_eq!(taken[0].interval().as_nanos(), 5);
        assert_eq!(taken[1].parent(), None);
        assert_eq!(taken[1].interval().as_nanos(), 10);
    }

    #[test]
    fn identifiers_are_drawn_in_sequence_and_do_not_repeat() {
        let clocks = ControlledClocks::new();
        let collected = Collected::new();
        let measurement = Measurement::new(&clocks, Some(&collected));

        let first = measurement.open(SpanName::PlayCore, None);
        let second = measurement.open(SpanName::PlayCore, None);
        assert_ne!(first.id(), second.id());
        assert_eq!(
            second.id().expect("a subscriber was supplied").get(),
            first.id().expect("a subscriber was supplied").get() + 1
        );
    }

    #[test]
    fn a_span_that_is_dropped_rather_than_closed_delivers_nothing() {
        let clocks = ControlledClocks::new();
        let collected = Collected::new();
        let measurement = Measurement::new(&clocks, Some(&collected));

        {
            let _abandoned = measurement.open(SpanName::PlayCore, None);
        }

        assert!(collected.taken().is_empty());
    }

    /// The bound 0061 states, as far as a clock can assert it: with no subscriber
    /// supplied, no reading is taken. This is the assertion that goes red if
    /// somebody moves the decision from open to close, which is the natural
    /// implementation and the one the record names.
    #[test]
    fn with_no_subscriber_no_clock_is_read() {
        let clocks = ControlledClocks::new();
        let measurement = Measurement::new(&clocks, None);
        assert!(!measurement.is_measuring());

        for _ in 0..200 {
            let span = measurement.open(SpanName::FirstTileCoreCold, None);
            let child = measurement.open(SpanName::FirstTileCoreWarm, span.id());
            child.close(SpanOutcome::Cancelled);
            span.close(SpanOutcome::Completed);
        }

        assert_eq!(clocks.readings(), 0);
    }

    /// The same wall, with a subscriber, so that the assertion above is about the
    /// absent subscriber rather than about a facility that never reads a clock.
    #[test]
    fn with_a_subscriber_every_span_reads_the_clock_twice() {
        let clocks = ControlledClocks::new();
        let collected = Collected::new();
        let measurement = Measurement::new(&clocks, Some(&collected));

        for _ in 0..200 {
            let span = measurement.open(SpanName::FirstTileCoreCold, None);
            span.close(SpanOutcome::Completed);
        }

        assert_eq!(clocks.readings(), 400);
        assert_eq!(collected.taken().len(), 200);
    }

    #[test]
    fn with_no_subscriber_no_identifier_is_drawn() {
        let clocks = ControlledClocks::new();
        let measurement = Measurement::new(&clocks, None);
        assert_eq!(measurement.open(SpanName::PlayCore, None).id(), None);
    }

    #[test]
    fn the_controlled_clock_refuses_to_be_moved_backwards() {
        let clocks = ControlledClocks::new();
        clocks.advance_steady(10);
        assert!(clocks.refuse_to_move_steady_backwards(10));
        assert!(clocks.refuse_to_move_steady_backwards(11));
        assert!(!clocks.refuse_to_move_steady_backwards(9));
    }

    #[test]
    fn the_controlled_wall_clock_moves_in_both_directions() {
        let clocks = ControlledClocks::new();
        clocks.set_wall_seconds(1_700_000_000);
        assert_eq!(clocks.wall().seconds_from_the_epoch(), 1_700_000_000);
        clocks.set_wall_seconds(-1);
        assert_eq!(clocks.wall().seconds_from_the_epoch(), -1);
    }
}
