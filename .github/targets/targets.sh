#!/usr/bin/env bash
# The library is compiled for every target triple a client links it on (#113).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHAT THIS IS FOR. `build` compiles the tree once, for whatever triple the runner
# happens to be, and 0011 says the cost of this means is one build leg per target
# triple. A core that compiles on the runner and not on the platform a client
# links it into is a defect discovered in somebody else's repository, by somebody
# who was not looking for it, at the moment they first try to ship. Nothing in
# this gate asked the question until this landed.
#
# IT COMPILES AND DOES NOT RUN. Not one line of the suite executes on any triple
# in the register beside this file except the host, and that bound is printed on
# every run rather than left for a green tick to be read past. What a compile
# catches is the class that stops a build: a construct one toolchain accepts and
# another does not, a pointer-width or alignment assumption, a conditional
# compilation arm that only exists on one platform. What it cannot catch is
# behaviour, which needs a device.
#
# THE SET IS DATA AND IS NOT WRITTEN IN THIS SCRIPT. The register beside this file
# holds it, one triple per line with the reason it is there, so adding a platform
# is editing a register and the argument for the platform sits on the same line as
# the platform. A list inside this file would be a rule nobody can argue with
# without reading shell.
#
# AN EMPTY REGISTER READS EXACTLY LIKE A CLEAN RUN, and so does a reader that
# stopped matching: both compile nothing, exit zero, and print a page with no
# failure on it. So the triples derived are counted against the raw lines they
# came from, a disagreement is refused, and a register carrying no entry at all is
# refused rather than passed as a set with nothing to do.
#
# A TRIPLE THE COMPILER DOES NOT KNOW IS REFUSED BY NAME rather than arriving as
# whatever the toolchain manager says when it is handed one. A typo in the
# register is the ordinary way that happens, and the message a reader needs is
# which line is wrong.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      read the register, then compile the library for every triple in it
#   triples    print the set, for the leg beside this one that surveys it
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

REGISTER=".github/targets/targets"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line.
#
# awk rather than grep: grep exits 1 when it selects nothing, and "this register
# holds no entry without a reason" is a legitimate answer this script has to tell
# apart from a scanner that broke.
# --------------------------------------------------------------------------

# The triple of every entry that carries a reason, one per line.
declared_triples() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "") next
      if (substr(line, 1, 1) == "#") next

      sep = index(line, " ")
      tab = index(line, "\t")
      if (tab > 0 && (sep == 0 || tab < sep)) sep = tab
      if (sep == 0) next

      reason = substr(line, sep + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", reason)
      if (reason == "") next

      print substr(line, 1, sep - 1)
    }
  '
}

# The triple of every entry that carries none. A bare identifier is a platform
# somebody added and nobody argued for.
triples_without_a_reason() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "") next
      if (substr(line, 1, 1) == "#") next

      sep = index(line, " ")
      tab = index(line, "\t")
      if (tab > 0 && (sep == 0 || tab < sep)) sep = tab
      if (sep == 0) { print line; next }

      reason = substr(line, sep + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", reason)
      if (reason == "") print substr(line, 1, sep - 1)
    }
  '
}

# How many lines the register offers as entries at all, read so that a reader
# which stopped matching cannot report an empty set and pass.
raw_entries() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "") next
      if (substr(line, 1, 1) == "#") next
      n = n + 1
    }
    END { printf "%d\n", n }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the register in this tree. A
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

judge_declared()   { printf '%s' "$1" | declared_triples; }
judge_unreasoned() { printf '%s' "$1" | triples_without_a_reason; }
judge_raw()        { printf '%s' "$1" | raw_entries; }

selftest() {
  echo "== an entry the gate compiles for =="
  assert_out "reads: a triple with its reason after it" \
    "aarch64-linux-android" \
    "$(judge_declared 'aarch64-linux-android Android phones and tablets.
')"
  assert_out "reads: two of them, and both, in the order written" \
    "$(printf 'aarch64-apple-ios\nx86_64-pc-windows-msvc')" \
    "$(judge_declared 'aarch64-apple-ios The phone.
x86_64-pc-windows-msvc The desktop.
')"
  assert_out "reads: an entry separated from its reason by a tab" \
    "aarch64-apple-darwin" \
    "$(judge_declared "$(printf 'aarch64-apple-darwin\tThe desktop on Apple silicon.\n')")"
  assert_out "reads: an entry indented by somebody tidying the file" \
    "x86_64-unknown-linux-gnu" \
    "$(judge_declared '   x86_64-unknown-linux-gnu The desktop.
')"

  echo "== what is not one =="
  assert_out "passes over: a comment, which is where the register argues for itself" \
    "" \
    "$(judge_declared '# aarch64-linux-android is not an entry when it is prose.
')"
  assert_out "passes over: a blank line" \
    "" \
    "$(judge_declared '

')"
  assert_out "does not read as declared: a triple with no reason after it" \
    "" \
    "$(judge_declared 'aarch64-linux-android
')"
  assert_out "does not read as declared: a triple with only spaces after it" \
    "" \
    "$(judge_declared "$(printf 'aarch64-linux-android    \n')")"

  echo "== the rule that refuses a platform nobody argued for =="
  assert_out "refuses: a bare triple on its own line" \
    "aarch64-linux-android" \
    "$(judge_unreasoned 'aarch64-linux-android
')"
  assert_out "refuses: a triple whose reason is whitespace" \
    "aarch64-apple-tvos" \
    "$(judge_unreasoned "$(printf 'aarch64-apple-tvos \t \n')")"
  assert_out "refuses: the bare one and not the neighbour one word away" \
    "armv7-linux-androideabi" \
    "$(judge_unreasoned 'aarch64-linux-android The 64-bit ABI.
armv7-linux-androideabi
')"
  assert_out "passes over: an entry that carries a reason" \
    "" \
    "$(judge_unreasoned 'aarch64-linux-android Android phones and tablets.
')"
  assert_out "passes over: a comment that is only a triple" \
    "" \
    "$(judge_unreasoned '#aarch64-linux-android
')"

  echo "== the count that catches a reader which stopped matching =="
  assert_out "counts: every line the register offers as an entry" \
    "3" \
    "$(judge_raw '# the register argues for itself here
aarch64-linux-android Android.
armv7-linux-androideabi

x86_64-pc-windows-msvc The desktop.
')"
  assert_out "counts: a bare triple, which is an entry and a refused one" \
    "1" \
    "$(judge_raw 'aarch64-linux-android
')"
  assert_out "counts: nothing in a register that is all prose" \
    "0" \
    "$(judge_raw '# nothing here is an entry
# and neither is this
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures target-register fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
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
  local declared unreasoned raw counted refused known triple failures=0

  if [ ! -f "$REGISTER" ]; then
    echo "::error::${REGISTER} is not here. This check reads the set it compiles for out of that file, and an absent register is not a register naming no platform."
    return 1
  fi

  declared="$(declared_triples < "$REGISTER")"
  unreasoned="$(triples_without_a_reason < "$REGISTER")"
  raw="$(raw_entries < "$REGISTER")"
  counted="$(printf '%s' "$declared" | grep -c . || true)"
  refused="$(printf '%s' "$unreasoned" | grep -c . || true)"

  echo "-- the register, read with its reasons"
  echo "      ${REGISTER}"
  sed 's/^/      /' "$REGISTER"
  echo

  if [ "$refused" -ne 0 ]; then
    while IFS= read -r triple; do
      [ -n "$triple" ] || continue
      echo "::error::${REGISTER} carries ${triple} with no reason after it. A platform somebody added and nobody argued for is a leg that runs on every pull request without anybody knowing what it protects."
    done <<UNREASONED
$unreasoned
UNREASONED
    return 1
  fi

  if [ "$((counted + refused))" -ne "$raw" ]; then
    echo "::error::${REGISTER} offers ${raw} entry line(s) and this check read ${counted} with a reason and ${refused} without. A reader that stopped matching reports an empty set and reads exactly like a register with nothing to compile, so the disagreement is refused rather than passed."
    return 1
  fi

  if [ "$counted" -eq 0 ]; then
    echo "::error::${REGISTER} names no target triple. An empty register compiles nothing, exits zero and prints a page indistinguishable from a run that compiled every platform, so it is refused rather than read as a set with nothing to do."
    return 1
  fi

  known="$(rustc --print target-list)"
  while IFS= read -r triple; do
    [ -n "$triple" ] || continue
    if ! printf '%s\n' "$known" | grep -qx -- "$triple"; then
      echo "::error::${REGISTER} names ${triple} and this compiler does not know that triple. A typo in the register is how that happens, and the toolchain manager's own message names neither the file nor the line."
      failures=$((failures + 1))
    fi
  done <<TRIPLES
$declared
TRIPLES

  if [ "$failures" -ne 0 ]; then
    echo "::error::${failures} of the ${counted} triple(s) above are not triples this compiler knows. Nothing was compiled."
    return 1
  fi

  while IFS= read -r triple; do
    [ -n "$triple" ] || continue
    echo "-- ${triple}"
    echo "      rustup target add ${triple}"
    if ! rustup target add "$triple"; then
      echo "::error::the standard library for ${triple} could not be installed, so the core was not compiled for it. That is a gate that reported nothing about this platform rather than a platform that failed."
      failures=$((failures + 1))
      echo
      continue
    fi
    echo "      cargo build --locked --lib --target ${triple}"
    if ! cargo build --locked --lib --target "$triple"; then
      echo "::error::the library did not compile for ${triple}. The build check compiles for the runner's own triple only, so nothing else in this gate would have said so."
      failures=$((failures + 1))
    fi
    echo
  done <<TRIPLES
$declared
TRIPLES

  if [ "$failures" -ne 0 ]; then
    echo "::error::${failures} of the ${counted} triple(s) above did not compile."
    return 1
  fi

  say "The library compiles for ${counted} target triple(s): $(printf '%s' "$declared" | tr '\n' ' ')"
  echo
  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether anything RUNS on any of these triples. This compiles and does not execute, and the suite runs on the runner's own host and nowhere else. A green run here says the core builds for a platform and says nothing about how it behaves on one."
  echo "NOT MADE HERE: the binding layer. 0011 puts a generated foreign function interface between this library and every client, no such artefact is in this tree, and nothing here compiles or links one."
  echo "NOT MADE HERE: a platform outside the register. The set is what the register names, so a television or a desktop nobody wrote a line for is not covered by this run and is not reported by it either."
  echo "NOT MADE HERE: the library profile. Each triple is compiled once, unoptimised, so an optimiser difference between platforms is outside every run this leg makes."
  echo
  echo "Every triple the register names compiles."
}

# The set itself, for a leg beside this one that has to survey the same triples.
# It is a verb here rather than a second reader there, because two readers of one
# register agree until the day they do not, and the disagreement then shows up as
# a platform one leg reports on and the other does not.
triples() {
  if [ ! -f "$REGISTER" ]; then
    echo "::error::${REGISTER} is not here." >&2
    return 1
  fi
  declared_triples < "$REGISTER"
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  triples)  triples ;;
  *)        echo "usage: $0 selftest|check|triples" >&2; exit 2 ;;
esac
