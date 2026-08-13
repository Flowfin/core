---
name: Something here is wrong
about: A defect, a gap, or a rule with nothing behind it
title: ''
labels: ''
assignees: ''
---

<!--
Nothing reads this template. No check parses it, no heading is required, and an
issue that deletes the whole thing and says what is wrong in three sentences
passes every check on this repository. It is a prompt for a person, not a gate,
and it is written down here so that nobody mistakes a filled-in form for a
checked one.

Delete these comments and the guidance under each heading as you fill it in.
-->

## What is wrong

<!--
The thing that is wrong, in the words somebody reading this in a year would use.

Where it is a gap rather than a defect, say what a reader expects to find and
what they find instead. Where it is a rule with nothing behind it, say which
sentence claims the rule and which check was supposed to refuse the violation.
-->

## The evidence

<!--
What you saw, rather than what you concluded from it.

Every number here carries the command that produced it, pasted with its output,
run against the reference a reader will have rather than against a working tree
that has something else in it:

    <the command>
    <its output>

That means every number. A count, a duration, a size, a version, a percentage,
how many files something touches. A sentence with a number and no command is a
claim, and the difference between a claim and evidence is one line.

Where a claim cannot be backed by a command, write it as a claim and say so,
rather than writing it as a measurement.
-->

## What done means

<!--
The condition under which this issue closes, written so that somebody other than
you can look at the repository and say whether it has been met.

"Handle this better" is not such a condition. A command that answers differently,
a file that exists, a check that refuses something it accepts today, or a test
that reddens when a guard is deleted, are.

Where what is being asked for is a guard, say what it has to refuse. A guard
nobody has watched fail is a guard nobody knows the direction of.
-->

## What this waits on

<!--
Anything this cannot move without: another issue, a decision nobody has taken, a
setting on the repository that no branch can change. Naming it here is what lets
a reader tell an issue that is ready from one that is waiting, without opening
every issue it mentions.

Delete this heading if it waits on nothing.
-->
