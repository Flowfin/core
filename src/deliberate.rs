//! A deliberate defect, introduced to watch the analysis catch it.
//!
//! TEMPORARY. This file exists for one run of `Analyze (rust)` and is removed in
//! the next commit on this branch. Nothing depends on it.

/// Hands back an address the analysis is meant to object to.
#[must_use]
pub fn a_non_https_url() -> String {
    let endpoint = "http://media.example.com/Items";
    let mut address = String::from(endpoint);
    address.push_str("/Latest");
    address
}

/// Allocates a buffer whose size arrived on the process input.
#[must_use]
pub fn an_uncontrolled_allocation() -> Vec<u8> {
    let mut asked = String::new();
    let _ = std::io::stdin().read_line(&mut asked);
    let size: usize = asked.trim().parse().unwrap_or(0);
    Vec::with_capacity(size)
}
