//! Every field name this core declares is in the statement 0071 asks for.
//!
//! `docs/decisions/0071-what-may-leave-through-a-diagnostic-event.md` asks the
//! core for the rule as data: for each field name it has ever emitted, which of
//! the three treatments it applies. A client puts that statement verbatim into a
//! bundle so whoever is about to send it can read what is not in it.
//!
//! The one way that statement can be wrong in the direction that matters is by
//! being SHORT. A field name is a constant beside the event identity that carries
//! it, which is where 0100 puts it and where 0071 wants it, so a subsystem can
//! declare one and emit it while the gathered list does not move. The statement
//! is then incomplete and reads as complete, and a bundle carrying it tells
//! somebody a value is not in their events when it is.
//!
//! Nothing inside the crate can catch that: the list and the declarations are the
//! same kind of thing to the compiler, and a missing entry is a shorter array
//! rather than an error. So the subject here is the SOURCE, read as bytes, and
//! this is the one test in this repository whose subject is that.
//!
//! # Why this is not inside the crate
//!
//! `.github/invariants/rules` refuses `std::fs` under `src/`, grounded in 0003.
//! Reading the tree is exactly what this needs to do, so it lives out here where
//! that rule does not reach, and it reads the same tracked bytes the gate judges.
//!
//! # What this cannot see, said once so a green run is not read as more
//!
//! A name assembled rather than written. The three constructors take a
//! `&'static str` and every declaration in this tree passes a literal, so the
//! pattern below finds them; one built some other way is invisible here and would
//! also be a departure from what 0071 says a name is.
//!
//! A declaration inside a file's own test module. Everything after the first
//! LINE THAT IS EXACTLY `#[cfg(test)]` AT COLUMN ZERO is that file's tests, by
//! this repository's layout, and is skipped. Column zero rather than the string
//! anywhere is not a detail: three files in this tree mention the attribute
//! inside a doc comment, and a skip that took the mention would cut a file off
//! above its declarations and report a green run over source it never read. The
//! case below asserts no file carries two such lines, so the skip cannot quietly
//! widen.
//!
//! Whether a treatment is the RIGHT one for a name. That is 0068's judgement and
//! no reading of this tree makes it, which `src/diagnostics/redaction.rs` already
//! says of itself.

use flowfin_core::diagnostics::redaction::Treatment;
use flowfin_core::lifecycle::every_field_name_the_core_emits;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The three constructors, with the treatment each one gives.
const CONSTRUCTORS: &[(&str, Treatment)] = &[
    ("FieldName::carried_whole(\"", Treatment::CarriedWhole),
    ("FieldName::reduced(\"", Treatment::Reduced),
    ("FieldName::excluded(\"", Treatment::Excluded),
];

/// Where the crate's own source is, from the manifest rather than the working
/// directory, so the run does not depend on where it was started.
fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file under `src/`, in a stable order.
fn every_source_file(under: &Path, into: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(under)
        .expect("src/ is readable")
        .map(|entry| entry.expect("a readable entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            every_source_file(&path, into);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            into.push(path);
        }
    }
}

/// The line a file's own test module opens with, at column zero.
const OPENS_A_TEST_MODULE: &str = "\n#[cfg(test)]\n";

/// The part of a file that is not its own test module.
///
/// Everything from the line above on is that file's tests. A name declared there
/// is a fixture rather than something the core emits. The attribute mentioned
/// inside a doc comment is indented and is not this.
fn outside_the_test_module(source: &str) -> &str {
    match source.find(OPENS_A_TEST_MODULE) {
        Some(at) => &source[..at],
        None => source,
    }
}

/// Every field name declared in one piece of source, with its treatment.
fn names_declared_in(source: &str) -> Vec<(String, Treatment)> {
    let mut found = Vec::new();
    for (opening, treatment) in CONSTRUCTORS {
        let mut rest = source;
        while let Some(at) = rest.find(opening) {
            let after = &rest[at + opening.len()..];
            let Some(end) = after.find('"') else {
                break;
            };
            found.push((after[..end].to_owned(), *treatment));
            rest = &after[end..];
        }
    }
    found
}

/// Every field name the tree declares outside a test module, by name.
fn declared_in_the_tree() -> BTreeMap<String, Treatment> {
    let mut files = Vec::new();
    every_source_file(&source_root(), &mut files);
    assert!(
        files.len() > 20,
        "the walk found {} source file(s), which is fewer than this tree has, so it read the wrong place",
        files.len(),
    );

    let mut declared = BTreeMap::new();
    for file in files {
        let source = fs::read_to_string(&file).expect("a readable source file");
        for (name, treatment) in names_declared_in(outside_the_test_module(&source)) {
            if let Some(already) = declared.insert(name.clone(), treatment) {
                assert_eq!(
                    already, treatment,
                    "the field name {name} is declared twice with two different treatments, \
                     which is the pair 0071 says comes apart",
                );
            }
        }
    }
    declared
}

/// The statement, by name.
fn stated() -> BTreeMap<String, Treatment> {
    let mut by_name = BTreeMap::new();
    for field in every_field_name_the_core_emits() {
        assert!(
            by_name
                .insert(field.as_str().to_owned(), field.treatment())
                .is_none(),
            "the statement names {} twice",
            field.as_str(),
        );
    }
    by_name
}

/// The property 0071 rests on: the statement is not short.
///
/// A subsystem declaring a name and not registering it makes the statement
/// incomplete while it goes on reading as complete, and a bundle carrying it then
/// tells somebody a value is not in their events when it is.
#[test]
fn every_name_the_tree_declares_is_in_the_statement() {
    let declared = declared_in_the_tree();
    let stated = stated();

    let missing: Vec<&String> = declared
        .keys()
        .filter(|name| !stated.contains_key(*name))
        .collect();
    assert!(
        missing.is_empty(),
        "declared in the tree and absent from 0071's statement: {missing:?}",
    );
}

/// The other direction, which fails differently and matters less but is not
/// nothing: a statement naming a field nothing declares tells somebody the core
/// handles a value it never emits.
#[test]
fn every_name_in_the_statement_is_declared_somewhere() {
    let declared = declared_in_the_tree();
    let stated = stated();

    let dangling: Vec<&String> = stated
        .keys()
        .filter(|name| !declared.contains_key(*name))
        .collect();
    assert!(
        dangling.is_empty(),
        "in 0071's statement and declared nowhere: {dangling:?}",
    );
}

/// The treatment a name carries in the statement is the one its declaration
/// gave it. The pair coming apart is 0071's own reversal condition.
#[test]
fn the_treatment_a_name_carries_is_the_one_it_was_declared_with() {
    let declared = declared_in_the_tree();

    for field in every_field_name_the_core_emits() {
        let name = field.as_str();
        let was = declared
            .get(name)
            .unwrap_or_else(|| panic!("{name} is stated and declared nowhere"));
        assert_eq!(
            *was,
            field.treatment(),
            "{name} is declared as {was:?} and stated as {:?}",
            field.treatment(),
        );
    }
}

/// The skip this file rests on cannot quietly widen.
///
/// Everything after the first line that is exactly `#[cfg(test)]` at column zero
/// is skipped. That is right while a file has one test module at its end, which
/// is this repository's layout. A second block would put ordinary source behind
/// the skip, and the declarations in it would be invisible to every case above.
#[test]
fn no_source_file_carries_more_than_one_test_module() {
    let mut files = Vec::new();
    every_source_file(&source_root(), &mut files);

    for file in files {
        let source = fs::read_to_string(&file).expect("a readable source file");
        let blocks = source.matches(OPENS_A_TEST_MODULE).count();
        assert!(
            blocks <= 1,
            "{} carries {blocks} test modules, and this file skips from the first one on",
            file.display(),
        );
    }
}

/// What the statement says today, asserted as a whole rather than as a count.
///
/// A count would move with any name and say nothing about which. This is the set
/// a reader of a bundle would meet, and it is a negative disclosure as much as a
/// positive one: NO FIELD THIS BUILD EMITS IS EXCLUDED. 0071 puts the session
/// token and anything derived from it under that treatment, and nothing in this
/// tree emits one yet, so a bundle assembled today says the reduced field is the
/// only one it holds a correlator for.
#[test]
fn the_statement_is_the_set_this_build_actually_carries() {
    let stated = stated();

    let mut reduced: Vec<&str> = stated
        .iter()
        .filter(|(_, treatment)| **treatment == Treatment::Reduced)
        .map(|(name, _)| name.as_str())
        .collect();
    reduced.sort_unstable();
    assert_eq!(reduced, vec!["entry"]);

    let excluded: Vec<&str> = stated
        .iter()
        .filter(|(_, treatment)| **treatment == Treatment::Excluded)
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        excluded.is_empty(),
        "this build emits an excluded field, which it did not before: {excluded:?}",
    );

    let mut whole: Vec<&str> = stated
        .iter()
        .filter(|(_, treatment)| **treatment == Treatment::CarriedWhole)
        .map(|(name, _)| name.as_str())
        .collect();
    whole.sort_unstable();
    assert_eq!(
        whole,
        vec![
            "check",
            "consecutive-refusals",
            "entry-kind",
            "for-tier",
            "released-bytes",
            "released-entries",
            "suspended-for",
            "version-found",
        ],
    );
}
