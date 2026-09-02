# 0269. A reading a record pasted, after the tree has moved

Date: 2026-09-02

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrows: 0001, on the rule that a record is never edited in place, which does not reach a pasted output deleted so that only the command that produced it is left

Issue: #269

## The decision

Where a landed record pastes a command and the output that command produced, and
the output no longer reproduces, the output is deleted and the command is left
standing alone; nothing is pasted in its place, and this is a fourth edit that is
not a supersession.

## The instance, read rather than recalled

[0103](0103-what-admits-a-dependency-and-what-is-refused.md) pastes a command and
three file names under a paragraph arguing that a refusal with a worked case
behind it is quotable:

    git show origin/main:docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md | sed -n '137,140p'

The same command run against the reference a reader has:

    git grep -l '#103' origin/main -- docs/decisions

returns five paths rather than three. Two of them the paste does not carry: the
record itself, which matches on its own `Issue:` line and always did, and
[0011](0011-the-language-the-toolchain-and-the-binding-layer.md), which wrote this
collision down as its own reversal condition before there was a graph to measure.

Nobody edited that paragraph. The count moved because records were added, which is
what makes this a class rather than a mistake somebody made.

One thing near it is untouched and is worth separating, so that a reader does not
raise it a second time. The sentence above the command counts three refusals with
a worked case behind them. That is a count of refusals and not of files, the
command returns files, and the two were never the same number - 0103 itself and
0011 are among the five and neither is a refusal with a worked case. The sentence
is the record's argument, it is unchanged, and this decision reaches the output
under it and nothing else.

## Why the output goes and the command stays

A reader who runs the command finds two files the record does not account for and
cannot tell whether the record is wrong or whether they are. That is the defect
class this repository's rules lead with, standing in the record whose subject is
what may be admitted at all.

Pasting a fresh output repairs the sentence and not the class. The reading was
correct on the day it was written and went stale without being touched; a second
paste is a second thing that will go stale, and the next record naming #103 moves
it again.

The command alone cannot fail in that direction. It either still runs or it does
not, and a command that no longer runs is visible the moment somebody tries it,
where a stale output is invisible to everyone who does not.

The tree already hands a reader a command rather than its output in the two places
that meet this most often. `README.md` says of the dependency graph that what is
there and what it reaches is read rather than written there. `docs/gate-parity.md`
does it twice: once as a sentence telling a reader to re-run rather than read the
paste, and once by declining to write a count into a table cell and handing over
the command instead. This record takes the second of those shapes for a landed
decision record, where the first is not available because it is an edit that adds
prose.

## Why this is a narrowing of 0001 and not a supersession of anything

[0001](0001-decision-records.md) says a record is added or superseded and never
edited in place, and gives the reason: the value of a record is the reasoning that
was available at the time, including the parts that turned out to be wrong, and a
record quietly corrected reads afterwards as a decision that was always right.

A pasted output is not reasoning. It is a reading of a tree at a moment, standing
as evidence for a sentence that is itself unchanged.
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s paragraph argues
that the refusals are quotable; the command is the quotation, and the three lines
under it are what the quotation returned once. Delete them and the record decides
what it decided, argues what it argued, and reverses under the same condition.

So the clause narrowed is the prohibition itself rather than any of the three
edits already carved out of it, and the field
[0001](0001-decision-records.md) receives says so. 0267 is the record that decided
what such a field looks like, one day before this one used it.

## What is lost, stated rather than softened

The historical reading goes out of the file. A reader who wants to know what the
command returned on 2026-08-24 has to go to the history for it:

    git log -p --follow -- docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md

That is a real cost and it is the one argument against this decision. It is
accepted because the alternative keeps a wrong reading in front of every reader in
order to serve one who is looking for a superseded one, and because the history is
where this repository keeps superseded things everywhere else.

Nothing refuses a stale paste. No check here re-runs a command a record wrote
down, and none could without running arbitrary commands out of tracked text, which
is a thing this gate deliberately does not do. So this rule is carried by the
record and by whoever notices, exactly as it was before, and what changed is that
noticing now has a route that does not require superseding a record whole. #269 is
where that gap was named and this record does not close it.

## Why this is written down before the code

Without it the only route 0001 offers for a stale reading is a supersession of the
whole record. Superseding
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) to correct one pasted
grep discards a licence set, a worth test, five grounds and a clause for a
standing requirement that nobody has argued with, and it does that every time any
record's reading moves. The cost is high enough that the honest route is the
expensive one, and a rule whose honest route is the expensive one selects for the
quiet one - which here is leaving the wrong output standing.

The specific failure has already happened and it is the one above. It will happen
again on a schedule nobody controls, because every pasted reading in every record
goes stale when the tree moves rather than when somebody edits the paragraph.

## Alternatives, and what each cost

A supersession of the whole record. It costs the six things above, discarded to
correct three lines, and it gets worse each time: a record superseded for a stale
reading has a successor that carries its own pasted readings and its own future
supersession.

A fresh output pasted in place of the stale one. Cheapest to write and it is the
option that looks like a fix. It costs nothing today and reproduces the defect on
the day the next record names #103, which is measured rather than supposed: the
count moved from three to five without anybody opening the paragraph.

A sentence beside the paste saying the output moves, which is what
`docs/gate-parity.md` does. It costs more than what it replaces: it is an edit
that adds prose to a landed record, so it needs the same permission this record is
about, and it leaves the wrong output standing as the first thing a reader meets.

Leaving it. The state this record ends. A reader who runs the command gets an
answer the record contradicts, in the record that says a claim carries the command
that produced it.

## What would reverse this

A deleted output turns out to have been the argument rather than evidence for one,
twice. One is a paragraph that was written badly. Two is a shape where the output
carried the reasoning, and the answer is a supersession rather than a wider
deletion rule.

The history stops reaching a deleted reading. This decision rests on the history
holding what the file no longer does, so a rewrite of it, or a move that breaks
`--follow`, takes the ground out from under this record rather than merely
inconveniencing a reader.

A check arrives that re-derives a record's pasted readings and refuses one that no
longer reproduces. Then the deletion is no longer the repair, because the paste
can be kept and held true, and this record is superseded by one describing what
that check reads.
