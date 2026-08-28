//! Every hostile shape the fake server can answer with, driven and mapped (#37).
//!
//! #37 asks for a test that drives the fake server through at least a dozen
//! hostile responses and asserts the named kind for each. This is that test. It
//! runs against `127.0.0.1` on a port the operating system chose, with no
//! external network, and it drives one row of 0010's table at a time so that a
//! failure names the capability it came from.
//!
//! # What plays the caller, and why that matters here
//!
//! There is no transport. #27 is where one arrives, so what reads the answer is
//! `fake_server::client` and what decides the kind is
//! `flowfin_core::failure::Failure`. The split is the point: the client learns
//! the status, whether the body arrived whole, and what the bytes were, and it
//! hands those to the one mapping point rather than deciding anything itself.
//! When #27 lands, what changes here is who reads the socket and not what maps.
//!
//! # What this cannot say
//!
//! It proves the sites it reached. 0037 says so in as many words: that every
//! failure the core produces went through the mapping point is a property of the
//! source rather than of a run, and the honest form of that proof is a check over
//! the tree. What holds it today is the compiler, because no variant of the
//! vocabulary can be built outside `src/failure/`, and the deliberate violation
//! that shows it is in the pull request rather than here - a failure to compile
//! is not something a test can assert.
//!
//! Nothing here is a recording from a real server. Every answer comes from
//! `fake_server::surface`, which says so about itself, and #104 is where a
//! fixture is held honest against a real one.

mod fake_server;

use fake_server::FakeServer;
use fake_server::client::{LONG_ENOUGH_TO_CALL_IT_SILENCE, ask, ask_carrying, read_back, send};
use fake_server::surface::{Hostile, Method, Reached, Row, SURFACE};
use flowfin_core::artwork::format::admitted;
use flowfin_core::failure::{
    Answered, Attempt, Capability, Deadline, Expected, Failure, Kind, ReadingSite, TransportOutcome,
};

/// What a caller makes of each row's hostile shape, in `SURFACE` order.
///
/// `None` is a row whose hostile shape is hostile to a caller that trusts a
/// declaration and is not a failure to this core. Both of them are the same rule
/// arriving twice: 0101 puts a server's declared content type on its untrusted
/// list and 0055 takes the consequence, so a type that disagrees with the bytes
/// is not an error in itself and the bytes decide.
const WHAT_THE_HOSTILE_SHAPE_BECOMES: &[Option<Kind>] = &[
    // server-identity, answered as HTML with a body that is still readable.
    None,
    // password-sign-in, credentials refused.
    Some(Kind::NotAuthenticated),
    // quick-connect enabled, the route is gone from a path naming nothing.
    Some(Kind::CapabilityAbsent),
    // quick-connect initiate, the operator turned the route off mid-exchange.
    Some(Kind::NotAuthenticated),
    // quick-connect connect, a secret the server does not hold.
    Some(Kind::NotFound),
    // authenticate with quick connect, no answer at all.
    Some(Kind::TimedOut),
    // sign-out with a token that is already dead.
    Some(Kind::NotAuthenticated),
    // device-capabilities, a route this server does not have.
    Some(Kind::CapabilityAbsent),
    // the top of the library, cut off part way through.
    Some(Kind::ServerUnreachable),
    // the query surface, answering nothing.
    Some(Kind::TimedOut),
    // one item that is not there.
    Some(Kind::NotFound),
    // the resume list on a server refusing load.
    Some(Kind::ServerBusy),
    // user data for an item that is not there.
    Some(Kind::NotFound),
    // user data written while the token dies part way through.
    Some(Kind::ServerUnreachable),
    // artwork in a format 0055 does not accept.
    Some(Kind::AnswerNotUnderstood),
    // a HEAD declaring a format the bytes on the GET are not.
    None,
    // playback selection refused as wrong.
    Some(Kind::RequestRefused),
    // playback started with a dead token.
    Some(Kind::NotAuthenticated),
    // the progress cadence answering nothing.
    Some(Kind::TimedOut),
    // the last report of a session, and the server broke.
    Some(Kind::ServerFailed),
    // the played mark, set on an item that is not there.
    Some(Kind::NotFound),
    // the played mark, cleared and cut off part way.
    Some(Kind::ServerUnreachable),
    // the change-notification upgrade, refused without a token.
    Some(Kind::NotAuthenticated),
];

/// The capability of the core's own set that this row names.
///
/// It is looked up by the name rather than written twice, which makes every case
/// below also a comparison of two sets: the table the fake serves and the set
/// `capability-absent` carries. A row naming something the core has no value for
/// fails here rather than arriving as a string nobody can group.
fn capability_of(row: &Row) -> Capability {
    const ALL: &[Capability] = &[
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
    *ALL.iter()
        .find(|capability| capability.declared_name() == row.capability)
        .unwrap_or_else(|| {
            panic!(
                "the table names the capability {} and the core has no value for it",
                row.capability
            )
        })
}

/// The caller-supplied value a 404 on this row could be about.
///
/// 0004 makes the 404 split depend on the core's own list rather than on the
/// answer, and 0010 fixes the list: a path carrying no caller-supplied identifier
/// can only be missing its route. The brace in the template is that, with one
/// exception 0010 names by hand, where the caller-supplied value is a secret in
/// the query string rather than a segment of the path.
fn identifier_for(row: &Row) -> Option<&'static str> {
    let Reached::Path { template, .. } = row.reached else {
        return None;
    };
    if template.contains('{') || template == "/QuickConnect/Connect" {
        return Some("a-value-the-caller-supplied");
    }
    None
}

/// A path a row is reached at, with a value in place of every caller-supplied
/// segment.
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

/// Drives one row's hostile shape and hands what came back to the mapping point.
///
/// `None` is an answer this core does not treat as a failure at all.
fn what_a_caller_makes_of(index: usize) -> Option<Failure> {
    let row = &SURFACE[index];
    // A server per row, because two of the hostile shapes change the row's state
    // for every later request and a shared server would make the case order
    // matter.
    let server = FakeServer::start();
    server.misbehave(index);

    let answered = Answered {
        capability: capability_of(row),
        identifier: identifier_for(row),
        retry_after: None,
        server_code: None,
    };
    let address = server.base_address();

    let (method, path, extra) = match row.reached {
        Reached::Path { method, .. } => {
            let (_, path) = a_path_for(row).expect("a row reached at a path");
            (method.on_the_wire(), path, Vec::new())
        }
        // 0010 names the upgrade and gives no path, so the request carries the
        // header the fake matches on rather than a path that record does not
        // have.
        Reached::Upgrade => (
            "GET",
            "/the-origin-the-upgrade-is-made-on".to_owned(),
            vec![
                ("Upgrade", "websocket"),
                ("Sec-WebSocket-Key", fake_server::surface::UPGRADE_KEY),
            ],
        ),
    };

    match row.hostile {
        Hostile::Withheld => {
            let mut connection = send(&server, method, &path, &extra);
            assert!(
                read_back(&mut connection, Some(LONG_ENOUGH_TO_CALL_IT_SILENCE)).is_none(),
                "{} answered where its hostile shape is silence",
                row.capability
            );
            Some(Failure::from_transport(
                &TransportOutcome::DeadlineReached {
                    deadline: Deadline::FirstByte,
                    // The client's own wait rather than a deadline the core
                    // holds. 0027's two seconds are what the transport in #27
                    // will pass here.
                    elapsed: LONG_ENOUGH_TO_CALL_IT_SILENCE,
                },
                &Attempt {
                    address: &address,
                    bytes_reached_the_server: true,
                },
            ))
        }
        Hostile::TruncatedBody(_) | Hostile::UnauthorizedMidStream(_) => {
            // A body carried on the write, so that the record on the server side
            // holds what a later cadence in #57 will be asserted against.
            let received = ask_carrying(&server, method, &path, &extra, b"a body a test wrote");
            let declared = received.declared_length.expect("a declared length");
            assert!(
                received.body.len() < declared,
                "{} declared {declared} bytes and delivered all of them",
                row.capability
            );
            Some(Failure::from_transport(
                &TransportOutcome::ConnectionDroppedMidBody,
                &Attempt {
                    address: &address,
                    bytes_reached_the_server: true,
                },
            ))
        }
        Hostile::WrongContentType(_) => {
            let received = ask(&server, method, &path, &extra);
            // The declared type decides nothing. What the core does with the
            // bytes is what decides, and both rows carrying this shape hand back
            // bytes the core can still read.
            assert_ne!(
                received.header("content-type"),
                Some(row.healthy.content_type),
                "{} declared the healthy type, so nothing disagreed",
                row.capability
            );
            None
        }
        Hostile::Absent | Hostile::Answers(_) => {
            let received = ask(&server, method, &path, &extra);
            // An artwork payload is judged on its bytes before anything else,
            // which is 0055's order, so a 200 carrying a format the core does not
            // accept never reaches the status table.
            if received.status == 200
                && received
                    .header("content-type")
                    .is_some_and(|declared| declared.starts_with("image/"))
            {
                let _refused = admitted(&received.body)
                    .expect_err("the hostile artwork fixture is admitted, so nothing was refused");
                return Some(Failure::answer_not_understood(
                    ReadingSite::ImageFormatRefused,
                    Expected::AnAcceptedImageFormat,
                    0,
                ));
            }
            Some(Failure::from_status(received.status, &answered))
        }
    }
}

#[test]
fn every_row_of_the_surface_has_an_expectation_beside_it() {
    assert_eq!(
        WHAT_THE_HOSTILE_SHAPE_BECOMES.len(),
        SURFACE.len(),
        "a row was added to 0010's table with nothing here saying what its \
         hostile shape becomes, and a loop over the shorter of the two would \
         pass over it"
    );
}

#[test]
fn every_hostile_answer_becomes_the_kind_it_should() {
    let mut named = 0;
    for (index, expected) in WHAT_THE_HOSTILE_SHAPE_BECOMES.iter().enumerate() {
        let row = &SURFACE[index];
        let mapped = what_a_caller_makes_of(index);
        match (*expected, mapped) {
            (Some(kind), Some(failure)) => {
                assert_eq!(
                    failure.kind(),
                    kind,
                    "{} at row {index} became {} rather than {}",
                    row.capability,
                    failure.kind().declared_name(),
                    kind.declared_name()
                );
                named += 1;
            }
            (None, None) => {}
            (expected, mapped) => panic!(
                "{} at row {index} expected {expected:?} and produced {:?}",
                row.capability,
                mapped.map(|f| f.kind())
            ),
        }
    }
    assert!(
        named >= 12,
        "#37 asks for at least a dozen hostile responses with a named kind each, \
         and this run named {named}"
    );
}

#[test]
fn a_dead_token_after_a_cut_answer_is_not_authenticated_rather_than_unreachable() {
    // The one row whose hostile shape changes what the next request gets. The
    // first answer is a body cut off, which is a transport condition; the second
    // is the token being gone, which is a status. A caller that mapped the second
    // the way it mapped the first would retry a request nothing will accept.
    let index = SURFACE
        .iter()
        .position(|row| matches!(row.hostile, Hostile::UnauthorizedMidStream(_)))
        .expect("a row whose token dies mid-stream");
    let row = &SURFACE[index];
    let (method, path) = a_path_for(row).expect("a row reached at a path");
    let server = FakeServer::start();
    server.misbehave(index);

    let cut = ask(&server, method.on_the_wire(), &path, &[]);
    assert!(cut.body.len() < cut.declared_length.expect("a declared length"));

    let after = ask(&server, method.on_the_wire(), &path, &[]);
    let answered = Answered {
        capability: capability_of(row),
        identifier: identifier_for(row),
        retry_after: None,
        server_code: None,
    };
    let mapped = Failure::from_status(after.status, &answered);
    assert_eq!(mapped.kind(), Kind::NotAuthenticated);
    let Failure::NotAuthenticated {
        a_token_was_presented,
        ..
    } = mapped
    else {
        panic!("a dead token mapped onto something else");
    };
    assert!(
        a_token_was_presented,
        "#35 acts on the difference between a token that was rejected and no \
         token to present, and this call had one"
    );
}

#[test]
fn a_capability_absent_from_a_route_names_the_capability_the_call_was_made_under() {
    let index = SURFACE
        .iter()
        .position(|row| matches!(row.hostile, Hostile::Absent))
        .expect("a row whose route is absent");
    let row = &SURFACE[index];
    let mapped = what_a_caller_makes_of(index).expect("an absent route is a failure");
    let Failure::CapabilityAbsent { capability, .. } = mapped else {
        panic!("an absent route mapped onto something else");
    };
    assert_eq!(
        capability.declared_name(),
        row.capability,
        "the kind named a capability other than the one the call was made under, \
         which is what an operator reads when they are told to upgrade a server"
    );
}

#[test]
fn a_refused_image_is_counted_apart_from_an_answer_the_core_could_not_read() {
    // 0055 pushes a refused format under `answer-not-understood` knowing the fit
    // is wrong, and names the measurement that would overturn it. This is the
    // field that makes that measurement readable: the kind is the same and the
    // site is not.
    let index = SURFACE
        .iter()
        .position(|row| {
            row.capability == "artwork"
                && matches!(
                    row.reached,
                    Reached::Path {
                        method: Method::Get,
                        ..
                    }
                )
        })
        .expect("the artwork row");
    let mapped = what_a_caller_makes_of(index).expect("a refused format is a failure");
    assert_eq!(mapped.kind(), Kind::AnswerNotUnderstood);
    let Failure::AnswerNotUnderstood { site, expected, .. } = mapped else {
        panic!("a refused format mapped onto something else");
    };
    assert_eq!(site, ReadingSite::ImageFormatRefused);
    assert_ne!(
        site,
        ReadingSite::AnswerBody,
        "a refused format is counted with the answers the core could not read, \
         and the three repairs behind that number are different"
    );
    assert_eq!(expected, Expected::AnAcceptedImageFormat);
}
