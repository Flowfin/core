#!/usr/bin/env bash
# The invariants a compiler cannot see (#82).
#
# The rules are DATA, in .github/invariants/rules, and this file is the loader,
# the prover and the judge. That split is the point of the check rather than a
# style: a rule that lives in a register is added by editing a register, with the
# record it comes from and the failure it prevents on the same block, and a rule
# that lives in a script is added by editing a script.
#
# Every rule proves itself on every run. Each block carries a line of source that
# violates it and a line that nearly does, and `selftest` judges both against the
# WHOLE rule set: the violation has to produce exactly that rule's id and nothing
# else, and the near miss has to produce nothing. A rule that reddens a second
# rule's fixture is a pattern that is about to refuse honest work.
#
# WHAT A PATTERN CAN AND CANNOT BE ASKED. This check reads source text, so it
# holds only rules whose forbidden side can be written down. Which of #82's four
# seeds could not be, and why, is on #82 and in the register's own header rather
# than restated here.
#
# THE SUBJECT IS NO LONGER SOURCE TEXT ALONE. #77's rule over the dependency graph
# reads the committed lockfile, which is the resolved graph as a tracked file, so
# `paths` names a file there rather than a directory. Nothing in the loader or the
# judge changed for it: the subject was always git's set, and a rule naming one
# tracked file is the same rule as one naming a prefix.
#
# Verbs:
#   selftest   prove every rule bites its own fixture, alone, and passes the near miss
#   check      load the rule set, print it, and judge the tracked subject
#
# No POSIX character classes and no interval expressions in any pattern in this
# file. The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.
#
# The `pattern` field of a rule is a different case: it is handed to grep rather
# than to awk, as an extended regular expression, and that is the one language a
# rule author writes in.

set -euo pipefail

RULES_FILE="$(dirname "$0")/rules"

# Every field a block must carry. `except` is deliberately not here: a rule with
# no exception is the ordinary case, and requiring the field would put an empty
# one on every block.
REQUIRED_FIELDS="id pattern paths grounds prevents fixture clean"

# --------------------------------------------------------------------------
# The loader.
#
# Reads the register on stdin and writes one line per block, as
# VERDICT<TAB>ID<TAB>FIELD<TAB>VALUE for a loaded field, or
# REFUSE<TAB>ID<TAB>(line)<TAB>reason for a block that cannot be loaded.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is a verdict
# this loader has to tell apart from a register it could not read.
# --------------------------------------------------------------------------

parse_rules() {
  awk -v required="$REQUIRED_FIELDS" '
    function flush(   i, n, names, missing, id) {
      if (nfields == 0) return
      id = value["id"]
      if (id == "") id = "(no id)"
      n = split(required, names, " ")
      missing = ""
      for (i = 1; i <= n; i++) {
        if (!(names[i] in value) || value[names[i]] == "") {
          missing = missing (missing == "" ? "" : ", ") names[i]
        }
      }
      if (missing != "") {
        printf "REFUSE\t%s\t%d\tcarries no %s\n", id, start, missing
      } else {
        for (i = 1; i <= n; i++) printf "FIELD\t%s\t%s\t%s\n", id, names[i], value[names[i]]
        if ("except" in value) printf "FIELD\t%s\texcept\t%s\n", id, value["except"]
      }
      delete value
      nfields = 0
      start = 0
    }
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*#/) next
      if (line ~ /^[ \t]*$/) { flush(); next }
      i = index(line, ":")
      if (i == 0) {
        printf "REFUSE\t(unreadable)\t%d\tis not a field, and a register the loader cannot read is not one it may skip\n", FNR
        next
      }
      name = substr(line, 1, i - 1)
      val = substr(line, i + 1)
      sub(/^[ \t]+/, "", val)
      if (nfields == 0) start = FNR
      value[name] = val
      nfields = nfields + 1
    }
    END { flush() }
  '
}

# The ids a piece of text trips, one per line, sorted. The whole rule set is
# applied rather than the one rule being proven, which is what makes a fixture
# prove that its rule bites ALONE.
judge_text() {
  local text="$1" records="$2" id pattern
  while IFS=$'\t' read -r id pattern; do
    [ -n "${id:-}" ] || continue
    if printf '%s\n' "$text" | grep -qE -- "$pattern"; then
      echo "$id"
    fi
  done < <(printf '%s\n' "$records" | awk -F'\t' '$1 == "FIELD" && $3 == "pattern" { print $2 "\t" $4 }') \
    | sort
}

# --------------------------------------------------------------------------
# selftest
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

judge_register() {
  printf '%s' "$1" | parse_rules
}

# The loader's own rules, proven against text rather than against the register in
# this tree. A row that judged the real register would prove the state of the
# tree on the day it ran, not the rule.
selftest_loader() {
  echo "== the register the loader refuses =="
  assert_out "bites: a block with no stated failure it prevents, which is #82's own condition" \
    "$(printf 'REFUSE\tno-text-output\t1\tcarries no prevents')" \
    "$(judge_register 'id: no-text-output
pattern: println!
paths: src/
grounds: docs/decisions/0003-what-the-core-does-not-do.md
fixture: println!("x");
clean: /// nothing
')"
  assert_out "bites: a block naming no record, so the rule cannot be traced to a decision" \
    "$(printf 'REFUSE\tno-text-output\t1\tcarries no grounds')" \
    "$(judge_register 'id: no-text-output
pattern: println!
paths: src/
prevents: a core that logs
fixture: println!("x");
clean: /// nothing
')"
  assert_out "bites: a block with no pattern, which would judge nothing while reading as a rule" \
    "$(printf 'REFUSE\tno-text-output\t1\tcarries no pattern')" \
    "$(judge_register 'id: no-text-output
paths: src/
grounds: docs/decisions/0003-what-the-core-does-not-do.md
prevents: a core that logs
fixture: println!("x");
clean: /// nothing
')"
  assert_out "bites: a field name and nothing after it, which is an empty rule wearing a full one" \
    "$(printf 'REFUSE\tno-text-output\t1\tcarries no prevents')" \
    "$(judge_register 'id: no-text-output
pattern: println!
paths: src/
grounds: docs/decisions/0003-what-the-core-does-not-do.md
prevents:
fixture: println!("x");
clean: /// nothing
')"
  assert_out "bites: several absences at once, each named rather than the first" \
    "$(printf 'REFUSE\t(no id)\t1\tcarries no id, grounds, prevents, clean\n')" \
    "$(judge_register 'pattern: println!
paths: src/
fixture: println!("x");
')"
  assert_out "bites: a line that is not a field at all" \
    "$(printf 'REFUSE\t(unreadable)\t1\tis not a field, and a register the loader cannot read is not one it may skip')" \
    "$(judge_register 'this is prose somebody left in the register
')"
  assert_out "passes over: a comment, which is most of the register" \
    "" "$(judge_register '# The invariants a compiler cannot see.
')"
}

# Every rule in the real register, proven against its own two lines.
selftest_rules() {
  local records ids id fixture clean verdict
  records="$(parse_rules < "$RULES_FILE")"

  echo "== every rule bites its own fixture, and bites it alone =="
  ids="$(printf '%s\n' "$records" | awk -F'\t' '$1 == "FIELD" && $3 == "id" { print $4 }' | sort -u)"
  for id in $ids; do
    fixture="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "fixture" { print $4 }')"
    clean="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "clean" { print $4 }')"

    verdict="$(judge_text "$fixture" "$records")"
    assert_out "bites: ${id}" "$id" "$verdict"

    verdict="$(judge_text "$clean" "$records")"
    assert_out "passes: the near miss beside ${id}" "" "$verdict"
  done
}

selftest() {
  local refused
  selftest_loader
  echo

  if [ ! -f "$RULES_FILE" ]; then
    echo "::error::${RULES_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi
  refused="$(parse_rules < "$RULES_FILE" | awk -F'\t' '$1 == "REFUSE"' | wc -l | tr -d ' ')"
  if [ "$refused" -ne 0 ]; then
    echo "::error::${refused} block(s) of ${RULES_FILE} were refused by the loader, so no rule below was proven."
    parse_rules < "$RULES_FILE" | awk -F'\t' '$1 == "REFUSE" { printf "      %s:%s: %s %s\n", "'"$RULES_FILE"'", $3, $2, $4 }'
    return 1
  fi

  selftest_rules

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures invariant fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local records ids id pattern paths except grounds prevents
  local refusals=0 rules=0 subject hits

  if [ ! -f "$RULES_FILE" ]; then
    echo "::error::${RULES_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi

  records="$(parse_rules < "$RULES_FILE")"
  if printf '%s\n' "$records" | grep -q '^REFUSE'; then
    printf '%s\n' "$records" | awk -F'\t' -v f="$RULES_FILE" '$1 == "REFUSE" { printf "::error file=%s,line=%s::%s:%s: %s %s\n", f, $3, f, $3, $2, $4 }'
    echo "::error::${RULES_FILE} carries a block the loader refused. A rule with no stated failure it prevents is a rule nobody argued with."
    return 1
  fi

  ids="$(printf '%s\n' "$records" | awk -F'\t' '$1 == "FIELD" && $3 == "id" { print $4 }' | sort -u)"

  echo "-- the rules this gate applies"
  for id in $ids; do
    rules=$((rules + 1))
    prevents="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "prevents" { print $4 }')"
    grounds="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "grounds" { print $4 }')"
    echo "      ${id}"
    echo "        prevents: ${prevents}"
    echo "        grounds:  ${grounds}"
  done
  if [ "$rules" -eq 0 ]; then
    echo "::error::The register loaded no rule at all. A run with no rule in it passes everything."
    return 1
  fi
  echo
  echo "      ${rules} rule(s)"
  echo

  echo "-- the subject"
  for id in $ids; do
    pattern="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "pattern" { print $4 }')"
    paths="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "paths" { print $4 }')"
    except="$(printf '%s\n' "$records" | awk -F'\t' -v r="$id" '$1 == "FIELD" && $2 == r && $3 == "except" { print $4 }')"

    # The authority for what exists is git's set. A file on disk that nobody
    # added is not judged here and is not reported clean either.
    if [ -n "$except" ]; then
      subject="$(git ls-files -- "$paths" | grep -v "^${except}" || true)"
      echo "      ${id}: ${paths} except ${except}"
    else
      subject="$(git ls-files -- "$paths")"
      echo "      ${id}: ${paths}"
    fi

    if [ -z "$subject" ]; then
      echo "        NO SUBJECT: no tracked file under that prefix, so this rule judged nothing on this run."
      continue
    fi

    # xargs is not used: a path list handed to grep through one is a second
    # quoting problem, and the set here is small enough to pass whole.
    # shellcheck disable=SC2086
    hits="$(grep -nE -- "$pattern" $subject || true)"
    if [ -n "$hits" ]; then
      while IFS= read -r line; do
        [ -n "$line" ] || continue
        refusals=$((refusals + 1))
        echo "::error::${id}: ${line}"
        echo "        REFUSED: ${line}"
      done <<HITS
$hits
HITS
    else
      echo "        clean"
    fi
  done
  echo

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} line(s) broke an invariant above. Each rule prints the failure it prevents and the record it comes from; argue with the record rather than with the pattern."
    return 1
  fi

  echo "-- what this run did not read"
  echo "NOT MADE HERE: anything whose forbidden side cannot be written as a pattern. docs/decisions/0003-what-the-core-does-not-do.md says of the drawing boundary in so many words that it cannot be expressed as data, and that is unchanged."
  echo "NOT MADE HERE: that boundary itself, although three rules above are grounded in the record that draws it. Each holds a list of what has been named - a set of dependency names, a set of view words, one trait - so a crossing written in a name nobody listed walks past all three, and a green run here is not a run that found the boundary uncrossed."
  echo "NOT MADE HERE: whether a rule is a good rule. The register requires a record and a stated failure, which is decidable; whether the rule follows from the record is what the review is for."
  echo "NOT MADE HERE: a file the tree does not track, and a prefix nothing is tracked under. A rule with no subject says so above rather than passing quietly."
  echo
  echo "Every rule above was applied to its subject and refused nothing."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
