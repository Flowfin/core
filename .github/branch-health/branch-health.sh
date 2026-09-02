#!/usr/bin/env bash
# A workflow standing red on the default branch, reported (#90).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement every other script in this gate already uses.
#
# WHAT THIS IS FOR. A check on a pull request has somebody waiting on it, so it
# is noticed the hour it goes red. A run that only happens after a merge or on a
# schedule has nobody waiting on it at all, and it can stay red for weeks with
# every pull request green beside it. This tree has such runs already rather than
# expecting them later: the mutation score, the supply-chain self-audit and every
# leg carrying a push trigger conclude on the default branch, and nothing in this
# repository reads a single one of those conclusions.
#
# THE VERDICT IS OVER THE LATEST CONCLUSION PER WORKFLOW AND NOT OVER EVERY RUN
# THE READ CARRIED. A workflow that failed once and has concluded success since is
# not standing red, and reporting it would train a reader to close this report
# unread. The three cancellations on `4bcdf97` are the shape that forced the
# distinction: two merges landed in the same moment, the second cancelled the
# first, and three superseded cancellations sat on the default branch for weeks
# while nothing was wrong. They are out of the verdict by having a later success,
# and the run still NAMES them, under the history heading, so the exclusion is a
# sentence a reader can disagree with rather than a filter nobody sees.
#
# IT REPORTS AND IT DOES NOT REFUSE, AND THE TWO ARE SEPARATED ON PURPOSE. A red
# workflow does not fail this run; it opens an issue. What fails this run is
# being unable to produce a report at all - the listing could not be read, the
# rows came back in a shape this script does not parse, a workflow could not be
# asked about at all. So a green verdict here asserts exactly one thing,
# that a report exists for this run, which is what separates a reporting leg
# whose green means something from one that is green whatever it found. That
# posture was argued on #26 against the one leg in this tree whose green asserts
# nothing, and it is taken here rather than invented.
#
# IT WOULD OTHERWISE REPORT ITSELF FOREVER. A run that failed because it found
# something red would conclude `failure` on the default branch, and the next run
# would read that conclusion and find this workflow standing red. The loop is
# closed by the posture above rather than by an exception carved out for this
# workflow's own name, because a name-shaped exception is one rename away from
# hiding a real failure of this leg.
#
# THE READ IS PER WORKFLOW AND NOT ONE READ OF THE WHOLE BRANCH, WHICH IS A
# CORRECTION RATHER THAN A PREFERENCE. The branch-wide listing answers a hundred
# rows against a total in the hundreds when it is not paged, and the rows it
# drops are the oldest, so the question gets quieter as the branch collects runs
# - the opposite of what a report on unwatched conclusions is for. Paging it does
# not fix that here: the platform stops handing over rows at a thousand, and this
# branch already carries more than a thousand runs in a month, so a paged read of
# a dated window refuses with the same rows missing. Read per workflow instead
# and neither bound is in the way: each workflow is asked for its own most recent
# runs on the branch, which is a listing of tens, and the verdict needs only the
# newest conclusion in it.
#
# A WORKFLOW WITH NO RUN ON THE BRANCH IS NAMED AS ONE THIS RUN DID NOT EXAMINE.
# The registry and the runs are two different sets: a workflow that fires only on
# a pull request never concludes on the default branch at all. A list of examined
# names with no line saying which of the registry it could not see is read as the
# whole registry examined and found healthy, which is the reading this leg is
# against, so the registry is what this run walks and every entry gets a line.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      read the default branch's runs, report, and open an issue per
#              workflow standing red
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# How many of each workflow's most recent runs on the default branch this run
# reads. The verdict needs the newest one that carried a conclusion, so the
# number only has to be larger than the number of runs of one workflow that can
# be in flight at once; the rest of what it buys is the recent history the report
# prints. It is a constant here rather than an input, because a depth chosen per
# run is a depth nobody can compare two runs against.
RUNS_PER_WORKFLOW=20

# What an opened issue carries. A label, because every issue on this board has
# one, and an assignee, because an issue nobody holds is an issue nobody reads.
# No milestone is set, and the run prints that absence rather than leaving it to
# be noticed: which milestone a red workflow belongs to is a judgement about the
# plan, and this run has nothing to make it with.
ISSUE_LABEL="ci"
ISSUE_ASSIGNEE="iderex"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line. A run row is six tab-separated fields:
#
#     workflow name, event, conclusion, created at, head sha, run id
#
# and the conclusion is empty for a run that has not concluded yet.
#
# awk rather than grep: grep exits 1 when it selects nothing, and "no workflow on
# this branch is standing red" is a legitimate answer this script has to tell
# apart from a reader that broke.
# --------------------------------------------------------------------------

# One workflow's listing, turned into rows carrying the name the REGISTRY gives
# that workflow rather than the name each run gives itself.
#
# THOSE TWO ARE NOT THE SAME STRING AND ASSUMING THEY WERE IS THE DEFECT THIS
# FUNCTION EXISTS AGAINST. A run's own name is a display name: it comes from a
# `run-name` expression where a workflow declares one, and the two workflows this
# repository does not declare in its own tree name every run differently -
# `CodeQL Setup`, `Push on main`, `cargo in /. - Update #1546415969`. Grouped by
# that string, one workflow becomes a handful of one-run workflows, each of them
# standing on its own single conclusion, and a red one that ran twice under two
# display names is never seen to have recovered.
#
# The listing carries its own total on a two-field line ahead of the rows, so the
# count of runs that exist is read in the same request as the runs themselves
# rather than in a second one that could answer about a different moment.
rows_of_workflow() {
  awk -v w="$1" '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n == 2 && f[1] == "TOTAL") next
      if (n != 5) next
      printf "%s\t%s\t%s\t%s\t%s\t%s\n", w, f[1], f[2], f[3], f[4], f[5]
    }
  '
}

# The number of runs of that workflow the platform says exist on the branch,
# which is what the rows read are a most-recent slice of.
#
# It reads to the end rather than leaving at the first match. Leaving early closes
# the pipe under whatever is still writing into it, and a write that fails that
# way ends this script under `pipefail` for a reason that has nothing to do with
# what it was reading.
total_of_workflow() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n == 2 && f[1] == "TOTAL" && found == 0) { total = f[2]; found = 1 }
    }
    END { if (found == 1) print total }
  '
}

# Every line of that listing which is offered as a run rather than as the total.
# The rule above drops what it cannot read, so this is what it is counted against.
listing_lines() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n == 2 && f[1] == "TOTAL") next
      c = c + 1
    }
    END { printf "%d\n", c + 0 }
  '
}

# How many lines the rows offer at all, read so that a parser which stopped
# matching cannot report an empty set and pass.
rows_offered() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = n + 1
    }
    END { printf "%d\n", n + 0 }
  '
}

# Every row this script can read: six fields, a name, and a creation stamp. A
# line that is not one is a parser disagreement rather than a run, and the count
# above is what catches it.
well_formed_rows() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 6) next
      if (f[1] == "") next
      if (f[4] == "") next
      print line
    }
  '
}

# The latest CONCLUDED run of each workflow, one per line, as
#
#     workflow name, conclusion, created at, head sha, run id
#
# The stamps are the platform's own UTC form, fixed width, so the later of two is
# the greater of the two read as text and no date arithmetic happens anywhere in
# this file.
latest_conclusion_per_workflow() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 6) next
      if (f[1] == "") next
      if (f[3] == "") next
      if (!(f[1] in when) || f[4] > when[f[1]]) {
        when[f[1]] = f[4]
        verdict[f[1]] = f[3]
        head[f[1]] = f[5]
        id[f[1]] = f[6]
      }
    }
    END {
      for (w in when) printf "%s\t%s\t%s\t%s\t%s\n", w, verdict[w], when[w], head[w], id[w]
    }
  ' | LC_ALL=C sort
}

# Of those, the ones that did not conclude success. This is the verdict set, and
# it is the only thing that opens an issue.
standing_red() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 5) next
      if (f[2] == "success") next
      print line
    }
  '
}

# Every concluded run the read carried that did not conclude success, whatever the
# workflow has done since. This is the history, and it is printed rather than
# acted on.
every_non_success() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 6) next
      if (f[3] == "") next
      if (f[3] == "success") next
      print line
    }
  '
}

# Runs that have not concluded. A run still in flight carries no conclusion, and
# a reader that took the absent field for a failure would report every queued job
# as a red branch.
not_concluded_yet() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 6) next
      if (f[3] != "") next
      print line
    }
  '
}

# The distinct workflow names the listing carries, which is the set this run
# examined.
workflows_examined() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 6) next
      if (f[1] == "") next
      print f[1]
    }
  ' | LC_ALL=C sort -u
}

# The oldest and the newest creation stamp among the rows, as two tab-separated
# fields. This is the span the rows actually cover, which the report states rather
# than leaves to be assumed: a workflow that runs often is covered for days and
# one that runs weekly for months, off the same depth.
window_covered() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 6) next
      if (f[4] == "") next
      if (oldest == "" || f[4] < oldest) oldest = f[4]
      if (newest == "" || f[4] > newest) newest = f[4]
    }
    END {
      if (oldest != "") printf "%s\t%s\n", oldest, newest
    }
  '
}

# The title an issue carries, one per standing-red workflow. It names the
# workflow and nothing else that moves, so tomorrow's run finds the issue this
# one opened rather than opening a second one when the conclusion goes from
# `failure` to `cancelled`.
issue_titles() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line == "") next
      n = split(line, f, "\t")
      if (n != 5) next
      printf "The latest run of %s on the default branch did not conclude success\n", f[1]
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than the branch this repository
# happens to have. A fixture that judged the real listing would prove the state
# of the branch on the day it ran, not the rule.
# --------------------------------------------------------------------------

selftest_failures=0

assert_out() {
  local what="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    printf 'ok    %s\n' "$what"
  else
    printf 'FAIL  %s\n        expected: %s\n        actual:   %s\n' \
      "$what" "$(printf '%s' "$expected" | tr '\n\t' '|>')" "$(printf '%s' "$actual" | tr '\n\t' '|>')"
    selftest_failures=$((selftest_failures + 1))
  fi
}

# A run row, built with printf so no literal tab has to survive an editor.
row() { printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5" "$6"; }

# A latest-conclusion row, which is the shape the two rules downstream of it read.
verdict_row() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5"; }

# A line of one workflow's listing as the platform hands it over: five fields and
# no workflow name, because the name of the workflow is not in the run.
listing_row() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5"; }

# The count line that listing carries ahead of its rows.
listing_total() { printf 'TOTAL\t%s\n' "$1"; }

judge_offered()    { printf '%s' "$1" | rows_offered; }
judge_wellformed() { printf '%s' "$1" | well_formed_rows; }
judge_latest()     { printf '%s' "$1" | latest_conclusion_per_workflow; }
judge_red()        { printf '%s' "$1" | standing_red; }
judge_history()    { printf '%s' "$1" | every_non_success; }
judge_pending()    { printf '%s' "$1" | not_concluded_yet; }
judge_examined()   { printf '%s' "$1" | workflows_examined; }
judge_window()     { printf '%s' "$1" | window_covered; }
judge_titles()     { printf '%s' "$1" | issue_titles; }
judge_named()      { printf '%s' "$2" | rows_of_workflow "$1"; }
judge_wtotal()     { printf '%s' "$1" | total_of_workflow; }
judge_lines()      { printf '%s' "$1" | listing_lines; }

selftest() {
  local one two three four short superseded listing

  echo "== the name a row carries, which is the registry's and never the run's own =="
  listing="$(listing_total 137)
$(listing_row push success 2026-09-02T03:37:36Z 15453c7 33587745392)"
  assert_out "reads: the registry name in front of a run that names itself nothing" \
    "$(row 'Scorecard supply-chain security' push success 2026-09-02T03:37:36Z 15453c7 33587745392)" \
    "$(judge_named 'Scorecard supply-chain security' "$listing")"
  assert_out "reads: one name for two runs a display name would have split in two" \
    "$(row CodeQL push success 2026-08-28T13:07:23Z f8aaba7 1)
$(row CodeQL push failure 2026-08-27T13:07:23Z aaaaaaa 2)" \
    "$(judge_named CodeQL "$(listing_total 3)
$(listing_row push success 2026-08-28T13:07:23Z f8aaba7 1)
$(listing_row push failure 2026-08-27T13:07:23Z aaaaaaa 2)")"
  assert_out "passes over: the count line, which is not a run" \
    "" \
    "$(judge_named build "$(listing_total 79)")"
  assert_out "passes over: a line this parser cannot read, rather than guessing at its fields" \
    "" \
    "$(judge_named build "$(printf 'push\tsuccess')")"

  echo "== the count of runs that exist, read in the same request as the runs =="
  assert_out "reads: the count the listing carries" \
    "137" \
    "$(judge_wtotal "$listing")"
  assert_out "reads: nothing from a listing that carries no count" \
    "" \
    "$(judge_wtotal "$(listing_row push success 2026-09-02T03:37:36Z 15453c7 1)")"

  echo "== the count that catches a listing parser which stopped matching =="
  assert_out "counts: every line offered as a run, and not the count line" \
    "1" \
    "$(judge_lines "$listing")"
  assert_out "counts: a line this parser cannot read, which is offered and unreadable" \
    "1" \
    "$(judge_lines "$(listing_total 3)
$(printf 'push\tsuccess')")"
  assert_out "reads: nothing out of that line, so the two counts disagree and the run refuses" \
    "" \
    "$(judge_named build "$(listing_total 3)
$(printf 'push\tsuccess')")"
  assert_out "counts: nothing in a listing with no run in it" \
    "0" \
    "$(judge_lines "$(listing_total 0)")"

  echo "== the latest conclusion of a workflow, which is what the verdict is over =="
  one="$(row mutation schedule failure 2026-08-20T03:41:00Z aaaaaaa 11)"
  assert_out "reads: one run, whose own conclusion is the latest one" \
    "$(verdict_row mutation failure 2026-08-20T03:41:00Z aaaaaaa 11)" \
    "$(judge_latest "$one")"

  two="$one
$(row mutation schedule success 2026-08-27T03:41:00Z bbbbbbb 12)"
  assert_out "reads: the later of two runs of one workflow" \
    "$(verdict_row mutation success 2026-08-27T03:41:00Z bbbbbbb 12)" \
    "$(judge_latest "$two")"

  three="$(row mutation schedule success 2026-08-27T03:41:00Z bbbbbbb 12)
$(row mutation schedule failure 2026-08-20T03:41:00Z aaaaaaa 11)"
  assert_out "reads: the same answer with the newest row first, which is the order the platform sends" \
    "$(verdict_row mutation success 2026-08-27T03:41:00Z bbbbbbb 12)" \
    "$(judge_latest "$three")"

  four="$(row build push success 2026-08-27T09:00:00Z ccccccc 21)
$(row mutation schedule failure 2026-08-27T03:41:00Z aaaaaaa 11)"
  assert_out "reads: one line per workflow, in name order, for two workflows" \
    "$(verdict_row build success 2026-08-27T09:00:00Z ccccccc 21)
$(verdict_row mutation failure 2026-08-27T03:41:00Z aaaaaaa 11)" \
    "$(judge_latest "$four")"

  assert_out "passes over: a run that has not concluded, so a queued job is not a verdict" \
    "$(verdict_row mutation failure 2026-08-20T03:41:00Z aaaaaaa 11)" \
    "$(judge_latest "$one
$(row mutation schedule '' 2026-08-27T03:41:00Z bbbbbbb 12)")"

  short="$(printf 'mutation\tschedule\tfailure')"
  assert_out "passes over: a line this parser cannot read, rather than guessing at its fields" \
    "" \
    "$(judge_latest "$short")"

  echo "== the rule that decides a workflow is standing red =="
  assert_out "refuses: a workflow whose latest conclusion is failure" \
    "$(verdict_row mutation failure 2026-08-27T03:41:00Z aaaaaaa 11)" \
    "$(judge_red "$(verdict_row mutation failure 2026-08-27T03:41:00Z aaaaaaa 11)")"
  assert_out "refuses: a cancellation with nothing after it, which is a run that never finished" \
    "$(verdict_row unicode-guard cancelled 2026-08-27T03:41:00Z aaaaaaa 11)" \
    "$(judge_red "$(verdict_row unicode-guard cancelled 2026-08-27T03:41:00Z aaaaaaa 11)")"
  assert_out "refuses: a timed out run and one the platform could not start" \
    "$(verdict_row test timed_out 2026-08-27T03:41:00Z aaaaaaa 11)
$(verdict_row lint startup_failure 2026-08-27T03:42:00Z bbbbbbb 12)" \
    "$(judge_red "$(verdict_row test timed_out 2026-08-27T03:41:00Z aaaaaaa 11)
$(verdict_row lint startup_failure 2026-08-27T03:42:00Z bbbbbbb 12)")"
  assert_out "passes over: a workflow whose latest conclusion is success" \
    "" \
    "$(judge_red "$(verdict_row mutation success 2026-08-27T03:41:00Z aaaaaaa 11)")"
  assert_out "passes over: the green one and not the red one beside it" \
    "$(verdict_row mutation failure 2026-08-27T03:41:00Z aaaaaaa 11)" \
    "$(judge_red "$(verdict_row build success 2026-08-27T09:00:00Z ccccccc 21)
$(verdict_row mutation failure 2026-08-27T03:41:00Z aaaaaaa 11)")"

  echo "== the superseded cancellation, which is the shape that forced the verdict rule =="
  superseded="$(row unicode-guard push cancelled 2026-08-08T22:29:42Z 4bcdf97 31)
$(row unicode-guard push success 2026-08-08T22:29:45Z 4a24f93 32)"
  assert_out "does not stand red: a cancellation with a success three seconds after it" \
    "" \
    "$(judge_red "$(judge_latest "$superseded")")"
  assert_out "is still named: the same cancellation, in the history this run prints" \
    "$(row unicode-guard push cancelled 2026-08-08T22:29:42Z 4bcdf97 31)" \
    "$(judge_history "$superseded")"

  echo "== the history, which is printed and never acted on =="
  assert_out "reads: every non-success run the rows carry, whatever came after it" \
    "$(row mutation schedule failure 2026-08-20T03:41:00Z aaaaaaa 11)" \
    "$(judge_history "$two")"
  assert_out "passes over: a listing in which nothing concluded non-success" \
    "" \
    "$(judge_history "$(row build push success 2026-08-27T09:00:00Z ccccccc 21)")"

  echo "== the run that has not concluded, told apart from one that failed =="
  assert_out "reads: a row whose conclusion is empty" \
    "$(row mutation schedule '' 2026-08-27T03:41:00Z bbbbbbb 12)" \
    "$(judge_pending "$(row mutation schedule '' 2026-08-27T03:41:00Z bbbbbbb 12)")"
  assert_out "passes over: a row that concluded, whatever it concluded" \
    "" \
    "$(judge_pending "$(row mutation schedule failure 2026-08-27T03:41:00Z bbbbbbb 12)")"

  echo "== the set this run examined, which is what stops a registry reading as examined =="
  assert_out "reads: each name once, in name order" \
    "build
mutation" \
    "$(judge_examined "$four
$(row build push failure 2026-08-26T09:00:00Z ddddddd 22)")"
  assert_out "reads: nothing out of a listing with no readable row" \
    "" \
    "$(judge_examined 'not a row at all')"

  echo "== the span the rows cover, which is what the report states rather than assumes =="
  assert_out "reads: the oldest and the newest stamp in the listing" \
    "$(printf '2026-08-20T03:41:00Z\t2026-08-27T09:00:00Z')" \
    "$(judge_window "$four
$(row mutation schedule failure 2026-08-20T03:41:00Z aaaaaaa 11)")"
  assert_out "reads: one stamp twice where the listing carries one row" \
    "$(printf '2026-08-20T03:41:00Z\t2026-08-20T03:41:00Z')" \
    "$(judge_window "$one")"
  assert_out "says nothing about a span it read no row for" \
    "" \
    "$(judge_window '')"

  echo "== the count that catches a parser which stopped matching =="
  assert_out "counts: every line the listing offers as a row" \
    "2" \
    "$(judge_offered "$four")"
  assert_out "counts: a line this parser cannot read, which is a row and an unreadable one" \
    "1" \
    "$(judge_offered "$short")"
  assert_out "reads: nothing out of that line, so the two counts disagree and the run refuses" \
    "" \
    "$(judge_wellformed "$short")"
  assert_out "reads: a row carrying all six fields, so the count it is compared against is not zero for everything" \
    "$four" \
    "$(judge_wellformed "$four")"
  assert_out "reads: nothing out of a row whose workflow name is empty" \
    "" \
    "$(judge_wellformed "$(row '' push success 2026-08-27T09:00:00Z ccccccc 21)")"
  assert_out "reads: nothing out of a row whose creation stamp is empty" \
    "" \
    "$(judge_wellformed "$(row build push success '' ccccccc 21)")"
  assert_out "counts: nothing in an empty listing" \
    "0" \
    "$(judge_offered '')"

  echo "== the title an issue carries, which is what makes tomorrow's run find today's issue =="
  assert_out "writes: one title naming the workflow" \
    "The latest run of mutation on the default branch did not conclude success" \
    "$(judge_titles "$(verdict_row mutation failure 2026-08-27T03:41:00Z aaaaaaa 11)")"
  assert_out "writes: the same title when the conclusion moves from failure to cancelled" \
    "The latest run of mutation on the default branch did not conclude success" \
    "$(judge_titles "$(verdict_row mutation cancelled 2026-08-28T03:41:00Z bbbbbbb 12)")"
  assert_out "writes: one title per workflow, and none for the green one beside it" \
    "The latest run of mutation on the default branch did not conclude success" \
    "$(judge_titles "$(judge_red "$(judge_latest "$four")")")"
  assert_out "writes: nothing where nothing is standing red" \
    "" \
    "$(judge_titles "$(judge_red "$(judge_latest "$(row build push success 2026-08-27T09:00:00Z ccccccc 21)")")")"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures branch-health fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules this run applies are the rules these fixtures ran."
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
  local repo branch event registry branch_total
  local runs="" offered readable examined covered
  local latest red history pending titles existing
  local id name state path out wtotal wrows wread woffered
  local line title body opened=0 held=0 walked=0 silent=0

  repo="${GITHUB_REPOSITORY:-}"
  if [ -z "$repo" ]; then
    echo "::error::GITHUB_REPOSITORY is not set, so this run does not know whose default branch to read. The platform sets it on every run and it has to be supplied by hand anywhere else."
    return 1
  fi

  if ! command -v gh > /dev/null 2>&1; then
    echo "::error::the command line client is not on the path. This report reads the run listing of a branch, which is a question about the platform rather than about this tree, and there is nothing here for it to read instead."
    return 1
  fi

  if ! branch="$(gh api "repos/${repo}" --jq '.default_branch')" || [ -z "$branch" ]; then
    echo "::error::the default branch of ${repo} could not be read. This report is about that branch by name rather than about whatever branch the run was started on, so it stops here instead of reporting on a branch nobody asked about."
    return 1
  fi

  event="${GITHUB_EVENT_NAME:-none}"

  if ! branch_total="$(gh api -X GET "repos/${repo}/actions/runs" \
        -f "branch=${branch}" -f "per_page=1" --jq '.total_count')"; then
    echo "::error::the run listing of ${branch} could not be counted. That count is the population this run reports against, and a report that cannot say what it did not read is the shape this leg exists against."
    return 1
  fi

  echo "-- what this run read"
  echo "      repository:     ${repo}"
  echo "      branch:         ${branch}"
  echo "      started by:     ${event}"
  echo "      runs on branch: ${branch_total}, of which this run reads the ${RUNS_PER_WORKFLOW} most recent per workflow"
  echo

  if ! registry="$(gh api --paginate -X GET "repos/${repo}/actions/workflows" -f "per_page=100" \
        --jq '.workflows[] | [(.id | tostring), .name, .state, .path] | @tsv')"; then
    echo "::error::the workflow registry of ${repo} could not be read. The registry is what this run walks, so without it there is no set to report on and no set to say what was left out of."
    return 1
  fi

  if [ -z "$registry" ]; then
    echo "::error::the workflow registry of ${repo} names nothing. An empty registry is walked in no time, reports on no workflow and prints a page indistinguishable from a branch on which everything is green, so it is refused rather than read as a repository with no workflows."
    return 1
  fi

  echo "-- every workflow the registry holds, and what this run read of it"
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    id="$(printf '%s' "$line" | cut -f1)"
    name="$(printf '%s' "$line" | cut -f2)"
    state="$(printf '%s' "$line" | cut -f3)"
    path="$(printf '%s' "$line" | cut -f4)"
    walked=$((walked + 1))

    if ! out="$(gh api -X GET "repos/${repo}/actions/workflows/${id}/runs" \
          -f "branch=${branch}" -f "per_page=${RUNS_PER_WORKFLOW}" \
          --jq '"TOTAL\t\(.total_count)", (.workflow_runs[] | [.event, (.conclusion // ""), .created_at, .head_sha, (.id | tostring)] | @tsv)')"; then
      echo "::error::the runs of ${name} on ${branch} could not be read. A workflow this run could not ask about and one that is healthy print the same line, so this run refuses rather than reporting a set it did not see the whole of."
      return 1
    fi

    wtotal="$(printf '%s\n' "$out" | total_of_workflow)"
    wrows="$(printf '%s\n' "$out" | rows_of_workflow "$name")"
    wread="$(printf '%s\n' "$wrows" | rows_offered)"
    woffered="$(printf '%s\n' "$out" | listing_lines)"

    if [ "$wread" -ne "$woffered" ]; then
      echo "::error::the listing for ${name} offered ${woffered} run(s) and this run could read ${wread} of them. A parser that stopped matching drops rows silently and the workflow then stands on whichever conclusions survived, so the disagreement is refused rather than passed."
      return 1
    fi

    if [ -z "$wtotal" ]; then
      echo "::error::the listing for ${name} carried no count of the runs that exist, so this run cannot say what its rows are a slice of. A report that cannot state what it did not read is the shape this leg exists against."
      return 1
    fi

    if [ "$wread" -eq 0 ]; then
      silent=$((silent + 1))
      printf '      NOT EXAMINED  %s (%s, %s) - no run of it on %s at all\n' \
        "$name" "$state" "$path" "$branch"
    else
      printf '      examined      %s (%s, %s) - read %s of its %s run(s) on %s\n' \
        "$name" "$state" "$path" "$wread" "$wtotal" "$branch"
      runs="${runs}${wrows}
"
    fi
  done <<REGISTRY
$registry
REGISTRY
  echo

  offered="$(printf '%s\n' "$runs" | rows_offered)"
  readable="$(printf '%s\n' "$runs" | well_formed_rows | rows_offered)"

  if [ "$offered" -ne "$readable" ]; then
    echo "::error::the listings offered ${offered} row(s) and this run could read ${readable} of them. A parser that stopped matching reports an empty set and prints a page indistinguishable from a branch with nothing red on it, so the disagreement is refused rather than passed."
    return 1
  fi

  if [ "$offered" -eq 0 ]; then
    echo "::error::none of the ${walked} workflow(s) in the registry has a run on ${branch}, and the branch itself reports ${branch_total}. Those two cannot both be true, so this run refuses rather than reporting a branch on which it saw nothing."
    return 1
  fi

  covered="$(printf '%s\n' "$runs" | window_covered)"
  examined="$(printf '%s\n' "$runs" | workflows_examined)"
  latest="$(printf '%s\n' "$runs" | latest_conclusion_per_workflow)"
  red="$(printf '%s\n' "$latest" | standing_red)"
  history="$(printf '%s\n' "$runs" | every_non_success)"
  pending="$(printf '%s\n' "$runs" | not_concluded_yet)"

  echo "      registry walked: ${walked} workflow(s), of which ${silent} have never run on ${branch}"
  echo "      rows read:       ${offered}"
  echo "      span covered:    $(printf '%s' "$covered" | tr '\t' ' ')"
  echo

  echo "-- the latest conclusion of every workflow this run examined"
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    printf '      %s\n' "$(printf '%s' "$line" | tr '\t' ' ')"
  done <<LATEST
$latest
LATEST
  echo

  echo "-- runs this read carried that concluded non-success, whatever the workflow has done since"
  if [ -z "$history" ]; then
    echo "      none"
  else
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      name="$(printf '%s' "$line" | cut -f1)"
      if printf '%s\n' "$red" | cut -f1 | grep -qxF -- "$name"; then
        printf '      standing red  %s\n' "$(printf '%s' "$line" | tr '\t' ' ')"
      else
        printf '      recovered     %s - out of the verdict because a later run of it concluded success\n' \
          "$(printf '%s' "$line" | tr '\t' ' ')"
      fi
    done <<HISTORY
$history
HISTORY
  fi
  echo

  if [ -n "$pending" ]; then
    echo "-- runs that had not concluded when this one read the listing, which are neither green nor red here"
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      printf '      %s\n' "$(printf '%s' "$line" | tr '\t' ' ')"
    done <<PENDING
$pending
PENDING
    echo
  fi

  titles="$(printf '%s\n' "$red" | issue_titles)"

  if [ -z "$red" ]; then
    say "No workflow on ${branch} is standing red. $(printf '%s\n' "$examined" | grep -c . || true) of the ${walked} workflow(s) the registry holds were examined."
  else
    say "$(printf '%s\n' "$red" | grep -c . || true) workflow(s) on ${branch} are standing red: $(printf '%s\n' "$red" | cut -f1 | tr '\n' ' ')"
  fi
  echo

  echo "-- what this run did about it"
  case "$event" in
    schedule | workflow_dispatch)
      if [ -z "$red" ]; then
        echo "      nothing to open"
      else
        if ! existing="$(gh issue list --repo "$repo" --state open --limit 500 --json title --jq '.[].title')"; then
          echo "::error::the open issues of ${repo} could not be listed, so this run cannot tell an issue it already opened from one it has not, and opening without that reading is how one red workflow becomes an issue a day."
          return 1
        fi
        while IFS= read -r title; do
          [ -n "$title" ] || continue
          if printf '%s\n' "$existing" | grep -qxF -- "$title"; then
            printf '      already open  %s\n' "$title"
            held=$((held + 1))
            continue
          fi
          body="$(mktemp)"
          {
            printf 'A run of this workflow on `%s` concluded something other than success, and no run of it has concluded success since.\n\n' "$branch"
            printf 'The reading, taken by `.github/branch-health/branch-health.sh` in run %s, as workflow, conclusion, created at, head, run id:\n\n' "${GITHUB_RUN_ID:-unknown}"
            printf '```\n'
            printf '%s\n' "$red" | tr '\t' ' '
            printf '```\n\n'
            printf 'Nobody waits on a run that concludes after a merge or on a schedule, so it can stay red for weeks with every pull request green beside it. That is what this issue is against.\n\n'
            printf 'A scheduled run opened this rather than a person. It carries no milestone: which milestone a red workflow belongs to is a judgement about the plan, and the run has nothing to make it with.\n'
          } > "$body"
          if gh issue create --repo "$repo" --title "$title" --body-file "$body" \
               --label "$ISSUE_LABEL" --assignee "$ISSUE_ASSIGNEE" > /dev/null; then
            printf '      opened        %s\n' "$title"
            opened=$((opened + 1))
          else
            rm -f "$body"
            echo "::error::an issue could not be opened for a workflow standing red on ${branch}. The report above is then the whole of what this run produced, and a report nobody is waiting on is the thing that gets missed, which is why this issue exists at all."
            return 1
          fi
          rm -f "$body"
        done <<TITLES
$titles
TITLES
        say "${opened} issue(s) opened, ${held} already open."
      fi
      ;;
    *)
      echo "      NOT MADE HERE: no issue was opened. This run was started by \`${event}\` rather than by the schedule or by hand, and an issue opened while judging a pull request would be a reading of the default branch that the change in front of it did not cause."
      ;;
  esac
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: anything but the ${RUNS_PER_WORKFLOW} most recent runs of each workflow. The line per workflow above says how many of its runs on ${branch} this one looked at and how many there are, and everything older than that is outside this report."
  echo "NOT MADE HERE: a workflow the registry no longer holds. This walks the registry, so a workflow file that was deleted keeps its conclusions on ${branch} and this run never asks about them."
  echo "NOT MADE HERE: a workflow that never reaches ${branch}. One firing only on a pull request concludes on a pull request head, which these listings do not carry, so its health is outside every run this leg makes, and it is named as NOT EXAMINED rather than counted as healthy."
  echo "NOT MADE HERE: the order the platform returned. The latest conclusion is the greatest creation stamp among the rows read rather than the first row of the listing, so a page in another order still answers; what no reading here can see is a newer run the platform did not hand over."
  echo "NOT MADE HERE: why a run concluded what it did. This reads a conclusion and never a log, so a failure and a timeout are two words here and the reason for either is in the run itself."
  echo "NOT MADE HERE: whether a workflow that stopped firing should have fired. A cron nobody removed and a cron the platform stopped scheduling both produce no run, and this reports the absence rather than deciding which of the two it is."
  echo "NOT MADE HERE: closing an issue this report opened. A workflow that recovers stops being standing red and opens nothing further, and the issue already open stays open until a person closes it, because a report that closed its own issues would be deciding the work behind them was done."
  echo
  echo "A report exists for this run. That is what a green verdict here asserts, and it asserts nothing about the colour of the branch."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
