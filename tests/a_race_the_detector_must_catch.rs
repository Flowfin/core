//! The deliberate race the detector in #117 has to report.
//!
//! # Why this exists at all
//!
//! A detector that reports nothing because it was never actually enabled looks
//! exactly like a clean tree. The run in `.github/thread-detector/thread-detector.sh`
//! therefore does two things rather than one: it runs the suite and requires no
//! report, and it runs this target and requires a report. Without the second, a
//! green leg says only that the command exited zero.
//!
//! # Why this is a separate target rather than an ignored test
//!
//! `Cargo.toml` declares it with `test = false`, so `cargo test --locked` - the
//! command `README.md` and `CONTRIBUTING.md` give a contributor, and the one the
//! `test` check runs - never invokes it. That matters more here than for the
//! harness beside it: the body of this file is undefined behaviour on purpose,
//! and a contributor running the ordinary suite must not execute it. An
//! `#[ignore]` attribute would leave it inside that command's accounting and one
//! `--include-ignored` away from running.
//!
//! What that costs is that no ordinary check compiles this file: `--all-targets`
//! selects the test targets carrying `test = true`, and `Cargo.toml` carries the
//! run that measured it. What keeps this file from rotting is the leg that reads
//! it, `.github/workflows/thread-detector.yml`, which builds and runs it on every
//! pull request and refuses a run of it that reports no race.
//!
//! Running it is deliberate and explicit, and it is only meaningful under the
//! detector:
//!
//! ```text
//! cargo test --locked --test a_race_the_detector_must_catch
//! ```
//!
//! # The one place this repository writes a data race, and why it is written here
//!
//! `src/lib.rs` carries `#![forbid(unsafe_code)]` and this file is not inside it.
//! A data race cannot be written in safe Rust at all, which is the language
//! property the core relies on; proving that the detector is switched on
//! therefore needs the one thing the core is built not to contain, and the honest
//! place for it is a target the ordinary suite cannot reach, saying so at the top.
//!
//! `harness = false` so that this file owns its own exit. The detector's report
//! is what the run reads, and a test harness printing a pass beside a race report
//! is two verdicts where there should be one.

use std::process::ExitCode;
use std::thread;

/// The value two threads write and read without agreeing on an order.
///
/// A plain `static mut` rather than an atomic, because an atomic is exactly what
/// a detector does not report: the point of this file is an unsynchronised
/// access, and the ordinary repair for it is the thing under test.
static mut CONTESTED: u64 = 0;

/// Reads and writes the value with no synchronisation at all.
///
/// # Safety
///
/// This is unsound on purpose and is the subject of the run rather than a
/// mistake. It is called only from this target, which `Cargo.toml` keeps out of
/// `cargo test --locked`.
unsafe fn race_on_it(rounds: u64) {
    for _ in 0..rounds {
        // SAFETY: nothing here is safe, and that is the point. Two threads reach
        // this line at once with no ordering between them, which is the data race
        // the detector is meant to report.
        unsafe {
            let seen = *(&raw const CONTESTED);
            *(&raw mut CONTESTED) = seen.wrapping_add(1);
        }
    }
}

fn main() -> ExitCode {
    println!("-- what this target is");
    println!(
        "      a deliberate data race, run so that a detector has something to report. \
         A run of this that reports nothing is a detector that is not switched on."
    );

    let rounds = 100_000;
    let first = thread::spawn(move || {
        // SAFETY: see the function's own note. This target exists to be unsound.
        unsafe { race_on_it(rounds) };
    });
    let second = thread::spawn(move || {
        // SAFETY: see the function's own note.
        unsafe { race_on_it(rounds) };
    });

    let joined = first.join().is_ok() && second.join().is_ok();
    if !joined {
        println!("::error::a thread of this target did not join, so the race may not have run");
        return ExitCode::FAILURE;
    }

    // SAFETY: both threads have joined, so this read is the only one left.
    let total = unsafe { *(&raw const CONTESTED) };
    println!("-- what it ended with");
    println!("      {total}, which is a number nobody should rely on and is not read as a verdict");

    // Zero whatever the detector said. The run that judges this target reads the
    // detector's report rather than this exit code, and an exit code here that
    // depended on the race would make the verdict depend on a schedule.
    ExitCode::SUCCESS
}
