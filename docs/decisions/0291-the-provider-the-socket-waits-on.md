# 0291. The provider the socket waits on, and what the wait costs

Date: 2026-09-04

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #291

## The decision

The manifest entry the means in
[0243](0243-the-means-a-certificate-is-validated-with.md) requires does not
arrive until a pure-Rust crypto provider for `rustls` reaches a release the
target leg compiles on every triple in
[0113](0113-the-target-triples-the-gate-compiles-for.md)'s set; until then no C
cross-toolchain is added to that leg, no triple leaves that set, and the socket
in #27 waits with everything behind it.

## What was in the way, and it was not the means

[0243](0243-the-means-a-certificate-is-validated-with.md) decided what validates
a certificate and stopped one step short of the thing that decides when it can be
built. Every crypto provider `rustls` offers is C; the target leg compiles the
library for seven triples on one Linux runner with no C cross-toolchain at all;
so the first compile after the entry arrives goes red on every triple whose
compiler the runner does not carry. That record measured the shape, named three
ways out and took none of them, and #291 is where the choice sat.

The leg and the set it compiles, read at the mainline this record was written
against:

    git rev-parse origin/main
    a9dfdca1a71410761bf54f97738b5ba65cc9e581
    git show "origin/main:.github/workflows/targets.yml" | grep -n 'runs-on'
    69:    runs-on: ubuntu-latest
    git show "origin/main:.github/targets/targets" | grep -v '^#' | grep -c .
    7

The graph is still the one that needs no C compiler, and nothing in it is the
provider or the crates that would bring it:

    git show origin/main:Cargo.lock | grep -c '^\[\[package\]\]'
    17
    git show origin/main:Cargo.lock | grep -cE '^name = "(rustls|rustls-platform-verifier|rustls-rustcrypto|aws-lc-sys|ring)"'
    0

The provider that would end the wait states its own maturity in its version
string, re-read for this record rather than carried over from
[0243](0243-the-means-a-certificate-is-validated-with.md):

    cargo search rustls-rustcrypto --limit 1
    rustls-rustcrypto = "0.0.2-alpha"    # Pure Rust cryptography provider for the Rustls TLS library...

## Why the reversible way out was taken

The choice was taken on #291 on 2026-09-04, and this record executes it.

Of the three ways out, waiting is the only one that is reversible and the only
one that costs neither a runner bill nor a client platform. A C cross-toolchain
per triple is a bill that grows with every triple added and a matrix somebody
maintains afterwards; a triple leaving
[0113](0113-the-target-triples-the-gate-compiles-for.md)'s set is a client
platform somebody loses, and that record is explicit that a triple leaves the set
there rather than anywhere else. Both are paid today, permanently, for a cost
whose replacement already exists and matures on somebody else's clock. Waiting is
the one that can be un-taken on the day the version string changes, and nothing
has to be undone when it is.

What waiting does not buy is any part of the certificate evaluation
[0029](0029-certificate-validation-and-the-self-signed-server.md) requires. It
buys the order in which the two costs are paid, and that is the whole of it.

## What this costs, and it is the largest thing in this record

The socket is what the rest of this board's building sits behind. Nothing under
`src/` reaches the network at all:

    git grep -n 'std::net' origin/main -- src/ ; echo "exit=$?"
    exit=1

so every condition on this board that names a request, an exchange, a sign-in or
a session is unreachable for as long as this record holds, and the issues that
carry those conditions stay open with their decisions landed and their code
unwritten. The tracker is where that population is read rather than listed here,
because a list in this record would be a second declaration of it and would
disagree with the tracker on the first day one of them moves:

    gh issue list --repo Flowfin/core --state open --limit 500 --json number --jq 'length'
    55

This is a decision to hold a milestone rather than a scheduling detail, and it is
written as one. A reader who takes this record for a note about a build matrix
has read the smaller half of it.

## What the target leg does today, and what changes when the entry arrives

Today it compiles all seven and there is nothing for it to report: the graph is
pure Rust and no triple is short of a compiler. The wait changes no line of it,
which is why this record touches neither the target register nor the workflow
beside it.

What the choice asks of the change that finally adds the entry is that the leg
builds what it can build and names the triples it did not, so the gap is a
printed line rather than a silent green. That is a requirement on that change and
not a thing this record builds. Nothing in the tree does it today and nothing
refuses its absence, because there is no triple it could be absent for; the
sentence is here so the obligation arrives with the entry rather than being
invented alongside it by whoever meets the red leg first.

## What this record does not do

It does not add a dependency. `Cargo.toml` and `Cargo.lock` are unchanged by it,
and the clause
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) requires beside a
manifest entry is still written when the entry arrives.

It does not narrow a clause of
[0243](0243-the-means-a-certificate-is-validated-with.md), so it carries no
`Narrows:` field and that record receives no `Narrowed-by:` one. It takes one of
the three ways out that record priced and left open, which is going further on a
case rather than narrowing a clause, and
[0267](0267-a-record-that-narrows-one-clause-of-another.md) gives the field pair
to the second of those and not to the first. A reader of
[0243](0243-the-means-a-certificate-is-validated-with.md) who follows its own
reversal condition to the version string arrives here on their own.

It does not settle the Android licence term, which is #268, and it does not touch
the means the core speaks HTTP with, which is
[0292](0292-the-means-the-core-speaks-http-with.md) and is already decided and
already in the graph. Those are the two neighbours of this question and neither
moves.

It refuses one thing by name, and #291 opened asking for that refusal: a
transport landing with a socket and no TLS, on the reading that
[0028](0028-the-address-a-person-typed.md) honours a typed address in the clear
scheme. That sends a credential in clear over any address the core is handed, in
a core whose whole position is that a credential travels to one place, and it is
the way out that gets taken by somebody meeting a red gate at the end of a change
rather than at the start of one.

## What was not measured for this

Which C cross-compilers the runner image carries. #291's body names that
measurement as its own first act and it is still unmade; every cross-compile
reading behind this question, in
[0243](0243-the-means-a-certificate-is-validated-with.md) and on #291, was taken
on a Windows machine outside this tree and is that machine's rather than the
runner's.

The choice does not rest on it, and that is why this record lands without it:
under the way out taken, no cross-toolchain is added and no triple is dropped, so
which compilers the Linux runner image carries changes nothing about what is
done. It would have decided between the other two. A reading showing the runner
already carries a working C cross-toolchain for all seven triples is a reversal
condition below rather than a gap in the argument here, and taking it needs a run
on the runner rather than a claim about the image.

## Why this is written down before the code

A wait that lives in a comment is a wait the next reader re-derives from prose,
and this one has been re-derived on #27 more than once, each time by walking a
chain of records to find out that the socket cannot be opened yet. The reading is
cheap once and the third one is waste.

The specific failure is narrower and it is the fourth way out above. Written
nowhere, this choice is met for the first time as a red target leg on somebody's
branch, at the end of a change rather than at the start of one, and the cheapest
thing to do with a red gate at that point is to take the entry back out and open
the socket without it. That is one commit away from a core that speaks in clear,
and whoever does it will have a green gate to show for it.

The neighbouring failure is the opposite one and costs less: a runner bill or a
dropped client platform taken inside a change about a transport, because the leg
was red and the record that priced both was one directory away. Either is a
decision above the issue being worked, taken by whoever met the gate first.

## Alternatives, and what each cost

A C cross-toolchain per triple on the target leg: an Android NDK on the Linux
runner and a runner per Apple platform family, because an Apple SDK does not run
on Linux. It costs the minutes on every pull request, a matrix somebody
maintains, and the sentence in the target register saying the set is a set of
client platforms and not of runners - which is the sentence that stops the set
being trimmed to what a runner happens to have. It buys the socket now.

A triple leaves [0113](0113-the-target-triples-the-gate-compiles-for.md)'s set
until it can be served. It is the cheapest in code of the three and the only one
that also removes the Android licence term #268 answers.
[0243](0243-the-means-a-certificate-is-validated-with.md) names it for that
reason. It costs a client platform, permanently in practice: a platform that
leaves a gate's set is not compiled, and what is not compiled is not returned to
without somebody making the argument that was never written down for taking it
out.

The pure-Rust provider today, at the version string quoted above. It costs the
certificate evaluation of every client, on a pre-release, and
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) answers a
cryptographic primitive by class: a wrong implementation is a defect nobody sees
until it is exploited. That version string is what this record waits on rather
than what it takes.

A socket with no TLS behind a flag or a scheme. Refused by name above, and #291
opened asking for the refusal rather than discovering it.

## What would reverse this

A pure-Rust provider for `rustls` reaches a release the target leg compiles on
every triple in
[0113](0113-the-target-triples-the-gate-compiles-for.md)'s set. The version
string is the first half and a compile on all seven is the second, and the second
is the one that ends the wait: a stable release that still fails a triple leaves
this record standing. That is also
[0243](0243-the-means-a-certificate-is-validated-with.md)'s own reversal
condition arriving, and the record naming the provider supersedes both.

The runner image is measured carrying a working C cross-toolchain for every
triple in that set, on the runner rather than on a contributor's machine. The
first way out then costs no bill and no maintenance, the reason for waiting is
gone, and the choice is retaken rather than inherited.

A triple leaves that set under
[0113](0113-the-target-triples-the-gate-compiles-for.md) for a reason of its own,
and what is left is a set the provider builds on. The wait ends as a side effect
of a decision this record does not take, which is worth naming so that it is
noticed rather than discovered.

The count of open issues on this board that can be worked without a request
reaches zero. The wait is then the whole of this board's throughput rather than
one milestone's, which is a different price from the one weighed above, and the
choice is retaken against it:

    gh issue list --repo Flowfin/core --state open --limit 500 --json number,labels \
      --jq '[.[] | select([.labels[].name] | map(startswith("blocked-on-")) | any | not)] | length'
    25
