#!/usr/bin/env bash
# Mutation testing over the surface that decides security outcomes (#85).
#
# WHAT IT ASKS THAT NOTHING ELSE HERE ASKS. `.github/test/test.sh` refuses a run
# that collected no tests, and `.github/coverage/coverage.sh` refuses a surface
# whose lines nobody reached. Neither can ask whether a test that reached a line
# would have noticed the line being wrong. A test that calls a function and
# asserts nothing about the answer is green in both of those checks and holds
# nothing. This leg is what says so, by changing the code under the suite and
# reporting which changes the suite failed to notice.
#
# WHY IT IS ON A SCHEDULE AND GATES NOTHING. Every mutant is a build and a test
# run of its own, so the question is far too slow to sit in front of a merge, and
# the gate this board is measured against reached the same placement. That is the
# whole of the deviation: none in placement, and the scope list differs because
# the modules differ.
#
# WHAT A SURVIVOR OBLIGES, AND IT IS NOT A RED GATE. The run publishes its score
# and names every mutant that survived, in the analyser's own words. Each
# survivor obliges an issue on this board carrying that string, the module it is
# in, and what would have caught it. Not a re-run, and not a quiet acceptance: a
# number produced on a schedule that nobody is obliged to answer is a number
# nobody reads, which is the whole of the difference between a reported check and
# an ignored one.
#
# THERE IS NO SUPPRESSION REGISTER HERE, ON PURPOSE, AND THAT IS THE DIFFERENCE
# FROM THE OTHER LEGS. `.github/shell-analysis/excluded-rules` and
# `.github/codeql/excluded-rules` each excuse a rule, and a rule excused there
# refuses nothing anywhere afterwards. Excusing a mutant is not that: it raises
# the published score without any test having been written, so the number moves
# in the direction that looks like progress for the one reason that is not. A
# mutant nobody can kill - because the behaviour it changes is not behaviour this
# core promises - is written into the issue with that reason and the issue is
# closed on it, where a reader meets it beside the survivor rather than in a file
# whose only effect is to make the score go up.
#
# WHERE THE SCOPE COMES FROM. `.github/coverage/pinned-surface`, the same
# register #84 pins its bar to, read for its `area` lines rather than for its
# `module` lines. The areas are what makes the scope hold: every tracked source
# file under one of them is mutated whether or not anybody remembered to add it,
# and a surface list maintained by memory is a list that is wrong within a month.
# That register's own argument for which areas are on it, and which directories
# are deliberately not, is written in it and is not repeated here.
#
# WHAT THE ANALYSER IS. `cargo-mutants`, installed on the runner as a tool rather
# than entering the manifest. `Cargo.toml` and `Cargo.lock` are untouched and the
# crate graph stays empty, so
# `docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md` is not
# what admits it, in the same way that record is not what admits shellcheck or
# the two tools the coverage leg reads counters with. What it costs is a compile
# on the runner, on a leg that gates nothing.
#
# Verbs:
#   selftest   prove that a suite asserting nothing is reported as a suite
#              asserting nothing, that its neighbour one assertion away is not,
#              that a run which tested no mutant is refused rather than scored,
#              that an area naming nothing tracked is refused, and that the
#              directory a run writes into is made under a parent that is not
#              there, that a run answering none of its own mutants is refused,
#              and that a completed run is one of three exit codes
#   check      derive the scope from the register, run the analyser, publish the
#              score with the command that produced it, and list the survivors
#
# `check` reads the repository through `git ls-files`, so the authority for what
# is mutated is the tracked set. A file present on disk and not added is not a
# file this leg runs over.
#
# No POSIX character classes and no interval expressions in any pattern below,
# for the reason `.github/coverage/coverage.sh` already gives about the two awks
# this gate meets.

set -euo pipefail

# The register naming the surface. One file read by two legs, so the surface the
# coverage bar is pinned to and the surface the mutants are generated over cannot
# come apart.
REGISTER_FILE="$(dirname "$0")/../coverage/pinned-surface"

# The version of the analyser this leg expects. Named for the reason
# `rust-toolchain.toml` pins a compiler: a tool that is whatever the runner
# happens to resolve is a score that moves without a commit, and a mutation score
# is exactly the kind of number somebody compares against last month's. The
# workflow installs this version and every run prints the version that actually
# produced its accounting, so the two can be compared rather than assumed equal.
MUTANTS_VERSION=27.1.0

# Where the run writes what it did. Under the build directory so a clean build
# removes it and nothing that reads this repository through git sees it.
OUTPUT_DIR=target/mutation

# The exit codes a COMPLETED run produces. Any other code is a run that did not
# complete, and this leg refuses one rather than reading its absent survivors as
# none.
#
# THIS LIST HELD TWO OF THE THREE UNTIL THE FIRST RUN OVER THIS REPOSITORY. That
# run tested all 219 mutants, wrote its accounting, and exited 3, which is the
# code for a run in which something timed out. The leg refused it. Refusing was
# the right shape and the list was wrong: a timeout is an outcome this score
# already counts outside itself and prints on its own line, so a run carrying one
# is a run with a number rather than a run that did not happen.
#
# WHAT THE THIRD CODE DOES NOT BUY IS A RUN WHOSE VIABLE SET COLLAPSED, and that
# is refused below rather than here. A run in which every mutant timed out also
# exits 3, and its score would be over nothing.
EXIT_NOTHING_SURVIVED=0
EXIT_SOMETHING_SURVIVED=2
EXIT_SOMETHING_TIMED_OUT=3

# The areas the register names, one per line. Only the `area` lines are read
# here. The `module` lines are the coverage bar's subject and this leg does not
# need them, because a file under an area is on this scope by being there.
register_areas() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*$/) next
      if (line ~ /^[ \t]*#/) next

      work = line
      sub(/^[ \t]+/, "", work)

      i = index(work, " ")
      j = index(work, "\t")
      if (j > 0 && (i == 0 || j < i)) i = j
      if (i == 0) next
      kind = substr(work, 1, i - 1)
      if (kind != "area") next

      rest = substr(work, i + 1)
      sub(/^[ \t]+/, "", rest)
      i = index(rest, " ")
      j = index(rest, "\t")
      if (j > 0 && (i == 0 || j < i)) i = j
      if (i == 0) { print rest; next }
      print substr(rest, 1, i - 1)
    }
  ' "$1"
}

# Every tracked source file under one area, in the shape git writes a path.
files_under() {
  git ls-files -- "$1" | awk '/\.rs$/ { print }'
}

# Refuses an area that matches no tracked source file, and says which one.
#
# THIS IS THE RULE THAT STOPS THE SCOPE SHRINKING IN SILENCE. An area renamed in
# the tree and not in the register, or written with a separator the tracked set
# does not use, resolves to nothing. The run then generates fewer mutants, every
# one of them is caught, and the score goes UP. A surface that went missing and a
# suite that improved are the same number, which is why this is refused before
# the run rather than read out of it afterwards.
judge_areas() {
  local register="$1"
  local area found empty=""
  while IFS= read -r area; do
    [ -n "$area" ] || continue
    found="$(files_under "$area")"
    if [ -z "$found" ]; then
      empty="${empty}      ${area}
"
    fi
  done < <(register_areas "$register")

  if [ -n "$empty" ]; then
    echo "::error::The register names an area matching no tracked source file. The run would generate fewer mutants, catch all of them, and report a HIGHER score for a surface that had gone missing."
    printf '%s' "$empty"
    return 1
  fi
  return 0
}

# The scope, as the tracked source files under the areas. The files rather than
# the directories, so what was mutated is printed as a list a reader can compare
# against the tree.
scope_files() {
  local register="$1"
  local area
  while IFS= read -r area; do
    [ -n "$area" ] || continue
    files_under "$area"
  done < <(register_areas "$register") | sort -u
}

# Makes the directory the run writes into, including every parent of it.
#
# THE ANALYSER MAKES ITS OWN OUTPUT DIRECTORY AND NOT THE PARENT OF ONE, AND THE
# DIFFERENCE ONLY SHOWS ON A MACHINE THAT HAS NOT BUILT YET. `target/` is there
# on any machine that has run `cargo build` once, so the absence is invisible
# where this was written and certain on a fresh runner, which is exactly where
# this leg runs. The first real run of it exited 1 with `create output parent
# directory "target/mutation"` and refused, rather than publishing a score it did
# not have.
prepare_output() {
  mkdir -p "$1"
}

# Refuses a run that tested no mutant, whether it wrote an accounting or not.
#
# BOTH SHAPES REACH THE SAME PLACE AND ONE OF THEM IS INVISIBLE. A scope that
# resolved to nothing makes the analyser write no accounting at all and exit
# zero, so a run over an empty set reads exactly like a run that caught
# everything. A suite that fails before any mutant is applied writes an
# accounting saying zero were tested. Neither is a suite that noticed anything,
# and neither may be published as a score.
judge_outcomes() {
  local outcomes="$1"
  local total viable
  if [ ! -f "$outcomes" ]; then
    echo "::error::The run wrote no accounting at ${outcomes}, which is what the analyser does when the scope it was given matched nothing. A run over an empty set exits zero and reads exactly like a run that caught everything."
    return 1
  fi
  total="$(jq -r '.total_mutants // 0' "$outcomes")"
  if [ "$total" -eq 0 ]; then
    echo "::error::The run tested no mutant, so there is no score to publish. The accounting says total_mutants is zero, which is what a suite failing before any mutant is applied leaves behind."
    return 1
  fi
  viable="$(jq -r '(.caught // 0) + (.missed // 0)' "$outcomes")"
  if [ "$viable" -eq 0 ]; then
    echo "::error::The run generated ${total} mutant(s) and not one of them was a question the suite could answer: every one either failed to compile or timed out. There is no score over that, and a percentage of nothing is not a clean run."
    return 1
  fi
  return 0
}

# Refuses an exit code that is neither of the two a completed run produces.
judge_exit() {
  local status="$1"
  case "$status" in
    "$EXIT_NOTHING_SURVIVED" | "$EXIT_SOMETHING_SURVIVED" | "$EXIT_SOMETHING_TIMED_OUT") return 0 ;;
    *)
      echo "::error::The analyser exited ${status}, which is none of ${EXIT_NOTHING_SURVIVED} for a run where nothing survived, ${EXIT_SOMETHING_SURVIVED} for one where something did, or ${EXIT_SOMETHING_TIMED_OUT} for one where something timed out. A run that did not complete has no survivors to report, and reporting none would be wrong in the direction nobody checks."
      return 1
      ;;
  esac
}

# The score, in per cent of viable mutants caught, to two decimal places.
#
# THE DENOMINATOR IS CAUGHT PLUS MISSED AND NOT THE TOTAL. A mutant that does not
# compile was never a question the suite could answer, and counting one as caught
# would raise the score for a change that made the code less mutable rather than
# better tested. Unviable and timed-out mutants are printed on their own lines
# instead, so a run whose viable set collapsed says so.
score() {
  jq -r '
    (.caught // 0) as $caught
    | (.missed // 0) as $missed
    | ($caught + $missed) as $viable
    | if $viable == 0 then "none" else ($caught * 10000 / $viable | round / 100 | tostring) end
  ' "$1"
}

# The mutants the suite did not notice, one per line, in the analyser's own
# words, so an issue opened for one carries the string the run printed rather
# than somebody's paraphrase of it.
survivors() {
  jq -r '[ .outcomes[]? | select(.summary == "MissedMutant") | .scenario.Mutant.name ] | .[]' "$1"
}

# One line onto the run summary where there is one, and onto the log always, so a
# person reading either sees the same thing.
say() {
  echo "$1"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "$1" >> "$GITHUB_STEP_SUMMARY"
  fi
}

# --------------------------------------------------------------------------
# selftest
#
# Four fixtures. The first two are one assertion apart and are the whole reason
# this leg exists: the same lines, reached by the same test, scoring differently
# because one of them checks the answer. The third and fourth are the two shapes
# of a run that measured nothing.
# --------------------------------------------------------------------------

# Set here rather than inside `selftest` because the trap that removes it runs
# after that function has returned, and a name local to the function is gone by
# then. `.github/shell-analysis/shell-analysis.sh` carries the same note beside
# the same arrangement, having reached it the same way.
FIXTURE_DIR=""

remove_fixtures() {
  if [ -n "${FIXTURE_DIR:-}" ]; then
    rm -rf "$FIXTURE_DIR"
  fi
}

# One crate: its directory, its package name, and the body of its test.
# Everything else is identical between the two fixtures, so the difference a run
# reports is the difference the fixture carries and nothing else.
write_crate() {
  local dir="$1" name="$2" body="$3"
  mkdir -p "${dir}/src"
  cat > "${dir}/Cargo.toml" <<EOF
[package]
name = "${name}"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
EOF
  cat > "${dir}/src/lib.rs" <<EOF
pub fn is_even(n: u32) -> bool {
    n % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::is_even;

    #[test]
    fn it_answers_for_both_parities() {
${body}
    }
}
EOF
}

fixture_dir() {
  local dir="$1"

  # The suite that checks the answers it asked for.
  write_crate "${dir}/asserted" fixture_asserted \
    '        assert!(is_even(4));
        assert!(!is_even(5));'

  # The same suite with the two assertions replaced by two calls. Every line of
  # the function is still reached, so line coverage over the two crates is
  # identical, and that identity is the difference no coverage number can see.
  write_crate "${dir}/unasserted" fixture_unasserted \
    '        let _ = is_even(4);
        let _ = is_even(5);'

  # A crate offering the analyser nothing to change.
  mkdir -p "${dir}/nothing/src"
  cat > "${dir}/nothing/Cargo.toml" <<'MANIFEST'
[package]
name = "fixture_nothing"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
MANIFEST
  cat > "${dir}/nothing/src/lib.rs" <<'SOURCE'
pub struct Nothing;
SOURCE

  # A register whose second area names a directory this tree does not track.
  cat > "${dir}/register-empty-area" <<'REGISTER'
# A register whose second area matches nothing in the tracked set.
area    src/session/    the reason this one carries
area    src/sesssion/   the reason this one carries, with the directory misspelt
REGISTER

  cat > "${dir}/register-answered" <<'REGISTER'
# The same register with the misspelt area corrected, and nothing else changed.
area    src/session/    the reason this one carries
area    src/server/     the reason this one carries
REGISTER

  # An accounting from a run that generated mutants and answered none of them:
  # every one either failed to compile or timed out. Written rather than produced,
  # because arranging a real one costs a crate whose every mutant hangs.
  cat > "${dir}/outcomes-nothing-viable.json" <<'ACCOUNTING'
{ "outcomes": [], "total_mutants": 12, "caught": 0, "missed": 0, "unviable": 7, "timeout": 5 }
ACCOUNTING

  # The same accounting with one mutant answered, and nothing else changed.
  cat > "${dir}/outcomes-one-viable.json" <<'ACCOUNTING'
{ "outcomes": [], "total_mutants": 12, "caught": 1, "missed": 0, "unviable": 6, "timeout": 5 }
ACCOUNTING
}

# Runs the analyser over one fixture crate and prints its exit code, so the
# caller reads both the accounting it wrote and the code it left with.
run_fixture() {
  local dir="$1" out="$2"
  local status=0
  cargo mutants --dir "$dir" --output "$out" > "${out}.log" 2>&1 || status=$?
  echo "$status"
}

selftest() {
  local dir status score_asserted score_unasserted missed findings exit_status
  local exit_status
  FIXTURE_DIR="$(mktemp -d)"
  dir="$FIXTURE_DIR"
  trap remove_fixtures EXIT
  fixture_dir "$dir"

  echo "-- a suite that checks its answers notices every change to them"
  status="$(run_fixture "${dir}/asserted" "${dir}/out-asserted")"
  judge_exit "$status" || { cat "${dir}/out-asserted.log"; return 1; }
  judge_outcomes "${dir}/out-asserted/mutants.out/outcomes.json" || return 1
  score_asserted="$(score "${dir}/out-asserted/mutants.out/outcomes.json")"
  if [ "$score_asserted" != "100" ]; then
    echo "::error::The fixture whose test checks its answers scored ${score_asserted} rather than 100. Either the analyser generated something this fixture cannot answer, or the score is not counting what it says it counts. What survived:"
    survivors "${dir}/out-asserted/mutants.out/outcomes.json"
    return 1
  fi
  echo "ok    scored ${score_asserted}, exit ${status}, nothing survived"
  echo

  echo "-- the same suite with the assertions removed notices none of them"
  status="$(run_fixture "${dir}/unasserted" "${dir}/out-unasserted")"
  judge_exit "$status" || { cat "${dir}/out-unasserted.log"; return 1; }
  judge_outcomes "${dir}/out-unasserted/mutants.out/outcomes.json" || return 1
  score_unasserted="$(score "${dir}/out-unasserted/mutants.out/outcomes.json")"
  if [ "$score_unasserted" = "$score_asserted" ]; then
    echo "::error::The two fixtures scored the same, at ${score_asserted}. They differ by exactly the assertions, so a leg that cannot separate them is measuring coverage a second time and this whole check is worth nothing."
    return 1
  fi
  missed="$(survivors "${dir}/out-unasserted/mutants.out/outcomes.json")"
  if [ -z "$missed" ]; then
    echo "::error::The fixture asserting nothing reported no survivor, so the run has nothing to publish and a reader would be told the suite noticed changes it did not."
    return 1
  fi
  echo "ok    scored ${score_unasserted}, exit ${status}, and the survivors are named:"
  printf '%s\n' "$missed" | sed 's/^/      /'
  echo

  echo "-- a run that tested no mutant is refused rather than scored"
  status="$(run_fixture "${dir}/nothing" "${dir}/out-nothing")"
  if judge_outcomes "${dir}/out-nothing/mutants.out/outcomes.json" > /dev/null 2>&1; then
    echo "::error::A run that tested no mutant was accepted. That run exited ${status}, and accepting it publishes a clean verdict for a surface nothing was measured over."
    return 1
  fi
  echo "ok    refused, and the analyser had exited ${status} on it"
  echo

  echo "-- an area matching no tracked source file is refused"
  findings=0
  judge_areas "${dir}/register-empty-area" > "${dir}/areas.log" 2>&1 || findings=$?
  if [ "$findings" -eq 0 ]; then
    echo "::error::A register naming an area that matches nothing was accepted. The scope would shrink to whatever is left and the score would rise for it."
    return 1
  fi
  if ! grep -q 'src/sesssion/' "${dir}/areas.log"; then
    echo "::error::The register was refused, but the message does not name the area that matched nothing. What it said:"
    cat "${dir}/areas.log"
    return 1
  fi
  echo "ok    refused, and the message names the area that matched nothing"
  echo

  echo "-- the same register with that area corrected is not refused"
  findings=0
  judge_areas "${dir}/register-answered" > "${dir}/areas-ok.log" 2>&1 || findings=$?
  if [ "$findings" -ne 0 ]; then
    echo "::error::The neighbouring register, which differs by one corrected directory, was refused. What it said:"
    cat "${dir}/areas-ok.log"
    return 1
  fi
  echo "ok    not refused"
  echo

  echo "-- a run that answered none of the mutants it generated is refused"
  if judge_outcomes "${dir}/outcomes-nothing-viable.json" > /dev/null 2>&1; then
    echo "::error::An accounting with no viable mutant at all was accepted. The score is over caught plus missed, so publishing that one is publishing a percentage of nothing."
    return 1
  fi
  echo "ok    refused"
  echo

  echo "-- the same accounting with one mutant answered is not refused"
  if ! judge_outcomes "${dir}/outcomes-one-viable.json" > /dev/null 2>&1; then
    echo "::error::The neighbouring accounting, which differs by one caught mutant, was refused. A guard that refuses its own near miss refuses honest work."
    return 1
  fi
  echo "ok    not refused, and it scores $(score "${dir}/outcomes-one-viable.json")"
  echo

  echo "-- a completed run is one of three exit codes, and nothing else is"
  for exit_status in "$EXIT_NOTHING_SURVIVED" "$EXIT_SOMETHING_SURVIVED" "$EXIT_SOMETHING_TIMED_OUT"; do
    if ! judge_exit "$exit_status" > /dev/null 2>&1; then
      echo "::error::Exit code ${exit_status} was refused. That is a run that completed, wrote its accounting, and has a score to publish."
      return 1
    fi
  done
  for exit_status in 1 4; do
    if judge_exit "$exit_status" > /dev/null 2>&1; then
      echo "::error::Exit code ${exit_status} was accepted. That is a run that did not complete, and its absent survivors would be published as none."
      return 1
    fi
  done
  echo "ok    ${EXIT_NOTHING_SURVIVED}, ${EXIT_SOMETHING_SURVIVED} and ${EXIT_SOMETHING_TIMED_OUT} accepted; 1 and 4 refused"
  echo

  # A run rather than a refusal, and it is here because the failure it prevents
  # cannot happen on a machine that has built once. See `prepare_output`.
  echo "-- the directory the run writes into is made, parents and all"
  prepare_output "${dir}/absent-parent/output"
  if [ ! -d "${dir}/absent-parent/output" ]; then
    echo "::error::The output directory was not created under a parent that did not exist. On a fresh runner that is every run, and the analyser refuses to make a parent of its own output directory."
    return 1
  fi
  echo "ok    made under a parent that did not exist"
  echo

  echo "Every fixture behaved as this check claims. The analyser this leg is written against is cargo-mutants ${MUTANTS_VERSION}."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local outcomes="${OUTPUT_DIR}/mutants.out/outcomes.json"
  local files=() argv=() line status=0
  local command_line total caught missed unviable timed_out version the_score

  judge_areas "$REGISTER_FILE" || return 1

  while IFS= read -r line; do
    [ -n "$line" ] && files+=("$line")
  done < <(scope_files "$REGISTER_FILE")

  if [ "${#files[@]}" -eq 0 ]; then
    echo "::error::The register named areas and none of them holds a tracked source file. There is nothing to mutate, which is a change to the tree rather than a pass."
    return 1
  fi

  echo "The areas this run mutates, from ${REGISTER_FILE}:"
  register_areas "$REGISTER_FILE" | sed 's/^/      /'
  echo
  echo "The ${#files[@]} tracked source file(s) under them:"
  printf '      %s\n' "${files[@]}"
  echo

  # Built as an array and printed as one string, so what is published beside the
  # score is the command that ran rather than a sentence describing it.
  argv=(cargo mutants --output "$OUTPUT_DIR")
  for line in "${files[@]}"; do
    argv+=(--file "$line")
  done
  command_line="${argv[*]}"

  echo "The command the score below is published with:"
  echo "      ${command_line}"
  echo
  prepare_output "$OUTPUT_DIR"
  "${argv[@]}" || status=$?

  judge_exit "$status" || return 1
  judge_outcomes "$outcomes" || return 1

  total="$(jq -r '.total_mutants // 0' "$outcomes")"
  caught="$(jq -r '.caught // 0' "$outcomes")"
  missed="$(jq -r '.missed // 0' "$outcomes")"
  unviable="$(jq -r '.unviable // 0' "$outcomes")"
  timed_out="$(jq -r '.timeout // 0' "$outcomes")"
  version="$(jq -r '.cargo_mutants_version // "unknown"' "$outcomes")"
  the_score="$(score "$outcomes")"

  say "## Mutation score: ${the_score} per cent of viable mutants caught"
  say ""
  say "Measured over the areas \`.github/coverage/pinned-surface\` names, by cargo-mutants ${version}, with:"
  say ""
  say "    ${command_line}"
  say ""
  say "    ${total} mutant(s) generated"
  say "    ${caught} caught"
  say "    ${missed} survived"
  say "    ${unviable} did not compile and are outside the score"
  say "    ${timed_out} timed out and are outside the score"
  say ""

  if [ "$missed" -eq 0 ]; then
    say "No mutant survived this run."
  else
    say "The mutants this suite did not notice. Each obliges an issue on this board carrying the line as written here, the module it is in, and what would have caught it. This run gates nothing, and there is no register that excuses one:"
    say ""
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      say "    ${line}"
    done < <(survivors "$outcomes")
  fi
  say ""

  echo "-- what this run did not measure"
  echo "NOT MEASURED HERE: the directories the register deliberately leaves off the surface. Nothing outside the areas printed above carried a mutant in this run, and .github/coverage/pinned-surface is where that choice is argued rather than here."
  echo "NOT MEASURED HERE: whether a mutant that died was noticed by a test that meant to notice it. A mutant dies to any failing test in the run, so a caught one says the suite reacted and never says which assertion reacted."
  echo "NOT MEASURED HERE: the two targets Cargo.toml keeps out of \`cargo test\`. The analyser runs that same command, so neither the real-server harness nor the deliberate race was a test any mutant here was put in front of."
  echo "NOT DECIDED HERE: whether a survivor is worth a test. Every one of them is published above and each obliges an issue, and this run makes no judgement about which of them the core should care about."
  echo
  say "This leg gates nothing. A falling score obliges an issue and never a red gate, which is what taking the question off the merge path costs and what paying it looks like."

  return 0
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)
    echo "usage: $0 selftest|check" >&2
    exit 2
    ;;
esac
