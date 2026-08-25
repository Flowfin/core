//! The harness for the paths a fake cannot prove (#22).
//!
//! Some paths are not provable against a fake. Image decoding at real sizes on
//! real hardware, a genuine TLS handshake against a genuine certificate chain, a
//! real server's behaviour under a real sign-in. Pretending the headless suite
//! covers those is worse than admitting it does not, so they live here instead,
//! behind a name that says what they need.
//!
//! # Why this is a separate target rather than an ignored test
//!
//! `Cargo.toml` declares this target with `test = false`, so `cargo test
//! --locked` - the command `README.md` and `CONTRIBUTING.md` give a contributor,
//! and the one the `test` check runs - never invokes it. An `#[ignore]` attribute
//! would leave it inside that command's accounting, where a skipped case reads as
//! a case that was there, and the birth requirement in #20 is exactly that the
//! headless suite never becomes conditional on something a fake cannot supply.
//!
//! Running it is deliberate and explicit:
//!
//! ```text
//! cargo test --locked --test needs_a_real_server_or_real_hardware
//! ```
//!
//! # Why it refuses rather than skipping
//!
//! A skip is the failure mode this file exists against. A harness that finds no
//! server and prints "skipped" exits zero, and a gate reading that exit code
//! cannot tell it from a run that proved something. So an absent prerequisite is
//! a refusal, naming every missing one at once rather than the first, because a
//! person setting this up wants the whole list rather than one round trip per
//! variable.
//!
//! A run that carries no case is a refusal for the same reason. It is the state
//! this file is in today.
//!
//! `harness = false` is what makes both of those possible: the default harness
//! owns the process exit and prints its own accounting, and neither of the two
//! refusals above is a failing test case.

use std::process::ExitCode;

/// What this harness needs before it can prove anything, and what each one is
/// for.
///
/// The name is read from the environment rather than from a file in the tree. A
/// real server's address and the credentials to reach it are not the sort of
/// thing that belongs in a public repository, which is the same rule #109 is
/// about one register over.
const PREREQUISITES: &[(&str, &str)] = &[
    (
        "FLOWFIN_INTEGRATION_SERVER",
        "the base address of a real Jellyfin server to run against",
    ),
    (
        "FLOWFIN_INTEGRATION_USERNAME",
        "an account on that server the cases may sign in as",
    ),
    (
        "FLOWFIN_INTEGRATION_PASSWORD",
        "the password for that account",
    ),
];

/// The cases this harness runs once its prerequisites are present.
///
/// EMPTY TODAY, AND THAT IS A FACT ABOUT THE TREE RATHER THAN ABOUT THE HARNESS.
/// Every case this file is meant to carry - a genuine TLS handshake, a real
/// sign-in, decoding at real sizes - needs behaviour that does not exist yet, and
/// the first of it arrives with the transport in #27. A case added here before
/// then would be a test of nothing dressed as an integration test.
///
/// The empty list is refused rather than reported, so this cannot be read as a
/// harness that ran and found nothing.
const CASES: &[(&str, fn() -> Result<(), String>)] = &[];

fn main() -> ExitCode {
    let missing: Vec<&(&str, &str)> = PREREQUISITES
        .iter()
        .filter(|(name, _)| std::env::var_os(name).is_none())
        .collect();

    if !missing.is_empty() {
        eprintln!(
            "This harness needs a real server or real hardware and {} of its {} prerequisite(s) are absent:",
            missing.len(),
            PREREQUISITES.len()
        );
        for (name, what) in missing {
            eprintln!("  {name} is not set: {what}");
        }
        eprintln!(
            "Refusing rather than skipping. A skipped run exits zero and cannot be told from one that proved something."
        );
        return ExitCode::FAILURE;
    }

    if CASES.is_empty() {
        eprintln!(
            "Every prerequisite is present and this harness carries no case, so it proved nothing."
        );
        eprintln!(
            "Refusing rather than passing. A run of nothing that exits zero is the failure this harness is built against."
        );
        return ExitCode::FAILURE;
    }

    let mut failed = 0_usize;
    for (name, case) in CASES {
        match case() {
            Ok(()) => println!("ok    {name}"),
            Err(why) => {
                println!("FAIL  {name}: {why}");
                failed += 1;
            }
        }
    }

    println!("{} case(s) ran, {failed} failed.", CASES.len());
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
