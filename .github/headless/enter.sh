#!/usr/bin/env bash
# Enter the environment the suite is required to run in, then run it (#20).
#
# THIS FILE CREATES THE THREE PROPERTIES. `.github/headless/headless.sh` reads
# them back and refuses if they are not there, and the two are deliberately
# separate: a sandbox nobody checks is a sandbox that silently stops being one,
# which is the failure #20 is about.
#
# The three, and what each one costs:
#
#   No display server.    Nothing is done here for it. The runner image carries
#                         none, and `headless.sh` refuses if one appears rather
#                         than this file pretending to remove it.
#
#   Not elevated.         The suite runs under the runner's own unprivileged
#                         user, with `--no-new-privs` set, so a test that asks
#                         for elevation is refused by the kernel rather than
#                         granted by a passwordless sudo. Being non-root is not
#                         the same as being unable to become root, and on this
#                         runner the second is the one that matters.
#
#   Loopback and nothing  A network namespace with only `lo` in it. This is what
#   else.                 makes the concrete case in `CONTRIBUTING.md` fail: a
#                         test binding a socket to the machine's own interface
#                         address cannot, because that address does not exist in
#                         here. An address filter would not do it - a filter
#                         drops packets and a bind is a local operation that
#                         succeeds before any packet exists.
#
# WHY THIS RUNS ITSELF THREE TIMES RATHER THAN NESTING QUOTED SHELL. Each stage
# is one `exec` into the next, and the alternative is a command line carrying two
# levels of quoted script, which is where a sandbox stops being read by anybody
# and stops being correct shortly afterwards.
#
# The privilege goes up once, at `sudo`, and comes straight back down at
# `setpriv` without the suite ever running in between. Nothing between those two
# lines does anything but bring loopback up.

set -euo pipefail

case "${1:-outside}" in
  outside)
    if [ "$(id -u)" -eq 0 ]; then
      echo "::error::This is already running as uid 0. The suite is meant to run unprivileged, and dropping from root here would hide who it started as."
      exit 1
    fi
    # HOME carries the toolchain manager's own directories, and without it the
    # compiler resolved inside would be root's rather than the pinned one.
    exec sudo --preserve-env=HOME,PATH,CARGO_HOME,RUSTUP_HOME \
      unshare --net -- \
      bash "$0" inside "$(id -u)" "$(id -g)"
    ;;
  inside)
    # A fresh network namespace carries loopback down. A suite that binds to it
    # would fail for the wrong reason.
    ip link set lo up
    exec setpriv --reuid="$2" --regid="$3" --clear-groups --no-new-privs -- \
      bash "$0" suite
    ;;
  suite)
    # The reading first, then the run. A suite that ran in the wrong environment
    # proves nothing about the environment it is meant to run in.
    bash .github/headless/headless.sh check
    exec bash .github/test/test.sh check
    ;;
  *)
    echo "usage: $0 [outside|inside <uid> <gid>|suite]" >&2
    exit 2
    ;;
esac
