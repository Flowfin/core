# 0034. Renewal, and the token generation a rejection is answered against

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #34

## The decision

Every token a session has held carries a generation number, a rejection is acted
on only where it names the generation the session currently holds and otherwise
joins the renewal that already replaced it, so a session has at most one renewal
in flight however many calls were rejected together; a renewal is attempted once
per rejection and never in a loop, a rejected call is retried exactly once against
the new token and a rejection of that retry starts nothing further, and a renewal
that fails because the server refused it signs the session out while a renewal
that fails because no server answered leaves the session and its token exactly as
they were.

## What is already decided elsewhere

0005 fixes the direction: the ground truth is a server rejecting a request, and
renewing ahead of a stated expiry is an optimisation on top of that path rather
than a replacement for it. It also fixes the order on a rejection, which is stop
sending on that session, attempt renewal once, retry the rejected call on success,
and on failure discard the token, move the session to signed-out, tell the client
through 0100 and map the original call to `not-authenticated`.

0102 fixes that a token is never refused on the device's own reading of a clock,
that the stated expiry is a hint used only for scheduling, and that the renewal
timer is an interval on `elapsed` so a device that slept through the moment owes
one renewal on waking rather than one per interval it slept through.

0004 fixes the kind and its payload: `not-authenticated`, carrying whether a token
was presented and rejected or whether there was none to present.

This record adds what none of them says: how many renewals happen when twenty
calls are rejected in the same instant, what happens to each of those calls, and
what a renewal that could not be attempted at all does to the session.

## Telling the three cases apart

Three conditions arrive at the same moment in a person's day and have to produce
different behaviour, and 0004 already separates them by kind rather than by
inspection.

A token presented and rejected is `not-authenticated` with the payload saying a
token was presented. This is the only condition that starts a renewal.

A wrong password is `not-authenticated` with the payload saying no token was
presented, because at sign-in there is none. Nothing is renewed: there is no token
to renew and the person is already in front of the thing that asks for one.

An unreachable or slow server is `server-unreachable` or `timed-out`, which are
different kinds entirely. A server that did not answer has said nothing about the
token, and treating silence as a rejection is how a home connection dropping for
ten seconds signs somebody out of a television.

The distinction is carried by the kind and its payload rather than by a status
code, so nothing in the renewal path reads HTTP. That is 0004's mapping doing the
work it exists for.

## The generation, and why a counter rather than a lock

A session holds a generation number alongside its token. It starts at one when the
session is acquired and increases by one every time a token replaces another.
Every request the core sends records the generation of the token it went out
under.

A rejection is answered against the generation it names.

Where the rejection names the generation the session currently holds, the token
that was rejected is the token the session still has, so a renewal is started and
the rejected call waits for it.

Where the rejection names an earlier generation, the token that was rejected has
already been replaced. No renewal is started. The call is retried once against the
current token, which is the retry 0005 already allows it, and if that is rejected
in turn the paragraph below applies.

Twenty tile requests rejected in the same instant all name the same generation, so
the first starts a renewal and the other nineteen find one in flight and wait for
its outcome. This is the property #34 asks for, and the reason it is a generation
rather than a flag saying a renewal is running is that a flag answers the question
"is one running now", which is the wrong question. The right one is "has the token
I was rejected under already been replaced", and a call whose rejection arrives
after a renewal has completed needs the second answer rather than the first. A
flag would start a second renewal for it.

The counter is per session. Two sessions renewing at once are two renewals, which
is correct: they are different servers, different accounts or different devices,
and 0005 refuses an ambient current session precisely so that one cannot be
answered with the other's token.

## What happens to each call that was in flight

Every call that was outstanding when the token died ends in exactly one of three
outcomes, and none of them is an empty answer.

Renewal succeeded and the call was retried against the new token, once. Its
outcome is whatever the retry produced, which may itself be a failure of some
other kind, and the caller is not told that a renewal happened. This is the
transparent path 0005 describes, and it is the one that is taken most.

The retry was rejected as well. The call fails with `not-authenticated`, and no
second renewal is started. A token issued seconds ago and immediately refused is
not a token that will be fixed by asking for a third one, and the loop this
refuses is the shape that gets a device's address blocked at an authentication
endpoint.

Renewal was not possible or was refused. The call fails with
`not-authenticated`, the session is signed out, and the section below says which
of those two cases happened.

A cancelled call is none of the three. 0009 already fixes that no outcome is
delivered for a call the caller cancelled, and a renewal in flight is not
cancelled when the call that started it is, because nineteen other calls may be
waiting on it. The renewal is the session's rather than the call's.

Nothing anywhere in this path returns a result with no items in it. An empty
library and a library nobody was allowed to read are the two answers this is
between, and a cache read that finds nothing is `absent` under 0043 rather than an
empty answer, so there is no route by which a rejected call becomes one.

## A refusal and a silence are not the same failure

This is the part that is decided wrongly by reflex, because both are a renewal
that did not produce a token.

The server refused the renewal. It answered, and the answer was that this session
is over. The session moves to signed-out, the token is discarded, and 0005's
sequence runs. Asking again would produce the same answer.

Nothing answered the renewal, or it timed out. The core learned nothing about the
token. The session stays exactly as it was, the token is not discarded, the
generation does not move, and the calls waiting on the renewal fail with the
transport's own kind rather than with `not-authenticated`. The next call the
client makes tries the same token again, because it may well still be valid and
the only thing that was wrong was the network.

Getting this wrong in the other direction empties a person's session because their
connection dropped, and the damage is not the failed call. It is that the token is
gone, so the recovery is a sign-in on an on-screen keyboard rather than a retry.

The renewal request is an ordinary request in every other respect. 0038 governs it,
so a renewal that times out is attempted at most three times inside its own five
seconds, and those are attempts at one renewal rather than three renewals. The
distinction matters because 0005 says renewal is attempted once and this record has
to say what that counts: it counts renewals, not the transport attempts 0038 makes
underneath one.

## Renewing before the rejection

0102 allows a renewal to be scheduled against the stated expiry and fixes the
clock. What this record adds is when it fires and what it does not do.

The renewal is scheduled for the later of two moments: the stated expiry less five
minutes, and the halfway point of the token's stated lifetime. Five minutes is
chosen because it is longer than any single call the core makes, which 0007 bounds
at five seconds, so a renewal that fires on schedule is not competing with a call a
person is waiting on. The halfway rule exists for the short-lifetime case: a token
stated to last two minutes would otherwise be scheduled for renewal three minutes
before it was issued, and a schedule in the past is a renewal on every call.

The interval is on `elapsed`, per 0102. A device that was asleep past the moment
owes one renewal when it wakes.

A scheduled renewal that fails changes nothing about the session. It is not a
rejection: no call was refused, so there is no evidence the token is dead. The
schedule is not retried on a shorter interval either, because the rejection path
below it is the one that is guaranteed to work, and a failing schedule that keeps
trying is a stream of requests against an authentication endpoint for a session
that is still functioning.

The stated expiry is never read to refuse anything. 0102 fixes that and this record
does not weaken it: a token whose stated expiry passed an hour ago is still sent,
and the server still decides.

## A server with no renewal route

Whether the server offers a way to exchange a live token for a fresh one is a fact
about the server interface, and #10 owns that list. This record is written for both
answers rather than waiting for one.

Where a renewal route exists, everything above is what the core does.

Where none exists, the renewal attempt is not made, the first rejection signs the
session out, and the call fails with `not-authenticated` carrying that a token was
presented. Nothing else changes: the generation still exists, the single-renewal
property is still held trivially, and no call anywhere in the core has a second
shape for the two cases.

What the core does not do in that case is treat the sign-in routes in #30, #31 and
#32 as a renewal. Re-running one of them needs a password, a person at a second
device, or a browser, and doing any of that without the person asking is a sign-in
the person did not make. A session that cannot be renewed ends, and the client is
told through 0100 so that it can ask.

Which of the two the core is dealing with is read from the capability answers 0005
already holds on the session, so it is one answer per server rather than a
discovery at every rejection.

## Why this is written down before the code

The single-renewal property is the one that is written wrongly and looks right. A
flag on the session, set before the renewal and cleared after, passes every test
somebody writes for it, because the test has one call in it. The failure needs a
wall of tiles failing together, and what it produces is not an error: it is twenty
renewals, of which nineteen are refused because the first already invalidated the
token they were made against, and the visible outcome is a session that signs
itself out whenever the network hiccups during a scroll. The generation is barely
more code and it is the version that survives the twentieth caller.

The refusal-and-silence split has the same shape. The natural code has one branch
for "renewal did not return a token", because that is what the call site sees, and
it discards the token there. It is correct on every machine where the server is
reachable, and on a train it deletes the session.

Neither of these can be repaired cheaply once clients exist. Both change what a
client sees at the moment a person is not looking at the application, which is the
moment nobody is filing a report about.

## Alternatives, and what each cost

A lock around renewal, with rejected calls blocking on it. Fewer moving parts than
a counter, and it makes the single-renewal property obvious at the call site. It
costs the late rejection: a call rejected under the old token whose rejection
arrives after the renewal completed finds the lock free and starts a second
renewal, which is the same bug with a smaller window. It also puts a wait on the
waiting lane for every rejected call, which 0009 sizes for connections rather than
for queued waiters.

Renewing on a schedule only, with no rejection path. One code path and no
generation. 0005 already refuses it and gives the reason: a token can be ended at
the server before its stated expiry, and the rejection path then gets written in a
hurry inside whichever call met it first.

Retrying renewal with backoff. It would recover the case where the authentication
endpoint is briefly broken while the rest of the server is fine. 0005 refuses a
loop, and the honest version of this is already present one level down: 0038
retries the renewal request itself, bounded, inside one renewal.

Discarding the token whenever a renewal does not produce one, including on a
timeout. Consistent, and it means the session state is never wrong in the direction
of holding a dead token. It costs a working session every time a network drops
during a renewal, and it costs it silently, which is worse than the state it
protects against: a dead token produces one rejection and recovers, and a discarded
token produces a sign-in screen.

Retrying every in-flight call against the new token as many times as it is
rejected. It would eventually get through if the server were issuing tokens it then
refused. Nothing recovers from that state, and the shape of trying is
indistinguishable from an attack on the endpoint.

Making the caller responsible for renewal, with the core reporting
`not-authenticated` and doing nothing. The smallest core, and every client decides
its own policy. It puts the twenty-renewals bug in eleven places, and it breaks the
guarantee in 0005 that the mid-playback case is invisible to the person watching.

## What would reverse this

A server line issues tokens that a renewal can produce more than one live copy of,
so that generations are not a total order on a session's tokens. The counter is
then describing something that is not true and is superseded by whatever the server
actually offers.

A rejection is observed that names a generation newer than the one the session
holds. That is not possible under this record, so one instance is a defect and two
means the generation is not being carried where this record assumes it is, and the
mechanism is superseded by one that does not depend on the request carrying it.

The five-minute lead is measured against the budget in #62 and found to be either
so early that tokens are renewed several times per use or so late that rejections
land during playback, which is the condition #35 exists for. 0102 already names
that measurement as its own reversal condition, and the number moves in a record
that supersedes this one rather than by an edit here.

#10 answers that capability is probed per call rather than per session, in which
case the answer to whether a renewal route exists is not something the session
holds, and the section above is asking a question at the wrong place.
