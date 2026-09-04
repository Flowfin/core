//! Creating the core, stopping it, and a host that suspends it.
//!
//! `docs/decisions/0115-creating-and-stopping-the-core.md` decides all three and
//! `docs/decisions/0009-the-concurrency-model.md` carries the shape underneath
//! them. What is here is the part of both that a value settles: what a client
//! hands to creation, which of those the core has, the bound a stop is given,
//! which lane a stop that expired names, and what a call does once a stop has
//! been asked for.
//!
//! # What is here, and what is deliberately not
//!
//! WHAT IS NOT HERE IS A LANE. 0009 creates two threads when the core is created
//! and a stop is the one call in the core that waits on one. Nothing in this tree
//! starts a thread, so nothing here starts, cancels or waits for anything: this
//! module holds what the answers are, and the day something runs, it is what the
//! running code is judged against. That is the same position
//! [`crate::server::transport`] takes about the socket it does not hold.
//!
//! WHAT IS NOT HERE IS THE FLOOR UNDER THE STOP BOUND, and its absence is a gap
//! rather than a decision. 0115 requires one and says it is not a preference,
//! because 0009 fixes two things the core cannot interrupt - a decode running to
//! the end of its current step, and a read already begun through the byte store.
//! Neither record states how long either of those may take, and no other record
//! in this tree does, so a floor written here would be a number this repository
//! invented at the one call site that needed it. [`StopBound`] carries the
//! default and no floor, and says so where somebody setting one meets it.
//!
//!
//! # The rule as data, and why it is gathered here
//!
//! `docs/decisions/0071-what-may-leave-through-a-diagnostic-event.md` refuses a
//! diagnostics bundle assembled by the core, because 0068 and 0100 both fix that
//! an event is handed over and forgotten in the same call, so there is no store
//! of past events here to assemble one out of. What that record asks the core for
//! instead is the rule as data: for each field name it has ever emitted, which of
//! the three treatments it applies, so that a client assembling a bundle out of
//! what its own sink kept can include the statement verbatim and whoever is about
//! to send it can read what is not in it.
//!
//! It is gathered here because a field name deliberately lives beside the event
//! identity that carries it, which is 0100's placement, so the only place that
//! sees all of them is the one that sees every subsystem. That is creation, and
//! this is the module creation's own answers are in.
//!
//! # Why creation reaching nothing is a property rather than a description
//!
//! 0115 refuses a creation call that restores a session, opens a connection,
//! resolves a name, reads an entry or reads a secret. [`Supplied`] is the whole
//! of what creation takes and it holds borrowed implementations and nothing else,
//! so there is no state for a creation to have filled in and no call for one to
//! have made. [`Supplied::what_is_present`] reads the three it was handed and
//! asks none of them anything.

use core::time::Duration;

use crate::cache::ByteStore;
use crate::cache::bound;
use crate::cache::envelope;
use crate::diagnostics::DiagnosticsSink;
use crate::diagnostics::redaction::FieldName;
use crate::failure::Failure;
use crate::server::write_queue;
use crate::session::SecretStore;
use crate::session::mid_playback;

/// The two seconds a stop is bounded by where a client sets nothing.
///
/// From 0115. Two is chosen so that a stop called from inside a platform's own
/// termination callback returns while that callback still has time left. The
/// windows those platforms allow are a claim in that record rather than a
/// measurement: nothing in this tree runs on a platform, and no command in this
/// repository produces them.
///
/// A client that knows its own window sets the bound rather than accepting a
/// default chosen against a claim.
pub const A_STOP_IS_BOUNDED_AT: Duration = Duration::from_secs(2);

/// One of the two threads 0009 creates with the core.
///
/// The set is closed and it is 0009's rather than one invented here. A stop that
/// expired reports which of them did not stop, because "the core could not stop"
/// tells whoever reads it nothing they can act on, and the two lanes carry
/// different work for different reasons.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lane {
    /// Everything waiting on a server.
    Waiting,
    /// Everything that costs a processor, which is where a decode runs.
    Processing,
}

impl Lane {
    /// The name this lane is reported under.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Processing => "processing",
        }
    }
}

/// How a stop ended.
///
/// 0009 refuses reporting an expiry as a stop, and this is that refusal as a
/// type: there is no variant that says a stop succeeded without saying that both
/// lanes stopped, and the expiry carries the lane rather than being a bare
/// failure.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowTheStopEnded {
    /// Both lanes stopped inside the bound.
    BothLanesStopped,
    /// The bound expired with this lane still running.
    ALaneDidNotStop(Lane),
}

/// The bound a stop is given.
///
/// THERE IS NO FLOOR HERE AND 0115 ASKS FOR ONE. That record says a bound below
/// what 0009 makes uninterruptible produces a stop that always reports failure,
/// which teaches a client to ignore the report, and it says the floor is not a
/// preference. Neither 0009 nor any other record in this tree says how long a
/// decode step or a byte-store read may take, so the number that floor would be
/// does not exist yet, and inventing one here would fix it in the place least
/// likely to be argued with. What is refused instead is a bound of nothing,
/// which is a stop that cannot wait for a lane at all.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopBound {
    within: Duration,
}

impl Default for StopBound {
    fn default() -> Self {
        Self::the_default()
    }
}

impl StopBound {
    /// The bound a client that sets nothing gets.
    #[must_use]
    pub const fn the_default() -> Self {
        Self {
            within: A_STOP_IS_BOUNDED_AT,
        }
    }

    /// The bound a client set.
    ///
    /// A zero bound is refused rather than accepted, because 0009 makes a stop
    /// wait for both lanes and a bound of nothing is a stop that never waits,
    /// which reports an expiry on a core that would have stopped. Every other
    /// value is taken, and the paragraph on this type says why no floor above it
    /// is applied.
    #[must_use]
    pub const fn of(within: Duration) -> Option<Self> {
        if within.is_zero() {
            return None;
        }
        Some(Self { within })
    }

    /// How long a stop may wait for the lanes.
    #[must_use]
    pub const fn within(self) -> Duration {
        self.within
    }
}

/// Which of the three implementations a client handed to creation are present.
///
/// 0115 asks for this because three separate absences produce a core that works
/// in three different reduced ways, and a client that cannot ask is a client that
/// cannot explain any of them to an operator. What each absence costs is decided
/// in 0033, 0040 and 0100, one each, and is not repeated here.
///
/// Thread safety, from 0009: a plain value, safe from any thread. 0115 makes the
/// call that answers it one that cannot wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatIsPresent {
    byte_store: bool,
    secret_store: bool,
    diagnostics_sink: bool,
}

impl WhatIsPresent {
    /// Whether the byte store from 0040 was supplied.
    #[must_use]
    pub const fn byte_store(self) -> bool {
        self.byte_store
    }

    /// Whether the secret store from 0033 was supplied.
    #[must_use]
    pub const fn secret_store(self) -> bool {
        self.secret_store
    }

    /// Whether the diagnostics sink from 0100 was supplied.
    #[must_use]
    pub const fn diagnostics_sink(self) -> bool {
        self.diagnostics_sink
    }

    /// Whether a client supplied none of the three.
    ///
    /// This is the core 0115's first condition is about, and it is legal rather
    /// than degraded: each absence has a record saying what the core does
    /// instead.
    #[must_use]
    pub const fn nothing_was_supplied(self) -> bool {
        !self.byte_store && !self.secret_store && !self.diagnostics_sink
    }
}

/// Everything the core wants from a client, as creation takes it.
///
/// One value rather than three arguments, so that adding a fourth
/// implementation is a method here rather than a change to every call site, and
/// so that [`Supplied::what_is_present`] is derived from the same thing creation
/// was handed rather than from a second record of it.
///
/// Each may be absent. A [`Supplied::nothing`] is the whole of 0115's "no client
/// implementations supplied at all", and it is a legal core.
///
/// Thread safety, from 0009: safe from any thread. Each of the three interfaces
/// requires it of the client's own implementation, so this value carries the
/// same bound rather than weakening it.
#[derive(Clone, Copy)]
pub struct Supplied<'a> {
    byte_store: Option<&'a dyn ByteStore>,
    secret_store: Option<&'a dyn SecretStore>,
    diagnostics_sink: Option<&'a dyn DiagnosticsSink>,
}

impl core::fmt::Debug for Supplied<'_> {
    /// Says which of the three are present and nothing about any of them.
    ///
    /// The implementations are the client's own and this core has no words for
    /// them, so what is written is the answer [`Supplied::what_is_present`]
    /// gives. A derived one would need each interface to carry a formatting of
    /// its own, which is a demand on a client's type for the benefit of a line
    /// nobody reads.
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("Supplied")
            .field("byte_store", &self.byte_store.is_some())
            .field("secret_store", &self.secret_store.is_some())
            .field("diagnostics_sink", &self.diagnostics_sink.is_some())
            .finish()
    }
}

impl Default for Supplied<'_> {
    fn default() -> Self {
        Self::nothing()
    }
}

impl<'a> Supplied<'a> {
    /// A client supplying none of the three.
    #[must_use]
    pub const fn nothing() -> Self {
        Self {
            byte_store: None,
            secret_store: None,
            diagnostics_sink: None,
        }
    }

    /// With the byte store 0040 defines.
    #[must_use]
    pub const fn and_the_byte_store(mut self, store: &'a dyn ByteStore) -> Self {
        self.byte_store = Some(store);
        self
    }

    /// With the secret store 0033 defines.
    #[must_use]
    pub const fn and_the_secret_store(mut self, store: &'a dyn SecretStore) -> Self {
        self.secret_store = Some(store);
        self
    }

    /// With the diagnostics sink 0100 defines.
    #[must_use]
    pub const fn and_the_diagnostics_sink(mut self, sink: &'a dyn DiagnosticsSink) -> Self {
        self.diagnostics_sink = Some(sink);
        self
    }

    /// The byte store, where one was supplied.
    #[must_use]
    pub const fn byte_store(&self) -> Option<&'a dyn ByteStore> {
        self.byte_store
    }

    /// The secret store, where one was supplied.
    #[must_use]
    pub const fn secret_store(&self) -> Option<&'a dyn SecretStore> {
        self.secret_store
    }

    /// The diagnostics sink, where one was supplied.
    #[must_use]
    pub const fn diagnostics_sink(&self) -> Option<&'a dyn DiagnosticsSink> {
        self.diagnostics_sink
    }

    /// Which of the three are present.
    ///
    /// It reads what it was handed and asks none of them anything, which is
    /// 0115's "creation reaches nothing" holding for the capability answer too:
    /// a call that probed a store to find out whether it worked would be a
    /// creation-time store read under another name.
    #[must_use]
    pub const fn what_is_present(&self) -> WhatIsPresent {
        WhatIsPresent {
            byte_store: self.byte_store.is_some(),
            secret_store: self.secret_store.is_some(),
            diagnostics_sink: self.diagnostics_sink.is_some(),
        }
    }
}

/// What a call made against the core does now.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatACallDoes {
    /// The core has not been asked to stop, so the call is made.
    ///
    /// A suspended core is here rather than beside it. 0115 says a suspend
    /// cancels nothing the caller still wants, because the host has not asked
    /// for that.
    GoesAhead,
    /// A stop was asked for, so the call fails without being made.
    ///
    /// `cancelled` is an imperfect fit and 0115 says so rather than growing the
    /// vocabulary: its meaning in 0004 is that the caller asked for this to stop.
    /// What makes the reuse tolerable is that nothing but the client's own stop
    /// puts a core into this state.
    FailsWith(Failure),
}

/// Where the core is in the lifetime 0115 fixes.
///
/// Four positions and three moves. A core is running when it is created; a
/// suspend and a resume move between running and suspended and keep everything;
/// a stop moves to finished from either and keeps nothing.
///
/// THERE IS NO MOVE BACK OUT OF FINISHED, and that is 0115 rather than an
/// omission. A restartable core needs a rule for what survives a stop, per piece
/// of state, and that record's own list of what would need one is long: the
/// lanes, the capability answers, the correlator salt in 0071, whatever the
/// transport was holding. Creating a second core is the answer.
///
/// Thread safety, from 0009: a plain value, safe from any thread. What the
/// running core does with it is #115's remaining conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lifetime {
    suspended: bool,
    stopped: Option<HowTheStopEnded>,
}

impl Default for Lifetime {
    fn default() -> Self {
        Self::created()
    }
}

impl Lifetime {
    /// A core that has just been created.
    ///
    /// Running, not suspended and not stopped. 0115 says a core that has just
    /// been created has started its two lanes and has done nothing else.
    #[must_use]
    pub const fn created() -> Self {
        Self {
            suspended: false,
            stopped: None,
        }
    }

    /// The host is setting the process aside.
    ///
    /// It stops the core's own scheduled work, which is 0045's recovery schedule
    /// and 0057's reporting cadence. It cancels nothing and flushes nothing.
    ///
    /// A suspend after a stop was asked for changes nothing. The core is
    /// finished, and a host setting aside a process whose core has stopped is
    /// not a reason to unsay that.
    #[must_use]
    pub const fn suspended(mut self) -> Self {
        if self.stopped.is_some() {
            return self;
        }
        self.suspended = true;
        self
    }

    /// The host has brought the process back.
    ///
    /// It restarts what a suspend stopped and does nothing else. 0115 refuses a
    /// resume that refreshes, revalidates the cache or renews a token ahead of a
    /// call, because each of those spends a person's connection at the moment
    /// they have picked the device up and are waiting to see something.
    ///
    /// What a resume may not assume is written in 0115 and reaches other
    /// modules: the token is neither trusted nor refused here, and every
    /// connection the transport was holding is discarded rather than reused.
    /// Neither is a state of this value.
    #[must_use]
    pub const fn resumed(mut self) -> Self {
        if self.stopped.is_some() {
            return self;
        }
        self.suspended = false;
        self
    }

    /// Whether the host has set this core aside.
    #[must_use]
    pub const fn is_suspended(self) -> bool {
        self.suspended
    }

    /// A stop was asked for and ended this way.
    ///
    /// A STOP IS IDEMPOTENT AND A SECOND ONE DOES NOT REPLACE THE FIRST'S
    /// OUTCOME. 0115 says a second stop returns at once with the same outcome,
    /// and taking the later one would let a core that reported a lane still
    /// running be asked again and answer that everything stopped, which is the
    /// negative disclosure turned positive by repetition.
    #[must_use]
    pub const fn stopped(mut self, ended: HowTheStopEnded) -> Self {
        if self.stopped.is_some() {
            return self;
        }
        self.stopped = Some(ended);
        self
    }

    /// How the stop ended, where one was asked for.
    #[must_use]
    pub const fn how_the_stop_ended(self) -> Option<HowTheStopEnded> {
        self.stopped
    }

    /// Whether a stop has been asked for at all.
    #[must_use]
    pub const fn a_stop_was_asked_for(self) -> bool {
        self.stopped.is_some()
    }

    /// What a call made now does.
    ///
    /// IT ASKS WHETHER A STOP WAS ASKED FOR AND NEVER WHETHER IT SUCCEEDED,
    /// which is 0115 in its own words: every call made after a stop was
    /// requested fails, whether the stop succeeded or timed out. Reading the
    /// outcome instead would leave a core whose stop expired accepting work
    /// while a lane it could not stop is still running, which is the state the
    /// bound exists to report rather than to continue through.
    #[must_use]
    pub fn what_a_call_does(self) -> WhatACallDoes {
        if self.stopped.is_some() {
            return WhatACallDoes::FailsWith(Failure::cancelled());
        }
        WhatACallDoes::GoesAhead
    }
}

/// Every field name this build of the core emits, with the treatment each one
/// carries.
///
/// This is 0071's rule as data. A client puts it verbatim into a bundle it
/// assembled out of what its own sink kept, so that whoever is about to send that
/// bundle can read which values never appear in it, which appear only as a
/// correlator, and which are carried unchanged.
///
/// THE STATEMENT IS ABOUT WHAT THE CORE DID AND NOT ABOUT WHAT A SINK DID
/// AFTERWARDS, which is 0071's own sentence. A client that writes events out into
/// a log file of its own has made its own decisions, and a bundle carrying this
/// says so rather than implying a guarantee across a boundary the core cannot
/// see.
///
/// WHAT NOTHING HERE REFUSES IS A NAME DECLARED AND NOT LISTED. A field name is a
/// constant beside the event identity that carries it, which is where 0100 puts
/// it and where 0071 wants it, so a subsystem can declare one and emit it without
/// this list moving. Then the statement is short by that name and reads as
/// complete, which is the one way it can be wrong in the direction that matters.
/// No reading of this tree catches it: what would is a check whose subject is
/// every construction of [`FieldName`] anywhere under `src/`, and this repository
/// has none. It is the same bound the name-list rules in
/// `.github/invariants/rules` print about themselves.
#[must_use]
pub const fn every_field_name_the_core_emits() -> &'static [FieldName] {
    &[
        bound::RELEASED_BYTES,
        bound::RELEASED_ENTRIES,
        bound::FOR_TIER,
        bound::CONSECUTIVE_REFUSALS,
        bound::SUSPENDED_FOR,
        envelope::ENTRY,
        envelope::ENTRY_KIND,
        envelope::CHECK,
        envelope::VERSION_FOUND,
        mid_playback::POSITIONS_HELD,
        write_queue::FOR_TARGET,
        write_queue::ASSERTED_ABOUT,
    ]
}

#[cfg(test)]
mod tests {
    //! 0115's creation, its capability answer, its bound and its lifetime, asked
    //! of the values.
    //!
    //! What these cannot ask is two of #115's three conditions. Stopping a set of
    //! outstanding requests and proving no thread outlived the stop, and
    //! suspending and resuming across a clock jump, each need something running,
    //! and nothing in this tree starts a thread or makes a request.

    use super::{
        A_STOP_IS_BOUNDED_AT, HowTheStopEnded, Lane, Lifetime, StopBound, Supplied, WhatACallDoes,
    };
    use crate::cache::{ByteStore, EntryKey, StorageUnavailable};
    use crate::diagnostics::{DiagnosticsSink, Event};
    use crate::failure::Kind;
    use crate::session::{SecretName, SecretStore, SecretStoreUnavailable};
    use core::time::Duration;

    /// A store that answers nothing, because no case here asks it anything.
    ///
    /// 0115 says creation reaches no store, so a fixture that recorded its calls
    /// would be proving the absence of a call this module has no way to make.
    /// What it is here for is to be present.
    struct AStore;

    impl ByteStore for AStore {
        fn read(&self, _: &EntryKey) -> Result<Option<Vec<u8>>, StorageUnavailable> {
            Ok(None)
        }

        fn write(&self, _: &EntryKey, _: &[u8]) -> Result<(), StorageUnavailable> {
            Ok(())
        }

        fn remove(&self, _: &EntryKey) -> Result<(), StorageUnavailable> {
            Ok(())
        }

        fn held_bytes(&self) -> Result<u64, StorageUnavailable> {
            Ok(0)
        }
    }

    struct ASecretStore;

    impl SecretStore for ASecretStore {
        fn keep(&self, _: &SecretName, _: &[u8]) -> Result<(), SecretStoreUnavailable> {
            Ok(())
        }

        fn read(&self, _: &SecretName) -> Result<Option<Vec<u8>>, SecretStoreUnavailable> {
            Ok(None)
        }

        fn forget(&self, _: &SecretName) -> Result<(), SecretStoreUnavailable> {
            Ok(())
        }
    }

    struct ASink;

    impl DiagnosticsSink for ASink {
        fn event(&self, _: &Event<'_>) {}
    }

    /// #115's first condition names a core created with no client
    /// implementations supplied at all, and 0115 makes that legal rather than
    /// degraded.
    #[test]
    fn a_core_created_with_nothing_supplied_says_so_for_each_of_the_three() {
        let present = Supplied::nothing().what_is_present();

        assert!(present.nothing_was_supplied());
        assert!(!present.byte_store());
        assert!(!present.secret_store());
        assert!(!present.diagnostics_sink());
    }

    /// Three separate absences produce a core that works in three different
    /// reduced ways, which is 0115's reason for the answer being per
    /// implementation rather than one flag.
    #[test]
    fn each_of_the_three_is_answered_on_its_own() {
        let store = AStore;
        let secrets = ASecretStore;
        let sink = ASink;

        let only_bytes = Supplied::nothing()
            .and_the_byte_store(&store)
            .what_is_present();
        assert!(only_bytes.byte_store());
        assert!(!only_bytes.secret_store());
        assert!(!only_bytes.diagnostics_sink());
        assert!(!only_bytes.nothing_was_supplied());

        let only_secrets = Supplied::nothing()
            .and_the_secret_store(&secrets)
            .what_is_present();
        assert!(!only_secrets.byte_store());
        assert!(only_secrets.secret_store());
        assert!(!only_secrets.diagnostics_sink());

        let only_events = Supplied::nothing()
            .and_the_diagnostics_sink(&sink)
            .what_is_present();
        assert!(!only_events.byte_store());
        assert!(!only_events.secret_store());
        assert!(only_events.diagnostics_sink());
    }

    /// All three, so that the answer is not one that happens to be right for
    /// every core with one implementation in it.
    #[test]
    fn a_client_supplying_all_three_is_told_all_three() {
        let store = AStore;
        let secrets = ASecretStore;
        let sink = ASink;

        let present = Supplied::nothing()
            .and_the_byte_store(&store)
            .and_the_secret_store(&secrets)
            .and_the_diagnostics_sink(&sink)
            .what_is_present();

        assert!(present.byte_store());
        assert!(present.secret_store());
        assert!(present.diagnostics_sink());
        assert!(!present.nothing_was_supplied());
    }

    /// The default is 0115's two seconds, read out of the constant rather than
    /// written twice.
    #[test]
    fn a_stop_a_client_bounded_at_nothing_takes_the_bound_it_was_given() {
        assert_eq!(StopBound::the_default().within(), A_STOP_IS_BOUNDED_AT);
        assert_eq!(A_STOP_IS_BOUNDED_AT, Duration::from_secs(2));

        let set = StopBound::of(Duration::from_millis(500)).expect("a bound above nothing");
        assert_eq!(set.within(), Duration::from_millis(500));
    }

    /// A bound of nothing is a stop that never waits for a lane, so it reports an
    /// expiry against a core that would have stopped.
    #[test]
    fn a_bound_of_nothing_is_refused() {
        assert!(StopBound::of(Duration::ZERO).is_none());
    }

    /// A core that has just been created is running and has been asked for
    /// nothing.
    #[test]
    fn a_created_core_is_running_and_takes_calls() {
        let core = Lifetime::created();

        assert!(!core.is_suspended());
        assert!(!core.a_stop_was_asked_for());
        assert_eq!(core.what_a_call_does(), WhatACallDoes::GoesAhead);
    }

    /// 0115 says a suspend cancels nothing the caller still wants, because the
    /// host has not asked for that.
    #[test]
    fn a_suspended_core_still_takes_a_call() {
        let core = Lifetime::created().suspended();

        assert!(core.is_suspended());
        assert_eq!(core.what_a_call_does(), WhatACallDoes::GoesAhead);

        let back = core.resumed();
        assert!(!back.is_suspended());
        assert_eq!(back.what_a_call_does(), WhatACallDoes::GoesAhead);
    }

    /// Every call made after a stop was requested fails, and the kind is 0004's
    /// `cancelled`, which 0115 takes knowing the fit is imperfect.
    #[test]
    fn a_call_after_a_stop_was_asked_for_fails_with_cancelled() {
        let core = Lifetime::created().stopped(HowTheStopEnded::BothLanesStopped);

        let WhatACallDoes::FailsWith(failure) = core.what_a_call_does() else {
            panic!("0115 refuses a call made after a stop was requested");
        };
        assert_eq!(failure.kind(), Kind::Cancelled);
    }

    /// The half a reader gets wrong: it is whether a stop was ASKED FOR, not
    /// whether it succeeded. A core whose stop expired is finished too.
    #[test]
    fn a_call_after_a_stop_that_expired_fails_the_same_way() {
        let core = Lifetime::created().stopped(HowTheStopEnded::ALaneDidNotStop(Lane::Processing));

        assert!(core.a_stop_was_asked_for());
        assert!(matches!(
            core.what_a_call_does(),
            WhatACallDoes::FailsWith(_)
        ));
        assert_eq!(
            core.how_the_stop_ended(),
            Some(HowTheStopEnded::ALaneDidNotStop(Lane::Processing)),
        );
    }

    /// 0115 says a second stop returns at once with the same outcome. Taking the
    /// later one would let a core that reported a lane still running be asked
    /// again and answer that everything stopped.
    #[test]
    fn a_second_stop_does_not_replace_the_first_outcome() {
        let expired = Lifetime::created().stopped(HowTheStopEnded::ALaneDidNotStop(Lane::Waiting));

        let asked_again = expired.stopped(HowTheStopEnded::BothLanesStopped);

        assert_eq!(
            asked_again.how_the_stop_ended(),
            Some(HowTheStopEnded::ALaneDidNotStop(Lane::Waiting)),
        );
    }

    /// There is no move back out of finished, and a host setting the process
    /// aside is not one.
    #[test]
    fn a_finished_core_is_not_moved_by_a_suspend_or_a_resume() {
        let finished = Lifetime::created().stopped(HowTheStopEnded::BothLanesStopped);

        let after = finished.suspended().resumed();

        assert!(after.a_stop_was_asked_for());
        assert!(!after.is_suspended());
        assert!(matches!(
            after.what_a_call_does(),
            WhatACallDoes::FailsWith(_)
        ));
    }

    /// A stop that expired names which lane did not stop, because "the core
    /// could not stop" tells whoever reads it nothing they can act on.
    #[test]
    fn the_two_lanes_are_named_apart() {
        assert_ne!(
            HowTheStopEnded::ALaneDidNotStop(Lane::Waiting),
            HowTheStopEnded::ALaneDidNotStop(Lane::Processing),
        );
        assert_eq!(Lane::Waiting.declared_name(), "waiting");
        assert_eq!(Lane::Processing.declared_name(), "processing");
    }
}
