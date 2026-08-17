# 0031. Quick Connect, its poll, and its three endings

Date: 2026-08-17

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #31

## The decision

Where the configured server offers Quick Connect, the core starts one exchange
per call, hands the client the code the server issued and nothing else the
exchange needs, asks that server about the exchange every five seconds on the
steady clock rather than on a backoff, sets no total limit of its own, and ends
the call in exactly one of four ways: a session, a denial, an expiry, or the
caller's own cancellation, where a denial and an expiry are answers rather than
failures so that a client can show three different things without the vocabulary
in 0004 growing by two.

## Knowing that this is the route

The core asks the configured server and never guesses, which is the rule 0032
already states for its own route and is repeated here only in what it decides,
not in why.

Where the server states that it offers this route, the core reports it and a
client offers it. Where it states nothing, or states that it is off, the call is
`capability-absent` from 0004, carrying the capability name out of the set #10
fixes. That kind is the answer to the call that would have started an exchange,
so a client that asked is told before a person is shown a code that nothing will
ever approve.

Which endpoints carry any of this is #10's, and this record names none of them.
The shape is the one 0005 fixed: begin an exchange, receive a code for the client
to show, wait for the server to say the exchange was approved elsewhere.

What the core never does is read a failure as evidence about the route. A 404 on
the call that starts an exchange is already `capability-absent` under 0004's own
table where the interface says the resource should exist, and that is the whole
of the inference. A refused password on the other route says nothing about this
one.

## What the client is handed, and what stays inside

The client is handed the code the server issued, and that is the whole of what
crosses the boundary while an exchange is open. The code exists to be read off a
screen and repeated somewhere else, so it is the one value on this route that has
to leave.

Where the server issues a second value alongside the code, one the core presents
when it asks about that exchange, that value stays inside the core. It is held in
memory for the length of the call, it is never written through the store in 0033
because it does not outlive the process, it never reaches the cache, and it is
excluded from a diagnostic event by 0071's default for a field nobody classified.
A client has nothing to do with it: the only thing a client shows is the code, and
the only thing it waits on is the call it already holds.

The code itself is excluded from a diagnostic event on the same default. It is
not a credential and it is on a television screen in somebody's living room, so
the argument for excluding it is not confidentiality. It is that an event
carrying it is an event that pairs one person's pending sign-in with the moment
they made it, on a route whose whole shape is that the core cannot see where the
approving happens.

Nothing about this route reaches a second host. Every request it makes goes to
the origin 0028 resolved, which is 0069's set unchanged, and there is no address
handed to a browser here at all. That is the one structural difference from 0032
worth naming, and it is why this route needs no value tying an answer to an
attempt: no answer arrives through the client, so there is nothing arriving from
outside the process to match against something the core started.

## The poll, and the number this record owns

Five seconds between one question and the next, measured on the steady clock 0102
fixes for an interval inside one run. Fixed, not doubling, and not drawn from a
range.

The number is chosen and not measured, and this record says so rather than
implying a run behind it. What it is chosen against is a person: the delay
between somebody approving on a second device and the television in front of them
moving on is at most one interval, and five seconds is short enough to read as
the screen responding rather than as the screen being stuck. Against that sits
what it costs the operator, which is one request every five seconds to the
operator's own machine for as long as a sign-in screen is open, on a route where
0005 has already refused to bound how long that is.

It does not back off, and that is the part most likely to be changed by somebody
being careful. 0038's doubling range is for a request that failed and might
succeed a moment later, and 0045's doubling schedule is for a server that is
gone. A server answering that an exchange is still pending is a server answering
normally. Backing off makes the person who approved at the ninth minute wait
longer than the person who approved at the first, for no reason either of them
could discover, and it does it by treating a healthy answer as a failure.

The interval is the core's and not the client's, which is what #31 asks for. A
client that could set it would set it, eleven of them would set eleven values,
and the operator's server would carry whichever one the most impatient client
author chose.

## The four endings

Approved is a session, established the way 0005 fixes for all three routes: a
token, an account identifier, and whatever the server said about validity. The
token goes to the store in 0033 as any other, and nothing after that point knows
which route produced it.

Denied is an answer. The exchange ended because somebody refused it, which is the
route working, and a person who taps the wrong button on their phone has not met
a failure of the core, of the network or of the server.

Expired is an answer. The code stopped being approvable because the server's own
limit passed. That limit is the server's and this record neither reads it nor
sets one beside it.

Cancelled is 0004's `cancelled`, delivered when the caller stops the call. The
poll stops with it, and the stopping is immediate in the sense 0009 already fixes:
cancelling is a call that cannot wait, so a client is not waiting on a cancel
while a poll it no longer wants is still in flight. A core stopping under 0115
ends an open call the same way.

Denied and expired being answers rather than kinds is the decision in this
section, and the alternative is what makes it worth stating. Two more kinds is a
sixteenth and a seventeenth, which 0004 prices as a change to that record and to
every client, for two conditions that are not failures. One shared kind is worse:
`request-refused` would carry both, a client could not tell them apart, and the
three different things #31 asks a client to show would be two. So the call that
starts an exchange ends in a value with three states, and the failure vocabulary
is reached only when something actually failed. 0055 draws the same line from the
other side for an image that is absent rather than refused.

## A poll that fails is not an ending

A poll is an ordinary request. It carries 0007's deadline, 0038 decides whether
it is retried and how long it waits, and 0069's destination set applies with no
exception. This record adds no number to any of those.

What it decides is what a poll that fails in the end does to the exchange, which
is nothing. The attempt stays open and the next question goes out on the ordinary
cadence. A person holding a phone in a corridor with one bar should not be signed
out of a sign-in because one request timed out, and the four endings above are
the only things that end the call.

The client is not silent while that happens. The failure is a diagnostic event
under 0100, which is 0005's rule that the client hears nothing on this route
except through diagnostics, and 0007's four states already describe a server that
has gone quiet without any of this having to say it again.

The cost of that is one case, and it is worth naming rather than leaving for
somebody to find. Where the server is gone for good, the core cannot learn that
the code expired, because the only party who knows is not answering. The call
then sits polling until the caller ends it. That is the same unbounded wait 0005
already accepted for this route, arriving from the other direction, and the
alternative is a total limit the core invents, which 0005 refuses and 0032 prices
again for its own route.

## Why this is written down before the code

Three of the properties above are the kind that are wrong in a way nothing
reports.

The backoff is the first. Adding one is a small, confident edit that makes a
graph look better and is defended by every instinct about polling. It is invisible
in a test that approves the exchange immediately, which is the test somebody
writes, and it only shows up as a television that takes half a minute to notice an
approval that happened at once.

The endings are the second. The shortest code that compiles maps a denial onto
whatever failure is nearest, because the call already has a failure path and does
not yet have a three-state answer. That reads as correct, it passes a test that
asserts the call did not succeed, and it arrives at eleven clients as one sentence
where three were owed.

The exchange value is the third. A value the server issued for the core to
present is exactly the kind of thing that ends up handed to the client, because
the client is holding the code already and one more field is convenient. Once one
client has it, the property that only the code leaves is not recoverable by
changing the core.

None of the three has happened here, because there is no code in this tree and no
language in which to write any.

## Alternatives, and what each cost

A total time limit on the exchange, so that the set of open attempts cannot grow.
It bounds something and it sounds careful. The number would be about how long a
person takes to find their phone, 0005 already refuses it for this route by name,
and 0032 refuses the same number for its own route on the same reasoning. What
would actually be bounded is the caller's call, which the caller already controls.

A poll interval each client sets. It lets a television poll slowly and a phone
poll quickly, which is a real difference between the two. It puts one number in
eleven places, it is the number an operator would have to reason about across
eleven clients when their server gets busy, and #31 asks for the opposite.

A doubling interval, so that a long wait costs the server less. It reduces load
in exactly the case where the person has walked away, which is the case where
nobody is waiting for anything. It pays for that by making the answer slowest at
the moment it is most likely to arrive, since a person who has been at it for two
minutes is a person who is still trying.

Waiting on a connection the server holds open instead of asking repeatedly. It is
the shape that gives the fastest answer with the least traffic, and it needs the
server to offer it. Whether any supported line does is #10's, nothing in this tree
answers it, and a route built on it would have to carry the polling route anyway
for every server that does not.

Two more kinds in 0004 for denied and expired. It says what happened, in the
vocabulary a client already handles exhaustively. It costs a change to 0004 and to
every client for two outcomes that are not failures, and it puts an ordinary
answer, somebody deciding not to sign in, into the set a client shows errors from.

## What would reverse this

A supported server line refuses polls at this cadence, arriving as `server-busy`
from 0004 and read off the diagnostic events in #100 rather than assumed. The
fixed five seconds is then a number that fights a server's own rate limit, and the
replacement takes the interval from what the server states rather than from this
record.

A supported server line offers a way to wait for the answer without asking
repeatedly. The poll is then the fallback rather than the route, and this record is
superseded by one that says which servers get which and what a client is told about
the difference.

The server's own expiry turns out not to be observable, because the answer for an
expired exchange is indistinguishable from the answer for one that was never
started. Expired then cannot be one of three endings, and the replacement decides
between a limit the core imposes after all and a client that is told two states
instead of three.

#10 answers that whether this route is offered cannot be read before the exchange
is started. The `capability-absent` answer above is then produced by the first
call rather than in front of it, which is the same reading 0004's own reversal
condition names for its two 404 rows, and this record is superseded by one that
says what a person is shown while that call is in flight.
