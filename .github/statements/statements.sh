#!/usr/bin/env bash
# Every scanner exclusion and every dismissed finding has a published statement (#89).
#
# Every scanner pointed at this repository will eventually report something that is
# not exploitable here. The choices are to fix it, to suppress it silently, or to
# say publicly why it does not apply, and the third is the only one a downstream
# consumer can act on. `security/statements.json` is where that is said, and this
# is what stops it drifting against the configurations it describes.
#
# WHY AN EXCLUSION IS INSIDE THE RULE AND NOT BESIDE IT. A directory a scanner is
# never pointed at produces no finding, so nobody dismisses anything, so a rule
# written only about dismissals owes nothing for it. But a consumer reading the
# published statements sees the dismissals with their reasons and concludes that
# everything else was looked at and came back clean, and where a class was excluded
# that conclusion is false and nothing says so. A rule whose honest route costs a
# written statement while its evasion - narrowing the scanner so the finding never
# fires - costs nothing selects for the evasion, and it does so without anybody
# deciding to evade, because the configuration change looks like tuning.
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules to the document and the tree, and refuse
#
# The set of configurations is DERIVED rather than written down here or in the
# document. `derived_directives` reads every tracked file under .github/ whose name
# begins with `excluded` or is `suppressions`, which is the shape this repository
# gives a register of directives a scanner is told to skip. A register added
# tomorrow under that shape is read on the day it lands, and a document that
# enumerated them instead would drift against the thing it describes.

set -euo pipefail

# The fields every entry of each kind must carry, non-empty.
#
# `rule` is deliberately absent from the exclusion list. An exclusion expressed as a
# directive in a register carries the directive; one expressed as a persona nobody
# selected or a codepoint nobody listed has no directive to carry, and demanding one
# would be demanding an identifier that does not exist. `rule` is `null` there, and
# the two directions below are written so that null means "not derivable" rather
# than "unchecked".
FINDING_FIELDS='id scanner subject status justification impact retires_when'
EXCLUSION_FIELDS='id scanner configuration scope reason not_looked_for retires_when'

# What a dismissed finding may say about itself. A free-text status is a status
# nobody can group on, which is the whole point of the document being machine
# readable.
FINDING_STATUSES='not_applicable not_exploitable accepted fixed'

# --------------------------------------------------------------------------
# Rule 1. The document parses and carries the shape the rest of the rules read.
#
# It is first because every rule after it reads fields out of the document, and a
# checker that cannot parse its subject reports nothing and reads exactly like a
# clean run.
# --------------------------------------------------------------------------

doc_shape() {
  local doc="$1"

  if ! jq -e . "$doc" > /dev/null 2>&1; then
    echo "REFUSE	${doc}	is not valid JSON, so nothing below could read it"
    return 0
  fi

  jq -r --arg doc "$doc" '
    def bad(m): "REFUSE\t" + $doc + "\t" + m;
    [
      (if type != "object" then bad("is not a JSON object") else empty end),
      (if has("version") and (.version | type) == "number" then empty
       else bad("carries no numeric `version`") end),
      (if has("updated") and (.updated | type) == "string" and (.updated | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")) then empty
       else bad("carries no `updated` date written as YYYY-MM-DD") end),
      (if has("about") and (.about | type) == "array" and (.about | length) > 0 then empty
       else bad("carries no `about`, which is what a consumer reads before the entries") end),
      (if has("findings") and (.findings | type) == "array" then empty
       else bad("carries no `findings` array") end),
      (if has("exclusions") and (.exclusions | type) == "array" then empty
       else bad("carries no `exclusions` array") end)
    ] | .[]
  ' "$doc"
}

# --------------------------------------------------------------------------
# Rule 2, 3 and 4. Every entry carries its fields, its status is one the document
# declares, and no identifier is used twice.
#
# An empty string is refused as hard as an absent key. A field somebody left blank
# to get past a required-fields check is the silent suppression this document
# exists to replace, wearing the shape of a statement.
# --------------------------------------------------------------------------

doc_entries() {
  local doc="$1"

  jq -e . "$doc" > /dev/null 2>&1 || return 0

  jq -r --arg doc "$doc" \
        --arg ff "$FINDING_FIELDS" \
        --arg ef "$EXCLUSION_FIELDS" \
        --arg st "$FINDING_STATUSES" '
    def bad(m): "REFUSE\t" + $doc + "\t" + m;
    def entlabel(kind; i; e): kind + " entry " + (i | tostring)
      + (if (e.id | type) == "string" and e.id != "" then " (" + e.id + ")" else "" end);
    def fields(kind; list; arr):
      [ arr | to_entries[] as $p
        | ($p.value) as $e
        | (list | split(" "))[] as $f
        | if ($e | has($f)) and (($e[$f] | type) == "string") and ($e[$f] != "")
          then empty
          else bad(entlabel(kind; $p.key; $e) + " carries no `" + $f + "`, or carries it empty") end
      ];
    ((.findings // []) | if type == "array" then . else [] end) as $fs
    | ((.exclusions // []) | if type == "array" then . else [] end) as $xs
    | (fields("findings"; $ff; $fs))
      + (fields("exclusions"; $ef; $xs))
      + [ $fs | to_entries[] as $p
          | if (($st | split(" ")) | index($p.value.status)) != null then empty
            else bad(entlabel("findings"; $p.key; $p.value) + " carries status `"
                     + (($p.value.status // "") | tostring)
                     + "`, which is not one this document declares") end ]
      + [ ($fs + $xs) | map(.id) | group_by(.) | .[]
          | select(length > 1)
          | bad("uses the identifier `" + .[0] + "` " + (length | tostring) + " times") ]
    | .[]
  ' "$doc"
}

# --------------------------------------------------------------------------
# The directives this repository actually carries, derived rather than declared.
#
# One line per directive, as CONFIGURATION<TAB>RULE. A comment line and a blank
# line are not directives; the first whitespace-delimited token of anything else is
# the identifier, which is the shape every register of this kind in the tree takes.
# --------------------------------------------------------------------------

register_paths() {
  git ls-files '.github/*' | awk '
    {
      sub(/\r$/, "")
      if ($0 == "") next
      p = $0
      i = length(p)
      while (i > 0 && substr(p, i, 1) != "/") i--
      base = substr(p, i + 1)
      # A register is data. The script that reads one frequently shares its
      # directory and half its name, and reading a shell file line by line as
      # though every statement in it were a directive turns two hundred lines of
      # source into two hundred refusals, which is a gate nobody can read.
      if (length(base) > 3 && substr(base, length(base) - 2) == ".sh") next
      if (base == "suppressions" || substr(base, 1, 8) == "excluded") print p
    }
  ' | sort -u
}

directives_in() {
  awk -v cfg="$1" '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*#/) next
      if (line ~ /^[ \t]*$/) next
      n = split(line, t, /[ \t]+/)
      tok = (t[1] == "") ? t[2] : t[1]
      if (tok == "") next
      printf "%s\t%s\n", cfg, tok
    }
  '
}

derived_directives() {
  local p
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    directives_in "$p" < "$p"
  done < <(register_paths)
}

# --------------------------------------------------------------------------
# Rule 5, 6 and 7. The two directions, and the path.
#
# A directive with no entry is the case the whole document is for. An entry naming
# a directive nothing carries any more is the other direction, and it fails closed
# for the reason the store of waivers in the fleet does: a statement about
# something that is no longer excluded tells a consumer the scanner is narrower
# than it is. An entry naming a configuration that is not tracked is a pointer at
# nothing, which rots in silence.
# --------------------------------------------------------------------------

coverage() {
  local doc="$1" directives="$2" universe="$3"

  jq -e . "$doc" > /dev/null 2>&1 || return 0

  # Every input is a real file rather than a process substitution. A jq built for
  # Windows cannot open the /proc descriptor a substitution hands it, and the
  # failure is a message on standard error and an empty verdict, which reads
  # exactly like a run that found nothing.
  local dj uj
  dj="$(mktemp)"; uj="$(mktemp)"
  jq -R 'split("\t") | {configuration: .[0], rule: .[1]}' < "$directives" > "$dj"
  jq -R '.' < "$universe" > "$uj"

  jq -rn --arg doc "$doc" \
         --slurpfile d "$dj" \
         --slurpfile u "$uj" \
         --slurpfile j "$doc" '
    def bad(m): "REFUSE\t" + $doc + "\t" + m;
    ($j[0].exclusions // []) as $xs
    | ($d | map(select(.configuration != null and .configuration != ""))) as $dir
    | ($u | map(select(. != ""))) as $tracked
    | [ $dir[] as $one
        | if ([ $xs[] | select(.configuration == $one.configuration and .rule == $one.rule) ] | length) > 0
          then empty
          else bad("names no statement for `" + $one.rule + "` in " + $one.configuration
                   + ", which that configuration excludes") end ]
      + [ $xs[] | select(.rule != null and .rule != "") as $e
          | if ([ $dir[] | select(.configuration == $e.configuration and .rule == $e.rule) ] | length) > 0
            then empty
            else bad("carries a statement for `" + $e.rule + "` in " + $e.configuration
                     + ", which that configuration does not exclude") end ]
      + [ $xs[] | select((.configuration | type) == "string" and .configuration != "") as $e
          | if ($tracked | index($e.configuration)) != null then empty
            else bad("names configuration `" + $e.configuration + "`, which is not a tracked path in this tree") end ]
    | .[]
  '
  rm -f "$dj" "$uj"
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture below judges its own document against its own directive list and
# its own path universe. A row that judged this repository would prove the state of
# the tree on the day it ran, not the rule.
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

# The refusals one document produces, with the message alone.
judge_shape() {
  local df
  df="$(mktemp)"
  printf '%s' "$1" > "$df"
  { doc_shape "$df"; doc_entries "$df"; } | awk -F'\t' '$1 == "REFUSE" { print $3 }'
  rm -f "$df"
}

judge_coverage() {
  local body="$1" dirs="$2" uni="$3"
  local df ddf uf
  df="$(mktemp)"; ddf="$(mktemp)"; uf="$(mktemp)"
  printf '%s' "$body" > "$df"
  printf '%s' "$dirs" > "$ddf"
  printf '%s' "$uni" > "$uf"
  coverage "$df" "$ddf" "$uf" | awk -F'\t' '$1 == "REFUSE" { print $3 }'
  rm -f "$df" "$ddf" "$uf"
}

FIXTURE_FINDING='{"id":"s/A","scanner":"Scorecard","subject":"a check","status":"not_applicable","justification":"one maintainer","impact":"nothing stands in place of it","retires_when":"a second account exists"}'
FIXTURE_EXCLUSION='{"id":"sc/SC1","scanner":"shellcheck","configuration":".github/x/excluded-rules","rule":"SC1","scope":"the rule alone","reason":"it is the ordinary case here","not_looked_for":"the mistake the rule exists for","retires_when":"the checks stop doing that"}'

doc_with() {
  printf '{"version":1,"updated":"2026-09-02","about":["why"],"findings":[%s],"exclusions":[%s]}' "$1" "$2"
}

selftest() {
  local good_dirs='.github/x/excluded-rules	SC1
'
  local good_uni='.github/x/excluded-rules
'

  echo "== the document parses and carries its shape =="
  assert_out "passes: a document carrying one entry of each kind" \
    "" "$(judge_shape "$(doc_with "$FIXTURE_FINDING" "$FIXTURE_EXCLUSION")")"
  assert_out "bites: a trailing comma, which is the edit somebody actually makes" \
    "is not valid JSON, so nothing below could read it" \
    "$(judge_shape '{"version":1,"updated":"2026-09-02","about":["why"],"findings":[],"exclusions":[],}')"
  assert_out "bites: the exclusions array missing entirely" \
    "carries no \`exclusions\` array" \
    "$(judge_shape '{"version":1,"updated":"2026-09-02","about":["why"],"findings":[]}')"
  assert_out "bites: a date written the other way round" \
    "carries no \`updated\` date written as YYYY-MM-DD" \
    "$(judge_shape '{"version":1,"updated":"02-09-2026","about":["why"],"findings":[],"exclusions":[]}')"
  assert_out "bites: an about section emptied rather than removed" \
    "carries no \`about\`, which is what a consumer reads before the entries" \
    "$(judge_shape '{"version":1,"updated":"2026-09-02","about":[],"findings":[],"exclusions":[]}')"
  assert_out "passes: both arrays present and empty, which is a tree with nothing to state" \
    "" "$(judge_shape '{"version":1,"updated":"2026-09-02","about":["why"],"findings":[],"exclusions":[]}')"

  echo "== every entry carries its fields =="
  assert_out "bites: an exclusion with no not_looked_for, which is the field an author leaves out" \
    "exclusions entry 0 (sc/SC1) carries no \`not_looked_for\`, or carries it empty" \
    "$(judge_shape "$(doc_with "$FIXTURE_FINDING" "$(printf '%s' "$FIXTURE_EXCLUSION" | jq -c 'del(.not_looked_for)')")")"
  assert_out "bites: the same field present and blank, which passes a presence test" \
    "exclusions entry 0 (sc/SC1) carries no \`not_looked_for\`, or carries it empty" \
    "$(judge_shape "$(doc_with "$FIXTURE_FINDING" "$(printf '%s' "$FIXTURE_EXCLUSION" | jq -c '.not_looked_for = ""')")")"
  assert_out "bites: a finding with no retires_when, so its disposition never expires" \
    "findings entry 0 (s/A) carries no \`retires_when\`, or carries it empty" \
    "$(judge_shape "$(doc_with "$(printf '%s' "$FIXTURE_FINDING" | jq -c 'del(.retires_when)')" "$FIXTURE_EXCLUSION")")"
  assert_out "bites: a status outside the declared vocabulary" \
    "findings entry 0 (s/A) carries status \`wontfix\`, which is not one this document declares" \
    "$(judge_shape "$(doc_with "$(printf '%s' "$FIXTURE_FINDING" | jq -c '.status = "wontfix"')" "$FIXTURE_EXCLUSION")")"
  assert_out "bites: one identifier used by a finding and an exclusion" \
    "uses the identifier \`s/A\` 2 times" \
    "$(judge_shape "$(doc_with "$FIXTURE_FINDING" "$(printf '%s' "$FIXTURE_EXCLUSION" | jq -c '.id = "s/A"')")")"
  assert_out "passes: a null rule, which is an exclusion with no directive to name" \
    "" "$(judge_shape "$(doc_with "$FIXTURE_FINDING" "$(printf '%s' "$FIXTURE_EXCLUSION" | jq -c '.rule = null')")")"

  echo "== the two directions, and the path =="
  assert_out "passes: one directive with one statement naming it" \
    "" "$(judge_coverage "$(doc_with "" "$FIXTURE_EXCLUSION")" "$good_dirs" "$good_uni")"
  assert_out "bites: a directive nothing states, which is the case the document exists for" \
    "names no statement for \`SC2\` in .github/x/excluded-rules, which that configuration excludes" \
    "$(judge_coverage "$(doc_with "" "$FIXTURE_EXCLUSION")" "$(printf '.github/x/excluded-rules\tSC1\n.github/x/excluded-rules\tSC2\n')" "$good_uni")"
  assert_out "bites: a statement for a directive that has been taken back out" \
    "carries a statement for \`SC1\` in .github/x/excluded-rules, which that configuration does not exclude" \
    "$(judge_coverage "$(doc_with "" "$FIXTURE_EXCLUSION")" "" "$good_uni")"
  assert_out "bites: a statement naming a configuration that is not tracked" \
    "$(printf 'names no statement for `SC1` in .github/x/excluded-rules, which that configuration excludes\ncarries a statement for `SC1` in .github/y/excluded-rules, which that configuration does not exclude\nnames configuration `.github/y/excluded-rules`, which is not a tracked path in this tree')" \
    "$(judge_coverage "$(doc_with "" "$(printf '%s' "$FIXTURE_EXCLUSION" | jq -c '.configuration = ".github/y/excluded-rules"')")" "$good_dirs" "$good_uni")"
  assert_out "passes: a null rule against a directive list that does not carry it" \
    "" "$(judge_coverage "$(doc_with "" "$(printf '%s' "$FIXTURE_EXCLUSION" | jq -c '.rule = null')")" "" "$good_uni")"

  echo "== what a register line is, and is not =="
  assert_out "reads: the identifier and not the reason after it" \
    "$(printf 'r\tSC2016')" \
    "$(printf 'SC2016 Every rule in this gate is an awk program.\n' | directives_in r)"
  assert_out "does not read: a comment line, which is where every one of these registers explains itself" \
    "" "$(printf '# SC2016 would go here\n' | directives_in r)"
  assert_out "does not read: a blank line" \
    "" "$(printf '\n   \n' | directives_in r)"
  assert_out "reads: an identifier written with leading whitespace" \
    "$(printf 'r\tSC2016')" \
    "$(printf '  SC2016 a reason\n' | directives_in r)"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures statement fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

DOCUMENT='security/statements.json'

check() {
  local dirs uni out refusals=0 findings exclusions ndir

  if ! git ls-files --error-unmatch "$DOCUMENT" > /dev/null 2>&1; then
    echo "::error file=${DOCUMENT}::${DOCUMENT} is not a tracked file. Every rule below reads it, so this run judges nothing."
    return 1
  fi

  dirs="$(mktemp)"; uni="$(mktemp)"
  derived_directives > "$dirs"
  git ls-files > "$uni"

  ndir="$(awk 'END { print NR }' "$dirs")"
  findings="$(jq -r '(.findings // []) | length' "$DOCUMENT" 2>/dev/null || echo '?')"
  exclusions="$(jq -r '(.exclusions // []) | length' "$DOCUMENT" 2>/dev/null || echo '?')"

  echo "Document read: ${DOCUMENT}."
  echo "Registers derived: every tracked file under .github/ whose name begins with 'excluded' or is 'suppressions'."
  echo "Registers found: $(register_paths | tr '\n' ' ')"
  echo "Directives in them: ${ndir}. Statements in the document: ${findings} finding(s), ${exclusions} exclusion(s)."
  echo

  echo "-- every statement has its fields, and every exclusion is named from both ends"
  out="$({ doc_shape "$DOCUMENT"; doc_entries "$DOCUMENT"; coverage "$DOCUMENT" "$dirs" "$uni"; })"
  while IFS=$'\t' read -r tag a b; do
    [ "$tag" = "REFUSE" ] || continue
    echo "::error file=${a}::${a} ${b}"
    echo "      ${a} ${b}"
    refusals=$((refusals + 1))
  done <<EOF
$out
EOF
  rm -f "$dirs" "$uni"

  if [ "$refusals" -eq 0 ]; then
    echo "ok    every directive these registers carry has a statement, and every statement names a directive that exists"
  fi
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether a statement is TRUE. Every field either carries text or it does not; whether the reasoning is sound, and whether what it says is not looked for is what is actually not looked for, is a judgement no reading of the tree makes."
  echo "NOT MADE HERE: an exclusion that is not a directive in a register. The persona a workflow does not select and the codepoint a pattern does not list are stated in the document and derived by nothing, so their entries carry a null rule and only their configuration path is checked. An exclusion of that kind added tomorrow with no statement is silent to every rule above."
  echo "NOT MADE HERE: a finding dismissed on the code-scanning surface. A dismissal is a state on the repository rather than a byte in the tree, no commit records one, and the listing needs a permission a reader of this board does not hold. The findings entries in the document are written by hand and nothing here compares them against the live dismissals."
  echo "NOT MADE HERE: a dependency advisory. None has been raised against this repository, the document says so of itself, and the shape one takes here is decided when the first one arrives."
  echo "NOT MADE HERE: whether the document is valid OpenVEX. It does not claim to be, and no projection to it has been produced."
  echo

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} statement(s) do not hold. Each one is printed above."
    return 1
  fi
  echo "Every directive this repository excludes has a published statement saying why, and what is consequently not looked for."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
