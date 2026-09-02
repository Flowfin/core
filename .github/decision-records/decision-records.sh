#!/usr/bin/env bash
# A narrowing is named in both directions (#267).
#
# 0001 fixes that a record is added or superseded and never edited in place, and
# permits a pointer to a later record that goes further on a case the record
# already names. 0267 decides the shape that pointer takes where the later record
# narrows one clause of an earlier one: a `Narrowed-by:` line in the narrowed
# record's header and a `Narrows:` line in the narrowing record's, each naming the
# other and each naming the clause.
#
# What this check is for: a pointer only one side carries rots the first time a
# record is renumbered or withdrawn, and it rots in silence. That is the same
# defect class the field exists to fix, one level up - a reader who does not
# follow the pointer reads a clause wider than the rule in force, and a reader who
# follows a pointer nothing answers learns nothing at all.
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules to every tracked decision record, and refuse
#
# `check` reads the repository through `git ls-files`, so the authority for what
# exists is the set of tracked records rather than the working tree. A record
# present on disk and not added is not one a reader can reach, and a pointer at it
# is a pointer at nothing.

set -euo pipefail

# --------------------------------------------------------------------------
# Rules. `scan_fields` reads one record on stdin and writes records to stdout,
# one per line, as LINE<TAB>KIND<TAB>RAW.
#
# KIND is `narrows`, `narrowed-by`, `narrows-late` or `narrowed-by-late`, where
# late means the field was written after the first `## ` heading. The header is
# where a reader lands before the prose, which is the whole reason the decision
# took a field rather than a sentence in a section, so a field written below the
# first heading is refused there rather than being counted as though it had been
# in front of the reader.
#
# awk rather than grep throughout, for the reason .github/doc-paths/doc-paths.sh
# already gives: grep exits 1 when it selects nothing, which is the ordinary
# answer here, and a pipeline that has to tell "nothing matched" from "the scanner
# broke" is how a gate ends up passing on everything.
#
# No POSIX character classes and no interval expressions in any pattern below, for
# the reason that file gives too: the awk on the runner is mawk and the awk on a
# contributor's machine is frequently gawk, and those two constructs are where the
# older mawk builds disagree with it.
#
# Fenced and indented blocks are not read. A record arguing about this format
# quotes the field it is about, and a quotation of a field is not a field.
# --------------------------------------------------------------------------

scan_fields() {
  awk '
    BEGIN { fence = 0; inprose = 0 }
    {
      line = $0
      sub(/\r$/, "", line)

      if (line ~ /^[ \t]*```/ || line ~ /^[ \t]*~~~/) { fence = 1 - fence; next }
      if (fence) next
      if (line ~ /^(    |\t)/) next
      if (line ~ /^## /) { inprose = 1 }

      if (line ~ /^Narrows:/) {
        kind = inprose ? "narrows-late" : "narrows"
        raw = substr(line, 9)
      } else if (line ~ /^Narrowed-by:/) {
        kind = inprose ? "narrowed-by-late" : "narrowed-by"
        raw = substr(line, 13)
      } else next

      gsub(/^[ \t]+|[ \t]+$/, "", raw)
      printf "%d\t%s\t%s\n", FNR, kind, raw
    }
  '
}

# Every tracked decision record, and the four-digit number each one carries, as
# NUMBER<TAB>PATH.
#
# The number comes from the file name rather than from the first line, because the
# file name is what a pointer resolves against and 0001 fixes the two to agree.
record_universe() {
  git ls-files 'docs/decisions/*.md' | awk '
    {
      sub(/\r$/, "")
      if ($0 == "") next
      p = $0
      i = length(p)
      while (i > 0 && substr(p, i, 1) != "/") i--
      base = substr(p, i + 1)
      if (base !~ /^[0-9][0-9][0-9][0-9]-/) next
      printf "%s\t%s\n", substr(base, 1, 4), p
    }
  ' | sort -u
}

# The verdict over the whole corpus.
#
# Standard input is the corpus: PATH<TAB>NUMBER<TAB>LINE<TAB>KIND<TAB>RAW, every
# field of every record in one stream. $1 is the universe `record_universe`
# writes.
#
# It is one pass over everything rather than one pass per record, because the
# second direction is not a property of any single record. A record carrying
# `Narrowed-by: 0243` is right or wrong depending on what 0243 carries, and a
# checker judging each file alone could only ever see half of that.
verdicts() {
  awk -F'\t' -v universe="$1" '
    BEGIN {
      while ((getline u < universe) > 0) {
        sub(/\r$/, "", u)
        if (u == "") continue
        split(u, uf, "\t")
        exists[uf[1]] = 1
      }
    }
    {
      n = ++nf
      f_path[n] = $1; f_num[n] = $2; f_line[n] = $3; f_kind[n] = $4; f_raw[n] = $5
      if (($4 == "narrows" || $4 == "narrowed-by") && $5 ~ /^[0-9][0-9][0-9][0-9],/) {
        pair[$4 "|" $2 "|" substr($5, 1, 4)] = 1
      }
    }
    END {
      for (i = 1; i <= nf; i++) {
        path = f_path[i]; num = f_num[i]; ln = f_line[i]; kind = f_kind[i]; raw = f_raw[i]

        if (kind == "narrows-late" || kind == "narrowed-by-late") {
          field = (kind == "narrows-late") ? "Narrows:" : "Narrowed-by:"
          printf "REFUSE\t%s\t%d\tcarries %s below the first heading, where a reader who stopped at the header has already passed it\n", path, ln, field
          refusals++
          continue
        }

        field = (kind == "narrows") ? "Narrows:" : "Narrowed-by:"

        if (raw !~ /^[0-9][0-9][0-9][0-9],/) {
          printf "REFUSE\t%s\t%d\t%s is not a four-digit record number followed by a comma and the clause\n", path, ln, field
          refusals++
          continue
        }

        target = substr(raw, 1, 4)
        clause = substr(raw, 6)
        sub(/^[ \t]+/, "", clause)

        if (target == num) {
          printf "REFUSE\t%s\t%d\t%s names %s, which is this record itself\n", path, ln, field, target
          refusals++
          continue
        }

        if (!(target in exists)) {
          printf "REFUSE\t%s\t%d\t%s names %s, which is no record in docs/decisions/\n", path, ln, field, target
          refusals++
          continue
        }

        if (clause == "") {
          printf "REFUSE\t%s\t%d\t%s names record %s and no clause, so a reader still has to diff two records to find which one moved\n", path, ln, field, target
          refusals++
          continue
        }

        if (kind == "narrowed-by") {
          if (!(("narrows|" target "|" num) in pair)) {
            printf "REFUSE\t%s\t%d\tnames %s as narrowing it, and %s carries no Narrows: line naming %s back\n", path, ln, target, target, num
            refusals++
            continue
          }
        } else {
          if (!(("narrowed-by|" target "|" num) in pair)) {
            printf "REFUSE\t%s\t%d\tnarrows %s, and %s carries no Narrowed-by: line naming %s back\n", path, ln, target, target, num
            refusals++
            continue
          }
        }

        paired++
      }
      printf "COUNT\t%d\t%d\n", paired + 0, refusals + 0
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture below judges against its own universe rather than against this
# repository. A row that judged the real tree would prove the state of the tree on
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

# The refusals a corpus produces against a fixed universe, one per line.
#
# A fixture is one or more records, each given as NUMBER and body. The bodies are
# scanned by the same function the gate scans a tracked record with.
judge_fixture() {
  local universe="$1"
  shift
  local cf num body
  cf="$(mktemp)"
  while [ "$#" -gt 0 ]; do
    num="$1"
    body="$2"
    shift 2
    printf '%s' "$body" | scan_fields \
      | awk -F'\t' -v p="docs/decisions/$num-x.md" -v n="$num" \
          '{ printf "%s\t%s\t%s\t%s\t%s\n", p, n, $1, $2, $3 }' >> "$cf"
  done
  verdicts "$universe" < "$cf" | awk -F'\t' '$1 == "REFUSE" { print $2 ":" $3 ": " $4 }'
  rm -f "$cf"
}

selftest() {
  local uni
  uni="$(mktemp)"
  printf '0001\tdocs/decisions/0001-x.md\n0103\tdocs/decisions/0103-x.md\n0243\tdocs/decisions/0243-x.md\n' > "$uni"

  local narrowed clean_status
  narrowed='# 0103. A record

Date: 2026-08-24

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrowed-by: 0243, on the fourth refused behaviour

Issue: #103

## The decision
'
  local narrowing='# 0243. A later record

Date: 2026-08-31

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrows: 0103, on the fourth refused behaviour

Issue: #243

## The decision
'

  echo "== a narrowing named in both directions =="
  assert_out "passes: both halves present, each naming the other and the clause" \
    "" "$(judge_fixture "$uni" 0103 "$narrowed" 0243 "$narrowing")"
  assert_out "bites: the narrowed record alone, with nothing pointing back" \
    "docs/decisions/0103-x.md:7: names 0243 as narrowing it, and 0243 carries no Narrows: line naming 0103 back" \
    "$(judge_fixture "$uni" 0103 "$narrowed")"
  assert_out "bites: the narrowing record alone, with nothing pointing back" \
    "docs/decisions/0243-x.md:7: narrows 0103, and 0103 carries no Narrowed-by: line naming 0243 back" \
    "$(judge_fixture "$uni" 0243 "$narrowing")"
  assert_out "bites: both halves present and each naming a third record instead of the other" \
    "$(printf 'docs/decisions/0103-x.md:7: names 0001 as narrowing it, and 0001 carries no Narrows: line naming 0103 back\ndocs/decisions/0243-x.md:7: narrows 0001, and 0001 carries no Narrowed-by: line naming 0243 back')" \
    "$(judge_fixture "$uni" \
        0103 "${narrowed/0243, on the fourth/0001, on the fourth}" \
        0243 "${narrowing/0103, on the fourth/0001, on the fourth}")"

  echo "== the record named has to exist =="
  assert_out "bites: a pointer at a number no record carries" \
    "docs/decisions/0103-x.md:7: Narrowed-by: names 0244, which is no record in docs/decisions/" \
    "$(judge_fixture "$uni" 0103 "${narrowed/0243, on the fourth/0244, on the fourth}")"
  assert_out "bites: a record narrowing itself" \
    "docs/decisions/0103-x.md:7: Narrowed-by: names 0103, which is this record itself" \
    "$(judge_fixture "$uni" 0103 "${narrowed/0243, on the fourth/0103, on the fourth}")"

  echo "== the clause is the payload and is required =="
  assert_out "bites: the record named and the clause left off" \
    "docs/decisions/0103-x.md:7: Narrowed-by: names record 0243 and no clause, so a reader still has to diff two records to find which one moved" \
    "$(judge_fixture "$uni" 0103 "${narrowed/0243, on the fourth refused behaviour/0243, }")"
  assert_out "bites: a bare record number with no comma after it" \
    "docs/decisions/0103-x.md:7: Narrowed-by: is not a four-digit record number followed by a comma and the clause" \
    "$(judge_fixture "$uni" 0103 "${narrowed/0243, on the fourth refused behaviour/0243}")"
  assert_out "bites: an issue reference written where the record number belongs" \
    "docs/decisions/0103-x.md:7: Narrowed-by: is not a four-digit record number followed by a comma and the clause" \
    "$(judge_fixture "$uni" 0103 "${narrowed/0243, on the fourth/#243, on the fourth}")"

  echo "== the field sits in the header =="
  clean_status='# 0103. A record

Date: 2026-08-24

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #103

## The decision

Narrowed-by: 0243, on the fourth refused behaviour
'
  assert_out "bites: the same field one heading too far down" \
    "docs/decisions/0103-x.md:11: carries Narrowed-by: below the first heading, where a reader who stopped at the header has already passed it" \
    "$(judge_fixture "$uni" 0103 "$clean_status")"

  echo "== what a field is not =="
  assert_out "passes: a Status line, which carries the other kind of pointer" \
    "" "$(judge_fixture "$uni" 0103 '# 0103. A record

Status: accepted. Supersedes nothing. Superseded by 0243.

Issue: #103
')"
  assert_out "passes: the field quoted inside a fenced block, which is a record arguing about the format" \
    "" "$(judge_fixture "$uni" 0103 '# 0103. A record

The shape is:

```
Narrowed-by: 9999, a clause naming nothing
```

## The decision
')"
  assert_out "passes: the field quoted inside an indented block, where this board writes them more often" \
    "" "$(judge_fixture "$uni" 0103 '# 0103. A record

The shape is:

    Narrowed-by: 9999, a clause naming nothing

## The decision
')"
  assert_out "passes: a sentence beginning with the word and no colon at column zero" \
    "" "$(judge_fixture "$uni" 0103 '# 0103. A record

Narrowed by 0243 is what this became, in prose rather than in a field.

## The decision
')"
  assert_out "passes: the field indented by one space, which is prose and not a header line" \
    "" "$(judge_fixture "$uni" 0103 '# 0103. A record

 Narrowed-by: 9999, a clause naming nothing

## The decision
')"
  assert_out "passes: a record carrying neither field, which is nearly all of them" \
    "" "$(judge_fixture "$uni" 0103 '# 0103. A record

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #103

## The decision
')"

  rm -f "$uni"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures decision-record fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local universe corpus num path out
  universe="$(mktemp)"
  corpus="$(mktemp)"
  record_universe > "$universe"

  while IFS=$'\t' read -r num path; do
    [ -n "${path:-}" ] || continue
    scan_fields < "$path" \
      | awk -F'\t' -v p="$path" -v n="$num" \
          '{ printf "%s\t%s\t%s\t%s\t%s\n", p, n, $1, $2, $3 }' >> "$corpus"
  done < "$universe"

  local records fields paired=0 refusals=0
  records="$(awk 'END { print NR }' "$universe")"
  fields="$(awk 'END { print NR }' "$corpus")"

  echo "Records read: every tracked file under docs/decisions/ whose name begins with four digits and a hyphen."
  echo "Read for: a Narrows: or Narrowed-by: line at column zero, outside a fenced or indented block."
  echo "Records: ${records}. Fields found: ${fields}."
  echo

  echo "-- a narrowing is named in both directions"
  out="$(verdicts "$universe" < "$corpus")"
  while IFS=$'\t' read -r tag a b c; do
    case "$tag" in
      REFUSE)
        echo "::error file=${a},line=${b}::${a}:${b}: ${c}"
        echo "      ${a}:${b}: ${c}"
        refusals=$((refusals + 1))
        ;;
      COUNT)
        paired=$((paired + a))
        ;;
    esac
  done <<EOF
$out
EOF
  rm -f "$universe" "$corpus"

  if [ "$refusals" -eq 0 ]; then
    echo "ok    ${paired} field(s) name a record that exists, name a clause, and are named back"
  else
    echo "      ${paired} field(s) name a record that exists, name a clause, and are named back"
  fi
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether the clause a field names is the clause that actually moved. The field either carries text after the record number or it does not; whether that text describes the right sentence is a judgement about meaning, and the review is where a wrong one is caught."
  echo "NOT MADE HERE: whether a record that narrows another declared it at all. A later record that quietly narrows an earlier one and writes no field is silent to every rule above, which is the residual 0267 states of itself."
  echo "NOT MADE HERE: the Status: line and the supersession it names. That pointer is 0001's and no rule here reads it."
  echo "NOT MADE HERE: a narrowing written between this repository and another one. The universe is docs/decisions/ in this tree, so a four-digit number is resolved here or nowhere."
  echo

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} narrowing field(s) do not hold. Each one is printed above with its record and line."
    return 1
  fi
  echo "Every narrowing these records declare is named from both ends."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
