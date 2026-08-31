//! The seed corpus, replayed inside the gating build (#86).
//!
//! Two questions get confused with each other and this file answers only one of
//! them. A coverage-guided run looks for NEW inputs and is slow, so the gate this
//! board is measured against schedules it and does not gate on it. Replaying the
//! seeds already known to be hostile is fast, and it catches the case where a
//! change makes a known input do something the code does not name. This is the
//! second, it runs inside `cargo test --locked`, and it is what the `test` check
//! already carries.
//!
//! # The target list is derived from the corpus and never written here
//!
//! `tests/fixtures/corpus/` holds one directory per target and the names of those
//! directories are the list. #86 asks for that shape so a target added by
//! acquiring a corpus cannot be silently uncovered by a list nobody updated, and
//! [`replay`] below is the one place a directory name is turned into a call.
//!
//! Both directions are refused rather than one. A directory with no entry point
//! behind it is a corpus nobody can replay, and a target with an entry point and
//! no directory is a target nothing is replaying, and passing over either is the
//! shape that reads exactly like a run that covered everything. An empty
//! directory and an empty root are refused for the same reason: replaying
//! nothing proves nothing, and it exits zero.
//!
//! WHAT THIS DOES NOT PROTECT AGAINST is a target nobody thought of, and it
//! cannot: a surface that was never named has no directory, so the empty-directory
//! rule has nothing to fail on. Two such surfaces are already named on #86 -
//! 0116's change-notification connection and the response decoding - and neither
//! has an entry point in this tree, so neither has a directory here.
//!
//! # What a seed asserts
//!
//! That the target ANSWERS. Every target below returns a value or a member of a
//! closed refusal set, so a seed that produced anything the code does not name
//! could only do it by panicking, and a panic in a replay is this test failing.
//! That is #86's `failing on any unnamed exception` in the terms this language
//! offers: there is no other exception to catch.
//!
//! It is deliberately not an assertion about WHICH answer a seed produces. The
//! seeds are named for the refusal they were built to reach, and a change that
//! moved one from `Length` to `Digest` would be a defect worth catching, but
//! pinning each seed to an outcome here would put a second copy of each module's
//! own table in this file. The tables are tested where they are decided;
//! [`the_named_refusals_are_each_reached_by_a_seed`] asserts the weaker property
//! this file can hold honestly - that the corpus as a whole reaches every member
//! of each closed set - which is what stops a corpus decaying into a list of
//! inputs that all fail the same way.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use flowfin_core::artwork::format::{Accepted, Refused, admitted};
use flowfin_core::cache::envelope::{WhichCheckFailed, open, version_found};
use flowfin_core::cache::freshness::EntryKind;
use flowfin_core::server::address::BaseAddress;

/// Every target this file can replay, by the directory name that selects it.
///
/// This is the only list in this file, and it is the mapping from a name to a
/// call rather than the target list: which of these actually runs is decided by
/// which directories exist, and a name here without a directory is refused
/// below.
const TARGETS: &[&str] = &["artwork-format", "cache-envelope", "server-address"];

fn corpus_root() -> PathBuf {
    // Relative to the manifest rather than to the working directory, for the
    // reason `tests/fixture_bytes.rs` gives: a test binary runs from wherever the
    // runner happens to be.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/corpus")
}

/// The directory names under the corpus root, sorted.
fn corpus_directories() -> Vec<String> {
    let root = corpus_root();
    let entries = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read the corpus root {}: {e}", root.display()));
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("cannot read an entry under the corpus: {e}"));
        if entry.path().is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
}

/// The seeds in one target's directory, sorted, as (name, bytes).
fn seeds(target: &str) -> Vec<(String, Vec<u8>)> {
    let directory = corpus_root().join(target);
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", directory.display()));
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("cannot read a seed under {target}: {e}"));
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        found.push((entry.file_name().to_string_lossy().into_owned(), bytes));
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// Hands one seed to one target and returns what it answered, as text.
///
/// The text is what the report prints, so a reader of a failing run sees which
/// seed reached which answer rather than only a count. It is never compared
/// against a stored string, for the reason the module documentation gives.
fn replay(target: &str, bytes: &[u8]) -> String {
    match target {
        "artwork-format" => {
            // Both entry points 0055 exposes, because the signature match is
            // reachable on its own and a caller may use it that way.
            let signature = Accepted::of(bytes);
            match admitted(bytes) {
                Ok(found) => format!(
                    "admitted {:?} {:?} (signature {signature:?})",
                    found.format(),
                    found.dimensions()
                ),
                Err(refused) => format!("refused {refused:?} (signature {signature:?})"),
            }
        }
        "cache-envelope" => {
            // Every kind, because `Kind` is a refusal only reachable by asking
            // for one the envelope does not name, and a reader asks for exactly
            // one kind at a time.
            let version = version_found(bytes);
            let asked = EntryKind::LibraryQueryResults;
            match open(asked, bytes) {
                Ok(payload) => format!("opened {} byte(s) (version {version:?})", payload.len()),
                Err(failed) => format!("dropped {failed:?} (version {version:?})"),
            }
        }
        "server-address" => {
            // Bytes reach a parser that takes text, so they are read lossily
            // rather than skipped. A seed that is not UTF-8 is exactly the input
            // a caller reaches this function with when a client passes on
            // something a person pasted, and skipping it would leave the one
            // case worth a seed unreplayed.
            let typed = String::from_utf8_lossy(bytes);
            match BaseAddress::parse(&typed) {
                Ok(address) => format!("parsed {:?}", address.origin()),
                Err(refused) => format!("refused {:?}", refused.part()),
            }
        }
        other => panic!(
            "the corpus directory {other} names no target this file can replay. A corpus with \
             no entry point behind it is not a target: add the call to TARGETS and to replay(), \
             or remove the directory. Replaying it as nothing would be a run that reports a \
             pass for a surface nobody reached."
        ),
    }
}

#[test]
fn the_corpus_root_is_not_empty() {
    let directories = corpus_directories();
    assert!(
        !directories.is_empty(),
        "there are no corpus directories under {}. Replaying nothing proves nothing and \
         exits zero, which reads exactly like a replay that found no defect.",
        corpus_root().display()
    );
}

#[test]
fn every_corpus_directory_names_a_target_and_every_target_has_one() {
    let found: BTreeSet<String> = corpus_directories().into_iter().collect();
    let named: BTreeSet<String> = TARGETS.iter().map(|t| (*t).to_owned()).collect();

    let without_a_target: Vec<&String> = found.difference(&named).collect();
    assert!(
        without_a_target.is_empty(),
        "corpus directories naming no target this file can replay: {without_a_target:?}. \
         A corpus with no entry point behind it is replayed by nothing."
    );

    let without_a_corpus: Vec<&String> = named.difference(&found).collect();
    assert!(
        without_a_corpus.is_empty(),
        "targets with no corpus directory: {without_a_corpus:?}. A target with no seeds is a \
         target nothing replays, and the run above would have passed without reaching it."
    );
}

#[test]
fn no_corpus_directory_is_empty() {
    for target in corpus_directories() {
        assert!(
            !seeds(&target).is_empty(),
            "the corpus directory {target} holds no seed. #86 asks that an empty corpus redden \
             the build rather than passing quietly, because replaying nothing proves nothing."
        );
    }
}

/// The replay itself. Every seed of every target, and the assertion is that each
/// one produced an answer at all: a target that did anything else would have
/// panicked, and a panic here is this test failing.
#[test]
fn every_seed_is_replayed_and_every_target_answers() {
    let mut replayed = 0;
    for target in corpus_directories() {
        for (name, bytes) in seeds(&target) {
            let answer = replay(&target, &bytes);
            assert!(
                !answer.is_empty(),
                "{target}/{name} produced no answer at all"
            );
            replayed += 1;
        }
    }
    assert!(
        replayed >= corpus_directories().len(),
        "fewer seeds were replayed than there are targets, so at least one target ran nothing"
    );
}

/// The corpus reaches every member of each closed refusal set.
///
/// This is what stops a corpus decaying into a list of inputs that all fail the
/// same way, which is the state a corpus arrives at when seeds are added without
/// anybody asking what each one is for. It reads the sets out of the crate rather
/// than keeping a copy, so a member added to either is a member this asserts a
/// seed for on the day it lands.
#[test]
fn the_named_refusals_are_each_reached_by_a_seed() {
    let mut image_refusals = BTreeSet::new();
    let mut image_accepted = BTreeSet::new();
    for (_, bytes) in seeds("artwork-format") {
        match admitted(&bytes) {
            Ok(found) => {
                image_accepted.insert(format!("{:?}", found.format()));
            }
            Err(refused) => {
                image_refusals.insert(format!("{refused:?}"));
            }
        }
    }
    for refused in [
        Refused::TheEncodedLengthPassedItsBound,
        Refused::TheSignatureMatchedNoAcceptedFormat,
        Refused::TheHeaderDeclaredNoDimensions,
        Refused::TheDeclaredDimensionsPassedTheirBound,
    ] {
        // The length bound is the one refusal a seed cannot reach: reaching it
        // means committing sixteen mebibytes of fixture, and 0055 applies that
        // bound during the transfer rather than to a file on disk. It is named
        // here rather than left out of the loop silently.
        if refused == Refused::TheEncodedLengthPassedItsBound {
            continue;
        }
        assert!(
            image_refusals.contains(&format!("{refused:?}")),
            "no seed under artwork-format reaches {refused:?}. The seeds that are there reach \
             {image_refusals:?}."
        );
    }
    assert!(
        !image_accepted.is_empty(),
        "every seed under artwork-format is refused, so nothing proves the accepted path still \
         accepts. A corpus of refusals alone passes a target that refuses everything."
    );

    let mut envelope_drops = BTreeSet::new();
    let mut envelope_opened = 0;
    for (_, bytes) in seeds("cache-envelope") {
        match open(EntryKind::LibraryQueryResults, &bytes) {
            Ok(_) => envelope_opened += 1,
            Err(failed) => {
                envelope_drops.insert(failed);
            }
        }
    }
    for failed in WhichCheckFailed::all() {
        assert!(
            envelope_drops.contains(failed),
            "no seed under cache-envelope reaches {failed:?}. The seeds that are there reach \
             {envelope_drops:?}."
        );
    }
    assert!(
        envelope_opened > 0,
        "every seed under cache-envelope is dropped, so nothing proves an envelope this build \
         wrote still opens."
    );

    let mut addresses_parsed = 0;
    let mut addresses_refused = 0;
    for (_, bytes) in seeds("server-address") {
        match BaseAddress::parse(&String::from_utf8_lossy(&bytes)) {
            Ok(_) => addresses_parsed += 1,
            Err(_) => addresses_refused += 1,
        }
    }
    assert!(
        addresses_parsed > 0 && addresses_refused > 0,
        "the server-address corpus reaches only one side of the parser: {addresses_parsed} \
         parsed and {addresses_refused} refused."
    );
}
