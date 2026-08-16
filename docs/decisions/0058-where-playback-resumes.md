# 0058. Where playback resumes, the two ends of an item, and whose position wins

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #58

## The decision

A resume starts ten seconds before the position that was recorded, which is 0057's
interval at the boundary of the constraint that record already states; an item is
finished rather than resumed once the recorded position is inside the last ninety
seconds or the last five per cent of its duration, whichever of the two is
shorter; no resume position is kept at all below the first sixty seconds or the
first five per cent, again whichever is shorter; and where the device holds a
position it has not yet delivered, that one wins over the server's, decided by who
has not yet spoken rather than by which number is larger.

## What these numbers are made of

Three durations, two proportions and a rule. The unit and the precision they are
expressed in are #56's, and nothing here depends on either beyond a comparison at
a boundary, so this record is written for whatever answer that issue takes.

The proportions need a duration to be computed against, and the server does not
always know one. Where it does not, none of the three thresholds applies: a
position is kept from the first moment, an item is never treated as finished on
its own, and the rewind is the only rule that still holds, because it needs
nothing but the position itself. That is the same absence 0060 answers for a
watched mark, answered the same way here, and it is stated in both records
because a reader arriving at either one is deciding what to do with the same
missing number.

## Ten seconds before the recorded position

0057 fixes the reporting interval at ten seconds and states the constraint in the
direction that can be checked: the interval may not exceed the rewind. Ten
satisfies it at the boundary, and the boundary is the right place to sit rather
than a place to leave margin at.

Every second of rewind beyond that is paid on every resume, including the
overwhelming majority where nothing was lost at all, and the person paying it
most is the one who pauses most. What it buys is nothing, because the loss it
would be buying against is already bounded by 0057 at ten seconds and a longer
rewind does not recover a position that was never recorded. So the rewind is
sized to the constraint and not to a feeling about how much of a sentence a
person wants back.

What a person actually re-watches is not the rewind alone. Where the platform
produced a stop and the position is current, it is ten seconds. Where nothing
produced a stop, because the process was killed for memory or a television lost
power, the recorded position is up to one interval behind reality and the
re-watch is up to twenty seconds. That is the honest number, it is the sum of two
records rather than a number either one states, and it is written here because a
reader looking for "how much do I see again" will otherwise find ten and be
wrong.

The rewind is not applied where the recorded position is below it. A resume to a
negative position is not a case to handle, it is a start, and #56 already refuses
a signed position going negative on a seek to the start.

## The last ninety seconds, or the last five per cent

Past that boundary an item is finished. The core offers no resume for it, and
0060 marks it watched at the same boundary rather than at one of its own.

Neither half of the pair works alone, which is why it is a pair. A fixed duration
alone is wrong at the short end: ninety seconds into a three minute item is half
of it, and a person who watched ninety seconds of something three minutes long
has watched half of something rather than finished it. A proportion alone is
wrong at the long end: five per cent of a two hour film is six minutes, which is
a scene rather than a credit roll, and a person who stopped six minutes from the
end stopped in the middle of something.

Taking the shorter of the two gives ninety seconds to anything above thirty
minutes and a proportion to everything below it, and the crossing point is where
the two rules agree rather than a third number.

Ninety seconds rather than sixty is the end of the argument rather than the whole
of it. Closing credits on television drama run about a minute, and on anything
with a song over them they run longer; sixty seconds marks an item finished while
the last of it is still playing, and the person that fails is the one who was
still watching.

## The first sixty seconds, or the first five per cent

Below that boundary no resume position is kept. The item is offered from the
start next time, and nothing appears in whatever a client builds out of items
with a position.

The failure this is against is the list rather than the item. A person who opened
something, decided it was not what they wanted and left is a person who has just
put an entry into a list of things they are part way through, and a list built
that way fills with items nobody intends to return to until it is worthless. That
cost is carried by the list, so the threshold is sized to what a person does
before deciding rather than to anything about the item.

Sixty seconds is above the rewind by a margin that matters, and a threshold below
the rewind would do nothing at all: a recorded position under ten seconds already
resumes at the start. So the number has to clear ten to have any effect, and
sixty is where a person has seen an opening rather than a moment of it.

The proportion is the same shape as the one at the other end and is there for the
same reason, since sixty seconds into a three minute item is a third of it.

## Whose position wins

The two disagree whenever the same person watched some of the same item
somewhere else, which on a household with a television and a phone is ordinary
rather than exceptional.

The rule is delivery order rather than magnitude. Where the device holds a
position for that item that it has not yet delivered, on the queue in 0047, the
device's position wins and the server's is not read. Where it holds none, the
server's position is taken as it stands and no comparison is made at all.

That rule needs no clock, which is what makes it worth taking. Comparing two
moments means comparing two devices' readings of a wall clock, 0102 already
places a moment on the clock a device can have wrong, and a rule that resolves a
person's viewing history by trusting whichever device's clock is further ahead
fails in a way nobody can see. Comparing two positions instead means the larger
number wins, and the larger number is wrong in the case that matters most: a
person who finished something on a phone and deliberately started it again on a
television would be sent back to the end of it.

Delivery order is derivable from what the device already holds. An undelivered
statement has by construction not reached the server, so nothing the server holds
can be a reply to it, and 0047 already makes every queued action an assertion of
a desired state rather than a step, so the device's statement is the last thing
the person said from this device and it is going to overwrite the server's
anyway.

The case it gets wrong is worth naming rather than leaving to be discovered. A
device that queued a position, then did not reach a server for a long time while
the person watched the same item somewhere else, comes back holding a statement
that is older than the server's and still wins. What bounds that window is
0047's own bound on the queue and the recovery schedule in 0045 that drains it,
and neither of them is a bound on how long a person may leave a device switched
off. So this rule is right in the ordinary case and stale in the case where a
device was away longer than the person's viewing, and the alternative that fixes
that case pays a clock for it.

#59 reconciles the queue against the server on return and 0047 already says the
rule there is this one. That is the same sentence read from the other side, and
nothing in this record adds to it.

## What this leaves for other issues

The unit and the precision a position is expressed in, and what happens past the
end and before the beginning. #56.

What counts as watched, whether a mark was the person's or the core's, and the
behaviour of a mark while offline. 0060, which takes the boundary above rather
than choosing one.

The cadence a position is reported on and which events report at once. 0057,
which this record satisfies rather than revisits.

What happens to a position recorded while the server was gone, on the way back.
#59, on 0047's queue.

## Why this is written down before the code

Three of these four are numbers, and a number that is not decided is not absent.
It is present in whatever the first caller wrote, and the first caller writes the
one that makes the case in front of them behave. That is how eleven clients end up
with eleven rewinds, which is the drift this repository exists against, and it is
worse here than for most numbers because a person notices this one immediately and
attributes it to whichever client they saw it in.

The pair at each end is the part that is got wrong even when somebody does decide.
A single fixed duration is the obvious choice, it is correct for the item the
author was looking at, and it is wrong at the other end of the length range in a
way that only shows on content nobody tested with. The same is true of a single
proportion in the opposite direction. Writing the pair down with the crossing
point costs a paragraph now and costs a change to every client later.

The disagreement rule is the one that is not written at all. Two positions arrive,
one of them is used, and which one is decided by the order the code happened to
read them in. It has no visible failure until a person watches something in two
places, at which point it produces a result nobody can explain, because there was
never a rule to explain.

None of that has happened here. There is no code in this tree and no language in
which to write any, so every number above is a decision and not a measurement.

## Alternatives, and what each cost

A rewind longer than the interval, ten being the floor rather than the answer.
Fifteen or twenty seconds gives a person more context and it is what several
things a person has used already do. It is paid on every ordinary resume, where
nothing was lost, and it buys against a loss 0057 has already bounded. The
argument for it is comfort rather than correctness, and if that argument is
accepted it should be made from a measurement rather than from a preference,
which is what #65's harness would be for.

No rewind at all, resuming exactly where the position says. It is the honest
reading of the recorded number and it needs no argument about how much context
somebody wants. It breaks 0057's constraint outright, so a device killed without a
stop event resumes ahead of nothing but still lands the person mid-word, and it
makes the ten second interval a loss a person feels rather than one absorbed.

A single near-end duration with no proportion, or a single proportion with no
duration. One number each, easier to state and easier to test. Each is correct
over one part of the length range and visibly wrong over the other, and the part
it is wrong over is short items in the first case and films in the second, which
are the two things this core is for.

Resolving a disagreement by the later moment, using the wall clock on each side.
It is the rule that is actually right, in the sense that it answers the question
somebody is asking. It costs a clock comparison across devices, which 0102 places
on the clock that can be wrong, and a device with a clock an hour ahead would win
every disagreement it took part in for as long as nobody noticed.

Resolving it by the larger position. No clock, no queue inspection, and it is
right whenever a person is moving forward through something. It is wrong in the
one case a person would definitely notice, which is deliberately starting again
from the beginning, and being wrong there converts a feature into a thing people
warn each other about.

Asking the client to resolve it. The client knows what is on screen and could ask
the person. It puts the rule in eleven places, and the version each one writes is
the one above that needs no queue inspection, so the outcome is the larger
position with extra steps.

## What would reverse this

#56 fixes a precision coarser than the thresholds here can be compared at. The
numbers are then boundaries nothing can sit exactly on, and the replacement states
them in the unit that record chose rather than in seconds.

0057's interval moves. The rewind is tied to it by that record's constraint rather
than by preference, so an interval that moves takes the rewind with it, and the
record that changes one names the other.

A measurement from #65's harness shows that the re-watch of up to twenty seconds
after a process is killed is one people notice and complain about. The answer is
then a position the core observes more often rather than a longer rewind, which is
0005's own reversal condition reached from this side.

The disagreement rule is met by a person whose device was away long enough for the
stale case above to bite, twice. Once is an unlucky device. Twice means the window
0047 and 0045 bound is not in fact bounded by anything a person experiences, and
the replacement pays for a clock and says which one.

Content is met whose closing credits routinely exceed ninety seconds, so that an
item is offered as unfinished while nothing but a logo is on screen. The pair is
then the wrong shape rather than the wrong size, and what replaces it takes the
boundary from something the server states about the item rather than from a
number here.
