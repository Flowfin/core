//! DELIBERATE VIOLATION, PUSHED TO WATCH THE GATE GO RED, AND REMOVED IN THE
//! NEXT COMMIT ON THIS BRANCH.
//!
//! #20 requires that a test which opens a display, requests elevation, or binds
//! to a non-loopback address fails in the gate. A guard nobody watched fail is a
//! guard nobody knows the direction of, so this file is the three violations,
//! written the way somebody would write them if they did not know the rule.
//!
//! Every one of these passes on an ordinary machine with a display, a
//! passwordless `sudo` and a network, which is what makes them a proof rather
//! than a formality.

#[cfg(unix)]
#[test]
fn opens_a_display() {
    use std::os::unix::net::UnixStream;

    let socket = "/tmp/.X11-unix/X0";
    let _connection =
        UnixStream::connect(socket).expect("a display server to be listening on the X socket");
}

#[cfg(unix)]
#[test]
fn requests_elevation() {
    use std::process::Command;

    let status = Command::new("sudo")
        .args(["-n", "true"])
        .status()
        .expect("sudo to be runnable");
    assert!(status.success(), "elevation was refused");
}

#[test]
fn binds_to_the_machines_own_interface_address() {
    use std::net::{TcpListener, UdpSocket};

    // The ordinary way to learn a machine's own address without a dependency:
    // ask the routing table where a packet to somewhere else would leave from.
    let probe = UdpSocket::bind("0.0.0.0:0").expect("the wildcard address to be bindable");
    probe
        .connect("1.1.1.1:80")
        .expect("a non-loopback address to be reachable");
    let own = probe.local_addr().expect("this machine's own address");
    assert!(!own.ip().is_loopback(), "expected a non-loopback address");

    let _listener = TcpListener::bind((own.ip(), 0))
        .expect("the machine's own interface address to be bindable");
}
