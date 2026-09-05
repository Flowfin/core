# 0091. What the first release contains, and what it does not

Date: 2026-08-31

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #91

## The decision

The first release of this repository is the library compiled for the triples
[0113](0113-the-target-triples-the-gate-compiles-for.md) names together with the
probe #92 builds, published as a `0.x` tag on this repository alone with its
checksums and #87's attestations beside it, described by #95 as something an
operator points at their own server rather than as a client; it contains no user
interface, no playback and no client for any platform; and it is called a release
rather than a preview only when every condition listed below is a closed issue or
a green check rather than a judgement somebody makes on the day.

## What a release of this repository is

A library has nothing to run, which is #92's opening sentence and the reason this
record has to answer what a release even means here before anything is tagged. An
operator cannot form an opinion about something they cannot execute, so the
release is the smallest thing they can: the library, plus one program that drives
it against their own server and prints what happened.

That fixes the register the documentation is written in. The first paragraph of
#95 says what this is and what it is not, and the sentence it may not write is any
that lets a reader install this expecting a client.

Four answers this rests on were taken on 2026-08-24, on the issue that collects
the decisions this plan may not make:

    gh issue view 1 --repo Flowfin/core --json comments \
      --jq '.comments[] | select(.body | startswith("Answering the open entries")) | .body'

Entry 2 is Rust with a foreign function interface per platform, so there is a
library per target triple to release at all. Entry 3 is the two server lines
10.11 and 12.0. Entry 4 is that the interface stays at `0.x` and breaks freely
until a client actually consumes it. Entry 6 is releases on this repository only,
with the attestations attached and no language registry before a consumer exists.

## What the first release contains

Each line is a closed issue rather than a description of work, because a scope
whose items are descriptions is a scope somebody grades on the day.

The library, compiled for every triple in
[0113](0113-the-target-triples-the-gate-compiles-for.md)'s set, built only by the
release workflow from a tag. #94, and #14 for the pinned toolchain it is built
with.

    git show origin/main:.github/targets/targets | grep -cE '^[a-z0-9_]+-'
    7

The probe, run against a real server by the operator who installed it, printing a
timing for every step and exercising each unhappy path on demand. #92, and behind
it #39 for the library surface it lists, #49 for the artwork it fetches and #57
for the position it reports. The unhappy paths it offers are the ones the core has
routes for: #30, #31 and #32 for the sign-in routes, #34 and #35 for a token that
dies, #44 for a slow server and #45 for a server that is gone. A route no issue
above has closed is a route the probe does not offer and #95 lists as absent.

The checksums, and the bill of materials and the provenance attestation beside
them. #87, attached by #94, which is what entry 6 decided the release carries in
place of a registry.

A changelog section for the version, refused by a check rather than by a reviewer
when it is missing. #93, and #78 behind it for the scheme that section is written
against.

The documentation an operator reads before installing. #95, with the data page
from #74 linked and summarised, and the licence named:

    gh api repos/Flowfin/core --jq '.license.spdx_id'
    AGPL-3.0

The licence the reading above names is the one that was superseded. What #95
names is the answer that stands, in
[0303](0303-the-licence-the-core-is-offered-under.md).

## What it does not contain, said before it is installed rather than after

No user interface. No playback of anything. No client for any platform. Those
three are [0003](0003-what-the-core-does-not-do.md)'s boundary rather than a gap
this release happens to leave, and #95 writes them in its first paragraph rather
than in a limitations section at the foot.

No package in any language registry. Entry 6 answered that a registry can come
later, additively, when a client needs it, so the first release publishes bytes
and a checksum and nothing an author adds one line to a manifest for.

No frozen interface. Entry 4 keeps this at `0.x`, so the version scheme #93 adopts
is a note to whoever reads this repository rather than a promise to strangers, and
#95 says so rather than letting a version number imply the other thing.

No server line beyond the two in entry 3. What the release is built and tested
against is #88's set, and a line outside it is untested rather than unsupported by
implication.

No verification on a platform family #97 did not reach. That issue's own condition
requires the unverified families to be named in its run output, so the release
names them too rather than letting a green run read as covering all seven triples.

## The bar, as conditions rather than as a feeling

It is a release when all of the following hold, and a preview until then.

Every issue named in the two sections above is closed.

The tag ran the full gate before anything was published, and the publishing job
restored no cache. #96.

The verification on a machine that never built the artifact passed for each
claimed platform family, and a deliberately corrupted artifact failed it. #97.

The gate's check names are required on `main`, so that a red check refuses the
merge that produced the tag rather than being weighed by a person. #26.

Nothing above asks anybody to decide whether the thing is ready. Each one is a
closed issue, a green check or a run whose failure mode was demonstrated, which is
the whole of what this record is for.

## The speed numbers this release publishes, and the ones it does not

[0008](0008-what-the-core-can-measure-of-the-speed-budget.md) fixes which numbers
the core can measure alone and
[0064](0064-the-numbers-the-core-does-not-report.md) fixes the two it cannot. This
record adds one condition on top of them and no new numbers.

A speed number appears in the first release only where #67 has published it with
the command that produced it, measured by #65's harness. Every number in
[0008](0008-what-the-core-can-measure-of-the-speed-budget.md) that #67 has not
published by then is listed in #95 as not measured, in those words.

The two published targets #62 and #63 are the core's share and never the whole
number, which is
[0064](0064-the-numbers-the-core-does-not-report.md)'s sentence rather than this
record's, and #95 repeats it beside any figure it quotes.

This is deliberately not "the numbers we have by then". A list assembled on the
day is a list whose absences are invisible, and the absence is the part an
operator needs.

## Why this is written down before the code

Nothing is published yet:

    gh api repos/Flowfin/core/releases --jq 'length'
    0

which is the only moment this can be written without the answer being read off
whatever happened to be finished. A scope decided at the tag is a scope shaped by
what compiled that week, and the parts that did not compile leave no trace in it
at all.

The specific failure is narrower and it is the one #95 exists against. A
repository with a tag on it is read as a product by everybody who did not follow
the tracker, and the first person to install this will be looking for a client.
The distance between what they expect and what this is cannot be closed by a
release note written after they have installed it, because by then the sentence
they needed was the first one.

The second failure is the bar. "Is it ready" asked on the day is answered by
whoever is most tired of asking, and a preview shipped as a release is not
withdrawn afterwards - the version number is already in somebody's manifest and
the expectation is already set. Conditions written now are conditions nobody has
an interest in yet, which is the only kind worth writing.

## Alternatives, and what each cost

No release until a client exists. The most honest position, and it is what entry 6
priced for the publishing route. It costs the feedback the probe was built to
collect: the core would reach its first real server inside somebody's client, at
the moment when a second thing is already going wrong, and the fake server would
have been the only audience for every measurement until then.

A release of the library alone, with no probe. Smaller, and it is what a library
repository normally publishes. It costs the operator entirely, since a library is
not something they can run, and it costs this repository the only route by which a
real server's behaviour reaches it before a client is written.

A release when a milestone completes rather than when a list of issues closes.
Simpler to state and it reads well on a plan. It costs precision in the direction
that matters: a milestone is a bucket somebody can move an issue out of, and the
scope would then be whatever remained in it, decided by nobody.

Calling the first one a preview and deferring this record. Cheap, and it removes
every argument above by removing the word. It costs the same argument later with a
version number already published, and the second time it is had against whoever
has already installed the thing.

## What would reverse this

A client begins consuming the interface before the first tag. Entry 4's condition
for freezing has then happened ahead of this record, the release is a promise to a
consumer rather than a note to a reader, and this record is superseded by one that
says what the release promises that client.

Entry 6 is reopened and a language registry is chosen. What is published is then a
package rather than a file with a checksum beside it, the withdrawal properties are
different, and the contents section above is superseded rather than amended.

#92's probe cannot be run by an operator on a platform family the release claims,
because it needs something their machine does not have. The release then contains
a library for a triple and nothing runnable on it, which is the case this record's
first section rules out, and it is superseded by one that says what a release
means for that family.

Two of the conditions in the bar turn out to be uncheckable in practice, so that
somebody judges them on the day anyway. One is a condition written badly. Two is
this record's method failing, and it is replaced by one built from what the checks
can actually answer.
