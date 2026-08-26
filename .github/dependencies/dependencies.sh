#!/usr/bin/env bash
# The graph the build resolved is the graph that was committed, and it carries no
# known advisory (#19).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHAT THIS ASKS THAT THE DEPENDENCY REVIEW ON A PULL REQUEST DOES NOT. That
# action reads the diff of the graph against an advisory database. It cannot ask
# whether the graph the build actually resolves is the graph the tree carries,
# because a lockfile a restore would rewrite produces a different graph on every
# machine and the diff is then between two of them. So the first rule here is the
# restore itself, in locked mode, and the advisory scan is the second.
#
# THE ACCOUNTING IS PART OF THE VERDICT RATHER THAN DECORATION. A scanner pointed
# at nothing exits zero and prints a page that reads exactly like a clean scan,
# and the ways that arrives are a moved lockfile, a renamed argument and a tool
# that changed its own default. So the number of packages the scanner says it
# examined is read back and compared against the number the committed lockfile
# declares, and a disagreement is refused rather than reported.
#
# A REGISTRY ENTRY CARRYING NO `source` LINE IS INVISIBLE TO THE SCANNER. Found
# by writing the known-vulnerable fixture below without one and watching the scan
# pass; the same entry with the line restored is refused. The fixture that records
# it asserts the pass on purpose, so the bound is proven rather than described,
# and it turns red on the day the scanner starts matching those entries.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      restore in locked mode, scan the graph, account for what was
#              examined, and refuse
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# The lockfile this check is about, read from the repository root, because the
# whole subject of the check is the committed one.
LOCKFILE="Cargo.lock"

# The manifest the locked restore is run against.
MANIFEST="Cargo.toml"

# The fixture lockfiles the live fixtures below are run against. Never this
# tree's own: a fixture that judged the real graph would prove the state of the
# tree on the day it ran, not the rule.
FIXTURES="$(dirname "$0")/fixtures"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes one line to stdout, or exits
# non-zero when the subject does not carry what it is asked for.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is a verdict
# this script has to tell apart from a scanner that broke, and a pipeline that
# does that one `set -o pipefail` at a time is how a gate ends up passing on
# everything.
# --------------------------------------------------------------------------

# The packages a lockfile declares, as TOTAL<TAB>SOURCED.
#
# Both numbers, because they answer different questions. The total is what the
# scanner's own count is compared against. The sourced count is how many of them
# a registry advisory could match at all, and a graph where the two are far apart
# is a graph most of which nothing scanned.
#
# A package stanza written inside a comment declares nothing. The generated file
# carries two comment lines of its own and a person editing one adds more.
declared_packages() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      sub(/^[ \t]+/, "", line)
      if (line ~ /^#/) next
      if (line ~ /^\[\[package\]\][ \t]*$/) { total = total + 1; in_package = 1; next }
      if (line ~ /^\[/) { in_package = 0; next }
      if (!in_package) next
      if (line ~ /^source[ \t]*=/) sourced = sourced + 1
    }
    END { printf "%d\t%d\n", total, sourced }
  '
}

# The number of packages the scanner says it examined, out of its own accounting
# line. Absent is not zero: a run that printed no accounting at all is a scanner
# that did not get as far as reading a lockfile, and this rule refuses rather
# than handing back a number nobody produced.
scanned_count() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line !~ /for vulnerabilities \([0-9]+ crate dependenc/) next
      rest = line
      sub(/^.*for vulnerabilities \(/, "", rest)
      sub(/ crate dependenc.*$/, "", rest)
      if (rest != "") { print rest; found = 1; exit }
    }
    END { if (!found) exit 1 }
  '
}

# The size of the advisory database the scan was read against, out of the
# scanner's own line. Reported rather than refused: a database is somebody else's
# register, and a number from it is evidence about the run rather than a rule.
advisory_database_size() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line !~ /Loaded [0-9]+ security advisor/) next
      rest = line
      sub(/^.*Loaded /, "", rest)
      sub(/ security advisor.*$/, "", rest)
      if (rest != "") { print rest; found = 1; exit }
    }
    END { if (!found) exit 1 }
  '
}

# --------------------------------------------------------------------------
# The two tools, reached through one function each, so that a fixture and the
# gate cannot come apart on an argument.
# --------------------------------------------------------------------------

# The locked restore. `--locked` is the whole rule: it refuses to rewrite the
# lockfile rather than rewriting it quietly, so a manifest that no longer
# resolves to the committed graph is a failure instead of a diff nobody reads.
restore_locked() {
  cargo metadata --locked --format-version 1 --manifest-path "$1" >/dev/null
}

# The advisory scan over one lockfile. `--file` rather than a working directory,
# so the subject is a path this script names and never whatever the process
# happened to be standing in.
scan_lockfile() {
  cargo audit --file "$1" 2>&1
}

# --------------------------------------------------------------------------
# selftest
#
# The text rules judge their own text. The live fixtures below run the real tools
# against fixture lockfiles and against a fixture crate in a temporary directory,
# and never against this tree.
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

judge_lock() {
  printf '%s' "$1" | declared_packages
}

judge_scan_output() {
  printf '%s' "$1" | scanned_count || echo "REFUSED"
}

text_fixtures() {
  echo "== what a lockfile declares =="
  assert_out "reads: the one package that is this crate, carrying no source of its own" \
    "$(printf '1\t0')" \
    "$(judge_lock '# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "flowfin-core"
version = "0.0.0"
')"
  assert_out "reads: a registry package beside it, counted in both columns" \
    "$(printf '2\t1')" \
    "$(judge_lock '[[package]]
name = "flowfin-core"
version = "0.0.0"

[[package]]
name = "a-registry-package"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
')"
  assert_out "bites: a package stanza written inside a comment, which declares nothing" \
    "$(printf '1\t0')" \
    "$(judge_lock '# [[package]]
# name = "not-a-package"

[[package]]
name = "flowfin-core"
version = "0.0.0"
')"
  assert_out "bites: a source key under the following table, which belongs to no package" \
    "$(printf '1\t0')" \
    "$(judge_lock '[[package]]
name = "flowfin-core"
version = "0.0.0"

[metadata]
source = "somewhere else"
')"
  assert_out "reads: a lockfile declaring no package at all" \
    "$(printf '0\t0')" \
    "$(judge_lock 'version = 4
')"

  echo "== the count the scanner says it examined =="
  assert_out "reads: the accounting line the scanner writes" "2" \
    "$(judge_scan_output '    Scanning /tmp/x.lock for vulnerabilities (2 crate dependencies)
')"
  assert_out "reads: a lockfile path carrying the words the line is matched on" "1" \
    "$(judge_scan_output '    Scanning /home/for vulnerabilities (9)/Cargo.lock for vulnerabilities (1 crate dependencies)
')"
  assert_out "bites: a run that printed no accounting line, which is not a count of zero" "REFUSED" \
    "$(judge_scan_output 'error: couldnt open /tmp/x.lock: No such file or directory
')"
  assert_out "bites: a sentence naming vulnerabilities and carrying no count" "REFUSED" \
    "$(judge_scan_output '    Scanning Cargo.lock for vulnerabilities
error: 1 vulnerability found!
')"
}

# --------------------------------------------------------------------------
# The live fixtures. Each is a one-change neighbour of the one beside it, so that
# a pass proves the rule rather than the absence of a subject.
# --------------------------------------------------------------------------

assert_status() {
  local what="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    printf 'ok    %s\n' "$what"
  else
    printf 'FAIL  %s\n        expected exit: %s\n        actual exit:   %s\n' \
      "$what" "$expected" "$actual"
    selftest_failures=$((selftest_failures + 1))
  fi
}

# The exit status of a command, captured rather than allowed to end this script,
# because a fixture that expects a refusal needs the number.
status_of() {
  local status=0
  "$@" >/dev/null 2>&1 || status=$?
  echo "$status"
}

live_fixtures() {
  local work status

  echo "== the locked restore refuses a lockfile the manifest has outgrown =="
  work="$(mktemp -d)"
  mkdir -p "$work/src"
  : > "$work/src/lib.rs"
  printf '%s\n' \
    '[package]' \
    'name = "drift-fixture"' \
    'version = "0.0.0"' \
    'edition = "2024"' > "$work/Cargo.toml"

  # The lockfile the manifest actually resolves to, written by the build tool
  # rather than by hand, so the agreeing case is the tool's own answer. Offline
  # throughout: the fixture crate has no dependencies, and a fixture that reached
  # a registry would make this gate's verdict depend on somebody else's uptime.
  ( cd "$work" && cargo generate-lockfile --offline >/dev/null 2>&1 )
  status="$(status_of restore_locked "$work/Cargo.toml")"
  assert_status "passes: the lockfile the manifest resolves to" "0" "$status"

  # One field of drift, the one a person leaves behind when they move a version
  # in the manifest and do not commit the lockfile with it.
  sed 's/^version = "0.0.0"$/version = "0.0.1"/' "$work/Cargo.lock" > "$work/drifted"
  mv "$work/drifted" "$work/Cargo.lock"
  status="$(status_of restore_locked "$work/Cargo.toml")"
  assert_status "bites: the same lockfile with the version moved by one" "101" "$status"
  rm -rf "$work"

  echo "== the advisory scan refuses a graph carrying a known advisory =="
  status="$(status_of scan_lockfile "$FIXTURES/carries-a-known-advisory.lock")"
  assert_status "bites: a release with an advisory published against it" "1" "$status"

  status="$(status_of scan_lockfile "$FIXTURES/carries-no-advisory.lock")"
  assert_status "passes: the one-change neighbour, the same graph at the release that carries the fix" "0" "$status"

  echo "== the bound this scan has, proven rather than described =="
  status="$(status_of scan_lockfile "$FIXTURES/carries-a-known-advisory-unsourced.lock")"
  assert_status "passes, and this is the bound: the same release with no source line is not matched" "0" "$status"
}

selftest() {
  text_fixtures
  echo
  live_fixtures

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures dependency-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
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
  local accounting total sourced output status=0 examined loaded

  if [ ! -f "$LOCKFILE" ]; then
    echo "::error::${LOCKFILE} is not committed. Without it the graph is resolved fresh on every build, and no run of this check says anything about what the last one compiled."
    return 1
  fi

  accounting="$(declared_packages < "$LOCKFILE")"
  IFS=$'\t' read -r total sourced <<ACCOUNTING
$accounting
ACCOUNTING

  echo "-- what the committed lockfile declares"
  echo "      ${total} package(s), ${sourced} of them carrying a source a registry advisory could match"
  echo

  if [ "$total" -eq 0 ]; then
    echo "::error::${LOCKFILE} declares no package at all. A lockfile that locks nothing reads exactly like one that locks a clean graph."
    return 1
  fi

  echo "-- the restore, in locked mode"
  if ! restore_locked "$MANIFEST"; then
    echo "::error::${MANIFEST} does not resolve to the committed ${LOCKFILE}. Run 'cargo generate-lockfile' and commit the result, rather than letting each machine resolve a graph of its own."
    return 1
  fi
  echo "      ${MANIFEST} resolves to the committed ${LOCKFILE}, and the restore rewrote nothing"
  echo

  echo "-- the advisory scan"
  if ! command -v cargo-audit >/dev/null 2>&1; then
    echo "::error::cargo-audit is not on this machine, so no advisory scan ran. Install it with 'cargo install cargo-audit --locked'. A check that could not scan is refused rather than passed, because a run that scanned nothing prints a page that reads exactly like a clean one."
    return 1
  fi

  output="$(scan_lockfile "$LOCKFILE")" || status=$?
  printf '%s\n' "$output"
  echo

  if ! examined="$(printf '%s\n' "$output" | scanned_count)"; then
    echo "::error::The scanner printed no accounting line, so how many packages it examined cannot be read. Refusing rather than reading an absent count as zero problems."
    return 1
  fi

  echo "-- what it was read against"
  if loaded="$(printf '%s\n' "$output" | advisory_database_size)"; then
    echo "      ${loaded} advisories in the database this run loaded"
  else
    echo "      the scanner named no database size on this run. Reported rather than refused: the size is somebody else's register and no rule here rests on it."
  fi
  echo

  echo "-- what it examined"
  say "${examined} package(s) examined by the scan, against ${total} declared by ${LOCKFILE}."
  echo

  if [ "$examined" -ne "$total" ]; then
    echo "::error::The scan examined ${examined} package(s) and ${LOCKFILE} declares ${total}. A scanner pointed at a different file, or at a graph resolved somewhere else, exits zero and prints a clean page, which is the failure this comparison exists for."
    return 1
  fi

  if [ "$status" -ne 0 ]; then
    echo "::error::The scan refused the graph. The report above names the package, the advisory, and the release that carries the fix."
    return 1
  fi

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether a dependency was admitted by any clause of docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md. That record says of itself that nothing in this repository reads the line beside a manifest entry, and this check does not become that reader."
  echo "NOT MADE HERE: the licences in the graph. 0103 names the admitted set, and #87 is where the graph is read back as a bill of materials."
  echo "NOT MADE HERE: an advisory nobody has published. The scan reports what a database held on the day it ran, so a graph with no finding is a graph nothing is known against rather than a graph with nothing in it."
  echo "NOT MADE HERE: a package this lockfile declares with no source line. ${sourced} of ${total} carry one, and an entry without one is invisible to the scan, which a fixture in this script proves rather than assumes."
  echo
  echo "${MANIFEST} resolves to the committed lockfile, and the ${examined} package(s) it declares carry no advisory this database knows."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
