//! The bytes of a fixture survive the checkout (#99).
//!
//! A fixture exists to prove an exact sequence of bytes. If the checkout pipeline
//! is free to normalise line endings, the byte a fixture was written to prove is
//! deleted on the way into the tree or on the way out of it, and the test resting
//! on it goes on passing against a file that no longer has it. That is a guard
//! that cannot fail, which is worse than no guard, because somebody is relying on
//! it.
//!
//! This file is what makes the rule in `.gitattributes` fail loudly instead. Take
//! the `tests/fixtures/** -text` line out, let a clone with line-ending
//! translation switched on rewrite the fixture, and the first assertion below
//! goes red.
//!
//! Both directions are checked. A rule that added a carriage return to every line
//! would be as wrong as one that removed it, and a test asserting only the
//! presence of one would not see it.
//!
//! The file is read as bytes rather than as text on purpose. Reading it as a
//! string and comparing lines is the shape that cannot see the defect, because
//! every convenience for reading lines treats the two endings as the same thing.

use std::path::Path;

/// The fixture, read exactly as it sits on disk.
fn fixture() -> Vec<u8> {
    // Relative to the manifest rather than to the working directory: a test binary
    // is run from wherever the runner happens to be, and a relative path from
    // there resolves differently on a developer machine and on a gate.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/line-endings.txt");
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn the_carriage_return_survived_the_checkout() {
    let bytes = fixture();
    assert!(
        bytes.windows(2).any(|w| w == b"\r\n"),
        "the fixture holds no CRLF. The checkout pipeline rewrote the byte this \
         fixture exists to prove, which means the rule for tests/fixtures/ in \
         .gitattributes is missing or was not in force when this clone was made."
    );
}

#[test]
fn the_bare_line_feed_survived_the_checkout() {
    let bytes = fixture();
    let bare = bytes
        .iter()
        .enumerate()
        .any(|(i, b)| *b == b'\n' && (i == 0 || bytes[i - 1] != b'\r'));
    assert!(
        bare,
        "every line feed in the fixture is preceded by a carriage return. \
         Something added the byte rather than removing it, which is the other \
         direction of the same defect."
    );
}

#[test]
fn the_fixture_is_the_length_it_was_written_at() {
    // The count is the whole point of the two tests above stated once more as a
    // number: a translation in either direction changes it, including one that
    // happens to leave both shapes present.
    assert_eq!(
        fixture().len(),
        260,
        "the fixture is not the length it was committed at, so some byte in it \
         has been added or removed since."
    );
}
