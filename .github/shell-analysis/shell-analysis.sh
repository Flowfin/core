#!/usr/bin/env bash
# The shell this gate runs is analysed (#81).
#
# Two of this gate's legs are shell scripts and nothing read them for defects. The
# workflow YAML has an analyser, the repository's own configuration has one, and
# the language the checks are actually written in had none, so the analysed set and
# the executed set had come apart.
#
# The rules are not written here. The rule set is shellcheck's, and what this file
# owns is which files it is pointed at, what severity gates, and the fixtures that
# prove the analyser refuses the defect the two scripts are most exposed to. That
# is the same arrangement the other two legs use: `selftest` and `check` reach the
# analyser through one function, so a fixture that passes and a gate that judges
# something else cannot come apart.
#
# What the check is for: the defect class that made this worth a leg is an
# expansion left unquoted, which word-splits its input and silently changes what a
# rule was pointed at. A scope gate elsewhere in this organisation split its input
# on whitespace and refused valid work for it. Both scripts here read paths and
# pull-request bodies, which are exactly the inputs that carry a space.
#
# Verbs:
#   selftest   prove the analyser refuses that defect, and that the same fixture
#              with the one character repaired is not refused
#   check      run the fixtures, then analyse every tracked shell file, and refuse
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

# Every tracked shell file.
shell_files() {
  git ls-files '*.sh'
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
  shellcheck --format=gcc --severity="$SEVERITY" "$@"
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

  echo "Both fixtures behaved as this check claims, at severity ${SEVERITY}."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local line status=0
  local files=()
  while IFS= read -r line; do
    [ -n "$line" ] && files+=("$line")
  done < <(shell_files)

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

  echo
  echo "-- what this run did not read"
  echo "NOT READ HERE: the core's own language. None is chosen, no code is in this tree, and #11 is where that is decided. This leg covers the shell the gate runs and nothing else."
  echo "NOT READ HERE: the workflow YAML. .github/workflows/zizmor.yml analyses that, and a second analyser over one subject is a separate argument rather than a setting here."
  echo "NOT UPLOADED: nothing here reaches the code-scanning surface. This run refuses in place rather than filing an alert, so a finding is read in the job log and nowhere else."
  echo

  if [ "$status" -ne 0 ]; then
    echo "::error::shellcheck refused at least one tracked shell file. Each finding is printed above with its file, its line and the rule that raised it."
    return 1
  fi
  echo "Every tracked shell file passed at severity ${SEVERITY}, with nothing suppressed by this check."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
