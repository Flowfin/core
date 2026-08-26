#!/usr/bin/env bash
# The targets `--all-targets` leaves out are compiled anyway (#201).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHAT THIS IS FOR. `cargo build --locked --all-targets` selects the test targets
# carrying `test = true`, so a target this manifest declares with `test = false`
# is not compiled by it. `Cargo.toml` said one of them was, and it was not.
# Measured by breaking the file and watching the command stay green:
#
#     printf '\nthis is not rust and will not compile;\n' >> tests/needs_a_real_server_or_real_hardware.rs
#     cargo build --locked --all-targets ; echo "exit=$?"
#     exit=0
#     cargo build --locked --test needs_a_real_server_or_real_hardware ; echo "exit=$?"
#     exit=101
#
# So a rename of a type such a file names, or the removal of a function it calls,
# leaves the whole gate green and is found by whoever next runs that target by
# hand. That is nobody on a schedule.
#
# IT COMPILES AND DOES NOT RUN, which is the whole of the difference between this
# and the reasons those targets are excluded in the first place. One of them needs
# a real server or real hardware and refuses rather than skipping when it does not
# have them; the other is undefined behaviour on purpose and a contributor running
# the ordinary suite must not execute it. Neither reason is a reason not to
# compile the file, and compiling is what catches the defect this exists for.
#
# THE LIST IS DERIVED FROM THE MANIFEST AND IS NOT WRITTEN HERE. A list in this
# script would be the second declaration of a set the manifest already holds, and
# it would go stale on the day somebody adds a target - which is the one day this
# check is worth having.
#
# A PARSER THAT STOPPED MATCHING READS AS A TREE WITH NO SUCH TARGETS. So the
# names are counted against the raw occurrences of the setting they come from, and
# a disagreement is refused rather than reported as nothing to do.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      derive the excluded targets from the manifest and compile each one
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

MANIFEST="Cargo.toml"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line.
#
# awk rather than grep: grep exits 1 when it selects nothing, and "this manifest
# excludes no target" is a legitimate answer this script has to tell apart from a
# scanner that broke.
# --------------------------------------------------------------------------

# The name of every `[[test]]` target the manifest excludes from the ordinary
# command, one per line.
#
# The block is what decides it, not the line. `test = false` under `[lib]` is a
# different key on a different table and says nothing about a test target, and
# taking any occurrence of the setting would read it as one.
excluded_targets() {
  awk '
    function flush() {
      if (in_test && excluded && name != "") print name
      name = ""
      excluded = 0
    }
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      sub(/#.*$/, "", line)
      sub(/[ \t]+$/, "", line)

      if (line ~ /^\[/) {
        flush()
        in_test = (line == "[[test]]")
        next
      }
      if (!in_test) next

      if (line ~ /^name[ \t]*=/) {
        v = line
        sub(/^name[ \t]*=[ \t]*/, "", v)
        gsub(/"/, "", v)
        name = v
        next
      }
      if (line ~ /^test[ \t]*=[ \t]*false$/) {
        excluded = 1
        next
      }
    }
    END { flush() }
  '
}

# How many times the setting itself appears, anywhere in the manifest, outside a
# comment. Read so that a block parser that stopped matching cannot report an
# empty set and pass.
raw_exclusions() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/#.*$/, "", line)
      if (line ~ /(^|[ \t])test[ \t]*=[ \t]*false([ \t]*)$/) n = n + 1
    }
    END { printf "%d\n", n }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the manifest in this tree. A
# fixture that judged the real one would prove the state of the tree on the day it
# ran, not the rule.
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

judge_manifest() { printf '%s' "$1" | excluded_targets; }
judge_raw()      { printf '%s' "$1" | raw_exclusions; }

selftest() {
  echo "== a target the ordinary command leaves out =="
  assert_out "reads: a test target excluded by the setting" \
    "needs_a_real_server_or_real_hardware" \
    "$(judge_manifest '[[test]]
name = "needs_a_real_server_or_real_hardware"
path = "tests/needs_a_real_server_or_real_hardware.rs"
harness = false
test = false
')"
  assert_out "reads: two of them, and both" \
    "$(printf 'first\nsecond')" \
    "$(judge_manifest '[[test]]
name = "first"
test = false

[[test]]
name = "second"
test = false
')"
  assert_out "reads: the setting written before the name, which is the same block" \
    "either_order" \
    "$(judge_manifest '[[test]]
test = false
name = "either_order"
')"
  assert_out "reads: the setting written without the spaces around it" \
    "tight" \
    "$(judge_manifest '[[test]]
name = "tight"
test=false
')"

  echo "== what is not one =="
  assert_out "passes over: a test target the ordinary command does compile" \
    "" \
    "$(judge_manifest '[[test]]
name = "ordinary"
test = true
')"
  assert_out "passes over: a test target that says nothing about it, which is the default" \
    "" \
    "$(judge_manifest '[[test]]
name = "silent"
path = "tests/silent.rs"
')"
  assert_out "passes over: the same setting on the library table, which is a different key" \
    "" \
    "$(judge_manifest '[lib]
name = "flowfin_core"
test = false
')"
  assert_out "passes over: a benchmark table, which this check is not about" \
    "" \
    "$(judge_manifest '[[bench]]
name = "a_benchmark"
test = false
')"
  assert_out "passes over: the setting inside a comment, which is prose" \
    "" \
    "$(judge_manifest '[[test]]
name = "commented"
# test = false
')"
  assert_out "passes over: a block that carries the setting and no name at all" \
    "" \
    "$(judge_manifest '[[test]]
path = "tests/nameless.rs"
test = false
')"

  echo "== the count that catches a parser which stopped matching =="
  assert_out "counts: every occurrence of the setting, wherever it sits" \
    "3" \
    "$(judge_raw '[lib]
test = false

[[test]]
name = "one"
test = false

[[bench]]
test = false
')"
  assert_out "counts: nothing in a manifest that excludes nothing" \
    "0" \
    "$(judge_raw '[[test]]
name = "ordinary"
test = true
')"
  assert_out "counts: not a comment" \
    "0" \
    "$(judge_raw '# test = false is what excludes a target
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures excluded-target fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
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
  local names raw counted target failures=0

  if [ ! -f "$MANIFEST" ]; then
    echo "::error::${MANIFEST} is not here. This check derives what it compiles from that file, and an absent manifest is not a manifest that excludes nothing."
    return 1
  fi

  names="$(excluded_targets < "$MANIFEST")"
  raw="$(raw_exclusions < "$MANIFEST")"
  counted="$(printf '%s' "$names" | grep -c . || true)"

  echo "-- what the ordinary build leaves out"
  echo "      derived from ${MANIFEST}, never written here"
  if [ -z "$names" ]; then
    echo "      none"
  else
    printf '%s\n' "$names" | sed 's/^/      /'
  fi
  echo

  if [ "$counted" -ne "$raw" ]; then
    echo "::error::${MANIFEST} carries the exclusion setting ${raw} time(s) and this check derived ${counted} target name(s) from it. A block reader that stopped matching reports an empty set and reads exactly like a manifest with nothing to compile, so the disagreement is refused rather than passed."
    return 1
  fi

  if [ -z "$names" ]; then
    echo "-- nothing to compile"
    echo "This manifest excludes no target from the ordinary command, and the count above agrees, so there is nothing here for this check to build."
    echo
    echo "-- what this run did not read"
    echo "NOT MADE HERE: whether the ordinary command compiled anything. .github/workflows/build.yml runs it in the step beside this one."
    return 0
  fi

  while IFS= read -r target; do
    [ -n "$target" ] || continue
    echo "-- compiling ${target}"
    echo "      cargo build --locked --test ${target}"
    if ! cargo build --locked --test "$target"; then
      echo "::error::${target} did not compile. The ordinary build command does not reach it, so nothing else in this gate would have said so."
      failures=$((failures + 1))
    fi
    echo
  done <<NAMES
$names
NAMES

  if [ "$failures" -ne 0 ]; then
    echo "::error::${failures} of the ${counted} target(s) above did not compile."
    return 1
  fi

  say "${counted} target(s) outside \`cargo build --locked --all-targets\` were compiled."
  echo
  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether any of them passes. Each is excluded from the suite for a reason its own block states, and this step compiles rather than runs."
  echo "NOT MADE HERE: a target the manifest does not declare. The set is derived from ${MANIFEST}, so a file under tests/ that no block names is not compiled by anything."
  echo
  echo "Every target the ordinary command leaves out compiles."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
