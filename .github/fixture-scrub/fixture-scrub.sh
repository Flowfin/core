#!/usr/bin/env bash
# What a recorded fixture may carry (#109).
#
# The rules are DATA, in .github/fixture-scrub/values, and this file is the
# loader, the prover and the judge. The split is the same one
# .github/invariants/invariants.sh takes and for the same reason: a rule that
# lives in a register is added by editing a register, with the record it comes
# from and the failure it prevents on the same block.
#
# WHAT THIS JUDGES THAT A PATTERN ALONE CANNOT. A recording is scrubbed by
# replacing a personal value with a synthetic one, and a synthetic account name
# has the shape of an account name, so a shape alone refuses every recording or
# none of them. Each rule therefore locates a candidate with `find` and then
# judges it: under `absent` any candidate is refused, and under `synthetic` a
# candidate is refused unless `allow` admits the WHOLE of it. That is membership
# of a declared set rather than a shape, which is the only property this can
# hold, and docs/decisions/0109-what-a-recorded-fixture-may-carry.md is where it
# is argued.
#
# Every rule proves itself on every run. Each block carries a line that violates
# it and a line that nearly does, and `selftest` judges both against the WHOLE
# rule set: the violation has to produce exactly that rule's id and nothing else,
# and the near miss has to produce nothing.
#
# Verbs:
#   selftest   prove every rule bites its own fixture, alone, and passes the near miss
#   check      load the rule set, print it, and judge the tracked subject
#
# No POSIX character classes and no interval expressions in any awk program in
# this file. The awk on the runner is mawk and the awk on a contributor's machine
# is frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. The `find` and `allow` fields of a rule are a different case:
# they are handed to grep as extended regular expressions, and that is the one
# language a rule author writes in.
#
# A REFUSAL NAMES THE FILE, THE LINE AND THE KIND, AND NEVER THE VALUE. Printing
# the value would copy a token out of a file nobody should have committed into a
# log more people can read than the file. The line number is what a reader opens.

set -euo pipefail

VALUES_FILE="$(dirname "$0")/values"

# Every field a block must carry whatever its treatment. `allow` is conditional
# and is judged separately below, because requiring it here would refuse the
# `absent` blocks, which admit nothing and so have nothing to declare.
REQUIRED_FIELDS="id kind treatment find paths grounds prevents fixture clean"

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

parse_values() {
  awk -v required="$REQUIRED_FIELDS" '
    function reset() {
      delete value
      nfields = 0
      start = 0
    }
    function flush(   i, n, names, missing, id, treatment) {
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
        reset()
        return
      }
      treatment = value["treatment"]
      if (treatment != "absent" && treatment != "synthetic") {
        printf "REFUSE\t%s\t%d\tcarries a treatment that is neither absent nor synthetic, and a third one decides nothing\n", id, start
        reset()
        return
      }
      if (treatment == "synthetic" && (!("allow" in value) || value["allow"] == "")) {
        printf "REFUSE\t%s\t%d\tcarries no allow, and a synthetic treatment with nothing admitted refuses every recording\n", id, start
        reset()
        return
      }
      if (treatment == "absent" && ("allow" in value)) {
        printf "REFUSE\t%s\t%d\tcarries an allow beside an absent treatment, and an absent treatment admits nothing at any value\n", id, start
        reset()
        return
      }
      for (i = 1; i <= n; i++) printf "FIELD\t%s\t%s\t%s\n", id, names[i], value[names[i]]
      if ("allow" in value) printf "FIELD\t%s\tallow\t%s\n", id, value["allow"]
      reset()
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

# The value of one field of one rule.
field_of() {
  printf '%s\n' "$2" | awk -F'\t' -v r="$1" -v f="$3" '$1 == "FIELD" && $2 == r && $3 == f { print $4 }'
}

# Whether the whole of one candidate is admitted by an allow pattern.
admitted() {
  printf '%s' "$1" | grep -qE -- "^($2)$"
}

# Every candidate a pattern locates in a piece of text, one per line.
#
# grep exits 1 when it selected nothing and 2 or more when it could not read what
# it was pointed at, and those two are the pair a `|| true` collapses. A reading
# that did not happen prints exactly like a clean one, which is the failure
# .github/dependencies/dependencies.sh already names for a scanner pointed at a
# moved file, so anything above 1 is returned as a failure rather than as silence.
candidates_in() {
  local text="$1" pattern="$2" found status=0
  found="$(printf '%s\n' "$text" | grep -oE -- "$pattern")" || status=$?
  if [ "$status" -gt 1 ]; then
    return "$status"
  fi
  printf '%s\n' "$found"
}

# Every candidate a pattern locates in one tracked file, as LINE:VALUE.
candidates_in_file() {
  local file="$1" pattern="$2" found status=0
  found="$(grep -noE -- "$pattern" "$file")" || status=$?
  if [ "$status" -gt 1 ]; then
    return "$status"
  fi
  printf '%s\n' "$found"
}

# The ids of every rule in the records that refuse a piece of text, one per
# line, sorted. The whole rule set is applied rather than the one rule being
# proven, which is what makes a fixture prove that its rule bites ALONE.
judge_text() {
  local text="$1" records="$2" id treatment find_pattern allow_pattern candidate found
  while IFS= read -r id; do
    [ -n "${id:-}" ] || continue
    treatment="$(field_of "$id" "$records" treatment)"
    find_pattern="$(field_of "$id" "$records" find)"
    allow_pattern="$(field_of "$id" "$records" allow)"
    found="$(candidates_in "$text" "$find_pattern")"
    while IFS= read -r candidate; do
      [ -n "${candidate:-}" ] || continue
      if [ "$treatment" = "synthetic" ] && admitted "$candidate" "$allow_pattern"; then
        continue
      fi
      echo "$id"
      break
    done <<CANDIDATES
$found
CANDIDATES
  done < <(printf '%s\n' "$records" | awk -F'\t' '$1 == "FIELD" && $3 == "id" { print $4 }' | sort -u) \
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
  printf '%s' "$1" | parse_values
}

# The loader's own rules, proven against text rather than against the register in
# this tree. A case that judged the real register would prove the state of the
# tree on the day it ran, not the rule.
selftest_loader() {
  echo "== the register the loader refuses =="
  assert_out "bites: a block with no stated failure it prevents, which is the condition a rule nobody argued with fails" \
    "$(printf 'REFUSE\ttoken-is-not-a-recorded-value\t1\tcarries no prevents')" \
    "$(judge_register 'id: token-is-not-a-recorded-value
kind: the session token
treatment: absent
find: Token
paths: tests/
grounds: docs/decisions/0071-what-may-leave-through-a-diagnostic-event.md
fixture: a token
clean: TokenGeneration = 4
')"
  assert_out "bites: a block naming no record, so the rule cannot be traced to a decision" \
    "$(printf 'REFUSE\ttoken-is-not-a-recorded-value\t1\tcarries no grounds')" \
    "$(judge_register 'id: token-is-not-a-recorded-value
kind: the session token
treatment: absent
find: Token
paths: tests/
prevents: a credential in a public tree
fixture: a token
clean: TokenGeneration = 4
')"
  assert_out "bites: a synthetic treatment with nothing admitted, which refuses every recording it meets" \
    "$(printf 'REFUSE\taddress-is-not-synthetic\t1\tcarries no allow, and a synthetic treatment with nothing admitted refuses every recording')" \
    "$(judge_register 'id: address-is-not-synthetic
kind: the server address
treatment: synthetic
find: an address
paths: tests/recorded/
grounds: docs/decisions/0068-the-data-locality-position.md
prevents: the address of a household in a public tree
fixture: an address somebody typed
clean: the reserved name
')"
  assert_out "bites: an absent treatment carrying an admitted set, which is two rules written as one" \
    "$(printf 'REFUSE\ttoken-is-not-a-recorded-value\t1\tcarries an allow beside an absent treatment, and an absent treatment admits nothing at any value')" \
    "$(judge_register 'id: token-is-not-a-recorded-value
kind: the session token
treatment: absent
find: Token
allow: Token
paths: tests/
grounds: docs/decisions/0071-what-may-leave-through-a-diagnostic-event.md
prevents: a credential in a public tree
fixture: a token
clean: TokenGeneration = 4
')"
  assert_out "bites: a treatment that is neither, which would load as a rule and judge nothing" \
    "$(printf 'REFUSE\ttoken-is-not-a-recorded-value\t1\tcarries a treatment that is neither absent nor synthetic, and a third one decides nothing')" \
    "$(judge_register 'id: token-is-not-a-recorded-value
kind: the session token
treatment: reduced
find: Token
paths: tests/
grounds: docs/decisions/0071-what-may-leave-through-a-diagnostic-event.md
prevents: a credential in a public tree
fixture: a token
clean: TokenGeneration = 4
')"
  assert_out "bites: several absences at once, each named rather than the first" \
    "$(printf 'REFUSE\t(no id)\t1\tcarries no id, kind, grounds, prevents, clean\n')" \
    "$(judge_register 'treatment: absent
find: Token
paths: tests/
fixture: a token
')"
  assert_out "bites: a line that is not a field at all" \
    "$(printf 'REFUSE\t(unreadable)\t1\tis not a field, and a register the loader cannot read is not one it may skip')" \
    "$(judge_register 'this is prose somebody left in the register
')"
  assert_out "passes over: a comment, which is most of the register" \
    "" "$(judge_register '# What a recorded fixture may carry.
')"
}

# The reading itself, proven in both directions.
#
# The whole leg rests on grep having read what it was pointed at, and the two
# outcomes it reports with an exit code are one apart: nothing selected, and
# nothing read. A run that collapses them prints a page indistinguishable from a
# clean tree, so the difference is asserted here rather than assumed.
selftest_readings() {
  local status
  echo "== a reading that did not happen is not a clean one =="

  status=0
  candidates_in_file "$(dirname "$0")" 'anything' > /dev/null 2>&1 || status=$?
  assert_out "bites: a subject grep could not read comes back as a failure and not as silence" \
    "failed" "$([ "$status" -ne 0 ] && echo failed || echo "passed as clean")"

  status=0
  candidates_in_file "$VALUES_FILE" 'a-string-no-register-carries' > /dev/null 2>&1 || status=$?
  assert_out "passes: a subject grep read and selected nothing in" \
    "read" "$([ "$status" -eq 0 ] && echo read || echo "reported as unreadable")"
}

# Every rule in the real register, proven against its own two lines.
selftest_rules() {
  local records ids id fixture clean verdict
  records="$(parse_values < "$VALUES_FILE")"

  echo "== every rule bites its own fixture, and bites it alone =="
  ids="$(printf '%s\n' "$records" | awk -F'\t' '$1 == "FIELD" && $3 == "id" { print $4 }' | sort -u)"
  for id in $ids; do
    fixture="$(field_of "$id" "$records" fixture)"
    clean="$(field_of "$id" "$records" clean)"

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
  selftest_readings
  echo

  if [ ! -f "$VALUES_FILE" ]; then
    echo "::error::${VALUES_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi
  refused="$(parse_values < "$VALUES_FILE" | awk -F'\t' '$1 == "REFUSE"' | wc -l | tr -d ' ')"
  if [ "$refused" -ne 0 ]; then
    echo "::error::${refused} block(s) of ${VALUES_FILE} were refused by the loader, so no rule below was proven."
    parse_values < "$VALUES_FILE" | awk -F'\t' -v f="$VALUES_FILE" '$1 == "REFUSE" { printf "      %s:%s: %s %s\n", f, $3, $2, $4 }'
    return 1
  fi

  selftest_rules

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local records ids id kind treatment find_pattern allow_pattern paths grounds prevents
  local refusals=0 rules=0 readings=0 subject file hits hit line candidate

  if [ ! -f "$VALUES_FILE" ]; then
    echo "::error::${VALUES_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi

  records="$(parse_values < "$VALUES_FILE")"
  if printf '%s\n' "$records" | grep -q '^REFUSE'; then
    printf '%s\n' "$records" | awk -F'\t' -v f="$VALUES_FILE" '$1 == "REFUSE" { printf "::error file=%s,line=%s::%s:%s: %s %s\n", f, $3, f, $3, $2, $4 }'
    echo "::error::${VALUES_FILE} carries a block the loader refused. A rule with no stated failure it prevents is a rule nobody argued with."
    return 1
  fi

  ids="$(printf '%s\n' "$records" | awk -F'\t' '$1 == "FIELD" && $3 == "id" { print $4 }' | sort -u)"

  echo "-- the rules this gate applies"
  for id in $ids; do
    rules=$((rules + 1))
    kind="$(field_of "$id" "$records" kind)"
    treatment="$(field_of "$id" "$records" treatment)"
    prevents="$(field_of "$id" "$records" prevents)"
    grounds="$(field_of "$id" "$records" grounds)"
    echo "      ${id}"
    echo "        about:     ${kind}"
    echo "        treatment: ${treatment}"
    echo "        prevents:  ${prevents}"
    echo "        grounds:   ${grounds}"
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
    kind="$(field_of "$id" "$records" kind)"
    treatment="$(field_of "$id" "$records" treatment)"
    find_pattern="$(field_of "$id" "$records" find)"
    allow_pattern="$(field_of "$id" "$records" allow)"
    paths="$(field_of "$id" "$records" paths)"

    # The authority for what exists is git's set. A file on disk that nobody
    # added is not judged here and is not reported clean either.
    subject="$(git ls-files -- "$paths")"
    echo "      ${id}: ${paths}"

    if [ -z "$subject" ]; then
      echo "        NO SUBJECT: no tracked file under that prefix, so this rule judged nothing on this run."
      continue
    fi

    while IFS= read -r file; do
      [ -n "${file:-}" ] || continue
      readings=$((readings + 1))
      if ! hits="$(candidates_in_file "$file" "$find_pattern")"; then
        echo "::error file=${file}::${id}: ${file} could not be read, so this rule judged it and reported nothing. A reading that did not happen is not a clean one."
        return 1
      fi
      while IFS= read -r hit; do
        [ -n "${hit:-}" ] || continue
        line="${hit%%:*}"
        candidate="${hit#*:}"
        if [ "$treatment" = "synthetic" ] && admitted "$candidate" "$allow_pattern"; then
          continue
        fi
        refusals=$((refusals + 1))
        echo "::error file=${file},line=${line}::${id}: ${file} line ${line} carries ${kind}, which a scrubbed recording does not."
        echo "        REFUSED: ${file}:${line}: ${kind}"
      done <<HITS
$hits
HITS
    done <<SUBJECT
$subject
SUBJECT
    echo "        $(printf '%s\n' "$subject" | wc -l | tr -d ' ') file(s) read"
  done
  echo
  echo "      ${readings} file reading(s) across ${rules} rule(s)"
  echo

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} value(s) above are not what a scrubbed recording carries. Each rule prints the failure it prevents and the record it comes from, and tests/recorded/README.md is the procedure that produces a value which passes."
    return 1
  fi

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether a value that IS in the admitted set is the right one. Membership is decidable and correctness is a judgement, which is the trade docs/decisions/0109-what-a-recorded-fixture-may-carry.md takes and states."
  echo "NOT MADE HERE: a personal value whose kind is not one of the rules above. The list in docs/decisions/0068-the-data-locality-position.md is closed by a question a contributor answers rather than by a set of shapes, so a title, an account name or a viewing history in a recording walks past every rule here. That is the bound on this leg rather than an omission somebody fills in with another block."
  echo "NOT MADE HERE: anything outside the prefixes printed above, and anything the tree does not track. A rule with no subject says so rather than passing quietly."
  echo "NOT MADE HERE: the history. This judges the tree at the commit it was handed, and a value removed in a later commit is still in the history, which is why this leg is here before the first recording rather than after it."
  echo
  echo "Every rule above was applied to its subject and refused nothing."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
