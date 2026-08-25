#!/usr/bin/env bash
# The formatter the language ships, run over the tree rather than over the crate
# (#18).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement the other three scripts in this gate already use.
#
# WHY THE SUBJECT IS THE TRACKED SET AND NOT THE CRATE. `cargo fmt` reaches a
# source file by walking the module graph from each target, so a tracked `.rs`
# file that no `mod` declares is never opened and never judged. The set this
# script formats comes from `git ls-files` instead, which is the same authority
# `.github/doc-paths/doc-paths.sh` uses and for the same reason: what a reader
# meets is what the tree carries, not what a graph happens to reach.
#
# WHAT THE CONFIGURATION SAYS IS IN `rustfmt.toml` AND IS NOT RESTATED HERE. That
# file carries the line-ending statement #99 put in `.gitattributes`, and every
# other rule is the formatter's own default for the style edition below. In
# `--check` mode a carriage return is REFUSED rather than converted, which is the
# direction worth having: converting hides the case where the byte arrived from
# somewhere other than the checkout.
#
# THE EDITION IS READ FROM THE MANIFEST RATHER THAN TYPED HERE. Invoked directly
# the formatter defaults to the 2015 edition and to that edition's style, whatever
# the manifest says, so the number has to be passed. A copy of it in this file
# would be a second declaration of one fact, and the two would disagree at the
# first edition change.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules, print the register, and run the formatter
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# The tracked source files this gate does not ask the formatter about, and the
# reason for each. Beside this script rather than inside it, so that a person
# deciding to exempt a file edits a register instead of a script.
EXEMPTIONS_FILE="$(dirname "$0")/unformatted-paths"

# The manifest the edition is read out of.
MANIFEST="Cargo.toml"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line, as VERDICT<TAB>LINE<TAB>SUBJECT<TAB>DETAIL.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is the
# ordinary answer here, and a pipeline that has to tell "nothing matched" from
# "the scanner broke" one `set -o pipefail` at a time is how a gate ends up
# passing on everything.
# --------------------------------------------------------------------------

parse_exemptions() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*$/) next
      if (line ~ /^[ \t]*#/) next

      # The path is the first field and the reason is everything after it. A
      # split on whitespace would make a reason of several words several fields
      # and lose the sentence, so the cut is made once, by hand.
      work = line
      sub(/^[ \t]+/, "", work)
      i = index(work, " ")
      if (i == 0) {
        printf "REFUSE\t%d\t%s\tcarries a path and no reason\n", FNR, work
        next
      }
      name = substr(work, 1, i - 1)
      reason = substr(work, i + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", reason)
      if (reason == "") {
        printf "REFUSE\t%d\t%s\tcarries a path and no reason\n", FNR, name
        next
      }
      # The path is compared against the shape `git ls-files` writes: from the
      # repository root, forward slashes, no leading dot segment, and a source
      # file rather than a directory. A path written any other way never matches
      # the tracked set, so it would exempt nothing while reading as though it
      # did.
      #
      # Three explicit tests rather than one character class. A backslash inside a
      # bracket expression is an escape in one awk and a member of the class in
      # another, so the class that refuses a Windows-shaped path here would admit
      # one on the runner, and the fixture below would pass on this machine and
      # not on that one.
      first = substr(name, 1, 1)
      if (index(name, "\\") > 0 || first == "." || first == "/" || name !~ /\.rs$/) {
        printf "REFUSE\t%d\t%s\tis not written the way the tracked set is written\n", FNR, name
        next
      }
      printf "ALLOW\t%d\t%s\t%s\n", FNR, name, reason
    }
  '
}

# The edition, out of the `[package]` table and out of no other. A dependency
# table carrying a key of the same spelling is a different key, and taking the
# first match in the file would read it.
manifest_edition() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      if (line ~ /^\[/) { in_package = (line ~ /^\[package\][ \t]*$/); next }
      if (!in_package) next
      if (line !~ /^edition[ \t]*=/) next
      sub(/^edition[ \t]*=[ \t]*/, "", line)
      sub(/#.*$/, "", line)
      gsub(/"/, "", line)
      gsub(/[ \t]/, "", line)
      if (line ~ /^[0-9][0-9][0-9][0-9]$/) { print line; found = 1; exit }
    }
    END { if (!found) exit 1 }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the register or the manifest in
# this tree. A row that judged the real ones would prove the state of the tree on
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

judge_exemption() {
  printf '%s' "$1" | parse_exemptions
}

judge_manifest() {
  printf '%s' "$1" | manifest_edition || echo "REFUSED"
}

selftest() {
  echo "== a path with a reason is admitted =="
  assert_out "passes: a path and a sentence" \
    "$(printf 'ALLOW\t1\ttests/recorded.rs\tthe recorder in #21 writes it and reformatting would move bytes a fixture exists to hold')" \
    "$(judge_exemption 'tests/recorded.rs the recorder in #21 writes it and reformatting would move bytes a fixture exists to hold
')"

  echo "== what is refused =="
  assert_out "bites: a bare path, which is the whole reason this register exists" \
    "$(printf 'REFUSE\t1\tsrc/server/mod.rs\tcarries a path and no reason')" \
    "$(judge_exemption 'src/server/mod.rs
')"
  assert_out "bites: a path with trailing space and nothing after it" \
    "$(printf 'REFUSE\t1\tsrc/server/mod.rs\tcarries a path and no reason')" \
    "$(judge_exemption "$(printf 'src/server/mod.rs   \n')")"
  assert_out "bites: a path written relative to the register rather than to the root" \
    "$(printf 'REFUSE\t1\t./src/lib.rs\tis not written the way the tracked set is written')" \
    "$(judge_exemption './src/lib.rs it looks like the same file and matches nothing
')"
  assert_out "bites: a directory, which would exempt a set nobody enumerated" \
    "$(printf 'REFUSE\t1\tsrc/server\tis not written the way the tracked set is written')" \
    "$(judge_exemption 'src/server the whole module is generated
')"
  assert_out "bites: a path written with the other separator, which no tracked name carries" \
    "$(printf 'REFUSE\t1\tsrc\\server\\mod.rs\tis not written the way the tracked set is written')" \
    "$(judge_exemption 'src\server\mod.rs it is the same file on Windows and matches nothing here
')"

  echo "== what is not a line =="
  assert_out "passes over: a comment, which is most of the register" \
    "" "$(judge_exemption '# Files the formatter is not asked about.
')"
  assert_out "passes over: a blank line" \
    "" "$(judge_exemption '
')"

  echo "== the reason is kept whole =="
  assert_out "passes: a reason of several words is one reason and not several fields" \
    "$(printf 'ALLOW\t1\tsrc/artwork/mod.rs\tit retires when the decoder in #50 replaces the table by hand')" \
    "$(judge_exemption 'src/artwork/mod.rs   it retires when the decoder in #50 replaces the table by hand
')"

  echo "== the edition comes out of the package table =="
  assert_out "reads: the edition of the package" "2024" \
    "$(judge_manifest '[package]
name = "flowfin-core"
edition = "2024"
')"
  assert_out "reads: a value with a comment after it" "2021" \
    "$(judge_manifest '[package]
edition = "2021"  # whatever a comment says
')"
  assert_out "bites: an edition under another table, which is a different key" "REFUSED" \
    "$(judge_manifest '[package]
name = "flowfin-core"

[dependencies.foo]
edition = "2015"
')"
  assert_out "bites: a manifest declaring no edition at all" "REFUSED" \
    "$(judge_manifest '[package]
name = "flowfin-core"
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures format-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local records refusals=0 exempt=0 judged=0 edition tracked subject=""
  local verdict line name detail

  if [ ! -f "$EXEMPTIONS_FILE" ]; then
    echo "::error::${EXEMPTIONS_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi

  # The authority for what exists is the tracked set, not the working tree. A
  # file present on disk and never added is not a file a reader receives.
  tracked="$(git ls-files -- '*.rs')"

  echo "-- the tracked source files this gate does not ask the formatter about"
  records="$(parse_exemptions < "$EXEMPTIONS_FILE")"
  while IFS=$'\t' read -r verdict line name detail; do
    [ -n "${verdict:-}" ] || continue
    case "$verdict" in
      ALLOW)
        # A register line naming a path the tree no longer carries exempts
        # nothing while reading as though it did, so it is refused rather than
        # passed over.
        if printf '%s\n' "$tracked" | grep -qxF -- "$name"; then
          exempt=$((exempt + 1))
          echo "      ${name}: ${detail}"
        else
          refusals=$((refusals + 1))
          echo "::error file=${EXEMPTIONS_FILE},line=${line}::${EXEMPTIONS_FILE}:${line}: ${name} names no tracked file"
          echo "      ${EXEMPTIONS_FILE}:${line}: ${name} names no tracked file"
        fi
        ;;
      REFUSE)
        refusals=$((refusals + 1))
        echo "::error file=${EXEMPTIONS_FILE},line=${line}::${EXEMPTIONS_FILE}:${line}: ${name} ${detail}"
        echo "      ${EXEMPTIONS_FILE}:${line}: ${name} ${detail}"
        ;;
    esac
  done <<RECORDS
$records
RECORDS

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} line(s) of ${EXEMPTIONS_FILE} were refused. A register the gate cannot read is not a register it may skip."
    return 1
  fi
  if [ "$exempt" -eq 0 ]; then
    echo "      none. Every tracked source file below is judged."
  fi
  echo

  echo "-- which edition the formatter was told to read"
  if ! edition="$(manifest_edition < "$MANIFEST")"; then
    echo "::error::${MANIFEST} declares no edition under [package]. Invoked directly the formatter falls back to the 2015 edition and to that edition's style, and this run will not do that silently."
    return 1
  fi
  echo "      ${edition}, read from ${MANIFEST}"
  echo

  echo "-- which formatter judged this run"
  rustfmt --version
  echo

  echo "-- what it read"
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    if printf '%s\n' "$records" | awk -F'\t' -v p="$name" '$1 == "ALLOW" && $3 == p { found = 1 } END { exit !found }'; then
      continue
    fi
    subject="$subject $name"
    judged=$((judged + 1))
    echo "      ${name}"
  done <<TRACKED
$tracked
TRACKED

  if [ "$judged" -eq 0 ]; then
    echo "::error::No tracked source file reached the formatter. A run that judged nothing is not a run that judged everything and found nothing."
    return 1
  fi
  echo
  echo "      ${judged} file(s) judged"
  echo

  echo "-- the verdict"
  # Word splitting on $subject is what this line is for: it holds the list built
  # above, and quoting it would hand the formatter one argument with spaces in it.
  # shellcheck disable=SC2086
  rustfmt --check --edition "$edition" $subject
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether the code compiles. .github/workflows/build.yml is where that is decided, and a formatter parses without building."
  echo "NOT MADE HERE: a lint. That is .github/lint/lint.sh, a different tool with a different verdict."
  echo "NOT MADE HERE: a file the tree does not track. The subject above is git's set, so a file on disk that nobody added is neither judged here nor reported as clean."
  echo "NOT MADE HERE: whether a reason in the register is a good reason. The rule is that one exists, which is decidable; whether it argues anything is what the review is for."
  echo
  echo "Every tracked source file above is written the way the formatter would write it."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
