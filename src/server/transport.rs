//! The bounds every request is carried inside.
//!
//! `docs/decisions/0027-the-transports-timeouts-and-connections.md` decides all
//! of it and this module is that record as values: two per-attempt bounds inside
//! the call deadline 0007 already sets, a limit of six connections to one server
//! and twelve across all of them, an idle connection kept for sixty seconds and
//! reused for any session against the same origin, and a cancelled response read
//! only while reading is cheaper than the handshake it saves.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything 0027 decides that a reading of two clock values
//! settles: which bound an attempt runs against and which of them expired, how
//! many connections may be outstanding and against whom, which idle connection
//! may be handed back and which has to go, and how far a cancelled body is read
//! before the connection is closed instead. Each is a decision the transport
//! makes many times per request, each is provable in microseconds against the
//! controlled clock 0102 requires, and each is wrong in a way nothing downstream
//! would report.
//!
//! WHAT IS NOT HERE IS THE SOCKET, and it is absent for a reason rather than for
//! want of time. A connection is opened after 0029 has decided whether the
//! certificate presented on it is acceptable - 0027 puts that decision inside
//! the connect bound in so many words - and 0029 is not built. A transport that
//! opened a socket today would be one that speaks in clear over any address it
//! was handed, in a core whose whole position is that a credential travels to
//! one place. #27 is where that is written down against the issue rather than
//! only here.
//!
//! So nothing in this module reads or writes a byte, and the type parameter on
//! [`IdleConnections`] is what keeps it that way: the pool is written over
//! whatever a connection turns out to be, so the file that eventually holds a
//! socket is a different file from the one that decides how long an idle one
//! lives.
//!
//! # Every number here is chosen and none is measured
//!
//! 0027 says so of its own values, in the same words 0007 uses for its
//! thresholds and 0038 for its waits: there was no code in this repository to
//! measure. #65 is the harness that would replace a choice with a number, and
//! until it exists a reader should take each constant below as an argument
//! rather than as a measurement.

use core::time::Duration;

use crate::clock::SteadyInstant;
use crate::failure::Deadline;
use crate::server::address::Origin;

/// The two seconds an attempt has to reach a connection.
///
/// From 0027. It runs from the moment an attempt begins and covers resolving the
/// name, reaching the machine, and the handshake through the point where 0029
/// decides whether the certificate is acceptable. Resolution is inside it rather
/// than beside it, because a resolver that does not answer and a server that
/// does not answer are the same thing from the caller's side.
///
/// A name that resolves to nothing is not this bound at all. 0007 reports
/// `unreachable` at once on a name that did not resolve, on a refused connection
/// and on an absent route, and none of the three involves waiting.
pub const REACHING_A_CONNECTION: Duration = Duration::from_secs(2);

/// The two seconds an answer has to begin after the request has been written.
///
/// From 0027. This is the bound 0007 says a single overall timeout handles
/// worst: a server that accepts a connection and then says nothing is otherwise
/// indistinguishable from one that is merely slow until the whole deadline has
/// gone.
///
/// It is above 0007's 400 ms by a distance, and that distance is load-bearing. A
/// bound at or below it would abandon attempts before the core had ever reported
/// `late`, which would make that state unreachable in exactly the case it was
/// written for.
pub const REACHING_THE_FIRST_BYTE: Duration = Duration::from_secs(2);

/// The five seconds a whole call has, which 0007 sets and 0027 does not move.
///
/// It belongs to the call rather than to the attempt, so an attempt ends on
/// whichever arrives first: its own bound above, or what is left of this.
/// [`CallDeadline`] is where the arithmetic lives.
pub const A_CALL_IS_ABANDONED_AFTER: Duration = Duration::from_secs(5);

/// How long an idle connection is kept before it is closed.
///
/// From 0027. A connection nothing has used for a minute is one whose survival
/// the core cannot know, and writing on a dead one is not free: it costs the
/// first-byte bound above and then a retry, which is worse than the handshake it
/// was avoiding.
pub const AN_IDLE_CONNECTION_IS_KEPT_FOR: Duration = Duration::from_secs(60);

/// How long the remainder of a cancelled response is read for.
///
/// From 0027, and the same argument as the byte bound beside it: reading on is
/// worth doing only while it is cheaper than the handshake it saves.
pub const A_CANCELLED_BODY_IS_READ_FOR: Duration = Duration::from_secs(1);

/// How many bytes of a cancelled response are read before the connection is
/// closed instead.
///
/// From 0027. Sixty-four kilobytes is the order of a body the core would have
/// taken in a read or two anyway. A cancelled image is the case that decides the
/// shape: at up to sixteen megabytes under 0055 an artwork body is far past this,
/// so a tile scrolled off the screen closes its connection rather than
/// downloading in full to save a handshake.
pub const A_CANCELLED_BODY_IS_READ_TO: usize = 64 * 1024;

/// How many requests may be outstanding against one server.
///
/// From 0027. It bounds the two hundred tiles in #53 into a queue rather than
/// into two hundred sockets, and a server people run at home is frequently a
/// small machine, where the sixth concurrent request is already competing with
/// the one somebody is waiting for.
///
/// The count is over requests outstanding against one server, whatever number of
/// sockets carries them, which is what keeps it meaning the same thing if the
/// protocol version turns out to multiplex.
pub const REQUESTS_OUTSTANDING_TO_ONE_SERVER: usize = 6;

/// How many requests may be outstanding across every server at once.
///
/// From 0027. It is a ceiling and not a reservation, and the cost of that is the
/// third server: where three or more are active they contend for these twelve,
/// and a stalled one can hold six of them for as long as 0007's deadline. That
/// is named in the record rather than solved, because reserving slots per server
/// would idle most of them on the ordinary single-server device.
pub const REQUESTS_OUTSTANDING_ACROSS_ALL_SERVERS: usize = 12;

/// The five seconds of a call, anchored at the moment it began.
///
/// 0102 puts every one of 0027's bounds on the steady clock, because each is an
/// interval between two events inside one run. A device that suspended mid-call
/// has no call left to time out, which is the reason the record gives for not
/// using the elapsed clock.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallDeadline {
    began: SteadyInstant,
}

impl CallDeadline {
    /// A deadline beginning now.
    #[must_use]
    pub const fn beginning_at(began: SteadyInstant) -> Self {
        Self { began }
    }

    /// How long the call has run.
    #[must_use]
    pub fn run_for(self, now: SteadyInstant) -> Duration {
        now.interval_since(self.began)
    }

    /// What is left of the five seconds, floored at nothing.
    ///
    /// Nothing left and expired are the same reading, and they are meant to be:
    /// a caller asking how much is left of a deadline that has passed is asking
    /// how long it may still wait, and the honest answer is not at all.
    #[must_use]
    pub fn left_at(self, now: SteadyInstant) -> Duration {
        A_CALL_IS_ABANDONED_AFTER.saturating_sub(self.run_for(now))
    }

    /// Whether the call has run out of time.
    #[must_use]
    pub fn passed_at(self, now: SteadyInstant) -> bool {
        self.run_for(now) >= A_CALL_IS_ABANDONED_AFTER
    }
}

/// The bound one attempt actually runs against, and which one it is.
///
/// 0027's sentence is that an attempt is bounded by whichever arrives first, its
/// own two seconds or what is left of the caller's five. Both halves are needed
/// rather than only the smaller number: the duration is what a wait is measured
/// against, and the name is what a failure carries, since 0004 asks which of the
/// three deadlines expired and a caller acts differently on the answer.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptBound {
    within: Duration,
    named: Deadline,
}

impl AttemptBound {
    /// The bound an attempt about to reach a connection runs against.
    #[must_use]
    pub fn reaching_a_connection(call: CallDeadline, now: SteadyInstant) -> Self {
        Self::narrower_of(REACHING_A_CONNECTION, Deadline::Connect, call, now)
    }

    /// The bound an attempt waiting for a first byte runs against.
    #[must_use]
    pub fn reaching_the_first_byte(call: CallDeadline, now: SteadyInstant) -> Self {
        Self::narrower_of(REACHING_THE_FIRST_BYTE, Deadline::FirstByte, call, now)
    }

    /// Which of 0027's per-attempt bounds and the call deadline is nearer.
    ///
    /// A tie goes to the call. Both readings are true at that instant and only
    /// one name can be carried, and 0007's deadline is the one a caller can act
    /// on: it says the call is over, where a per-attempt name says another
    /// attempt might be worth spending. Telling somebody to try again inside a
    /// call that has just ended is the wrong half of a tie to report.
    fn narrower_of(
        per_attempt: Duration,
        named: Deadline,
        call: CallDeadline,
        now: SteadyInstant,
    ) -> Self {
        let left = call.left_at(now);
        if per_attempt < left {
            Self {
                within: per_attempt,
                named,
            }
        } else {
            Self {
                within: left,
                named: Deadline::WholeRequest,
            }
        }
    }

    /// How long the attempt may run.
    #[must_use]
    pub const fn within(self) -> Duration {
        self.within
    }

    /// Which deadline a failure at this bound carries.
    #[must_use]
    pub const fn named(self) -> Deadline {
        self.named
    }

    /// Whether an attempt that began at `began` has reached this bound.
    #[must_use]
    pub fn reached_at(self, began: SteadyInstant, now: SteadyInstant) -> bool {
        now.interval_since(began) >= self.within
    }
}

/// The count of requests outstanding, per server and in total.
///
/// 0027 states both limits over requests rather than over sockets, and this type
/// counts what that sentence counts. It answers whether one more may start and
/// it does not wait: 0009 puts the waiting in a lane sized from these numbers,
/// and a permit that blocked would be a second place a request can wait.
///
/// Thread safety, from 0009: a plain value with no interior mutability. The lane
/// that owns it is what serialises access, which is the same arrangement 0009
/// describes for everything else on that lane.
#[derive(Debug, Default)]
pub struct Outstanding {
    against: Vec<(Origin, usize)>,
}

/// Why one more request may not start yet.
///
/// Neither is a failure. 0027's limits produce a wait rather than a refusal, and
/// 0009 sizes its waiting lane at one waiter per permitted connection so that no
/// request ever waits for a lane before it waits for a connection. The two
/// values are separate because the wait ends on different events: the first ends
/// when that server frees a slot, the second when any server does.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waits {
    /// This server already holds every slot it is allowed.
    ThisServerIsAtItsSix,
    /// Every slot across every server is in use.
    EverySlotIsInUse,
}

impl Outstanding {
    /// Nothing outstanding anywhere.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            against: Vec::new(),
        }
    }

    /// Starts one more request against `origin`, or says what it waits for.
    ///
    /// The per-server limit is asked first. Both readings can be true at once,
    /// and the one to report is the narrower cause: a caller told every slot is
    /// in use looks at the other servers, and a caller told this server is at its
    /// six knows the wait ends when one of its own answers.
    ///
    /// # Errors
    ///
    /// [`Waits`], naming which of the two limits it is behind.
    pub fn start(&mut self, origin: &Origin) -> Result<(), Waits> {
        if self.against(origin) >= REQUESTS_OUTSTANDING_TO_ONE_SERVER {
            return Err(Waits::ThisServerIsAtItsSix);
        }
        if self.total() >= REQUESTS_OUTSTANDING_ACROSS_ALL_SERVERS {
            return Err(Waits::EverySlotIsInUse);
        }
        match self.against.iter_mut().find(|(held, _)| held == origin) {
            Some((_, count)) => *count += 1,
            None => self.against.push((origin.clone(), 1)),
        }
        Ok(())
    }

    /// Ends one request against `origin`.
    ///
    /// Ending one that was never started does nothing, and the lookup is what
    /// makes that true. The mistake this shape is written against is the one
    /// that reads better: finding the entry and unwrapping it, which turns an
    /// end with no start into a panic inside the lane 0009 puts every wait on.
    ///
    /// THERE IS NO FLOOR UNDER THE SUBTRACTION AND ONE WOULD BE UNREACHABLE. A
    /// kept count is never zero, because the entry is removed at the moment it
    /// reaches zero two lines below, so the only way into the subtraction is
    /// through a count of at least one. A `saturating_sub` here was written
    /// first, and deleting it left the suite green: it was a guard that could
    /// not fail, which is worse than none because somebody would have relied on
    /// it.
    pub fn end(&mut self, origin: &Origin) {
        if let Some(at) = self.against.iter().position(|(held, _)| held == origin) {
            let (_, count) = &mut self.against[at];
            *count -= 1;
            if *count == 0 {
                self.against.swap_remove(at);
            }
        }
    }

    /// How many requests are outstanding against one server.
    #[must_use]
    pub fn against(&self, origin: &Origin) -> usize {
        self.against
            .iter()
            .find(|(held, _)| held == origin)
            .map_or(0, |(_, count)| *count)
    }

    /// How many are outstanding across every server.
    #[must_use]
    pub fn total(&self) -> usize {
        self.against.iter().map(|(_, count)| *count).sum()
    }
}

/// Why a connection was closed rather than kept.
///
/// 0027 names five and this is that list. It is exhaustive on purpose: a sixth
/// reason is a change to that record, and a caller matching on this is told by
/// the compiler when one arrives rather than falling into a branch somebody
/// wrote for something else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndsAConnection {
    /// Nothing used it for the minute in [`AN_IDLE_CONNECTION_IS_KEPT_FOR`].
    ItWasIdleTooLong,
    /// The server closed it. This needs no rule and is here so the list is the
    /// whole list.
    TheServerClosedIt,
    /// The core did not read the response to its end, so bytes are still on it.
    /// 0009 says a connection with unread bytes cannot be reused.
    ABodyWasNotReadToItsEnd,
    /// A cancelled response went past [`A_CANCELLED_BODY_IS_READ_TO`] or
    /// [`A_CANCELLED_BODY_IS_READ_FOR`].
    ACancelledBodyWentPastTheBound,
    /// The pin for that server changed under 0029, so every connection accepted
    /// under the pin that has just been replaced goes with it. This is the one
    /// case where reuse would carry an old answer forward.
    ThePinForThatServerChanged,
}

/// Connections nothing is using, kept for reuse.
///
/// 0027 reuses a connection for any request to the same origin, including for a
/// different session against that server, because identity travels in the
/// request and the core presents no certificate of its own. It is never reused
/// across origins.
///
/// The connection is a type parameter. Nothing in this module opens one, for the
/// reason the module documentation gives, and writing the pool over whatever a
/// connection turns out to be is what lets the rule about how long an idle one
/// lives be proven before the thing it is about exists.
///
/// Thread safety, from 0009: as thread-safe as the connection it holds. The lane
/// that owns the pool is what serialises access to it.
#[derive(Debug)]
pub struct IdleConnections<C> {
    kept: Vec<(Origin, C, SteadyInstant)>,
}

impl<C> Default for IdleConnections<C> {
    fn default() -> Self {
        Self::none()
    }
}

impl<C> IdleConnections<C> {
    /// A pool holding nothing.
    #[must_use]
    pub const fn none() -> Self {
        Self { kept: Vec::new() }
    }

    /// Keeps a connection that has just gone idle.
    pub fn keep(&mut self, origin: Origin, connection: C, now: SteadyInstant) {
        self.kept.push((origin, connection, now));
    }

    /// Hands back a connection to `origin` that is still inside its minute, or
    /// nothing.
    ///
    /// The most recently used one is taken first. Two idle connections to one
    /// server differ only in how much of their minute is left, and taking the
    /// one with more of it left means the other reaches its end and is closed,
    /// rather than both being kept alive forever by alternating use.
    ///
    /// This does not close what it passes over. A caller that wants the expired
    /// ones takes them with [`IdleConnections::take_the_expired`], and keeping
    /// the two apart is what stops a connection being closed inside a call that
    /// was only asking for one.
    pub fn take(&mut self, origin: &Origin, now: SteadyInstant) -> Option<C> {
        let at = self
            .kept
            .iter()
            .enumerate()
            .filter(|(_, (held, _, since))| held == origin && !Self::expired(*since, now))
            .max_by_key(|(_, (_, _, since))| *since)
            .map(|(at, _)| at)?;
        let (_, connection, _) = self.kept.swap_remove(at);
        Some(connection)
    }

    /// Takes every connection whose minute has ended, for the caller to close.
    ///
    /// They are returned rather than dropped, because closing a socket is the
    /// socket layer's and this module holds none. The reason is
    /// [`EndsAConnection::ItWasIdleTooLong`] for every one of them.
    pub fn take_the_expired(&mut self, now: SteadyInstant) -> Vec<C> {
        let mut ended = Vec::new();
        let mut at = 0;
        while at < self.kept.len() {
            if Self::expired(self.kept[at].2, now) {
                let (_, connection, _) = self.kept.swap_remove(at);
                ended.push(connection);
            } else {
                at += 1;
            }
        }
        ended
    }

    /// How many connections are being kept.
    #[must_use]
    pub fn kept(&self) -> usize {
        self.kept.len()
    }

    /// Whether a connection idle since `since` has outlived its minute.
    ///
    /// The comparison is inclusive. A connection exactly at the bound is closed
    /// rather than handed back, because the bound is the point at which the core
    /// stops being able to know the connection survived, and the two readings
    /// that produce equality are the same instant.
    fn expired(since: SteadyInstant, now: SteadyInstant) -> bool {
        now.interval_since(since) >= AN_IDLE_CONNECTION_IS_KEPT_FOR
    }
}

/// How far the remainder of a cancelled response has been read.
///
/// 0009 says bytes in flight are read and discarded rather than left in the
/// socket, because a connection with unread bytes on it cannot be reused, and it
/// leaves open how far that reading goes. 0027 closes it at sixty-four kilobytes
/// or one second, whichever comes first, and this is that pair counted.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ACancelledBody {
    began: SteadyInstant,
    read: usize,
}

impl ACancelledBody {
    /// A drain beginning now, with nothing read.
    #[must_use]
    pub const fn beginning_at(began: SteadyInstant) -> Self {
        Self { began, read: 0 }
    }

    /// Records bytes that were read and discarded.
    pub fn read(&mut self, bytes: usize) {
        self.read = self.read.saturating_add(bytes);
    }

    /// How many have been read.
    #[must_use]
    pub const fn read_so_far(self) -> usize {
        self.read
    }

    /// Whether reading on is still worth more than the handshake it saves.
    ///
    /// Both bounds are asked, and either ends it. A body that arrives fast hits
    /// the byte bound and a body that trickles hits the time bound, and those
    /// are two different servers rather than two spellings of one.
    #[must_use]
    pub fn may_read_on(self, now: SteadyInstant) -> bool {
        self.read < A_CANCELLED_BODY_IS_READ_TO
            && now.interval_since(self.began) < A_CANCELLED_BODY_IS_READ_FOR
    }
}

#[cfg(test)]
mod tests {
    use super::{
        A_CALL_IS_ABANDONED_AFTER, A_CANCELLED_BODY_IS_READ_TO, ACancelledBody, AttemptBound,
        CallDeadline, EndsAConnection, IdleConnections, Outstanding,
        REQUESTS_OUTSTANDING_ACROSS_ALL_SERVERS, REQUESTS_OUTSTANDING_TO_ONE_SERVER, Waits,
    };
    use crate::clock::SteadyInstant;
    use crate::failure::Deadline;
    use crate::server::address::{BaseAddress, Origin};
    use core::time::Duration;

    const A_SECOND: u64 = 1_000_000_000;

    fn at(seconds: u64) -> SteadyInstant {
        // Above zero, for the reason the controlled source in the suite starts
        // above zero: a reading at the origin cannot be told from one never
        // taken.
        SteadyInstant::from_nanos(A_SECOND + seconds * A_SECOND)
    }

    fn after(seconds: u64, millis: u64) -> SteadyInstant {
        SteadyInstant::from_nanos(A_SECOND + seconds * A_SECOND + millis * 1_000_000)
    }

    fn origin(typed: &str) -> Origin {
        BaseAddress::parse(typed)
            .expect("an address the parse admits")
            .origin()
    }

    #[test]
    fn an_origin_is_the_scheme_the_host_and_the_port_and_never_the_path() {
        let root = origin("https://server.invalid:8920");
        let sub = origin("https://server.invalid:8920/jellyfin");
        assert_eq!(root, sub, "0069 puts the path outside the origin");
        assert_eq!(root.host(), "server.invalid");
        assert_eq!(root.port(), Some(8920));
    }

    /// An absent port is not the scheme's default filled in. Two addresses that
    /// reach the same machine are still two origins here, because 0028 supplies
    /// nothing and an origin that guessed would be a comparison this core makes
    /// and that record does not.
    #[test]
    fn an_absent_port_is_not_the_same_origin_as_the_number_written_out() {
        assert_ne!(
            origin("https://server.invalid"),
            origin("https://server.invalid:443")
        );
    }

    #[test]
    fn a_connection_is_never_reused_across_a_scheme_a_host_or_a_port() {
        let base = origin("https://server.invalid:8920");
        assert_ne!(base, origin("http://server.invalid:8920"));
        assert_ne!(base, origin("https://other.invalid:8920"));
        assert_ne!(base, origin("https://server.invalid:8921"));
    }

    #[test]
    fn a_call_reports_what_is_left_of_its_five_seconds() {
        let call = CallDeadline::beginning_at(at(0));
        assert_eq!(call.left_at(at(0)), A_CALL_IS_ABANDONED_AFTER);
        assert_eq!(call.left_at(at(2)), Duration::from_secs(3));
        assert!(!call.passed_at(after(4, 999)));
        assert!(call.passed_at(at(5)));
    }

    /// The floor rather than a wrap. A call read past its deadline has nothing
    /// left, and the unguarded subtraction would hand a caller the largest
    /// duration this core can express as the time it may still wait.
    #[test]
    fn a_call_read_past_its_deadline_has_nothing_left_and_not_everything() {
        let call = CallDeadline::beginning_at(at(0));
        assert_eq!(call.left_at(at(60)), Duration::ZERO);
    }

    #[test]
    fn an_attempt_early_in_a_call_runs_against_its_own_two_seconds() {
        let call = CallDeadline::beginning_at(at(0));
        let bound = AttemptBound::reaching_a_connection(call, at(0));
        assert_eq!(bound.within(), Duration::from_secs(2));
        assert_eq!(bound.named(), Deadline::Connect);

        let bound = AttemptBound::reaching_the_first_byte(call, at(1));
        assert_eq!(bound.within(), Duration::from_secs(2));
        assert_eq!(bound.named(), Deadline::FirstByte);
    }

    /// 0027's own sentence: an attempt is bounded by whichever arrives first,
    /// its own two seconds or what is left of the caller's five. This is the
    /// second half, which a per-attempt timeout written on its own would get
    /// wrong by letting a third attempt run four seconds past the call.
    #[test]
    fn an_attempt_late_in_a_call_runs_against_what_is_left_of_the_call() {
        let call = CallDeadline::beginning_at(at(0));
        let bound = AttemptBound::reaching_the_first_byte(call, after(4, 500));
        assert_eq!(bound.within(), Duration::from_millis(500));
        assert_eq!(
            bound.named(),
            Deadline::WholeRequest,
            "a failure at this bound is 0007's deadline and not 0027's"
        );
    }

    /// A tie goes to the call. Both readings are true at that instant, one name
    /// can be carried, and the call is the one a caller can act on.
    #[test]
    fn a_bound_that_ties_is_reported_as_the_call_deadline() {
        let call = CallDeadline::beginning_at(at(0));
        let bound = AttemptBound::reaching_a_connection(call, at(3));
        assert_eq!(bound.within(), Duration::from_secs(2));
        assert_eq!(bound.named(), Deadline::WholeRequest);
    }

    #[test]
    fn an_attempt_reaches_its_bound_at_the_bound_and_not_after_it() {
        let call = CallDeadline::beginning_at(at(0));
        let bound = AttemptBound::reaching_a_connection(call, at(0));
        assert!(!bound.reached_at(at(0), after(1, 999)));
        assert!(bound.reached_at(at(0), at(2)));
    }

    #[test]
    fn six_requests_reach_one_server_and_the_seventh_waits_for_that_server() {
        let mut outstanding = Outstanding::none();
        let server = origin("https://server.invalid");
        for _ in 0..REQUESTS_OUTSTANDING_TO_ONE_SERVER {
            outstanding.start(&server).expect("a slot for this server");
        }
        assert_eq!(outstanding.start(&server), Err(Waits::ThisServerIsAtItsSix));
        assert_eq!(outstanding.against(&server), 6);
    }

    #[test]
    fn twelve_requests_reach_two_servers_and_the_thirteenth_waits_for_any_of_them() {
        let mut outstanding = Outstanding::none();
        let first = origin("https://first.invalid");
        let second = origin("https://second.invalid");
        let third = origin("https://third.invalid");
        for _ in 0..REQUESTS_OUTSTANDING_TO_ONE_SERVER {
            outstanding.start(&first).expect("a slot for the first");
            outstanding.start(&second).expect("a slot for the second");
        }
        assert_eq!(outstanding.total(), REQUESTS_OUTSTANDING_ACROSS_ALL_SERVERS);
        assert_eq!(outstanding.start(&third), Err(Waits::EverySlotIsInUse));
    }

    /// The narrower cause is the one reported. A caller told every slot is in
    /// use looks at the other servers; a caller told this server is at its six
    /// knows the wait ends when one of its own answers.
    #[test]
    fn a_server_at_its_six_inside_a_full_twelve_is_told_about_its_own_six() {
        let mut outstanding = Outstanding::none();
        let first = origin("https://first.invalid");
        let second = origin("https://second.invalid");
        for _ in 0..REQUESTS_OUTSTANDING_TO_ONE_SERVER {
            outstanding.start(&first).expect("a slot for the first");
            outstanding.start(&second).expect("a slot for the second");
        }
        assert_eq!(outstanding.start(&first), Err(Waits::ThisServerIsAtItsSix));
    }

    #[test]
    fn ending_a_request_frees_the_slot_for_that_server_and_for_the_total() {
        let mut outstanding = Outstanding::none();
        let server = origin("https://server.invalid");
        for _ in 0..REQUESTS_OUTSTANDING_TO_ONE_SERVER {
            outstanding.start(&server).expect("a slot");
        }
        outstanding.end(&server);
        assert_eq!(outstanding.against(&server), 5);
        assert_eq!(outstanding.total(), 5);
        outstanding.start(&server).expect("the freed slot");
        assert_eq!(outstanding.against(&server), 6);
    }

    /// The lookup rather than an unwrap. Ending a request that never started is
    /// not a caller mistake worth a panic: it is what a cancellation racing a
    /// completion looks like from here, and 0009 puts both on the waiting lane.
    #[test]
    fn ending_a_request_that_never_started_leaves_the_count_where_it_was() {
        let mut outstanding = Outstanding::none();
        let server = origin("https://server.invalid");
        outstanding.end(&server);
        assert_eq!(outstanding.against(&server), 0);
        assert_eq!(outstanding.total(), 0);
        outstanding.start(&server).expect("a slot after the miss");
        assert_eq!(outstanding.against(&server), 1);
    }

    #[test]
    fn an_idle_connection_is_handed_back_to_the_same_origin() {
        let mut idle = IdleConnections::none();
        let server = origin("https://server.invalid");
        idle.keep(server.clone(), "a connection", at(0));
        assert_eq!(idle.take(&server, at(30)), Some("a connection"));
        assert_eq!(idle.kept(), 0);
    }

    #[test]
    fn an_idle_connection_is_never_handed_to_another_origin() {
        let mut idle = IdleConnections::none();
        idle.keep(origin("https://server.invalid"), "a connection", at(0));
        assert_eq!(idle.take(&origin("https://other.invalid"), at(1)), None);
        assert_eq!(idle.kept(), 1, "it is passed over rather than closed");
    }

    #[test]
    fn a_connection_past_its_minute_is_not_handed_back() {
        let mut idle = IdleConnections::none();
        let server = origin("https://server.invalid");
        idle.keep(server.clone(), "a connection", at(0));
        assert_eq!(idle.take(&server, after(59, 999)), Some("a connection"));

        idle.keep(server.clone(), "a connection", at(0));
        assert_eq!(idle.take(&server, at(60)), None);
    }

    #[test]
    fn the_connections_past_their_minute_are_handed_over_to_be_closed() {
        let mut idle = IdleConnections::none();
        let server = origin("https://server.invalid");
        idle.keep(server.clone(), "the old one", at(0));
        idle.keep(server.clone(), "the new one", at(30));
        assert_eq!(idle.take_the_expired(at(61)), vec!["the old one"]);
        assert_eq!(idle.kept(), 1);
        assert_eq!(idle.take(&server, at(61)), Some("the new one"));
    }

    /// The one with more of its minute left is taken, so the other reaches its
    /// end and is closed. Taking the older one keeps both alive forever under
    /// alternating use, which is the leak that reads as a working pool.
    #[test]
    fn the_connection_with_the_most_of_its_minute_left_is_taken_first() {
        let mut idle = IdleConnections::none();
        let server = origin("https://server.invalid");
        idle.keep(server.clone(), "the older one", at(0));
        idle.keep(server.clone(), "the newer one", at(30));
        assert_eq!(idle.take(&server, at(31)), Some("the newer one"));
    }

    #[test]
    fn a_cancelled_body_is_read_while_it_is_cheaper_than_a_handshake() {
        let mut body = ACancelledBody::beginning_at(at(0));
        assert!(body.may_read_on(at(0)));
        body.read(1024);
        assert_eq!(body.read_so_far(), 1024);
        assert!(body.may_read_on(after(0, 500)));
    }

    #[test]
    fn a_cancelled_body_past_the_byte_bound_closes_the_connection_instead() {
        let mut body = ACancelledBody::beginning_at(at(0));
        body.read(A_CANCELLED_BODY_IS_READ_TO - 1);
        assert!(body.may_read_on(at(0)));
        body.read(1);
        assert!(
            !body.may_read_on(at(0)),
            "an artwork body under 0055 is far past this, so a cancelled tile closes"
        );
    }

    #[test]
    fn a_cancelled_body_past_the_second_closes_the_connection_instead() {
        let mut body = ACancelledBody::beginning_at(at(0));
        body.read(16);
        assert!(body.may_read_on(after(0, 999)));
        assert!(
            !body.may_read_on(at(1)),
            "a body that trickles hits the time bound rather than the byte bound"
        );
    }

    /// The count saturates rather than wrapping. A read that overflowed the
    /// count would make a drained body read as one that has read nothing, which
    /// is the bound removed rather than reached.
    #[test]
    fn a_cancelled_body_that_read_more_than_a_count_can_hold_stays_past_the_bound() {
        let mut body = ACancelledBody::beginning_at(at(0));
        body.read(usize::MAX);
        body.read(usize::MAX);
        assert_eq!(body.read_so_far(), usize::MAX);
        assert!(!body.may_read_on(at(0)));
    }

    /// 0027 names five things that end a connection and the set is closed. A
    /// sixth is a change to that record, and this is what tells a reader the
    /// list here is the whole list rather than the ones somebody needed first.
    #[test]
    fn every_reason_zero_zero_two_seven_names_has_a_value() {
        let all = [
            EndsAConnection::ItWasIdleTooLong,
            EndsAConnection::TheServerClosedIt,
            EndsAConnection::ABodyWasNotReadToItsEnd,
            EndsAConnection::ACancelledBodyWentPastTheBound,
            EndsAConnection::ThePinForThatServerChanged,
        ];
        assert_eq!(all.len(), 5);
        for (i, one) in all.iter().enumerate() {
            for other in &all[i + 1..] {
                assert_ne!(one, other, "two reasons that compare equal are one reason");
            }
        }
    }
}
