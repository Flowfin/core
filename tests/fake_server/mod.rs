//! The fake server the suite runs against (#21).
//!
//! Nothing on this board may require a running Jellyfin server to test. A suite
//! that needs one is green on the machine with the server and unrunnable
//! everywhere else, which is the fastest way to lose the headless property #20
//! made a birth requirement.
//!
//! # What it is
//!
//! A loopback HTTP/1.1 server the suite starts and stops in the process running
//! the tests. It binds `127.0.0.1` and port zero, so the operating system chooses
//! a free port and two test binaries running at once cannot collide on one. The
//! bind is to loopback rather than to the machine's own interface address on
//! purpose: `CONTRIBUTING.md` records that the second raises a firewall consent
//! dialog on Windows, answered by an administrator, whose subject is the
//! executable's path, so answering it settles nothing for the next build
//! directory.
//!
//! # What it serves
//!
//! `surface::SURFACE` is the table, transcribed from 0010. Every row carries a
//! healthy answer and one hostile shape, and a path this table does not carry is
//! answered 404 - which is 0010's fallback rule arriving from a server rather
//! than from a branch somewhere in the core.
//!
//! # No dependency
//!
//! The manifest admits a dependency under 0103's rules and none of them reaches
//! here. This is `std::net` and `std::thread` and nothing else: an HTTP server
//! framework would be a dependency taken for test infrastructure, with a licence
//! set and four behaviours to argue, in exchange for framing that fits in one
//! file. `no-network-outside-the-transport` refuses `std::net` under `src/` and
//! this file is not under `src/`, which is where that rule's boundary is rather
//! than a way around it.
//!
//! # What it cannot prove
//!
//! A fake proves the core's reaction to a shape. It cannot prove that the shape
//! is what a real server sends. No answer in `surface.rs` is a recording; #104 is
//! where a fixture is held honest against a real server and it is open.

// TWO TEST BINARIES COMPILE THIS MODULE AND EACH USES A SUBSET OF IT, WHICH IS
// WHAT THIS ALLOW IS FOR AND THE WHOLE OF WHAT IT IS FOR. A shared module under
// tests/ is compiled once per target that declares it, and dead-code analysis
// runs per target, so an item both binaries need but only one of them calls is
// reported against the other. The alternatives are worse in the direction that
// matters: making every target touch every item distorts the cases into
// exercising scaffolding rather than behaviour, and splitting the module per
// target is the second copy of a fake that stops agreeing with the first.
//
// What it costs is stated rather than hidden: an item here that NO target uses
// is not reported either, so a reader who wants to know whether something is
// still driven greps for it rather than trusting a green build.
#![allow(dead_code)]

pub mod client;
pub mod clock;
pub mod surface;

use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use surface::{Answer, Hostile, Method, Reached, Row, SURFACE};

/// How long a connection the server is deliberately not answering stays open
/// before the server lets go of it.
///
/// It is a backstop against a hung suite rather than a deadline anything is
/// measured against. A test meets the silence through its own read timeout and
/// returns long before this; what this bounds is the thread, in the case where a
/// test panicked while the connection was open.
const A_WITHHELD_CONNECTION_IS_LET_GO_AFTER: Duration = Duration::from_secs(5);

/// One request the server answered, as it arrived.
#[derive(Debug, Clone)]
pub struct Seen {
    /// The method token from the request line.
    pub method: String,
    /// The request target, query string included.
    pub target: String,
    /// The header names, lowercased, with their values.
    pub headers: Vec<(String, String)>,
    /// The body, exactly as many bytes as the request declared.
    pub body: Vec<u8>,
}

impl Seen {
    /// The value of a header, matched without regard to case.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let wanted = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(had, _)| *had == wanted)
            .map(|(_, value)| value.as_str())
    }
}

/// What every connection thread shares with the test that started the server.
struct Shared {
    /// The rows a test has asked to misbehave, by their index in `SURFACE`.
    misbehaving: Mutex<HashSet<usize>>,
    /// The rows whose token has already died mid-stream, by the same index. A
    /// row lands here after `UnauthorizedMidStream` has cut one answer, and every
    /// later request on it is answered 401, which is what a dead token does.
    token_died: Mutex<HashSet<usize>>,
    /// Every request the server answered, in order.
    seen: Mutex<Vec<Seen>>,
    /// False from the moment the server is being taken down.
    running: AtomicBool,
}

/// A loopback server the suite starts and stops.
///
/// It stops when it is dropped, which is what makes a test that panicked leave no
/// listener behind.
pub struct FakeServer {
    address: SocketAddr,
    shared: Arc<Shared>,
    accepting: Option<JoinHandle<()>>,
}

impl FakeServer {
    /// Starts a server on a loopback address the operating system chooses.
    ///
    /// # Panics
    ///
    /// When the bind fails. There is nothing useful a test can do with a server
    /// that is not listening, and a returned error here would be checked by the
    /// first caller and unwrapped by every one after it.
    #[must_use]
    pub fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .expect("a loopback listener on a port the operating system chooses");
        let address = listener
            .local_addr()
            .expect("the address the listener actually bound");
        let shared = Arc::new(Shared {
            misbehaving: Mutex::new(HashSet::new()),
            token_died: Mutex::new(HashSet::new()),
            seen: Mutex::new(Vec::new()),
            running: AtomicBool::new(true),
        });
        let accepting = std::thread::spawn({
            let shared = Arc::clone(&shared);
            move || accept_until_stopped(&listener, &shared)
        });
        Self {
            address,
            shared,
            accepting: Some(accepting),
        }
    }

    /// The address the server is listening on.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// The base address a request path is joined to, in the form 0028 resolves.
    #[must_use]
    pub fn base_address(&self) -> String {
        format!("http://{}", self.address)
    }

    /// Makes the row at `index` in `SURFACE` answer its hostile shape instead of
    /// its healthy one.
    ///
    /// # Panics
    ///
    /// When `index` is past the end of the table, because a test naming a row
    /// that does not exist is a test whose subject moved.
    pub fn misbehave(&self, index: usize) {
        assert!(index < SURFACE.len(), "no row {index} in 0010's table");
        self.shared
            .misbehaving
            .lock()
            .expect("the misbehaviour set")
            .insert(index);
    }

    /// Every request the server has answered so far.
    ///
    /// # Panics
    ///
    /// When a connection thread panicked while holding the record, which is a
    /// defect in this file rather than in the test reading it.
    #[must_use]
    pub fn seen(&self) -> Vec<Seen> {
        self.shared.seen.lock().expect("the request record").clone()
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.shared.running.store(false, Ordering::SeqCst);
        // The accept call blocks, so it is woken by one connection that carries
        // no request rather than by a timeout somebody has to choose.
        if let Ok(waker) = TcpStream::connect(self.address) {
            let _ = waker.shutdown(Shutdown::Both);
        }
        if let Some(accepting) = self.accepting.take() {
            let _ = accepting.join();
        }
    }
}

/// Accepts connections until the server is taken down, one thread per
/// connection.
///
/// One thread per connection rather than one loop, because a row answering
/// `Withheld` holds its connection open for as long as the test wants it held,
/// and a single loop would stop answering everything else while it did.
fn accept_until_stopped(listener: &TcpListener, shared: &Arc<Shared>) {
    let mut connections: Vec<JoinHandle<()>> = Vec::new();
    while shared.running.load(Ordering::SeqCst) {
        let Ok((connection, _)) = listener.accept() else {
            break;
        };
        if !shared.running.load(Ordering::SeqCst) {
            break;
        }
        let shared = Arc::clone(shared);
        connections.push(std::thread::spawn(move || answer(connection, &shared)));
    }
    for connection in connections {
        let _ = connection.join();
    }
}

/// Reads one request off `connection` and answers it.
fn answer(mut connection: TcpStream, shared: &Arc<Shared>) {
    let Some(request) = read_request(&mut connection) else {
        return;
    };
    let matched = matching_row(&request);
    shared
        .seen
        .lock()
        .expect("the request record")
        .push(request.clone());

    let Some(index) = matched else {
        write_answer(&mut connection, &not_found(), false);
        return;
    };
    let row = &SURFACE[index];
    if shared
        .token_died
        .lock()
        .expect("the dead-token set")
        .contains(&index)
    {
        write_answer(&mut connection, &unauthorized(), false);
        return;
    }
    let misbehaving = shared
        .misbehaving
        .lock()
        .expect("the misbehaviour set")
        .contains(&index);
    if !misbehaving {
        let headers_only = matches!(
            row.reached,
            Reached::Path {
                method: Method::Head,
                ..
            }
        );
        write_answer_for(&mut connection, row, &row.healthy, headers_only);
        return;
    }
    answer_hostile(&mut connection, shared, index, row);
}

/// Answers the row's hostile shape.
fn answer_hostile(connection: &mut TcpStream, shared: &Arc<Shared>, index: usize, row: &Row) {
    let headers_only = matches!(
        row.reached,
        Reached::Path {
            method: Method::Head,
            ..
        }
    );
    match row.hostile {
        Hostile::Answers(answer) | Hostile::WrongContentType(answer) => {
            write_answer(connection, &answer, headers_only);
        }
        Hostile::Absent => write_answer(connection, &not_found(), headers_only),
        Hostile::TruncatedBody(answer) => write_truncated(connection, &answer),
        Hostile::UnauthorizedMidStream(answer) => {
            shared
                .token_died
                .lock()
                .expect("the dead-token set")
                .insert(index);
            write_truncated(connection, &answer);
        }
        Hostile::Withheld => withhold(connection),
    }
}

/// Writes the answer a row gives, which for the upgrade row is a handshake
/// rather than a body.
fn write_answer_for(connection: &mut TcpStream, row: &Row, answer: &Answer, headers_only: bool) {
    if matches!(row.reached, Reached::Upgrade) && answer.status == 101 {
        write_upgrade(connection);
        return;
    }
    write_answer(connection, answer, headers_only);
}

/// Reads one request, or `None` where the connection carried none.
fn read_request(connection: &mut TcpStream) -> Option<Seen> {
    // The read timeout bounds a connection that opened and sent nothing, which is
    // the one the shutdown in `Drop` makes on purpose. It is not a deadline
    // anything here is measured against.
    let _ = connection.set_read_timeout(Some(A_WITHHELD_CONNECTION_IS_LET_GO_AFTER));
    let mut received = Vec::new();
    let mut chunk = [0_u8; 1024];
    let head_ends_at = loop {
        if let Some(at) = position_of(&received, b"\r\n\r\n") {
            break at;
        }
        let read = connection.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        received.extend_from_slice(&chunk[..read]);
    };
    let head = String::from_utf8(received[..head_ends_at].to_vec()).ok()?;
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next()?.split(' ');
    let method = request_line.next()?.to_owned();
    let target = request_line.next()?.to_owned();
    let mut headers = Vec::new();
    for line in lines {
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }
    let declared = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = received[head_ends_at + 4..].to_vec();
    while body.len() < declared {
        let read = connection.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(declared);
    Some(Seen {
        method,
        target,
        headers,
        body,
    })
}

/// Where `needle` starts in `haystack`.
fn position_of(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Which row of 0010's table this request reaches, if any.
///
/// An upgrade is matched by the request's own header, because 0010 names the
/// upgrade and gives no path for it. Everything else is matched by method and
/// path, and where two templates match, the one with more literal segments wins,
/// so a row naming a caller-supplied segment never takes a request a literal row
/// was written for.
fn matching_row(request: &Seen) -> Option<usize> {
    matching_row_in(SURFACE, request)
}

/// The same, against any table.
///
/// It takes the table rather than reading `SURFACE` so that the preference for a
/// literal segment can be proven against a pair of rows that actually collide.
/// No pair in 0010's own table does - every pair sharing a method either differs
/// in segment count or differs in a literal segment - so a rule proven only
/// against `SURFACE` would be a rule nothing bites on, and deleting it would
/// leave the suite green.
pub fn matching_row_in(table: &[Row], request: &Seen) -> Option<usize> {
    if request
        .header("upgrade")
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
    {
        return table
            .iter()
            .position(|row| matches!(row.reached, Reached::Upgrade));
    }
    let method = Method::from_the_wire(&request.method)?;
    let path = request
        .target
        .split_once('?')
        .map_or(request.target.as_str(), |(before, _)| before);
    let mut best: Option<(usize, usize)> = None;
    for (index, row) in table.iter().enumerate() {
        let Reached::Path {
            method: wanted,
            template,
        } = row.reached
        else {
            continue;
        };
        if wanted != method {
            continue;
        }
        let Some(literals) = literal_segments_matched(template, path) else {
            continue;
        };
        if best.is_none_or(|(_, best_literals)| literals > best_literals) {
            best = Some((index, literals));
        }
    }
    best.map(|(index, _)| index)
}

/// How many literal segments of `template` matched `path`, or `None` where the
/// path is not this template's at all.
fn literal_segments_matched(template: &str, path: &str) -> Option<usize> {
    let template_segments: Vec<&str> = template.split('/').collect();
    let path_segments: Vec<&str> = path.split('/').collect();
    if template_segments.len() != path_segments.len() {
        return None;
    }
    let mut literals = 0;
    for (wanted, had) in template_segments.iter().zip(path_segments.iter()) {
        if wanted.starts_with('{') && wanted.ends_with('}') {
            if had.is_empty() {
                return None;
            }
        } else if wanted == had {
            literals += 1;
        } else {
            return None;
        }
    }
    Some(literals)
}

/// The 404 a path this table does not carry is answered with.
const fn not_found() -> Answer {
    Answer {
        status: 404,
        reason: "Not Found",
        content_type: "application/json; charset=utf-8",
        body: b"{}",
    }
}

/// The 401 a row whose token has died answers from then on.
const fn unauthorized() -> Answer {
    Answer {
        status: 401,
        reason: "Unauthorized",
        content_type: "application/json; charset=utf-8",
        body: b"{}",
    }
}

/// Writes a whole answer and closes the connection.
fn write_answer(connection: &mut TcpStream, answer: &Answer, headers_only: bool) {
    let mut wire = header_bytes(answer);
    if !headers_only {
        wire.extend_from_slice(answer.body);
    }
    let _ = connection.write_all(&wire);
    let _ = connection.flush();
}

/// Writes the header and stops the body short of what the header declared.
///
/// One byte short rather than half, because a caller that reads until the
/// connection ends and never compares the count against the declared length
/// passes on any truncation, and the smallest one is the one that is hardest to
/// notice by eye in a failure message.
fn write_truncated(connection: &mut TcpStream, answer: &Answer) {
    let mut wire = header_bytes(answer);
    let stop_at = answer.body.len().saturating_sub(1);
    wire.extend_from_slice(&answer.body[..stop_at]);
    let _ = connection.write_all(&wire);
    let _ = connection.flush();
    let _ = connection.shutdown(Shutdown::Both);
}

/// Holds the connection open and writes nothing.
fn withhold(connection: &mut TcpStream) {
    let _ = connection.set_read_timeout(Some(A_WITHHELD_CONNECTION_IS_LET_GO_AFTER));
    let mut discarded = [0_u8; 256];
    // Reading rather than sleeping: the thread ends when the test drops its own
    // socket, which is a signal rather than an interval, and the timeout above is
    // the backstop for a test that panicked while the connection was open.
    while let Ok(read) = connection.read(&mut discarded) {
        if read == 0 {
            break;
        }
    }
}

/// The handshake the upgrade row answers.
fn write_upgrade(connection: &mut TcpStream) {
    let wire = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         \r\n",
        surface::UPGRADE_ACCEPT
    );
    let _ = connection.write_all(wire.as_bytes());
    let _ = connection.flush();
}

/// The status line and headers of an answer, with the length it declares.
fn header_bytes(answer: &Answer) -> Vec<u8> {
    // Pushed rather than formatted. It reads as the wire it is producing, and the
    // analyser this gate runs refuses a format string appended to a string it
    // already has.
    let mut head = String::from("HTTP/1.1 ");
    head.push_str(&answer.status.to_string());
    head.push(' ');
    head.push_str(answer.reason);
    head.push_str("\r\n");
    if !answer.content_type.is_empty() {
        head.push_str("Content-Type: ");
        head.push_str(answer.content_type);
        head.push_str("\r\n");
    }
    head.push_str("Content-Length: ");
    head.push_str(&answer.body.len().to_string());
    head.push_str("\r\n");
    head.push_str("Connection: close\r\n\r\n");
    head.into_bytes()
}
