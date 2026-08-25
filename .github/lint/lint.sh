#!/usr/bin/env bash
# The analyser the language ships, at the strictest setting this tree can hold
# (#17).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement the other two legs of this gate already use.
#
# What the check is for: static analysis is cheapest at the beginning, when there
# is nothing to fix. Added after a codebase exists it is a backlog, a backlog gets
# an allowlist, and an allowlist gets forgotten. This tree has no backlog yet, so
# the strict setting costs nothing today and cannot be afforded later.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules, print the register, and run the analyser
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# The lints this gate does not refuse, and the reason each one is not refused. The
# file is beside this one rather than inside it so that a person deciding whether
# to add an exclusion edits a register instead of a script.
EXCLUSIONS_FILE="$(dirname "$0")/excluded-lints"

# The groups this gate denies. Written here rather than in the workflow so that a
# person running the verb by hand runs the same setting the gate runs.
#
# `all` is the analyser's default set and `pedantic` is the one that costs
# something to adopt late, which is the whole argument of #17. `cargo` is here
# because the manifest is as much a subject as the source: a dependency added
# under a wrong feature or a duplicate version is exactly the kind of thing #103
# is a rule about, and it is cheaper to have the analyser say so.
#
# TWO GROUPS ARE DELIBERATELY NOT DENIED, AND NEITHER IS AN EXCLUSION IN THE
# REGISTER, because the register holds lints rather than groups.
#
# `nursery` holds lints the analyser's own authors describe as still under
# development. Denying them means a toolchain upgrade can redden this gate for a
# lint nobody finished, and the version this repository builds with is not even
# pinned yet, which is #14. It is worth revisiting once it is.
#
# `restriction` is not a set of defects at all. It is a menu of blanket
# prohibitions, several of which contradict each other, and its own documentation
# says enabling the whole group is a mistake. Individual members of it may be
# denied later, one at a time, with the reason beside each.
#
# NAMED DENIED_GROUPS RATHER THAN GROUPS, and that is a repair rather than a
# preference. `GROUPS` is a variable bash maintains itself, holding the numeric
# group ids of whoever is running; an assignment to it is ignored, and the first
# run of this file passed the analyser one of those numbers instead of a lint
# group. It was found by running the verb rather than by reading it.
DENIED_GROUPS="clippy::all clippy::pedantic clippy::cargo"

# --------------------------------------------------------------------------
# Rules. The register is read on stdin and each line becomes one record, as
# VERDICT<TAB>LINE<TAB>SUBJECT<TAB>DETAIL.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is the
# ordinary answer here, and a pipeline that has to tell "nothing matched" from
# "the scanner broke" one `set -o pipefail` at a time is how a gate ends up
# passing on everything.
# --------------------------------------------------------------------------

parse_exclusions() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*$/) next
      if (line ~ /^[ \t]*#/) next

      # The name is the first field and the reason is everything after it. A
      # split on whitespace would make a reason of several words several fields
      # and lose the sentence, so the cut is made once, by hand.
      work = line
      sub(/^[ \t]+/, "", work)
      i = index(work, " ")
      if (i == 0) {
        printf "REFUSE\t%d\t%s\tcarries a name and no reason\n", FNR, work
        next
      }
      name = substr(work, 1, i - 1)
      reason = substr(work, i + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", reason)
      if (reason == "") {
        printf "REFUSE\t%d\t%s\tcarries a name and no reason\n", FNR, name
        next
      }
      # The name is compared against the shape the analyser writes, so that a
      # bare lint with no namespace, or a group where a lint belongs, is caught
      # here rather than being passed to the analyser and silently ignored.
      if (name !~ /^clippy::[a-z_]+$/) {
        printf "REFUSE\t%d\t%s\tis not written the way the analyser writes a lint name\n", FNR, name
        next
      }
      printf "ALLOW\t%d\t%s\t%s\n", FNR, name, reason
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the register in this tree. A row
# that judged the real register would prove the state of the tree on the day it
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

judge_fixture() {
  printf '%s' "$1" | parse_exclusions
}

selftest() {
  echo "== a name with a reason is admitted =="
  assert_out "passes: a name and a sentence" \
    "$(printf 'ALLOW\t1\tclippy::too_many_lines\tthe fake server in #21 has one long table and splitting it would hide the shape')" \
    "$(judge_fixture 'clippy::too_many_lines the fake server in #21 has one long table and splitting it would hide the shape
')"

  echo "== what is refused =="
  assert_out "bites: a bare name, which is the whole reason this register exists" \
    "$(printf 'REFUSE\t1\tclippy::too_many_lines\tcarries a name and no reason')" \
    "$(judge_fixture 'clippy::too_many_lines
')"
  assert_out "bites: a name with trailing space and nothing after it" \
    "$(printf 'REFUSE\t1\tclippy::too_many_lines\tcarries a name and no reason')" \
    "$(judge_fixture "$(printf 'clippy::too_many_lines   \n')")"
  assert_out "bites: a lint written without the namespace the analyser uses" \
    "$(printf 'REFUSE\t1\ttoo_many_lines\tis not written the way the analyser writes a lint name')" \
    "$(judge_fixture 'too_many_lines it reads like a lint and the analyser would ignore it
')"
  assert_out "bites: a compiler lint, which this register is not for" \
    "$(printf 'REFUSE\t1\tunused_imports\tis not written the way the analyser writes a lint name')" \
    "$(judge_fixture 'unused_imports the build check owns compiler warnings, not this one
')"

  echo "== what is not a line =="
  assert_out "passes over: a comment, which is most of the register" \
    "" "$(judge_fixture '# Lints this gate does not refuse.
')"
  assert_out "passes over: a blank line" \
    "" "$(judge_fixture '
')"

  echo "== the reason is kept whole =="
  assert_out "passes: a reason of several words is one reason and not several fields" \
    "$(printf 'ALLOW\t1\tclippy::missing_errors_doc\tit retires when the vocabulary in #4 lands and every failure has a name to document')" \
    "$(judge_fixture 'clippy::missing_errors_doc   it retires when the vocabulary in #4 lands and every failure has a name to document
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures lint-register fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local records refusals=0 allowed=0 args="" verdict line name detail

  if [ ! -f "$EXCLUSIONS_FILE" ]; then
    echo "::error::${EXCLUSIONS_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi

  records="$(parse_exclusions < "$EXCLUSIONS_FILE")"

  echo "-- the lints this gate does not refuse"
  while IFS=$'\t' read -r verdict line name detail; do
    [ -n "${verdict:-}" ] || continue
    case "$verdict" in
      ALLOW)
        allowed=$((allowed + 1))
        args="$args -A $name"
        echo "      ${name}: ${detail}"
        ;;
      REFUSE)
        refusals=$((refusals + 1))
        echo "::error file=${EXCLUSIONS_FILE},line=${line}::${EXCLUSIONS_FILE}:${line}: ${name} ${detail}"
        echo "      ${EXCLUSIONS_FILE}:${line}: ${name} ${detail}"
        ;;
    esac
  done <<EOF
$records
EOF

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} line(s) of ${EXCLUSIONS_FILE} were refused. A register the gate cannot read is not a register it may skip."
    return 1
  fi
  if [ "$allowed" -eq 0 ]; then
    echo "      none. Every lint in the groups below is refused."
  fi
  echo

  echo "-- what the analyser was told to refuse"
  echo "      groups denied: ${DENIED_GROUPS}"
  echo "      every remaining warning is an error"
  echo

  echo "-- which analyser judged this run"
  cargo clippy --version
  echo

  echo "-- the analysis"
  local deny=""
  for g in $DENIED_GROUPS; do deny="$deny -D $g"; done
  # Word splitting on $deny and $args is what this line is for: each holds a list
  # of arguments built above, and quoting either would hand the analyser one
  # argument with spaces in it.
  # shellcheck disable=SC2086
  cargo clippy --all-targets -- -D warnings $deny $args
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: a compiler warning. .github/workflows/build.yml refuses those with the same command a contributor runs, and a second implementation of one rule is two things that agree until they drift."
  echo "NOT MADE HERE: the formatting. That is #18 and is a different tool with a different verdict."
  echo "NOT MADE HERE: whether a reason in the register is a good reason. The rule is that one exists, which is decidable; whether it argues anything is what the review is for."
  echo
  echo "Every lint the groups above carry was refused, outside the register printed with it."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
