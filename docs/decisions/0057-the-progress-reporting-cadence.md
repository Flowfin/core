# 0057. The cadence a playback position is reported on

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #57

## The decision

While something is playing the core reports the position every ten seconds on the
elapsed clock, five events report the moment they happen rather than waiting for
the interval, the interval does not run while playback is paused, and every one
of those reports is put on the queue in 0047 rather than sent to the server, so
that a scrub becomes one report through that record's coalescing rule and not
through a second one written here.

## The interval, and the two things it is between

Ten seconds.

The number sits between a cost that is paid always and a loss that is paid
rarely, and it is chosen rather than measured. The harness in #65 is where a
measured replacement would come from.

The cost is arithmetic rather than a claim. One stream reporting every ten
seconds is three hundred and sixty reports an hour, and a household watching four
things at once is one thousand four hundred and forty writes an hour arriving at
one server. That is what any shorter interval multiplies, and it lands on a
machine an operator runs at home, which on this board is the machine that is a
small computer under a television rather than a server rack.

The loss is what somebody's position is worth when the report that would have
carried it never happened. A process the platform killed for memory, a television
losing power, an application swiped away: none of those produces a stop event, so
the last thing the server heard is the last interval report. The interval is
therefore the upper bound on how far back somebody is thrown when they come back
to an item.

That bound has to be read against the rewind #58 decides, because a resume that
rewinds by more than the interval absorbs the whole loss and a resume that
rewinds by less does not. So this record puts a constraint on that one rather
than taking a number from it: the interval may not exceed the rewind. If #58
fixes a rewind shorter than ten seconds, one of the two records is wrong and the
disagreement is visible rather than latent, which is the point of writing the
constraint down in the direction that can be checked.

## The interval does not run while paused

A paused stream has a position that is not moving. An interval that kept running
would enqueue the same position repeatedly, 0047 would coalesce every one of them
into the single entry that is already there, and the only thing left would be the
wake-up. On a handheld that is a wake-up per interval for as long as somebody
leaves something paused, which can be overnight.

Pausing itself reports, and so does resuming, so the server is told where
playback stopped and where it started again. Nothing is lost by the interval
standing still between the two.

## What reports without waiting

Five events, and they report at the moment they happen: started, paused, resumed,
seeked, stopped.

They are events rather than intervals, so no clock moves them and 0102 has
nothing to say about them. Each of the five is a moment where the position
changed in a way the interval would misrepresent if it were the only route:
started and stopped bound the whole thing, paused and resumed are what a person
will look for when they come back, and a seek moves the position by an amount an
interval cannot interpolate.

Reporting immediately means enqueued immediately. Whether it reaches the server
now depends on whether the server is there, which is 0047's and 0045's, and this
record does not promise delivery it is not in a position to make. The distinction
matters most for stopped, since that is the report a person most expects to have
landed, and the honest statement is that it is durable rather than delivered.

Stopping the core is not one of the five. 0115 already fixes that a stop neither
drains the queue nor discards it, so a report enqueued a moment before is still
there afterwards, and adding a sixth event for it would be a second mechanism for
a promise 0047 already keeps.

## Coalescing is 0047's, and this record only says what a target is

The issue this record comes from asks that a person scrubbing through a film
sends one report rather than forty. That is already true, and it is true because
0047 coalesces at the moment of enqueue, replacing an earlier action for one
target with the later one while keeping the earlier one's position in the order.

Writing a second coalescing rule here is the thing to avoid, so what this record
adds is only the granularity 0047's rule is applied at.

The target is the item within the session. Two positions for one item collapse to
the later one. Positions for two items do not collapse, which is the person who
started something else and came back.

The kind is the position. 0047 already fixes that coalescing is per kind as well
as per target, so a position and a watched mark are two statements about one item
and neither replaces the other. A seek, a pause and an interval tick all produce
the same kind, which is why forty of them become one.

The consequence worth stating is that the immediate events are immediate in when
they are enqueued and not in how many entries they leave behind. Forty seeks in
five seconds enqueue forty times and leave one entry, and the entry holds the
last position rather than the first. That is the same behaviour a device that
slept through three cadence ticks gets, which 0102 already settles as owing one
report rather than three, and the two agree because they are the same mechanism
rather than two that were made to match.

## What a report carries, and what decides it

The position, whose unit and precision are #56's and are not restated here.

Where a report carries a moment as well as a position, that moment is on the wall
clock, because the server has an opinion about it too and 0102 allows a moment on
no other clock.

Nothing else in this record depends on the unit, which is why the cadence could
be decided before #56 lands. An interval is a duration between reports and a
position is what a report carries, and the two do not constrain each other.

## Why this is written down before the code

Three landed records lean on a cadence that does not exist. 0047 says progress
reports were the first thing to require that every write go through the queue and
names this issue for it. 0102 fixes which clock the cadence is on, which is a
sentence about a thing not yet decided. 0007 and 0045 both describe what happens
to a report when the server is gone.

Without the number, the cadence is decided by whichever code first has a position
to report, and the two shapes that arrive there are both wrong in ways that are
invisible at the time. One is reporting on every position change, because that is
what the player hands you and it is obviously correct; it produces a request per
second per stream and nobody notices until an operator with four televisions
asks why their server is busy. The other is reporting only on the five events,
because those are the interesting moments; it is correct on every machine a
developer tests on, since a developer stops playback deliberately, and it loses
the whole of somebody's position on the case that is normal on a television,
which is the process ending without a stop.

Written afterwards, the interval is a number somebody changes in a pull request
with no argument attached, and the constraint against #58's rewind is not
discoverable at all, because nothing connects the two once both are code.

## Alternatives, and what each cost

Reporting on every position change. The most accurate possible answer and the
simplest to write, since the player is already producing the events. It costs a
request per second per stream, multiplied by every stream in a household, at the
one place on this board where the machine belongs to the person paying for it.

A longer interval, thirty seconds or a minute. Fewer writes by a factor of three
to six, which is the direction an operator with a small machine wants. It costs a
loss larger than any rewind #58 is likely to choose, so somebody whose television
lost power comes back to a position visibly behind where they were, which is the
failure that reads as the application losing their place.

An interval that adapts to the connection, reporting less on something metered.
It is the answer that sounds right, and it is refused because the core cannot
know. 0003 refuses the core platform knowledge, so whether a connection is
metered is a thing only a client holds, and an adaptive rule here would be
adapting to a guess.

Reporting only on the five events and not on an interval at all. Nothing periodic
to schedule, no wake-ups, and no writes during steady playback. It costs the case
this cadence exists for, which is the process that ends without a stop event, and
that case is ordinary rather than exceptional on a television.

Sending reports to the server directly and using the queue only when the server
is unreachable. It reads as the smaller change and it is what a first
implementation does. 0047 already refuses it, and the reason applies here with
particular force: the two paths agree until they do not, and the disagreement is
reachable only on a device whose connectivity changed in the middle of playback,
which is a phone on a train, which is the case somebody built the queue for.

Coalescing at the moment of drain instead of at enqueue. It keeps every position
somebody produced, which is more information. 0047 refuses it because the queue
then holds all ninety positions from a scrub and its bound is reached by activity
rather than by breadth, punishing the person who used the application most.

Keeping the interval running while paused, so that the cadence is one rule with
no exception. Slightly less to explain. It costs a wake-up per interval for as
long as something is left paused, and it produces nothing, because 0047 coalesces
every one of those reports into an entry that is already there.

## What would reverse this

#58 fixes a rewind shorter than ten seconds. The constraint above is then broken,
and one of the two records is superseded rather than the interval being quietly
adjusted, because the argument for the number is what has to change.

The measured request volume from the harness in #65 shows the interval is a
visible share of what a server spends on a household. The number then comes from
that measurement, and the record is superseded by one carrying it with the
command that produced it.

A position is observed lost, past the rewind, on a device that did not lose
power, twice. One is a defect in the reporting path. Two means the interval is
not the bound this record says it is, and the record is superseded by one written
against whatever was actually happening.

The five immediate events turn out not to be five. An event that has to report
without waiting and is not on the list, or one on the list that produces nothing
a server does anything with, is a change to what this record decided rather than
an addition to a list, and it lands as a superseding record.
