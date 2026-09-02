# 0103. What admits a dependency, and what is refused

Date: 2026-08-24

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrowed-by: 0243, on the fourth behaviour refused outright, a dependency that writes to a log

Issue: #103

## The decision

A dependency enters this core only where writing the equivalent here would cost
more than carrying somebody else's release cadence, security response and licence
for as long as the core lives; its licence is one of the set named below; it does
nothing on its own that a record here decided the core would do deliberately; and
it enters with the clause that admitted it and the evidence that would retire it
written beside it.

## Why the rule is more than the usual one

Every dependency this core takes is taken eleven times, because a client embeds
the core and embeds its graph. That is #103's own opening sentence, and it is what
makes the ordinary test insufficient: a dependency that saves an afternoon here is
a binary-size negotiation on a television, a store review somebody else sits
through, and a licence obligation in a repository this one never sees.

Two of the questions the rule needs answered were open until 2026-08-24 and are
answered now. The licence this core publishes under:

    gh api repos/Flowfin/core --jq '.license.spdx_id'
    AGPL-3.0

and the means, which decides how large a graph the core needs at all. 0011
measures five absences in the standard library, and each is a dependency-shaped
hole this record is the rule for.

## What a dependency has to be worth

The test is not whether it saves work today. It is whether writing the equivalent
here would cost more than carrying it, over the life of the core, where carrying
it means all four of the following at once.

Its release cadence. Every release is a change the gate has to run against, and a
dependency that releases weekly is a weekly interruption whether or not the core
needed anything in it.

Its security response. A dependency is a party this core now depends on to answer
an advisory, and the core inherits whatever answer it gives, including silence.
#19 is what notices an advisory; nothing makes somebody else fix one.

Its licence, in eleven client repositories rather than in this one.

Its reach. A dependency that pulls its own graph is not one dependency. The count
that matters is the transitive one, and it is read rather than assumed.

Against that, what writing it here costs: the code, the tests, the review, and the
same security response for the core's own account. Where the thing being replaced
is small, well specified, and already has a test somebody would write anyway, the
answer is usually to write it. Where it is a protocol, a parser of somebody else's
format, or a cryptographic primitive, the answer is usually not, because a wrong
implementation of any of the three is a defect nobody sees until it is exploited.

## The licence set

Named explicitly, because a general principle is a thing two readers apply
differently.

Admitted anywhere in the graph, shipping tree or test tree: `MIT`, `Apache-2.0`,
`BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `Unlicense`, `CC0-1.0`, `MPL-2.0`,
`LGPL-2.1-or-later`, `LGPL-3.0-or-later`, `GPL-2.0-or-later`, `GPL-3.0-or-later`,
`AGPL-3.0-or-later`. A dual offer that includes one of these is admitted on that
member, which is how the two most common offers in this ecosystem arrive.

Refused: `GPL-2.0-only`, and every other licence whose terms cannot be satisfied
inside a work distributed under AGPL-3.0-or-later. Refused for a second reason,
which is that they are not free software licences at all whatever their terms say
about source: `SSPL-1.0`, `BUSL-1.1`, `Elastic-2.0`, anything carrying the Commons
Clause, and every licence with a non-commercial or field-of-use restriction.
Refused, finally, and this is the one that arrives most often: a package with no
licence statement at all, which is not permissive by default but reserved by
default.

Why the set is shaped this way. The core is distributed under
AGPL-3.0-or-later, so the one-way compatibility is the direction that matters: a
permissive or weak-copyleft node can be carried inside this work, and a node whose
own terms forbid the conditions AGPL-3.0-or-later imposes cannot, whichever way
round it is more convenient. `GPL-2.0-only` is named rather than left to the
reader because it is the one widely used free licence in the refused half, and the
`-only` is the whole of the difference.

What this set does not settle is a node whose licence is stated one way in its
manifest and another way in its own tree. The manifest is what a tool reads and
the tree is what a court reads. Where the two disagree the node is refused until
they agree, because a licence nobody can state is the same problem as none.

## What is refused outright, whatever it is worth

Four behaviours, each of which overturns a decision this board took somewhere
else, so that admitting the dependency would move a decision without a record.

A dependency that reaches the network on its own. 0069 decides every host the core
may contact and #70 is the test that fails when it reaches one nobody configured.
A node with its own client, its own updater, or its own exporter defeats both, and
the defeat is invisible in a diff.

A dependency that reads or writes the filesystem without being told where. 0040
puts the storage location in the client's hands, and a node that picks its own
cache directory has taken that decision away from eleven client authors who each
had a reason.

A dependency that starts a thread the core did not ask for. 0009 says two lanes
and nothing else, no work posted to a shared pool and no timer thread of its own.
A node with a background worker is a third lane the core does not own, cannot size
and cannot stop, and #115's stop call then returns while something is still
running.

A dependency that writes to a log. 0100 and 0071 decide what leaves the core and
in what shape, and a node writing to a global logger is a second exit for exactly
the values 0071 classifies field by field.

0243 goes further on this fourth behaviour and narrows it: what is refused is
writing to a log, and linking a logging facade whose sink is absent is not that,
on the condition that the core installs no logger and states that it installs
none. The reasoning, the readings it rests on and what it costs are in that
record. Nothing else in this one moves.

A fifth ground, which 0061 supplied by refusing a real candidate rather than being
argued from a principle. A dependency that carries its own field-bearing surface
makes a rule in another record reach a second place: a tracing library's spans
carry attributes, so #71's redaction rule would have to cover a second facility
rather than one. The cost has nothing to do with what the dependency does on its
own, and it is the ground least likely to be noticed at the moment of taking one.

The three refusals with a worked case behind them are all quotable. 0061 refuses a
tracing library, and 0112 refuses a cross-platform media framework on size on
every target:

    git grep -l '#103' -- docs/decisions

## A dependency a record already standing requires

The list above is about what a dependency has to be worth and what is refused, and
neither question is the one 0041 asks. That record requires a cryptographic
digest, 0105 rests on the same requirement, and 0011 measures that the toolchain
offers none. So a landed record already needs a dependency, and a rule written
without that case would refuse the digest under a clause nobody wrote for it.

The clause. Where a record that has already landed states a requirement the means
cannot meet, a dependency meeting exactly that requirement is admitted on the
record's authority rather than on this one's, provided it is the smallest thing
that meets it, its licence is in the set above, and none of the five outright
refusals applies to it. What this record contributes in that case is the licence
set and the four behaviours, not the judgement of worth, because the worth was
already decided by the record that stated the requirement.

The bound on that clause. It admits a dependency for the requirement the record
states and nothing else in the same package. A package offering a digest and a
transport is admitted for the digest only if the transport is separable; where it
is not, the transport is a second dependency and is judged on its own terms.

## The test tree and the shipping tree

They carry different risk and one rule for both is either too strict for the suite
or too loose for what an operator installs.

In the shipping tree, everything above applies in full.

In the test tree, the worth test is relaxed and nothing else is. A test-only
dependency is not distributed, so its size on a television, its release cadence in
eleven client repositories and its behaviour on an operator's machine are not
costs anybody pays. What still applies without relaxation: the licence set, because
a licence obligation attaches to what is distributed and a test fixture can end up
distributed by accident; and the four behaviours, because a node that starts a
thread or reaches the network inside the suite is a node that makes the suite's
verdict depend on something outside the run. A flaky gate is the specific failure,
and it is worse than a slow one because it teaches people that red means nothing.

What no relaxation reaches: a dependency that is in the test tree today and the
shipping tree tomorrow is judged as a shipping dependency on the day it moves, not
grandfathered by having been there.

## How one leaves

A dependency with no stated removal condition is permanent, so one is written when
it enters, beside the clause that admitted it.

The condition is written so a reader can look at the world and say whether it has
happened, in the same sense a reversal condition in any record here is. Three
shapes cover almost every case. The means grows the facility: 0011 already names
this for the source of unpredictable bytes, where a stable compiler gaining one
retires the dependency and the client seam together. The requirement disappears: a
record is superseded and what it required is no longer required. The cost turns:
the dependency's transitive count, its size on the smallest target, or its
advisory history crosses a number that was written down when it entered.

"When we no longer need it" is not such a condition and is refused as one.

## Where the line lives

Every dependency carries, beside its entry in the manifest, one line naming the
clause of this record that admitted it and one naming what would retire it. The
manifest rather than a separate document, because a separate list is a thing that
drifts from the graph it describes, and the drift is invisible until somebody
audits it.

#87 produces the bill of materials, which is where the graph is read back, and #19
is what refuses a graph that does not match the committed lockfile or that carries
a known advisory. Neither reads this record, and nothing in this repository refuses
a dependency admitted by no clause today. The rule is carried by the review and by
the line beside the entry until something reads it.

## Why this is written down before the code

There is no code and therefore no graph, which is the only moment this record can
be written honestly. A rule about dependencies written after the first ten exist is
a rule with ten exceptions in it, and each exception is defended by the work
already resting on it.

The specific failure it prevents is narrower than that and is already visible in
this tree. 0011 measures five absences: no source of unpredictable bytes on a
stable build, no cryptographic digest, no transport security, no HTTP, and no
promise about clearing a credential's bytes. Each is a hole that will be met by
somebody at a call site, and a person at a call site takes the first package that
compiles. Five separate answers nobody compared is the outcome this record exists
to replace with one question asked five times.

## Alternatives, and what each cost

No rule, deciding each dependency in its own pull request. The cheapest, and it is
what happens by default. It costs consistency in the direction that matters least
and predictability in the direction that matters most: a contributor cannot tell
before doing the work whether the work will be accepted, so the rule is discovered
at review time, which is the most expensive place to discover anything.

A number instead of a test, a cap on the transitive count. Checkable by a machine,
which is the strongest argument for anything here. It refuses a small graph of
three excellent nodes and admits a large graph of one bad one, and the number
would be chosen without any graph to choose it against.

An allow-list of named packages. The most predictable of all and the easiest to
enforce, and it is the one to revisit once a graph exists. Today it would be a
list of nothing, maintained by whoever is asked, and every addition would be this
same argument with no rule to have it against.

A licence rule only, leaving worth and behaviour to review. It covers the risk that
is hardest to undo, since a licence obligation is not repaired by deleting the
dependency later. It leaves the four behaviours entirely to a reviewer noticing
them, and a background thread inside a package is exactly what a reviewer does not
notice.

## What would reverse this

The licence set refuses a dependency that a landed record requires and nothing
admitted by the set can meet. Then the set and the record are in conflict, and
which of the two moves is a decision above both rather than an exception written
into either.

A dependency admitted under the clause for a standing requirement is measured to
carry more than the requirement, twice. One case is a package judged wrongly; two
is the clause being wider than it reads, and it is replaced by one naming what a
package may contain beside the thing it was taken for.

Something in this repository begins to read the line beside a manifest entry, for
example under #87 or #19. This record is then superseded by one describing what is
refused rather than what is expected, because a rule nothing refuses is a
suggestion and this record says so of itself.

The core stops being distributed as something a client links, so that the graph is
no longer taken eleven times. The multiplier is the whole argument for the strength
of this rule, and without it the rule is stricter than its reason.
