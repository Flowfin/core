#!/usr/bin/env bash
# The suite runs headless, non-elevated and on loopback alone, and this reads
# whether it did (#20).
#
# The rules live here as shell functions rather than as steps inside the workflow
# because each one owes a fixture proving it bites, and a fixture run against a
# second copy of the logic proves the copy. `selftest` and `check` call the same
# functions, so a rule cannot pass its fixture and refuse something else in the
# gate. That is the arrangement the other scripts in this gate already use.
#
# WHAT THIS IS AND WHAT IT IS NOT. It does not create the three properties. The
# workflow does that, by entering a network namespace that carries only loopback
# and dropping into it with new privileges refused, and `.github/workflows/test.yml`
# is where that is written and argued. This script READS the environment the
# suite is about to run in and refuses to run it in the wrong one. The two are
# separate on purpose: a sandbox nobody checks is a sandbox that silently stops
# being one, which is the shape #20 is about - a property that is impossible to
# add later and easy to lose without anybody noticing.
#
# THE THIRD PROPERTY IS READ AS AN ADDRESS LIST RATHER THAN AS A REACHABILITY
# TEST. Asking whether some host answers proves the state of the network on the
# day it ran; asking which addresses this process can see is a property of the
# namespace it is in. A process that can see only loopback cannot bind a socket
# to a machine's own interface address, which is the concrete case
# `CONTRIBUTING.md` names, because that address does not exist for it.
#
# Verbs:
#   selftest   run every fixture and prove each rule bites
#   check      read the environment the suite is about to run in, and refuse
#
# No POSIX character classes and no interval expressions in any pattern below.
# The awk on the runner is mawk and the awk on a contributor's machine is
# frequently gawk, and those two constructs are where the older mawk builds
# disagree with it. A rule that matches on one machine and not on the other is a
# gate whose verdict depends on who ran it.

set -euo pipefail

# Where a display server puts the socket a client connects to. Read as well as
# the two variables, because a variable somebody unset does not remove the
# server, and a test that opens a display finds the socket without being told
# where it is.
X_SOCKET_DIR="/tmp/.X11-unix"

# --------------------------------------------------------------------------
# Rules. Each reads its subject on stdin and writes records to stdout, one per
# line, as VERDICT<TAB>SUBJECT<TAB>DETAIL.
#
# awk rather than grep: grep exits 1 when it selects nothing, which is the
# ordinary answer for two of these, and a pipeline that has to tell "nothing
# matched" from "the scanner broke" one `set -o pipefail` at a time is how a gate
# ends up passing on everything.
# --------------------------------------------------------------------------

# The interfaces this process can see, read from the address listing on stdin.
# Anything other than loopback is refused by name.
judge_addresses() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /^[ \t]*$/) next
      seen = seen + 1
      if (line == "lo") next
      printf "REFUSE\t%s\tis an interface other than loopback\n", line
      bad = bad + 1
    }
    END {
      if (seen == 0) {
        printf "REFUSE\t(none)\tno interface was listed at all, so what this process can reach cannot be read\n"
        exit
      }
      if (bad == 0) printf "ALLOW\tlo\tis the only interface this process can see\n"
    }
  '
}

# The environment, read as NAME=VALUE lines on stdin. Only the variables a
# display client reads are judged; everything else is passed over, because this
# is a rule about a display rather than an inventory of the environment.
judge_display_variables() {
  awk '
    {
      line = $0
      sub(/\r$/, "", line)
      i = index(line, "=")
      if (i == 0) next
      name = substr(line, 1, i - 1)
      value = substr(line, i + 1)
      if (name != "DISPLAY" && name != "WAYLAND_DISPLAY") next
      if (value == "") next
      printf "REFUSE\t%s\tnames a display server: %s\n", name, value
      bad = bad + 1
    }
    END { if (bad == 0) printf "ALLOW\t(display)\tneither DISPLAY nor WAYLAND_DISPLAY names a display server\n" }
  '
}

# --------------------------------------------------------------------------
# selftest
#
# Every fixture judges its own text rather than this machine. A row that judged
# the real environment would prove the state of the machine on the day it ran,
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

judge_a() { printf '%s' "$1" | judge_addresses; }
judge_d() { printf '%s' "$1" | judge_display_variables; }

selftest() {
  echo "== the interfaces this process can see =="
  assert_out "passes: loopback alone, which is the namespace the suite runs in" \
    "$(printf 'ALLOW\tlo\tis the only interface this process can see')" \
    "$(judge_a 'lo
')"
  assert_out "bites: the machine's own interface, which is what a non-loopback bind reaches" \
    "$(printf 'REFUSE\teth0\tis an interface other than loopback')" \
    "$(judge_a 'lo
eth0
')"
  assert_out "bites: several, each named rather than counted" \
    "$(printf 'REFUSE\tdocker0\tis an interface other than loopback\nREFUSE\teth0\tis an interface other than loopback')" \
    "$(judge_a 'docker0
eth0
lo
')"
  assert_out "bites: an empty listing, which is not the same as loopback alone" \
    "$(printf 'REFUSE\t(none)\tno interface was listed at all, so what this process can reach cannot be read')" \
    "$(judge_a '')"

  echo "== the variables a display client reads =="
  assert_out "passes: neither set" \
    "$(printf 'ALLOW\t(display)\tneither DISPLAY nor WAYLAND_DISPLAY names a display server')" \
    "$(judge_d 'HOME=/home/runner
PATH=/usr/bin
')"
  assert_out "bites: an X display" \
    "$(printf 'REFUSE\tDISPLAY\tnames a display server: :99')" \
    "$(judge_d 'DISPLAY=:99
')"
  assert_out "bites: a Wayland display" \
    "$(printf 'REFUSE\tWAYLAND_DISPLAY\tnames a display server: wayland-0')" \
    "$(judge_d 'WAYLAND_DISPLAY=wayland-0
')"
  assert_out "passes over: the variable set to nothing, which names no server" \
    "$(printf 'ALLOW\t(display)\tneither DISPLAY nor WAYLAND_DISPLAY names a display server')" \
    "$(judge_d 'DISPLAY=
')"
  assert_out "passes over: a variable whose name merely contains the word" \
    "$(printf 'ALLOW\t(display)\tneither DISPLAY nor WAYLAND_DISPLAY names a display server')" \
    "$(judge_d 'NO_DISPLAY=:99
DISPLAY_MANAGER=gdm
')"

  echo
  if [ "$selftest_failures" -ne 0 ]; then
    echo "::error::$selftest_failures headless-gate fixture(s) did not hold. The rules below are not the rules that were proven, so this run judges nothing."
    return 1
  fi
  echo "Every fixture held. The rules the gate applies are the rules these fixtures ran."
}

# --------------------------------------------------------------------------
# check
# --------------------------------------------------------------------------

check() {
  local refusals=0 verdict subject detail uid sockets addresses

  echo "-- no display server"
  while IFS=$'\t' read -r verdict subject detail; do
    [ -n "${verdict:-}" ] || continue
    case "$verdict" in
      ALLOW) echo "      ${subject} ${detail}" ;;
      REFUSE)
        refusals=$((refusals + 1))
        echo "::error::${subject} ${detail}"
        echo "      REFUSED: ${subject} ${detail}"
        ;;
    esac
  done < <(env | judge_display_variables)

  # A variable somebody unset does not remove the server. The socket is what a
  # client finds when the variable is absent, so it is read as well.
  if [ -d "$X_SOCKET_DIR" ]; then
    sockets="$(find "$X_SOCKET_DIR" -maxdepth 1 -name 'X*' 2>/dev/null | wc -l | tr -d ' ')"
  else
    sockets=0
  fi
  if [ "$sockets" -ne 0 ]; then
    refusals=$((refusals + 1))
    echo "::error::${X_SOCKET_DIR} carries ${sockets} display socket(s). A display server is present whatever the environment says."
    echo "      REFUSED: ${sockets} socket(s) under ${X_SOCKET_DIR}"
  else
    echo "      no display socket under ${X_SOCKET_DIR}"
  fi
  echo

  echo "-- not elevated"
  uid="$(id -u)"
  if [ "$uid" -eq 0 ]; then
    refusals=$((refusals + 1))
    echo "::error::The suite is about to run as uid 0. A test that requests elevation would be granted it, and the requirement is that such a test fails."
    echo "      REFUSED: running as uid 0"
  else
    echo "      running as uid ${uid}"
  fi

  # Being non-root is not the same as being unable to become root. The workflow
  # asks the kernel to refuse every privilege gain for this process tree, and
  # this reads the flag back rather than trusting that it was asked for.
  if [ -r /proc/self/status ]; then
    if grep -qx 'NoNewPrivs:	1' /proc/self/status; then
      echo "      new privileges are refused for this process tree (NoNewPrivs is 1)"
    else
      refusals=$((refusals + 1))
      echo "::error::NoNewPrivs is not set for this process, so a test could still gain privileges through a setuid program. The requirement is that a test requesting elevation fails."
      echo "      REFUSED: NoNewPrivs is not 1"
    fi
  else
    refusals=$((refusals + 1))
    echo "::error::/proc/self/status is not readable, so whether new privileges are refused cannot be read. Refusing rather than assuming."
    echo "      REFUSED: /proc/self/status is not readable"
  fi
  echo

  echo "-- loopback and nothing else"
  addresses="$(ip -o addr show | awk '{ print $2 }' | sort -u)"
  while IFS=$'\t' read -r verdict subject detail; do
    [ -n "${verdict:-}" ] || continue
    case "$verdict" in
      ALLOW) echo "      ${subject} ${detail}" ;;
      REFUSE)
        refusals=$((refusals + 1))
        echo "::error::${subject} ${detail}"
        echo "      REFUSED: ${subject} ${detail}"
        ;;
    esac
  done < <(printf '%s\n' "$addresses" | judge_addresses)
  echo
  echo "      the interfaces this process can see:"
  printf '%s\n' "$addresses" | sed 's/^/        /'
  echo

  if [ "$refusals" -ne 0 ]; then
    echo "::error::${refusals} headless requirement(s) were not met, so the suite was not run. A suite that ran in the wrong environment proves nothing about the environment it is meant to run in."
    return 1
  fi

  echo "-- what this run did not read"
  echo "NOT MADE HERE: whether a test would actually try any of the three. This reads the environment the suite runs in; what the suite contains is what the suite contains."
  echo "NOT MADE HERE: a bind to the wildcard address. A process that sees only loopback cannot bind the machine's own interface address, and it can still bind 0.0.0.0, which reaches nothing here and is not refused."
  echo "NOT MADE HERE: anything on a contributor's own machine. This is what the gate's environment is, and CONTRIBUTING.md is what asks a contributor for the same property."
  echo
  echo "The suite may run: no display server, uid ${uid} with new privileges refused, loopback and nothing else."
}

case "${1:-}" in
  selftest) selftest ;;
  check)    selftest && echo && check ;;
  *)        echo "usage: $0 selftest|check" >&2; exit 2 ;;
esac
