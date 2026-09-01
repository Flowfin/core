# 0268. A conjunctive licence expression, and the term the set does not name

Date: 2026-09-01

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #268

## The decision

A conjunctive licence expression is admitted only where every member of it is
admitted, so a conjunction carrying a term
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s set names in
neither half is not admitted until the set names it; `Unicode-3.0` is named
admitted here, on the ground that each of its three conditions is a term
`AGPL-3.0-or-later` enumerates as one that may supplement it, so it passes that
record's own test for the refused half and adds no obligation class the admitted
half does not already carry; and what a package reaches decides nothing, because
that record already applies the licence set without relaxation to a test tree
that is distributed to nobody.

## What was in the way, and it was not whether the licence is acceptable

The set is an enumeration with an admitted half and a refused half, and a reader
holding an expression is expected to place it in one of the two. That works for
the shapes the set was written for, which its own sentence names - a single term,
and a dual offer that includes an admitted member:

    git show origin/main:docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md | sed -n '67,71p'
    Admitted anywhere in the graph, shipping tree or test tree: `MIT`, `Apache-2.0`,
    `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `Unlicense`, `CC0-1.0`, `MPL-2.0`,
    `LGPL-2.1-or-later`, `LGPL-3.0-or-later`, `GPL-2.0-or-later`, `GPL-3.0-or-later`,
    `AGPL-3.0-or-later`. A dual offer that includes one of these is admitted on that
    member, which is how the two most common offers in this ecosystem arrive.

A conjunction is the third shape and the set says nothing about it. That did not
matter while every conjunction in front of the board had all its members inside
the admitted half, which is what
[0243](0243-the-means-a-certificate-is-validated-with.md) found for the two it
could place and recorded as the reason they passed. The one it could not place is
`unicode-ident`, offered as `(MIT OR Apache-2.0) AND Unicode-3.0`, and the term
the set names in neither half is still unnamed at the commit this record is
written against:

    git rev-parse origin/main
    2ed77e65c0c2b144dd3ef4befb7c63aa23c6ac31

    git show origin/main:docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md | grep -c 'Unicode-3.0'
    0

So there were two questions and only one of them is about a licence. The first is
what an enumerated set does with an expression it has no rule for, which is a
question about the set and would be the same for any term. The second is whether
this particular term belongs in the admitted half. Answering the second alone
would have left the next unnamed term in exactly the state this one was in.

## The conjunction rule, stated in full

A conjunction is satisfied only if every member of it is satisfied, so a
conjunction is admitted only where every member is in the admitted half. One
member in the refused half refuses the whole expression, and the other members
being admitted changes nothing, because a conjunction offers no choice between
them. That is the direction a reader gets right without being told.

The direction that needed writing down is the third state. A member the set names
in neither half leaves the expression UNDECIDED, and undecided is not admitted.
The enumeration is therefore total in the only way an enumeration can be: named
in the admitted half is admitted, named in the refused half is refused, and
unnamed is a question nobody has answered rather than a permission nobody wrote
out. Reading an unnamed term as admitted would make the refused half do all the
work and the admitted half decorative, which is the opposite of what
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) says the set is for -
named explicitly, because a general principle is a thing two readers apply
differently.

What that costs is one dependency stopped per unnamed term until somebody writes
a record. That is the cost being chosen rather than an oversight: the alternative
is a graph admitted by silence, and a licence obligation is the one class of
mistake in this repository that deleting the dependency later does not repair.

A dual offer INSIDE a conjunction is read as
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) already reads a dual
offer, on the member that is admitted. `(MIT OR Apache-2.0) AND Unicode-3.0` is
therefore two members: the offer, which is admitted on either of its two, and
`Unicode-3.0`, which is what the rest of this record is about.

## Why what a package reaches decides nothing here

The issue names a third answer: admit the package because it reaches no client
binary, on the ground that the distinction is real. It is refused, and the
argument against it is
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s own rather than a
new one.

That record relaxes the worth test for the test tree and states in the same
breath what it does not relax, and the reason it gives is about distribution:

    git show origin/main:docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md | sed -n '172,175p'
    eleven client repositories and its behaviour on an operator's machine are not
    costs anybody pays. What still applies without relaxation: the licence set, because
    a licence obligation attaches to what is distributed and a test fixture can end up
    distributed by accident; and the four behaviours, because a node that starts a

A test-tree dependency ships to nobody and the licence set still reaches it in
full. A build-time dependency of the shipping tree is in the lockfile every client
resolves, is fetched by every build, and lands in the vendor directory of any
client that vendors its graph. So a reach test admitting the second would be
looser than the rule already in force for the first, which ships less far. There
is no reading of that record under which the ordering comes out the other way.

The distinction the issue calls real is real, and it is a fact about where an
obligation attaches rather than about whether one exists. `unicode-ident` says so
in its own words - the terms are conjunctive because the artefact is two bodies
of property in one crate:

    gh api repos/dtolnay/unicode-ident/contents/README.md --jq '.content' | base64 -d | sed -n '266,270p'
    The **generated** files incorporate tabular data derived from the Unicode
    Character Database, together with intellectual property from the original source
    code content of the crate. One must comply with the terms of both the Unicode
    License Agreement and either of the Apache license or MIT license when those
    generated files are involved.

Compliance with an attribution term is owed by whoever conveys the material, and
eleven clients conveying a graph is exactly the multiplier
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) opens with. A rule
that switched the licence set off for a build-time node would put that obligation
on eleven repositories that never read a record saying it was there.

## Unicode-3.0 read rather than assumed

The expression is read from the package's own manifest rather than from a
resolver's summary of it, which is a second route to the fact
[0243](0243-the-means-a-certificate-is-validated-with.md) took with
`cargo metadata`:

    gh api repos/dtolnay/unicode-ident/contents/Cargo.toml --jq '.content' | base64 -d | grep '^license'
    license = "(MIT OR Apache-2.0) AND Unicode-3.0"

The identifier is current rather than deprecated, and it is approved by the body
this ecosystem's manifests are keyed to:

    gh api repos/spdx/license-list-data/contents/json/details/Unicode-3.0.json --jq '.content' \
      | base64 -d | jq -c '{licenseId, name, isOsiApproved, isDeprecatedLicenseId}'
    {"licenseId":"Unicode-3.0","name":"Unicode License v3","isOsiApproved":true,"isDeprecatedLicenseId":false}

Approval by a body is not the test
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) states, so it is
recorded as context and the terms are read. The grant and its condition:

    gh api repos/spdx/license-list-data/contents/text/Unicode-3.0.txt --jq '.content' | base64 -d | sed -n '13,22p'
    Permission is hereby granted, free of charge, to any person obtaining a
    copy of data files and any associated documentation (the "Data Files") or
    software and any associated documentation (the "Software") to deal in the
    Data Files or Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, and/or sell
    copies of the Data Files or Software, and to permit persons to whom the
    Data Files or Software are furnished to do so, provided that either (a)
    this copyright and permission notice appear with all copies of the Data
    Files or Software, or (b) this copyright and permission notice appear in
    associated Documentation.

and the last of the three things it asks:

    gh api repos/spdx/license-list-data/contents/text/Unicode-3.0.txt --jq '.content' | base64 -d | sed -n '36,39p'
    Except as contained in this notice, the name of a copyright holder shall
    not be used in advertising or otherwise to promote the sale, use or other
    dealings in these Data Files or Software without prior written
    authorization of the copyright holder.

Three conditions and nothing else: a notice carried with the copies or in the
documentation, a warranty and liability disclaimer, and a bar on using a
copyright holder's name to promote. There is no source-availability condition, no
field-of-use restriction, no non-commercial restriction, and no term reaching a
work the material is combined with.

THE TEST IS THE ONE 0103 WROTE, AND IT IS ANSWERED OUT OF THIS REPOSITORY'S OWN
LICENCE FILE. That record refuses every licence whose terms cannot be satisfied
inside a work distributed under `AGPL-3.0-or-later`, and that licence enumerates
the supplementary terms it tolerates:

    git show origin/main:LICENSE | sed -n '349,365p'
      Notwithstanding any other provision of this License, for material you
    add to a covered work, you may (if authorized by the copyright holders of
    that material) supplement the terms of this License with terms:

        a) Disclaiming warranty or limiting liability differently from the
        terms of sections 15 and 16 of this License; or

        b) Requiring preservation of specified reasonable legal notices or
        author attributions in that material or in the Appropriate Legal
        Notices displayed by works containing it; or

        c) Prohibiting misrepresentation of the origin of that material, or
        requiring that modified versions of such material be marked in
        reasonable ways as different from the original version; or

        d) Limiting the use for publicity purposes of names of licensors or
        authors of the material; or

Each of the three lands on one of those. The disclaimer is (a), the notice
condition is (b), and the bar on promotional use of a name is (d). So the terms
are satisfiable inside the work this core is distributed as, by the enumeration
that work's own licence carries, rather than by anybody's judgement about whether
they read as permissive.

The second half of the ground is that the admitted set already carries both
shapes, so no obligation class arrives with this term. The notice condition is
`MIT`'s:

    gh api repos/spdx/license-list-data/contents/text/MIT.txt --jq '.content' | base64 -d | sed -n '11,12p'
    The above copyright notice and this permission notice shall be included in all copies or substantial
    portions of the Software.

and the name condition is `BSD-3-Clause`'s third clause:

    gh api repos/spdx/license-list-data/contents/text/BSD-3-Clause.txt --jq '.content' | base64 -d | sed -n '9p'
    3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

Both are in the admitted half already. Admitting `Unicode-3.0` therefore adds a
fourth notice to carry and no obligation an operator or a client author was not
already meeting.

## What this decides about unicode-ident, and where it stops

`unicode-ident` is admitted, by the rule above rather than by a reading of the
graph. Its expression is a conjunction of two members; the dual offer is admitted
on `MIT` or on `Apache-2.0`, and `Unicode-3.0` is admitted by the section above.
Every member is in the admitted half, so the conjunction is.

That closes the collision
[0243](0243-the-means-a-certificate-is-validated-with.md) left open by name. That
record says of the tenth expression it read that it neither admits nor refuses
it, and that until this question is answered the Android half of the means it
decides is a dependency this board has not licensed. It is licensed now, and that
record's reversal condition for it does not fire, because that condition names
the refusal rather than the admission:

    git show origin/main:docs/decisions/0243-the-means-a-certificate-is-validated-with.md | grep -n 'Android licence term is refused'
    537:The Android licence term is refused under #268. The two Android triples then

Where this stops. Nothing in this repository reads a licence expression, so the
rule above is carried by the review and by whoever writes the line beside a
manifest entry, which is what
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) already says of
itself. And the package is not in this tree's graph today, so nothing here has to
carry a notice yet:

    git grep -n 'unicode-ident' origin/main -- Cargo.lock Cargo.toml ; echo "exit=$?"
    exit=1

It arrives with the manifest entry
[0243](0243-the-means-a-certificate-is-validated-with.md) leaves to #27 and #29,
and the notice this record admits is one an operator has to be able to find.
Where the assembled notices for a release live is #74's and #87's question, and
this record does not answer it.

## What this record does to 0103, and what it does not

It adds a term to the admitted half and a rule for a shape the set had none for.
Both are changes to what that record decided, so neither is written into it:
[0001](0001-decision-records.md) permits three edits to a landed record and a
change to what it decided is not among them.

NO POINTER IS ADDED TO 0103 EITHER, AND THAT IS A NARROWER CLAIM THAN THE ONE
ABOVE. The pointer edit is permitted for a later record that goes further on a
case the earlier one ALREADY NAMES.
[0243](0243-the-means-a-certificate-is-validated-with.md) took it on the fourth
refused behaviour, which
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) names in its own
words. A conjunction is not a case that record names at all - it names a single
term and a dual offer - so an edit pointing here would be adding the case and the
route in one move, which is the abuse
[0001](0001-decision-records.md) names when it gives the test for that edit.

THE RESIDUAL IS THE SAME ONE 0243 STATED AND IT IS NOW LARGER RATHER THAN
REPAIRED. A reader who opens
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) reads a licence set
that is one term short and a rule covering two of the three shapes an expression
takes, and nothing routes them here. #267 is where the shape of a partial
supersession is asked for, and this record does not invent one, for the reason
that record already gave: superseding
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) entirely would
discard a worth test, four behaviours and a clause that are all unchanged.

## Why this is written down before the code

The graph this decides on is not in the tree - the exit code above says so - which
is the only moment the question can be answered as a rule instead of at a manifest
entry somebody is trying to land.

The specific failure is narrower than that and this board came within one merge of
it. A reader meeting `(MIT OR Apache-2.0) AND Unicode-3.0` with no rule for a
conjunction has three ways out and two of them are wrong quietly. Reading the dual
offer and stopping admits the package on a term nobody examined. Reading the
conjunction as a choice admits it on `MIT`. Both produce a green manifest line
citing a clause of
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) that does not reach
the expression, nothing in this repository reads that line, and the first reader
to notice is whoever assembles the notices for a release under #74 or #87 - after
eleven clients have shipped the graph.

Written afterwards, the rule also arrives with a package already resting on it,
which is the shape
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s own opening
argument is against: a rule about dependencies written after the first ten exist
is a rule with ten exceptions in it.

## Alternatives, and what each cost

**Refuse the term.** The Android triples then carry a package this board may not
carry, and what moves is the means, the licence set, or the triples themselves,
which is
[0243](0243-the-means-a-certificate-is-validated-with.md)'s own reversal
condition and would supersede that record. It costs the widest single entry in
the target set, or a certificate means chosen twice, and it buys nothing an
examination of the terms supports: the three conditions land inside the
enumeration this core's own licence carries. Refusing a licence whose terms the
work tolerates protects nobody, and the cost falls on the platform with the most
people on it.

**Admit it because the package reaches no client binary.** Refused above on
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s own reasoning
about the test tree. Its further cost is that it decides one package and leaves a
rule that has to be applied by measuring a graph per triple, which is the reading
this issue exists because somebody would otherwise have to make at a manifest.

**Admit any conjunction whose members are individually not refused.** The
cheapest to apply and it needs no record per term. It converts the enumeration
into a deny-list, which is a different decision from the one
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) took, and it would
have admitted this term without anybody reading it. The moment it costs something
is the first proprietary or field-of-use term nobody has thought to name yet,
which is exactly the case an enumeration is chosen over a deny-list for.

**Rewrite the set as a rule about terms rather than a list of identifiers.**
Admits every future licence whose obligations are of an admitted shape, and no
record is owed per term.
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) states the cost in
its own words where it explains why the set is named explicitly: a general
principle is a thing two readers apply differently. It is also the option that
cannot be checked by a machine later, and a list of identifiers is the one shape
a resolver's output can be compared against.

**Leave it open and decide it at the manifest entry.** Costs nothing today and is
what happens by default. It moves the decision to whoever is landing #27 or #29,
at the moment they are trying to land something else, which is the most expensive
place to take a licence decision and the one where the cheapest answer is a
clause reference that nothing reads.

## What would reverse this

The Unicode licence changes its terms, or a package in this graph carries a
version of it whose conditions are not the three above. The mapping onto the
supplementary terms this core's licence enumerates is what admitted it, and a
fourth condition is a new question rather than a detail.

A conjunction arrives whose members are each admitted and which is refused
anyway, by a lawyer or by a client author who has to distribute it. That is the
member-by-member rule failing on a composition effect it cannot see, and it is
superseded by a record saying what is read instead of the members.

The count of unnamed terms this rule stops reaches a number nobody is willing to
write a record for. The rule is then costing more than the graph it protects, and
what replaces it is a rule about terms, with the price
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) already names for
that shape paid deliberately.

Something in this repository begins to read a licence expression against the set,
under #87 or #19. This record is then superseded by one written as what a machine
refuses, because the three-state rule above is stated for a reader and a checker
needs the unnamed state to have a name it can print.
