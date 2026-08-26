#!/usr/bin/env bash
# The core's own language is analysed, and an actionable finding fails the gate
# (#81).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHAT THIS OWNS AND WHAT IT DOES NOT. The analysis is the code-scanning
# surface's own, run by the action in `.github/workflows/codeql.yml`, and the
# queries are that surface's rather than this repository's. What this file owns is
# the verdict: the analysis writes a SARIF file, this reads it, and a finding that
# is not excused by name fails the run. That division is deliberate. The action
# uploads findings and does not fail a build on them, so a repository that stops
# at the upload has alerts nobody is required to answer, and #81 asks for a gate
# rather than a report.
#
# A RUN THAT ANALYSED NOTHING IS REFUSED RATHER THAN READ AS CLEAN. A SARIF file
# carrying no run, or a run whose tool declares no rule at all, is a query set
# that never loaded, and it exits with a page indistinguishable from a clean
# analysis. So the counts are read out of the file, printed, and refused when
# either is zero.
#
# A rule this gate does not refuse is written in `.github/codeql/excluded-rules`
# with the reason it is not refused, and a line there carrying an identifier and
# no reason is itself refused. Every run prints that file with its reasons, so an
# exclusion is read beside the verdict rather than found later by somebody
# auditing. An exclusion is a debt and each reason says what retires it.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      read the SARIF the analysis wrote, judge the register, and refuse
#
# `jq` reads the SARIF. The file is JSON produced by somebody else's tool, and a
# JSON reader written in awk here would be a second parser to keep in step with a
# format this repository does not own. `.github/shell-analysis/shell-analysis.sh`
# already reaches for the same tool for the same reason, and the workflow prints
# which build of it ran.
#
# No POSIX character classes and no interval expressions in any awk pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it.

set -euo pipefail

# The rules this gate does not refuse, beside this script rather than inside it,
# so that a person excusing a rule edits a register instead of a script.
EXCLUSIONS_FILE="$(dirname "$0")/excluded-rules"

# The fixture SARIF files. Never the file a real run wrote: a fixture that judged
# a real analysis would prove the state of the tree on the day it ran, not the
# rule.
FIXTURES="$(dirname "$0")/fixtures"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line.
# --------------------------------------------------------------------------

# Every finding in a SARIF file, as RULE<TAB>LEVEL<TAB>URI<TAB>LINE.
#
# The rule identifier is taken from the result's own `ruleId` and falls back to
# the rule's index into the tool's rule set, because a result naming no rule is
# still a finding and dropping it would be the quietest way to lose one.
findings() {
  jq -r '
    [ .runs[]? | .results[]? ] | .[] |
    [ (.ruleId // "(no rule id)")
    , (.level // "warning")
    , (.locations[0].physicalLocation.artifactLocation.uri // "(no file)")
    , ((.locations[0].physicalLocation.region.startLine // 0) | tostring)
    ] | @tsv
  '
}

# What a SARIF file says it did, as RUNS<TAB>RULES<TAB>RESULTS.
#
# The rule count is the union of the ones the driver declares and the ones its
# extensions declare, because a query pack loaded as an extension puts its rules
# there and reading only the driver would report a full analysis as an empty one.
accounting() {
  jq -r '
    [ (.runs // [] | length)
    , ([ .runs[]? | (.tool.driver.rules // []), (.tool.extensions[]?.rules // []) ] | flatten | length)
    , ([ .runs[]? | .results[]? ] | length)
    ] | @tsv
  '
}

# The exclusion register, as VERDICT<TAB>LINE<TAB>RULE<TAB>REASON.
#
# A line carrying an identifier and no reason is REFUSE, because a bare
# identifier is a rule somebody turned off and nobody argued with. Comments and
# blank lines are not entries.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is the
# ordinary answer for an empty register, and a pipeline that has to tell that
# from a scanner that broke one `set -o pipefail` at a time is how a gate ends up
# passing on everything.
parse_exclusions() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == "" || line ~ /^#/) next
      rule = line
      reason = ""
      i = index(line, " ")
      j = index(line, "\t")
      if (j > 0 && (i == 0 || j < i)) i = j
      if (i > 0) {
        rule = substr(line, 1, i - 1)
        reason = substr(line, i + 1)
        sub(/^[ \t]+/, "", reason)
      }
      if (reason == "") { printf "REFUSE\t%d\t%s\t%s\n", FNR, rule, "carries a name and no reason" }
      else { printf "ALLOW\t%d\t%s\t%s\n", FNR, rule, reason }
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text. A row that judged a real analysis would
# prove the state of the tree on the day it ran, not the rule.
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

judge_findings() {
  findings < "$1"
}

judge_accounting() {
  accounting < "$1"
}

judge_register() {
  printf '%s\n' "$1" | parse_exclusions
}

selftest() {
  echo "== a finding is read out of the file the analysis wrote =="
  assert_out "reads: the rule, the level, the file and the line" \
    "$(printf 'rust/a-finding\terror\tsrc/session/mod.rs\t12')" \
    "$(judge_findings "$FIXTURES/one-finding.sarif")"
  assert_out "reads: the one-change neighbour, the same analysis with nothing found" \
    "" \
    "$(judge_findings "$FIXTURES/no-finding.sarif")"
  assert_out "bites: a result naming no rule, which is still a finding" \
    "$(printf '(no rule id)\twarning\tsrc/session/mod.rs\t12')" \
    "$(judge_findings "$FIXTURES/finding-without-a-rule-id.sarif")"

  echo "== what the file says the analysis did =="
  assert_out "reads: one run, the rules its extension declares, one result" \
    "$(printf '1\t2\t1')" \
    "$(judge_accounting "$FIXTURES/one-finding.sarif")"
  assert_out "reads: the same run with nothing found, and the rule set unchanged" \
    "$(printf '1\t2\t0')" \
    "$(judge_accounting "$FIXTURES/no-finding.sarif")"
  assert_out "bites: a run whose tool declares no rule, which is a query set that never loaded" \
    "$(printf '1\t0\t0')" \
    "$(judge_accounting "$FIXTURES/no-rules-loaded.sarif")"
  assert_out "bites: a file carrying no run at all" \
    "$(printf '0\t0\t0')" \
    "$(judge_accounting "$FIXTURES/no-run.sarif")"

  echo "== the register refuses a rule somebody turned off and nobody argued with =="
  assert_out "reads: an identifier with the reason after it" \
    "$(printf 'ALLOW\t1\trust/a-finding\tit retires when the query stops firing on a fixture')" \
    "$(judge_register 'rust/a-finding it retires when the query stops firing on a fixture')"
  assert_out "bites: an identifier and nothing after it" \
    "$(printf 'REFUSE\t1\trust/a-finding\tcarries a name and no reason')" \
    "$(judge_register 'rust/a-finding')"
  assert_out "bites: an identifier followed by whitespace and nothing else" \
    "$(printf 'REFUSE\t1\trust/a-finding\tcarries a name and no reason')" \
    "$(judge_register "$(printf 'rust/a-finding   \n')")"
  assert_out "passes over: a comment, which is most of the register" \
    "" \
    "$(judge_register '# a comment naming rust/a-finding')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures code-scanning-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
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
  local sarif="${1:-}" register acct runs rules results refused=0 excused=0

  if [ -z "$sarif" ]; then
    echo "usage: $0 check <sarif-file>" >&2
    return 2
  fi

  echo "-- the rules this gate does not refuse"
  if [ ! -f "$EXCLUSIONS_FILE" ]; then
    echo "::error::${EXCLUSIONS_FILE} does not exist. An absent register and an empty one read the same in a verdict, so this refuses rather than treating a missing file as nothing excused."
    return 1
  fi
  register="$(parse_exclusions < "$EXCLUSIONS_FILE")"
  if [ -z "$register" ]; then
    echo "      none. Every finding the analysis reports fails this run."
  else
    printf '%s\n' "$register" | while IFS=$'\t' read -r verdict at rule reason; do
      printf '      %s  line %s  %s  %s\n' "$verdict" "$at" "$rule" "$reason"
    done
  fi
  if printf '%s\n' "$register" | grep -q '^REFUSE'; then
    echo "::error::${EXCLUSIONS_FILE} carries an entry with no reason after it. A bare identifier is a rule somebody turned off and nobody argued with, so the register is refused before any finding is judged."
    return 1
  fi
  echo

  if [ ! -f "$sarif" ]; then
    echo "::error::${sarif} was not written, so no analysis reached this step. A run with no file is refused rather than passed, because an absent set of findings and an empty one read the same in a verdict."
    return 1
  fi

  acct="$(accounting < "$sarif")"
  IFS=$'\t' read -r runs rules results <<ACCOUNTING
$acct
ACCOUNTING

  echo "-- what the analysis says it did"
  say "${runs} run(s), ${rules} rule(s) loaded, ${results} finding(s) reported."
  echo

  if [ "$runs" -eq 0 ]; then
    echo "::error::${sarif} carries no analysis run at all. A file with no run exits with a page that reads exactly like an analysis that found nothing."
    return 1
  fi

  if [ "$rules" -eq 0 ]; then
    echo "::error::The analysis loaded no rule, so it asked nothing of this tree. A query set that never loaded reports zero findings, which is the failure this count exists for."
    return 1
  fi

  echo "-- what it found"
  if [ "$results" -eq 0 ]; then
    echo "      nothing."
  else
    while IFS=$'\t' read -r rule level uri at; do
      [ -z "$rule" ] && continue
      if printf '%s\n' "$register" | awk -F'\t' -v r="$rule" '$3 == r { found = 1 } END { exit !found }'; then
        printf '      EXCUSED  %s  %s:%s  (%s)\n' "$rule" "$uri" "$at" "$level"
        excused=$((excused + 1))
      else
        printf '      REFUSED  %s  %s:%s  (%s)\n' "$rule" "$uri" "$at" "$level"
        refused=$((refused + 1))
      fi
    done <<FINDINGS
$(findings < "$sarif")
FINDINGS
  fi
  echo

  if [ "$refused" -ne 0 ]; then
    say "${refused} finding(s) refused, ${excused} excused by the register."
    echo "::error::${refused} finding(s) the register does not excuse. Fix them, or excuse the rule in ${EXCLUSIONS_FILE} with the reason and what retires it, which every run then prints beside its verdict."
    return 1
  fi

  echo "-- what this run did not read"
  echo "NOT MADE HERE: the choice of queries. The set belongs to the code-scanning surface rather than to this repository, and the count above says how many of its rules loaded rather than which. The step before this one in .github/workflows/codeql.yml prints their identifiers."
  echo "NOT MADE HERE: whether a query exists for a defect this tree could carry. A clean run is a run in which the loaded set found nothing, not a tree in which there is nothing to find."
  echo "NOT MADE HERE: the shell this gate runs, which is .github/shell-analysis/shell-analysis.sh, and the workflow files, which are .github/workflows/zizmor.yml. This leg reads the core's own language and nothing else."
  echo
  echo "The analysis loaded ${rules} rule(s) over this tree and reported ${results} finding(s), ${excused} of them excused by name."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check "${2:-}" ;;
  *)        echo "usage: $0 selftest|check <sarif-file>" >&2; exit 2 ;;
esac
