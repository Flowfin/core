#!/usr/bin/env bash
# The shell this gate runs is analysed (#81).
#
# Two of this gate's legs are shell scripts and nothing read them for defects. The
# workflow YAML has an analyser, the repository's own configuration has one, and
# the language the checks are actually written in had none, so the analysed set and
# the executed set had come apart.
#
# The rules are not written here. The rule set is shellcheck's, and what this file
# owns is which files it is pointed at, what severity gates, which rules are not
# refused and why, and the fixtures that prove both of those bite. That is the same
# arrangement the other two legs use: `selftest` and `check` reach the analyser
# through one function, so a fixture that passes and a gate that judges something
# else cannot come apart.
#
# A rule this gate does not refuse is written in `.github/shell-analysis/excluded-rules`
# with the reason it is not refused, and a line there carrying an identifier and no
# reason is itself refused. Every run prints that file with its reasons, so an
# exclusion is read beside the verdict rather than found later by somebody
# auditing. An exclusion is a debt and each reason says what retires it.
#
# What the check is for: the defect class that made this worth a leg is an
# expansion left unquoted, which word-splits its input and silently changes what a
# rule was pointed at. A scope gate elsewhere in this organisation split its input
# on whitespace and refused valid work for it. Both scripts here read paths and
# pull-request bodies, which are exactly the inputs that carry a space.
#
# Where a finding is read: the job log, and the code-scanning surface. A run writes
# the findings as SARIF when `SHELL_ANALYSIS_SARIF` names a path, and the workflow
# is what uploads that file. A run with the variable unset writes nothing anywhere,
# which is what a run on somebody's own machine does, and it says which of the two
# it was on its own last lines rather than leaving a reader to assume.
#
# The surface and the gate read one setting. The SARIF is produced by the same
# function the gate is, with the same severity and the same exclusions, so an alert
# that is filed is a finding that was refused and a rule this register excuses
# reaches neither.
#
# Verbs:
#   selftest   prove the analyser refuses that defect, that the same fixture with
#              the quotation marks added is not refused, that a rule excluded with
#              no reason after it is refused while the same line answered is not,
#              and that the SARIF a run writes carries the refused finding, carries
#              nothing for the neighbour, and carries nothing for an excused rule
#   check      run the fixtures, judge the exclusion register, print it, analyse
#              every tracked shell file, write the SARIF where one is asked for,
#              and refuse
#
# `check` reads the repository through `git ls-files`, so the authority for what is
# analysed is the tracked set. A file present on disk and not added is not a file
# this gate runs.

set -euo pipefail

# The severity that gates. shellcheck's own floor, which reports error, warning,
# info and style, so nothing it can say is dropped before a person sees it. It is
# written here rather than left to the tool's default because a default is a
# decision somebody else can change under this gate without a commit here.
SEVERITY=style

# The rules this gate does not refuse, and the reason each one is not refused.
# The file is beside this one rather than inside it so that a person deciding
# whether to add an exclusion edits a register instead of a script.
EXCLUSIONS_FILE="$(dirname "$0")/excluded-rules"

# Where a run writes its findings for the code-scanning surface. Empty means it
# writes nothing anywhere. It is read from the environment rather than taken as an
# argument so that the verb a person runs by hand and the verb the workflow runs
# are the same verb, and the difference between them is one variable a reader can
# see in the workflow file.
SARIF_OUT="${SHELL_ANALYSIS_SARIF:-}"

# The schema the written file declares. SARIF 2.1.0 is the version the
# code-scanning surface accepts.
SARIF_SCHEMA="https://json.schemastore.org/sarif-2.1.0.json"

# Every tracked shell file.
shell_files() {
  git ls-files '*.sh'
}

# The register, as RULE<TAB>REASON, one per line. Comments and blank lines are
# dropped. A line carrying an identifier and no reason is reported as BARE rather
# than dropped: a bare rule identifier is a rule somebody turned off and nobody
# argued with, and that is the shape this register exists against.
excluded_rules() {
  awk '
    { sub(/\r$/, "") }
    /^[ \t]*$/ { next }
    /^[ \t]*#/ { next }
    {
      id = $1
      reason = $0
      sub(/^[ \t]*[^ \t]+[ \t]*/, "", reason)
      if (reason == "") { printf "BARE\t%s\t%d\n", id, NR; next }
      printf "%s\t%s\n", id, reason
    }
  ' "$1"
}

# The identifiers only, comma separated, in the order the register writes them.
excluded_ids() {
  excluded_rules "$1" | awk -F'\t' '$1 != "BARE" { printf "%s%s", (n++ ? "," : ""), $1 } END { print "" }'
}

# Refuses the register itself, and says which line is bare.
judge_register() {
  local bare
  bare="$(excluded_rules "$1" | awk -F'\t' '$1 == "BARE" { print "      line " $3 ": " $2 " carries no reason after it" }')"
  if [ -n "$bare" ]; then
    echo "::error::The exclusion register carries a rule identifier with no reason after it. An exclusion nobody argued with is a rule somebody turned off."
    printf '%s\n' "$bare"
    return 1
  fi
  return 0
}

# The analyser, reached through one function so the fixtures and the gate cannot
# be pointed at different settings.
#
# The gcc format prints one finding per line as PATH:LINE:COL: LEVEL: MESSAGE
# followed by the rule identifier in brackets, so a fixture can assert which rule
# bit rather than only that something did. The shell dialect is left to each
# file's shebang: forcing one here would read a file declaring another dialect as
# if it had declared this one, which is the finding rather than a setting.
analyse() {
  analyse_as gcc "$@"
}

# The one place shellcheck is invoked. The format is the only thing that varies
# between the gate and the file written for the code-scanning surface, so the
# severity and the exclusions cannot differ between what is refused and what is
# filed as an alert.
analyse_as() {
  local format="$1"
  shift
  local excluded
  excluded="$(excluded_ids "$EXCLUSIONS_FILE")"
  if [ -n "$excluded" ]; then
    shellcheck --format="$format" --severity="$SEVERITY" --exclude="$excluded" "$@"
  else
    shellcheck --format="$format" --severity="$SEVERITY" "$@"
  fi
}

# The build of the analyser that judged this run, so the written file says which
# one produced its findings rather than leaving that to the runner image.
analyser_version() {
  shellcheck --version | awk -F': ' '/^version:/ { print $2; exit }'
}

# Writes the findings for the files given, as SARIF, to the path in $1.
#
# The conversion is from shellcheck's own `json1`, which carries the rule number,
# the level, the message and both ends of the region, rather than from the line
# format the gate prints: reconstructing JSON out of a message that may itself hold
# a quotation mark is how a file that parses locally is refused by the surface.
#
# The level names differ between the two vocabularies and the mapping is written
# out rather than passed through. shellcheck says error, warning, info and style;
# SARIF has error, warning, note and none. info and style both become note, which
# is the level this surface shows without gating, and nothing becomes none, because
# a finding this gate refuses is not one to file as having no level at all.
#
# The region's end is written only where it is past the start. An end equal to the
# start is a zero-width region, which is a shape the surface rejects on some
# findings and renders as nothing on others.
write_sarif() {
  local out="$1"
  shift
  local found status=0
  found="$(analyse_as json1 "$@")" || status=$?
  if [ -z "$found" ]; then
    echo "::error::The analyser wrote no JSON to convert, so there is nothing to hand the code-scanning surface. It exited ${status}."
    return 1
  fi
  printf '%s\n' "$found" | jq \
    --arg schema "$SARIF_SCHEMA" \
    --arg analyser "$(analyser_version)" '
    def sarif_level:
      { "error": "error", "warning": "warning", "info": "note", "style": "note" }[.] // "note";
    def region($c):
      { startLine: $c.line, startColumn: $c.column }
      + (if ($c.endLine > $c.line) or ($c.endColumn > $c.column)
         then { endLine: $c.endLine, endColumn: $c.endColumn }
         else {} end);
    [ .comments[]? ] as $found
    | { "$schema": $schema,
        version: "2.1.0",
        runs: [ {
          tool: { driver: {
            name: "shellcheck",
            informationUri: "https://www.shellcheck.net/",
            version: $analyser,
            rules: ( $found
                     | map({ id: ("SC" + (.code | tostring)) })
                     | unique_by(.id)
                     | map(. + { helpUri: ("https://www.shellcheck.net/wiki/" + .id) }) )
          } },
          results: ( $found | map({
            ruleId: ("SC" + (.code | tostring)),
            level: (.level | sarif_level),
            message: { text: .message },
            locations: [ { physicalLocation: {
              artifactLocation: { uri: .file, uriBaseId: "%SRCROOT%" },
              region: region(.)
            } } ]
          }) )
        } ] }
  ' > "$out"
}

# How many findings the written file carries.
sarif_results() {
  jq '.runs[0].results | length' "$1"
}

# --------------------------------------------------------------------------
# selftest
#
# Two fixtures, one change apart. The first carries an unquoted expansion and has
# to be refused for that rule by name. The second is the same file with the two
# quotes added and has to be refused for nothing at all. A guard proven only by the
# first is a guard that could be refusing everything.
# --------------------------------------------------------------------------

# Where the fixtures are written. It is set here rather than inside `selftest`
# because the trap that removes it runs after that function has returned, and a
# name local to the function is gone by then. Reached that way once, and the
# cleanup failed with the directory still on the disk.
FIXTURE_DIR=""

remove_fixtures() {
  if [ -n "${FIXTURE_DIR:-}" ]; then
    rm -rf "$FIXTURE_DIR"
  fi
}

fixture_dir() {
  local dir="$1"

  cat > "$dir/refused.sh" <<'FIXTURE'
#!/usr/bin/env bash
# A fixture. The expansion below is unquoted, so a path carrying a space arrives at
# the command as two arguments and the rule this stands for is pointed at something
# nobody named.
set -euo pipefail
count_lines() {
  local path="$1"
  wc -l < $path
}
count_lines "$@"
FIXTURE

  cat > "$dir/kept.sh" <<'FIXTURE'
#!/usr/bin/env bash
# The same fixture with the expansion quoted, and nothing else changed.
set -euo pipefail
count_lines() {
  local path="$1"
  wc -l < "$path"
}
count_lines "$@"
FIXTURE

  cat > "$dir/register-bare" <<'FIXTURE'
# A register whose second entry is an identifier with nothing after it.
SC2086 the reason this one carries
SC2129
FIXTURE

  cat > "$dir/register-answered" <<'FIXTURE'
# The same register with the one line answered, and nothing else changed.
SC2086 the reason this one carries
SC2129 the reason this one carries
FIXTURE
}

selftest() {
  local dir findings status
  FIXTURE_DIR="$(mktemp -d)"
  dir="$FIXTURE_DIR"
  trap remove_fixtures EXIT
  fixture_dir "$dir"

  echo "-- the analyser refuses an unquoted expansion"
  status=0
  findings="$(analyse "$dir/refused.sh" 2>&1)" || status=$?
  if [ "$status" -eq 0 ]; then
    echo "::error::The fixture carrying an unquoted expansion was not refused. This guard is not biting, and a green run of it means nothing."
    return 1
  fi
  if ! printf '%s\n' "$findings" | grep -q 'SC2086'; then
    echo "::error::The fixture was refused, but not for the rule this check exists for. What it said:"
    printf '%s\n' "$findings"
    return 1
  fi
  echo "ok    refused, and the finding names SC2086"
  printf '%s\n' "$findings" | sed 's|^'"$dir"'/|      |'
  echo

  echo "-- the same fixture with the expansion quoted is not refused"
  status=0
  findings="$(analyse "$dir/kept.sh" 2>&1)" || status=$?
  if [ "$status" -ne 0 ]; then
    echo "::error::The neighbouring fixture, which differs by two quotation marks, was refused. A guard that refuses its own near miss refuses honest work. What it said:"
    printf '%s\n' "$findings"
    return 1
  fi
  echo "ok    not refused"
  echo

  echo "-- the register refuses a rule identifier with no reason"
  status=0
  findings="$(judge_register "$dir/register-bare" 2>&1)" || status=$?
  if [ "$status" -eq 0 ]; then
    echo "::error::A register carrying a bare rule identifier was accepted. An exclusion nobody argued with would then be one line away."
    return 1
  fi
  if ! printf '%s\n' "$findings" | grep -q 'SC2129'; then
    echo "::error::The register was refused, but the message does not name the line that is bare. What it said:"
    printf '%s\n' "$findings"
    return 1
  fi
  echo "ok    refused, and the message names SC2129"
  echo

  echo "-- the same register with that one line answered is not refused"
  status=0
  findings="$(judge_register "$dir/register-answered" 2>&1)" || status=$?
  if [ "$status" -ne 0 ]; then
    echo "::error::The neighbouring register, which differs by one reason, was refused. What it said:"
    printf '%s\n' "$findings"
    return 1
  fi
  echo "ok    not refused"
  echo

  # The three below are about the file a run hands the code-scanning surface. The
  # four above prove what is refused; these prove that what is refused is what is
  # filed, that the neighbour files nothing, and that a rule the register excuses
  # reaches the surface no more than it reaches the gate.
  local wanted_line ruleid uri line results
  wanted_line="$(awk '/wc -l </ { print NR; exit }' "$dir/refused.sh")"

  echo "-- the refused finding is written for the code-scanning surface"
  write_sarif "$dir/refused.sarif" "$dir/refused.sh" || return 1
  results="$(sarif_results "$dir/refused.sarif")"
  if [ "$results" -ne 1 ]; then
    echo "::error::The written file carries ${results} finding(s) where the fixture raises one. What is filed and what is refused have come apart."
    return 1
  fi
  ruleid="$(jq -r '.runs[0].results[0].ruleId' "$dir/refused.sarif")"
  uri="$(jq -r '.runs[0].results[0].locations[0].physicalLocation.artifactLocation.uri' "$dir/refused.sarif")"
  line="$(jq -r '.runs[0].results[0].locations[0].physicalLocation.region.startLine' "$dir/refused.sarif")"
  if [ "$ruleid" != "SC2086" ] || [ "${uri##*/}" != "refused.sh" ] || [ "$line" != "$wanted_line" ]; then
    echo "::error::The written finding does not name the rule, the file and the line the analyser refused. It says ${ruleid} at ${uri}:${line}, and the fixture carries SC2086 at line ${wanted_line}."
    return 1
  fi
  echo "ok    one finding, SC2086, at ${uri##*/} line ${line}"
  echo

  echo "-- the same fixture with the expansion quoted files nothing"
  write_sarif "$dir/kept.sarif" "$dir/kept.sh" || return 1
  results="$(sarif_results "$dir/kept.sarif")"
  if [ "$results" -ne 0 ]; then
    echo "::error::The neighbouring fixture, which differs by two quotation marks, produced ${results} finding(s) for the surface. An alert on honest work is worse than none, because somebody has to close it."
    return 1
  fi
  echo "ok    no finding written"
  echo

  echo "-- a rule the register excuses is not written either"
  local register_was="$EXCLUSIONS_FILE"
  EXCLUSIONS_FILE="$dir/register-answered"
  write_sarif "$dir/excused.sarif" "$dir/refused.sh" || { EXCLUSIONS_FILE="$register_was"; return 1; }
  EXCLUSIONS_FILE="$register_was"
  results="$(sarif_results "$dir/excused.sarif")"
  if [ "$results" -ne 0 ]; then
    echo "::error::A register excusing SC2086 still produced ${results} finding(s) for the surface. The gate and the surface are reading different settings, so an alert could name a rule this repository has argued is not a defect here."
    return 1
  fi
  echo "ok    no finding written, from the same fixture the first of these three refused"
  echo

  echo "Every fixture behaved as this check claims, at severity ${SEVERITY}."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local line status=0
  local id reason
  local files=()
  while IFS= read -r line; do
    [ -n "$line" ] && files+=("$line")
  done < <(shell_files)

  judge_register "$EXCLUSIONS_FILE" || return 1

  echo "Rules this run does not refuse, from ${EXCLUSIONS_FILE}:"
  if [ -z "$(excluded_ids "$EXCLUSIONS_FILE")" ]; then
    echo "      none"
  else
    while IFS=$'	' read -r id reason; do
      [ -n "${id:-}" ] || continue
      echo "      ${id}: ${reason}"
    done < <(excluded_rules "$EXCLUSIONS_FILE")
  fi
  echo

  echo "Analysed ${#files[@]} tracked shell file(s) at severity ${SEVERITY}:"
  printf '      %s
' "${files[@]:-}"
  echo

  if [ "${#files[@]}" -eq 0 ]; then
    echo "::error::No tracked shell file was found. This check has nothing to judge, which is a change to the tree rather than a pass."
    return 1
  fi

  # The files are handed over as an array rather than through word splitting, so a
  # path carrying a space is one path here as well, and the analyser is reached
  # through the same function the fixtures used.
  analyse "${files[@]}" || status=$?

  # Written before the refusal below, so a run that refuses something still hands
  # the surface what it refused. A run that could only file its findings when it
  # had none would be filing exactly the set nobody needs.
  if [ -n "$SARIF_OUT" ]; then
    echo
    write_sarif "$SARIF_OUT" "${files[@]}" || return 1
    echo "Written for the code-scanning surface: ${SARIF_OUT}, carrying $(sarif_results "$SARIF_OUT") finding(s)."
  fi

  echo
  echo "-- what this run did not read"
  echo "NOT READ HERE: the core's own language. None is chosen, no code is in this tree, and #11 is where that is decided. This leg covers the shell the gate runs and nothing else."
  echo "NOT READ HERE: the workflow YAML. .github/workflows/zizmor.yml analyses that, and a second analyser over one subject is a separate argument rather than a setting here."
  if [ -n "$SARIF_OUT" ]; then
    echo "NOT UPLOADED FROM HERE: this run writes the file named above and uploads nothing. The workflow step that uploads it is what reaches the code-scanning surface, and it is skipped where the token cannot write there, which is every pull request from a fork."
  else
    echo "NOT WRITTEN HERE: nothing was written for the code-scanning surface, because SHELL_ANALYSIS_SARIF names no path. This run refuses in place, so a finding is read in the job log and nowhere else."
  fi
  echo

  if [ "$status" -ne 0 ]; then
    echo "::error::shellcheck refused at least one tracked shell file. Each finding is printed above with its file, its line and the rule that raised it."
    return 1
  fi
  echo "Every tracked shell file passed at severity ${SEVERITY}, with the rules named above not refused and every other rule in force."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
