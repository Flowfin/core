# 0267. A record that narrows one clause of another

Date: 2026-09-02

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrows: 0001, on the third permitted edit, the pointer to a later record, which becomes a pair of named fields

Issue: #267

## The decision

A record that narrows one clause of an earlier record while leaving the rest of
it standing carries `Narrows: NNNN, <the clause>` in its header, the record it
narrows carries `Narrowed-by: NNNN, <the clause>` beside its `Status:` line, and
a check refuses either field where the record it names does not exist, where no
clause is named after the number, or where the other record does not name it
back.

## Why the pointer is a field and not a sentence

[0001](0001-decision-records.md) permits a pointer to a later record that goes
further on a case this record already names, and says nothing about where the
pointer goes or what it looks like. The one instance in the tree took the shape
that reading allows, which is a paragraph inside the section it concerns:

    sed -n '120,124p' docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md

A reader who reaches that paragraph learns what moved. The reader this decision is
for is the one who does not reach it - who opens
[0103](0103-what-admits-a-dependency-and-what-is-refused.md), reads a `Status:`
line saying `accepted` with nothing else beside it, and takes the whole record as
the rule in force. The narrowing is at line 120 of a record with more than two
hundred, and following a pointer is only optional when it is possible to miss one.

The header is where every reader lands before the prose. A field there is the
smallest change that makes the pointer unmissable, and it is the reason this
decision is a field rather than a convention about where in a section to put the
paragraph.

## What the field carries, and why the clause is not optional

`Narrowed-by: 0243` alone leaves the reader diffing two records to find which of
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s five grounds moved.
The complaint this record answers is that a reader takes the fourth behaviour one
clause wider than the rule in force, so the clause identity is the payload and the
field is refused without it.

The clause is prose and no rule here reads it for sense. What is refused is its
absence, and whether the sentence names the clause that actually moved is caught
by the review. That bound is printed on every run of the check rather than only
written here.

## Why both directions, and what the second one is for

A pointer only one end carries rots the first time a record is renumbered or
withdrawn, and it rots in silence, which is the same defect class this record
exists to fix one level up. Two fields naming each other make the rot a red gate:
a record deleted, renumbered or written with the pointer on one side only is
refused by name.

That is also what makes the backward field worth its cost.
[0243](0243-the-means-a-certificate-is-validated-with.md) already argues the
narrowing at length in its own prose, so `Narrows:` adds no argument to it and
tells its reader nothing they could not find. What it does is give the check
something to compare, and a rule that can only be evaluated from one side is a
rule that goes stale from the other.

## What this asks of 0001, stated rather than assumed

[0001](0001-decision-records.md)'s third permitted edit is a pointer to a later
record where the pointer changes no sentence's meaning and adds no argument. This
record changes that clause in two ways and both are written here so a reader can
argue with them.

The pointer takes a fixed form where the later record narrows rather than merely
goes further. A paragraph is no longer one of the shapes it may take in that case.

The pointer becomes a pair, so the narrowing record receives a field naming an
*earlier* record. Clause three as written permits a pointer to a later record and
says nothing about the other direction, and `Narrows:` is the other direction.
Both halves pass clause three's own test - remove either field and both records
say exactly what they said before - which is why this is a narrowing of that
clause rather than a supersession of the record carrying it.

Where the field goes is not something [0001](0001-decision-records.md) fixes and
this record does not read it as though it did. Its header rule says which three
lines a record must carry and in what order; it does not say that a record carries
nothing else, any more than its heading rule does, which says in so many words
that a record's own headings may sit between the first two. A reader who takes the
three-line rule for an exhaustive list should argue with that reading here rather
than discover it.

## A pointer in the tree that is already an argument

[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s paragraph does not
only point. It restates the narrowed rule and the condition it rests on, so
removing it would take something away, and by
[0001](0001-decision-records.md)'s own test that makes it an argument rather than
a pointer. This record does not repair that: the no-editing rule is what stops a
landed paragraph being trimmed, and the paragraph is where it is. The field lands
beside it and the record keeps its text, which is the arrangement 0001 asks for
everywhere else.

## Why this is written down before the code

Without a shape, every later narrowing is argued from scratch by whoever writes
it, and the three answers the issue set out are all still available each time. The
tree then carries pointers in three shapes, a reader learns which shape to look
for by having met one, and the check that would catch a broken pointer cannot be
written at all because there is nothing fixed for it to read.

The specific failure is the one already in the tree rather than an expected one. A
reader of [0103](0103-what-admits-a-dependency-and-what-is-refused.md) who stops
at its header applies a refusal one clause wider than the rule in force, and
refuses a dependency this board decided is admissible. Nothing today tells that
reader anything is missing, which is what makes it expensive: a wrong reading of a
record reads exactly like a right one.

It is also cheaper now than later by the same arithmetic
[0001](0001-decision-records.md) uses on itself. One instance exists. Fixing the
shape after five means five records to reach, none of which may be edited except
under whatever this record decides.

## Alternatives, and what each cost

A supersession of the whole record. `superseded by NNNN` on
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) and a fresh record
restating it. It costs five things nobody argued with - the licence set, the worth
test, the other four grounds and the clause for a standing requirement - discarded
to move one, and it gets worse with use: every later narrowing restates a record
in full, so the change that matters is buried in a diff that does not, and the
reader who wants to know what moved reconstructs it.

A partial supersession. `superseded in part by NNNN` in the `Status:` line, which
is closer and fails on the point this is about. It tells a reader that something
was narrowed without telling them what, so they read both records end to end to
find the clause. It also makes `Status:` false in the direction that costs most: a
reader who concludes the record is spent stops applying four grounds that still
hold.

Neither, with the pointer left as prose. The cheapest, and it is the state this
record ends. It leaves a reader who stops at the header applying a rule that was
narrowed, leaves the shape to be re-argued at every instance, and leaves nothing
for a check to read.

A field with the record number and no clause. One token shorter and mechanically
identical to check for existence. It costs the reader the thing they came for, and
it is the option that looks like this decision from a distance, which is why the
clause is refused by name rather than recommended in prose.

## What would reverse this

A narrowing that cannot be named in one clause, twice. One awkward fit is a field
written badly. Two is a narrowing that is really a supersession wearing a field,
and the answer is the partial supersession this record declined rather than a
longer clause.

A record is narrowed by two later records on the same clause and the two disagree.
This record fixes a pointer and decides nothing about precedence, and the first
time that matters the format needs a rule this one does not carry.

The check is superseded by a decision-record reader that judges the whole header,
under #110's successor or otherwise, and the shape that reader can actually read
differs from the shape written here. The check wins, for the reason
[0001](0001-decision-records.md) already gives about a shape nothing refuses.
