//! A deliberate defect, introduced to watch the analysis catch it.
//!
//! TEMPORARY. This file exists for one run of `Analyze (rust)` and is removed in
//! the next commit on this branch. Nothing depends on it.

/// Allocates a buffer whose size came from outside the process.
#[must_use]
pub fn an_uncontrolled_allocation() -> Vec<u8> {
    let asked: usize = std::env::args()
        .nth(1)
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    Vec::with_capacity(asked)
}

/// Hands back a credential written into the source.
#[must_use]
pub fn a_hard_coded_credential() -> &'static str {
    const PASSWORD: &str = "correct-horse-battery-staple";
    PASSWORD
}

/// Writes a value that arrived from outside into the process output.
pub fn a_cleartext_log() {
    let password = std::env::args().nth(1).unwrap_or_default();
    println!("signing in with {password}");
}
