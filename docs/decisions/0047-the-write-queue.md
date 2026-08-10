# 0047. The queue every write goes through

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #47

## The decision

Every write the core makes to a server goes onto one durable queue per session
whether the server is reachable or not, ordered by a counter stored with the queue
rather than by any clock, coalesced at the moment of enqueue so that a later
action for one target replaces an earlier one, expressed as an assertion of a
desired state so that delivering it twice is the same as delivering it once,
bounded by a stated count whose overflow drops the oldest entry and reports every
drop, and never expired by age, because an action a person took is not less true
because the device was away.

## There is one write path

The queue is not the offline path. It is the path.

A write made while the server is answering goes onto the queue and is drained
immediately, and it takes the same route as one made on a train. #57 already
requires this of progress reports, on the grounds that the offline case then needs
no separate path and cannot be forgotten, and this record makes it the rule for
every write rather than for that one caller.

The alternative is two paths that agree until they do not. The direct path is
written first, works, and is what every caller reaches for; the queue is written
second, for the offline case, and is exercised only by whoever is testing the
offline case. Then the two disagree about ordering, about what happens when the
server answers slowly, and about what a caller is told, and the disagreement
appears only on a device that changed state mid-write.

## Ordering

Order is the order the actions were taken, and it is carried by a counter stored
with the queue and increased once per entry. Not a timestamp on any of the three
clocks in 0102.

A counter rather than a clock because the order of somebody's own actions is the
one thing that must not be disturbed by a device clock being corrected, by a
suspension, or by a restart. 0102 already says both monotonic clocks reset at a
restart and that the wall clock moves in both directions. A counter has none of
those properties and needs none of the record's machinery.

The queue is per session, which 0005 already fixes and which follows from the
keying in #41: an action belongs to the account that took it, on the server it was
taken against, and a second account signing in on that device has its own queue and
cannot drain somebody else's.

Draining is in counter order and stops at the first entry that could not be
delivered. Continuing past it would deliver a later action for the same target
ahead of an earlier one for a different target, and the order somebody's actions
arrive in is the only thing the server can use to reconstruct what they did.

## Coalescing happens at enqueue

Two actions touching the same target, of the same kind, in one session, are one
entry: the later replaces the earlier in place, keeping the earlier entry's
position in the order.

At enqueue rather than at drain, and that is the decision rather than an
implementation detail. Coalescing at drain means the queue holds every one of the
ninety positions somebody produced while scrubbing through a film, so the bound
below is reached by activity rather than by breadth, and the person punished by it
is the one who used the application most. Coalescing at enqueue means the queue's
size is the number of distinct things somebody touched.

Keeping the earlier entry's position is what makes the replacement invisible to
the order. A person who marks an episode watched, then plays something else, then
changes their mind about the episode has told the server two things about the
episode and one about the other item, and the last thing they said about the
episode is what stands.

Coalescing is per kind as well as per target, because a position report and a
watched mark are two different statements about one item and neither replaces the
other.

## Delivery is safe to repeat, by construction

Every queued action is an assertion of a desired state. The position for this item
is this. This item is watched. Not a delta, not an increment, not a step.

So a delivery that reached the server and whose acknowledgement did not reach the
device can be sent again with no second effect, which is the case a flaky
reconnection produces and which no amount of bookkeeping on the device can
distinguish from a delivery that never arrived.

An action that cannot be expressed as an assertion is not queued at all. Where a
server offers only an operation that accumulates, the core does not put it on this
queue: it is attempted when it is asked for and it fails when the server is not
there, and the caller is told with a kind from 0004. That is a real loss of
function stated in the open rather than a queue that quietly counts something
twice, and 0009 already refuses reporting a cancelled or undelivered thing as
something it is not.

## The bound, and what overflow costs

The bound is one thousand entries per session. It is a chosen number rather than a
measured one, and what makes it defensible is the coalescing rule above: with
coalescing at enqueue, a thousand entries is a thousand distinct items somebody
touched while the server was away, which is far outside what a month offline
produces. What would produce a measured replacement is the harness in #65 driving
a queue rather than an estimate.

At the bound, a new entry displaces the oldest. The most recent intent is kept,
because the oldest entry is the one whose target the person is least likely to
still be thinking about, and because the alternative refuses to record what
somebody just did while holding something from three weeks ago.

Every drop is reported, at the moment it happens, as an event through the
interface in 0100, carrying the kind of action and the correlator for its target
under 0071 rather than the identifier itself. And the queue keeps its own standing
count of what it dropped, per session, readable through a call that cannot wait in
the terms of 0009, so that a client can tell an operator that something was lost
without having been listening at the moment it was.

Silently discarding a person's action is the failure this whole issue exists to
prevent, so the drop is the one thing here that is both reported as it happens and
recoverable afterwards.

## A restart, and what an entry's wait is anchored on

This is the case #47's own thread leaves open, and the answer follows 0102's shape
for a cache entry rather than reaching for a fourth clock.

An entry stores two moments when it is enqueued: the server's own last stated time
and the device's wall reading at that instant. Its age after a restore is computed
the way 0043 computes a cache entry's age, on 0102's anchor, with the same
correction and the same two guards.

That age is carried for reporting and never acts. It is what lets a client say
that something has been waiting three weeks. It is not a reason to drop an entry,
not an input to the bound, and not a threshold. An action somebody took is not
less true because their device was off, and expiring one would be the silent
discard this record exists against, arriving through a mechanism that looks like
hygiene.

The wait 0102 puts on the elapsed clock is a different quantity and it is not the
one restored. It is the interval between delivery attempts inside one run, it
resets when the process does, and on a restore the core does not resume a wait at
all: it attempts delivery when the server is next known to be reachable, which is
#45's recovery reporting rather than a timer of this queue's own.

A restored queue that treated every entry as freshly enqueued would keep its order
and lose every age, so a client could say only that something is pending. Keeping
the anchor costs two stored moments per entry.

## What this does not decide

The schedule on which the core tries an absent server again, and how the recovery
is reported. #45, and this queue drains on that report rather than polling.

The cadence progress is reported on, and which playback events report immediately.
#57.

What happens when the server's own position moved while the device was away,
because the same person watched elsewhere. #59, and it is the same rule as #58.
This record delivers what somebody did; deciding which of two truths wins is not
delivery.

What a half-written or previous-version queue entry may do. #105, which this
record's own comment thread already places there, and until it lands everything
here describes entries this version wrote completely.

What signing out does to a queue that is not empty. #114, which 0005 already
requires to answer the same way whether the sign-out was asked for or forced.

Where the bytes live. The store in 0040, keyed under #41, with 0068 placing every
queued action inside its personal data list.

## Why this is written down before the code

The queue is built by whoever first needs it, which is progress reporting, and a
queue built for progress reporting is a queue with one kind of entry in it. The
ordering rule then comes from what positions need, the coalescing from what
scrubbing needs, and the bound from nothing at all, because a position queue never
looked like it needed one. Every later caller inherits those three answers without
the argument that produced them.

Two of the four decisions here are the kind that cannot be corrected afterwards.
An action expressed as a delta is one that has already been counted twice on
somebody's server by the time anybody notices, and no change to the core removes it
from there. An unbounded queue on a device left offline is discovered as a storage
problem on the day it is full, at which point whatever is done about it discards
somebody's actions under time pressure with no rule written down.

The third, ordering by a clock rather than a counter, produces a reordering that
nothing reports and that looks like the server being wrong.

None of this has happened here, because there is no queue in this tree.

## Alternatives, and what each cost

Writing directly when the server is reachable and queueing only when it is not.
The obvious shape, and the direct path is simpler at every call site. It costs one
behaviour: two paths that are exercised by different tests, disagreeing about
order and about what a caller is told, with the disagreement reachable only on a
device whose connectivity changed mid-write.

Coalescing at drain rather than at enqueue. It keeps the whole history, which is
strictly more information, and the drain gets to decide what to do with it. It
costs the bound, since the queue then grows with activity rather than with breadth,
and the person who hits it is the one who used the application most.

Ordering by a timestamp. It is what everybody reaches for, it survives a restart
with no counter to persist, and it sorts across two queues if there ever are two.
0102 already says what it costs: a corrected clock, a suspension, and a television
that believes it is 1970 each reorder somebody's actions, and nothing in the result
says a clock was involved.

Expiring queued actions past some age. It bounds the queue by time as well as by
count and it avoids delivering something very old. It costs a person's actions,
silently, which is what this issue exists to prevent, and it is the version of that
failure that looks most like good housekeeping.

An unbounded queue. Nothing is ever dropped and the honest report is never needed.
It costs the device, and the failure arrives as a store that cannot be written on
a device that has been offline for a month, at the moment when the queue is most
valuable.

Refusing the newest entry at the bound instead of dropping the oldest. It preserves
the earliest record of what somebody did, which is arguably what they would want
kept. It refuses to record what a person just did while the application is in front
of them, which is the version of the failure they will notice.

Delivering with a device-generated identity per action and relying on the server to
reject a repeat. It is the general answer to repeated delivery and it does not
require actions to be assertions. It costs a promise from every server line that it
will honour such an identity, which #10 would have to establish and which an older
line will not have.

## What would reverse this

An action the core must offer cannot be expressed as an assertion of state, named
with the endpoint that only accumulates. The rule above then excludes something
real rather than something hypothetical, and the replacement says what a queue of
non-repeatable actions does instead.

The bound is measured against a real device and found to be reached in ordinary
use, on the harness in #65 rather than estimated. The number moves, and a record
that only moves a number supersedes this one so the reasoning stays beside what
replaced it.

#59 decides a reconciliation rule that needs the history this record coalesces
away, for instance because the server's own movement has to be compared against
more than the last thing the device recorded. Then coalescing at enqueue is wrong
and the replacement says what is kept.

A drop is observed in the field, reported through #100, on a queue that had not
been offline for anything like a month. That is evidence the coalescing rule is not
doing what this record claims, since the bound is defended by it, and the
replacement says what is actually accumulating.
