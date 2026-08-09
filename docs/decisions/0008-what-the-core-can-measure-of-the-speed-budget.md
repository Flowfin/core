# 0008. What the core can measure of the speed budget

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #8

## The decision

Of the four published numbers, the core can measure no whole one and claims none:
focus change and dropped frames are client-side by nature and the core offers
nothing in their place, while first usable tile and press play to first frame each
have a core-owned interval with named endpoints, so the core gates a stated share
of those two and a whole number exists only once a client reports its own half in
the shape given here.

## The four numbers, and the honest verdict for each

| Published number | What the core can do |
| --- | --- |
| Focus change under 80 ms | Nothing. No measurement, no component, no timestamp. |
| No dropped frames across 200 tiles at 60 fps | Nothing about the number itself. |
| First usable tile under 1.2 s from a cold start | Measures its own interval, which is part of the number. |
| Press play to first frame under 2 s | Measures its own interval, which is part of the number. |

### Focus change is not the core's, and gets no proxy

Moving focus from one tile to the next is a view-layer event that begins and ends
inside the client. On the path a client should be on, the core is not called at
all: the item is already on screen, and anything the core would be asked for was
asked for during prefetch. An 80 ms budget for something the core does not
participate in is not a budget the core can spend a part of.

No proxy is offered. The tempting one would be a bound on how long a core call
may occupy the calling thread, and it is a real property that this repository
should hold, but it is a different property with a different name and it does not
become this number by being related to it. It belongs to the concurrency model in
#9 and it is measured as itself. A record that offered it here would produce a
green number for focus change on a build where focus change was never timed.

### Dropped frames is not the core's, and gets no proxy either

Frames are produced by the client's rendering loop against a display the core
knows nothing about. Whether one was dropped is a question only the thing driving
that loop can answer, and the core has no access to the vertical sync it would
have to be answered against.

The same refusal applies. The core's decoding work and its thread behaviour can
plausibly cause a dropped frame, and both are measured under #61 as what they are.
Neither is reported as this number.

### First usable tile has a core-owned interval

The whole number runs from a person launching a cold client to the first tile
being drawn with its image. Three parts, and the core owns the middle one.

The client owns process start until the core is created and the first library
query is issued. It also owns everything after the core hands back a decoded
bitmap: laying out the tile, uploading the image and presenting a frame.

The core owns what is between. The interval is opened by the call that issues the
first library query after the core was created, and it is closed by the return of
the first decoded artwork bitmap belonging to an item in that query's answer. Both
endpoints are calls across the core's own public interface, which is what makes
them nameable: two people instrumenting this independently put their marks on the
same two function boundaries and measure the same thing.

Two variants are measured and reported separately, because they are different
questions and one number for both is a number that hides whichever is worse. The
empty-cache variant starts from a store containing nothing for this key space.
The warm-cache variant starts from a store holding a complete previous answer for
the same query, which is #46's path and the one a returning person actually takes.

### Press play to first frame has a core-owned interval

The whole number runs from the press to a frame of the film on screen. The core
owns from the call asking what to play until the handover in #111 returns
something the platform's own player can open. Decoding and presenting the first
frame are outside, for the reason recorded in #112.

The interval is opened by the call that asks the core what to play for a given
item and closed by that call returning the playable handover. It is one call, so
the endpoints need no further definition, and that is deliberate: an interval
spanning two calls has a gap between them that belongs to nobody.

## The named intervals

Each is a span under #61, and each is reported with its own name so that a run
that measured one and not the other cannot read as a run that measured both.

    first-tile.core.cold      first library query issued  ->  first decoded
                              artwork bitmap returned, store empty

    first-tile.core.warm      the same two endpoints, store holding a complete
                              previous answer for the same query

    play.core                 what-to-play call entered  ->  playable handover
                              returned

Inside each, the sub-intervals that say where the time went: the cache read, the
request, the wait for the server, the parse, the artwork fetch and the artwork
decode. These are reported and none of them is gated on its own, because a bound
on a part is a bound that has to be moved every time work legitimately moves
between parts.

Which clock these are measured on is #102 and is not decided here. What is
decided here is that it is one clock for both endpoints of any interval, and that
an interval whose two endpoints came from different clocks is discarded rather
than reported.

## What a client has to report for a whole number to exist

The core's half is not a number anybody published. A whole number exists when a
client sends back its own half, and the report is data with no prose in it,
carrying:

    number          which of the four
    span            the identifier of the core interval this joins to
    client-start    the moment the client began, on its own clock
    client-end      the moment the client considers the thing done
    clock           which clock those two came from
    platform        what it ran on, at whatever granularity the client has
    build           what was running, so a number can be attributed

The join is the span identifier. Without it a client's interval and the core's
interval are two durations that cannot be shown not to overlap, and a sum of them
is a guess. With it, the client's half and the core's half are one timeline and
the gap between them is visible as its own quantity, which is where a surprise
usually is.

The core does not aggregate these, does not store them, and does not send them
anywhere. It defines the shape and nothing else. Where such reports go is a
client's decision, and #73 is what keeps the core from acquiring an opinion about
it.

## What fails a build

Only the two core intervals. Nothing here fails a build on focus change or on
dropped frames, and no build in this repository will ever report a value for
either.

The allocations, as a share of the published number:

    first-tile.core.warm      150 ms   of the 1.2 s number
    first-tile.core.cold      600 ms   of the 1.2 s number
    play.core                 500 ms   of the 2 s number

These are allocations of a published budget, not measurements. Nothing has been
measured: there is no code in this repository to measure and no harness to
measure it with, which is #65. The numbers say how much of each published budget
the core is allowed to spend, chosen so that the client keeps the majority of the
time in both cases, since the client's half contains the display and the decoder
and the core's does not.

The run they are judged against, and the allowance for the machine:

At least twenty timed repetitions per interval, against the fake server in #21 so
that no real network is in the number. The first three are discarded as warm-up.

The build fails when the median of the remaining repetitions exceeds the
allocation. Median rather than worst, because a gate that reddens on the slowest
of twenty runs on a shared build machine reddens on something other than the
change being built, and a gate that reddens on noise is turned off within a month.

The 95th percentile is computed, printed and not gated, along with the spread
between the fastest and slowest repetition. It is not gated because nobody knows
yet what the spread is on the machine the gate will run on, and a threshold
chosen before the spread is known is a threshold chosen from nothing. #66 is where
the gate is built, and it is the place that either adds the second threshold once
there are runs to derive it from or records that it did not.

A run that could not complete twenty repetitions fails rather than reporting the
ones it got. A partial run reported as a number is the failure this repository's
own rules exist against.

## Why this is written down before the code

A number nothing measures is a wish, and the way a wish becomes a claim is not
dishonesty. It is a build that reports three numbers green and says nothing about
the fourth, read by somebody who counted four published numbers and four green
lines. Deciding now that two of the four produce no line at all, ever, is what
keeps that reading from being available.

The second failure is the proxy. Once there is code, somebody wanting a green
line for dropped frames will find something plausible to time, because a
plausible thing to time always exists. It ships, it is reported as the number, and
the actual property is never measured on any platform. This is easier to refuse
before the plausible thing exists.

The third is the endpoints. Two people instrumenting "first usable tile" without
written endpoints will measure two different intervals, and the difference shows
up as an unexplained regression when the second one lands. Naming both endpoints
as calls on the public interface costs nothing today and cannot be retrofitted
onto measurements already taken.

## Alternatives, and what each cost

Claiming all four numbers in the core, with proxies for the two it cannot see.
Four green lines, a complete-looking dashboard, and the parity with the published
budget that somebody will ask for. It costs the meaning of the two proxied
numbers entirely, and it costs them silently, because a proxy that is wrong looks
exactly like a proxy that is right.

Measuring nothing in the core and leaving all four to the clients. Honest in a
different direction, and there is no risk of claiming a number the core cannot
see. It costs the ability to catch a regression in the core before a client
integrates it, which means every regression is found by whichever client happened
to update, and attributed to that client's change.

Gating on the whole published number rather than on the core's share, by having
the harness stand in for a client. The number that is gated is then the number
that was published. It costs the fake client being the thing that is actually
measured, and a fake client is exactly as fast as somebody wrote it, so the gate
would move whenever the harness did.

Gating on the worst repetition rather than the median. Catches the tail, which is
what a person actually experiences on a bad launch. It costs the gate's
credibility on shared hardware, and a gate people disable is worth less than a
looser one they keep.

One first-tile number instead of a cold and a warm variant. One threshold, one
line, less to explain. It costs the visibility of whichever variant is worse,
and the two differ by whether a network round trip is in the interval at all,
which is not a difference a single number can carry.

## What would reverse this

A client is found to have needed the core on the focus-change path, on any
platform. That would mean the boundary in #3 is not where this record assumed it
is, and the first row of the table becomes wrong rather than conservative.

The harness in #65 measures a spread on the gate's own machine wide enough that
the median rule passes changes that a person would notice. The variance allowance
is then the wrong shape, and this record is superseded by one built on the
distribution that was actually observed, with the command that produced it.

Either allocation turns out to be unreachable for a reason that is not a defect,
for instance because a server round trip alone exceeds it on the oldest server
line #1 entry 3 selects. The allocation is then a number chosen against an
assumption that did not hold, and it is re-derived in a new record rather than
quietly raised in this one.

A client-side reporting route lands under #100 that carries whole numbers back
into a build. This record's split between what the core gates and what a client
completes would then be describable end to end, and the gating decision is worth
retaking against a whole number rather than against a share of one.
