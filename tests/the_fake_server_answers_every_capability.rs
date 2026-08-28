//! What the fake server in #21 answers, and what it refuses to pretend.
//!
//! Every case here runs against `127.0.0.1` on a port the operating system
//! chose, in the process running the test, with no external network. The `test`
//! check runs this suite inside a network namespace carrying only loopback, so a
//! case that reached outside would fail there rather than pass quietly; what this
//! file adds is the assertion that the address is a loopback one, which holds on
//! a contributor machine where that namespace does not exist.
//!
//! # The client here is not the transport
//!
//! The transport is #27 and does not exist. What drives the server below is
//! sixty lines of `std::net` written for this file, which reads a status line,
//! headers and a body and does nothing else. It is deliberately not a general
//! client: the moment #27 lands, the cases here are the ones that should be
//! rewritten to drive the real one, and a capable client written now would be a
//! second transport nobody decided to have.

mod fake_server;

use fake_server::clock::ControlledClocks;
use fake_server::surface::{
    A_SMALL_JPEG, Answer, CAPABILITIES_WITH_NO_ROUTE, Hostile, Method, NOT_AN_ACCEPTED_IMAGE,
    Reached, Row, SURFACE, UPGRADE_ACCEPT, UPGRADE_KEY,
};
use fake_server::{FakeServer, Seen, matching_row_in};
use flowfin_core::artwork::format::{Accepted, Refused, admitted};
use flowfin_core::clock::{Clocks, WallMoment};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// How long a test waits on a connection the server is deliberately not
/// answering before it calls the silence silence.
///
/// This is the test waiting rather than the core, and it is the one interval in
/// this file measured against real time. The core's own deadlines are on the
/// injected source in `fake_server::clock`, where a test moves the clock instead
/// of waiting.
const LONG_ENOUGH_TO_CALL_IT_SILENCE: Duration = Duration::from_millis(50);

/// What the client above read back.
struct Received {
    status: u16,
    reason: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    declared_length: Option<usize>,
}

impl Received {
    fn header(&self, name: &str) -> Option<&str> {
        let wanted = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(had, _)| *had == wanted)
            .map(|(_, value)| value.as_str())
    }
}

/// Sends one request and reads the answer until the connection ends.
fn ask(server: &FakeServer, method: &str, path: &str, extra: &[(&str, &str)]) -> Received {
    read_back(&mut send(server, method, path, extra), None).expect("an answer on loopback")
}

/// Sends one request carrying a body and reads the answer.
fn ask_carrying(
    server: &FakeServer,
    method: &str,
    path: &str,
    extra: &[(&str, &str)],
    body: &[u8],
) -> Received {
    read_back(&mut send_carrying(server, method, path, extra, body), None)
        .expect("an answer on loopback")
}

/// Opens a connection and writes one request onto it.
fn send(server: &FakeServer, method: &str, path: &str, extra: &[(&str, &str)]) -> TcpStream {
    send_carrying(server, method, path, extra, b"")
}

/// Opens a connection and writes one request carrying `body` onto it.
fn send_carrying(
    server: &FakeServer,
    method: &str,
    path: &str,
    extra: &[(&str, &str)],
    body: &[u8],
) -> TcpStream {
    let mut connection =
        TcpStream::connect(server.address()).expect("a connection to the fake server");
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {}\r\n", server.address());
    for (name, value) in extra {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    request.push_str("Content-Length: ");
    request.push_str(&body.len().to_string());
    request.push_str("\r\n\r\n");
    let mut wire = request.into_bytes();
    wire.extend_from_slice(body);
    connection
        .write_all(&wire)
        .expect("the request onto the connection");
    connection.flush().expect("the request flushed");
    connection
}

/// Reads an answer off a connection, or `None` where nothing arrived inside
/// `patience`.
fn read_back(connection: &mut TcpStream, patience: Option<Duration>) -> Option<Received> {
    connection
        .set_read_timeout(patience)
        .expect("a read timeout on the client's own socket");
    let mut received = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match connection.read(&mut chunk) {
            // Nothing more is coming, either because the server closed or
            // because the patience above ran out. Which of the two it was is the
            // caller's to tell, from whether anything arrived at all.
            Ok(0) | Err(_) => break,
            Ok(read) => received.extend_from_slice(&chunk[..read]),
        }
    }
    if received.is_empty() {
        return None;
    }
    let head_ends_at = received
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("an answer with a head");
    let head = String::from_utf8(received[..head_ends_at].to_vec()).expect("a head that is text");
    let mut lines = head.split("\r\n");
    let mut status_line = lines.next().expect("a status line").split(' ');
    let _version = status_line.next();
    let status: u16 = status_line
        .next()
        .expect("a status")
        .parse()
        .expect("a status that is a number");
    let reason = status_line.collect::<Vec<_>>().join(" ");
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    let declared_length = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse::<usize>().ok());
    Some(Received {
        status,
        reason,
        headers,
        body: received[head_ends_at + 4..].to_vec(),
        declared_length,
    })
}

/// The answer both rows of the colliding fixture below carry. What that case is
/// about is which row answers, so the answers are the same on purpose.
const A_ROW_THAT_ANSWERS: Answer = Answer {
    status: 200,
    reason: "OK",
    content_type: "application/json; charset=utf-8",
    body: b"{}",
};

/// A request as the server records one, built without a socket, so that the
/// matcher can be judged against a table this repository does not serve.
fn a_request_for(method: &str, target: &str) -> Seen {
    Seen {
        method: method.to_owned(),
        target: target.to_owned(),
        headers: Vec::new(),
        body: Vec::new(),
    }
}

/// A path a row is reached at, with a value put in place of every
/// caller-supplied segment.
fn a_path_for(row: &Row) -> Option<(Method, String)> {
    let Reached::Path { method, template } = row.reached else {
        return None;
    };
    let path = template
        .split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                "an-identifier-the-caller-supplied"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    Some((method, path))
}

#[test]
fn the_server_listens_on_loopback_and_on_nothing_else() {
    let server = FakeServer::start();
    assert!(
        server.address().ip().is_loopback(),
        "the fake server bound {}, which is not a loopback address. A bind to the \
         machine's own interface address raises a firewall consent dialog on \
         Windows and belongs in the separate harness, which CONTRIBUTING.md \
         records.",
        server.address()
    );
    assert!(server.base_address().starts_with("http://127.0.0.1:"));
}

#[test]
fn every_row_of_the_surface_answers_its_healthy_fixture() {
    let server = FakeServer::start();
    for row in SURFACE {
        let Some((method, path)) = a_path_for(row) else {
            continue;
        };
        let answered = ask(&server, method.on_the_wire(), &path, &[]);
        assert_eq!(
            answered.status, row.healthy.status,
            "{} at {path} answered {} rather than the healthy fixture's status",
            row.capability, answered.status
        );
        assert_eq!(answered.reason, row.healthy.reason);
        assert_eq!(
            answered.header("content-type"),
            Some(row.healthy.content_type)
        );
        assert_eq!(answered.declared_length, Some(row.healthy.body.len()));
        if method == Method::Head {
            assert!(
                answered.body.is_empty(),
                "a HEAD answered with a body, which is the one thing it may not do"
            );
        } else {
            assert_eq!(answered.body.as_slice(), row.healthy.body);
        }
    }
}

#[test]
fn every_row_of_the_surface_answers_its_hostile_shape() {
    for (index, row) in SURFACE.iter().enumerate() {
        let Some((method, path)) = a_path_for(row) else {
            continue;
        };
        // A server per row, because two of the hostile shapes change the row's
        // state for every later request and a shared server would make the case
        // order matter.
        let server = FakeServer::start();
        server.misbehave(index);
        match row.hostile {
            Hostile::Withheld => {
                let mut connection = send(&server, method.on_the_wire(), &path, &[]);
                assert!(
                    read_back(&mut connection, Some(LONG_ENOUGH_TO_CALL_IT_SILENCE)).is_none(),
                    "{} answered something where its hostile shape is silence",
                    row.capability
                );
            }
            Hostile::Absent => {
                let answered = ask(&server, method.on_the_wire(), &path, &[]);
                assert_eq!(answered.status, 404, "{} is absent", row.capability);
            }
            Hostile::Answers(answer) | Hostile::WrongContentType(answer) => {
                let answered = ask(&server, method.on_the_wire(), &path, &[]);
                assert_eq!(
                    answered.status, answer.status,
                    "{} answered {} rather than its hostile status",
                    row.capability, answered.status
                );
                assert_eq!(answered.header("content-type"), Some(answer.content_type));
            }
            Hostile::TruncatedBody(answer) | Hostile::UnauthorizedMidStream(answer) => {
                let answered = ask(&server, method.on_the_wire(), &path, &[]);
                assert_eq!(answered.declared_length, Some(answer.body.len()));
                assert!(
                    answered.body.len() < answer.body.len(),
                    "{} declared {} bytes and delivered all of them, so nothing was \
                     truncated",
                    row.capability,
                    answer.body.len()
                );
            }
        }
    }
}

#[test]
fn a_path_the_table_does_not_carry_is_answered_404() {
    let server = FakeServer::start();
    let answered = ask(&server, "GET", "/Nothing/This/Table/Carries", &[]);
    assert_eq!(answered.status, 404);
    assert_eq!(answered.reason, "Not Found");
}

#[test]
fn the_two_capabilities_no_supported_line_offers_have_no_row() {
    for capability in CAPABILITIES_WITH_NO_ROUTE {
        assert!(
            !SURFACE.iter().any(|row| row.capability == *capability),
            "{capability} has a route in the table, and 0010 says no supported \
             line offers one"
        );
    }
    assert_eq!(CAPABILITIES_WITH_NO_ROUTE.len(), 2);
}

#[test]
fn the_table_names_the_sixteen_capabilities_the_record_names() {
    let mut named: Vec<&str> = SURFACE.iter().map(|row| row.capability).collect();
    named.extend_from_slice(CAPABILITIES_WITH_NO_ROUTE);
    named.sort_unstable();
    named.dedup();
    assert_eq!(
        named.len(),
        16,
        "0010 names sixteen capabilities and this table names {}: {named:?}",
        named.len()
    );
}

#[test]
fn no_two_rows_are_reached_the_same_way() {
    // A duplicated row is invisible: the matcher answers one of the two and the
    // other's hostile shape is unreachable, so a case armed against it passes by
    // asking the wrong row.
    let mut reached: Vec<(&str, &str)> = Vec::new();
    let mut upgrades = 0;
    for row in SURFACE {
        match row.reached {
            Reached::Path { method, template } => {
                reached.push((method.on_the_wire(), template));
            }
            Reached::Upgrade => upgrades += 1,
        }
    }
    let before = reached.len();
    reached.sort_unstable();
    reached.dedup();
    assert_eq!(
        reached.len(),
        before,
        "two rows are reached by the same method and path, so one of them can \
         never answer"
    );
    assert_eq!(
        upgrades, 1,
        "0010 names one upgrade and this table names {upgrades}"
    );
}

#[test]
fn every_misbehaviour_the_issue_names_is_on_a_row() {
    let mut withheld = 0;
    let mut truncated = 0;
    let mut wrong_type = 0;
    let mut mid_stream = 0;
    let mut absent = 0;
    for row in SURFACE {
        match row.hostile {
            Hostile::Withheld => withheld += 1,
            Hostile::TruncatedBody(_) => truncated += 1,
            Hostile::WrongContentType(_) => wrong_type += 1,
            Hostile::UnauthorizedMidStream(_) => mid_stream += 1,
            Hostile::Absent => absent += 1,
            Hostile::Answers(_) => {}
        }
    }
    // The five #21's body names, one assertion each rather than a total, because a
    // total is satisfied by five of one.
    assert!(withheld > 0, "no row answers slowly");
    assert!(truncated > 0, "no row truncates a body");
    assert!(wrong_type > 0, "no row declares a type its bytes are not");
    assert!(mid_stream > 0, "no row loses its token mid-stream");
    assert!(absent > 0, "no row is absent");
}

#[test]
fn every_row_carries_a_healthy_fixture_and_a_hostile_one() {
    for row in SURFACE {
        assert!(
            row.healthy.status >= 100,
            "{} carries no healthy status",
            row.capability
        );
        assert!(
            !row.healthy.reason.is_empty(),
            "{} carries a healthy answer with no reason phrase",
            row.capability
        );
        // The hostile shape has to differ from the healthy one in something a
        // caller can see, or the row has a hostile fixture in name only.
        let differs = match row.hostile {
            Hostile::Answers(answer) => {
                answer.status != row.healthy.status || answer.body != row.healthy.body
            }
            Hostile::WrongContentType(answer) => answer.content_type != row.healthy.content_type,
            Hostile::Withheld
            | Hostile::TruncatedBody(_)
            | Hostile::UnauthorizedMidStream(_)
            | Hostile::Absent => true,
        };
        assert!(
            differs,
            "{}'s hostile answer is its healthy one, so nothing hostile is proven \
             by asking for it",
            row.capability
        );
    }
}

#[test]
fn the_healthy_artwork_bytes_are_admitted_and_the_hostile_ones_are_refused() {
    let healthy = admitted(A_SMALL_JPEG).expect("the healthy artwork fixture is an accepted image");
    assert_eq!(healthy.format(), Accepted::Jpeg);
    assert_eq!(healthy.dimensions().width(), 4);
    assert_eq!(healthy.dimensions().height(), 3);
    assert_eq!(
        admitted(NOT_AN_ACCEPTED_IMAGE),
        Err(Refused::TheSignatureMatchedNoAcceptedFormat),
        "the hostile artwork fixture is one this tree already admits, so the row \
         proves nothing about a format outside the accepted set"
    );
}

#[test]
fn a_head_on_the_artwork_path_declares_the_length_and_sends_no_bytes() {
    let server = FakeServer::start();
    let answered = ask(&server, "HEAD", "/Items/an-item/Images/Primary", &[]);
    assert_eq!(answered.status, 200);
    assert_eq!(answered.header("content-type"), Some("image/jpeg"));
    assert_eq!(answered.declared_length, Some(A_SMALL_JPEG.len()));
    assert!(answered.body.is_empty());
}

#[test]
fn a_get_and_a_head_on_the_artwork_path_declare_the_same_answer() {
    let server = FakeServer::start();
    let with_body = ask(&server, "GET", "/Items/an-item/Images/Primary", &[]);
    let without = ask(&server, "HEAD", "/Items/an-item/Images/Primary", &[]);
    assert_eq!(with_body.status, without.status);
    assert_eq!(
        with_body.header("content-type"),
        without.header("content-type")
    );
    assert_eq!(with_body.declared_length, without.declared_length);
}

#[test]
fn the_upgrade_is_answered_with_the_accept_value_for_the_key_it_was_offered() {
    let server = FakeServer::start();
    let answered = ask(
        &server,
        "GET",
        "/anything-at-all",
        &[
            ("Upgrade", "websocket"),
            ("Connection", "Upgrade"),
            ("Sec-WebSocket-Key", UPGRADE_KEY),
        ],
    );
    assert_eq!(answered.status, 101);
    assert_eq!(answered.reason, "Switching Protocols");
    assert_eq!(
        answered.header("sec-websocket-accept"),
        Some(UPGRADE_ACCEPT),
        "the handshake carries an accept value that is not the one this key \
         requires, so a client that checks it would refuse the fake and a client \
         that does not would look correct"
    );
}

#[test]
fn the_upgrade_is_refused_where_its_row_is_misbehaving() {
    let index = SURFACE
        .iter()
        .position(|row| matches!(row.reached, Reached::Upgrade))
        .expect("the upgrade row");
    let server = FakeServer::start();
    server.misbehave(index);
    let answered = ask(
        &server,
        "GET",
        "/anything-at-all",
        &[("Upgrade", "websocket"), ("Sec-WebSocket-Key", UPGRADE_KEY)],
    );
    assert_eq!(
        answered.status, 401,
        "0010 says the connection is authenticated on both lines and refused \
         without a token"
    );
}

#[test]
fn a_token_that_died_mid_stream_answers_401_from_then_on() {
    let index = SURFACE
        .iter()
        .position(|row| matches!(row.hostile, Hostile::UnauthorizedMidStream(_)))
        .expect("a row whose token dies mid-stream");
    let (method, path) = a_path_for(&SURFACE[index]).expect("that row has a path");
    let server = FakeServer::start();
    server.misbehave(index);

    let cut = ask(&server, method.on_the_wire(), &path, &[]);
    assert!(
        cut.body.len() < cut.declared_length.expect("a declared length"),
        "the first answer arrived whole, so nothing died mid-stream"
    );

    let after = ask(&server, method.on_the_wire(), &path, &[]);
    assert_eq!(
        after.status, 401,
        "the row answered {} after its token died, and a dead token answers 401 \
         until somebody signs in again",
        after.status
    );
}

#[test]
fn the_server_records_the_request_it_was_asked() {
    let server = FakeServer::start();
    let _ = ask(
        &server,
        "GET",
        "/Items?SortBy=SortName",
        &[("X-Emby-Authorization", "a value a test wrote")],
    );
    let seen: Vec<Seen> = server.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].method, "GET");
    assert_eq!(seen[0].target, "/Items?SortBy=SortName");
    assert_eq!(
        seen[0].header("x-emby-authorization"),
        Some("a value a test wrote"),
        "the record lost a header, and a test that asserts what the core sent has \
         nothing to assert against"
    );
}

#[test]
fn the_server_records_the_body_a_write_carried() {
    // The bytes a write carries are what #47's queue and #57's cadence will be
    // asserted against, and a record that dropped them would let a case about
    // what the core sent assert against nothing.
    let server = FakeServer::start();
    let written = b"a body a test wrote, and no field name a server was ever asked for";
    let answered = ask_carrying(
        &server,
        "POST",
        "/Sessions/Playing/Progress",
        &[("Content-Type", "application/json")],
        written,
    );
    assert_eq!(answered.status, 200);
    let seen = server.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].body.as_slice(),
        written.as_slice(),
        "the record kept {} bytes of a body that was {} long",
        seen[0].body.len(),
        written.len()
    );
}

#[test]
fn a_query_string_does_not_change_which_row_answers() {
    let server = FakeServer::start();
    let bare = ask(&server, "GET", "/Items", &[]);
    let with_query = ask(&server, "GET", "/Items?SortBy=SortName&Limit=20", &[]);
    assert_eq!(bare.status, with_query.status);
    assert_eq!(bare.body, with_query.body);
}

#[test]
fn two_rows_of_one_length_answer_the_request_each_was_written_for() {
    let server = FakeServer::start();
    let list = ask(&server, "GET", "/Items", &[]);
    let one = ask(&server, "GET", "/Items/an-item", &[]);
    assert_eq!(list.status, 200);
    assert_eq!(one.status, 200);
    let seen = server.seen();
    assert_eq!(seen[0].target, "/Items");
    assert_eq!(seen[1].target, "/Items/an-item");
}

#[test]
fn a_literal_row_wins_over_one_naming_a_caller_supplied_segment() {
    // NO PAIR IN 0010's OWN TABLE COLLIDES. Every pair sharing a method differs
    // in segment count or in a literal segment, so the preference for a literal
    // has no subject there and a case written against `SURFACE` would pass with
    // the preference deleted. The pair below is a fixture: two rows of the same
    // method and the same length, one literal where the other is a
    // caller-supplied segment, in the order that makes the wrong answer the
    // convenient one.
    let colliding: [Row; 2] = [
        Row {
            capability: "a-row-naming-a-caller-supplied-segment",
            reached: Reached::Path {
                method: Method::Get,
                template: "/Items/{itemId}",
            },
            healthy: A_ROW_THAT_ANSWERS,
            hostile: Hostile::Absent,
        },
        Row {
            capability: "a-row-that-is-all-literal",
            reached: Reached::Path {
                method: Method::Get,
                template: "/Items/Resume",
            },
            healthy: A_ROW_THAT_ANSWERS,
            hostile: Hostile::Absent,
        },
    ];
    let asked = a_request_for("GET", "/Items/Resume");
    assert_eq!(
        matching_row_in(&colliding, &asked),
        Some(1),
        "the row naming a caller-supplied segment took a request the literal row \
         was written for, so a row added later can shadow one already in the table"
    );
    // The neighbour, one segment different: with no literal row to prefer, the
    // caller-supplied one is the answer rather than nothing.
    let other = a_request_for("GET", "/Items/an-item");
    assert_eq!(matching_row_in(&colliding, &other), Some(0));
}

#[test]
fn no_test_target_in_this_repository_reaches_anything_but_loopback() {
    // The whole of what this case can say is about the server this file starts.
    // Whether some other file opens a socket is not something a running test can
    // read, and the negative in #21's fourth condition is carried by the
    // arrangement rather than by an assertion: the one harness for a real server
    // is `tests/needs_a_real_server_or_real_hardware.rs`, it is declared with
    // `test = false` so `cargo test --locked` never invokes it, and it carries no
    // case. That is stated in the pull request with the commands behind it, and
    // this case asserts the part it can.
    let server = FakeServer::start();
    assert!(server.address().ip().is_loopback());
}

#[test]
fn the_controlled_clock_moves_forward_by_what_a_test_asks_for() {
    let clocks = ControlledClocks::started();
    let started = clocks.steady();
    clocks.advance_steady(1_500);
    assert_eq!(clocks.steady().interval_since(started).as_nanos(), 1_500);
}

#[test]
fn a_suspension_moves_the_elapsed_clock_and_leaves_the_steady_one_standing() {
    let clocks = ControlledClocks::started();
    let steady_before = clocks.steady();
    let elapsed_before = clocks.elapsed();
    clocks.advance_elapsed(9_000);
    assert_eq!(clocks.steady().interval_since(steady_before).as_nanos(), 0);
    assert_eq!(
        clocks.elapsed().interval_since(elapsed_before).as_nanos(),
        9_000
    );
}

#[test]
fn the_wall_clock_is_set_in_both_directions() {
    let clocks = ControlledClocks::started();
    let power_cut = WallMoment::from_epoch(0, 0);
    clocks.set_wall(power_cut);
    assert_eq!(clocks.wall(), power_cut);
    let set_forward = WallMoment::from_epoch(4_000_000_000, 0);
    clocks.set_wall(set_forward);
    assert_eq!(clocks.wall(), set_forward);
    let before_the_epoch = WallMoment::from_epoch(-86_400, 5);
    clocks.set_wall(before_the_epoch);
    assert_eq!(clocks.wall().seconds_from_the_epoch(), -86_400);
}

#[test]
#[should_panic(expected = "the steady clock may not be wound back")]
fn the_source_refuses_to_wind_the_steady_clock_back() {
    let clocks = ControlledClocks::started();
    clocks.advance_steady(1_000);
    clocks.set_steady(5);
}

#[test]
#[should_panic(expected = "the elapsed clock may not be wound back")]
fn the_source_refuses_to_wind_the_elapsed_clock_back() {
    let clocks = ControlledClocks::started();
    clocks.advance_elapsed(1_000);
    clocks.set_elapsed(5);
}

#[test]
fn the_source_counts_what_was_read_out_of_it() {
    let clocks = ControlledClocks::started();
    assert_eq!(clocks.readings_taken(), (0, 0, 0));
    let _ = clocks.steady();
    let _ = clocks.steady();
    let _ = clocks.wall();
    assert_eq!(clocks.readings_taken(), (2, 0, 1));
}

#[test]
fn a_setting_that_does_not_move_a_monotonic_clock_is_allowed() {
    // The near miss beside the two refusals above. The rule is that a monotonic
    // clock may not go backwards, and a test setting one to where it already
    // stands has not moved it. A refusal written as `>` rather than `>=` would
    // refuse this and read exactly like the correct one.
    let clocks = ControlledClocks::started();
    clocks.advance_steady(400);
    clocks.set_steady(1_400);
    let standing = clocks.steady();
    clocks.set_steady(1_400);
    assert_eq!(clocks.steady().interval_since(standing).as_nanos(), 0);
}
