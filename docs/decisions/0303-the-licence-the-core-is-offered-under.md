# 0303. The licence the core is offered under, and the dependency set that follows from it

Date: 2026-09-05

Status: accepted. Supersedes 0103. Superseded by nothing.

Issue: #303

## The decision

The core is offered under `MIT OR Apache-2.0`, at the recipient's choice,
superseding the `AGPL-3.0-or-later` answer this board recorded on 2026-08-24;
and because the outbound licence is the premise
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) derives its licence
set from, that record is superseded here rather than edited, with its worth test,
its outright refusals, its clause for a standing requirement, its test-tree split
and its removal rule carried forward unchanged and its licence set reversed in
direction.

## The two answers, and which one stands

Entry 1 of #1 was answered twice on that issue. The first, read out of the issue
that holds it:

    gh api repos/Flowfin/core/issues/1/comments \
      --jq '.[] | select(.created_at | startswith("2026-08-24")) | .body' | sed -n '3p'
    Entry 1, recorded rather than newly decided: the fleet-wide licence answer covers this board - AGPL-3.0-or-later, the variant settled today on Flowfin/hub#1.

The second, and the one that stands:

    gh api repos/Flowfin/core/issues/1/comments \
      --jq '.[] | select(.created_at | startswith("2026-09-04T21:44")) | .body' | head -1
    The answer of 2026-09-04 stands: MIT OR Apache-2.0. Entry 1 was answered twice because the second reading revisited it with the reason the first did not carry, that a core whose purpose is to be linked by clients this organisation does not write cannot be copyleft in any strength; the 2026-08-24 entry is superseded by that reading, and the record in docs/decisions/ says so in one sentence rather than leaving two answers on this issue for the next reader.

The reason given for the second is the sentence quoted inside it: a core whose
whole purpose is to be linked by clients this organisation does not write cannot
be copyleft in any strength. The first answer carried no reason of its own beyond
the fleet-wide variant it adopted, which is what made it revisitable without
anything new having happened.

The pair rather than either member alone is the second half of that answer, in
the same comment's own words: `MIT`'s four paragraphs for a recipient who wants
nothing else, `Apache-2.0` for the express patent grant and the defensive
termination, which matter when the subject is formats and protocols that vendors
consider theirs. A recipient picks either.

## What a licence already granted keeps

Every commit on the default branch before this one was published under
`AGPL-3.0-or-later`, and anybody who received a copy under those terms keeps them
for that copy. A licence is a grant to a recipient rather than a property of a
repository, so what changes here are the terms the core is offered on from this
commit, and nothing already given is revoked. What made the change possible at
all is that there is one author to renegotiate with, which #1 read rather than
assumed:

    git log --format='%aN' origin/main | sort -u
    Nils Lehnen
    gh api repos/Flowfin/core/contributors --jq '.[] | "\(.login) \(.contributions)"'
    iderex 211

## The licence set

Named explicitly, because a general principle is a thing two readers apply
differently. That sentence is
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s and it is why this
record enumerates rather than states a test.

Admitted anywhere in the graph, shipping tree or test tree: `MIT`, `Apache-2.0`,
`BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `Unlicense`, `CC0-1.0`,
`Unicode-3.0`. A dual offer that includes one of these is admitted on that
member, which is how the two most common offers in this ecosystem arrive.

Refused, and this is the half that moves: `MPL-2.0`, `LGPL-2.1-or-later`,
`LGPL-3.0-or-later`, `GPL-2.0-only`, `GPL-2.0-or-later`, `GPL-3.0-or-later`,
`AGPL-3.0-or-later`. Every strength of copyleft is in the refused half now, and
six of those seven were in the admitted half under the answer this record
supersedes.

Refused for a second reason, which is that they are not free software licences at
all whatever their terms say about source, and which the direction does not
touch: `SSPL-1.0`, `BUSL-1.1`, `Elastic-2.0`, anything carrying the Commons
Clause, and every licence with a non-commercial or field-of-use restriction.

Refused, finally, and this is the one that arrives most often, also untouched: a
package with no licence statement at all, which is not permissive by default but
reserved by default.

## Why the set is shaped this way, which is the sentence that reversed

The superseded record derived its set from one-way compatibility in the copyleft
direction, in its own words:

    git show origin/main:docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md | sed -n '84,88p'
    Why the set is shaped this way. The core is distributed under
    AGPL-3.0-or-later, so the one-way compatibility is the direction that matters: a
    permissive or weak-copyleft node can be carried inside this work, and a node whose
    own terms forbid the conditions AGPL-3.0-or-later imposes cannot, whichever way
    round it is more convenient.

Under `MIT OR Apache-2.0` the direction runs the other way and the admitted half
shrinks. A work offered on those terms grants a recipient the choice of either,
so what may be carried inside it is what everybody conveying it can satisfy under
either member. That admits obligations of a shape both members already carry -
carrying a notice, preserving attribution, disclaiming warranty, a bar on using a
name to promote - and it refuses an obligation that reaches the work the material
is combined with, because neither member imposes one and a client author reading
`MIT OR Apache-2.0` on this core has not agreed to one.

Two grounds do that refusing and they are worth keeping apart, because only the
first is a compatibility fact.

The strong and weak copyleft family - `GPL`, `AGPL`, `LGPL` in every variant -
imposes conditions on the combined work: the whole conveyed work under the same
terms, or, for `LGPL`, the relinking condition over a client binary. A client
that links this core cannot meet either while offering their own client on their
own terms, and eleven clients this organisation does not write is exactly the
population that would have to. Refused because the terms cannot be met inside
what this core is offered as.

`MPL-2.0` is refused on the second ground, which is a choice rather than a
compatibility fact, so it is written as one. Its copyleft is per file and its own
third section permits a Larger Work under other terms, so a client could convey a
binary containing `MPL-2.0` files while offering their client permissively. What
travels with those files is a source-availability obligation on them, owed by
whoever conveys the binary. That obligation is invisible in `MIT OR Apache-2.0`,
which is the whole of what a client author reads before linking, and it would
arrive in eleven repositories that never read a record saying it was there. The
multiplier is the argument, and it is the same one the superseded record opens
with: every dependency this core takes is taken eleven times.

`GPL-2.0-only` is no longer the term the set names specially. The superseded
record named it because it was the one widely used free licence in its refused
half and the `-only` was the whole of the difference. With the whole copyleft
column refused, the `-only` distinguishes nothing here, and the term sits in the
refused half beside its `-or-later` sibling rather than ahead of it.

What this set does not settle, carried forward unchanged: a node whose licence is
stated one way in its manifest and another way in its own tree. The manifest is
what a tool reads and the tree is what a court reads. Where the two disagree the
node is refused until they agree, because a licence nobody can state is the same
problem as none.

## 0268 re-read against the premise that moved

[0268](0268-a-conjunctive-licence-expression.md) decides two things, and only one
of the two grounds it gives for the second rests on the premise this record
moves.

Its conjunction rule stands untouched, and it is restated here as part of the set
above rather than left for a reader to combine: a conjunction is admitted only
where every member is admitted, one member in the refused half refuses the whole
expression, and a member the set names in neither half leaves the expression
UNDECIDED, which is not admitted. A dual offer inside a conjunction is read on
its admitted member. Nothing in that rule depends on which licence the core is
offered under.

`Unicode-3.0` stays in the admitted half, and the ground for it is now the second
of that record's two rather than the first. The first mapped the term's three
conditions onto the supplementary terms `AGPL-3.0-or-later` enumerates as ones
that may supplement it, quoting this repository's own licence file to do so. That
file is not what the core is offered under any more, so the mapping is a reading
of a licence this work no longer carries and it decides nothing here.

The second ground survives the move intact, because it is about the admitted half
rather than about the outbound licence. The term's three conditions are a notice
carried with the copies or in the documentation, a warranty and liability
disclaimer, and a bar on using a copyright holder's name to promote. The first
two are `MIT`'s and the third is `BSD-3-Clause`'s third clause, both of which are
in the admitted half above, so admitting `Unicode-3.0` adds a fourth notice to
carry and no obligation class a client author or an operator was not already
meeting. There is no source-availability condition in it, no field-of-use
restriction, no non-commercial restriction, and no term reaching a work the
material is combined with, which is the test this record states.

So `unicode-ident` stays admitted, and the collision
[0243](0243-the-means-a-certificate-is-validated-with.md) left open by name stays
closed, on a ground read out of the licence the core is offered under today.

## What a dependency has to be worth

Carried forward from the superseded record, because none of it rests on the
outbound licence.

A dependency enters this core only where writing the equivalent here would cost
more than carrying somebody else's release cadence, security response and licence
for as long as the core lives. Every dependency this core takes is taken eleven
times, because a client embeds the core and embeds its graph, and that multiplier
is what makes the ordinary test insufficient: a dependency that saves an afternoon
here is a binary-size negotiation on a television, a store review somebody else
sits through, and a licence obligation in a repository this one never sees.

Carrying it means four things at once. Its release cadence, because every release
is a change the gate has to run against. Its security response, because the core
inherits whatever answer the dependency gives to an advisory, including silence.
Its licence, in eleven client repositories rather than in this one. Its reach,
because a dependency that pulls its own graph is not one dependency, and the
count that matters is the transitive one, read rather than assumed.

Against that, what writing it here costs: the code, the tests, the review, and
the same security response for the core's own account. Where the thing being
replaced is small, well specified, and already has a test somebody would write
anyway, the answer is usually to write it. Where it is a protocol, a parser of
somebody else's format, or a cryptographic primitive, the answer is usually not,
because a wrong implementation of any of the three is a defect nobody sees until
it is exploited.

## What is refused outright, whatever it is worth

Five behaviours, each of which overturns a decision this board took somewhere
else, so that admitting the dependency would move a decision without a record.
Carried forward, with
[0243](0243-the-means-a-certificate-is-validated-with.md)'s narrowing of the
fourth written into it rather than pointed at, so that a reader of this record
reads the rule in force.

A dependency that reaches the network on its own.
[0069](0069-every-host-the-core-may-contact.md) decides every host the core may
contact and #70 is the test that fails when it reaches one nobody configured. A
node with its own client, its own updater, or its own exporter defeats both, and
the defeat is invisible in a diff.

A dependency that reads or writes the filesystem without being told where.
[0040](0040-the-cache-store-interface.md) puts the storage location in the
client's hands, and a node that picks its own cache directory has taken that
decision away from eleven client authors who each had a reason.

A dependency that starts a thread the core did not ask for.
[0009](0009-the-concurrency-model.md) says two lanes and nothing else, no work
posted to a shared pool and no timer thread of its own. A node with a background
worker is a third lane the core does not own, cannot size and cannot stop, and
#115's stop call then returns while something is still running.

A dependency that writes to a log.
[0100](0100-the-diagnostics-interface.md) and
[0071](0071-what-may-leave-through-a-diagnostic-event.md) decide what leaves the
core and in what shape, and a node writing to a global logger is a second exit
for exactly the values 0071 classifies field by field. Narrowed by
[0243](0243-the-means-a-certificate-is-validated-with.md): what is refused is
writing to a log, and linking a logging facade whose sink is absent is not that,
on the condition that the core installs no logger and states that it installs
none. The reasoning behind that narrowing and what it costs are in that record.

A dependency that carries its own field-bearing surface, which
[0061](0061-the-span-facility.md) supplied by refusing a real candidate rather
than by arguing from a principle. A tracing library's spans carry attributes, so
#71's redaction rule would have to cover a second facility rather than one. The
cost has nothing to do with what the dependency does on its own, and it is the
ground least likely to be noticed at the moment of taking one.

## A dependency a record already standing requires

Carried forward unchanged, because it is a rule about worth rather than about
licences.

[0041](0041-how-a-cache-key-is-built.md) requires a cryptographic digest,
[0105](0105-an-entry-this-version-did-not-write.md) rests on the same
requirement, and [0011](0011-the-language-the-toolchain-and-the-binding-layer.md)
measures that the toolchain offers none. So a landed record already needs a
dependency, and a rule written without that case would refuse the digest under a
clause nobody wrote for it.

The clause. Where a record that has already landed states a requirement the means
cannot meet, a dependency meeting exactly that requirement is admitted on the
record's authority rather than on this one's, provided it is the smallest thing
that meets it, its licence is in the set above, and none of the five outright
refusals applies to it. What this record contributes in that case is the licence
set and the five behaviours, not the judgement of worth, because the worth was
already decided by the record that stated the requirement.

The bound on that clause. It admits a dependency for the requirement the record
states and nothing else in the same package. A package offering a digest and a
transport is admitted for the digest only if the transport is separable; where it
is not, the transport is a second dependency and is judged on its own terms.

## The test tree and the shipping tree

Carried forward unchanged. They carry different risk and one rule for both is
either too strict for the suite or too loose for what an operator installs.

In the shipping tree, everything above applies in full.

In the test tree, the worth test is relaxed and nothing else is. A test-only
dependency is not distributed, so its size on a television, its release cadence
in eleven client repositories and its behaviour on an operator's machine are not
costs anybody pays. What still applies without relaxation: the licence set,
because a licence obligation attaches to what is distributed and a test fixture
can end up distributed by accident; and the five behaviours, because a node that
starts a thread or reaches the network inside the suite is a node that makes the
suite's verdict depend on something outside the run. A flaky gate is the specific
failure, and it is worse than a slow one because it teaches people that red means
nothing.

What no relaxation reaches: a dependency that is in the test tree today and the
shipping tree tomorrow is judged as a shipping dependency on the day it moves,
not grandfathered by having been there.

## How one leaves

Carried forward unchanged. A dependency with no stated removal condition is
permanent, so one is written when it enters, beside the clause that admitted it.

The condition is written so a reader can look at the world and say whether it has
happened. Three shapes cover almost every case. The means grows the facility:
[0011](0011-the-language-the-toolchain-and-the-binding-layer.md) already names
this for the source of unpredictable bytes, where a stable compiler gaining one
retires the dependency and the client seam together. The requirement disappears:
a record is superseded and what it required is no longer required. The cost
turns: the dependency's transitive count, its size on the smallest target, or its
advisory history crosses a number that was written down when it entered.

"When we no longer need it" is not such a condition and is refused as one.

## Where the line lives

Every dependency carries, beside its entry in the manifest, one line naming the
clause of this record that admitted it and one naming what would retire it. The
manifest rather than a separate document, because a separate list is a thing that
drifts from the graph it describes, and the drift is invisible until somebody
audits it.

#87 produces the bill of materials, which is where the graph is read back, and
#19 is what refuses a graph that does not match the committed lockfile or that
carries a known advisory. Neither reads this record, and nothing in this
repository refuses a dependency admitted by no clause today. The rule is carried
by the review and by the line beside the entry until something reads it.

The two entries standing in the manifest cite the superseded record by number and
they are not rewritten here. Every statement they make about a licence is still
true - both members of `MIT OR Apache-2.0` are in the admitted half above, and
`ureq-proto`'s graph reaches no conjunction - so what is stale in them is the
number and not the judgement. That is a residual this change leaves rather than
one it repairs, and it is written here so a reader meeting `0103` beside a
manifest entry knows which record the clause now lives in.

## The graph as it stands, read against this set

Nothing in the resolved graph is admitted only by the clause that moves. Read at
the head of the branch this record lands on:

    cargo metadata --format-version 1 --locked \
      | jq -r '.packages[] | select(.name != "flowfin-core") | "\(.name) \(.version) \(.license)"' \
      | sort
    base64 0.23.1 MIT OR Apache-2.0
    block-buffer 0.12.1 MIT OR Apache-2.0
    bytes 1.12.1 MIT
    cfg-if 1.0.4 MIT OR Apache-2.0
    cpufeatures 0.3.0 MIT OR Apache-2.0
    crypto-common 0.2.2 MIT OR Apache-2.0
    digest 0.11.3 MIT OR Apache-2.0
    http 1.5.0 MIT OR Apache-2.0
    httparse 1.10.1 MIT OR Apache-2.0
    hybrid-array 0.4.14 MIT OR Apache-2.0
    itoa 1.0.18 MIT OR Apache-2.0
    libc 0.2.189 MIT OR Apache-2.0
    log 0.4.34 MIT OR Apache-2.0
    sha2 0.11.0 MIT OR Apache-2.0
    typenum 1.20.1 MIT OR Apache-2.0
    ureq-proto 0.6.1 MIT OR Apache-2.0

Sixteen packages, every one of them a single admitted term or a dual offer both
of whose members are admitted. The shrinking set costs this graph nothing.

That is a reading of one day's graph and not a promise about the next entry. What
changes here is which entries can arrive at all, and the population now refused
is a real one in this ecosystem rather than a hypothetical: an `MPL-2.0` node is
the shape most likely to be proposed and refused under the set above.

## What this does not decide

Where the assembled notices for a release live. That is #74's and #87's question,
and the superseded record did not answer it either.

Whether anything in this repository reads a licence expression. Nothing does. The
set above is applied by the review and by the line beside a manifest entry, which
is what the superseded record already says of itself.

What the hosting provider's own listing reports for a repository offering two
licences. That listing is a state on the provider rather than a byte in this
tree, and what the tree carries is the two texts at the root and the expression
in the manifest.

## Why this is written down before the code

An outbound licence that two artefacts state differently is the cheapest kind of
wrong and the most expensive to leave standing. It is the first thing a client
author checks before writing a line against this core, and until this change the
answer they found was the one that had been superseded on the issue that took it:
four statements of one fact, in the manifest, the readme, the licence file and
the provider's listing, all of them the earlier answer.

The dependency set is the second half and it cannot lag behind the first. A rule
whose stated ground is a licence the work is no longer offered under is a rule
two readers apply differently, and the direction it gets wrong is the permissive
one: a reader applying the superseded set admits a copyleft node, the node lands,
and a licence obligation is the one class of mistake here that deleting the
dependency later does not repair. Taking it now costs nothing in the graph, which
the reading above measures; taking it after a copyleft node has landed costs that
node and everything resting on it.

## Alternatives, and what each cost

**Keep `AGPL-3.0-or-later` and close the question the other way.** Costs nothing
to execute, since the tree already publishes it, and it is what a reader who
never opened #1 would assume. What it costs is the plan: eleven clients this
organisation does not write cannot link an `AGPL-3.0-or-later` core without
becoming that themselves, which is the reason the second answer gives and the one
the first did not carry.

**`MIT` alone.** Four paragraphs, the shortest thing anybody has to read, and the
most widely understood. It carries no express patent grant, so a contributor
keeps whatever patent claim they hold over what they contributed, and the subject
here is formats and protocols vendors consider theirs. The pair costs one more
file and buys that grant.

**`Apache-2.0` alone.** The patent grant and the defensive termination without
the second file. It cannot be combined with `GPL-2.0-only` code, and it is the
longer text a small client author reads. Offering the pair leaves that choice to
the recipient instead of taking it for them.

**`MPL-2.0`, the middle option #1 lists.** Per-file copyleft, so changes to the
core's own files come back and a client linking it does not become `MPL-2.0`. It
costs a per-file header convention, and it puts a source-availability obligation
on eleven client repositories that a permissive expression does not - which is
the same obligation the set above refuses to accept from a dependency, and
accepting it outbound while refusing it inbound is a position that cannot be
argued.

**Supersede only the licence and edit
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s set in place.**
The smallest diff by a wide margin. [0001](0001-decision-records.md) forbids it:
a change to what a record decided is a new record, and the licence set is what
that record decided. Editing it would also erase the reasoning that was available
when it was written, which is what a decision record is kept for.

**Write the licence record and leave
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) standing with a
pointer.** Cheaper than a full successor, and it keeps the worth test, the
behaviours and the clause in one place rather than restating them.
[0267](0267-a-record-that-narrows-one-clause-of-another.md)'s field is for a
clause narrowed while the rest stands, and this is a premise replaced: the
sentence that derives the set is false under the answer that stands, and a reader
who stops at the header of a record whose central enumeration has reversed reads
the wrong set. The cost paid instead is length, which is this record restating
what did not move.

## What would reverse this

A client, or an organisation adopting one, is unable to ship because of a term in
either member of the pair. That is the pair failing at the only thing it was
chosen for, and what replaces it is a record naming the term and the offer that
avoids it.

The core stops being something a client links, so that neither the outbound
permissive answer nor the eleven-times multiplier behind the dependency set
holds. Both halves rest on that shape, and without it the rule is stricter than
its reason.

The licence set refuses a dependency that a landed record requires and nothing
admitted by the set can meet. Then the set and the record are in conflict, and
which of the two moves is a decision above both rather than an exception written
into either.

A dependency admitted under the clause for a standing requirement is measured to
carry more than the requirement, twice. One case is a package judged wrongly; two
is the clause being wider than it reads, and it is replaced by one naming what a
package may contain beside the thing it was taken for.

Something in this repository begins to read the line beside a manifest entry, for
example under #87 or #19. This record is then superseded by one describing what
is refused rather than what is expected, because a rule nothing refuses is a
suggestion and this record says so of itself.
