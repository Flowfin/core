//! The client the suite drives the fake server with.
//!
//! The transport is #27 and does not exist. This reads a status line, headers and
//! a body over a loopback socket and does nothing else. It is deliberately not a
//! general client: the moment #27 lands, what drives these cases should be the
//! real one, and a capable client written now would be a second transport nobody
//! decided to have.
//!
//! It lives beside the fake rather than inside one test file because two targets
//! drive the same server and a second copy of a client is a second thing to keep
//! in step.

use super::FakeServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// How long a test waits on a connection the server is deliberately not
/// answering before it calls the silence silence.
///
/// This is the test waiting rather than the core, and it is the one interval in
/// the suite measured against real time. The core's own deadlines are on the
/// injected source in `super::clock`, where a test moves the clock instead of
/// waiting.
pub const LONG_ENOUGH_TO_CALL_IT_SILENCE: Duration = Duration::from_millis(50);

/// What the client above read back.
pub struct Received {
    /// The status line's number.
    pub status: u16,
    /// The status line's reason phrase.
    pub reason: String,
    /// The header names, lowercased, with their values.
    pub headers: Vec<(String, String)>,
    /// The bytes that arrived after the head, however many that was.
    pub body: Vec<u8>,
    /// The length the answer declared, where it declared one.
    pub declared_length: Option<usize>,
}

impl Received {
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

/// Sends one request and reads the answer until the connection ends.
pub fn ask(server: &FakeServer, method: &str, path: &str, extra: &[(&str, &str)]) -> Received {
    read_back(&mut send(server, method, path, extra), None).expect("an answer on loopback")
}

/// Sends one request carrying a body and reads the answer.
pub fn ask_carrying(
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
pub fn send(server: &FakeServer, method: &str, path: &str, extra: &[(&str, &str)]) -> TcpStream {
    send_carrying(server, method, path, extra, b"")
}

/// Opens a connection and writes one request carrying `body` onto it.
pub fn send_carrying(
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
pub fn read_back(connection: &mut TcpStream, patience: Option<Duration>) -> Option<Received> {
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
