# Reporting a security problem

Report privately, through the form under Security on this repository:

    https://github.com/Flowfin/core/security/advisories/new

Please do not open an issue about a security problem. An issue here is public
from the moment it is submitted, and a report of this kind is the one thing
that should not be.

The route is a repository setting rather than a file, and it does not answer
today:

    gh api repos/Flowfin/core/private-vulnerability-reporting
    {"enabled":false}

Run 2026-08-19. So this file names the destination and states plainly that the
destination is currently shut, rather than sending a reporter to a door that
does not open. The address above is the one the form uses once the setting is
on, so nothing here moves when it changes.

The organisation-wide file in Flowfin/.github says the form is enabled on every
repository in this organisation. By the reading above that is not true of this
one today. Where the two disagree about this repository, this file is the one
that was measured against it.

Until the setting is on, the honest alternative is a public issue carrying as
little as the report can carry: the file and the claim, not the working path,
and no more of the mechanism than it takes to tell me where to look. That is a
poor arrangement for anything serious, and I would rather write it down as poor
than describe a private channel that is not there.

## What this repository is

Read the tree before deciding what to look for. The description calls this the
shared core every Flowfin client uses. What it holds is a Rust crate under `src/`
and its suite under `tests/`, decision records under `docs/decisions` and the
index that lists them, GitHub Actions workflows and the shell scripts those
workflows run, a gate-parity document, this file, the licence, a DCO, a notice, a
README, two issue templates with the config beside them, a pull-request template,
a manifest, a lockfile, a toolchain pin, and a `.gitattributes` that fixes the
line ending.

THIS PARAGRAPH NAMED NEITHER THE CRATE NOR THE SUITE, AND THE ONE AFTER IT SAID
THE CORE WAS NOT WRITTEN. Both were true when they were written and had stopped
being true. A reporter who believed either would not have opened the code,
because the file told them there was none. Read at
`5d67a074202de4d8069eee55d44812bf7fa9e201`:

    git rev-parse origin/main
    5d67a074202de4d8069eee55d44812bf7fa9e201
    git ls-tree -r --name-only origin/main | grep -c '^src/'
    10
    git ls-tree -r --name-only origin/main | grep -c '^tests/'
    6
    gh api repos/Flowfin/core --jq .language
    Rust

The inventory above is a list, and a list drifts, which is what happened. The
count beside it always said to derive it and still does:

    git ls-tree -r --name-only origin/main | wc -l

There are no releases and no tags. How many branches there are moves with every
change in flight, so it is derived here rather than counted:

    gh api repos/Flowfin/core/releases --jq length
    0
    gh api repos/Flowfin/core/tags --jq length
    0
    gh api repos/Flowfin/core/branches --jq length

WHAT IS ABSENT IS BEHAVIOUR RATHER THAN CODE, and that distinction is the one a
reporter needs. The crate compiles, a suite runs against it, and what the types
in it hold today is a name, a statement about which thread a caller may use it
from, and the measurement facility. There is no request, no cache, no decode and
no sign-in. `README.md` says the same thing in its own words, and the section
below on what is not a vulnerability here says what follows from it.

## Why a defect here is wider than one repository

Every client this organisation plans is meant to be built against these
records. A wrong sentence in one of them does not reach eleven clients by being
a bug; it reaches them by being implemented faithfully, in eleven places, by
people who were right to trust it. That is the failure this repository can have
today, and it is why its documents are in scope below.

The second radius arrives with the code. The core is a library inside each
client's process, holding that client's privileges, doing the network parsing
and the image decoding for all of them. Record 0003 names image decoding as the
most attacked surface this repository will carry, and 0101 puts every network
byte, every cached byte and every stored secret on the untrusted side. None of
that executes yet. All of it is already decided.

## What somebody could actually report

The workflows and the shell scripts they run are the only executable things in
this tree. How many of each there are moves whenever one lands, and a paragraph
that counts them is wrong the next time one does, which is what happened to this
one. Take both from the tree rather than from here:

    git ls-tree -r --name-only origin/main -- .github/workflows | wc -l
    git ls-tree -r --name-only origin/main | grep -E '\.sh$'

All but one of those workflows trigger on `pull_request`, so they run against a
branch and a pull request body a stranger controls. Which one does not is read
rather than named a second time:

    git grep -L 'pull_request:' origin/main -- .github/workflows/
    origin/main:.github/workflows/scorecard.yml

A way to make one of them run with more than its declared permission, run
against a fork's branch, or execute something a pull request supplied is a real
report. Today none of them uses `pull_request_target`, every checkout runs with
`persist-credentials: false`, and the pull request body reaches
`.github/pr-hygiene/hygiene.sh` through the environment and a file rather than
by interpolation:

    git grep -c 'pull_request_target' origin/main -- .github/workflows/ ; echo "exit=$?"
    exit=1

    git grep -l 'actions/checkout' origin/main -- .github/workflows/ | wc -l
    git grep -l 'persist-credentials: false' origin/main -- .github/workflows/ | wc -l

Those last two answer with the same number while that sentence holds, and stop
doing so on the day a checkout arrives without it.

Most jobs a pull request triggers here declare read scopes only. The ones that do
not grant `security-events: write`, which is what an upload to the code-scanning
tab costs, and it is the widest scope any pull-request-triggered job here
declares. Which files those are moves whenever a leg starts or stops uploading,
so read it rather than taking a name from this paragraph:

    git grep -n ': write' origin/main -- .github/workflows/

One of the files that reading returns is `.github/workflows/scorecard.yml`, which
is the one no pull request triggers, named above, and it is also the only job here
holding anything beyond that one scope. A hole in any of that, present now or
introduced later, is the thing to send.

An action pinned to a tag or a moving reference instead of a commit, or a pin
whose version comment does not match the commit it names.

A decision record that commits every future client to something unsafe. This is
the class particular to this repository, and it is a real report even though
nothing executes it. Concretely: a bound stated after an allocation instead of
before it, a rule that would let a session token reach the byte store in 0040,
a redirect rule in 0069 that widens the set of destinations on a server's
say-so, a field in 0071 that carries a secret through to a diagnostic sink, a
cache key in 0041 under which two accounts on one device could collide, or
anything in 0029 that would let the core weaken certificate validation on its
own.

A document that names an external address which no longer belongs to whoever it
belonged to when the paragraph was written.

## What is not a vulnerability here

The absence of an implementation. The core opens no socket, writes no byte to a
disk and parses nothing that came off a network, so a report that it fails to
validate its input has measured the state of the tree rather than found a defect
in it. Behaviour that has not been written is a plan, not a hole. The scripts are
the exception, and they are in scope above.

THIS PARAGRAPH SAID THAT WAS SO BECAUSE NONE OF IT WAS WRITTEN, AND THAT IS NO
LONGER THE WHOLE REASON. Some of the crate is written. What keeps a socket and a
file out of it now is a refusal as well as an absence: the `invariants` check
refuses a network call anywhere under `src/` outside the one transport, and a
filesystem call anywhere under `src/` at all. Neither is a claim in this file:

    git grep -n '^id: no-network-outside-the-transport' -- .github/invariants/rules
    git grep -n '^id: no-filesystem-access' -- .github/invariants/rules
    git grep -c 'std::net|TcpStream|std::fs' -- src/ ; echo "exit=$?"
    exit=1

The first two return the rules, the third selects nothing and says so with its
exit code. A rule that stopped biting is itself a report, and it is a better one
than the absence it replaced.

Disagreeing with a decision. Every record states the alternatives it rejected
and what each would have cost. Preferring one of them is a good issue and is not
an advisory, and it gets a better answer on the tracker, where that argument
already lives.

An alert copied out of the code-scanning tab. The Scorecard workflow says in its
own header that none of its findings are triaged yet, so those alerts sit where
that workflow published them. Sending one back to me privately reports what I
already published about myself.

Anything below the layer this project occupies: a compromised operating system,
a platform keychain that can already be read, a device whose owner has removed
the platform's own restrictions. Record 0101 names these as out of scope, and
this file does not widen it.

A Jellyfin server that lies. The core will not be able to tell a healthy server
from a compromised one and does not try; what it promises against a hostile one
is narrow, and 0101 states it exactly. A problem in Jellyfin itself belongs to
[the Jellyfin project](https://github.com/jellyfin/jellyfin/security/policy),
and a report that lands here instead is pointed the right way rather than
closed.

Scan output against a host. This repository serves nothing and deploys nowhere,
so there is no endpoint here to scan. Anything about flowfin.dev belongs to
Flowfin/site.

## What a reporter gets

Every report is answered, whether or not it turns out to be a problem, and one
that is not gets the reason it is not.

There is no acknowledgement deadline, and there will not be one. A window this
project cannot hold is worse than no window at all: a reporter told to expect an
answer by a certain day and left without one cannot tell a busy week from a
report that never arrived, and that guessing is the thing a deadline was
supposed to remove. With the private channel shut today, a promise about timing
would rest on a route that does not exist.

Credit goes to the reporter unless they ask otherwise. A working exploit is
useful in the report and not in public before there is a fix.

## What is covered

The `main` branch, which is the only branch. There are no releases and no tags,
so there is nothing to backport a fix to and no older version maintained in
parallel. When this repository starts publishing something, that sentence stops
being true and this file changes with it.
