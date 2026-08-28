//! The sixteen capabilities of 0010, as a table the fake server serves.
//!
//! The rows here are the rows of the table in
//! `docs/decisions/0010-the-server-surface-and-what-an-absence-does.md`, in that
//! record's own order, with a healthy answer and a hostile one beside each. That
//! record is the authority; this file is a transcription of it into something a
//! socket can answer, and a row here that the record does not carry is a row
//! nobody decided.
//!
//! # No answer here is a recording, and that is the part to read
//!
//! #21's body asks for recorded responses from real servers. There is no real
//! server on the machine this was written on, and
//! `tests/needs_a_real_server_or_real_hardware.rs` is the harness that exists for
//! exactly that boundary and carries no case. So every answer below is written by
//! hand from the record and NOT recorded from a server, and it proves the framing
//! rather than the shape: a status, a content type, a declared length, and bytes
//! that either arrive whole or do not.
//!
//! What follows from that, said once so a later reading does not take it for
//! more: no field name in any body below was read out of a server. Where a body
//! carries one it is quoted from a record in this tree and nowhere else, and
//! everything else is an empty object on purpose, because a field name invented
//! here would be a claim about an interface nobody read. #104 is where a fixture
//! is held honest against a real server, and it is open.
//!
//! # The two capabilities with no path
//!
//! `delegated-sign-in` and `token-renewal` have no route on either supported
//! line. 0010 names them anyway, because a capability with no name cannot be
//! reported. They are in `CAPABILITIES_WITH_NO_ROUTE` rather than in the route
//! table, and what the fake does for them is what it does for any path it does
//! not carry: it answers 404, which is 0010's fallback rule arriving from the
//! server rather than from a branch in the core.

/// The method a route is reached with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// A read.
    Get,
    /// A write, and the method every accumulating row in 0010's table uses.
    Post,
    /// The one removal in the table, which is 0060's mark cleared.
    Delete,
    /// The header of an answer without its body, which 0010 says the artwork
    /// path answers the same way as a `GET`.
    Head,
}

impl Method {
    /// The token this method is written as on the wire.
    #[must_use]
    pub const fn on_the_wire(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
        }
    }

    /// The method a request line names, or `None` for a token no row uses.
    #[must_use]
    pub fn from_the_wire(token: &str) -> Option<Self> {
        match token {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "DELETE" => Some(Self::Delete),
            "HEAD" => Some(Self::Head),
            _ => None,
        }
    }
}

/// One answer, whole.
#[derive(Debug, Clone, Copy)]
pub struct Answer {
    /// The status line's number.
    pub status: u16,
    /// The status line's reason phrase. It is carried because a client reading a
    /// reason phrase instead of a status is a defect a fake with an empty one
    /// cannot expose.
    pub reason: &'static str,
    /// What the answer declares its bytes to be.
    pub content_type: &'static str,
    /// The bytes of the body.
    pub body: &'static [u8],
}

/// How a row misbehaves when a test asks it to.
///
/// The five shapes #21's body names are all here, and every one of them is on at
/// least one row of the table below, which the test target asserts rather than
/// leaving to a reader to count.
#[derive(Debug, Clone, Copy)]
pub enum Hostile {
    /// A status the caller has to map onto 0004's vocabulary, with a body that
    /// may add payload and may never change the kind.
    Answers(Answer),
    /// The answer is withheld past any deadline a caller could be holding. The
    /// header is not written either, so what the caller meets is silence on an
    /// open connection rather than a slow body.
    ///
    /// It carries no answer, and that is the shape rather than an omission:
    /// nothing is ever written, so an answer here would be a fixture no case
    /// could read and the first person to add one would be describing bytes that
    /// never leave.
    Withheld,
    /// The header declares a longer body than the connection carries, and the
    /// connection ends where the bytes stop.
    TruncatedBody(Answer),
    /// The bytes are the fixture's and the declared type is not what they are.
    /// 0055 and 0101 decide that the bytes win; this is what lets a test show it.
    WrongContentType(Answer),
    /// The body begins, the connection ends part way through it, and every later
    /// request on this row is answered 401. That is the token dying during
    /// playback in #35, expressed in what a socket can actually do.
    UnauthorizedMidStream(Answer),
    /// The route is not there at all. On a path carrying no caller-supplied
    /// identifier that is `capability-absent` by 0010's fallback rule, and on one
    /// that carries an identifier it is `not-found`.
    Absent,
}

/// Where a row is reached.
#[derive(Debug, Clone, Copy)]
pub enum Reached {
    /// A method and a path, joined to the base address by 0028's rule. A segment
    /// written between braces is a caller-supplied identifier and matches any one
    /// segment.
    Path {
        /// The method the row answers, and only that one.
        method: Method,
        /// The path, with a caller-supplied segment written between braces.
        template: &'static str,
    },
    /// An upgrade on the resolved origin. 0010 names the upgrade and gives no
    /// path for it, so this row is matched by the request's own upgrade header
    /// rather than by a path that record does not carry.
    Upgrade,
}

/// One row of 0010's table.
#[derive(Debug, Clone, Copy)]
pub struct Row {
    /// The capability name, which is what `capability-absent` carries in 0004.
    pub capability: &'static str,
    /// How the row is reached.
    pub reached: Reached,
    /// What a healthy server answers.
    pub healthy: Answer,
    /// The one hostile shape this row's caller has to survive.
    pub hostile: Hostile,
}

/// An empty JSON object, which is what a body is here wherever no record in this
/// tree names a field.
const NOTHING: &[u8] = b"{}";

/// The one body carrying a field name, and both halves of it are quoted from
/// 0010 rather than from a server: that record names the `Version` field of this
/// answer by name, and names `10.11.11` as the version the older of the two
/// supported lines reported at the commit it was read at.
const SERVER_IDENTITY: &[u8] = b"{\"Version\":\"10.11.11\"}";

/// A baseline JPEG header declaring four by three, built the way
/// `src/artwork/format.rs` builds one in its own cases: the signature, an
/// application segment for the walk to step over, then the frame header with
/// height before width.
pub const A_SMALL_JPEG: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x03, 0x00,
    0x04,
];

/// Bytes that are a picture and are not one of the three 0055 accepts, served
/// under a type that says they are. The signature match in
/// `src/artwork/format.rs` matches none of the three on these, which is the
/// refusal the artwork row exists to reach.
pub const NOT_AN_ACCEPTED_IMAGE: &[u8] = b"GIF89a\x01\x00\x01\x00\x00\x00\x00";

/// A healthy JSON answer.
const fn json(body: &'static [u8]) -> Answer {
    Answer {
        status: 200,
        reason: "OK",
        content_type: "application/json; charset=utf-8",
        body,
    }
}

/// An answer carrying a status and nothing a caller reads out of the body.
const fn status(status: u16, reason: &'static str) -> Answer {
    Answer {
        status,
        reason,
        content_type: "application/json; charset=utf-8",
        body: NOTHING,
    }
}

/// The key a test offers on the upgrade.
///
/// The accept value below is computed rather than remembered, and the command
/// that produced it is in the pull request that added this file. It is a fixture
/// in the strict sense: an exact sequence of bytes a client checking the
/// handshake compares against, so a fake answering an arbitrary string here would
/// let a client that never checks look correct.
pub const UPGRADE_KEY: &str = "x3JJHMbDL1EzLkh9GBhXDw==";

/// The accept value the protocol requires for `UPGRADE_KEY`.
pub const UPGRADE_ACCEPT: &str = "HSmrc0sMlYUkAGmm5OPpG2HaGWk=";

/// The two capabilities 0010 names that no supported line offers.
pub const CAPABILITIES_WITH_NO_ROUTE: &[&str] = &["delegated-sign-in", "token-renewal"];

/// The table, in 0010's order.
///
/// The hostile shape on each row is chosen for what that row's caller has to
/// survive rather than picked to spread the variants evenly, and the reason is on
/// the row.
pub const SURFACE: &[Row] = &[
    // The identifier and version the server reports about itself. Unauthenticated
    // on both lines, read once per server, and 0041's server part comes out of it.
    // Hostile: the answer arrives with a type that is not JSON, which is the shape
    // a reader that trusts the declaration rather than the bytes walks into.
    Row {
        capability: "server-identity",
        reached: Reached::Path {
            method: Method::Get,
            template: "/System/Info/Public",
        },
        healthy: json(SERVER_IDENTITY),
        hostile: Hostile::WrongContentType(Answer {
            status: 200,
            reason: "OK",
            content_type: "text/html; charset=utf-8",
            body: SERVER_IDENTITY,
        }),
    },
    // 0030's route. Hostile: the credentials are refused, which is the answer this
    // route gives most often and the one 0004 maps to `not-authenticated`.
    Row {
        capability: "password-sign-in",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Users/AuthenticateByName",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(401, "Unauthorized")),
    },
    // Whether the operator has quick connect on. Hostile: the route is gone, which
    // on a path carrying no caller-supplied identifier is `capability-absent`.
    Row {
        capability: "quick-connect",
        reached: Reached::Path {
            method: Method::Get,
            template: "/QuickConnect/Enabled",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Absent,
    },
    // Begin an exchange. Hostile: the operator turned the route off between the
    // read above and this call, which 0010 records as answering 401 and states the
    // cost of rather than repairing.
    Row {
        capability: "quick-connect",
        reached: Reached::Path {
            method: Method::Post,
            template: "/QuickConnect/Initiate",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(401, "Unauthorized")),
    },
    // 0031's poll. Hostile: a 404 for a secret the server does not hold, which
    // 0010 places on the `not-found` side because the caller supplied the secret.
    Row {
        capability: "quick-connect",
        reached: Reached::Path {
            method: Method::Get,
            template: "/QuickConnect/Connect",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(404, "Not Found")),
    },
    // Exchange an approved secret for a session. Hostile: the answer never comes,
    // which is what a caller holding a person in front of a code meets when the
    // server stops answering mid-exchange.
    Row {
        capability: "quick-connect",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Users/AuthenticateWithQuickConnect",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Withheld,
    },
    // 0114's ending of one session. Hostile: the token is already dead, so signing
    // out is refused, which is the case a client must not treat as a reason to
    // keep the session.
    Row {
        capability: "sign-out",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Sessions/Logout",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(401, "Unauthorized")),
    },
    // The description 0036 says a client supplies. Hostile: the route is not on
    // this server, which is `capability-absent` and must not stop a sign-in.
    Row {
        capability: "device-capabilities",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Sessions/Capabilities/Full",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Absent,
    },
    // The top of the library, which is a first screen's first request. Hostile: the
    // body is cut off part way, which is the shape #46's cold start meets on a
    // connection that dies while the first screen is being filled.
    Row {
        capability: "library-query",
        reached: Reached::Path {
            method: Method::Get,
            template: "/UserViews",
        },
        healthy: json(NOTHING),
        hostile: Hostile::TruncatedBody(json(NOTHING)),
    },
    // The query surface #39 builds on. Hostile: the server is answering and slowly,
    // which is 0007's slow server and the case #44 reports progressively against.
    Row {
        capability: "library-query",
        reached: Reached::Path {
            method: Method::Get,
            template: "/Items",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Withheld,
    },
    // One item, in full. Hostile: a 404 on a path carrying an identifier, which
    // 0010 splits to `not-found` rather than to `capability-absent`, and getting
    // that split wrong tells an operator to upgrade their server over a deleted
    // film.
    Row {
        capability: "item-detail",
        reached: Reached::Path {
            method: Method::Get,
            template: "/Items/{itemId}",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(404, "Not Found")),
    },
    // What 0058 resumes into. Hostile: the server is having a bad day, which 0038
    // retries and 0004 maps to its own kind.
    Row {
        capability: "resume-list",
        reached: Reached::Path {
            method: Method::Get,
            template: "/UserItems/Resume",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(503, "Service Unavailable")),
    },
    // The position and the watched mark, read. Hostile: a 404 on an identifier the
    // caller supplied, which is the item being gone rather than the route.
    Row {
        capability: "item-user-data",
        reached: Reached::Path {
            method: Method::Get,
            template: "/UserItems/{itemId}/UserData",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(404, "Not Found")),
    },
    // The same, written. Hostile: the token dies part way through the write, which
    // is #35 and the one shape a queue in #47 has to be able to tell from a
    // refusal.
    Row {
        capability: "item-user-data",
        reached: Reached::Path {
            method: Method::Post,
            template: "/UserItems/{itemId}/UserData",
        },
        healthy: json(NOTHING),
        hostile: Hostile::UnauthorizedMidStream(json(NOTHING)),
    },
    // The bytes #49 builds an address for. Hostile: a picture that is not one of
    // the three 0055 accepts, served under a type that says it is, which is the
    // input `src/artwork/format.rs` refuses before a decoder is reached.
    Row {
        capability: "artwork",
        reached: Reached::Path {
            method: Method::Get,
            template: "/Items/{itemId}/Images/{imageType}",
        },
        healthy: Answer {
            status: 200,
            reason: "OK",
            content_type: "image/jpeg",
            body: A_SMALL_JPEG,
        },
        hostile: Hostile::Answers(Answer {
            status: 200,
            reason: "OK",
            content_type: "image/jpeg",
            body: NOT_AN_ACCEPTED_IMAGE,
        }),
    },
    // 0010 says a HEAD on the same path answers the same, so the row is here
    // rather than folded into the one above: a fake that answered a HEAD by
    // accident would let #52's dimensions-before-bytes pass without that promise
    // being kept. Hostile: the header declares a format the bytes on the `GET` are
    // not, which is the disagreement #52 has to resolve towards the bytes and the
    // one a caller that trusts a header will get wrong before it has any bytes to
    // check it against.
    Row {
        capability: "artwork",
        reached: Reached::Path {
            method: Method::Head,
            template: "/Items/{itemId}/Images/{imageType}",
        },
        healthy: Answer {
            status: 200,
            reason: "OK",
            content_type: "image/jpeg",
            body: A_SMALL_JPEG,
        },
        hostile: Hostile::WrongContentType(Answer {
            status: 200,
            reason: "OK",
            content_type: "image/png",
            body: A_SMALL_JPEG,
        }),
    },
    // 0111's choice of source. An accumulation: a repeat can open a second live
    // stream, which is why #47 reads 0010's own column rather than deciding for
    // itself. Hostile: the item cannot be played, answered as a refusal rather
    // than as an absence.
    Row {
        capability: "playback-selection",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Items/{itemId}/PlaybackInfo",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(400, "Bad Request")),
    },
    // Playback started. Hostile: the token died before the report, which is the
    // first thing #59 has to hold offline rather than lose.
    Row {
        capability: "playback-progress",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Sessions/Playing",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(401, "Unauthorized")),
    },
    // 0057's cadence. Hostile: the answer is withheld, and a cadence that waits
    // for one is a cadence that stops.
    Row {
        capability: "playback-progress",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Sessions/Playing/Progress",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Withheld,
    },
    // Playback ended. Hostile: the server refuses the last report of a session,
    // which is the report that most needs to survive being lost.
    Row {
        capability: "playback-progress",
        reached: Reached::Path {
            method: Method::Post,
            template: "/Sessions/Playing/Stopped",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(500, "Internal Server Error")),
    },
    // 0060's mark, set. Hostile: a 404 on a path carrying an identifier, which is
    // the item being gone and not a statement about the server.
    Row {
        capability: "played-marking",
        reached: Reached::Path {
            method: Method::Post,
            template: "/UserPlayedItems/{itemId}",
        },
        healthy: json(NOTHING),
        hostile: Hostile::Answers(status(404, "Not Found")),
    },
    // 0060's mark, cleared. Hostile: the body arrives cut off, which a caller that
    // reads a declared length rather than the bytes it got will not notice.
    Row {
        capability: "played-marking",
        reached: Reached::Path {
            method: Method::Delete,
            template: "/UserPlayedItems/{itemId}",
        },
        healthy: json(NOTHING),
        hostile: Hostile::TruncatedBody(json(NOTHING)),
    },
    // 0116's connection. Authenticated on both lines and refused without a token,
    // which is why it is a capability of a session rather than of a server, and
    // which is exactly the hostile shape here.
    Row {
        capability: "change-notification",
        reached: Reached::Upgrade,
        healthy: Answer {
            status: 101,
            reason: "Switching Protocols",
            content_type: "",
            body: b"",
        },
        hostile: Hostile::Answers(status(401, "Unauthorized")),
    },
];
