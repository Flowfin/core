//! Holding a session.
//!
//! 0003 puts acquiring a session, renewing it, holding more than one at a time,
//! and handing the secret to a store the client supplies inside the core. The
//! records are 0005, 0030, 0031, 0032, 0033, 0034, 0036 and 0114, and the issues
//! are #30 through #36 and #114.

/// One signed-in session against one server.
///
/// Thread safety, from 0009: safe from any thread. Calling on a session while
/// another thread signs it out is defined rather than racing: the call either
/// goes out under a valid token or fails with the signed-out outcome, and never
/// goes out under a token that has been discarded.
///
/// Signing out and holding several at once is #114.
#[derive(Debug)]
pub struct Session {
    _private: (),
}

/// The place a client keeps a session secret.
///
/// The core never chooses where a secret is kept. 0033 is the record and #33 is
/// the issue that decides what this asks of a client.
///
/// Thread safety, from 0009: called from the waiting lane only, and never
/// concurrently for one session, so a client may implement it without locking.
/// This is the deliberate opposite of [`crate::cache::ByteStore`], and the reason
/// is that a keychain call is rare and a platform keychain is the place a client
/// is most likely to write something naive.
///
/// The `Send + Sync` bound is here because the lane that calls it is not the
/// thread that supplied it. It is not a licence to call it concurrently, and
/// 0009's sentence above is the rule.
pub trait SecretStore: Send + Sync {}
