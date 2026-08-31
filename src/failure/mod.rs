//! The error vocabulary every client shares.
//!
//! This is not one of the six things 0003 names. That record places the mapping
//! of every failure onto one vocabulary inside "reaching a server", and every
//! other module here produces failures too, so the vocabulary sits beside the six
//! rather than inside one of them. The records are 0004 and 0037, and the issues
//! are #4 and #37.
//!
//! 0037 requires one point at which a failure becomes a kind, with nothing
//! falling through to a default. On the chosen means that is a refusal rather
//! than a convention: a value of the set is built inside this module and the
//! compiler refuses construction anywhere else. 0011 carries the measurement that
//! says so.
//!
//! # How the refusal is written
//!
//! Every variant of [`Failure`] carries a field of type [`Constructed`], and that
//! type has one field which is private. Nothing outside this module can make one,
//! so nothing outside this module can name a variant of [`Failure`] in an
//! expression, while every field a caller reads stays public and a `match` over
//! the fifteen stays exhaustive. That is what 0004 asks of a client and what 0037
//! asks of the core, in one shape rather than two.
//!
//! A caller writes `Failure::NotFound { identifier, .. }` in a pattern and cannot
//! write it in an expression. The `..` is what hides the token, and it is also
//! why a field added to a variant later does not break a caller.
//!
//! # What arrives here
//!
//! 0004's three sources, and the two conditions the core raises about itself.
//! [`Failure::from_transport`] takes a transport outcome classified before any
//! HTTP exists. [`Failure::from_status`] reads 0004's status table. A
//! server-supplied error body reaches the same call as payload and never changes
//! the kind, which is why the code is a field of [`Answered`] rather than an
//! argument that could decide anything. [`Failure::answer_not_understood`] is the
//! fourth rule, and [`Failure::storage_unavailable`],
//! [`Failure::internal_fault`] and [`Failure::cancelled`] are mapped from
//! nothing and built here anyway, for the reason 0037 gives: a second
//! construction site is a second place a sixteenth thing can be invented.
//!
//! # What is not here, said once so a green build is not read as covering it
//!
//! **No caller in this tree reaches any of it yet.** The transport is #27, the
//! query surface is #39 and the artwork fetch is #49, so what exists today is the
//! point and the suite that drives it. 0037 already says the honest proof that
//! every failure went through this point is a check over the tree rather than a
//! test, because a test proves the sites it reached.
//!
//! **The retry-after hint 0004 describes as given or assumed is carried as given
//! or absent.** The assumed value is a delay 0038 decides and #38 is not built, so
//! no number is invented here, and a caller sees the absence rather than a
//! duration nothing chose.
//!
//! **`answer-not-understood` carries where reading stopped as an offset and the
//! reading it stopped inside, and not which field.** 0037 asks for the field as
//! well. Nothing in this tree parses a body, so there is no declared set of field
//! names for a body to name one out of, and a free string is the shape that
//! record refuses by name. That half is owed.

use crate::cache::StorageUnavailable;
use crate::server::address::{AddressNotUsable, UnusablePart};
use crate::session::SecretStoreUnavailable;
use core::time::Duration;

/// The token every variant of [`Failure`] carries.
///
/// Its one field is private, so a value of it can be made in this module and
/// nowhere else, and a variant carrying one can be built in this module and
/// nowhere else. That is 0037's single construction point, held by the compiler
/// rather than by a convention somebody follows.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constructed(());

/// Which of the fifteen a failure is, without its payload.
///
/// It exists so that a failure can be counted and grouped - which is 0004's own
/// reversal condition, read off the diagnostic events in #100 - without every
/// counting site matching fifteen variants and reaching into their fields.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// What was typed as a server address cannot be turned into somewhere to
    /// send a request.
    AddressNotUsable,
    /// Nothing at that address answered.
    ServerUnreachable,
    /// Something answered too slowly, or stopped answering part way.
    TimedOut,
    /// The machine that answered did not prove it is the one the address named.
    CertificateRejected,
    /// There is no session the server accepts.
    NotAuthenticated,
    /// The session is valid and this account may not do this.
    NotPermitted,
    /// The thing asked for is not on that server.
    NotFound,
    /// The server understood the request and rejected it as wrong.
    RequestRefused,
    /// The server is refusing load, not this request.
    ServerBusy,
    /// The server broke while handling a request that was not wrong.
    ServerFailed,
    /// Something arrived and it is not a shape the core knows how to read.
    AnswerNotUnderstood,
    /// This server does not offer the part of its interface this call needs.
    CapabilityAbsent,
    /// The caller asked for this to stop, and it stopped.
    Cancelled,
    /// A store the client supplied could not be read or written.
    StorageUnavailable,
    /// A defect in the core.
    InternalFault,
}

impl Kind {
    /// The name this kind is written as, in the spelling 0004's table uses.
    ///
    /// One place rather than one per reporting route, for the reason 0061 gives
    /// about span names: a set declared once can be printed, counted and grouped,
    /// and a literal at a call site can only be read by whoever is looking at
    /// that line.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::AddressNotUsable => "address-not-usable",
            Self::ServerUnreachable => "server-unreachable",
            Self::TimedOut => "timed-out",
            Self::CertificateRejected => "certificate-rejected",
            Self::NotAuthenticated => "not-authenticated",
            Self::NotPermitted => "not-permitted",
            Self::NotFound => "not-found",
            Self::RequestRefused => "request-refused",
            Self::ServerBusy => "server-busy",
            Self::ServerFailed => "server-failed",
            Self::AnswerNotUnderstood => "answer-not-understood",
            Self::CapabilityAbsent => "capability-absent",
            Self::Cancelled => "cancelled",
            Self::StorageUnavailable => "storage-unavailable",
            Self::InternalFault => "internal-fault",
        }
    }
}

/// The sixteen capabilities 0010 fixes.
///
/// `capability-absent` carries one of these, and 0004 says so: the name comes
/// from the set #10 fixes rather than from a string written where the failure is
/// raised. Two of them have no route on either supported line and are in the set
/// anyway, because a capability with no name cannot be reported.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Capability {
    /// The identifier and version the server reports about itself.
    ServerIdentity,
    /// 0030's route.
    PasswordSignIn,
    /// 0031's exchange, from the operator's setting to the session.
    QuickConnect,
    /// 0032's route, which no supported line offers.
    DelegatedSignIn,
    /// 0034's exchange of a live token for a fresh one, which no supported line
    /// offers.
    TokenRenewal,
    /// 0114's ending of one session.
    SignOut,
    /// The description 0036 says a client supplies.
    DeviceCapabilities,
    /// The top of the library and the query surface #39 builds on.
    LibraryQuery,
    /// One item, in full.
    ItemDetail,
    /// What 0058 resumes into.
    ResumeList,
    /// The position and the watched mark, read and written.
    ItemUserData,
    /// The bytes #49 builds an address for.
    Artwork,
    /// 0111's choice of source.
    PlaybackSelection,
    /// 0057's cadence, and the two reports around it.
    PlaybackProgress,
    /// 0060's mark, set and cleared.
    PlayedMarking,
    /// 0116's connection.
    ChangeNotification,
}

impl Capability {
    /// The name this capability is written as, in 0010's own spelling.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::ServerIdentity => "server-identity",
            Self::PasswordSignIn => "password-sign-in",
            Self::QuickConnect => "quick-connect",
            Self::DelegatedSignIn => "delegated-sign-in",
            Self::TokenRenewal => "token-renewal",
            Self::SignOut => "sign-out",
            Self::DeviceCapabilities => "device-capabilities",
            Self::LibraryQuery => "library-query",
            Self::ItemDetail => "item-detail",
            Self::ResumeList => "resume-list",
            Self::ItemUserData => "item-user-data",
            Self::Artwork => "artwork",
            Self::PlaybackSelection => "playback-selection",
            Self::PlaybackProgress => "playback-progress",
            Self::PlayedMarking => "played-marking",
            Self::ChangeNotification => "change-notification",
        }
    }
}

/// Where the core was reading when it produced `answer-not-understood`.
///
/// 0037 requires this to be a value from a set declared in one place and never a
/// string written where the failure is raised, and says why: 0004's reversal
/// condition is that this kind becomes the one an operator sees most, 0069 pushed
/// a refused cross-origin redirect under the same kind, 0055 pushed a refused
/// image format under it, and a count of one kind cannot choose between the three
/// repairs behind it. Grouping by this value is what turns one number into three.
///
/// Adding a site is not a change to 0037. Removing one, or collapsing two into
/// one, is, because that is the granularity the measurement is read at.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReadingSite {
    /// The status line, where the number it carried is outside every row of
    /// 0004's table.
    StatusLine,
    /// The body of an answer the core was reading.
    AnswerBody,
    /// A redirect leaving the origin the request was sent to. 0069 refuses it and
    /// takes this kind knowing the fit is imperfect: the core understood the
    /// shape and declined it. This value is what lets that population be counted
    /// on its own.
    CrossOriginRedirectRefused,
    /// An image in a format 0055 does not accept. The same imperfect fit, counted
    /// separately for the same reason.
    ImageFormatRefused,
}

impl ReadingSite {
    /// The name this site is written as, for the grouping the measurement needs.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::StatusLine => "status-line",
            Self::AnswerBody => "answer-body",
            Self::CrossOriginRedirectRefused => "cross-origin-redirect-refused",
            Self::ImageFormatRefused => "image-format-refused",
        }
    }
}

/// What the core expected where it found something it could not read.
///
/// It is a declared set for the reason the site is: an expectation written as a
/// sentence where the failure is raised cannot be grouped, and it is a field
/// somebody eventually puts a value from the answer into. The members are 0004's
/// own fourth-rule list rather than a set invented here.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expected {
    /// A status one of 0004's rows names.
    AStatusTheTableNames,
    /// A body in a shape the core can read at all.
    ABodyTheCoreCanRead,
    /// A field the core needs, in a body that otherwise parsed.
    AFieldTheCoreNeeds,
    /// A value inside a set the core knows, in a field that was present.
    AValueInsideASetTheCoreKnows,
    /// One of the three formats 0055 accepts.
    AnAcceptedImageFormat,
    /// An origin 0069 admits.
    AnOriginTheCoreMayContact,
}

/// Which defect in the core an `internal-fault` came from.
///
/// 0004 requires a stable identifier for the site, and 0037 declares it the same
/// way as [`ReadingSite`] and for the same reason, as a second set rather than as
/// more members of the first: the two answer different questions, and a shared
/// set would be one where half the values can never appear on half the kinds.
///
/// **One value, and nothing outside this module raises it.** The core has no
/// subsystem that can hold a defect about itself yet, so the only site is the one
/// this module can reach, and adding a site when a subsystem lands is a variant
/// here rather than a change to 0037.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FaultSite {
    /// A status inside the successful range was handed to the status mapping.
    /// 0004's table has a row for a 2xx whose body does not parse, and that row
    /// produces `answer-not-understood` through the fourth rule rather than
    /// through the table, so a 2xx arriving here is the core asking which failure
    /// a success is.
    SuccessMappedAsFailure,
}

impl FaultSite {
    /// The name this site is written as.
    #[must_use]
    pub const fn declared_name(self) -> &'static str {
        match self {
            Self::SuccessMappedAsFailure => "success-mapped-as-failure",
        }
    }
}

/// Which of the three deadlines 0027 sets expired.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deadline {
    /// The two seconds an attempt has to reach a connection.
    Connect,
    /// The two seconds an answer has to begin after the request is written.
    FirstByte,
    /// The five seconds 0007 sets for the whole request, which 0027 does not
    /// decide and does not move.
    WholeRequest,
}

/// Why a presented certificate was refused, in 0029's six classes.
///
/// The classes are written down rather than left to the platform, because a class
/// nobody wrote down becomes a string a client switches on.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateReason {
    /// The certificate signed itself, and nothing else vouches for it.
    SelfSigned,
    /// The chain ends at something the platform's trust store does not hold.
    IssuerUnknown,
    /// The chain is trusted and names a machine other than the one addressed.
    NameMismatch,
    /// The chain is trusted and its validity window has ended.
    Expired,
    /// The chain is trusted and its validity window has not begun.
    NotYetValid,
    /// Anything else the platform refused. This is the one that keeps the set
    /// closed.
    ChainUnusable,
}

/// Which store answered badly.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    /// The cache store 0040 puts behind the core, from #40.
    Cache,
    /// The secret store 0033 asks a client for, from #33.
    Secret,
}

/// Which way the store was being used.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Reading, where an absence is not this and a failure to answer is.
    Read,
    /// Writing.
    Write,
}

/// What the transport found, before any HTTP exists.
///
/// 0004 classifies these first, and the split between them is the split that
/// record makes: something that never answered, something that answered too
/// slowly or stopped, and a peer that did not prove who it is.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportOutcome<'a> {
    /// The name did not resolve.
    NameDidNotResolve,
    /// The connection was refused.
    ConnectionRefused,
    /// The network could not be reached.
    NetworkUnreachable,
    /// The connection dropped part way through a body.
    ConnectionDroppedMidBody,
    /// A deadline was reached with no answer.
    DeadlineReached {
        /// Which of 0027's three it was.
        deadline: Deadline,
        /// How long it ran before it expired. 0102 puts this on the steady clock.
        elapsed: Duration,
    },
    /// An answer began and then stalled part way through the body.
    AnswerStalledMidBody {
        /// Which of 0027's three it was.
        deadline: Deadline,
        /// How long it ran before it expired.
        elapsed: Duration,
    },
    /// The peer was not trusted.
    PeerNotTrusted {
        /// Which of 0029's six classes it was.
        reason: CertificateReason,
        /// The fingerprint of the certificate at the end of the presented chain,
        /// which is what a person compares against their own server.
        fingerprint: &'a str,
    },
}

/// What the attempt was, which every transport failure carries.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attempt<'a> {
    /// The address that was contacted.
    pub address: &'a str,
    /// Whether any bytes reached the server before it failed. 0004 makes this its
    /// own field because it separates a call that certainly did not happen from
    /// one that may have, and a write that may have happened is what #47 has to
    /// decide about.
    pub bytes_reached_the_server: bool,
}

/// What a server answered, beside its status.
///
/// A server-supplied error body may add payload and may never change the kind,
/// which is why nothing here is an argument the mapping branches on.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answered<'a> {
    /// The capability the call was made under, from 0010's set.
    pub capability: Capability,
    /// The caller-supplied identifier in the path, where the path carries one.
    ///
    /// This is the only thing outside the status that decides a kind, and 0004
    /// says so: the 404 split is decided by the core's own list of what the
    /// interface holds rather than by anything in the response. `None` is a path
    /// carrying nothing the caller named, where the only thing a 404 can be about
    /// is the route.
    pub identifier: Option<&'a str>,
    /// The retry-after hint, where the server gave one.
    pub retry_after: Option<Duration>,
    /// The server-supplied error code, where the server gave one, opaque.
    pub server_code: Option<&'a str>,
}

/// One of the fifteen kinds 0004 fixes, with the payload that kind carries.
///
/// Every variant carries a [`Constructed`], which is what stops one being built
/// outside this module. Read the module documentation before adding a variant:
/// a sixteenth kind is a change to 0004 and to every client.
///
/// Thread safety, from 0009: a value, safe from any thread. It is handed back to
/// a caller and never mutated afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// What was typed cannot be turned into somewhere to send a request.
    AddressNotUsable {
        /// The address as it was given, unmodified.
        typed: String,
        /// Which part of it could not be used.
        part: UnusablePart,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// Nothing at that address answered.
    ServerUnreachable {
        /// The address that was contacted.
        address: String,
        /// Whether any bytes reached the server before it failed.
        bytes_reached_the_server: bool,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// Something answered too slowly, or stopped answering part way.
    TimedOut {
        /// Which deadline expired.
        deadline: Deadline,
        /// The elapsed time it expired after.
        elapsed: Duration,
        /// Whether any bytes reached the server before it failed.
        bytes_reached_the_server: bool,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The machine that answered did not prove it is the one the address named.
    CertificateRejected {
        /// The address that was contacted.
        address: String,
        /// Which of 0029's six classes it was.
        reason: CertificateReason,
        /// The presented certificate's fingerprint.
        fingerprint: String,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// There is no session the server accepts.
    NotAuthenticated {
        /// Whether a token was presented and rejected, or there was none to
        /// present. #34 and #35 act on that difference.
        a_token_was_presented: bool,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The session is valid and this account may not do this. It carries nothing.
    NotPermitted {
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The thing asked for is not on that server.
    NotFound {
        /// The identifier that was asked for.
        identifier: String,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The server understood the request and rejected it as wrong.
    RequestRefused {
        /// The server-supplied error code where the server gave one, opaque.
        server_code: Option<String>,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The server is refusing load, not this request.
    ServerBusy {
        /// The retry-after hint where the server gave one. Where it gave none,
        /// this is `None` rather than a duration nothing chose: 0004 describes
        /// the alternative as assumed, and the assumed value is 0038's, which is
        /// not built.
        retry_after: Option<Duration>,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The server broke while handling a request that was not wrong.
    ServerFailed {
        /// The status, opaque.
        status: u16,
        /// The server-supplied error code where there is one, opaque.
        server_code: Option<String>,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// Something arrived and it is not a shape the core knows how to read.
    AnswerNotUnderstood {
        /// What the core was reading, from the declared set.
        site: ReadingSite,
        /// What it expected there.
        expected: Expected,
        /// Where in the answer it stopped, as an offset. Never the bytes at that
        /// offset: 0004 forbids carrying the answer, because an answer holds
        /// library contents and may hold a token.
        stopped_at: usize,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// This server does not offer the part of its interface this call needs.
    CapabilityAbsent {
        /// Which capability, from 0010's set.
        capability: Capability,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// The caller asked for this to stop, and it stopped. It carries nothing.
    Cancelled {
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// A store the client supplied could not be read or written.
    StorageUnavailable {
        /// Which store it was.
        store: Store,
        /// Whether the failure was a read or a write.
        operation: Operation,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
    /// A defect in the core. Nothing about the server or the network is claimed.
    InternalFault {
        /// A stable identifier for the site that produced it, from the declared
        /// set, and nothing derived from the data being handled.
        site: FaultSite,
        /// The token this module builds and nothing else can.
        constructed: Constructed,
    },
}

impl Failure {
    /// Which of the fifteen this is.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::AddressNotUsable { .. } => Kind::AddressNotUsable,
            Self::ServerUnreachable { .. } => Kind::ServerUnreachable,
            Self::TimedOut { .. } => Kind::TimedOut,
            Self::CertificateRejected { .. } => Kind::CertificateRejected,
            Self::NotAuthenticated { .. } => Kind::NotAuthenticated,
            Self::NotPermitted { .. } => Kind::NotPermitted,
            Self::NotFound { .. } => Kind::NotFound,
            Self::RequestRefused { .. } => Kind::RequestRefused,
            Self::ServerBusy { .. } => Kind::ServerBusy,
            Self::ServerFailed { .. } => Kind::ServerFailed,
            Self::AnswerNotUnderstood { .. } => Kind::AnswerNotUnderstood,
            Self::CapabilityAbsent { .. } => Kind::CapabilityAbsent,
            Self::Cancelled { .. } => Kind::Cancelled,
            Self::StorageUnavailable { .. } => Kind::StorageUnavailable,
            Self::InternalFault { .. } => Kind::InternalFault,
        }
    }

    /// What a person typed, mapped onto the vocabulary.
    ///
    /// `src/server/address.rs` says what it found and this says what that is in
    /// 0004's words, which is the arrangement 0037 requires: a module's own error
    /// stops here.
    #[must_use]
    pub fn address_not_usable(found: &AddressNotUsable) -> Self {
        Self::AddressNotUsable {
            typed: found.typed().to_owned(),
            part: found.part(),
            constructed: Constructed(()),
        }
    }

    /// A transport outcome, mapped before any HTTP exists.
    #[must_use]
    pub fn from_transport(outcome: &TransportOutcome<'_>, attempt: &Attempt<'_>) -> Self {
        match *outcome {
            TransportOutcome::NameDidNotResolve
            | TransportOutcome::ConnectionRefused
            | TransportOutcome::NetworkUnreachable
            | TransportOutcome::ConnectionDroppedMidBody => Self::ServerUnreachable {
                address: attempt.address.to_owned(),
                bytes_reached_the_server: attempt.bytes_reached_the_server,
                constructed: Constructed(()),
            },
            TransportOutcome::DeadlineReached { deadline, elapsed }
            | TransportOutcome::AnswerStalledMidBody { deadline, elapsed } => Self::TimedOut {
                deadline,
                elapsed,
                bytes_reached_the_server: attempt.bytes_reached_the_server,
                constructed: Constructed(()),
            },
            TransportOutcome::PeerNotTrusted {
                reason,
                fingerprint,
            } => Self::CertificateRejected {
                address: attempt.address.to_owned(),
                reason,
                fingerprint: fingerprint.to_owned(),
                constructed: Constructed(()),
            },
        }
    }

    /// A status, read through 0004's table.
    ///
    /// The body may add payload and may never change the kind, so nothing in
    /// [`Answered`] except the identifier decides anything, and that one is the
    /// 404 split 0004 makes on the core's own list rather than on the response.
    ///
    /// A status inside the successful range produces `internal-fault` rather than
    /// a kind. 0004's row for a 2xx is about a body that does not parse, which
    /// arrives through [`Failure::answer_not_understood`] with the site it stopped
    /// at, so a 2xx here is the core asking which failure a success is.
    #[must_use]
    pub fn from_status(status: u16, answered: &Answered<'_>) -> Self {
        match status {
            401 => Self::NotAuthenticated {
                // A 401 answered a request that carried a token unless the caller
                // had none to send, and which of the two it was is not in the
                // status. THIS COMMENT SAID NO CALLER KNEW YET AND THAT THE FLAG
                // WAS THE HONEST HALF UNTIL ONE EXISTED. One does:
                // [`Failure::from_status_with_no_token_presented`] is the door
                // 0030 opens, where the caller had nothing to present, and this
                // arm is what every other caller reaches. So the default is a
                // token presented and rejected because that is what a request
                // made in a session carried, rather than because the difference
                // is unavailable, and the flag carries it to #34 and #35 either
                // way.
                a_token_was_presented: true,
                constructed: Constructed(()),
            },
            403 => Self::NotPermitted {
                constructed: Constructed(()),
            },
            404 => match answered.identifier {
                Some(identifier) => Self::NotFound {
                    identifier: identifier.to_owned(),
                    constructed: Constructed(()),
                },
                None => Self::CapabilityAbsent {
                    capability: answered.capability,
                    constructed: Constructed(()),
                },
            },
            405 | 410 | 501 => Self::CapabilityAbsent {
                capability: answered.capability,
                constructed: Constructed(()),
            },
            429 | 503 => Self::ServerBusy {
                retry_after: answered.retry_after,
                constructed: Constructed(()),
            },
            200..=299 => Self::InternalFault {
                site: FaultSite::SuccessMappedAsFailure,
                constructed: Constructed(()),
            },
            400..=499 => Self::RequestRefused {
                server_code: answered.server_code.map(ToOwned::to_owned),
                constructed: Constructed(()),
            },
            500..=599 => Self::ServerFailed {
                status,
                server_code: answered.server_code.map(ToOwned::to_owned),
                constructed: Constructed(()),
            },
            // 1xx and 3xx surfacing to a caller, and any number outside the range
            // a status line can carry. Both are 0004's fourth rule, and the
            // offset is the status itself rather than a position in a body,
            // because the status line is where reading stopped.
            _ => Self::AnswerNotUnderstood {
                site: ReadingSite::StatusLine,
                expected: Expected::AStatusTheTableNames,
                stopped_at: 0,
                constructed: Constructed(()),
            },
        }
    }

    /// A status, read through 0004's table at a door where the request carried
    /// no token.
    ///
    /// 0030's route is the one that reaches this. A sign-in presents a name and
    /// a password and has nothing else to present, so a refused credential is
    /// `not-authenticated` with the payload saying there was no token - which is
    /// the opposite payload to the rejection 0034 acts on, on the same kind. One
    /// kind and two payloads is 0004's decision rather than this constructor's,
    /// and the difference is what #34 and #35 branch on.
    ///
    /// Everything else is [`Failure::from_status`] unchanged, so this adds no row
    /// to the table and no kind to the vocabulary. It is a second entrance to the
    /// one mapping point 0037 fixes rather than a second mapping: a caller that
    /// knows something the status does not carry says so here, and nothing else
    /// about the reading moves.
    #[must_use]
    pub fn from_status_with_no_token_presented(status: u16, answered: &Answered<'_>) -> Self {
        match Self::from_status(status, answered) {
            Self::NotAuthenticated { .. } => Self::NotAuthenticated {
                a_token_was_presented: false,
                constructed: Constructed(()),
            },
            otherwise => otherwise,
        }
    }

    /// The fourth rule: a shape none of the three sources produced a kind for.
    #[must_use]
    pub const fn answer_not_understood(
        site: ReadingSite,
        expected: Expected,
        stopped_at: usize,
    ) -> Self {
        Self::AnswerNotUnderstood {
            site,
            expected,
            stopped_at,
            constructed: Constructed(()),
        }
    }

    /// A capability this server does not offer, where the core learned it from
    /// something other than a status.
    #[must_use]
    pub const fn capability_absent(capability: Capability) -> Self {
        Self::CapabilityAbsent {
            capability,
            constructed: Constructed(()),
        }
    }

    /// A store the client supplied answering badly.
    #[must_use]
    pub const fn storage_unavailable(store: Store, operation: Operation) -> Self {
        Self::StorageUnavailable {
            store,
            operation,
            constructed: Constructed(()),
        }
    }

    /// The cache store's own refusal, mapped.
    #[must_use]
    pub const fn from_cache_store(_: StorageUnavailable, operation: Operation) -> Self {
        Self::storage_unavailable(Store::Cache, operation)
    }

    /// The secret store's own refusal, mapped.
    #[must_use]
    pub const fn from_secret_store(_: SecretStoreUnavailable, operation: Operation) -> Self {
        Self::storage_unavailable(Store::Secret, operation)
    }

    /// The caller asked for this to stop.
    ///
    /// 0009 separates a cancelled call from every failure and 0061 gives a span a
    /// third outcome for the same reason, so this is not a failure being
    /// classified. The value is still built here, because a caller holding an
    /// outcome should not be able to tell from its shape which part of the core
    /// built it.
    #[must_use]
    pub const fn cancelled() -> Self {
        Self::Cancelled {
            constructed: Constructed(()),
        }
    }

    /// A defect in the core.
    #[must_use]
    pub const fn internal_fault(site: FaultSite) -> Self {
        Self::InternalFault {
            site,
            constructed: Constructed(()),
        }
    }
}

#[cfg(test)]
mod tests {
    //! What the mapping point answers, case by case.
    //!
    //! Nothing here reads a network. Every input is a value, which is what makes
    //! 0004's table testable before the transport in #27 exists, and the suite
    //! that drives the fake server through hostile answers is in
    //! `tests/every_hostile_answer_becomes_a_named_kind.rs`.

    use super::{
        Answered, Attempt, Capability, CertificateReason, Deadline, Expected, Failure, FaultSite,
        Kind, Operation, ReadingSite, Store, TransportOutcome,
    };
    use crate::cache::StorageUnavailable;
    use crate::server::address::{BaseAddress, UnusablePart};
    use crate::session::SecretStoreUnavailable;
    use core::time::Duration;

    /// An answer carrying nothing a caller reads, on a path naming no identifier.
    fn a_route_call() -> Answered<'static> {
        Answered {
            capability: Capability::LibraryQuery,
            identifier: None,
            retry_after: None,
            server_code: None,
        }
    }

    /// The same on a path the caller supplied an identifier in.
    fn an_item_call() -> Answered<'static> {
        Answered {
            capability: Capability::ItemDetail,
            identifier: Some("an-item"),
            retry_after: None,
            server_code: None,
        }
    }

    fn an_attempt() -> Attempt<'static> {
        Attempt {
            address: "https://films.example",
            bytes_reached_the_server: false,
        }
    }

    #[test]
    fn the_fifteen_kinds_carry_the_names_0004_writes() {
        let names = [
            Kind::AddressNotUsable,
            Kind::ServerUnreachable,
            Kind::TimedOut,
            Kind::CertificateRejected,
            Kind::NotAuthenticated,
            Kind::NotPermitted,
            Kind::NotFound,
            Kind::RequestRefused,
            Kind::ServerBusy,
            Kind::ServerFailed,
            Kind::AnswerNotUnderstood,
            Kind::CapabilityAbsent,
            Kind::Cancelled,
            Kind::StorageUnavailable,
            Kind::InternalFault,
        ];
        assert_eq!(names.len(), 15, "0004 fixes fifteen and the set is closed");
        let mut written: Vec<&str> = names.iter().map(|k| k.declared_name()).collect();
        written.sort_unstable();
        written.dedup();
        assert_eq!(
            written.len(),
            15,
            "two kinds are written the same way, so a count grouped by name \
             cannot tell them apart"
        );
        assert_eq!(
            Kind::AnswerNotUnderstood.declared_name(),
            "answer-not-understood"
        );
    }

    #[test]
    fn the_capability_set_is_the_sixteen_0010_fixes() {
        let all = [
            Capability::ServerIdentity,
            Capability::PasswordSignIn,
            Capability::QuickConnect,
            Capability::DelegatedSignIn,
            Capability::TokenRenewal,
            Capability::SignOut,
            Capability::DeviceCapabilities,
            Capability::LibraryQuery,
            Capability::ItemDetail,
            Capability::ResumeList,
            Capability::ItemUserData,
            Capability::Artwork,
            Capability::PlaybackSelection,
            Capability::PlaybackProgress,
            Capability::PlayedMarking,
            Capability::ChangeNotification,
        ];
        assert_eq!(all.len(), 16);
        let mut written: Vec<&str> = all.iter().map(|c| c.declared_name()).collect();
        written.sort_unstable();
        written.dedup();
        assert_eq!(written.len(), 16);
        assert_eq!(
            Capability::ChangeNotification.declared_name(),
            "change-notification"
        );
    }

    #[test]
    fn every_reading_site_and_fault_site_is_written_once() {
        let sites = [
            ReadingSite::StatusLine,
            ReadingSite::AnswerBody,
            ReadingSite::CrossOriginRedirectRefused,
            ReadingSite::ImageFormatRefused,
        ];
        let mut written: Vec<&str> = sites.iter().map(|s| s.declared_name()).collect();
        written.sort_unstable();
        written.dedup();
        assert_eq!(
            written.len(),
            4,
            "two sites are written the same way, and the grouping three records \
             rest on is exactly this name"
        );
        assert_eq!(
            FaultSite::SuccessMappedAsFailure.declared_name(),
            "success-mapped-as-failure"
        );
    }

    #[test]
    fn an_address_that_cannot_be_used_carries_what_was_typed_unmodified() {
        let typed = "  ftp://films.example  ";
        let found = BaseAddress::parse(typed).expect_err("a scheme 0028 refuses");
        let mapped = Failure::address_not_usable(&found);
        assert_eq!(mapped.kind(), Kind::AddressNotUsable);
        let Failure::AddressNotUsable {
            typed: kept, part, ..
        } = mapped
        else {
            panic!("a scheme refusal mapped onto something else");
        };
        assert_eq!(kept, typed, "the address reached a caller changed");
        assert_eq!(part, UnusablePart::Scheme);
    }

    #[test]
    fn the_four_transport_outcomes_that_never_answered_are_one_kind() {
        for outcome in [
            TransportOutcome::NameDidNotResolve,
            TransportOutcome::ConnectionRefused,
            TransportOutcome::NetworkUnreachable,
            TransportOutcome::ConnectionDroppedMidBody,
        ] {
            let mapped = Failure::from_transport(&outcome, &an_attempt());
            assert_eq!(mapped.kind(), Kind::ServerUnreachable, "{outcome:?}");
        }
    }

    #[test]
    fn whether_bytes_reached_the_server_survives_the_mapping() {
        let attempt = Attempt {
            address: "https://films.example",
            bytes_reached_the_server: true,
        };
        let mapped = Failure::from_transport(&TransportOutcome::ConnectionDroppedMidBody, &attempt);
        let Failure::ServerUnreachable {
            address,
            bytes_reached_the_server,
            ..
        } = mapped
        else {
            panic!("a dropped connection mapped onto something else");
        };
        assert_eq!(address, "https://films.example");
        assert!(
            bytes_reached_the_server,
            "a call that may have happened arrived as one that certainly did not, \
             which is the difference #47 decides a replay on"
        );
    }

    #[test]
    fn a_deadline_and_a_stall_are_both_timed_out_and_keep_which_one_expired() {
        for outcome in [
            TransportOutcome::DeadlineReached {
                deadline: Deadline::Connect,
                elapsed: Duration::from_secs(2),
            },
            TransportOutcome::AnswerStalledMidBody {
                deadline: Deadline::WholeRequest,
                elapsed: Duration::from_secs(5),
            },
        ] {
            let mapped = Failure::from_transport(&outcome, &an_attempt());
            assert_eq!(mapped.kind(), Kind::TimedOut);
        }
        let mapped = Failure::from_transport(
            &TransportOutcome::DeadlineReached {
                deadline: Deadline::FirstByte,
                elapsed: Duration::from_millis(2_000),
            },
            &an_attempt(),
        );
        let Failure::TimedOut {
            deadline, elapsed, ..
        } = mapped
        else {
            panic!("a deadline mapped onto something else");
        };
        assert_eq!(deadline, Deadline::FirstByte);
        assert_eq!(elapsed, Duration::from_secs(2));
    }

    #[test]
    fn an_untrusted_peer_carries_the_class_and_the_fingerprint() {
        let mapped = Failure::from_transport(
            &TransportOutcome::PeerNotTrusted {
                reason: CertificateReason::SelfSigned,
                fingerprint: "ab:cd",
            },
            &an_attempt(),
        );
        let Failure::CertificateRejected {
            address,
            reason,
            fingerprint,
            ..
        } = mapped
        else {
            panic!("an untrusted peer mapped onto something else");
        };
        assert_eq!(address, "https://films.example");
        assert_eq!(reason, CertificateReason::SelfSigned);
        assert_eq!(fingerprint, "ab:cd");
    }

    #[test]
    fn the_status_table_answers_what_0004_says_it_answers() {
        let route = a_route_call();
        for (status, kind) in [
            (401_u16, Kind::NotAuthenticated),
            (403, Kind::NotPermitted),
            (404, Kind::CapabilityAbsent),
            (405, Kind::CapabilityAbsent),
            (410, Kind::CapabilityAbsent),
            (501, Kind::CapabilityAbsent),
            (429, Kind::ServerBusy),
            (503, Kind::ServerBusy),
            (400, Kind::RequestRefused),
            (418, Kind::RequestRefused),
            (500, Kind::ServerFailed),
            (502, Kind::ServerFailed),
            (100, Kind::AnswerNotUnderstood),
            (302, Kind::AnswerNotUnderstood),
            (600, Kind::AnswerNotUnderstood),
        ] {
            assert_eq!(
                Failure::from_status(status, &route).kind(),
                kind,
                "status {status}"
            );
        }
    }

    #[test]
    fn the_two_404_rows_are_decided_by_the_path_and_never_by_the_body() {
        let on_a_route = Failure::from_status(404, &a_route_call());
        assert_eq!(on_a_route.kind(), Kind::CapabilityAbsent);
        let Failure::CapabilityAbsent { capability, .. } = on_a_route else {
            panic!("a 404 on a route mapped onto something else");
        };
        assert_eq!(capability, Capability::LibraryQuery);

        let on_an_item = Failure::from_status(404, &an_item_call());
        assert_eq!(on_an_item.kind(), Kind::NotFound);
        let Failure::NotFound { identifier, .. } = on_an_item else {
            panic!("a 404 on an item mapped onto something else");
        };
        assert_eq!(
            identifier, "an-item",
            "an absent film arrived as an absent capability, which tells an \
             operator to upgrade their server over a deleted item"
        );
    }

    #[test]
    fn the_door_with_no_token_takes_the_other_payload_and_moves_no_other_row() {
        // 0030's route. The near miss is a door that reported a refused
        // credential as a token presented and rejected, which 0034 reads as a
        // session that has ended and answers with a renewal for a session that
        // was never established.
        let refused = Failure::from_status_with_no_token_presented(401, &a_route_call());
        assert_eq!(refused.kind(), Kind::NotAuthenticated);
        let Failure::NotAuthenticated {
            a_token_was_presented,
            ..
        } = refused
        else {
            panic!("a 401 at the password door mapped onto something else");
        };
        assert!(
            !a_token_was_presented,
            "a sign-in has no token to present, and saying it had one is the              rejection 0034 acts on arriving from a session that does not exist"
        );

        let in_a_session = Failure::from_status(401, &a_route_call());
        let Failure::NotAuthenticated {
            a_token_was_presented,
            ..
        } = in_a_session
        else {
            panic!("a 401 in a session mapped onto something else");
        };
        assert!(a_token_was_presented);

        // Every other row is the same reading through the same table.
        for status in [200_u16, 403, 404, 405, 410, 418, 429, 500, 503, 600] {
            for answered in [a_route_call(), an_item_call()] {
                assert_eq!(
                    Failure::from_status_with_no_token_presented(status, &answered),
                    Failure::from_status(status, &answered),
                    "the door moved a row that is not the 401"
                );
            }
        }
    }

    #[test]
    fn a_server_supplied_body_adds_payload_and_never_changes_the_kind() {
        let with_a_code = Answered {
            server_code: Some("SomethingTheServerCalledIt"),
            ..a_route_call()
        };
        let refused = Failure::from_status(400, &with_a_code);
        assert_eq!(refused.kind(), Kind::RequestRefused);
        let Failure::RequestRefused { server_code, .. } = refused else {
            panic!("a 400 mapped onto something else");
        };
        assert_eq!(server_code.as_deref(), Some("SomethingTheServerCalledIt"));

        // The same body under a status the table maps elsewhere. A proxy putting a
        // themed page in front of a 503 must not turn a retryable condition into
        // an unrecognised one.
        let busy = Failure::from_status(503, &with_a_code);
        assert_eq!(busy.kind(), Kind::ServerBusy);
    }

    #[test]
    fn a_retry_after_hint_is_carried_where_the_server_gave_one_and_absent_where_it_did_not() {
        let with_a_hint = Answered {
            retry_after: Some(Duration::from_secs(30)),
            ..a_route_call()
        };
        let Failure::ServerBusy { retry_after, .. } = Failure::from_status(429, &with_a_hint)
        else {
            panic!("a 429 mapped onto something else");
        };
        assert_eq!(retry_after, Some(Duration::from_secs(30)));

        let Failure::ServerBusy { retry_after, .. } = Failure::from_status(429, &a_route_call())
        else {
            panic!("a 429 mapped onto something else");
        };
        assert_eq!(
            retry_after, None,
            "a duration nothing chose reached a caller as though a server had \
             given it"
        );
    }

    #[test]
    fn a_server_failure_carries_its_status_opaque() {
        let Failure::ServerFailed { status, .. } = Failure::from_status(507, &a_route_call())
        else {
            panic!("a 507 mapped onto something else");
        };
        assert_eq!(status, 507);
    }

    #[test]
    fn a_success_handed_to_the_status_mapping_is_a_defect_in_the_core() {
        for status in [200_u16, 204, 299] {
            let mapped = Failure::from_status(status, &a_route_call());
            assert_eq!(mapped.kind(), Kind::InternalFault, "status {status}");
            let Failure::InternalFault { site, .. } = mapped else {
                panic!("a success mapped onto something else");
            };
            assert_eq!(site, FaultSite::SuccessMappedAsFailure);
        }
    }

    #[test]
    fn a_status_outside_every_row_stops_at_the_status_line() {
        let Failure::AnswerNotUnderstood {
            site,
            expected,
            stopped_at,
            ..
        } = Failure::from_status(600, &a_route_call())
        else {
            panic!("a status outside the range mapped onto something else");
        };
        assert_eq!(site, ReadingSite::StatusLine);
        assert_eq!(expected, Expected::AStatusTheTableNames);
        assert_eq!(stopped_at, 0);
    }

    #[test]
    fn the_two_refusals_pushed_under_the_catch_all_are_countable_on_their_own() {
        // 0069 and 0055 each take `answer-not-understood` knowing the fit is
        // wrong, and each names the same measurement as what would overturn it.
        // This is the field that makes that measurement three populations rather
        // than one count.
        let redirect = Failure::answer_not_understood(
            ReadingSite::CrossOriginRedirectRefused,
            Expected::AnOriginTheCoreMayContact,
            0,
        );
        let image = Failure::answer_not_understood(
            ReadingSite::ImageFormatRefused,
            Expected::AnAcceptedImageFormat,
            0,
        );
        let unread = Failure::answer_not_understood(
            ReadingSite::AnswerBody,
            Expected::AFieldTheCoreNeeds,
            91,
        );
        assert_eq!(redirect.kind(), image.kind());
        assert_eq!(image.kind(), unread.kind());
        let sites: Vec<Kind> = vec![redirect.kind(), image.kind(), unread.kind()];
        assert_eq!(sites.len(), 3);
        let Failure::AnswerNotUnderstood { site, .. } = redirect else {
            panic!("the catch-all mapped onto something else");
        };
        assert_eq!(site, ReadingSite::CrossOriginRedirectRefused);
        let Failure::AnswerNotUnderstood { stopped_at, .. } = unread else {
            panic!("the catch-all mapped onto something else");
        };
        assert_eq!(stopped_at, 91);
    }

    #[test]
    fn both_stores_reach_the_same_kind_and_say_which_they_were() {
        let cache = Failure::from_cache_store(StorageUnavailable, Operation::Write);
        assert_eq!(cache.kind(), Kind::StorageUnavailable);
        let Failure::StorageUnavailable {
            store, operation, ..
        } = cache
        else {
            panic!("a cache refusal mapped onto something else");
        };
        assert_eq!(store, Store::Cache);
        assert_eq!(operation, Operation::Write);

        let secret = Failure::from_secret_store(SecretStoreUnavailable, Operation::Read);
        let Failure::StorageUnavailable {
            store, operation, ..
        } = secret
        else {
            panic!("a secret refusal mapped onto something else");
        };
        assert_eq!(store, Store::Secret);
        assert_eq!(operation, Operation::Read);
    }

    #[test]
    fn cancellation_and_a_capability_the_core_learned_elsewhere_are_built_here_too() {
        assert_eq!(Failure::cancelled().kind(), Kind::Cancelled);
        let absent = Failure::capability_absent(Capability::TokenRenewal);
        assert_eq!(absent.kind(), Kind::CapabilityAbsent);
        let Failure::CapabilityAbsent { capability, .. } = absent else {
            panic!("an absent capability mapped onto something else");
        };
        assert_eq!(capability, Capability::TokenRenewal);
        assert_eq!(
            Failure::internal_fault(FaultSite::SuccessMappedAsFailure).kind(),
            Kind::InternalFault
        );
    }

    #[test]
    fn a_401_says_a_token_was_presented_so_that_35_can_tell_the_two_apart() {
        let Failure::NotAuthenticated {
            a_token_was_presented,
            ..
        } = Failure::from_status(401, &a_route_call())
        else {
            panic!("a 401 mapped onto something else");
        };
        assert!(a_token_was_presented);
    }

    #[test]
    fn a_403_and_a_cancellation_carry_nothing_a_caller_can_read() {
        // The near miss beside the payload cases: two kinds 0004 says carry
        // nothing, and a variant that grew a field would be a payload nobody
        // decided.
        assert_eq!(
            Failure::from_status(403, &a_route_call()),
            Failure::from_status(403, &an_item_call()),
            "a 403 carried something that differs between two calls"
        );
        assert_eq!(Failure::cancelled(), Failure::cancelled());
    }
}
