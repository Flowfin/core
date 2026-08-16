# 0060. What counts as watched, and who said so

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #60

## The decision

The core marks an item watched at exactly the boundary 0058 stops offering a
resume at, taking that record's number rather than choosing a second one; every
mark carries whether a person made it or the core did, and only a mark the core
made is ever reconsidered; an item whose duration the server does not state is
never marked watched by the core at all; and a mark is an assertion of a desired
state on the queue in 0047, so it survives an absent server exactly as a position
does and never as something that accumulates.

## One number, not two

0058 fixes the end of an item at the last ninety seconds or the last five per
cent of its duration, whichever is shorter. Passing that boundary is what the
core treats as having watched the item.

This record states no number of its own, and that is the decision rather than a
convenience. The issue asks that the two cannot disagree, and the only
arrangement in which two numbers cannot disagree is one number. Two numbers with
a rule that they must agree is a rule somebody has to check, and the check is a
test that reads two constants, which passes on the day it is written and fails to
exist on the day one of them is edited by somebody who did not know the other was
there.

What that leaves for #60's own condition is worth being plain about. A test that
would fail if the two numbers were changed independently cannot be written,
because there is nothing to change independently. What can be written, and what
that condition becomes, is a test that an item at the boundary is both marked
watched and offered no resume, and one a moment before the boundary is neither.
A change to the number moves both halves of that test together, which is the
property the condition was asking for.

The two are the same event seen from two sides. An item the core will not offer a
resume for and does not consider watched is an item in a state nobody asked for:
it is not in a list of things to continue and not in a list of things finished,
so it has silently left both, which is worse than being in the wrong one.

## An item whose duration the server does not state

Nothing is marked watched by the core.

A proportion cannot be computed without a duration, so 0058's boundary does not
exist for that item, and there is no second rule here to fall back on. Inventing
one would mean marking something watched on a fixed duration alone, which 0058
already refuses at both ends of the length range and refuses for a reason that
does not weaken because the duration is missing. A fixed rule applied to an item
of unknown length is a fixed rule applied to something that may be three minutes
long.

What still works for that item is a person saying so. A mark a person made needs
no duration and is not a computation, and it is the only way such an item ever
becomes watched.

0058 answers the same absence the same way, and both records say so, because a
reader arriving at either one is deciding what to do with the same missing
number.

## Who said so, and what that changes

Every mark carries which of the two it was. A person marked this watched, or the
core did on the rule above.

Only a mark the core made is ever reconsidered. A person who seeks back before
the boundary on an item the core marked has said something about it that the core
should believe, and the mark is withdrawn. The same seek on an item the person
marked themselves changes nothing, because the core would be undoing a statement
somebody made on purpose, and being wrong in that direction is the version people
notice and resent.

The distinction is the core's own and it lives beside the entry. A server states
that an item was played and does not state who decided that, so the distinction
does not survive a round trip through it.

That has a consequence which has to be chosen rather than left to the first
process that starts with an empty memory. A mark that came back from the server,
with nothing local saying otherwise, is treated as the person's and is never
reconsidered. That is the safe direction: treating it as the core's would let a
fresh process withdraw a mark somebody set deliberately on another device, and
the person would see something they had marked finished return to a list of
things to continue with no act of theirs behind it. The cost is the opposite
error, which is that a mark the core made on another device is never withdrawn on
this one, and that error leaves an item in the state it is already in.

## While the server is gone

A mark goes on the queue in 0047 exactly as a position does, and it inherits
everything that record already decided rather than acquiring a rule here.

It is an assertion of a desired state. This item is watched. Not an increment and
not a play count, so a delivery whose acknowledgement never arrived can be sent
again with no second effect, which 0047 fixes as the property that makes the
queue safe at all.

It coalesces per kind and per target, and 0047 already states that a position
report and a watched mark are two statements about one item and neither replaces
the other, so an item that was marked and then given a position carries both.

Withdrawing a mark is a statement of the same kind and replaces the earlier one
in place, keeping its position in the order, which is 0047's coalescing rule with
nothing added. A person who marked an episode watched and then changed their mind
has told the server two things and the last one stands.

Where a server offers only an operation that accumulates rather than an assertion,
0047 refuses to queue it at all and the caller is told with a kind from 0004.
Whether a supported server line offers an assertion here is part of the surface
#10 owns, and this record is written for the case where it does; where it does
not, the loss is 0047's stated loss rather than a new one.

## What this record does not decide

The boundary itself. 0058, which this record takes rather than restates.

The unit and the precision a position is expressed in. #56.

What happens to a mark on the way back from an absent server, when the server's
own state moved while the device was away. #59, under 0058's rule for the same
disagreement.

Whether a watched mark is personal data. 0068 already places it there by name,
and 0071 decides what that means for a diagnostic event.

What signing out does to a queue holding an undelivered mark. 0114.

## Why this is written down before the code

The completion rule is the one thing on this board that writes to a person's own
history without being asked to, so getting it wrong is not a defect a person
reports, it is a defect a person distrusts the application for. The two failures
are opposite and both are cheap to write. Marking too early loses a series out of
whatever a client builds from part-watched items, at the moment somebody left the
room during the credits. Marking too late leaves a finished film in that list
forever, which is the state people clear by hand and then stop trusting.

The part that gets written wrong even by somebody thinking about it is the second
number. Completion is written in one place, the resume boundary in another, and
they are written weeks apart by whoever needed each. They agree on the day both
are written, and the drift arrives when one is tuned in response to a report,
because nothing connects them and nobody knows to look. A number stated twice is
a number that has to be corrected twice, and the repair is to state it once and
refer to it.

The mark's origin is the part that is not written at all. A mark is a boolean by
the time it reaches a server, so a core that does not decide this ends up with a
boolean too, and every later question about whether the core may withdraw a mark
gets answered by whoever is looking at it. The answer that gets written is that
it may, because that is what makes seeking backwards behave, and the person it
fails is the one who marked something finished on purpose.

None of that has happened here. There is no code in this tree and no language in
which to write any.

## Alternatives, and what each cost

A completion proportion of its own, the widely used ninety per cent among them.
It is the number most things a person has used already behave like, so it would
surprise nobody. It is a second number that has to agree with 0058's boundary,
and it does not: ninety per cent of a two hour film is twelve minutes from the
end, so an item would be marked watched while the core still offered a resume
into it, and the two lists a client builds would both contain it.

Two numbers with a stated relation, for instance completion at or after the
resume boundary. More expressive, and it allows a deliberate gap between the two
if one is ever wanted. The relation is prose unless something checks it, checking
it means a test reading two constants, and 0001's own argument about a list in a
document drifting against the thing it describes is the same argument one step
down.

Marking watched only when the person asks. Nothing is ever wrong, nothing is ever
withdrawn, and the core writes to a person's history only on request. It moves the
rule into every client rather than removing it, since a client that shows a
continue-watching list still has to decide what leaves it, and it makes the core's
position reports arrive at a server that will apply its own completion rule
anyway, so the drift is between the core and the server instead of between
clients.

Trusting the server's own completion state and holding none of this. The server
already decides something like this and it is the thing the server will report to
every other client. It costs the case the core exists for, which is behaviour that
is the same across eleven clients on server lines that do not agree with each
other, and it costs the origin distinction outright, since a server states that an
item was played and not who decided it.

Reconsidering a person's own mark on a seek, treating any backward seek as the
newest statement. It is consistent, it needs no origin field, and seeking back is
genuinely a statement. It is wrong in exactly the direction people notice: a
person who marked a series finished and then opened one episode to show somebody a
scene has just unmarked it.

## What would reverse this

0058's boundary is superseded. This record takes that number by reference, so the
supersession reaches here without an edit, and the reversal is only recorded if
the replacement's boundary is one completion should not share.

A supported server line applies its own completion rule to a position report,
early enough that an item is marked watched at the server before the core's
boundary. The core's rule is then advisory rather than the rule, and the
replacement says which one a client is told about.

A person's own mark is found being withdrawn by a fresh process, which the rule
above is written to prevent. That means the origin does survive a round trip in
some form nobody accounted for, or that the local state is being lost more often
than assumed, and either one is a different record.

An item with no stated duration turns out to be common rather than exceptional on
a supported server line. Never marking it watched is then a hole a person meets
rather than an edge, and the replacement decides what a completion rule does with
no denominator instead of declining to have one.
