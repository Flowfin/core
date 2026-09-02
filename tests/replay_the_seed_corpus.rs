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
//! # The rules answer, and what the gate asserts on is the answer
//!
//! THE RULES USED TO BE ASSERTIONS OVER ONE FIXED DIRECTORY, WHICH IS A GUARD
//! NOBODY HAS WATCHED FAIL. Each of them reads the corpus this tree carries,
//! that corpus is healthy, and a rule that has only ever run against a healthy
//! subject cannot be told apart from a rule that refuses nothing. Two of #86's
//! four conditions ask for the opposite: that an empty corpus REDDEN the build
//! and that a deliberately unhandled input in a seed REDDEN it, which are
//! statements about a run going red rather than about an assertion existing.
//!
//! So [`rules`] takes the root and the target names as arguments and ANSWERS
//! with the refusals rather than asserting them. The gate hands it this tree's
//! corpus and the names in `TARGETS` and requires the answer to be empty; each
//! proof below hands it a root built for one defect and requires the answer to
//! name exactly that rule and no other. The rules the gate runs and the rules
//! the proofs trip are one function rather than two, which is what stops a proof
//! passing against a second copy of the logic.
//!
//! [`replay_every_seed`] is parameterised the same way and for the same reason:
//! the target it calls is an argument, so a proof can hand it a target that does
//! not name what it did with a seed and watch the panic come out rather than be
//! counted as an answer.
//!
//! Every root a proof builds is under the build directory rather than in the
//! tree, so nothing a proof writes is tracked and no such root can be mistaken
//! for a corpus.
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

use flowfin_core::artwork::address::{ImageKind, ImageTag, ItemId, NotUsableInARequest};
use flowfin_core::artwork::format::{Accepted, Refused, admitted};
use flowfin_core::artwork::shape::{AspectRatio, RatioNotUsable, WhatShapeIsKnown};
use flowfin_core::cache::envelope::{WhichCheckFailed, open, version_found};
use flowfin_core::cache::freshness::EntryKind;
use flowfin_core::server::address::BaseAddress;

/// Every target this file can replay, by the directory name that selects it.
///
/// This is the only list in this file, and it is the mapping from a name to a
/// call rather than the target list: which of these actually runs is decided by
/// which directories exist, and a name here without a directory is refused
/// below.
const TARGETS: &[&str] = &[
    "artwork-format",
    "artwork-identifier",
    "artwork-shape",
    "cache-envelope",
    "server-address",
];

fn corpus_root() -> PathBuf {
    // Relative to the manifest rather than to the working directory, for the
    // reason `tests/fixture_bytes.rs` gives: a test binary runs from wherever the
    // runner happens to be.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/corpus")
}

/// The directory names under one corpus root, sorted.
fn corpus_directories(root: &Path) -> Vec<String> {
    let entries = std::fs::read_dir(root)
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

/// The seeds in one target's directory under one root, sorted, as (name, bytes).
fn seeds(root: &Path, target: &str) -> Vec<(String, Vec<u8>)> {
    let directory = root.join(target);
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

/// One reason a corpus root cannot be replayed as it stands.
///
/// The rule is an identifier rather than a sentence, so a proof can require
/// exactly one of them and no other in a way a message could not. The sentence a
/// reader of a failing run needs is built beside the assertion that carries it.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Refusal {
    rule: &'static str,
    subject: String,
}

impl Refusal {
    fn new(rule: &'static str, subject: &str) -> Self {
        Self {
            rule,
            subject: subject.to_owned(),
        }
    }
}

/// Every rule over one corpus root, answering rather than asserting.
///
/// `named` is the set of targets [`replay`] can call, passed in rather than read
/// from the constant above, so a proof can build a root of its own and still run
/// this function rather than a copy of it.
fn rules(root: &Path, named: &[&str]) -> Vec<Refusal> {
    let mut refusals = Vec::new();
    let directories = corpus_directories(root);

    // Replaying nothing proves nothing and exits zero, which reads exactly like
    // a replay that found no defect.
    if directories.is_empty() {
        refusals.push(Refusal::new(
            "the-root-holds-no-directory",
            "the corpus root",
        ));
    }

    let found: BTreeSet<&str> = directories.iter().map(String::as_str).collect();
    let declared: BTreeSet<&str> = named.iter().copied().collect();

    // An empty corpus reddens the build rather than passing quietly, which is
    // #86's third condition.
    for directory in &directories {
        if seeds(root, directory).is_empty() {
            refusals.push(Refusal::new("a-directory-holds-no-seed", directory));
        }
    }

    // A corpus with no entry point behind it is replayed by nothing.
    for directory in found.difference(&declared) {
        refusals.push(Refusal::new("a-directory-names-no-target", directory));
    }

    // A target with no seeds is a target nothing replays, and the run above it
    // would have passed without ever reaching it.
    for target in declared.difference(&found) {
        refusals.push(Refusal::new("a-target-has-no-directory", target));
    }

    refusals.sort();
    refusals
}

/// The rule identifiers in one answer, sorted and deduplicated.
///
/// A proof compares this against the single rule its root was built to trip, so
/// a root that trips a second rule fails the proof rather than passing it.
fn rule_ids(refusals: &[Refusal]) -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = refusals.iter().map(|refusal| refusal.rule).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
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
        "artwork-identifier" => {
            // Both values 0049 refuses on their bytes before either is written
            // into a request, and both doors rather than one: they call one
            // function today, and replaying only the identifier is what would
            // pass on the day the tag stops sharing it. Bytes reach a parser
            // that takes text, so they are read lossily rather than skipped, for
            // the reason the server-address arm gives.
            //
            // THE LENGTH AND NEVER THE VALUE, which is the one decision in this
            // arm. 0068 places a server-supplied identifier in the personal set
            // and `NotUsableInARequest` carries the position rather than the
            // byte for that reason; an answer built here out of the admitted
            // value would put back into a report exactly what that type is
            // shaped to keep out of one.
            let sent = String::from_utf8_lossy(bytes);
            let identifier = match ItemId::from_server(&sent) {
                Ok(id) => format!("item of {} byte(s)", id.as_str().len()),
                Err(why) => format!("refused {why:?}"),
            };
            match ImageTag::from_server(&sent) {
                Ok(tag) => format!("tag of {} byte(s) ({identifier})", tag.as_str().len()),
                Err(why) => format!("refused {why:?} ({identifier})"),
            }
        }
        "artwork-shape" => {
            // Both doors 0052 opens, because the ratio is readable on its own
            // and a caller walking an item's five kinds reaches the other one.
            // Bytes reach a parser that takes text, so they are read lossily
            // rather than skipped, for the reason the server-address arm gives.
            let typed = String::from_utf8_lossy(bytes);
            let read = match AspectRatio::from_server(&typed) {
                Ok(ratio) => format!("read {}", ratio.ten_thousandths()),
                Err(why) => format!("refused {why:?}"),
            };
            match WhatShapeIsKnown::of_kind(ImageKind::Primary, Some(&typed)) {
                WhatShapeIsKnown::Stated(ratio) => {
                    format!("stated {} ({read})", ratio.ten_thousandths())
                }
                WhatShapeIsKnown::NothingStated => format!("nothing stated ({read})"),
                WhatShapeIsKnown::ARatioThatCannotBeUsed(why) => {
                    format!("unusable {why:?} ({read})")
                }
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

/// Every seed of every directory under one root, through one target function.
///
/// Returns how many seeds were replayed. What it asserts per seed is that the
/// target answered at all: anything the code does not name could only leave
/// through a panic, and a panic is deliberately not caught here. Catching one
/// and counting it would turn #86's fourth condition into a number in a report,
/// which is the one-line change this arrangement exists against.
fn replay_every_seed(root: &Path, call: fn(&str, &[u8]) -> String) -> usize {
    let mut replayed = 0;
    for target in corpus_directories(root) {
        for (name, bytes) in seeds(root, &target) {
            let answer = call(&target, &bytes);
            assert!(
                !answer.is_empty(),
                "{target}/{name} produced no answer at all"
            );
            replayed += 1;
        }
    }
    replayed
}

// --------------------------------------------------------------------------
// The gate: the rules and the replay, over the corpus this tree carries.
// --------------------------------------------------------------------------

#[test]
fn the_corpus_this_tree_carries_breaks_no_rule() {
    let root = corpus_root();
    let refusals = rules(&root, TARGETS);
    assert!(
        refusals.is_empty(),
        "the corpus under {} breaks {} rule(s): {refusals:?}. Each one is a state in which a \
         replay exits zero having covered less than the run reads as having covered.",
        root.display(),
        refusals.len()
    );
}

#[test]
fn every_seed_is_replayed_and_every_target_answers() {
    let root = corpus_root();
    let replayed = replay_every_seed(&root, replay);
    assert!(
        replayed >= corpus_directories(&root).len(),
        "fewer seeds were replayed than there are targets, so at least one target ran nothing"
    );
}

/// What a refused identifier or tag is, by name, for every member of that set.
///
/// The blocks below compare a member against the debug form of the value a seed
/// produced. `NotUsableInARequest::ByteAt` carries the position the refusal
/// stopped at, so its debug form differs per seed and no fixed string could ever
/// be found in a set of them. This is a total function over the type instead: a
/// member added tomorrow is a compile error here rather than a member nothing
/// asserts a seed for, which is the property the other blocks get from reading
/// their sets out of the crate.
fn what_a_refused_identifier_is(why: NotUsableInARequest) -> &'static str {
    match why {
        NotUsableInARequest::Empty => "nothing was there",
        NotUsableInARequest::ByteAt(_) => "a byte outside the admitted set",
    }
}

/// The identifier corpus reaches both members of that set and admits something.
///
/// A block of its own rather than twenty more lines inside the test below.
/// The analyser refuses a function past a hundred lines, and the repair that
/// keeps one test asserting one property over every closed set is a named
/// block rather than a waiver written beside the lint.
fn the_identifier_corpus_reaches_every_refusal(root: &Path) {
    let mut identifier_refusals = BTreeSet::new();
    let mut identifiers_admitted = 0;
    for (_, bytes) in seeds(root, "artwork-identifier") {
        match ItemId::from_server(&String::from_utf8_lossy(&bytes)) {
            Ok(_) => identifiers_admitted += 1,
            Err(why) => {
                identifier_refusals.insert(what_a_refused_identifier_is(why));
            }
        }
    }
    for why in [NotUsableInARequest::Empty, NotUsableInARequest::ByteAt(0)] {
        // Every member is reachable by a seed here, so nothing is skipped and
        // there is no `continue` above this assertion to read past.
        let named = what_a_refused_identifier_is(why);
        assert!(
            identifier_refusals.contains(named),
            "no seed under artwork-identifier reaches {named:?}. The seeds that are there reach \
             {identifier_refusals:?}."
        );
    }
    assert!(
        identifiers_admitted > 0,
        "every seed under artwork-identifier is refused, so nothing proves an identifier a \
         server sent is still admitted. A corpus of refusals alone passes a parser that \
         refuses everything."
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
    let root = corpus_root();
    let mut image_refusals = BTreeSet::new();
    let mut image_accepted = BTreeSet::new();
    for (_, bytes) in seeds(&root, "artwork-format") {
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

    the_identifier_corpus_reaches_every_refusal(&root);

    let mut shape_refusals = BTreeSet::new();
    let mut shape_read = 0;
    for (_, bytes) in seeds(&root, "artwork-shape") {
        match AspectRatio::from_server(&String::from_utf8_lossy(&bytes)) {
            Ok(_) => shape_read += 1,
            Err(why) => {
                shape_refusals.insert(format!("{why:?}"));
            }
        }
    }
    for why in [
        RatioNotUsable::NotADecimalNumber,
        RatioNotUsable::NarrowerThanAnyBoxTheLadderBuilds,
        RatioNotUsable::WiderThanAnyBoxTheLadderBuilds,
        RatioNotUsable::StatedForAKindNoSupportedLineStatesOneFor(ImageKind::Backdrop),
    ] {
        // One member of that set is not a property of the bytes and no seed can
        // reach it: it says a ratio was offered for a kind neither supported
        // line states one for, which is decided by the kind a caller passed and
        // not by what the value says. It is named here rather than left out of
        // the loop silently, on the shape the length bound above already takes.
        if matches!(
            why,
            RatioNotUsable::StatedForAKindNoSupportedLineStatesOneFor(_)
        ) {
            continue;
        }
        assert!(
            shape_refusals.contains(&format!("{why:?}")),
            "no seed under artwork-shape reaches {why:?}. The seeds that are there reach              {shape_refusals:?}."
        );
    }
    assert!(
        shape_read > 0,
        "every seed under artwork-shape is refused, so nothing proves a ratio a server states          is still read. A corpus of refusals alone passes a parser that refuses everything."
    );

    let mut envelope_drops = BTreeSet::new();
    let mut envelope_opened = 0;
    for (_, bytes) in seeds(&root, "cache-envelope") {
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
    for (_, bytes) in seeds(&root, "server-address") {
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

// --------------------------------------------------------------------------
// The proofs. Each builds a root outside the tree, hands it to the same
// function the gate above runs, and requires the rule it was built to trip.
// --------------------------------------------------------------------------

/// A root under the build directory, emptied first so a run is never read
/// against what the run before it left behind.
fn a_root_for(case: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("corpus-rules")
        .join(case);
    if root.exists() {
        std::fs::remove_dir_all(&root)
            .unwrap_or_else(|e| panic!("cannot clear {}: {e}", root.display()));
    }
    std::fs::create_dir_all(&root)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", root.display()));
    root
}

fn a_directory(root: &Path, target: &str) {
    let directory = root.join(target);
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", directory.display()));
}

fn a_seed(root: &Path, target: &str, name: &str, bytes: &[u8]) {
    a_directory(root, target);
    let path = root.join(target).join(name);
    std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// The healthy root every one-change neighbour below is a change to.
///
/// Two directories rather than one, each holding one seed, and both named. With
/// a single directory a rule that fires on the LAST directory it reads would be
/// indistinguishable from one that fires on the only directory there is.
fn a_healthy_root(case: &str) -> PathBuf {
    let root = a_root_for(case);
    a_seed(&root, "first-target", "a-seed", b"one");
    a_seed(&root, "second-target", "a-seed", b"two");
    root
}

const HEALTHY: &[&str] = &["first-target", "second-target"];

#[test]
fn the_healthy_root_breaks_no_rule() {
    let root = a_healthy_root("the-healthy-root");
    assert_eq!(
        rules(&root, HEALTHY),
        Vec::new(),
        "the root each proof below is a one-change neighbour of already breaks a rule, so \
         nothing those proofs report could be attributed to their own change."
    );
}

#[test]
fn a_root_with_no_directory_is_refused() {
    // The one proof that is not a neighbour of the healthy root. A root holding
    // no directory names no target either, so the target list goes with the
    // directories: both halves are the same absence rather than two changes.
    let root = a_root_for("a-root-with-no-directory");
    let refusals = rules(&root, &[]);
    assert_eq!(
        rule_ids(&refusals),
        vec!["the-root-holds-no-directory"],
        "an empty corpus root is what a replay of nothing looks like from the inside, and it \
         exits zero. Refusals were {refusals:?}."
    );
}

#[test]
fn a_directory_with_no_seed_is_refused() {
    let root = a_healthy_root("a-directory-with-no-seed");
    // The one change: the second target keeps its directory and loses its seed.
    std::fs::remove_file(root.join("second-target").join("a-seed")).expect("the seed is there");
    let refusals = rules(&root, HEALTHY);
    assert_eq!(
        rule_ids(&refusals),
        vec!["a-directory-holds-no-seed"],
        "this is #86's `an empty corpus reddens the build`. Refusals were {refusals:?}."
    );
    assert_eq!(
        refusals[0].subject, "second-target",
        "the refusal names the directory rather than only a count, so a corpus that lost one \
         seed is repaired without reading every directory in it."
    );
}

#[test]
fn a_directory_naming_no_target_is_refused() {
    let root = a_healthy_root("a-directory-naming-no-target");
    // The one change: a third directory, with a seed and nothing behind it.
    a_seed(&root, "a-surface-nobody-can-call", "a-seed", b"three");
    let refusals = rules(&root, HEALTHY);
    assert_eq!(
        rule_ids(&refusals),
        vec!["a-directory-names-no-target"],
        "a corpus with no entry point behind it is replayed by nothing, and the run that walks \
         past it exits zero. Refusals were {refusals:?}."
    );
    assert_eq!(refusals[0].subject, "a-surface-nobody-can-call");
}

#[test]
fn a_target_with_no_directory_is_refused() {
    let root = a_healthy_root("a-target-with-no-directory");
    // The one change: a third target is named and acquires no corpus.
    let named = &["first-target", "second-target", "a-target-with-no-seeds"];
    let refusals = rules(&root, named);
    assert_eq!(
        rule_ids(&refusals),
        vec!["a-target-has-no-directory"],
        "a target nothing replays is the half of #86's derivation that a directory listing \
         cannot see on its own. Refusals were {refusals:?}."
    );
    assert_eq!(refusals[0].subject, "a-target-with-no-seeds");
}

/// A target that does not name what it did with one of its seeds.
///
/// It exists only to be handed to [`replay_every_seed`], and the input it fails
/// on is a literal rather than a shape, so the two proofs below differ by that
/// seed and by nothing else.
fn a_target_that_does_not_handle_one_seed(_target: &str, bytes: &[u8]) -> String {
    assert!(
        bytes != b"the input this target does not name",
        "this target does not name what it did with the seed it was handed"
    );
    format!("answered {} byte(s)", bytes.len())
}

#[test]
#[should_panic(expected = "this target does not name what it did with the seed it was handed")]
fn a_seed_the_target_does_not_handle_reddens_the_replay() {
    let root = a_root_for("a-seed-the-target-does-not-handle");
    // Seeds are read in sorted order, so the ordinary one is replayed first and
    // the failure cannot be read as a loop that never started.
    a_seed(&root, "first-target", "a-handled-seed", b"one");
    a_seed(
        &root,
        "first-target",
        "b-unhandled-seed",
        b"the input this target does not name",
    );
    replay_every_seed(&root, a_target_that_does_not_handle_one_seed);
}

#[test]
fn the_same_root_without_that_seed_replays_green() {
    // The one-change neighbour of the proof above. Without it that proof shows a
    // target failing at everything, which says nothing about the seed.
    let root = a_root_for("without-the-unhandled-seed");
    a_seed(&root, "first-target", "a-handled-seed", b"one");
    assert_eq!(
        replay_every_seed(&root, a_target_that_does_not_handle_one_seed),
        1,
        "the target fails on one literal and answers everything else, so a green run here is \
         what makes the red run beside it about the seed rather than about the target."
    );
}

/// A target that returns, and returns nothing.
///
/// The other way a target can fail to answer. A count of seeds replayed is
/// identical either way, which is why the count is not what the replay asserts
/// on.
fn a_target_that_answers_nothing(_target: &str, _bytes: &[u8]) -> String {
    String::new()
}

#[test]
#[should_panic(expected = "produced no answer at all")]
fn a_target_that_answers_nothing_reddens_the_replay() {
    let root = a_root_for("a-target-that-answers-nothing");
    a_seed(&root, "first-target", "a-seed", b"one");
    replay_every_seed(&root, a_target_that_answers_nothing);
}
