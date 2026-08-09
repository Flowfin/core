# 0038. Retry and backoff

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #38

## The decision

One policy governs every request the core makes to a server: three kinds from
the vocabulary in 0004 are retried inside a call and no other kind is, a call
gets at most three attempts inside the same five second deadline record 0007
already sets for it, the wait before each retry is drawn at random from a
doubling range starting at 250 ms, and every one of those quantities is a
duration on the steady clock.

## What is retried, and what is not

0004 answers this for thirteen of the fifteen kinds in its own Retry column, so
this record is mostly reading that column rather than deciding again. What it
adds is the two places where the column is not the whole answer.

Retried inside the call: `timed-out`, `server-busy` and `server-failed`. Each is
a condition that can be different a moment later without anything else changing,
which is what 0004 means by Retry.

Not retried, ever, inside the call: `address-not-usable`,
`certificate-rejected`, `not-permitted`, `not-found`, `request-refused`,
`answer-not-understood`, `capability-absent`, `internal-fault` and `cancelled`.
Repeating any of these produces the same answer, and a retry is then a second
identical failure charged to the caller's deadline.

`not-authenticated` is not retried either, and it is worth separating from the
list above because it looks retryable. A rejected session becomes a valid one
only by being renewed, which is #34's, and a renewal followed by the original
call is not a retry of a failure but a different sequence with a different
proof. Sending the same rejected token again is the thing this record refuses.

`server-unreachable` is the second exception, and it is the one place where this
record and 0004's column read differently. 0004 marks it retryable after a
delay. 0007 already decided that a name that did not resolve, a refused
connection and an absent route are evidence about the server rather than about
the request, and that the core reports `unreachable` at once and attempts
nothing further against that server until the bounded recovery in #45 says
otherwise. So the delay 0004 refers to is #45's schedule, and there is no retry
of that request inside the call. Retrying one anyway would attempt a server the
core has just reported as absent, which is the contradiction the reader of two
records would otherwise have to resolve alone.

`storage-unavailable` is not a request to a server and this policy does not
reach it. What happens when a store the client supplied fails belongs with #40
and #33, and deciding it here would be deciding it in the wrong record.

## The deadline is the call's, not the attempt's

0007 abandons a request at five seconds and reports `abandoned` with an error
identity and whatever the cache still holds. This record puts every attempt and
every wait inside that same five seconds, measured from the moment the caller
made the call.

That is the whole answer to the overall bound #38 asks for. A retry cannot
extend a caller's wait, because the point at which the core says an answer is
not coming is fixed by 0007 and this record spends attempts inside it rather
than after it.

Two consequences that would otherwise be settled by whoever writes the loop.

The 400 ms at which 0007 reports `late` is not restarted by a retry. It is a
statement about how long the caller has been waiting, and a caller who has
waited 400 ms has waited 400 ms whether the core is on its first attempt or its
third. A clock restarted per attempt would let a caller sit past the published
budget while the core reports that nothing has gone wrong.

A call that spends all three attempts and reaches the deadline is one
abandonment. 0007 declares a server unreachable after two consecutive
abandonments, and counting attempts there instead of calls would declare a
healthy server absent on the strength of one slow endpoint on one call.

## The numbers, and the reason for each

Three attempts. One is no retry at all. Two gives a transient failure one chance
to have passed. Three covers the case the second misses, which is a retry that
lands inside the same brief server hiccup that failed the first attempt, since
the first wait is short by design. A fourth attempt has to be paid for out of
the same five seconds, and the budget is more usefully left as room for an
attempt that is actually in flight.

A count is needed alongside the deadline rather than instead of it. A connection
refused in a few milliseconds costs almost none of the deadline, so a deadline
on its own would permit hundreds of attempts against a server that is failing
fast, which is the case where retrying is least useful and most damaging.

250 ms as the first wait, doubling. It is short enough that the retry is still
inside the window a person perceives as one action, and long enough that a
server that dropped one request is not handed the replacement in the same
instant. Doubling rather than a constant wait, because a second failure is
evidence the condition is lasting rather than momentary, and the spacing should
reflect that without anybody having to choose a second number.

No ceiling on the doubling. At three attempts the computed waits are 250 ms and
500 ms and a ceiling would never be reached, so adding one now would be a number
with no argument behind it. The moment the attempt count rises, a ceiling is
owed, and this paragraph is the record that it was left out deliberately rather
than forgotten.

500 ms of remaining deadline as the floor for starting an attempt. An attempt
begun with less than that cannot plausibly complete, and it still takes a
connection out of the limit the transport in #27 holds, which is the resource
0007 abandons requests to protect. So the core stops early rather than starting
something it has already decided not to wait for.

Every number here is chosen and none is measured. There is no code in this
repository to measure, and the same is true of the thresholds in 0007 that these
are fitted around. #65 is the harness that would replace a choice with a number.

## Randomised spacing

The wait before a retry is drawn uniformly at random from zero to the computed
value: zero to 250 ms before the second attempt, zero to 500 ms before the
third.

The failure this is against is specific and is named in #53's wall of two
hundred tiles. Requests that fail together are requests that were issued
together, and a fixed wait moves the whole wall to a later instant without
thinning it, so the server gets the same burst a quarter of a second later, and
the retry of the burst is what turns a server having a bad moment into a server
being held down.

The range starts at zero rather than at half the computed value. Spreading over
the full range is what actually thins a burst, and the objection to it, that an
individual retry can go out almost immediately, does not bite here because the
deadline and the attempt count bound what any one caller can do regardless of
how the draw falls.

The draw is per attempt per caller. Two callers that failed at the same instant
draw independently, which is the property #38 asks a test to prove, and one
caller drawing once for all of its retries would preserve exactly the
correlation the jitter exists to break.

## The retry-after hint

`server-busy` carries a retry-after hint where the server gave one, and 0004
already says the retry waits for the hint rather than for the computed value. A
server refusing load knows more about when it will stop than any schedule here
does.

Two bounds on that, because the hint arrives from outside. A hint longer than
the deadline still remaining ends the call rather than parking the caller, and
the core reports `server-busy` with the hint intact so that whatever decides
what to do next has the same information. A hint that is absent, unreadable or
not a duration is not a hint, and the computed wait is used, which 0004 already
distinguishes with its given-or-assumed flag.

## A retry of something that changed the server

A repeated request that only reads is free to repeat. A repeated request that
changes something on the server is not, and the difference matters at exactly
one moment: a `timed-out` where bytes did reach the server. The core does not
know whether the server acted on them.

0004 already carries what is needed to decide it. Both `timed-out` and
`server-unreachable` carry a flag saying whether anything reached the server,
and it exists to separate a call that certainly did not happen from one that may
have. So a request that changes server state is retried only where that flag
says nothing reached the server. Where it may have, the call ends with the
timeout reported and the decision about repeating it belongs to whatever asked,
which for an action taken while the server was gone is the queue in #47.

This record does not decide which calls change server state. That is a property
of the surface #10 records, and asserting it here would put a list in the wrong
place and let it drift.

## The clock

Every quantity in this record is a duration between two events inside one run:
the deadline, the waits, and the floor for starting an attempt. Record 0102 puts
those on the steady clock, and the reason applies directly here, since an
interval taken across a wall clock correction is either instant or a month.

The steady clock rather than the elapsed one. A device that suspended between
two attempts has no attempt in flight and nothing waiting on it, and counting
the sleep would turn every wake into an immediate retry of everything that was
outstanding, which is the burst this record spends a whole section thinning.

## Why this is written down before the code

A retry policy is the clearest case of a decision that gets made by accident. It
is never designed; it appears the first time one call site needs to cope with a
flaky endpoint, and the loop written that afternoon is copied to the next site
because it is already there. The values in it were typed rather than argued, and
by the time anybody looks there are several of them with different values and no
way to tell which was intended.

The cost of that lands on somebody else's machine. Eleven clients each retrying
on their own terms turns a server having a bad minute into a server being
hammered by every device in a house, and the operator sees load rather than a
bug, so the report that arrives is about the server rather than about the
clients.

Writing it before the code also keeps it inside 0007's arithmetic. The five
seconds and the 400 ms there are divided out of the published budget in #62, and
a retry loop that appeared later would have been written against a per-attempt
timeout, which is the shape that quietly triples a caller's wait while every
individual number still looks right.

## Alternatives, and what each cost

No retry at all, with every failure reported to the caller. The simplest thing
that can be written down, and it makes the core's behaviour trivially
predictable. It costs the ordinary case: a single dropped request on a home
network becomes a visible failure, and every caller that cares then writes its
own retry, which is eleven policies arrived at by the route this record exists
to close.

A per-attempt timeout with retries after it. The common shape, and each attempt
gets a fair chance rather than inheriting whatever is left. It costs the bound
outright. Three attempts at five seconds each is a caller waiting fifteen
seconds while 0007 says the answer should have been given up on at five, and
nothing in the numbers looks wrong while it happens.

An unbounded exponential backoff with a long ceiling, of the kind a background
synchroniser uses. Right for work nobody is waiting for, and it is close to what
#45 needs for a server that is gone. It is wrong here because every request this
policy covers has a caller attached to it, and a caller cannot be kept past the
point where 0007 says an answer is not coming.

A circuit breaker per server, opening after a number of failures and refusing
calls while open. It protects a struggling server better than jitter does,
because it removes the load rather than spreading it. Most of what it offers is
already in 0007, which stops attempting a server after two consecutive
abandonments and hands recovery to #45, so adding a breaker would be a second
mechanism deciding the same thing with its own counters and its own reset rule.
If the failure it prevents turns up anyway, it belongs in #45 with that state
rather than in the per-call policy here.

A retry budget, where retries are allowed only while they stay under a fraction
of successful traffic. It is the strongest answer to a retry storm, since it
cannot amplify load beyond a stated ratio however many callers fail at once. It
costs a store of recent outcomes per server and a ratio nobody here can pick
without traffic to look at, and the storm it defends against is bounded already
by three attempts inside five seconds.

Client-supplied numbers, tuned per platform. A real argument on a television,
and it is the same argument 0007 rejected for its thresholds. It costs the drift
this repository exists to remove, and it makes the numbers in #62 and #63
unmeasurable, since the core's share would depend on values the core does not
know.

## What would reverse this

The five seconds in 0007 changes. Everything here is fitted inside it, so a
different deadline is a different attempt count and different waits, and the
arithmetic above is written so the refitting is mechanical.

The harness in #65 measures a real round trip against a supported server and
finds that three attempts inside five seconds cannot fit one, or that they fit
many more. That is the measurement this record is waiting for and it replaces
the chosen numbers with derived ones.

The attempt count rises above three for any reason. The missing ceiling on the
doubling then has to be decided, and this record is superseded rather than
edited, because the paragraph that argues the ceiling away stops being true.

Two callers are observed retrying together in ordinary use despite the jitter,
for instance because they are not independent, and the spreading has to move
from the individual call to something that sees the whole wall. That is a
different mechanism, not a different range, and it supersedes this.

A server line is found that answers `server-busy` with hints routinely longer
than the remaining deadline, so that the bound above ends nearly every call
under load. The hint handling then needs its own answer rather than a bound
borrowed from the deadline.
