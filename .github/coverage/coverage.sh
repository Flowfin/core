#!/usr/bin/env bash
# Line coverage, measured over the whole tree and pinned on the surface that
# decides security outcomes (#84).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHY THE BAR IS NOT OVER THE WHOLE REPOSITORY. A whole-codebase percentage can
# be met by covering the easy half, and it can be missed by adding a large,
# trivial, well understood module. The bar below is keyed to the modules the
# register names, so a thin non-critical path cannot trip it and a regression on
# the path that matters cannot slip through it. The whole-tree number is measured
# and printed on every run and gates nothing, which is the half that is tracked
# rather than enforced.
#
# WHAT MEASURES IT. The compiler's own instrumentation, `-C instrument-coverage`,
# and the two tools that read what it writes, which arrive with the toolchain as
# the `llvm-tools` component rather than as a dependency. `rust-toolchain.toml`
# is where that component is declared, so a fresh clone gets it without being
# told to install one, and this script finds the tools inside the sysroot the
# pinned compiler reports rather than on the path, where a system LLVM of another
# version would answer.
#
# WHY LINES AND NOT REGIONS OR BRANCHES. A line is the unit somebody reading a
# diff can point at. Regions are finer and move when the compiler's own splitting
# changes, so a bar over them moves at a toolchain upgrade for reasons no commit
# explains. The branch columns this tool prints are empty for this tree today,
# which the run says out loud rather than reporting a bar met over nothing.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules, print the register, measure, and judge
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# The register naming the surface and the modules on it.
REGISTER_FILE="$(dirname "$0")/pinned-surface"

# The bar, in per cent of lines, on the modules the register names.
#
# NINETY SIX AND A HALF, SET FROM THE MEASUREMENT ON THIS TREE. The pull request
# that set it carries the measurement, the room it left, and the run that watched
# a deliberate uncovered function cross it, all at the commit it was set at. The
# room is also printed on every run below, so how close this is does not have to
# be worked out from two percentages.
#
# The gate this board is measured against pins ninety two on its own decision
# surface. That number is a FLOOR this one is above rather than the number copied
# across, and the difference is what makes the bar do anything: taking ninety two
# as it stands would leave five points of headroom over what this surface
# measures, and a deliberate uncovered branch walks under a bar with five points
# of headroom without moving it. #84 states that a deliberate uncovered branch
# reddens the check, so a bar that cannot do that is the wrong bar however well
# the number is defended somewhere else.
#
# WHAT IT IS NOT SET TO IS THE MEASUREMENT ITSELF, and that is the other half.
# Rounding today's number down to two decimal places leaves room for about two
# lines, which is a gate that stops honest work rather than one that catches
# uncovered work. The room this leaves is a small uncovered function rather than
# a stray line, and a change that crosses it is answered by a commit that argues
# the bar down or covers the lines rather than by a run somebody waves through.
SECURITY_LINE_BAR=96.5

# Where the instrumented run writes what it counts, and where the merged profile
# is put. Under the build directory so that a clean build removes both.
COVERAGE_DIR="target/coverage"
PROFILE_DIR="${COVERAGE_DIR}/raw"
MERGED_PROFILE="${COVERAGE_DIR}/merged.profdata"

# The build directory the instrumented run uses. Separate from the ordinary one
# so that a coverage run does not force every later `cargo build` to recompile
# the world with different flags.
INSTRUMENTED_TARGET_DIR="target/coverage-build"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line, as VERDICT<TAB>LINE<TAB>KIND<TAB>PATH<TAB>DETAIL.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is the
# ordinary answer here, and a pipeline that has to tell "nothing matched" from
# "the scanner broke" one `set -o pipefail` at a time is how a gate ends up
# passing on everything.
# --------------------------------------------------------------------------

parse_register() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*$/) next
      if (line ~ /^[ \t]*#/) next

      work = line
      sub(/^[ \t]+/, "", work)

      # Three fields, cut by hand rather than by splitting on whitespace, so
      # that a reason of several words stays one reason.
      i = index(work, " ")
      j = index(work, "\t")
      if (j > 0 && (i == 0 || j < i)) i = j
      if (i == 0) {
        printf "REFUSE\t%d\t?\t%s\tcarries a kind and nothing else\n", FNR, work
        next
      }
      kind = substr(work, 1, i - 1)
      rest = substr(work, i + 1)
      sub(/^[ \t]+/, "", rest)

      i = index(rest, " ")
      j = index(rest, "\t")
      if (j > 0 && (i == 0 || j < i)) i = j
      if (i == 0) {
        printf "REFUSE\t%d\t%s\t%s\tcarries a path and no reason\n", FNR, kind, rest
        next
      }
      name = substr(rest, 1, i - 1)
      reason = substr(rest, i + 1)
      gsub(/^[ \t]+|[ \t]+$/, "", reason)
      if (reason == "") {
        printf "REFUSE\t%d\t%s\t%s\tcarries a path and no reason\n", FNR, kind, name
        next
      }

      if (kind != "area" && kind != "module") {
        printf "REFUSE\t%d\t%s\t%s\tnames a kind this register does not have\n", FNR, kind, name
        next
      }

      # The path is compared against the shape `git ls-files` writes: from the
      # repository root, forward slashes, no leading dot segment. A path written
      # any other way never matches the tracked set, so it would name nothing
      # while reading as though it named something.
      #
      # Three explicit tests rather than one character class. A backslash inside
      # a bracket expression is an escape in one awk and a member of the class in
      # another, so the class that refuses a Windows-shaped path here would admit
      # one on the runner.
      first = substr(name, 1, 1)
      if (index(name, "\\") > 0 || first == "." || first == "/") {
        printf "REFUSE\t%d\t%s\t%s\tis not written the way the tracked set is written\n", FNR, kind, name
        next
      }

      last = substr(name, length(name), 1)
      if (kind == "area" && last != "/") {
        printf "REFUSE\t%d\t%s\t%s\tis an area that does not end in a separator, so it names a file rather than a directory\n", FNR, kind, name
        next
      }
      if (kind == "module" && (last == "/" || name !~ /\.rs$/)) {
        printf "REFUSE\t%d\t%s\t%s\tis a module that is not a source file\n", FNR, kind, name
        next
      }

      printf "ALLOW\t%d\t%s\t%s\t%s\n", FNR, kind, name, reason
    }
  '
}

# Whether a path lies under one of the areas. Reads the areas on stdin, one per
# line, and takes the path as an argument.
under_an_area() {
  awk -v subject="$1" '
    {
      area = $0
      if (area == "") next
      if (substr(subject, 1, length(area)) == area) { found = 1; exit }
    }
    END { exit !found }
  '
}

# The line accounting for a set of paths, out of the table `llvm-cov report`
# writes.
#
# Fields 8 and 9 of a data row are the line count and the missed line count. The
# columns after them are the branch columns, which this tool prints as a dash
# for a tree with no branch data, so reading only these two is what keeps the
# parse from depending on whether they are there.
#
# The first field is normalised before it is compared: the separator differs by
# platform and the tool may write the path absolutely, so the repository root is
# stripped where it is present.
report_lines() {
  awk -v root="$1" -v prefixes="$2" '
    BEGIN { n = split(prefixes, wanted, ",") }
    {
      name = $1
      gsub("\\\\", "/", name)
      if (root != "" && substr(name, 1, length(root)) == root) {
        name = substr(name, length(root) + 1)
      }
      sub(/^\.\//, "", name)
      if (name == "TOTAL" || name == "Filename") next
      if ($8 !~ /^[0-9]+$/ || $9 !~ /^[0-9]+$/) next
      for (i = 1; i <= n; i++) {
        p = wanted[i]
        if (p == "") continue
        if (substr(name, 1, length(p)) == p) {
          lines += $8
          missed += $9
          break
        }
      }
    }
    END { printf "%d %d\n", lines + 0, missed + 0 }
  '
}

# The line accounting for one named path, so that a run can print what each
# module on the surface contributed rather than only the total.
report_one() {
  awk -v root="$1" -v subject="$2" '
    {
      name = $1
      gsub("\\\\", "/", name)
      if (root != "" && substr(name, 1, length(root)) == root) {
        name = substr(name, length(root) + 1)
      }
      sub(/^\.\//, "", name)
      if (name != subject) next
      if ($8 !~ /^[0-9]+$/ || $9 !~ /^[0-9]+$/) next
      printf "%d %d\n", $8, $9
      found = 1
      exit
    }
    END { if (!found) printf "0 0\n" }
  '
}

# Whether a covered-over-total ratio meets the bar, as integers, because the
# shell has no fractions and a bar compared with a string sort passes 9.0.
meets_the_bar() {
  awk -v lines="$1" -v missed="$2" -v bar="$3" '
    BEGIN {
      if (lines <= 0) exit 1
      covered = lines - missed
      exit !(covered * 10000 >= bar * 100 * lines)
    }
  '
}

# The percentage, for printing only. Nothing decides anything on this string.
as_percent() {
  awk -v lines="$1" -v missed="$2" '
    BEGIN {
      if (lines <= 0) { print "-"; exit }
      printf "%.2f", (lines - missed) * 100 / lines
    }
  '
}

# How many more uncovered lines this surface admits before it is under the bar.
#
# Printed on every run rather than left to be worked out from two percentages,
# because how close a bar is IS the thing a reader wants and a percentage two
# decimal places wide hides it. A bar with room in it refuses nothing anybody
# would write; a bar with none stops honest work. This number is what says which
# of the two is in force today.
headroom() {
  awk -v lines="$1" -v missed="$2" -v bar="$3" '
    BEGIN {
      if (lines <= 0 || bar <= 0) { print 0; exit }
      covered = lines - missed
      admits = int(covered * 10000 / (bar * 100))
      room = admits - lines
      # Parenthesised because the redirection operator and the conditional share
      # a character here: `print a > b ? c : d` writes to a file named by the
      # comparison in every awk this gate meets.
      print (room > 0 ? room : 0)
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the register or the tree. A row
# that judged the real ones would prove the state of the tree on the day it ran,
# not the rule.
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
  printf '%s' "$1" | parse_register
}

judge_report() {
  printf '%s' "$1" | report_lines "$2" "$3"
}

judge_bar() {
  if meets_the_bar "$1" "$2" "$3"; then echo "MET"; else echo "BELOW"; fi
}

judge_under() {
  if printf '%s' "$2" | under_an_area "$1"; then echo "UNDER"; else echo "OUTSIDE"; fi
}

# A table in the shape `llvm-cov report` writes one, with the separator and the
# absolute path a run on this machine produces, so that the parse is proven
# against the awkward shape rather than the convenient one.
A_WINDOWS_TABLE='Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover
--------------------------------------------------------------------------------------------------------------------------------------------------
G:\repo\src\session\mod.rs        198                 2    98.99%          22                 0   100.00%         115                 2    98.26%
G:\repo\src\clock\mod.rs           79                 0   100.00%          12                 0   100.00%          57                 0   100.00%
--------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                             277                 2    99.28%          34                 0   100.00%         172                 2    98.84%'

A_RUNNER_TABLE='Filename                Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
src/session/mod.rs          198                 2    98.99%          22                 0   100.00%         115                 2    98.26%           0                 0         -
src/cache/bound.rs         2028                45    97.78%          99                 5    94.95%        1054                29    97.25%           0                 0         -
src/measurement/mod.rs      519                37    92.87%          44                 5    88.64%         304                25    91.78%           0                 0         -
TOTAL                      2745                84    96.94%         165                10    93.94%        1473                56    96.20%           0                 0         -'

selftest() {
  echo "== a register line with a kind, a path and a reason is admitted =="
  assert_out "passes: an area" \
    "$(printf 'ALLOW\t1\tarea\tsrc/session/\tholding a session and the secret a client stores')" \
    "$(judge_register 'area    src/session/    holding a session and the secret a client stores
')"
  assert_out "passes: a module, with a reason of several words kept whole" \
    "$(printf 'ALLOW\t1\tmodule\tsrc/session/mod.rs\tthe session handle and the store interface a client implements')" \
    "$(judge_register 'module  src/session/mod.rs  the session handle and the store interface a client implements
')"

  echo "== what is refused =="
  assert_out "bites: a path with no reason, which is a module nobody argued with" \
    "$(printf 'REFUSE\t1\tmodule\tsrc/session/mod.rs\tcarries a path and no reason')" \
    "$(judge_register 'module  src/session/mod.rs
')"
  assert_out "bites: a path with trailing space and nothing after it" \
    "$(printf 'REFUSE\t1\tmodule\tsrc/session/mod.rs\tcarries a path and no reason')" \
    "$(judge_register "$(printf 'module  src/session/mod.rs   \n')")"
  assert_out "bites: a kind and nothing else" \
    "$(printf 'REFUSE\t1\t?\tmodule\tcarries a kind and nothing else')" \
    "$(judge_register 'module
')"
  assert_out "bites: a kind this register does not have" \
    "$(printf 'REFUSE\t1\tsurface\tsrc/session/\tnames a kind this register does not have')" \
    "$(judge_register 'surface src/session/  it reads like a word this file would use and is not one
')"
  assert_out "bites: an area that names a file, so nothing new under it is noticed" \
    "$(printf 'REFUSE\t1\tarea\tsrc/session/mod.rs\tis an area that does not end in a separator, so it names a file rather than a directory')" \
    "$(judge_register 'area    src/session/mod.rs  the session module
')"
  assert_out "bites: a module that names a directory" \
    "$(printf 'REFUSE\t1\tmodule\tsrc/session/\tis a module that is not a source file')" \
    "$(judge_register 'module  src/session/  the whole of it
')"
  assert_out "bites: a module that is not a source file" \
    "$(printf 'REFUSE\t1\tmodule\tsrc/session/notes.md\tis a module that is not a source file')" \
    "$(judge_register 'module  src/session/notes.md  it is prose rather than code
')"
  assert_out "bites: a path written relative to the register rather than to the root" \
    "$(printf 'REFUSE\t1\tmodule\t./src/lib.rs\tis not written the way the tracked set is written')" \
    "$(judge_register 'module  ./src/lib.rs  it looks like the same file and matches nothing
')"
  assert_out "bites: a path written with the other separator, which no tracked name carries" \
    "$(printf 'REFUSE\t1\tmodule\tsrc\\session\\mod.rs\tis not written the way the tracked set is written')" \
    "$(judge_register 'module  src\session\mod.rs  it is the same file on Windows and matches nothing here
')"

  echo "== what is not a line =="
  assert_out "passes over: a comment, which is most of the register" \
    "" "$(judge_register '# The surface a coverage bar is pinned to.
')"
  assert_out "passes over: a blank line" "" "$(judge_register '
')"

  echo "== a path is inside an area or it is not =="
  assert_out "reads: a module under one of the areas" "UNDER" \
    "$(judge_under 'src/session/mod.rs' 'src/session/
src/cache/')"
  assert_out "reads: a module under none of them" "OUTSIDE" \
    "$(judge_under 'src/clock/mod.rs' 'src/session/
src/cache/')"
  assert_out "bites: a name that only starts like an area, which is the near miss" "OUTSIDE" \
    "$(judge_under 'src/sessions-elsewhere/mod.rs' 'src/session/mod.rs
src/cache/')"

  echo "== the table is read the same way on either platform =="
  assert_out "reads: a runner table, one prefix" "115 2" \
    "$(judge_report "$A_RUNNER_TABLE" "" "src/session/")"
  assert_out "reads: a runner table, two prefixes summed" "1169 31" \
    "$(judge_report "$A_RUNNER_TABLE" "" "src/session/,src/cache/")"
  assert_out "reads: the whole tree, and the TOTAL row is not counted twice" "1473 56" \
    "$(judge_report "$A_RUNNER_TABLE" "" "src/")"
  assert_out "reads: a table written with the other separator and an absolute path" "115 2" \
    "$(judge_report "$A_WINDOWS_TABLE" "G:/repo/" "src/session/")"
  assert_out "reads: nothing, where no row matches" "0 0" \
    "$(judge_report "$A_RUNNER_TABLE" "" "src/artwork/")"

  echo "== the bar is a number and not a string =="
  assert_out "passes: exactly the bar" "MET" "$(judge_bar 100 8 92.0)"
  assert_out "bites: one line under it" "BELOW" "$(judge_bar 100 9 92.0)"
  assert_out "bites: nine per cent, which sorts above ninety two as a string" "BELOW" \
    "$(judge_bar 100 91 92.0)"
  assert_out "bites: nothing measured at all, which is not a bar met" "BELOW" \
    "$(judge_bar 0 0 92.0)"
  assert_out "passes: everything covered" "MET" "$(judge_bar 100 0 92.0)"

  echo "== the room left is counted rather than eyeballed =="
  assert_out "counts: a surface exactly at the bar has none" "0" "$(headroom 100 8 92.0)"
  assert_out "counts: a surface above it has what it has" "8" "$(headroom 100 0 92.0)"
  assert_out "counts: a surface under it has none rather than a negative" "0" "$(headroom 100 91 92.0)"
  assert_out "counts: nothing measured leaves no room" "0" "$(headroom 0 0 92.0)"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures coverage-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

llvm_tool() {
  local sysroot host tool="$1"
  sysroot="$(rustc --print sysroot)"
  host="$(rustc -vV | awk '/^host:/ { print $2 }')"
  local candidate="${sysroot}/lib/rustlib/${host}/bin/${tool}"
  if [ -x "$candidate" ]; then printf '%s\n' "$candidate"; return 0; fi
  if [ -x "${candidate}.exe" ]; then printf '%s\n' "${candidate}.exe"; return 0; fi
  return 1
}

check() {
  local records refusals=0 areas="" modules="" prefixes="" root
  local verdict line kind name detail
  local profdata cov table
  local tracked missing=0
  local src_lines src_missed pin_lines pin_missed

  if [ ! -f "$REGISTER_FILE" ]; then
    echo "::error::${REGISTER_FILE} does not exist. An absent register is not an empty one, and this run will not pass in place of reading it."
    return 1
  fi

  echo "-- the surface the bar is pinned to"
  records="$(parse_register < "$REGISTER_FILE")"
  while IFS=$'\t' read -r verdict line kind name detail; do
    [ -n "${verdict:-}" ] || continue
    case "$verdict" in
      ALLOW)
        case "$kind" in
          area)
            if [ -z "$(git ls-files -- "${name}*.rs")" ]; then
              refusals=$((refusals + 1))
              echo "::error file=${REGISTER_FILE},line=${line}::${REGISTER_FILE}:${line}: ${name} names no tracked source file"
              echo "      ${REGISTER_FILE}:${line}: ${name} names no tracked source file"
            else
              areas="${areas}${name}"$'\n'
              prefixes="${prefixes}${name},"
              echo "      area   ${name}: ${detail}"
            fi
            ;;
          module)
            if git ls-files --error-unmatch -- "$name" > /dev/null 2>&1; then
              modules="${modules}${name}"$'\n'
              echo "      module ${name}: ${detail}"
            else
              refusals=$((refusals + 1))
              echo "::error file=${REGISTER_FILE},line=${line}::${REGISTER_FILE}:${line}: ${name} names no tracked file"
              echo "      ${REGISTER_FILE}:${line}: ${name} names no tracked file"
            fi
            ;;
        esac
        ;;
      REFUSE)
        refusals=$((refusals + 1))
        echo "::error file=${REGISTER_FILE},line=${line}::${REGISTER_FILE}:${line}: ${name} ${detail}"
        echo "      ${REGISTER_FILE}:${line}: ${name} ${detail}"
        ;;
    esac
  done <<RECORDS
$records
RECORDS

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} line(s) of ${REGISTER_FILE} were refused. A register the gate cannot read is not a register it may skip."
    return 1
  fi
  if [ -z "$areas" ]; then
    echo "::error::${REGISTER_FILE} names no area. A bar pinned to nothing is met by everything."
    return 1
  fi
  if [ -z "$modules" ]; then
    echo "::error::${REGISTER_FILE} names no module. A bar measured over nothing is met by everything."
    return 1
  fi
  echo

  echo "-- every module the register names is on an area it names"
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    if printf '%s' "$areas" | under_an_area "$name"; then continue; fi
    refusals=$((refusals + 1))
    echo "::error file=${REGISTER_FILE}::${name} is listed as a module and lies under no area, so nothing here would notice it going missing"
    echo "      ${name} lies under no area"
  done <<MODULES
$modules
MODULES
  if [ "$refusals" -eq 0 ]; then
    echo "      every one of them does"
  fi
  echo

  echo "-- every tracked source file on those areas is listed"
  tracked="$(git ls-files -- '*.rs')"
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    if ! printf '%s' "$areas" | under_an_area "$name"; then continue; fi
    if printf '%s' "$modules" | grep -qxF -- "$name"; then continue; fi
    missing=$((missing + 1))
    refusals=$((refusals + 1))
    echo "::error file=${REGISTER_FILE}::${name} is on the pinned surface and is not listed in ${REGISTER_FILE}. Add it with the reason it is there, or move it off the surface."
    echo "      ${name} is on the surface and is not listed"
  done <<TRACKED
$tracked
TRACKED
  if [ "$missing" -eq 0 ]; then
    echo "      every one of them is"
  fi

  if [ "$refusals" -ne 0 ]; then
    echo
    echo "::error::${refusals} module(s) do not agree with ${REGISTER_FILE}. A surface list maintained by memory is a list that is wrong within a month, which is why this is a refusal and not a note."
    return 1
  fi
  echo

  echo "-- what measured it"
  rustc --version
  if ! profdata="$(llvm_tool llvm-profdata)" || ! cov="$(llvm_tool llvm-cov)"; then
    echo "::error::The llvm-tools component is not in the sysroot this compiler reports. rust-toolchain.toml declares it, so a clone that ran rustup once has it; install it with 'rustup component add llvm-tools'. This run measured nothing and will not pass in place of measuring."
    return 1
  fi
  echo "      ${profdata}"
  echo "      ${cov}"
  echo

  echo "-- the instrumented run"
  rm -rf "$COVERAGE_DIR"
  mkdir -p "$PROFILE_DIR"
  # An absolute pattern, because the instrumented binaries are started with the
  # working directory the harness chose rather than this one.
  RUSTFLAGS="-C instrument-coverage" \
  LLVM_PROFILE_FILE="$(pwd)/${PROFILE_DIR}/%p-%m.profraw" \
  CARGO_TARGET_DIR="$INSTRUMENTED_TARGET_DIR" \
    cargo test --locked --lib --tests
  echo

  echo "-- what it counted"
  local profiles
  profiles="$(find "$PROFILE_DIR" -type f -name '*.profraw' | wc -l | tr -d ' ')"
  if [ "$profiles" -eq 0 ]; then
    echo "::error::The instrumented run wrote no profile at all. Something ran and nothing was counted, which is not a bar met."
    return 1
  fi
  # The glob is what this line is for, and quoting it would hand the tool one
  # argument with a star in it.
  # shellcheck disable=SC2086
  "$profdata" merge -sparse ${PROFILE_DIR}/*.profraw -o "$MERGED_PROFILE"
  echo "      ${profiles} profile(s) merged into ${MERGED_PROFILE}"

  local objects="" binary object_count=0
  while IFS= read -r binary; do
    [ -n "$binary" ] || continue
    objects="${objects} --object ${binary}"
    object_count=$((object_count + 1))
  done <<BINARIES
$(RUSTFLAGS="-C instrument-coverage" CARGO_TARGET_DIR="$INSTRUMENTED_TARGET_DIR" \
    cargo test --locked --lib --tests --no-run --message-format=json 2>/dev/null \
  | awk '
      # A message with no executable, or one whose executable is null, is a
      # compilation that produced no test binary. Matching the opening QUOTE is
      # what tells those apart: without it the first substitution leaves the
      # whole message behind and the second truncates it to a brace, which then
      # reaches the tool as a file name and fails the run for the wrong reason.
      /"executable":"/ {
        if ($0 !~ /"test":true/) next
        line = $0
        sub(/.*"executable":"/, "", line)
        sub(/".*/, "", line)
        if (line == "") next
        gsub(/\\\\/, "/", line)
        print line
      }
    ')
BINARIES

  if [ -z "$objects" ]; then
    echo "::error::No instrumented test binary was found to report against. This run measured nothing."
    return 1
  fi
  echo "      ${object_count} test binary/binaries reported against"
  echo

  # The root the tool's paths are stripped of, with the separator normalised by
  # the shell's own substitution. A `tr` with a backslash in single quotes does
  # the same thing and the analyser reads it as somebody trying to escape a
  # quote, which is a note on every run for a line that was right.
  root="$(pwd)"
  root="${root//\\//}/"
  # Word splitting on $objects is what this line is for: it holds the list built
  # above, and quoting it would hand the tool one argument with spaces in it.
  # shellcheck disable=SC2086
  table="$("$cov" report --instr-profile="$MERGED_PROFILE" $objects \
    --ignore-filename-regex='(/rustc/|[.]cargo/registry/)')"

  echo "-- the whole tree, reported and tracked rather than gating"
  read -r src_lines src_missed <<SRC
$(printf '%s' "$table" | report_lines "$root" "src/")
SRC
  echo "      src/  ${src_lines} line(s), ${src_missed} missed, $(as_percent "$src_lines" "$src_missed")%"
  echo

  echo "-- the pinned surface, module by module"
  local one_lines one_missed
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    read -r one_lines one_missed <<ONE
$(printf '%s' "$table" | report_one "$root" "$name")
ONE
    if [ "$one_lines" -eq 0 ]; then
      echo "      ${name}: no executable line, so it neither raises nor lowers the number below"
    else
      echo "      ${name}: ${one_lines} line(s), ${one_missed} missed, $(as_percent "$one_lines" "$one_missed")%"
    fi
  done <<MODULES2
$modules
MODULES2
  echo

  echo "-- the verdict"
  read -r pin_lines pin_missed <<PIN
$(printf '%s' "$table" | report_lines "$root" "${prefixes}")
PIN
  echo "      bar    ${SECURITY_LINE_BAR}% of lines on the pinned surface"
  echo "      actual $(as_percent "$pin_lines" "$pin_missed")% (${pin_lines} line(s), ${pin_missed} missed)"
  echo "      room   $(headroom "$pin_lines" "$pin_missed" "$SECURITY_LINE_BAR") more uncovered line(s) on this surface before the bar is crossed"
  if ! meets_the_bar "$pin_lines" "$pin_missed" "$SECURITY_LINE_BAR"; then
    echo "::error::Line coverage on the pinned surface is $(as_percent "$pin_lines" "$pin_missed")%, under the bar of ${SECURITY_LINE_BAR}%. The modules and the areas are in ${REGISTER_FILE} and the bar is in $(basename "$0")."
    return 1
  fi
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: branch coverage. The tool prints the branch columns as a dash for this tree, so a bar over them would be a bar met over nothing."
  echo "NOT MADE HERE: region coverage. Regions move when the compiler's own splitting changes, so a bar over them moves at a toolchain upgrade for reasons no commit explains."
  echo "NOT MADE HERE: whether a covered line is a line anybody asserted anything about. Coverage says a line ran, and a test that runs a line and checks nothing raises this number exactly as much as one that checks everything."
  echo "NOT MADE HERE: the documentation examples. The instrumented run is the library and the integration targets, and the doctest the ordinary suite runs is not in it."
  echo "NOT MADE HERE: the two targets Cargo.toml excludes from the suite. Neither is run by anything here, and .github/excluded-targets/excluded-targets.sh is where they are compiled."
  echo "NOT MADE HERE: whether the areas in the register are the right areas. That the register is complete against the tree is refused above; that it names the surface that decides outcomes is what the review is for."
  echo
  echo "Every module on the pinned surface above is listed, and line coverage over them is at or above the bar."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
