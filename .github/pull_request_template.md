<!--
Nothing reads this template. No check parses it, no check requires a section to be
filled in, and a pull request that deletes the whole thing and says nothing passes
every check on this repository. It is a prompt for a person, not a gate, and it is
written down here so nobody mistakes a filled-in form for a verified one.

Delete these comments and the guidance under each heading as you fill it in.
-->

## The issue this belongs to

Closes #

<!--
One issue per pull request. If the change does not have an issue, it does not have
a statement of what was wrong, what the evidence was, or what done means, and the
review has nothing to check the change against.

If the change touches something a second issue owns, say which and say why the two
could not be separated.
-->

## What changed

<!--
What the change does, in the words somebody reading the repository in a year would
use. Not a list of the files, which the diff already carries.
-->

## What failure it prevents

<!--
The specific thing that goes wrong without this change. A failure that has already
happened is stronger than one that could: say which it is.

Where this is a correction, say what was wrong and how it was found.
-->

## Evidence

<!--
Every number in this pull request carries the command that produced it, pasted with
its output, run at the commit being pushed rather than in a working tree that has
something else in it.

That means every number. Faster, smaller, greener, higher coverage, fewer
allocations, a count of anything. A sentence with a number and no command is a
claim, and the difference between a claim and evidence is one line.

    <the command>
    <its output>

Where a claim cannot be backed by a command, write it as a claim and say so, rather
than writing it as a measurement.
-->

## What a guard here refuses, and the proof it bites

<!--
Only where this change adds or edits a guard, a check or a test that is meant to
refuse something. Delete this section otherwise.

Name what it refuses. Then show it refusing: the deliberate violation, the run that
went red on it, and a run that stays green without the violation. A guard nobody
watched fail is a guard nobody knows the direction of.
-->

## What this does not cover

<!--
What is deliberately left out, what was not measured, and what no run here touched.
A negative statement stays negative: if something was not done, say it was not done,
and do not soften it in a later edit.

Anything skipped, and why. A test that needs elevation, a platform nothing built
for, a path only a real server reaches.
-->

## Who has read it

<!--
Say plainly whether anybody other than the author has read this change. Where
nobody has, say so and let the evidence above stand in place of a review, rather
than leaving the question open.
-->
