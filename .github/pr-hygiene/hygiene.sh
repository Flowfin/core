#!/usr/bin/env bash
# Deterministic pull-request hygiene rules (#83).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate.
#
# Deterministic is the word that decides what may be a rule here. Every rule below
# is settled by reading the pull request and the issues it names, and none of them
# asks whether a sentence is any good. A rule needing a person to interpret it is
# not a hygiene rule, it is the review.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules to one pull request, from the environment below
#
# `check` reads:
#   PR_NUMBER           the pull request being judged
#   PR_BODY_FILE        a file holding its body verbatim
#   CHANGED_PATHS_FILE  a file holding one changed path per line
#   GH_REPO             owner/name, for the issue lookups
#   PR_AUTHOR           the login that opened it, for the exemption below. Unset
#                       or empty is read as an author that can write a body, so a
#                       run that could not read one refuses exactly as before.
#
# What this file does NOT judge, so that a green run cannot be read as covering it:
#
#   The DCO sign-off. `.github/workflows/dco.yml` refuses a commit whose
#   `Signed-off-by` trailer does not match its author, which is stricter than
#   asking whether a sign-off is present at all. A second implementation of one
#   rule is two things that agree until they drift, so this run reads no commits
#   and prints that it did not.

set -euo pipefail

# --------------------------------------------------------------------------
# Rules. Each one reads a document on stdin and writes its verdict to stdout.
# awk rather than grep throughout: grep exits 1 when it selects nothing, which is
# the ordinary answer for most of these, and a pipeline that has to tell "nothing
# matched" from "the scanner broke" one `set -o pipefail` at a time is how a gate
# ends up passing on everything.
#
# No POSIX character classes and no interval expressions in any pattern below. The
# awk on the runner is mawk and the awk on a contributor's machine is frequently
# gawk, and those two constructs are where the older mawk builds disagree with it.
# A rule that matches on one machine and not on the other is a gate whose verdict
# depends on who ran it.
# --------------------------------------------------------------------------

# Removes HTML comments, including ones spanning several lines. The pull-request
# template on this board is mostly comments, and text a contributor never wrote is
# not text they said.
strip_comments() {
  awk '
    {
      out = ""; s = $0
      while (length(s) > 0) {
        if (incomment) {
          i = index(s, "-->")
          if (i == 0) { s = "" } else { s = substr(s, i + 3); incomment = 0 }
        } else {
          i = index(s, "<!--")
          if (i == 0) { out = out s; s = "" }
          else { out = out substr(s, 1, i - 1); s = substr(s, i + 4); incomment = 1 }
        }
      }
      print out
    }
  '
}

# Every issue this body names, deduplicated, one per line.
#
# Named rather than closed, and that is the whole difference between a rule and a
# convention nobody agreed to. Most pull requests on this board name the issue
# they belong to and deliberately do not close it, because the definition of done
# is not met, and a rule demanding a closing keyword would refuse the honest half
# of this board's own practice and be worked around inside a week.
#
# The template's `Closes #` with nothing after it carries no number and is
# therefore no reference, which is the mistake this rule is really for. A heading
# is not one either: `## What changed` has no digits after the hash.
#
# A number qualified with another repository is not a reference here either, and
# that one is the case the check below fails closed on rather than passes over.
# `owner/repo#123` names an issue somewhere else, so looking it up here asks a
# question nobody asked and gets a 404 for it. The qualifier is read and compared:
# equal to the repository being judged, the number is ours; different, it is not;
# absent, the reference is bare and is read exactly as before. A qualifier with no
# slash in it is not a repository, so `issue#12` in a sentence stays what it was.
#
# What that buys is a body able to paste the output of a command verbatim. The
# rule this repository puts first is that a claim carries the command that
# produced it, and a grep over a file naming another repository's issue could not
# be pasted while every hash-then-digits in a body was a question about this one.
#
# The qualifier is walked out character by character against a literal set rather
# than cut off with a regular expression. `sub(/^.*[^set]/, "", s)` is the obvious
# way to write it and one of the two awks does not match it at all, which is the
# trap the header of this file already names for character classes and interval
# expressions, arriving through a third construct.
issue_refs() {
  local self="${1:-${GH_REPO:-}}"
  strip_comments | awk -v self="$self" \
    -v allowed="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._/-" '
    {
      line = $0
      prefix = ""
      while (match(line, /#[0-9]+/)) {
        head = prefix substr(line, 1, RSTART - 1)
        qualifier = ""
        i = length(head)
        while (i >= 1) {
          c = substr(head, i, 1)
          if (index(allowed, c) == 0) break
          qualifier = c qualifier
          i = i - 1
        }
        if (index(qualifier, "/") == 0 || qualifier == self) {
          print substr(line, RSTART + 1, RLENGTH - 1)
        }
        prefix = head substr(line, RSTART, RLENGTH)
        line = substr(line, RSTART + RLENGTH)
      }
    }
  ' | sort -un
}

# Whether the login in $1 belongs to an author that cannot write a pull-request
# body (#108).
#
# It exempts that author from `names-an-issue` and from nothing else. The rule
# exists so that a change somebody wrote names the work it serves; an update
# proposal has no such author and cannot acquire one, so applying the rule to it
# does not enforce that promise, it only paints a permanent red on every proposal
# forever, which teaches a reader that red here means nothing. What stands behind
# such a proposal instead is a person reading it against
# `docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md`, which is
# what that record already says stands behind an update, since nothing in this
# repository machine-reads the clause line beside a manifest entry.
#
# An explicit list of logins rather than a `*[bot]` pattern, for the reason
# `.github/workflows/dco.yml` gives at the same exemption: a pattern lets a person
# self-exempt by choosing a `[bot]`-shaped name, and the whole value of an
# identity exemption is that the identity cannot be chosen by the party it
# excuses. GitHub reserves the `[bot]` suffix, so these two strings are not
# available to a person, and a third one is added here by somebody arguing for it
# rather than by a pattern widening on its own.
#
# The comparison is exact and case-sensitive. `Dependabot[bot]`, a trailing space
# and `not-dependabot[bot]` are all different strings and none of them is exempt.
author_cannot_write_a_body() {
  case "${1:-}" in
    "dependabot[bot]" | "github-actions[bot]") return 0 ;;
    *) return 1 ;;
  esac
}

# What the author wrote, as opposed to what the template put on the page.
#
# Empty here means empty of the author: HTML comments removed, then headings and
# lines that are nothing but issue references dropped, because a body that is the
# template's skeleton with the number filled in is a form somebody submitted
# rather than a description somebody wrote. Anything else counts, including one
# sentence.
body_prose() {
  strip_comments | awk '
    function reference_only(s,   n, i, t, arr, ok) {
      n = split(tolower(s), arr, /[^a-z0-9#]+/)
      ok = 0
      for (i = 1; i <= n; i++) {
        t = arr[i]
        if (t == "") continue
        if (t ~ /^#[0-9]+$/) { ok = 1; continue }
        if (t ~ /^(close|closes|closed|fix|fixes|fixed|resolve|resolves|resolved|ref|refs|and)$/) continue
        return 0
      }
      return ok
    }
    { sub(/\r$/, "") }
    /^[ \t]*$/ { next }
    /^[ \t]*#+[ \t]/ { next }
    reference_only($0) { next }
    { print }
  '
}

# The path prefixes an issue body declares, from a `Scope:` line at column zero.
#
# Column zero so that the word inside a sentence is not a declaration. Separated
# by commas rather than by whitespace: a scope gate elsewhere in this organisation
# split its input on whitespace and refused valid work for it, and a path may
# carry a space far more easily than a comma.
scope_prefixes() {
  awk '
    /^Scope:/ {
      rest = substr($0, 7)
      n = split(rest, parts, ",")
      for (i = 1; i <= n; i++) {
        gsub(/^[ \t\r]+|[ \t\r]+$/, "", parts[i])
        if (parts[i] != "") print parts[i]
      }
    }
  '
}

# Changed paths matching none of the prefixes in the file named by $1.
#
# A prefix matches a whole path segment, so `docs/decisions` admits
# `docs/decisions/0001.md` and refuses `docs/decisions-old/0001.md`. Plain string
# prefixing would admit both, and the second is a different directory.
paths_outside() {
  awk -v pf="$1" '
    function inside(path, p) {
      if (path == p) return 1
      if (substr(p, length(p)) == "/") return index(path, p) == 1
      return index(path, p "/") == 1
    }
    BEGIN {
      n = 0
      while ((getline line < pf) > 0) { sub(/\r$/, "", line); if (line != "") prefixes[++n] = line }
    }
    { sub(/\r$/, ""); if ($0 == "") next
      for (i = 1; i <= n; i++) if (inside($0, prefixes[i])) next
      print }
  '
}

# --------------------------------------------------------------------------
# selftest
# --------------------------------------------------------------------------

selftest_failures=0

# The exemption above as a word, so a fixture reads the same way as every other
# one here. `assert_out` compares text and the rule answers with an exit code.
exemption_verdict() {
  if author_cannot_write_a_body "$1"; then echo exempt; else echo refused; fi
}

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

# The skeleton a contributor sends when they fill in the number and delete nothing
# else. Three of the template's headings; what makes it a skeleton is that every
# line is a heading, a comment or the reference itself.
skeleton() {
  cat <<'EOF'
<!--
Nothing reads this template.
-->

## The issue this belongs to

Closes #83

## What changed

<!-- What the change does. -->

## Evidence

<!-- Every number carries the command that produced it. -->
EOF
}

selftest() {
  echo "== names-an-issue =="
  assert_out "bites: the template's own reference, with no number after the hash" \
    "" "$(printf 'Closes #\n\nIt adds the check.\n' | issue_refs)"
  assert_out "passes: the same body one character longer" \
    "83" "$(printf 'Closes #83\n\nIt adds the check.\n' | issue_refs)"
  assert_out "passes: named without a closing keyword, which is this board's practice" \
    "27" "$(printf 'Refs #27. It does not close it, and the reason is below.\n' | issue_refs)"
  assert_out "bites: a reference only inside an HTML comment" \
    "" "$(printf '<!--\nCloses #83\n-->\nIt adds the check.\n' | issue_refs)"
  assert_out "bites: a heading is not a reference" \
    "" "$(printf '## What changed\n\nIt adds the check.\n' | issue_refs)"
  assert_out "passes: several issues, deduplicated and in order" \
    "$(printf '55\n57\n105')" "$(printf 'Three, and none of them closes here: #105, #55 and #57.\n' | issue_refs)"
  assert_out "bites: a number qualified with another repository" \
    "" "$(printf 'The argument was made in Flowfin/jellyfin-plugin-sso#263.\n' | issue_refs Flowfin/core)"
  assert_out "passes: the same number qualified with the repository being judged" \
    "151" "$(printf 'Flowfin/core#151 is the one this belongs to.\n' | issue_refs Flowfin/core)"
  assert_out "passes: a bare reference beside a qualified one, which is what pasted evidence looks like" \
    "$(printf '83\n151')" "$(printf 'Closes #151, quoting Flowfin/jellyfin-plugin-sso#263 and #83.\n' | issue_refs Flowfin/core)"
  assert_out "bites: an address whose fragment is digits" \
    "" "$(printf 'See https://example.com/x#12 for the rest.\n' | issue_refs Flowfin/core)"
  assert_out "passes: a word before the hash that is not a repository" \
    "12" "$(printf 'issue#12 in an ordinary sentence.\n' | issue_refs Flowfin/core)"

  echo "== names-an-issue: the author that cannot write a body =="
  assert_out "exempt: the update proposer, which is the author this was widened for" \
    "exempt" "$(exemption_verdict 'dependabot[bot]')"
  assert_out "exempt: the actions runner, the second identity dco.yml already names" \
    "exempt" "$(exemption_verdict 'github-actions[bot]')"
  assert_out "refused: an ordinary contributor" \
    "refused" "$(exemption_verdict 'iderex')"
  assert_out "refused: a login crafted to end in the reserved suffix" \
    "refused" "$(exemption_verdict 'not-dependabot[bot]')"
  assert_out "refused: the exempt login written in a different case" \
    "refused" "$(exemption_verdict 'Dependabot[bot]')"
  assert_out "refused: the exempt login with a trailing space" \
    "refused" "$(exemption_verdict 'dependabot[bot] ')"
  assert_out "refused: no author read at all, which is the fail-closed direction" \
    "refused" "$(exemption_verdict '')"

  echo "== body-not-empty =="
  assert_out "bites: the template with the issue number filled in and nothing written" \
    "" "$(skeleton | body_prose)"
  assert_out "passes: the same body with one sentence added" \
    "It adds the check." "$(skeleton | sed 's|^<!-- What the change does. -->$|It adds the check.|' | body_prose)"
  assert_out "bites: a body that is only whitespace" \
    "" "$(printf '   \n\n\t\n' | body_prose)"
  assert_out "bites: a body that is only a list of issue numbers" \
    "" "$(printf 'Closes #55, #57 and #105.\n' | body_prose)"
  assert_out "passes: a sentence about those issues rather than a list of them" \
    "Three, and none of them closes here: #105, #55 and #57." \
    "$(printf 'Three, and none of them closes here: #105, #55 and #57.\n' | body_prose)"

  echo "== changed-paths-inside-scope =="
  local pf; pf="$(mktemp)"
  printf 'docs/decisions\n' > "$pf"
  assert_out "bites: a neighbouring directory whose name starts the same way" \
    "docs/decisions-old/0001.md" \
    "$(printf 'docs/decisions/0001.md\ndocs/decisions-old/0001.md\n' | paths_outside "$pf")"
  assert_out "passes: every path under the declared prefix" \
    "" "$(printf 'docs/decisions/0001.md\ndocs/decisions\n' | paths_outside "$pf")"
  printf '.github/workflows\n.github/pr-hygiene\n' > "$pf"
  assert_out "bites: one path outside two declared prefixes" \
    "README.md" \
    "$(printf '.github/workflows/pr-hygiene.yml\n.github/pr-hygiene/hygiene.sh\nREADME.md\n' | paths_outside "$pf")"
  rm -f "$pf"

  echo "== scope-declaration =="
  assert_out "reads: a Scope line at column zero, split on commas" \
    "$(printf '.github/workflows\n.github/pr-hygiene')" \
    "$(printf 'Scope: .github/workflows, .github/pr-hygiene\n\nWhat is wrong.\n' | scope_prefixes)"
  assert_out "does not read: the same words inside a sentence" \
    "" "$(printf 'The Scope: .github/workflows line is what a route reads.\n' | scope_prefixes)"
  assert_out "does not read: a body declaring no scope at all" \
    "" "$(printf 'What is wrong, and what done means.\n' | scope_prefixes)"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures hygiene fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

# One issue body, or a non-zero exit. A lookup that failed is not an issue that
# declares no scope, and reading it as one would turn an outage into a pass.
issue_body() {
  gh api "repos/${GH_REPO}/issues/$1" --jq '.body // ""'
}

check() {
  local failures=0
  local body_file="${PR_BODY_FILE}" paths_file="${CHANGED_PATHS_FILE}"
  local author="${PR_AUTHOR:-}"

  echo "Pull request #${PR_NUMBER} on ${GH_REPO}."
  echo "Examined: the body as sent, and $(awk 'END { print NR }' "$paths_file") changed path(s)."
  echo "Opened by: ${author:-<not read>}."
  echo

  local refs
  refs="$(issue_refs < "$body_file")"

  echo "-- names-an-issue"
  if [ -z "$refs" ] && author_cannot_write_a_body "$author"; then
    echo "EXEMPT: ${author} cannot write a pull-request body, so this rule is not applied to it and no other rule below is relaxed. A proposal from this author is read by a person against docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md, which is what stands behind an update here; nothing in this repository machine-reads the clause line beside a manifest entry."
  elif [ -z "$refs" ]; then
    echo "::error::This pull request names no issue. Write the issue it belongs to in the body as #<number>. A reference inside an HTML comment is not one, the template's 'Closes #' carries no number, and a number written as owner/repo#<number> names an issue on that repository rather than one here."
    failures=$((failures + 1))
  else
    echo "ok    names: $(printf '%s' "$refs" | tr '\n' ' ')"
  fi
  echo

  echo "-- body-not-empty"
  if [ -z "$(body_prose < "$body_file")" ]; then
    echo "::error::This pull request body says nothing the template did not. Headings, HTML comments and the issue reference are what the template put there; what changed and what failure it prevents are what a reader needs."
    failures=$((failures + 1))
  else
    echo "ok    the body carries $(body_prose < "$body_file" | awk 'END { print NR }') line(s) the template did not"
  fi
  echo

  echo "-- changed-paths-inside-scope"
  local scope_file declared=0
  scope_file="$(mktemp)"
  : > "$scope_file"
  local n
  for n in $refs; do
    local b p
    if ! b="$(issue_body "$n")"; then
      echo "::error::Could not read issue #${n}. The scope comparison cannot be made, and this run will not pass in place of it."
      rm -f "$scope_file"
      return 1
    fi
    p="$(printf '%s' "$b" | scope_prefixes)"
    if [ -n "$p" ]; then
      declared=1
      printf '%s\n' "$p" >> "$scope_file"
      echo "      issue #${n} declares: $(printf '%s' "$p" | tr '\n' ' ')"
    else
      echo "      issue #${n} declares no Scope: line at column zero"
    fi
  done

  if [ "$declared" -eq 0 ]; then
    echo "NOT MADE: no issue this pull request names declares a 'Scope:' line at column zero, so the changed paths were compared against nothing. This is a pass with no comparison behind it, and it is not a pass with one."
  else
    local outside
    outside="$(paths_outside "$scope_file" < "$paths_file")"
    if [ -n "$outside" ]; then
      echo "::error::Changed paths outside every scope the named issues declare: $(printf '%s' "$outside" | tr '\n' ' ')"
      failures=$((failures + 1))
    else
      echo "ok    every changed path is inside a declared scope"
    fi
    echo "      The scopes are a union, so naming a further issue can only widen this comparison and never narrow it. It is a floor rather than a fence."
  fi
  rm -f "$scope_file"
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: the DCO sign-off. .github/workflows/dco.yml refuses a commit whose Signed-off-by does not match its author; this run read no commits."
  echo "NOT MADE HERE: anything needing judgement. Whether the body says something true, whether the evidence is evidence, and whether the change belongs to the issue it names are read by a person."
  echo

  if [ "$failures" -ne 0 ]; then
    echo "::error::${failures} hygiene rule(s) refused this pull request."
    return 1
  fi
  echo "Every rule this check owns passed."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
