#!/usr/bin/env bash
# The suite runs under a detector, and a broken concurrency claim reddens the run
# (#117).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHAT THIS IS FOR. `docs/decisions/0009-the-concurrency-model.md` promises which
# calls may block, what a returned cancellation guarantees, and what is safe to
# touch from two threads at once. `tests/thread_statements.rs` proves the last of
# those against the type system, which is a claim about a shape rather than about
# a schedule. A data race appears on a machine with a different number of cores
# under an interleaving nobody wrote, and passes ten thousand runs on the machine
# it was written on. This leg is where a schedule is watched rather than reasoned
# about.
#
# THE RUN HAS TWO LEGS AND NEITHER IS OPTIONAL. A detector that was never
# actually switched on reports nothing, exits zero, and prints a page that reads
# exactly like a clean tree. So this runs the suite and requires no report, and
# then runs `tests/a_race_the_detector_must_catch.rs` and requires a report. The
# second is the only thing here that can tell a clean tree from a detector that
# is not running.
#
# THE VERDICT IS THE REPORT AND NEVER THE EXIT CODE. The detector's own exit
# behaviour is a runtime option, a target that reports a race can still exit
# zero, and the deliberate race below is expected to report while the run around
# it carries on, so an exit code cannot carry both legs. Every verdict here is
# read out of what the detector printed.
#
# WHERE IT RUNS, AND WHAT THAT LEAVES UNCOVERED. The detector is a compiler flag
# on a second toolchain rather than something the pinned one offers, and it does
# not reach every target this core is meant to be hosted on:
# `docs/decisions/0011-the-language-the-toolchain-and-the-binding-layer.md`
# carries the readings. So this leg names its own toolchain and its own target,
# both below, and a race that manifests only on Windows or on Android is outside
# what any run here can see. That is a bound this leg states on every run rather
# than an absence it hides.
#
# A finding is fixed or it is written down. `.github/thread-detector/suppressions`
# is where a finding that is not going to be fixed lives, one per line with the
# reason on the same line, and an entry carrying no reason is itself refused
# before anything is judged. Every run prints that register beside its verdict,
# so a suppression is read at the moment somebody reads a verdict rather than
# found later by somebody auditing. An entry is a debt and each reason says what
# retires it.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      run both legs under the detector and judge what each one printed
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# The toolchain this leg runs on, named here and nowhere else.
#
# `rust-toolchain.toml` is the authority for the compiler every other check uses
# and it says in its own comments why this channel is not written there: a second
# channel in that file would install a nightly on every contributor's machine for
# a leg none of their commands run.
#
# IT IS A CHANNEL RATHER THAN A DATE, and that is a cost rather than an
# oversight. The compiler under this leg moves without a commit, so this leg can
# redden on a compiler change rather than on a tree change. What is done about it
# instead is that the run prints the exact build that judged it, so a verdict is
# read against a version rather than against a promise about one. A dated pin
# retires this the day somebody is named to move it, which is a person rather
# than a line of shell.
DETECTOR_CHANNEL="nightly"

# The target the detector is available on. A target triple whose specification
# does not list this sanitizer produces a run that reports nothing and reads
# exactly like a clean tree, which is the failure the second leg exists to catch.
DETECTOR_TARGET="x86_64-unknown-linux-gnu"

# The flag that switches the detector on, and the reason the standard library is
# rebuilt with it. A standard library compiled without the instrumentation is a
# body of synchronisation the detector cannot see, and what it reports then is
# that library's own locking rather than this tree's defect.
DETECTOR_RUSTFLAGS="-Zsanitizer=thread"
DETECTOR_STD="-Zbuild-std"

# The two commands, written out so the log carries them rather than a description
# of them.
#
# `--all-targets` ON THE SUITE LEG IS WHAT LEAVES THE DOCUMENTATION TESTS OUT, and
# it is there because they cannot be run under this flag rather than because they
# were not wanted. Without it the run reached them and the compiler refused, by
# name, on the first attempt at this leg:
#
#     error: mixing `-Zsanitizer` will cause an ABI mismatch in crate `flowfin_core`
#       = note: `-Zsanitizer` is unset in this crate which is incompatible with
#               `-Zsanitizer=thread` in dependency `std`
#
# The documentation tests are compiled by a second driver that is not handed this
# flag, so they link a crate built without the instrumentation against a standard
# library built with it. The escape hatch the compiler offers for that mismatch
# says in its own name that it is unsafe, and taking it here would make every
# verdict on this leg rest on an ABI the compiler has just said does not match.
#
# What that costs is measured rather than assumed. This crate carries no
# documentation test:
#
#     cargo test --locked --doc
#        Doc-tests flowfin_core
#        running 0 tests
#
# So nothing is excluded from this leg today, and the day one is written it is
# outside the detector rather than inside it. The run prints that bound.
SUITE_COMMAND="cargo +${DETECTOR_CHANNEL} test --locked --all-targets ${DETECTOR_STD} --target ${DETECTOR_TARGET}"
RACE_COMMAND="cargo +${DETECTOR_CHANNEL} test --locked ${DETECTOR_STD} --target ${DETECTOR_TARGET} --test a_race_the_detector_must_catch"

# The register of findings this run does not refuse, beside this script rather
# than inside it, so that a person suppressing one edits a register instead of a
# script.
SUPPRESSIONS_FILE="$(dirname "$0")/suppressions"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line.
#
# awk rather than grep: grep exits 1 when it selects nothing, and "the detector
# reported nothing" is the ordinary answer on one leg and a refusal on the other,
# so a pipeline that tells those apart one `set -o pipefail` at a time is how a
# gate ends up passing on everything.
# --------------------------------------------------------------------------

# Every report the detector printed, one kind per line.
#
# The subject is the detector's own announcement line and nothing else. Its
# closing summary line names the same finding a second time, so counting both
# would report one race as two, and a run that mentions the tool's name in prose
# - this script's own log lines, a test whose name carries it - is not a finding.
detector_reports() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      if (line !~ /^WARNING: ThreadSanitizer: /) next
      kind = line
      sub(/^WARNING: ThreadSanitizer: /, "", kind)
      sub(/ *\(pid=.*$/, "", kind)
      sub(/[ \t]+$/, "", kind)
      if (kind == "") kind = "(unnamed finding)"
      print kind
    }
  '
}

# What the run says it collected, as BINARIES<TAB>EXECUTED.
#
# THIS NUMBER HAS A SECOND READER IN THIS TREE AND THE SECOND ONE IS THIS.
# `.github/test/test.sh` refuses a run of nothing for `cargo test --locked` on
# the pinned compiler. That verdict says nothing about this leg: the command is a
# different one, the compiler is a different channel, the target triple is
# different, and the set of binaries a filter or a target-selection mistake would
# leave out is therefore a different set. A suite that collected nothing under
# this detector reports nothing, and this leg would read that as a clean run,
# which is the one shape this whole check exists to refuse.
count_run() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^running [0-9]+ test/) {
        n = line
        sub(/^running /, "", n)
        sub(/ test.*$/, "", n)
        binaries = binaries + 1
        executed = executed + n
      }
    }
    END {
      printf "%d\t%d\n", binaries, executed
    }
  '
}

# The suppression register, as VERDICT<TAB>LINE<TAB>ENTRY<TAB>REASON.
#
# An entry carrying no reason is REFUSE, because a bare suppression is a finding
# somebody turned off and nobody argued with. Comments and blank lines are not
# entries.
parse_suppressions() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == "" || line ~ /^#/) next
      entry = line
      reason = ""
      i = index(line, " ")
      j = index(line, "\t")
      if (j > 0 && (i == 0 || j < i)) i = j
      if (i > 0) {
        entry = substr(line, 1, i - 1)
        reason = substr(line, i + 1)
        sub(/^[ \t]+/, "", reason)
      }
      verdict = (reason == "") ? "REFUSE" : "ALLOW"
      printf "%s\t%d\t%s\t%s\n", verdict, NR, entry, reason
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than a real run. A fixture that judged
# a run made in this tree would prove the state of the tree on the day it ran,
# not the rule.
# --------------------------------------------------------------------------

selftest_failures=0

assert_out() {
  local what="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    printf 'ok    %s\n' "$what"
  else
    printf 'FAIL  %s\n        expected: %s\n        actual:   %s\n' \
      "$what" "$(printf '%s' "$expected" | tr '\n' '|')" "$(printf '%s' "$actual" | tr '\n' '|')"
    selftest_failures=$((selftest_failures + 1))
  fi
}

judge_reports()  { printf '%s' "$1" | detector_reports; }
judge_run()      { printf '%s' "$1" | count_run; }
judge_register() { printf '%s' "$1" | parse_suppressions; }

selftest() {
  echo "== a report is read, and read once =="
  assert_out "reads: a data race, named by its own kind" \
    "data race" \
    "$(judge_reports '==================
WARNING: ThreadSanitizer: data race (pid=2411)
  Write of size 8 at 0x5591 by thread T2:
SUMMARY: ThreadSanitizer: data race tests/a_race_the_detector_must_catch.rs:61 in race_on_it
==================
')"
  assert_out "reads: two findings as two, so a second cannot hide behind the first" \
    "$(printf 'data race\nlock-order-inversion (potential deadlock)')" \
    "$(judge_reports 'WARNING: ThreadSanitizer: data race (pid=2411)
SUMMARY: ThreadSanitizer: data race foo.rs:1 in bar
WARNING: ThreadSanitizer: lock-order-inversion (potential deadlock) (pid=2411)
SUMMARY: ThreadSanitizer: lock-order-inversion foo.rs:2 in baz
')"
  assert_out "reads: a finding of a kind nobody wrote a case for, rather than passing it over" \
    "signal-unsafe call inside of a signal" \
    "$(judge_reports 'WARNING: ThreadSanitizer: signal-unsafe call inside of a signal (pid=7)
')"

  echo "== what is not a report =="
  assert_out "passes over: a clean run of the suite" \
    "" \
    "$(judge_reports 'running 10 tests
test the_core_handle_is_safe_from_any_thread ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
')"
  assert_out "passes over: this script naming the tool in its own log line" \
    "" \
    "$(judge_reports '-- what this leg runs
      the suite under ThreadSanitizer, and then a deliberate race under it
')"
  assert_out "passes over: the closing summary alone, which names a finding already counted" \
    "" \
    "$(judge_reports 'SUMMARY: ThreadSanitizer: data race foo.rs:1 in bar
')"
  assert_out "passes over: a test whose own name carries the tool name" \
    "" \
    "$(judge_reports 'test a_thread_sanitizer_report_is_not_a_test_name ... ok
')"

  echo "== the run says what it collected =="
  assert_out "reads: several binaries, and the tests summed across them" \
    "$(printf '3\t10')" \
    "$(judge_run 'running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"
  assert_out "reads: one test, written in the singular by the harness" \
    "$(printf '1\t1')" \
    "$(judge_run 'running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"
  assert_out "reads: a run that collected nothing, which reports nothing and reads as clean" \
    "$(printf '1\t0')" \
    "$(judge_run 'running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
')"
  assert_out "reads: a harness that printed no accounting at all" \
    "$(printf '0\t0')" \
    "$(judge_run 'error: no test target named nothing
')"
  assert_out "passes over: the same sentence inside a compiler message" \
    "$(printf '0\t0')" \
    "$(judge_run '   Compiling flowfin-core v0.0.0 (/home/runner/work/core/core)
warning: running 5 tests is what the comment above says
')"

  echo "== the suppression register =="
  assert_out "allows: an entry carrying the reason it is not refused" \
    "$(printf 'ALLOW\t1\trace:some::path\tThe reason, and what would retire it.')" \
    "$(judge_register 'race:some::path The reason, and what would retire it.
')"
  assert_out "refuses: an entry with no reason, which is a finding nobody argued with" \
    "$(printf 'REFUSE\t1\trace:some::path\t')" \
    "$(judge_register 'race:some::path
')"
  assert_out "passes over: a comment, which is most of the register" \
    "" \
    "$(judge_register '# Findings this run does not refuse, with the reason on the same line.
')"
  assert_out "passes over: a blank line" \
    "" \
    "$(judge_register '
')"
  assert_out "passes: a reason of several words is one reason and not several fields" \
    "$(printf 'ALLOW\t1\trace:foo\tone two three')" \
    "$(judge_register 'race:foo one two three
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures thread-detector fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

say() {
  echo "$1"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "$1" >> "$GITHUB_STEP_SUMMARY"
  fi
}

# The register printed with its reasons, and refused where an entry carries none.
# The detector's own suppression file is derived from it here rather than being a
# second file somebody edits, so a suppression that reached the detector is one
# that carried a reason past this rule.
derived_suppressions=""

judge_register_file() {
  local records refused=0 allowed=0 line entry reason verdict

  if [ ! -f "$SUPPRESSIONS_FILE" ]; then
    echo "::error::${SUPPRESSIONS_FILE} is missing. The register is where a finding that is not going to be fixed is written down, and a run with no register is a run that could suppress anything."
    return 1
  fi

  records="$(parse_suppressions < "$SUPPRESSIONS_FILE")"
  derived_suppressions="$(mktemp)"

  echo "-- the findings this run does not refuse, from ${SUPPRESSIONS_FILE}"
  if [ -z "$records" ]; then
    echo "      none. Every finding the detector reports fails this run."
  else
    while IFS=$'\t' read -r verdict line entry reason; do
      [ -z "$verdict" ] && continue
      if [ "$verdict" = "REFUSE" ]; then
        echo "::error::${SUPPRESSIONS_FILE} line ${line}: '${entry}' carries no reason. A bare suppression is a finding somebody turned off and nobody argued with, so this run refuses the register rather than the finding."
        refused=$((refused + 1))
      else
        echo "      ${entry}: ${reason}"
        echo "$entry" >> "$derived_suppressions"
        allowed=$((allowed + 1))
      fi
    done <<REGISTER
$records
REGISTER
  fi
  echo

  if [ "$refused" -ne 0 ]; then
    echo "::error::${refused} entry/entries in the register carry no reason. Nothing was judged."
    return 1
  fi

  echo "      ${allowed} entry/entries reached the detector."
  echo
}

# One leg: run a command under the detector, print what it cost, and hand the
# output back for judging. The status is captured rather than allowed to end this
# script, because both legs owe the reader a verdict read from the report rather
# than from an exit code.
leg_output=""
leg_status=0
leg_seconds=0

run_under_detector() {
  local what="$1" command="$2" started

  echo "-- ${what}"
  echo "      ${command}"
  echo

  started="$SECONDS"
  leg_status=0
  # Word splitting on $command is what this line is for: the constants above hold
  # a command and its arguments, and quoting it would look for a program whose
  # name has spaces in it.
  # shellcheck disable=SC2086
  if ! leg_output="$(RUSTFLAGS="$DETECTOR_RUSTFLAGS" \
                     TSAN_OPTIONS="halt_on_error=0 exitcode=0 suppressions=${derived_suppressions}" \
                     $command 2>&1 | tee /dev/stderr)"; then
    leg_status=1
  fi
  leg_seconds=$((SECONDS - started))
  echo
}

check() {
  local reports binaries executed accounting suite_seconds race_seconds

  echo "-- what this leg is"
  echo "      the suite under a thread detector, and then a deliberate race under the same detector."
  echo "      A run of the second that reports nothing is a detector that is not switched on, and"
  echo "      it is the only thing here that tells that state from a clean tree."
  echo

  echo "-- which compiler judged this run"
  cargo "+${DETECTOR_CHANNEL}" --version
  rustc "+${DETECTOR_CHANNEL}" --version
  echo

  judge_register_file || return 1

  run_under_detector "the suite, under the detector" "$SUITE_COMMAND"
  suite_seconds="$leg_seconds"
  accounting="$(printf '%s\n' "$leg_output" | count_run)"
  IFS=$'\t' read -r binaries executed <<ACCOUNTING
$accounting
ACCOUNTING
  reports="$(printf '%s\n' "$leg_output" | detector_reports)"

  echo "-- what the suite leg collected"
  say "${executed} test(s) executed across ${binaries} test binary/binaries, in ${suite_seconds}s of wall clock."
  echo

  if [ "$binaries" -eq 0 ]; then
    echo "::error::The runner printed no accounting line at all under the detector, so how many tests it collected cannot be read. A run that collected nothing reports nothing, which is the one shape this check exists to refuse."
    return 1
  fi

  if [ "$executed" -eq 0 ]; then
    echo "::error::The suite collected 0 tests under the detector. A detector with nothing to watch reports nothing and prints a page that reads exactly like a clean run."
    return 1
  fi

  if [ -n "$reports" ]; then
    echo "-- what the detector reported against the suite"
    printf '%s\n' "$reports" | sed 's/^/      /'
    echo
    echo "::error::The detector reported against the suite. A promise in docs/decisions/0009-the-concurrency-model.md is broken, or a finding here needs an entry in ${SUPPRESSIONS_FILE} with the reason it is not being fixed."
    return 1
  fi

  if [ "$leg_status" -ne 0 ]; then
    echo "::error::The suite failed under the detector. The detector reported nothing, so this is the suite rather than a race: the run above says which test."
    return 1
  fi

  run_under_detector "the deliberate race, under the same detector" "$RACE_COMMAND"
  race_seconds="$leg_seconds"
  reports="$(printf '%s\n' "$leg_output" | detector_reports)"

  echo "-- what the detector reported against the deliberate race"
  if [ -z "$reports" ]; then
    say "nothing, in ${race_seconds}s of wall clock."
    echo
    echo "::error::tests/a_race_the_detector_must_catch.rs holds a data race written on purpose and the detector reported nothing against it. Either the detector is not switched on for this run, or the target did not build or did not run - the output above says which. A leg that cannot report this race cannot be read as having found the tree clean."
    return 1
  fi
  printf '%s\n' "$reports" | sed 's/^/      /'
  say "The deliberate race was reported, in ${race_seconds}s of wall clock, so the detector was switched on for this run."
  echo

  say "Wall clock: ${suite_seconds}s for the suite and ${race_seconds}s for the deliberate race, $((suite_seconds + race_seconds))s in total."
  echo

  echo "-- what this run did not read"
  echo "NOT COVERED HERE: a race that manifests only on a target this detector does not reach. It runs on ${DETECTOR_TARGET}, and the readings on #117 name Windows and Android as targets whose specifications do not carry this sanitizer, so a defect that appears only there is outside every run this leg makes."
  echo "NOT COVERED HERE: an interleaving the suite never produced. This detector reports a race it observed rather than one that is possible, so a promise in 0009 that no test exercises is unwatched here."
  echo "NOT COVERED HERE: a documentation test. The suite command above carries --all-targets, which leaves them out, because the driver that compiles them is not handed the detector's flag and the compiler refuses the resulting ABI mismatch by name. This crate carries none today, so nothing is left out of this run, and one written later is outside this leg rather than inside it."
  echo "NOT MADE HERE: whether the promises in 0009 are the right promises. This leg watches the ones the suite exercises being kept."
  echo "NOT MADE HERE: a merge condition. No check is required to merge on this board today and #26 is where a name is written into the ruleset."
  echo
  echo "The suite ran under the detector with nothing reported, and the deliberate race was reported."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
