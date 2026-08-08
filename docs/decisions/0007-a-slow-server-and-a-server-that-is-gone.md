# 0007. A slow server and a server that is gone

Date: 2026-08-08

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #7

## The decision

Slow and absent are separate conditions with separate recoveries, so the core
reports four distinct states rather than one pending state, it reports them
progressively as a request ages rather than once at the end, and it never makes a
client wait for the network to find out what it could already have been given from
cache.

## The four states

`fresh` means an answer arrived from the server and is current.

`late` means a request is still outstanding and has passed the point at which
waiting for it can still meet the published budget. The request has not failed.

`abandoned` means the core stopped waiting for that request. It says nothing about
the server beyond that one request.

`unreachable` means the server, not one request. Nothing will be attempted against
it until the recovery below says otherwise.

A client that treats `late` and `abandoned` as the same thing shows a person a
spinner and then an error, which is the failure this record exists to prevent. A
client that treats `abandoned` and `unreachable` as the same thing tells a person
their server is gone because one endpoint was slow.

## The thresholds, their values, and their reasons

### `late` at 400 ms

Derived from the cold-start budget rather than chosen for its shape. #62 publishes
1.2 seconds from a cold start to the first usable tile. If a client is going to
commit to what the cache already gave it instead of holding the screen for the
network, it needs that decision made with enough of the budget left to read, decode
and draw. The core cannot know how long a client's draw takes on a television, so
it takes one third of the budget for its own attempt and leaves two thirds. One
third of 1.2 seconds is 400 ms.

The split is a choice, not a measurement. Nothing in this repository has been
measured, because there is no code to measure. #65 is the harness that replaces the
choice with a number, and #62 is where the core's own share is established.

### `abandoned` at 5 seconds

An answer that arrives after 5 seconds has missed every number this board
publishes: it is more than four times the 1.2 seconds in #62 and more than twice
the 2 seconds in #63. The screen that asked for it was given the cached answer
long before, so the arriving bytes update something nobody is looking at.

The reason to abandon rather than to keep waiting is not the answer, it is the
slot. An outstanding request holds a connection out of the limit the transport in
#27 sets, and a queue of them behind one slow server is how a client loses the
ability to make the request that would have succeeded. Whether the abandoned
request is attempted again, and after how long, is #38.

### `unreachable` immediately, on evidence of absence

Three outcomes are evidence that the server is not there, and none of them needs a
threshold because none of them involves waiting: the name did not resolve, the
connection was refused, or there is no route to the address. The core reports
`unreachable` at once on any of the three.

A certificate the core will not accept is not one of them. That is a server that
answered, and it is #29's outcome with its own identity in #4. Reporting it as
unreachable would send a person looking for a network problem that does not exist.

### `unreachable` after two consecutive abandonments

Where the only evidence is that requests ran out of time, one abandonment is a fact
about a request and two consecutive ones with no success in between are a fact
about the server. At 5 seconds each that is 10 seconds before the core will say
what a person watching the screen worked out earlier, and a third would make it 15.
Two is where the accounting stops being useful and starts being ceremony.

A success against the same server at any point resets the count, because a server
that answered is not gone.

## What the core reports, and when

At the moment a request is made: which session it belongs to, whether the core
already has a cached answer for it, and the age of that answer if it does. This is
reported before any byte leaves the machine, so a client has something truthful to
draw at nought.

At 400 ms, if the request is still outstanding: `late`, with the age of the cached
answer if there is one. Nothing about the request has gone wrong and the core says
nothing that implies it has.

At 5 seconds: `abandoned`, with the vocabulary member from #4 that names a request
that ran out of time, and with whatever the core is still able to serve from cache.

On evidence of absence, or on a second consecutive abandonment: `unreachable`, and
from that point the behaviour in #45 applies.

When a server answers again after `unreachable`: the core reports the recovery
itself, unprompted. A client does not poll and does not ask a person to pull a
screen down to find out.

Every one of these is a report about a request or a server. None of them is a
sentence for a person to read, which is the boundary in record 0003.

## What is served from cache while a request is outstanding

Everything the cache holds for that session, marked with its age. Serving it is the
default rather than something a client opts into, because #46 requires the core to
serve something before the first network reply and a cache that only answers after
the network has failed cannot do that.

How a client asks for something other than the default. Three requests, and the
core distinguishes them:

Whatever is available soonest. The core answers from cache if it has anything,
marks the age, and reports the network states above as the request ages. This is
the request a screen makes.

The cached answer only, with no network attempt. The core answers from cache or
reports that it has nothing. This is the request something makes when it already
knows the server is unreachable and wants no further attempts.

The server's answer only, with cache used for nothing. The core makes the request
and reports the states above, and never substitutes a cached answer. This is the
request something makes before an action that must not be taken against stale
information.

A client that never chooses gets the first. Whether a cached entry may be served at
all, and what its age means, is #43; this record only fixes that the choice exists
and that the default is the one that meets #62.

## Recovery, per condition

`late` recovers by the answer arriving. The core does nothing, retries nothing, and
reports `fresh` when it lands. There is nothing to recover from.

`abandoned` is #38's. The core decides on its own whether to attempt the request
again, and the decision is per request rather than per server, because one endpoint
being slow is not the others being slow.

`unreachable` is #45's, and the core drives it without a client. It attempts the
server again on its own bounded schedule, reports the recovery when one succeeds,
and stops attempting when the bound is reached rather than continuing indefinitely
against an address that is not answering. The clock that schedule is measured
against is #102's to name.

What needs a person, and it is a short list. A server whose address is wrong. A
certificate the core will not accept, in #29. A session that could not be renewed,
in record 0005. Nothing else on this path asks for anybody: a slow server and a server
that is gone are both normal, and normal conditions do not interrupt somebody.

## What a client can put on screen, derived

The core supplies no wording. What follows is what a client can know at each
moment, which is what it needs in order to write its own.

At 0.3 seconds. The request was made, and either there is a cached answer with a
known age or there is nothing cached. No threshold has passed. A client with a
cached answer has a real library on screen and can say how old it is. A client with
nothing cached knows only that it asked, which is the one case where a person is
correctly shown that something is happening.

At 1.2 seconds. `late` was reported 800 ms ago. A client showing a cached answer
has no reason to change what is on screen and every reason not to, because
replacing a real library with an indicator loses information. A client with nothing
cached now knows the published budget has been missed, and that is the moment it
can say the server is slow rather than continuing to imply that an answer is
imminent.

At 5 seconds. `abandoned`, with an error identity and the age of anything cached. A
client with a cached answer keeps it and can now say plainly that this is not
current. A client with nothing cached has an empty result and a named reason for
it, which is enough to write a sentence that says what happened instead of one that
says an error occurred.

At no answer at all, meaning a refused connection or a name that did not resolve.
`unreachable` from the first attempt, with no waiting. Everything cached is still
served with its age. A client can say the server cannot be reached, can show the
library that was there before, and does not need to offer a way to retry, because
the core is already retrying and will report when it succeeds.

## Why this is written down before the code

Collapsing these states is not a decision anybody makes. It is what happens when
the first call site needs a way to say that something is pending, adds one, and
every later call site uses it. The states are then indistinguishable at the point
where a client would have to tell them apart, and the client's only remaining
option is a timer of its own, which is eleven timers with eleven values.

The thresholds have to be written before the code for a different reason. A
threshold that appears inside an implementation is a number nobody argued for, and
the argument is the useful part: 400 ms is only defensible while the budget it was
divided out of is 1.2 seconds, and a record says so where a constant cannot.

## Alternatives, and what each cost

One pending state and one failure state. The smallest interface, and it is the
failure named in #7: the client cannot tell late from gone, so it shows the same
indicator for both and then the same error for both.

Thresholds supplied by the client rather than decided here. Each client tunes for
its platform, which is a real argument on a television. It also means eleven
answers to when a server is slow, which is the drift this repository exists to
remove, and it makes the number in #62 unmeasurable, because the core's share
depends on a value the core does not know.

No abandonment at all, with the client cancelling when it loses interest. Honest in
that the core never discards an answer somebody might want. It leaves the
connection limit in the hands of whoever forgets to cancel, and forgetting to
cancel is the ordinary case.

Reporting only at the end of a request. Half the interface and none of the
progressive behaviour #44 requires. It also makes the 1.2 second number
unreachable by construction, because nothing can be shown until something has
finished.

Treating a timeout as evidence the server is gone. One threshold instead of two
states, and it declares a server absent on the strength of one slow endpoint, which
then stops the core attempting the endpoints that were working.

## What would reverse this

The number in #62 changes. Every threshold here is divided out of 1.2 seconds, so a
different budget is a different set of values, and the arithmetic is written down
above so that the recalculation is mechanical rather than a fresh argument.

The harness in #65 measures the core's own share and finds that 400 ms leaves less
than a client needs to draw, or more than it needs. That is the measurement this
record is waiting for, and it replaces a chosen value with one.

Two consecutive abandonments turns out to declare a healthy server unreachable in
ordinary use, for instance because one endpoint on a supported server line is
routinely slower than 5 seconds. The count then rises, or the count becomes
per-endpoint, and either way the reason above stops being true and is replaced
rather than adjusted quietly.

The transport in #27 turns out not to have a connection limit worth protecting,
because the platform pools connections in a way that makes an outstanding request
cheap. The reason for abandoning at all would then be gone, and what is left is the
weaker argument about answers nobody is looking at.
