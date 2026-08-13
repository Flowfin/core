#!/usr/bin/env bash
# A document names paths that resolve (#110).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate.
#
# What the check is for: a named path that does not resolve is the cheapest kind of
# wrong and the first one a reader meets, because a reader follows the path. It is
# also the kind that appears without anybody touching the document, when a file
# moves in a commit that never opened the paragraph naming it.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      apply the rules to every tracked document, and refuse
#   addresses  report on the external addresses documents name, and never refuse
#
# `check` and `addresses` read the repository through `git ls-files`, so the
# authority for what exists is the set of tracked paths rather than the working
# tree. A file present on disk and not added is not a path a reader can follow.

set -euo pipefail

# The marker a document uses where it names something as an example rather than as
# a reference. It carries a reason, it sits on the line it excuses, and both are
# deliberate: an escape kept in a list somewhere else is a way to turn the check
# off one line at a time, and an escape with no reason is a rule identifier
# somebody pasted.
ESCAPE_OPEN='<!-- names-an-example:'

# --------------------------------------------------------------------------
# Rules. Each one reads a document on stdin and writes records to stdout, one per
# line, as LINE<TAB>KIND<TAB>RAW<TAB>RESOLVED.
#
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

# Every repository path a document names, plus every escape marker it carries.
#
# $1 is the document's own path, because a Markdown link target is relative to the
# document that carries it while a code span is written from the repository root.
# Reading both the same way would either refuse every link in
# `docs/decisions/README.md` or accept a code span that resolves from nowhere.
#
# Fenced and indented blocks are not read. Those hold commands and their output,
# which is evidence of a run rather than a reference a reader follows, and the
# paths inside them are frequently another repository's.
scan_paths() {
  awk -v doc="$1" -v esc="$ESCAPE_OPEN" '
    function dirof(p,   i) {
      i = length(p)
      while (i > 0 && substr(p, i, 1) != "/") i--
      if (i == 0) return ""
      return substr(p, 1, i)
    }
    # Collapses "." and "..", and answers "" for a path that climbs out of the
    # repository root, which is not a path in this tree and is reported as one
    # that does not resolve rather than being silently dropped.
    function normalise(p,   n, parts, st, m, i, out) {
      n = split(p, parts, "/")
      m = 0
      for (i = 1; i <= n; i++) {
        if (parts[i] == "" || parts[i] == ".") continue
        if (parts[i] == "..") { if (m == 0) return ""; m--; continue }
        st[++m] = parts[i]
      }
      if (m == 0) return ""
      out = st[1]
      for (i = 2; i <= m; i++) out = out "/" st[i]
      return out
    }
    function emit(kind, raw, resolved) {
      printf "%d\t%s\t%s\t%s\n", FNR, kind, raw, resolved
    }
    # A code span is read as a path only where it carries a separator and its last
    # segment carries a dot, or where it ends in a separator and is therefore a
    # directory. The bound is deliberate and `docs/gate-parity.md` is why: its
    # first column names two dozen files inside another repository, and a rule
    # reading a bare name as a local path would refuse every one of them.
    function looks_like_path(s,   last, i) {
      if (index(s, "/") == 0) return 0
      if (substr(s, length(s)) == "/") return 1
      i = length(s)
      while (i > 0 && substr(s, i, 1) != "/") i--
      last = substr(s, i + 1)
      return index(last, ".") > 0
    }
    BEGIN { fence = 0; d = dirof(doc) }
    {
      line = $0
      sub(/\r$/, "", line)

      if (line ~ /^[ \t]*```/ || line ~ /^[ \t]*~~~/) { fence = 1 - fence; next }
      if (fence) next
      if (line ~ /^(    |\t)/) next

      # The marker is taken out of the line before anything else, so that a path
      # written inside the reason is part of the reason and not a reference.
      i = index(line, esc)
      if (i > 0) {
        rest = substr(line, i + length(esc))
        j = index(rest, "-->")
        if (j == 0) { reason = rest; line = substr(line, 1, i - 1) }
        else { reason = substr(rest, 1, j - 1); line = substr(line, 1, i - 1) substr(rest, j + 3) }
        gsub(/^[ \t]+|[ \t]+$/, "", reason)
        emit("escape", reason, "")
      }

      # Markdown link targets. A target with a scheme is an external address and
      # belongs to the other verb; an anchor, a query and a link title are removed
      # because none of them is part of the path on disk.
      s = line
      while (match(s, /\]\([^)]+\)/)) {
        t = substr(s, RSTART + 2, RLENGTH - 3)
        s = substr(s, RSTART + RLENGTH)
        if (index(t, " ") > 0) t = substr(t, 1, index(t, " ") - 1)
        if (substr(t, 1, 1) == "<" && substr(t, length(t)) == ">") t = substr(t, 2, length(t) - 2)
        if (index(t, "://") > 0) continue
        if (substr(t, 1, 1) == "#") continue
        if (index(t, ":") > 0) continue
        if (index(t, "#") > 0) t = substr(t, 1, index(t, "#") - 1)
        if (index(t, "?") > 0) t = substr(t, 1, index(t, "?") - 1)
        if (t == "") continue
        emit("link", t, normalise(substr(t, 1, 1) == "/" ? substr(t, 2) : d t))
      }

      # Code spans, read from the repository root.
      s = line
      while (match(s, /`[^`]+`/)) {
        span = substr(s, RSTART + 1, RLENGTH - 2)
        s = substr(s, RSTART + RLENGTH)
        if (index(span, "://") > 0) continue
        if (span ~ /[ \t]/) continue
        if (index(span, "*") > 0 || index(span, "?") > 0) continue
        if (substr(span, 1, 1) == "~") continue
        if (!looks_like_path(span)) continue
        emit("span", span, normalise(substr(span, 1, 1) == "/" ? substr(span, 2) : span))
      }
    }
  '
}

# Every external address a document names, from every line including the fenced
# and indented ones. An address inside a command block is still an address a
# reader follows, which is the opposite of the case for a path.
scan_addresses() {
  awk -v esc="$ESCAPE_OPEN" '
    {
      line = $0
      sub(/\r$/, "", line)
      escaped = index(line, esc) > 0 ? 1 : 0
      s = line
      while (match(s, /https?:\/\/[^ \t)"'"'"'`<>]+/)) {
        u = substr(s, RSTART, RLENGTH)
        s = substr(s, RSTART + RLENGTH)
        # Trailing sentence punctuation is not part of an address.
        while (length(u) > 0 && index(".,;:", substr(u, length(u))) > 0) u = substr(u, 1, length(u) - 1)
        if (u == "") continue
        printf "%d\t%s\t%s\n", FNR, escaped ? "address-escaped" : "address", u
      }
    }
  '
}

# Why an address is not requested, or an empty answer where it is.
#
# A userinfo component is never sent. A document naming credentials in an example
# address is naming them as an example, and a check that dialled it would be the
# one route in this repository that transmits a secret somebody wrote down.
#
# The reserved names are the ones RFC 2606 and RFC 6761 set aside for exactly this
# use, so an address under them is documentation by construction rather than by
# somebody's judgement about a hostname.
address_skip_reason() {
  awk '
    {
      u = $0
      rest = substr(u, index(u, "://") + 3)
      hostport = rest
      if (index(hostport, "/") > 0) hostport = substr(hostport, 1, index(hostport, "/") - 1)
      if (index(hostport, "@") > 0) { print "carries a userinfo component, which this run never sends"; next }
      host = hostport
      if (index(host, ":") > 0) host = substr(host, 1, index(host, ":") - 1)
      host = tolower(host)
      if (host == "localhost" || host == "example.com" || host == "example.org" || host == "example.net") {
        print "a name reserved for documentation"; next
      }
      n = length(host)
      if (substr(host, n - 7) == ".example" || substr(host, n - 7) == ".invalid" ||
          substr(host, n - 4) == ".test" || substr(host, n - 9) == ".localhost") {
        print "a name reserved for documentation"; next
      }
      print ""
    }
  '
}

# Every tracked path, plus every directory holding one. A document naming a
# directory is naming something a reader can open, so the set a candidate is
# compared against is not the file list alone.
path_universe() {
  git ls-files | awk '
    {
      sub(/\r$/, "")
      if ($0 == "") next
      print
      p = $0
      while (1) {
        i = length(p)
        while (i > 0 && substr(p, i, 1) != "/") i--
        if (i <= 1) break
        p = substr(p, 1, i - 1)
        print p
      }
    }
  ' | sort -u
}

# The verdict for one document, from its scan and the universe.
#
# Three refusals, and the third is the one that keeps the escape a debt rather
# than a dispensation: a marker on a line that has nothing to excuse is refused as
# stale, so an escape cannot survive the sentence it was written for.
verdicts() {
  awk -v doc="$1" -v universe="$2" -v addrfile="$3" '
    BEGIN {
      while ((getline p < universe) > 0) { sub(/\r$/, "", p); if (p != "") known[p] = 1 }
      while ((getline a < addrfile) > 0) {
        sub(/\r$/, "", a)
        if (a == "") continue
        split(a, f, "\t")
        addr_on_line[f[1]] = 1
      }
    }
    {
      split($0, r, "\t")
      ln = r[1]; kind = r[2]; raw = r[3]; res = r[4]
      if (kind == "escape") {
        esc_line[ln] = 1
        esc_reason[ln] = raw
        next
      }
      cand[++nc] = ln "\t" kind "\t" raw "\t" res
    }
    END {
      for (i = 1; i <= nc; i++) {
        split(cand[i], c, "\t")
        ln = c[1]; kind = c[2]; raw = c[3]; res = c[4]
        if (res != "" && (res in known)) { resolved++; continue }
        if (ln in esc_line) { excused[ln] = 1; excused_count++; continue }
        printf "REFUSE\t%s\t%d\tnames %s `%s`, which is not a tracked path in this tree\n", doc, ln, kind, raw
        refusals++
      }
      for (ln in esc_line) {
        if (esc_reason[ln] == "") {
          printf "REFUSE\t%s\t%d\tcarries an example marker with no reason after the colon\n", doc, ln
          refusals++
          continue
        }
        if (!(ln in excused) && !(ln in addr_on_line)) {
          printf "REFUSE\t%s\t%d\tcarries an example marker and names nothing that needs one\n", doc, ln
          refusals++
        }
      }
      printf "COUNT\t%d\t%d\t%d\n", resolved + 0, excused_count + 0, refusals + 0
    }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture below judges against its own path universe rather than against
# this repository. A row that judged the real tree would prove the state of the
# tree on the day it ran, not the rule.
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

# The refusals a document produces against a fixed universe, one per line.
judge_fixture() {
  local doc="$1" universe="$2" body="$3"
  local pf af
  pf="$(mktemp)"; af="$(mktemp)"
  printf '%s' "$body" | scan_addresses > "$af"
  printf '%s' "$body" | scan_paths "$doc" > "$pf"
  verdicts "$doc" "$universe" "$af" < "$pf" | awk -F'\t' '$1 == "REFUSE" { print $2 ":" $3 ": " $4 }'
  rm -f "$pf" "$af"
}

selftest() {
  local uni
  uni="$(mktemp)"
  printf 'docs\ndocs/decisions\ndocs/decisions/0001-a.md\ndocs/decisions/README.md\nREADME.md\n' > "$uni"

  echo "== a named path resolves =="
  assert_out "passes: a code span naming a tracked file" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'The rule is in `docs/decisions/0001-a.md`.
')"
  assert_out "bites: the same span one digit different" \
    "docs/x.md:1: names span \`docs/decisions/0002-a.md\`, which is not a tracked path in this tree" \
    "$(judge_fixture "docs/x.md" "$uni" 'The rule is in `docs/decisions/0002-a.md`.
')"
  assert_out "passes: a code span naming a tracked directory" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'They live under `docs/decisions/`.
')"
  assert_out "bites: a directory that holds nothing tracked" \
    "docs/x.md:1: names span \`docs/records/\`, which is not a tracked path in this tree" \
    "$(judge_fixture "docs/x.md" "$uni" 'They live under `docs/records/`.
')"
  assert_out "bites: a path that climbs out of the repository root" \
    "docs/x.md:1: names span \`../secrets.md\`, which is not a tracked path in this tree" \
    "$(judge_fixture "docs/x.md" "$uni" 'Never `../secrets.md`.
')"

  echo "== a link target resolves against its own document =="
  assert_out "passes: a sibling named without a directory" \
    "" "$(judge_fixture "docs/decisions/README.md" "$uni" '- [0001](0001-a.md)
')"
  assert_out "bites: the same link from a document one directory up" \
    "docs/README.md:1: names link \`0001-a.md\`, which is not a tracked path in this tree" \
    "$(judge_fixture "docs/README.md" "$uni" '- [0001](0001-a.md)
')"
  assert_out "passes: a bare file name, which a link is checked for and a span is not" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'See [the notice](../README.md).
')"
  assert_out "passes: an anchor and a title, neither of which is part of the path" \
    "" "$(judge_fixture "docs/decisions/README.md" "$uni" 'See [it](0001-a.md#the-decision "Title").
')"
  assert_out "passes: an external address, which the other verb owns" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'See [the page](https://example.com/nowhere).
')"

  echo "== what a code span is not read as =="
  assert_out "passes: a bare name with no separator, which another gate names two dozen of" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'That gate runs `codeql.yml` and `prettier.yml`.
')"
  assert_out "passes: a last segment with no dot, which is a server path and not a file" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'Resolving `Users/Me` against the base drops the sub-path.
')"
  assert_out "passes: a span carrying a scheme" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'Typing `https://example.com/jellyfin` is a choice.
')"
  assert_out "passes: a span carrying a space, which is a check-run name" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'It reports `Package (JPRM) / Build package`.
')"
  assert_out "passes: a fenced block, which is a command and its output" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'Run it:

```
ls docs/decisions/0002-a.md
```
')"
  assert_out "passes: an indented block, which is the same thing this board writes more often" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'Run it:

    ls docs/decisions/0002-a.md
')"

  echo "== the example escape =="
  assert_out "bites: the line the escape is written for, without it" \
    "docs/x.md:1: names span \`docs/decisions/0002-a.md\`, which is not a tracked path in this tree" \
    "$(judge_fixture "docs/x.md" "$uni" 'A record would be at `docs/decisions/0002-a.md`.
')"
  assert_out "passes: the same line with the escape and a reason" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'A record would be at `docs/decisions/0002-a.md`. <!-- names-an-example: the number a later record would take -->
')"
  assert_out "bites: an escape with nothing after the colon" \
    "docs/x.md:1: carries an example marker with no reason after the colon" \
    "$(judge_fixture "docs/x.md" "$uni" 'A record would be at `docs/decisions/0002-a.md`. <!-- names-an-example: -->
')"
  assert_out "bites: an escape left behind after the sentence it excused resolved" \
    "docs/x.md:1: carries an example marker and names nothing that needs one" \
    "$(judge_fixture "docs/x.md" "$uni" 'The rule is in `docs/decisions/0001-a.md`. <!-- names-an-example: it was an example once -->
')"
  assert_out "passes: an escape on a line naming an external address and no path" \
    "" "$(judge_fixture "docs/x.md" "$uni" 'A server at https://myserver.invalid/x answers. <!-- names-an-example: nobody owns this name -->
')"
  assert_out "bites: a path named inside the escape reason is part of the reason" \
    "docs/x.md:1: carries an example marker and names nothing that needs one" \
    "$(judge_fixture "docs/x.md" "$uni" 'Nothing here. <!-- names-an-example: not even `docs/decisions/0002-a.md` -->
')"

  echo "== which external addresses are requested =="
  assert_out "not requested: an address carrying credentials somebody wrote down" \
    "carries a userinfo component, which this run never sends" \
    "$(printf 'https://name:secret@host/x' | address_skip_reason)"
  assert_out "not requested: a name reserved for documentation" \
    "a name reserved for documentation" \
    "$(printf 'https://example.com/jellyfin' | address_skip_reason)"
  assert_out "not requested: a reserved suffix rather than a reserved name" \
    "a name reserved for documentation" \
    "$(printf 'http://server.invalid:8096/Users' | address_skip_reason)"
  assert_out "requested: an ordinary address" \
    "" "$(printf 'https://github.com/Flowfin/core' | address_skip_reason)"
  assert_out "found: an address inside an indented block, where this board writes them" \
    "$(printf '2\taddress\thttps://github.com/Flowfin/core')" \
    "$(printf 'Run:\n    git clone https://github.com/Flowfin/core\n' | scan_addresses)"
  assert_out "found: an address with the sentence punctuation taken off" \
    "$(printf '1\taddress\thttps://github.com/Flowfin/core')" \
    "$(printf 'It is at https://github.com/Flowfin/core.\n' | scan_addresses)"

  rm -f "$uni"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures doc-path fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

documents() {
  git ls-files '*.md'
}

check() {
  local universe pf af doc out
  universe="$(mktemp)"
  path_universe > "$universe"

  local docs=0 candidates=0 excused=0 refusals=0
  local docs_list
  docs_list="$(documents)"

  echo "Documents read: every tracked file whose name ends in .md."
  echo "Compared against: $(awk 'END { print NR }' "$universe") tracked paths and the directories holding them."
  echo

  echo "-- names-a-path-that-resolves"
  pf="$(mktemp)"; af="$(mktemp)"
  for doc in $docs_list; do
    docs=$((docs + 1))
    scan_addresses < "$doc" > "$af"
    scan_paths "$doc" < "$doc" > "$pf"
    out="$(verdicts "$doc" "$universe" "$af" < "$pf")"
    while IFS=$'\t' read -r tag a b c; do
      case "$tag" in
        REFUSE)
          echo "::error file=${a},line=${b}::${a}:${b}: ${c}"
          echo "      ${a}:${b}: ${c}"
          refusals=$((refusals + 1))
          ;;
        COUNT)
          candidates=$((candidates + a))
          excused=$((excused + b))
          ;;
      esac
    done <<EOF
$out
EOF
  done
  rm -f "$pf" "$af" "$universe"

  if [ "$refusals" -eq 0 ]; then
    echo "ok    ${docs} document(s), ${candidates} named path(s) resolved, ${excused} excused as examples"
  else
    echo "      ${docs} document(s), ${candidates} named path(s) resolved, ${excused} excused as examples"
  fi
  echo

  echo "-- what this run did not read"
  echo "NOT MADE HERE: a command a document tells a reader to run. Checking that a verb exists needs a verb in the tree or a pinned toolchain to look in, and this repository has neither; #14 pins one and entry 2 of #1 decides what it is."
  echo "NOT MADE HERE: a path written in plain prose. Only a Markdown link target and a code span are read, so a path in an ordinary sentence passes unexamined."
  echo "NOT MADE HERE: a code span with no separator in it. docs/gate-parity.md names two dozen files inside another repository that way, and reading one as a local path would refuse every one of them."
  echo "NOT MADE HERE: anything inside a fenced or indented block. Those carry commands and their output, and the paths in them are frequently another repository's."
  echo "NOT MADE HERE: a tracked file that is not .md. Nothing else in this tree is prose today."
  echo

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} named path(s) do not resolve. Each one is printed above with its document and line."
    return 1
  fi
  echo "Every path these documents name resolves against the tracked set."
}

# --------------------------------------------------------------------------
# addresses
#
# Reported and never refused. An address outside this repository that is down for
# an hour is not a defect here, and a gate that reddens for it teaches people that
# red means nothing.
# --------------------------------------------------------------------------

addresses() {
  local doc line kind url reason code requested=0 skipped=0 answered=0 silent=0
  echo "External addresses named in tracked documents. This job reports and never refuses."
  echo

  for doc in $(documents); do
    while IFS=$'\t' read -r line kind url; do
      [ -n "${url:-}" ] || continue
      if [ "$kind" = "address-escaped" ]; then
        skipped=$((skipped + 1))
        echo "not requested  ${doc}:${line}  ${url}  (the line carries an example marker)"
        continue
      fi
      reason="$(printf '%s' "$url" | address_skip_reason)"
      if [ -n "$reason" ]; then
        skipped=$((skipped + 1))
        echo "not requested  ${doc}:${line}  ${url}  (${reason})"
        continue
      fi
      requested=$((requested + 1))
      code="$(curl -sS -o /dev/null -w '%{http_code}' -L --max-time 20 --retry 1 "$url" 2>/dev/null || printf 'none')"
      if [ "$code" = "none" ] || [ "$code" = "000" ]; then
        silent=$((silent + 1))
        echo "no answer      ${doc}:${line}  ${url}"
      else
        answered=$((answered + 1))
        echo "${code}            ${doc}:${line}  ${url}"
      fi
    done < <(scan_addresses < "$doc")
  done

  echo
  echo "Examined ${requested} address(es): ${answered} answered, ${silent} did not. ${skipped} were not requested."
  echo "NOT PROVEN HERE: nothing in the fixtures reaches the request itself, so what a status code does to this report is unmeasured. The classification above it is what the fixtures cover."
  return 0
}

case "${1:-}" in
  selftest)  selftest ;;
  check)     selftest && echo && check ;;
  addresses) addresses ;;
  *)         echo "usage: $0 selftest|check|addresses" >&2; exit 2 ;;
esac
