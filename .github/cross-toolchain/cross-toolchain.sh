#!/usr/bin/env bash
# Which of 0113's target triples this runner can compile C for (#291).
#
# WHAT THIS IS FOR. 0243 decides that a certificate is validated through
# `rustls`. Every crypto provider `rustls` offers is C. The `targets` leg beside
# this one compiles the library for seven triples on one Linux runner with no C
# cross-toolchain, so the first compile after that manifest entry arrives goes
# red on every triple whose C compiler the runner does not carry. 0291 weighed
# three ways out of that, took the wait, and named a measurement that would
# reverse it: the runner image carrying a working C cross-toolchain for every
# triple in that set, taken on the runner rather than on a contributor's
# machine.
#
# THAT MEASUREMENT HAD NEVER BEEN TAKEN. #291's body names it as the issue's own
# first act; 0291 says in its own text that it is unmade, that every
# cross-compile reading behind the question was taken on a Windows machine
# outside this tree, and that taking it needs a run on the runner rather than a
# claim about the image. This leg is that run, and it is the only thing in this
# gate whose subject is the image rather than the tree.
#
# IT REPORTS AND REFUSES NOTHING ABOUT WHAT IT FOUND. A triple this runner
# cannot compile C for is the finding rather than a failure: it is the state
# 0291 already decided to wait through, and a leg reddening on every pull
# request for a decision already taken teaches people that red means nothing.
# Two runs in this gate already report without refusing and say so; this is the
# third and it says so here.
#
# WHAT IT DOES REFUSE is a survey that cannot be read. A block missing a field,
# a block naming a field this loader does not know, a triple in 0113's set that
# no block covers, a block naming a triple that set does not carry, a derived
# count that disagrees with the blocks it came from, and a register naming
# nothing. Each of those exits zero and prints a page indistinguishable from a
# run that examined seven triples and found them all served, which is the one
# reading this leg must never produce.
#
# THE SET IS NOT READ TWICE. The triples come from `.github/targets/targets`
# through the `triples` verb of `.github/targets/targets.sh`, which is the same
# reader the `targets` leg applies and the one whose fixtures prove it. A second
# reader of that register here would agree with it until the day it did not.
#
# A PROBE COMPILES A REAL TRANSLATION UNIT and never merely resolves a name. A
# compiler driver targets any triple it knows without a sysroot for as long as
# nothing asks for a header, so a probe that stopped at `command -v` would
# report every Apple triple served on a Linux image carrying no Apple SDK. The
# unit below includes the C library headers for that reason, which is the same
# thing a crypto provider's generated sources fail on when they fail.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      read the register, then compile the unit for every triple in it
#
# No POSIX character classes and no interval expressions in any pattern below,
# for the reason `.github/targets/targets.sh` gives: the awk on the runner is
# mawk and the awk on a contributor's machine is frequently gawk, and a rule
# that matches on one and not the other is a verdict that depends on who ran it.

set -euo pipefail

REGISTER=".github/cross-toolchain/probes"
TARGETS=".github/targets/targets.sh"
TAB="$(printf '\t')"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line.
#
# A block is a run of field lines. A blank line ends one and so does a comment,
# so the register can argue for itself between blocks without a comment being
# read into the block below it.
#
# awk rather than grep: grep exits 1 when it selects nothing, and "this register
# holds no incomplete block" is a legitimate answer this script has to tell
# apart from a scanner that broke.
# --------------------------------------------------------------------------

# The triple and the probe of every block carrying all three fields, tab
# separated, in the order the register writes them.
declared_probes() {
  awk '
    function flush() {
      if (fields > 0 && triple != "" && probe != "" && reason != "") {
        printf "%s\t%s\n", triple, probe
      }
      triple = ""; probe = ""; reason = ""; fields = 0
    }
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "" || substr(line, 1, 1) == "#") { flush(); next }

      sep = index(line, ":")
      if (sep == 0) { fields = fields + 1; next }
      name = substr(line, 1, sep - 1)
      value = substr(line, sep + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", name)
      gsub(/^[ \t]+|[ \t]+$/, "", value)
      fields = fields + 1
      if (name == "triple") triple = value
      else if (name == "probe") probe = value
      else if (name == "reason") reason = value
    }
    END { flush() }
  '
}

# One record per block that is missing one of the three fields, as the triple it
# names followed by the fields it lacks. A block naming no triple is reported by
# its position, because there is nothing else to call it by.
blocks_missing_a_field() {
  awk '
    function flush() {
      if (fields > 0) {
        block = block + 1
        missing = ""
        if (triple == "") missing = "triple"
        if (probe == "") missing = (missing == "" ? "probe" : missing ",probe")
        if (reason == "") missing = (missing == "" ? "reason" : missing ",reason")
        if (missing != "") {
          printf "%s\t%s\n", (triple == "" ? "block " block : triple), missing
        }
      }
      triple = ""; probe = ""; reason = ""; fields = 0
    }
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "" || substr(line, 1, 1) == "#") { flush(); next }

      sep = index(line, ":")
      if (sep == 0) { fields = fields + 1; next }
      name = substr(line, 1, sep - 1)
      value = substr(line, sep + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", name)
      gsub(/^[ \t]+|[ \t]+$/, "", value)
      fields = fields + 1
      if (name == "triple") triple = value
      else if (name == "probe") probe = value
      else if (name == "reason") reason = value
    }
    END { flush() }
  '
}

# The name of every field this loader does not know, one per line. A field
# spelled slightly wrong leaves its block incomplete, which the rule above
# reports; this one says which word was wrong rather than which value is absent.
unknown_fields() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "" || substr(line, 1, 1) == "#") next

      sep = index(line, ":")
      if (sep == 0) { print line; next }
      name = substr(line, 1, sep - 1)
      gsub(/^[ \t]+|[ \t]+$/, "", name)
      if (name != "triple" && name != "probe" && name != "reason") print name
    }
  '
}

# How many blocks the register offers at all, read so that a loader which
# stopped matching cannot report an empty set and pass.
raw_blocks() {
  awk '
    function flush() { if (fields > 0) n = n + 1; fields = 0 }
    {
      line = $0
      sub(/\r$/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      if (line == "" || substr(line, 1, 1) == "#") { flush(); next }
      fields = fields + 1
    }
    END { flush(); printf "%d\n", n }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the register in this tree. A
# fixture that judged the real one would prove the state of the tree on the day
# it ran, not the rule.
# --------------------------------------------------------------------------

selftest_failures=0

assert_out() {
  local what="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    printf 'ok    %s\n' "$what"
  else
    printf 'FAIL  %s\n        expected: %s\n        actual:   %s\n' \
      "$what" "$(printf '%s' "$expected" | tr '\n\t' '|>')" "$(printf '%s' "$actual" | tr '\n\t' '|>')"
    selftest_failures=$((selftest_failures + 1))
  fi
}

judge_declared() { printf '%s' "$1" | declared_probes; }
judge_missing()  { printf '%s' "$1" | blocks_missing_a_field; }
judge_unknown()  { printf '%s' "$1" | unknown_fields; }
judge_raw()      { printf '%s' "$1" | raw_blocks; }

selftest() {
  echo "== a block the survey runs =="
  assert_out "reads: a block carrying all three fields" \
    "$(printf 'aarch64-apple-ios\tclang --target=arm64-apple-ios')" \
    "$(judge_declared 'triple: aarch64-apple-ios
probe: clang --target=arm64-apple-ios
reason: The invocation a C build for that triple reaches for.
')"
  assert_out "reads: two blocks separated by a blank line, in the order written" \
    "$(printf 'aarch64-apple-ios\tclang --target=arm64-apple-ios\nx86_64-unknown-linux-gnu\tcc')" \
    "$(judge_declared 'triple: aarch64-apple-ios
probe: clang --target=arm64-apple-ios
reason: The phone.

triple: x86_64-unknown-linux-gnu
probe: cc
reason: The host.
')"
  assert_out "reads: two blocks naming one triple, because a triple is served where any probe compiles" \
    "$(printf 'aarch64-linux-android\tclang --target=aarch64-linux-android21\naarch64-linux-android\taarch64-linux-android21-clang')" \
    "$(judge_declared 'triple: aarch64-linux-android
probe: clang --target=aarch64-linux-android21
reason: The driver form.

triple: aarch64-linux-android
probe: aarch64-linux-android21-clang
reason: The wrapper the NDK installs.
')"
  assert_out "reads: a probe carrying a colon of its own, which stays in the value" \
    "$(printf 'x86_64-unknown-linux-gnu\tcc -DA=b:c')" \
    "$(judge_declared 'triple: x86_64-unknown-linux-gnu
probe: cc -DA=b:c
reason: A definition with a colon in it is a value and not a second field.
')"
  assert_out "reads: a block indented by somebody tidying the file" \
    "$(printf 'x86_64-unknown-linux-gnu\tcc')" \
    "$(judge_declared '   triple: x86_64-unknown-linux-gnu
   probe: cc
   reason: The host.
')"

  echo "== what is not one =="
  assert_out "passes over: a comment, which is where the register argues for itself" \
    "" \
    "$(judge_declared '# triple: aarch64-apple-ios is not a block when it is prose.
# probe: clang
# reason: none.
')"
  assert_out "ends a block: a comment between two of them, so the prose is not read into the second" \
    "$(printf 'x86_64-unknown-linux-gnu\tcc')" \
    "$(judge_declared 'triple: aarch64-apple-ios
probe: clang --target=arm64-apple-ios
# the reason for the block above went missing here
triple: x86_64-unknown-linux-gnu
probe: cc
reason: The host.
')"
  assert_out "does not read as declared: a block whose reason is whitespace" \
    "" \
    "$(judge_declared "$(printf 'triple: x86_64-unknown-linux-gnu\nprobe: cc\nreason:  \t \n')")"

  echo "== the rule that refuses a block nobody finished =="
  assert_out "refuses: a block with no reason, naming the triple and the field" \
    "$(printf 'x86_64-unknown-linux-gnu\treason')" \
    "$(judge_missing 'triple: x86_64-unknown-linux-gnu
probe: cc
')"
  assert_out "refuses: a block with no probe" \
    "$(printf 'aarch64-apple-tvos\tprobe')" \
    "$(judge_missing 'triple: aarch64-apple-tvos
reason: The television.
')"
  assert_out "refuses: a block with no triple, by its position, since there is nothing to call it by" \
    "$(printf 'block 1\ttriple')" \
    "$(judge_missing 'probe: cc
reason: The host.
')"
  assert_out "refuses: the unfinished block and not the finished neighbour one line away" \
    "$(printf 'aarch64-apple-tvos\treason')" \
    "$(judge_missing 'triple: x86_64-unknown-linux-gnu
probe: cc
reason: The host.

triple: aarch64-apple-tvos
probe: clang --target=arm64-apple-tvos
')"
  assert_out "passes over: a block carrying all three" \
    "" \
    "$(judge_missing 'triple: x86_64-unknown-linux-gnu
probe: cc
reason: The host.
')"

  echo "== the rule that names the word that was spelled wrong =="
  assert_out "refuses: a field this loader does not know" \
    "reasons" \
    "$(judge_unknown 'triple: x86_64-unknown-linux-gnu
probe: cc
reasons: The plural is the mistake somebody makes.
')"
  assert_out "refuses: a line carrying no colon at all" \
    "probe cc" \
    "$(judge_unknown 'triple: x86_64-unknown-linux-gnu
probe cc
reason: The host.
')"
  assert_out "passes over: the three fields it knows" \
    "" \
    "$(judge_unknown 'triple: x86_64-unknown-linux-gnu
probe: cc
reason: The host.
')"
  assert_out "passes over: a comment naming a field that does not exist" \
    "" \
    "$(judge_unknown '# reasons: a comment is not a field line
')"

  echo "== the count that catches a loader which stopped matching =="
  assert_out "counts: every run of field lines the register offers" \
    "2" \
    "$(judge_raw '# the register argues for itself here
triple: x86_64-unknown-linux-gnu
probe: cc
reason: The host.

triple: aarch64-apple-ios
probe: clang --target=arm64-apple-ios
reason: The phone.
')"
  assert_out "counts: an unfinished block, which is a block and a refused one" \
    "1" \
    "$(judge_raw 'triple: x86_64-unknown-linux-gnu
')"
  assert_out "counts: nothing in a register that is all prose" \
    "0" \
    "$(judge_raw '# nothing here is a block
# and neither is this
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures cross-toolchain fixture(s) did not hold. The rules below are not the rules that were proven, so this run measures nothing."
    return 1
  fi
  echo "Every fixture held. The rules the survey applies are the rules these fixtures ran."
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

# The translation unit every probe compiles. It asks for the C library headers
# rather than only for the ones a freestanding compiler supplies itself, because
# a driver reaches an unknown platform's triple without a sysroot for as long as
# nothing needs a header, and a unit that needed none would report a
# cross-toolchain wherever a driver exists.
write_the_unit() {
  cat > "$1" <<'THE_UNIT'
/* Compiled by .github/cross-toolchain/cross-toolchain.sh for one triple at a
   time. It asks for the platform's own C library headers on purpose: that is
   what a crypto provider's generated sources ask for, and it is what a
   compiler without that platform's sysroot cannot answer. */
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

size_t the_length_a_provider_would_copy(const uint8_t *bytes, size_t length) {
    void *copy = malloc(length);
    if (copy == NULL) {
        return 0;
    }
    memcpy(copy, bytes, length);
    free(copy);
    return length;
}
THE_UNIT
}

check() {
  local declared missing unknown raw counted refused wanted triple probe name
  local fields binary output unserved served=0 total=0 failures=0 workspace unit
  local this_one_is_served
  local argv=()

  if [ ! -f "$REGISTER" ]; then
    echo "::error::${REGISTER} is not here. This survey reads what it invokes out of that file, and an absent register is not a register naming no probe."
    return 1
  fi
  if [ ! -f "$TARGETS" ]; then
    echo "::error::${TARGETS} is not here. The set of triples is that script's answer rather than this one's, so without it this run has nothing to survey and nothing to say it left out."
    return 1
  fi

  declared="$(declared_probes < "$REGISTER")"
  missing="$(blocks_missing_a_field < "$REGISTER")"
  unknown="$(unknown_fields < "$REGISTER")"
  raw="$(raw_blocks < "$REGISTER")"
  counted="$(printf '%s' "$declared" | grep -c . || true)"
  refused="$(printf '%s' "$missing" | grep -c . || true)"

  echo "-- the register, read with its reasons"
  echo "      ${REGISTER}"
  sed 's/^/      /' "$REGISTER"
  echo

  if [ -n "$unknown" ]; then
    while IFS= read -r name; do
      [ -n "$name" ] || continue
      echo "::error::${REGISTER} carries a line this loader does not read as a field: ${name}. A field spelled wrong leaves its block short of a value, and the block is then a probe nobody argued for."
    done <<UNKNOWN
$unknown
UNKNOWN
    return 1
  fi

  if [ "$refused" -ne 0 ]; then
    while IFS="$TAB" read -r name fields; do
      [ -n "$name" ] || continue
      echo "::error::${REGISTER} carries ${name} without ${fields}. A block short of a field is a probe that runs whatever it happens to name, against a triple nobody stated, for a reason nobody wrote."
    done <<MISSING
$missing
MISSING
    return 1
  fi

  if [ "$counted" -ne "$raw" ]; then
    echo "::error::${REGISTER} offers ${raw} block(s) and this loader read ${counted} of them as complete with none reported short. A loader that stopped matching reports an empty set and reads exactly like a register with nothing to survey, so the disagreement is refused rather than passed."
    return 1
  fi

  if [ "$counted" -eq 0 ]; then
    echo "::error::${REGISTER} names no probe. An empty register invokes nothing, exits zero and prints a page indistinguishable from a run that surveyed every triple and found them served, so it is refused rather than read as a set with nothing to do."
    return 1
  fi

  wanted="$(bash "$TARGETS" triples)"
  if [ -z "$wanted" ]; then
    echo "::error::${TARGETS} names no triple. The set this run surveys is that register's answer, and a survey of nothing exits zero and prints a page that reads like a complete one."
    return 1
  fi

  echo "-- the set this run surveys, read through the reader the targets leg uses"
  echo "      bash ${TARGETS} triples"
  printf '%s\n' "$wanted" | sed 's/^/      /'
  echo

  while IFS= read -r triple; do
    [ -n "$triple" ] || continue
    if ! printf '%s\n' "$declared" | cut -f1 | grep -qx -- "$triple"; then
      echo "::error::${triple} is in 0113's set and ${REGISTER} carries no probe for it. A triple added to that set and left unsurveyed is a platform this run says nothing about, on a page that reads as though it said something about all of them."
      failures=$((failures + 1))
    fi
  done <<WANTED
$wanted
WANTED

  while IFS= read -r triple; do
    [ -n "$triple" ] || continue
    if ! printf '%s\n' "$wanted" | grep -qx -- "$triple"; then
      echo "::error::${REGISTER} carries a probe for ${triple} and 0113's set does not name that triple. A probe for a platform nobody links is a compiler this run reports on for no reason, and it is how a register outlives the platform it was written for."
      failures=$((failures + 1))
    fi
  done <<DECLARED
$(printf '%s\n' "$declared" | cut -f1 | sort -u)
DECLARED

  if [ "$failures" -ne 0 ]; then
    echo "::error::the register and 0113's set disagree in ${failures} place(s). Nothing was compiled."
    return 1
  fi

  workspace="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '${workspace}'" EXIT
  unit="${workspace}/the_unit_a_provider_would_compile.c"
  write_the_unit "$unit"

  echo "-- the unit every probe compiles"
  sed 's/^/      /' "$unit"
  echo

  echo "-- what each probe answered"
  unserved=""
  while IFS= read -r triple; do
    [ -n "$triple" ] || continue
    total=$((total + 1))
    this_one_is_served=0
    echo "-- ${triple}"
    while IFS="$TAB" read -r name probe; do
      [ "$name" = "$triple" ] || continue
      [ -n "$probe" ] || continue
      IFS=' ' read -r -a argv <<< "$probe"
      binary="${argv[0]}"
      output="${workspace}/$(printf '%s' "${triple}-${binary}" | tr -c 'A-Za-z0-9_.-' '_').o"
      if ! command -v "$binary" > /dev/null 2>&1; then
        echo "      ABSENT   ${probe}"
        echo "               ${binary} is not on this image's PATH."
        continue
      fi
      echo "      RUN      ${probe} -c <unit>.c -o <unit>.o"
      if "${argv[@]}" -c "$unit" -o "$output" > "${workspace}/out" 2>&1; then
        echo "      COMPILED ${probe}"
        this_one_is_served=1
      else
        echo "      FAILED   ${probe}"
        head -8 "${workspace}/out" | sed 's/^/               /'
      fi
    done <<PROBES
$declared
PROBES
    if [ "$this_one_is_served" -eq 1 ]; then
      served=$((served + 1))
      echo "      => ${triple}: a C compiler on this image compiles for it."
    else
      unserved="${unserved}${triple} "
      echo "      => ${triple}: no probe on this image compiled for it."
    fi
    echo
  done <<WANTED
$wanted
WANTED

  say "This runner compiles C for ${served} of the ${total} triple(s) 0113's set names."
  if [ "$served" -eq "$total" ]; then
    say "EVERY TRIPLE IS SERVED. That is the reading 0291 names as one of the conditions that reverse it: the cross-toolchain way out then costs no runner bill and no maintained matrix, the reason for waiting is gone, and the choice between the three is retaken rather than inherited."
  else
    say "NOT SERVED: ${unserved% }. 0291's wait stands on this reading rather than in spite of it, and the manifest entry 0243 requires would redden the targets leg for each of those."
  fi

  echo
  echo "-- what this run did not read"
  echo "NOT MEASURED HERE: whether a provider itself builds. This compiles one translation unit that asks for the C library headers, which is the thing a provider's generated sources fail on first, and it is not a build of aws-lc or of anything else. A triple reported served here can still fail deeper in a real build script."
  echo "NOT MEASURED HERE: linking. Every probe stops at -c, so a platform whose headers are present and whose linker or libraries are not is reported served by this run."
  echo "NOT MEASURED HERE: a compiler nobody wrote a block for. The register holds what somebody named, so an image carrying a working cross-toolchain under a name that is not in it is reported unserved, and the repair is a block rather than a change to this script."
  echo "NOT MEASURED HERE: any image but the one this job ran on. The verdict is this runner's at this moment, and an image that gains a toolchain next month changes it without anything here noticing."
  echo "NOT MEASURED HERE: whether the wait should end. 0291 asks for two things and this is one of them; the other is a pure-Rust provider reaching a release the targets leg compiles on every triple, which is a version string and not an image."
  echo
  echo "This leg refuses nothing about what it found. What it refuses is a survey that cannot be read, and every one of those rules ran its fixture above."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
