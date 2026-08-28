//! What the core tells a client about itself.
//!
//! This is not one of the six things 0003 names either. It is here because 0009
//! states a thread rule for the sink a client supplies, and a rule with no name
//! to attach to is a rule a reader meets nowhere. The record is 0100 and the
//! issue is #100. What may leave through an event is 0071 and #71.
//!
//! # What an event is, in one paragraph
//!
//! A severity from four values, an identity that is a name rather than a number,
//! fields as name and value pairs of data, and a moment read on `wall` through
//! the single injected [`crate::clock::Clocks`] source. Never a sentence written
//! for a person: 0003 gives the wording to the client and 0004 states the same
//! rule for errors, so a client owns it in both places rather than one of them.
//!
//! # What is deliberately not here
//!
//! No stack, no thread identity, no process detail. Those describe a platform
//! the core does not know, and a client that wants them adds them at the sink,
//! where the platform is known.
//!
//! No retention. 0068 promises an operator that an event is handed over and
//! forgotten in the same call, so this facility holds no ring buffer, no file and
//! no history, and [`Event`] borrows its fields rather than owning them: a type
//! that cannot outlive the call cannot be kept by accident.
//!
//! No set of identities declared centrally. 0100 places the identity of an event
//! with the issue that builds the subsystem emitting it, which is the opposite of
//! 0061's rule for span names and is decided that way in both records. What this
//! module owes instead is that a name is well formed, and
//! [`EventName::declared`] is where the compiler refuses one that is not.
//!
//! # Which events exist today
//!
//! None. Nothing in this tree emits one, because no subsystem that would is
//! built. That is a statement about this tree rather than about the interface,
//! and the same sentence stands over [`crate::session`] for its own reason.

pub mod redaction;

use crate::clock::{Clocks, WallMoment};
use core::sync::atomic::{AtomicU8, Ordering};
use core::time::Duration;
use redaction::{Correlator, CorrelatorSalt, FieldName, Treatment};

/// How much of what happened is worth somebody's attention.
///
/// Four values and no fifth. The set is closed for the reason 0004's fifteen
/// error kinds are closed: a client's filter is written against these
/// exhaustively, so a fifth is a change to 0100 and to every client that
/// filters, which is the cost that stops the set growing by accident.
///
/// The order is from the most severe to the least, so that a level compares
/// against an event with an ordinary `<=` and the comparison reads the way the
/// values are written. Severity graded on an open scale of numbers is what 0100
/// refuses, and the ordering here is not that: it is the order of four named
/// values rather than a scale a subsystem picks a point on.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// A defect in the core, the same subject as `internal-fault` in 0004.
    /// Nothing about a server or a network is being claimed, and the event is
    /// worth sending to this repository.
    Fault,
    /// Something the core was asked to do that did not happen. A kind from 0004
    /// was returned to the caller for it, and the event says which, so that a
    /// report and the client's own error handling describe one occurrence rather
    /// than two.
    Failure,
    /// Something happened, nothing failed, and somebody supporting an
    /// installation would want it in front of them.
    Notice,
    /// Everything else. The level somebody turns on to answer a question and off
    /// afterwards, and a shipped client is expected to be running with it off.
    Detail,
}

impl Severity {
    /// Every severity, most severe first.
    ///
    /// Here so that a client building a filter reads the set out of the crate
    /// rather than keeping a copy of it, and so the tests below apply a rule to
    /// the whole of it rather than to whichever members somebody remembered.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Fault, Self::Failure, Self::Notice, Self::Detail]
    }

    /// The severity as it is reported.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fault => "fault",
            Self::Failure => "failure",
            Self::Notice => "notice",
            Self::Detail => "detail",
        }
    }

    /// The value as it is held in the level, which has to be a number to be
    /// changed while the core runs without a lock.
    const fn as_number(self) -> u8 {
        match self {
            Self::Fault => 0,
            Self::Failure => 1,
            Self::Notice => 2,
            Self::Detail => 3,
        }
    }

    /// The value the number came from.
    ///
    /// The last arm is unreachable rather than a default: the only writer is
    /// [`Severity::as_number`] above, and a number outside the four is a defect
    /// in this file rather than an input. It answers `Detail` so that a core
    /// which somehow held one reports more rather than falling silent.
    const fn from_number(number: u8) -> Self {
        match number {
            0 => Self::Fault,
            1 => Self::Failure,
            2 => Self::Notice,
            _ => Self::Detail,
        }
    }
}

/// Whether a name is one an event may be reported under.
///
/// Lower case, dotted, and the same spelling 0061 fixes for a span name. Written
/// as a function so that the rule is applied to text rather than kept as a
/// sentence somebody follows, and it is `const` so that
/// [`EventName::declared`] can refuse a malformed name where it is written
/// rather than where it is emitted.
///
/// What it cannot ask is whether a name means what it says, or whether two
/// subsystems chose the same one. Both are judgements and no reading of this
/// file makes them.
#[must_use]
pub const fn is_a_well_formed_event_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut at = 0;
    let mut segment_length = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte == b'.' {
            if segment_length == 0 || bytes[at - 1] == b'-' {
                return false;
            }
            segment_length = 0;
        } else {
            let allowed = byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
            if !allowed {
                return false;
            }
            if byte == b'-' && segment_length == 0 {
                return false;
            }
            segment_length += 1;
        }
        at += 1;
    }
    segment_length != 0 && bytes[bytes.len() - 1] != b'-'
}

/// A stable name for what happened.
///
/// 0100 fixes three things about it. It is decided once by whichever subsystem
/// emits the event, it is never renamed without a record, and it is never a
/// number, because it is what a client filters on, what a report is grouped by,
/// and what somebody searches for when the same thing has been reported twice.
///
/// It is a name and not a variant of an enumeration in this module on purpose,
/// and that is 0100 differing from 0061 rather than this file being careless: a
/// central set would put every subsystem's identities in one file that every
/// subsystem edits, and the record places each identity with the issue that
/// builds the thing emitting it.
///
/// WHAT KEEPS IT FROM BEING A LITERAL ANYWAY IS THE CONSTRUCTOR. A name is
/// `&'static str`, so it cannot be assembled out of anything a request or a
/// person supplied, and [`EventName::declared`] is a `const fn` that refuses a
/// malformed one at compile time when the name is written as a constant, which
/// is how a subsystem is meant to write it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventName(&'static str);

impl EventName {
    /// Declares an identity.
    ///
    /// # Panics
    ///
    /// Where the name does not obey [`is_a_well_formed_event_name`]. A subsystem
    /// writes its identities as constants:
    ///
    /// ```
    /// use flowfin_core::diagnostics::EventName;
    ///
    /// const SERVER_DECLARED_UNREACHABLE: EventName =
    ///     EventName::declared("server.declared-unreachable");
    /// ```
    ///
    /// and a constant is evaluated when it is compiled, so a malformed name is a
    /// build that stops rather than an event nobody can filter on. Called with a
    /// name computed at run time it panics at run time instead, which is why the
    /// argument is `&'static str`: there is nothing to compute one from.
    #[must_use]
    pub const fn declared(name: &'static str) -> Self {
        assert!(
            is_a_well_formed_event_name(name),
            "an event identity is lower case, dotted, and never a number"
        );
        Self(name)
    }

    /// The name as it is reported.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// What a field carries.
///
/// The shapes of data 0100 names: a count, an identifier, a duration, a kind
/// from 0004, an address. Four variants rather than five, because an identifier,
/// a kind and an address are all text the core already holds and nothing here
/// treats them differently.
///
/// THAT IS NOT THE REDACTION RULE AND DOES NOT WEAKEN IT. 0100 requires the rule
/// in 0071 to decide per field NAME, because 0068 places a field carrying an
/// item identifier under its personal data list while the count beside it is
/// not, so what the rule reads is the name and never the variant. #71 is where
/// that rule is built and this module decides none of it.
///
/// A sentence written out for a person is not among them and is the thing this
/// type exists to make unwritable. A value with the numbers already substituted
/// into it cannot be redacted by name, cannot be translated, and is wording the
/// core gave away in 0003. The word 0100 uses for that shape is one the
/// `no-view-vocabulary` rule in `.github/invariants/rules` refuses anywhere
/// under `src/`, which is that rule biting a comment rather than a crossing, and
/// the sentence is written around it rather than the rule being widened.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValue<'a> {
    /// How many of something.
    Count(u64),
    /// How long something took. The clock it was measured on is 0102's table
    /// rather than this field's.
    Interval(Duration),
    /// Text the core already holds: an identifier, an error kind, an address.
    Text(&'a str),
    /// Something that either was or was not.
    Truth(bool),
}

/// One name and value pair on an event.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field<'a> {
    name: FieldName,
    value: FieldValue<'a>,
}

impl<'a> Field<'a> {
    /// A field.
    ///
    /// THE NAME CARRIES ITS TREATMENT AND THIS SIGNATURE IS WHERE THAT IS
    /// ENFORCED. A [`FieldName`] is built through one of three calls, each of
    /// which is one of 0071's three treatments, so a field whose treatment
    /// nobody chose cannot be written and the compiler refuses it rather than a
    /// review. 0071 states that default as a rule - a field nobody has
    /// classified is excluded - and this is the stronger form of it, because a
    /// default that cannot be reached cannot fall the wrong way.
    ///
    /// The name is still `&'static str` inside, for the reason an identity is:
    /// it is written at the emit site and is never assembled out of what arrived
    /// from somewhere, so what the rule reads is a fixed name rather than
    /// whatever a server put in an answer.
    #[must_use]
    pub const fn new(name: FieldName, value: FieldValue<'a>) -> Self {
        Self { name, value }
    }

    /// The name, with the treatment 0071's rule gives it.
    #[must_use]
    pub const fn name(&self) -> FieldName {
        self.name
    }

    /// The value.
    #[must_use]
    pub const fn value(&self) -> FieldValue<'a> {
        self.value
    }
}

/// One thing the core has to say, as it is handed to a client.
///
/// It borrows its fields, so nothing here is allocated and a sink cannot keep
/// one past the call it arrived in. That is 0068's retention promise expressed
/// as a type rather than as a sentence a sink is asked to honour.
///
/// Thread safety, from 0009: safe from any thread, and it is handed to a sink on
/// whichever lane produced it.
#[derive(Debug, Clone, Copy)]
pub struct Event<'a> {
    severity: Severity,
    name: EventName,
    fields: &'a [Field<'a>],
    moment: WallMoment,
}

impl<'a> Event<'a> {
    /// How much attention it is worth.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// What happened.
    #[must_use]
    pub const fn name(&self) -> EventName {
        self.name
    }

    /// The fields, in the order the emitting subsystem wrote them.
    #[must_use]
    pub const fn fields(&self) -> &'a [Field<'a>] {
        self.fields
    }

    /// When it happened, on `wall`.
    ///
    /// 0100 carries it for one purpose, which is lining an event up against a
    /// server's own log. The core never reads it back and no core behaviour
    /// depends on it, so a device with a wrong clock produces events that are
    /// hard to correlate rather than events that are wrong.
    #[must_use]
    pub const fn moment(&self) -> WallMoment {
        self.moment
    }
}

/// The place a client receives the core's diagnostic events.
///
/// Thread safety, from 0009: may be called from any lane, at any time, and
/// concurrently. It must be safe for that, it must not block, and it must not
/// call back into the core. The last of the three is the deadlock, and what the
/// interface can do about it is hand over a borrowed [`Event`] and no way back
/// to the facility; the rest is the rule rather than a mechanism.
///
/// It is handed an event once. Nothing is retained by the core, and a sink that
/// wants a history keeps its own.
///
/// What may appear in one at all is 0071 and #71. Nothing here decides it.
pub trait DiagnosticsSink: Send + Sync {
    /// An event happened.
    fn event(&self, event: &Event<'_>);
}

/// The facility every subsystem reports through.
///
/// Thread safety, from 0009: safe from any thread. Every lane emits, and a
/// facility that were not would make reporting a synchronisation point in the
/// code it is reporting on.
///
/// Where the sink comes from is 0100's answer, which is where the core is
/// created in #115, and it is not changed afterwards. The LEVEL is the one thing
/// that does move while the core runs, because turning `detail` on to answer a
/// question and off again is the only reason anybody touches this at all.
pub struct Diagnostics<'a> {
    clocks: &'a dyn Clocks,
    sink: Option<&'a dyn DiagnosticsSink>,
    /// What a reduced field is correlated under, for the life of this facility.
    /// 0071 has it created when the core is created and never changed, so it is
    /// held by value and there is no call that moves it.
    salt: CorrelatorSalt,
    /// Held as a number rather than as a [`Severity`] so that it can be changed
    /// from any thread without a lock. Every value written here comes from
    /// [`Severity::as_number`].
    level: AtomicU8,
}

/// Written out rather than derived, for the reason [`crate::measurement`] gives:
/// neither reference is to a type this crate can require `Debug` of, because both
/// are supplied by a client. What is printed is what this facility knows about
/// itself.
impl core::fmt::Debug for Diagnostics<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Diagnostics")
            .field("has_a_sink", &self.sink.is_some())
            .field("level", &self.level())
            .finish_non_exhaustive()
    }
}

impl<'a> Diagnostics<'a> {
    /// The facility, with the clock source it reads and the sink it delivers to.
    ///
    /// `None` is the ordinary case rather than a degraded one: a client that
    /// supplied no sink gets a core that works and says nothing.
    #[must_use]
    pub const fn new(
        clocks: &'a dyn Clocks,
        sink: Option<&'a dyn DiagnosticsSink>,
        level: Severity,
        salt: CorrelatorSalt,
    ) -> Self {
        Self {
            clocks,
            sink,
            salt,
            level: AtomicU8::new(level.as_number()),
        }
    }

    /// The level below which nothing is produced.
    #[must_use]
    pub fn level(&self) -> Severity {
        Severity::from_number(self.level.load(Ordering::Relaxed))
    }

    /// Moves the level, while the core is running.
    ///
    /// A call that cannot wait, in the terms of 0009. `Relaxed` is what the
    /// ordering has to be: the level is one value nothing else is published
    /// with, and an event emitted in the same instant the level moved may be
    /// judged against either value. What is not admitted is a torn read, and an
    /// atomic is what removes that.
    pub fn set_level(&self, level: Severity) {
        self.level.store(level.as_number(), Ordering::Relaxed);
    }

    /// Whether an event of this severity would reach anybody.
    ///
    /// Here so that a subsystem with an expensive field to assemble can ask
    /// before assembling it, rather than assembling it for a facility that will
    /// drop it.
    #[must_use]
    pub fn is_reporting(&self, severity: Severity) -> bool {
        self.sink.is_some() && severity <= self.level()
    }

    /// Reports one event.
    ///
    /// WITH NO SINK THIS TESTS ONE REFERENCE AND DOES NOTHING ELSE. No clock is
    /// read and nothing is allocated, which is the bound 0100 states in place of
    /// a time nobody has measured. It is the part that is easy to lose: the
    /// natural implementation reads the clock to build the event and then finds
    /// there is nobody to hand it to.
    ///
    /// Below the level the same holds, so a client running with `detail` off
    /// pays the same as a client with no sink for every `detail` event in the
    /// core.
    pub fn emit(&self, severity: Severity, name: EventName, fields: &[Field<'_>]) {
        let Some(sink) = self.sink else {
            return;
        };
        if severity > self.level() {
            return;
        }
        let moment = self.clocks.wall();

        // The ordinary event carries nothing 0071 touches, and it reaches the
        // sink as it was written. This is not an exception to the rule: it is
        // the rule finding nothing to do, and the branch exists so that an event
        // of counts and intervals costs no allocation for a redaction that would
        // change none of its fields.
        if fields
            .iter()
            .all(|field| field.name().treatment() == Treatment::CarriedWhole)
        {
            sink.event(&Event {
                severity,
                name,
                fields,
                moment,
            });
            return;
        }

        // Every reduced value first, one entry per field so the two sequences
        // line up by position, because the correlators have to outlive the
        // borrow the event takes of them. Position rather than a running count
        // so that there is no arm here that cannot be reached: a field with a
        // correlator beside it is the reduced one, and the second pass keeps the
        // order the emitting subsystem wrote, which 0100 states as a property of
        // an event rather than as a convenience.
        let correlators: Vec<Option<Correlator>> = fields
            .iter()
            .map(|field| match field.name().treatment() {
                Treatment::Reduced => Some(Correlator::of(&self.salt, field.value())),
                Treatment::Excluded | Treatment::CarriedWhole => None,
            })
            .collect();

        let mut kept = Vec::with_capacity(fields.len());
        for (field, correlator) in fields.iter().zip(&correlators) {
            if let Some(correlator) = correlator {
                kept.push(Field::new(
                    field.name(),
                    FieldValue::Text(correlator.as_str()),
                ));
            } else if field.name().treatment() == Treatment::CarriedWhole {
                kept.push(*field);
            }
        }

        sink.event(&Event {
            severity,
            name,
            fields: &kept,
            moment,
        });
    }
}

#[cfg(test)]
mod tests {
    //! The collector 0100 asks the suite for, and what it is used to prove.
    //!
    //! It keeps what it is handed in a vector under a lock, which is what a
    //! client that wanted a history would write, and it is behind `#[cfg(test)]`
    //! so nothing a client links can reach it. What it cannot keep is an
    //! [`Event`] itself, because that borrows its fields for the call: the
    //! collector copies out what it needs, which is 0068's retention rule met by
    //! a type rather than by a promise.
    //!
    //! The clock source counts its own readings, so that "no clock is read" is a
    //! measurement here rather than a claim. The allocation half of the bound
    //! needs an allocator the crate cannot hold and is in
    //! `tests/an_event_costs_nothing_when_nobody_is_listening.rs`.

    use super::redaction::{CORRELATOR_WIDTH, CorrelatorSalt, FieldName, Treatment};
    use super::{
        Diagnostics, DiagnosticsSink, Event, EventName, Field, FieldValue, Severity,
        is_a_well_formed_event_name,
    };
    use crate::clock::{Clocks, ElapsedInstant, SteadyInstant, WallMoment};
    use core::sync::atomic::{AtomicUsize, Ordering};
    use core::time::Duration;
    use std::sync::Mutex;

    /// A salt for the suite, fixed so that a correlator is the same on every run
    /// of a case. That is the opposite of what a real one is for, which is why
    /// this is a fixture rather than something a client would write: 0071's
    /// property is that a correlator means nothing outside the run that produced
    /// it, and a case asserting a correlator has to be able to name one.
    fn a_salt() -> CorrelatorSalt {
        CorrelatorSalt::from_bytes([0x5a; CorrelatorSalt::WIDTH])
    }

    /// A clock source that answers the same moment every time and counts how
    /// often it was asked.
    struct CountingClocks {
        wall_readings: AtomicUsize,
    }

    impl CountingClocks {
        const fn new() -> Self {
            Self {
                wall_readings: AtomicUsize::new(0),
            }
        }

        fn wall_readings(&self) -> usize {
            self.wall_readings.load(Ordering::Relaxed)
        }
    }

    impl Clocks for CountingClocks {
        fn steady(&self) -> SteadyInstant {
            SteadyInstant::from_nanos(7)
        }

        fn elapsed(&self) -> ElapsedInstant {
            ElapsedInstant::from_nanos(7)
        }

        fn wall(&self) -> WallMoment {
            self.wall_readings.fetch_add(1, Ordering::Relaxed);
            WallMoment::from_epoch(1_700_000_000, 5)
        }
    }

    /// What one event looked like to the sink, copied out of the borrowed value.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Collected {
        severity: Severity,
        name: &'static str,
        fields: Vec<(&'static str, String)>,
        seconds: i64,
    }

    /// The in-memory implementation the suite runs against.
    struct InMemory {
        held: Mutex<Vec<Collected>>,
    }

    impl InMemory {
        fn new() -> Self {
            Self {
                held: Mutex::new(Vec::new()),
            }
        }

        fn collected(&self) -> Vec<Collected> {
            self.held
                .lock()
                .expect("the fixture holds no poisoned lock")
                .clone()
        }
    }

    impl DiagnosticsSink for InMemory {
        fn event(&self, event: &Event<'_>) {
            self.held
                .lock()
                .expect("the fixture holds no poisoned lock")
                .push(Collected {
                    severity: event.severity(),
                    name: event.name().as_str(),
                    fields: event
                        .fields()
                        .iter()
                        .map(|field| (field.name().as_str(), format!("{:?}", field.value())))
                        .collect(),
                    seconds: event.moment().seconds_from_the_epoch(),
                });
        }
    }

    const SERVER_DECLARED_UNREACHABLE: EventName =
        EventName::declared("server.declared-unreachable");
    const ENTRY_SERVED_STALE: EventName = EventName::declared("cache.entry-served-stale");

    /// One name under each of 0071's three treatments, so that the conditions
    /// below can put all three in one event and read what came out.
    ///
    /// The names are the ones the record itself uses as its examples of each
    /// treatment: a count and an error kind are carried whole, a
    /// server-supplied identifier and an account are reduced, and a session
    /// token is excluded.
    const ATTEMPTS: FieldName = FieldName::carried_whole("attempts");
    const WAITED: FieldName = FieldName::carried_whole("waited");
    const KIND: FieldName = FieldName::carried_whole("kind");
    const ANYTHING_CACHED: FieldName = FieldName::carried_whole("anything-cached");
    const ITEM: FieldName = FieldName::reduced("item");
    const ACCOUNT: FieldName = FieldName::reduced("account");
    const TOKEN: FieldName = FieldName::excluded("token");

    /// What a value under a reduced name looks like in this suite, so that a
    /// condition asserting the correlator is not asserting the digest twice.
    const AN_ITEM: &str = "series/9d41/episode-3";
    const ANOTHER_ITEM: &str = "series/9d41/episode-4";

    /// A value under the excluded name. It is written as one string so that a
    /// condition can search everything the sink was handed for it, which is a
    /// stronger statement than the field being absent under its own name.
    const A_TOKEN: &str = "not-a-real-token-9d41f0c2";

    #[test]
    fn an_event_arrives_with_everything_the_record_says_it_carries() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

        diagnostics.emit(
            Severity::Notice,
            SERVER_DECLARED_UNREACHABLE,
            &[
                Field::new(ATTEMPTS, FieldValue::Count(2)),
                Field::new(WAITED, FieldValue::Interval(Duration::from_millis(5_000))),
                Field::new(KIND, FieldValue::Text("server-unreachable")),
                Field::new(ANYTHING_CACHED, FieldValue::Truth(true)),
            ],
        );

        let collected = sink.collected();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].severity, Severity::Notice);
        assert_eq!(collected[0].name, "server.declared-unreachable");
        assert_eq!(collected[0].seconds, 1_700_000_000);
        assert_eq!(
            collected[0]
                .fields
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),
            vec!["attempts", "waited", "kind", "anything-cached"]
        );
    }

    /// 0071 applied to one event carrying a name under each of its three
    /// treatments, read out of what the sink was handed rather than out of what
    /// was written.
    ///
    /// One condition over all three rather than three conditions, because what
    /// this is about is the boundary: the same call carries a value whole, a
    /// value reduced and a value not at all, and the sink sees the difference.
    #[test]
    fn each_of_the_three_treatments_reaches_the_sink_as_the_record_says() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

        diagnostics.emit(
            Severity::Notice,
            SERVER_DECLARED_UNREACHABLE,
            &[
                Field::new(ATTEMPTS, FieldValue::Count(2)),
                Field::new(TOKEN, FieldValue::Text(A_TOKEN)),
                Field::new(ITEM, FieldValue::Text(AN_ITEM)),
                Field::new(KIND, FieldValue::Text("server-unreachable")),
            ],
        );

        let collected = sink.collected();
        assert_eq!(collected.len(), 1);

        // The excluded name is not there at all, and the order of what is left
        // is the order the emitting subsystem wrote.
        assert_eq!(
            collected[0]
                .fields
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),
            vec!["attempts", "item", "kind"],
        );

        // The two carried whole are unchanged.
        assert_eq!(value_of(&collected[0], "attempts"), "Count(2)");
        assert_eq!(text_of(&collected[0], "kind"), "server-unreachable");

        // The reduced one is a correlator and never the value itself.
        let correlator = text_of(&collected[0], "item");
        assert_ne!(correlator, AN_ITEM);
        assert_eq!(correlator.len(), CORRELATOR_WIDTH);
        assert!(
            correlator.chars().all(|c| c.is_ascii_hexdigit()),
            "the correlator was {correlator}",
        );
    }

    /// The strongest form of the excluded treatment: the value is nowhere in
    /// what the sink was handed, under any name, rather than merely absent from
    /// the name it was written under.
    #[test]
    fn an_excluded_value_appears_nowhere_the_sink_can_see() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

        diagnostics.emit(
            Severity::Notice,
            SERVER_DECLARED_UNREACHABLE,
            &[
                Field::new(TOKEN, FieldValue::Text(A_TOKEN)),
                Field::new(ITEM, FieldValue::Text(A_TOKEN)),
                Field::new(ATTEMPTS, FieldValue::Count(1)),
            ],
        );

        let written_out = format!("{:?}", sink.collected());
        assert!(
            !written_out.contains(A_TOKEN),
            "the token was in what the sink was handed: {written_out}",
        );
    }

    /// What the correlator is for: within one run, two events about one value
    /// carry one correlator, so a report says that one thing failed twice.
    #[test]
    fn one_value_carries_one_correlator_within_a_run() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

        diagnostics.emit(
            Severity::Notice,
            SERVER_DECLARED_UNREACHABLE,
            &[Field::new(ITEM, FieldValue::Text(AN_ITEM))],
        );
        diagnostics.emit(
            Severity::Notice,
            ENTRY_SERVED_STALE,
            &[Field::new(ACCOUNT, FieldValue::Text(AN_ITEM))],
        );

        let collected = sink.collected();
        assert_eq!(
            text_of(&collected[0], "item"),
            text_of(&collected[1], "account"),
        );
    }

    /// The one-change neighbour of the condition above. Without it that
    /// assertion would hold for a reduction that answered the same thing for
    /// every value.
    #[test]
    fn two_values_carry_two_correlators() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

        diagnostics.emit(
            Severity::Notice,
            SERVER_DECLARED_UNREACHABLE,
            &[Field::new(ITEM, FieldValue::Text(AN_ITEM))],
        );
        diagnostics.emit(
            Severity::Notice,
            SERVER_DECLARED_UNREACHABLE,
            &[Field::new(ITEM, FieldValue::Text(ANOTHER_ITEM))],
        );

        let collected = sink.collected();
        assert_ne!(
            text_of(&collected[0], "item"),
            text_of(&collected[1], "item"),
        );
    }

    /// The property 0071 states and the one this treatment exists for: a
    /// correlator means nothing outside the run that produced it. Two
    /// facilities with two salts reduce one value two ways.
    #[test]
    fn two_salts_carry_two_correlators_for_one_value() {
        let clocks = CountingClocks::new();
        let one = InMemory::new();
        let other = InMemory::new();
        let first = Diagnostics::new(&clocks, Some(&one), Severity::Detail, a_salt());
        let second = Diagnostics::new(
            &clocks,
            Some(&other),
            Severity::Detail,
            CorrelatorSalt::from_bytes([0xa5; CorrelatorSalt::WIDTH]),
        );

        for diagnostics in [&first, &second] {
            diagnostics.emit(
                Severity::Notice,
                SERVER_DECLARED_UNREACHABLE,
                &[Field::new(ITEM, FieldValue::Text(AN_ITEM))],
            );
        }

        assert_ne!(
            text_of(&one.collected()[0], "item"),
            text_of(&other.collected()[0], "item"),
        );
    }

    /// A name carries the treatment it was built with, which is the whole of
    /// what makes a field nobody classified unwritable.
    #[test]
    fn a_name_carries_the_treatment_it_was_built_with() {
        assert_eq!(ATTEMPTS.treatment(), Treatment::CarriedWhole);
        assert_eq!(ITEM.treatment(), Treatment::Reduced);
        assert_eq!(TOKEN.treatment(), Treatment::Excluded);
        assert_eq!(TOKEN.as_str(), "token");
    }

    /// The debug text of one field of one collected event.
    fn value_of<'a>(collected: &'a Collected, name: &str) -> &'a str {
        collected
            .fields
            .iter()
            .find(|(field, _)| *field == name)
            .map_or_else(
                || panic!("no field named {name}"),
                |(_, value)| value.as_str(),
            )
    }

    /// The same, for a field the sink received as text, with the debug quoting
    /// taken off so a condition compares the value rather than the debug shape.
    fn text_of<'a>(collected: &'a Collected, name: &str) -> &'a str {
        let value = value_of(collected, name);
        value
            .strip_prefix("Text(\"")
            .and_then(|rest| rest.strip_suffix("\")"))
            .unwrap_or_else(|| panic!("{name} did not arrive as text: {value}"))
    }

    /// The rule 0100 states about cost, at the half a test inside the crate can
    /// reach: with nobody listening the clock is not read.
    #[test]
    fn with_no_sink_nothing_is_delivered_and_no_clock_is_read() {
        let clocks = CountingClocks::new();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Detail, a_salt());

        for _ in 0..1_000 {
            diagnostics.emit(Severity::Fault, ENTRY_SERVED_STALE, &[]);
        }

        assert_eq!(clocks.wall_readings(), 0);
        assert!(!diagnostics.is_reporting(Severity::Fault));
    }

    /// The one-change neighbour of the test above. Without it the assertion
    /// would hold for a facility that reads no clock whatever anybody supplied.
    #[test]
    fn with_a_sink_the_same_events_are_delivered_and_the_clock_is_read() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Detail, a_salt());

        for _ in 0..1_000 {
            diagnostics.emit(Severity::Fault, ENTRY_SERVED_STALE, &[]);
        }

        assert_eq!(sink.collected().len(), 1_000);
        assert_eq!(clocks.wall_readings(), 1_000);
    }

    /// A client running with `detail` off pays what a client with no sink pays.
    #[test]
    fn an_event_below_the_level_reads_no_clock_and_reaches_nobody() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Notice, a_salt());

        diagnostics.emit(Severity::Detail, ENTRY_SERVED_STALE, &[]);

        assert!(sink.collected().is_empty());
        assert_eq!(clocks.wall_readings(), 0);
        assert!(!diagnostics.is_reporting(Severity::Detail));
        assert!(diagnostics.is_reporting(Severity::Notice));
    }

    #[test]
    fn the_level_moves_while_the_core_is_running() {
        let clocks = CountingClocks::new();
        let sink = InMemory::new();
        let diagnostics = Diagnostics::new(&clocks, Some(&sink), Severity::Notice, a_salt());

        diagnostics.emit(Severity::Detail, ENTRY_SERVED_STALE, &[]);
        diagnostics.set_level(Severity::Detail);
        diagnostics.emit(Severity::Detail, ENTRY_SERVED_STALE, &[]);
        diagnostics.set_level(Severity::Fault);
        diagnostics.emit(Severity::Detail, ENTRY_SERVED_STALE, &[]);

        assert_eq!(sink.collected().len(), 1);
        assert_eq!(diagnostics.level(), Severity::Fault);
    }

    /// Every value survives the number it is held as, which is the part that
    /// would fail silently: a level stored wrongly filters the wrong events and
    /// nothing else goes wrong.
    #[test]
    fn every_severity_comes_back_out_of_the_level_it_was_put_into() {
        let clocks = CountingClocks::new();
        let diagnostics = Diagnostics::new(&clocks, None, Severity::Fault, a_salt());
        for severity in Severity::all() {
            diagnostics.set_level(*severity);
            assert_eq!(diagnostics.level(), *severity, "{}", severity.as_str());
        }
        assert_eq!(Severity::all().len(), 4);
    }

    #[test]
    fn the_severities_are_ordered_from_the_most_severe_to_the_least() {
        assert!(Severity::Fault < Severity::Failure);
        assert!(Severity::Failure < Severity::Notice);
        assert!(Severity::Notice < Severity::Detail);
    }

    #[test]
    fn every_severity_is_spelled_once() {
        let mut spellings: Vec<&str> = Severity::all().iter().map(|s| s.as_str()).collect();
        spellings.sort_unstable();
        let before = spellings.len();
        spellings.dedup();
        assert_eq!(spellings.len(), before);
    }

    /// The spelling rule, against text it controls rather than against the names
    /// this tree happens to hold, and against the one-character mistakes
    /// somebody actually makes.
    #[test]
    fn the_spelling_rule_reads_a_name_and_bites_the_near_misses() {
        for good in [
            "server.declared-unreachable",
            "cache.entry-served-stale",
            "session.token-renewed",
            "artwork.tier-evicted",
            "a",
            "http2.stream-reset",
        ] {
            assert!(is_a_well_formed_event_name(good), "{good}");
        }

        for bad in [
            "",
            "Server.declared-unreachable",
            "server..declared",
            ".server",
            "server.",
            "server declared",
            "server.-declared",
            "server.declared-",
            "server.declared_unreachable",
            "server.declaréd",
        ] {
            assert!(!is_a_well_formed_event_name(bad), "{bad}");
        }
    }

    #[test]
    fn a_declared_name_keeps_its_spelling() {
        assert_eq!(
            SERVER_DECLARED_UNREACHABLE.as_str(),
            "server.declared-unreachable"
        );
    }

    /// What a malformed name does where it is written as a constant is refuse to
    /// compile, which no test can assert from inside the suite. This is the same
    /// refusal reached at run time, which is what the constructor does when it is
    /// handed a name that was not evaluated at compile time.
    #[test]
    #[should_panic(expected = "an event identity is lower case, dotted, and never a number")]
    fn a_malformed_identity_is_refused_rather_than_reported_under() {
        let _ = EventName::declared("Server Declared Unreachable");
    }
}
