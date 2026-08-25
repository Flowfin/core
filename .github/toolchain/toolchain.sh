#!/usr/bin/env bash
# The compiler that ran is the compiler this tree pins (#14).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement the other four scripts in this gate already use.
#
# WHAT THIS IS FOR, AND WHO IT IS FOR. `rust-toolchain.toml` is read by the
# toolchain manager itself, so a contributor who has that manager gets the pinned
# compiler without being told anything and this comparison is green for them by
# construction. It is not green for a contributor whose compiler came from
# somewhere else - a distribution package, a company image, an installer - and
# for that person the alternative to this message is a compile error from a
# language edition or a lint their compiler does not have. The message names the
# file, the number it declares and the number that ran.
#
# NO VERSION STRING LIVES IN THIS FILE. The number is read out of
# `rust-toolchain.toml` every time. A copy here would be the second declaration
# the pin exists to remove.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      compare the compiler that ran against the file, and refuse
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

PIN_FILE="rust-toolchain.toml"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes one line to stdout, or exits
# non-zero when the subject does not carry what it is asked for.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is a verdict
# this script has to tell apart from a scanner that broke, and a pipeline that
# does that one `set -o pipefail` at a time is how a gate ends up passing on
# everything.
# --------------------------------------------------------------------------

# The channel, out of the `[toolchain]` table and out of no other. A key of the
# same spelling under another table is a different key, and taking the first
# match in the file would read it.
declared_channel() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      if (line ~ /^\[/) { in_toolchain = (line ~ /^\[toolchain\][ \t]*$/); next }
      if (!in_toolchain) next
      if (line !~ /^channel[ \t]*=/) next
      sub(/^channel[ \t]*=[ \t]*/, "", line)
      sub(/#.*$/, "", line)
      gsub(/"/, "", line)
      gsub(/[ \t]/, "", line)
      if (line != "") { print line; found = 1; exit }
    }
    END { if (!found) exit 1 }
  '
}

# The version that actually ran, out of the compiler's own long report. The
# `release:` line is read rather than the first line, because the first line
# carries a commit hash and a date beside the number and a comparison against it
# would refuse a rebuild of the same version.
running_release() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line !~ /^release:[ \t]*/) next
      sub(/^release:[ \t]*/, "", line)
      gsub(/[ \t]/, "", line)
      if (line != "") { print line; found = 1; exit }
    }
    END { if (!found) exit 1 }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the file or the compiler in this
# tree. A row that judged the real ones would prove the state of this machine on
# the day it ran, not the rule.
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

judge_pin() {
  printf '%s' "$1" | declared_channel || echo "REFUSED"
}

judge_report() {
  printf '%s' "$1" | running_release || echo "REFUSED"
}

selftest() {
  echo "== the pinned channel comes out of the toolchain table =="
  assert_out "reads: the channel this tree declares" "1.98.0" \
    "$(judge_pin '[toolchain]
channel = "1.98.0"
components = ["clippy", "rustfmt"]
')"
  assert_out "reads: a value with a comment after it" "1.90.0" \
    "$(judge_pin '[toolchain]
channel = "1.90.0"  # whatever a comment says
')"
  assert_out "bites: a channel under another table, which is a different key" "REFUSED" \
    "$(judge_pin '[toolchain]
profile = "minimal"

[other]
channel = "nightly"
')"
  assert_out "bites: a file declaring no channel at all" "REFUSED" \
    "$(judge_pin '[toolchain]
profile = "minimal"
')"

  echo "== the version that ran comes out of the release line =="
  assert_out "reads: the release, and not the summary line above it" "1.98.0" \
    "$(judge_report 'rustc 1.98.0 (88d9e12ae 2026-08-18)
binary: rustc
commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea
release: 1.98.0
')"
  assert_out "reads: a release carrying a channel suffix, whole" "1.99.0-beta.2" \
    "$(judge_report 'rustc 1.99.0-beta.2 (0000000 2026-09-01)
release: 1.99.0-beta.2
')"
  assert_out "bites: a report with no release line, which is not a version of nothing" "REFUSED" \
    "$(judge_report 'rustc 1.98.0 (88d9e12ae 2026-08-18)
binary: rustc
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures toolchain-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local pinned running report

  if [ ! -f "$PIN_FILE" ]; then
    echo "::error::${PIN_FILE} does not exist. Without it the compiler is whatever this machine happens to carry, and this run will not pass in place of the pin."
    return 1
  fi

  if ! pinned="$(declared_channel < "$PIN_FILE")"; then
    echo "::error::${PIN_FILE} declares no channel under [toolchain]. A pin file that pins nothing reads exactly like one that does."
    return 1
  fi

  report="$(rustc -vV)"
  if ! running="$(printf '%s\n' "$report" | running_release)"; then
    echo "::error::rustc -vV printed no release line, so which compiler produced this run cannot be read. Refusing rather than guessing."
    printf '%s\n' "$report"
    return 1
  fi

  echo "-- what this tree pins"
  echo "      ${pinned}, read from ${PIN_FILE}"
  echo
  echo "-- which compiler ran"
  printf '%s\n' "$report"
  echo

  if [ "$pinned" != "$running" ]; then
    echo "::error::This tree pins Rust ${pinned} in ${PIN_FILE} and rustc ${running} ran. Install the pinned toolchain rather than building with this one: 'rustup toolchain install ${pinned}' picks it up from ${PIN_FILE} on the next command, and a compiler installed some other way has to be changed by hand."
    echo "      REFUSED: pinned ${pinned}, running ${running}"
    return 1
  fi

  echo "-- the verdict"
  echo "      the compiler that ran is the compiler ${PIN_FILE} pins: ${pinned}"
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: which components are installed. The pin file names them, the toolchain manager installs them with the compiler, and a component missing on a machine that ignored the pin fails at the check that needs it rather than here."
  echo "NOT MADE HERE: the nightly toolchain. This tree pins one channel, the detector in #117 is a nightly flag, and that leg names its own toolchain where it runs."
  echo "NOT MADE HERE: whether the pinned version is a good version to pin. That is a judgement, and moving it is a commit somebody argues for."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
