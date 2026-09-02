# 0001. Decision records

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrowed-by: 0267, on the third permitted edit, the pointer to a later record, which takes a fixed form and becomes a pair of fields where the later record narrows a clause

Narrowed-by: 0269, on the rule that a record is never edited in place, which does not reach a pasted output deleted so that only the command that produced it is left

Issue: #2

## The decision

A decision that shapes this repository is written as one Markdown file under
`docs/decisions/`, carrying a number, a date, a status, the issue it came from,
the decision in a single sentence, the reasons stated as what would have gone
wrong without it, the alternatives with what each one cost, and a reversal
condition concrete enough that a reader can tell whether it has happened; a
record is added or superseded and never edited in place, and
`docs/decisions/README.md` lists every record by number and title.

## What a record must carry

The file name is `NNNN-short-title.md`, four digits, lower case, words separated
by hyphens.

The first line is `# NNNN. Title`, the same number as the file name.

Then three lines, each on its own, in this order:

    Date: YYYY-MM-DD

    Status: accepted. Supersedes nothing. Superseded by nothing.

    Issue: #N

`Date` is the day the decision was taken, not the day the file was last touched.
`Status` names both directions, and both halves say `nothing` where there is
nothing to name, so that a reader never has to decide whether an absent sentence
means no supersession or an unfinished record. `Issue` is the issue the decision
came from, so that the argument behind the record stays reachable after the
record has been compressed into its conclusions.

Then four headings, in this order, with any number of the record's own headings
between the first and the second:

`## The decision` states the decision in one sentence. One sentence is a
constraint with a purpose: a decision that cannot be stated in one sentence is
usually two decisions, and splitting it is cheaper before anything depends on
either half.

`## Why this is written down before the code` states what would have gone wrong
without the record. Not the benefit of having decided, which is always available
and never says anything, but the specific failure the absence produces, and
whether it has already happened somewhere or is expected.

`## Alternatives, and what each cost` states the options that were genuinely
considered and what each one would have cost. An alternative listed only to be
dismissed in the same sentence teaches a later reader nothing, and the reader
this section exists for is the one proposing that alternative again in a year.

`## What would reverse this` states the condition under which the decision stops
holding, written so that a reader can look at the world and say whether it has
happened. "If this turns out to be wrong" is not such a condition. A count, a
named event, or a measurement is.

## Numbering

`0001` is this record. Every record after it takes the number of the issue whose
decision it records, so a reader holding an issue number can reach the record
without the index, and a reader holding a record can reach the argument without
searching. Numbers are never reused, and the sequence has gaps wherever an issue
produced no record, which is most of them.

The records that were already in the tree when this was written follow it, which
is why the rule is written as it is rather than as a fresh convention:

    $ ls docs/decisions/
    0001-decision-records.md
    0003-what-the-core-does-not-do.md
    0004-the-error-vocabulary.md
    0005-the-session-model.md
    0006-the-cache-contract.md
    0007-a-slow-server-and-a-server-that-is-gone.md
    0008-what-the-core-can-measure-of-the-speed-budget.md
    0009-the-concurrency-model.md
    0102-the-clocks-every-deadline-is-measured-against.md
    0112-where-the-platform-decoder-begins.md
    README.md

## Superseding, and why nothing is edited in place

A record that stops being true keeps its text. Its `Status` line becomes
`superseded by NNNN`, and the new record's `Status` line names what it
supersedes. Nothing else in the old file changes.

Editing a record in place destroys the thing the record was written for. The
value of a decision record is not the decision, which the code also carries; it
is the reasoning that was available at the time, including the parts that turned
out to be wrong. A record quietly corrected reads afterwards as a decision that
was always right, and the next person facing the same question learns nothing
from it and repeats the correction.

Three edits are not supersessions and are allowed. Fixing a typographical error
that changes no meaning. Adding the `superseded by` half of a `Status` line when
the replacing record lands. Adding a pointer to a later record that goes further
on a case this record already names, where the pointer changes no sentence's
meaning and adds no argument.

All three are the same kind of change: they add a route between records, or
correct how a sentence is spelled, and none of them touches what was decided or
why. Anything that changes what the record decides, what it argued, or what would
reverse it, is a new record.

The third is the one that will be abused, so it is worth naming the test. If the
edit could be removed and the record would still say exactly what it said before,
it is a pointer. If removing it would take a reason away, it is an argument and it
belongs in the new record.

## The index

`docs/decisions/README.md` lists every record by number and title, in number
order, each as a link to its file. A reader opens the index and reaches every
record from it.

The index carries the number, the title and nothing else. A summary column would
be a second statement of what the record says, and the two drift apart the first
time a record is superseded and its one-line summary is not.

Adding a record without adding its index line leaves the index wrong, and nothing
in this repository refuses that today. There is no check over `docs/decisions/`,
and the workflows this repository carries are the ones the tree holds:

    $ ls .github/workflows/
    dco.yml
    dependency-review.yml
    scorecard.yml
    unicode-guard.yml
    zizmor.yml

None of them reads a decision record. #110 is where a document check is asked
for, and until something like it exists the index is held by whoever adds the
record.

## What a record is not

Not a specification. A record says what was decided and why; the interface, the
field names and the exact behaviour live where the code and its tests are, and a
record that restates them becomes wrong at the first change nobody thought to
copy back.

Not a status page. A record does not say what is implemented, in progress, or
planned. The board holds that, and it moves daily.

Not a place for numbers without their commands. Where a record asserts a
measurement, it carries the command that produced it, in the same shape as the
listings above.

## Why this is written down before the code

Records written on ten different days without a fixed shape answer ten different
sets of questions. The reader cannot tell a decision from an opinion, cannot find
the reversal condition because half the records have none, and cannot tell
whether an alternative went unmentioned because it was rejected or because
nobody thought of it. That gap is invisible while the records are being written
and expensive at the moment somebody wants to reopen one, which is the only
moment a record is read at all.

The specific failure this format is against is the record that says what was
decided and not what it cost. Such a record cannot be argued with, because every
argument against it is met with the decision restated. A record that names its
alternatives and its reversal condition can be argued with using its own text,
which is the property that makes reopening a decision a normal act rather than a
challenge to whoever took it.

Fixing the shape after ten records exist means either ten rewrites, which the
no-editing rule forbids, or ten supersessions that change nothing but their
headings. Either way the format costs ten times what it costs now.

## Alternatives, and what each cost

An issue thread as the record, with no file. The argument is already there, with
its dates and its participants, and nothing has to be maintained. It costs the
reader everything: an issue is a conversation, so the decision is somewhere in
the middle of it, the parts that were abandoned look the same as the parts that
were kept, and a decision reversed later leaves the original thread reading as
current. A tracker also belongs to a hosting provider, and a repository that
outlives one loses its reasoning.

A single decisions document, appended to. One file, one place to look, and no
index to keep. It costs the supersession rule, which needs a record to have an
identity that a later record can point at, and it makes every change to any
decision a change to one file that every open branch also touches.

A published template with fields, such as the widely used lightweight formats.
Cheap to adopt and understood outside this repository. Their required sections
are context, decision and consequences, which is a shape that records what
follows from the decision and not what the decision cost. The two sections this
format cares most about, the alternatives with their prices and a reversal
condition somebody can check, are optional there, and an optional section is one
that is absent on the day it matters.

Records with no fixed shape at all, trusting the author. The cheapest possible,
and it is the state this record ends. It works while one person writes all of
them and fails at the second author, which is the case this repository is built
for.

## What would reverse this

A record whose subject genuinely does not fit these headings, twice. One awkward
fit is a record written badly. Two is a shape the format does not cover, and the
format is superseded by one that covers it rather than being bent around it.

The index goes wrong twice without being noticed until a reader reports it. That
is evidence the index needs a check rather than a convention, and the record is
superseded by one that describes what the check reads.

A decision-record format becomes enforced by something in this repository, for
example under #110, and the shape that check can actually read differs from the
shape written here. The check wins, because a shape nothing refuses is a
suggestion, and this record is superseded by one describing what is refused.
