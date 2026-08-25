#!/usr/bin/env bash
# The suite runs, and a run that collected nothing is not a run that found
# nothing (#16).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement the other scripts in this gate already use.
#
# WHAT THIS IS FOR, AND IT IS NOT "DID A TEST FAIL". The test runner already
# answers that and its exit code is passed straight through. What no exit code
# answers is how many tests it collected. A harness that ran none exits zero and
# prints a page that reads exactly like a clean run, and the way that arrives is
# a filter, a renamed target or a harness change - never an edit anybody
# associates with the suite. So the count is read out of the run, printed, and
# refused when it is zero.
#
# A FILTER IS REFUSED RATHER THAN REPORTED. The command below passes none, so a
# non-zero filtered-out total means something between this script and the
# harness is selecting, and a partial run that reads as a whole one is the exact
# thing this check exists to stop.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      run the suite, count what it collected, and refuse a run of nothing
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# The command a contributor runs, character for character, out of README.md and
# CONTRIBUTING.md. A gate running a variant of it is how the three come apart.
TEST_COMMAND="cargo test --locked"

# --------------------------------------------------------------------------
# Rules. The runner's output is read on stdin and one accounting line is written
# to stdout, as BINARIES<TAB>EXECUTED<TAB>IGNORED<TAB>FILTERED.
#
# awk rather than grep: grep exits 1 when it selects nothing, and "the harness
# printed no accounting at all" is a verdict this script has to tell apart from
# a scanner that broke.
# --------------------------------------------------------------------------

count_run() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)

      # One per test binary, including the ones that carry no test. Counting the
      # binaries as well as the tests is what makes a suite that lost a target
      # visible: the total can stay the same while a binary disappears.
      if (line ~ /^running [0-9]+ test/) {
        n = line
        sub(/^running /, "", n)
        sub(/ test.*$/, "", n)
        binaries = binaries + 1
        executed = executed + n
        next
      }

      # The summary line the harness writes after each binary. Read for what the
      # line above cannot carry: what was skipped, and what was selected away.
      if (line ~ /^test result:/) {
        rest = line
        while (match(rest, /[0-9]+ ignored/)) {
          v = substr(rest, RSTART, RLENGTH); sub(/ ignored/, "", v)
          ignored = ignored + v
          rest = substr(rest, RSTART + RLENGTH)
        }
        rest = line
        while (match(rest, /[0-9]+ filtered out/)) {
          v = substr(rest, RSTART, RLENGTH); sub(/ filtered out/, "", v)
          filtered = filtered + v
          rest = substr(rest, RSTART + RLENGTH)
        }
        next
      }
    }
    END {
      printf "%d\t%d\t%d\t%d\n", binaries, executed, ignored, filtered
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than a real run. A row that judged the
# suite in this tree would prove the state of the tree on the day it ran, not the
# rule.
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

judge_run() {
  printf '%s' "$1" | count_run
}

selftest() {
  echo "== an ordinary run is counted =="
  assert_out "reads: several binaries, and the tests are summed across them" \
    "$(printf '4\t10\t0\t0')" \
    "$(judge_run 'running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"
  assert_out "reads: one test, written in the singular by the harness" \
    "$(printf '1\t1\t0\t0')" \
    "$(judge_run 'running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"

  echo "== the two states that read like a clean run =="
  assert_out "reads: a filter that stopped matching, which is zero collected and exit zero" \
    "$(printf '1\t0\t0\t10')" \
    "$(judge_run 'running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
')"
  assert_out "reads: a harness that printed no accounting at all" \
    "$(printf '0\t0\t0\t0')" \
    "$(judge_run 'error: no test target named `nothing`
')"

  echo "== what is counted apart from what ran =="
  assert_out "reads: an ignored test, which is not an executed one" \
    "$(printf '1\t3\t2\t0')" \
    "$(judge_run 'running 3 tests
test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"
  assert_out "reads: a failing run, counted the same way a passing one is" \
    "$(printf '1\t3\t0\t0')" \
    "$(judge_run 'running 3 tests
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"

  echo "== what is not an accounting line =="
  assert_out "passes over: a test whose own name begins with the word the count is read from" \
    "$(printf '1\t1\t0\t0')" \
    "$(judge_run 'running 1 test
test running_2_tests_is_not_a_count ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
')"
  assert_out "passes over: the same sentence inside a compiler message" \
    "$(printf '0\t0\t0\t0')" \
    "$(judge_run '   Compiling flowfin-core v0.0.0 (/home/runner/work/core/core)
warning: running 5 tests is what the comment above says
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures test-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
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

check() {
  local output status=0 accounting binaries executed ignored filtered

  echo "-- the command this gate runs"
  echo "      ${TEST_COMMAND}"
  echo

  echo "-- the run"
  # The runner's own output goes to the terminal as it arrives and is kept for
  # the accounting below. Its exit status is captured rather than allowed to end
  # this script, because a failing suite still owes the reader a count.
  #
  # Word splitting on $TEST_COMMAND is what this line is for: the constant above
  # holds a command and its arguments, and quoting it would look for a program
  # whose name has spaces in it.
  # shellcheck disable=SC2086
  if ! output="$($TEST_COMMAND 2>&1 | tee /dev/stderr)"; then
    status=1
  fi
  echo

  accounting="$(printf '%s\n' "$output" | count_run)"
  IFS=$'\t' read -r binaries executed ignored filtered <<ACCOUNTING
$accounting
ACCOUNTING

  echo "-- what it collected"
  say "${executed} test(s) executed across ${binaries} test binary/binaries, ${ignored} ignored, ${filtered} filtered out."
  echo

  if [ "$binaries" -eq 0 ]; then
    echo "::error::The runner printed no accounting line at all, so how many tests it collected cannot be read. Refusing rather than reading an absent count as zero problems."
    return 1
  fi

  if [ "$executed" -eq 0 ]; then
    echo "::error::The suite collected 0 tests. A run that collected nothing exits zero and prints a page that reads exactly like a clean run, which is the failure this check exists for."
    return 1
  fi

  if [ "$filtered" -ne 0 ]; then
    echo "::error::${filtered} test(s) were filtered out by '${TEST_COMMAND}'. This gate's command is meant to select nothing, so a non-zero total here means something between it and the harness is selecting, and a partial run that reads as a whole one is what this check refuses."
    return 1
  fi

  if [ "$status" -ne 0 ]; then
    echo "::error::The suite failed. The count above says what it collected; the run above says which test."
    return 1
  fi

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether the tests are the right tests. A count is a count, and a suite of one assertion that never fails passes this check."
  echo "NOT MADE HERE: coverage. Nothing here measures which lines a run reached, and #84 is where that lands."
  echo "NOT MADE HERE: a path that needs real hardware or a real server. That is the separate harness in #22 and no run here touches it."
  echo
  echo "The suite collected ${executed} test(s) and every one of them passed."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
