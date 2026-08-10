# 0045. The recovery schedule for a server that is gone

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #45

## The decision

While a server is unreachable the core probes it on its own, per server, starting
two seconds after the declaration and doubling to a ceiling of five minutes with
the same random spread 0038 uses, giving up after one hour of continuous
unreachability rather than probing for as long as the process lives; recovery is
reported unprompted and is also readable by a client that was not listening; and
recovery is a report rather than a refresh, so nothing is refetched and nothing is
invalidated by the server coming back.

## What this record adds and what it does not

0007 already fixes the condition and its shape: what makes a server unreachable,
that the state is distinct from every slow state, that the core drives the
recovery itself without a client polling, that it reports the recovery unprompted,
and that it stops at a bound rather than continuing indefinitely. 0038 already
fixes that a request to a server just declared unreachable is not retried inside
the call, and says in as many words that the delay 0004's retry column refers to
for `server-unreachable` is this schedule.

So what is left here is the schedule itself, the bound, what a recovery does
besides being reported, and what a client can do that is cheaper than any
schedule. The clock those waits are on is settled in this issue's own thread and
in 0102's table, and it is the elapsed clock, so a device that slept for an hour
has waited an hour.

## The schedule

Two seconds before the first probe. Not zero, because the core has just concluded
the server is not there and the evidence for that was either an immediate refusal
or two abandoned requests, and neither becomes false within a second. Not longer,
because the commonest cause on a home network is a router that dropped a
connection for a moment, and a person who walks back into range wants their
library, not a wait.

Doubling after each failure, to a ceiling of five minutes. Doubling for 0038's own
reason, that a second failure is evidence the condition is lasting rather than
momentary. A ceiling because doubling without one reaches intervals at which the
core is no longer probing in any useful sense, and five minutes is short enough
that somebody who fixes their server sees the application notice on its own
without opening it.

The same random spread 0038 applies to its retry waits applies here, and it
matters more here than there. Eleven clients on one household network, all
declared unreachable by the same router restart, would otherwise probe in step
forever.

Each of those numbers is chosen rather than measured, and what would produce
measured replacements is the harness in #65 driving an absent server rather than
an estimate.

The schedule is per server. Two configured servers under 0072 are two states, two
schedules and two recoveries, because one being absent says nothing about the
other.

## The bound

One hour of continuous unreachability, after which the core stops probing and
reports that it has stopped.

There is a bound at all because a probe is a network call on a device somebody is
carrying. A core that probes for as long as the process lives keeps a radio busy
on a phone in a bag overnight, for a server that has been off since yesterday
evening, and the cost lands on the person least able to see where it came from.

One hour rather than a count of attempts, because a count is a number whose
meaning changes every time the schedule does, and what is actually being bounded
is how long the core spends on a server that is not answering.

Stopping is not giving up on the session, the queue or the cache. Nothing is
discarded, the queue in 0047 keeps every entry and never expires one, and the
cached entries keep their ages under 0043. The core has stopped asking, and that
is all it has stopped doing.

What restarts it is any request a client makes, and the request a client makes
after a person has pulled a screen down is exactly the signal this bound was
waiting for. So the schedule is what covers the case where nobody is looking, and
a person who is looking is a better trigger than any timer.

## The way back that is cheaper than a schedule

A client may tell the core to attempt now, which resets the schedule and its
bound.

That call exists because the core cannot know what a client knows. 0003 refuses it
platform knowledge, and whether the device just joined a network, came off aeroplane
mode, or woke with a different interface is exactly the kind of platform fact a
client is holding and the core is not. A client that passes it on gets a recovery
in the moment rather than at the next doubling, and it costs one call rather than a
faster schedule that every other client would pay for.

The call is advisory. It does not promise the server is there, it does not fail
when it is not, and the answer arrives the same way every other recovery does.

## What a recovery does, and what it does not

It is reported unprompted, as 0007 requires, and it is reported twice over so that
a client which was not listening is not left behind: as an event through the
interface in 0100, and as a state readable through a call that cannot wait in the
terms of 0009. A client constructed after the recovery, or one whose sink was
absent, can still ask.

The queue in 0047 drains, in its own order, under its own rules. That is the whole
of what the core does with the news.

Nothing is refetched. The core does not reload a library list, does not revalidate
the cache, and does not warm anything, because it does not know what a client is
showing and refetching what nobody is looking at spends the connection of somebody
who has just got it back. A client that is showing a stale list knows it is stale,
because 0043 gives it the age with every read, and it asks again if it wants to.

Nothing is invalidated. A server coming back is not evidence that anything changed
while it was away, and treating it as such would turn every brief outage into a
cold start, which is the failure #46 exists against.

## What this does not decide

What makes a server unreachable, and how that state differs from the slow ones.
0007.

What is retried inside a single call, and the waits between those attempts. 0038.

What is served while the server is gone and what its age means. 0043, and the
cached content served with its age is that record's answer rather than a second
one here.

What happens to somebody's actions in the meantime, their order and their
delivery. 0047.

What a suspend does to this schedule and what a resume assumes. 0115, and the
elapsed clock is why a device that slept through the wait is due a probe on waking
rather than starting the wait again.

## Why this is written down before the code

Two records already point here for a number and neither can supply it. 0007 says
the schedule is bounded without saying by what, and 0038 says the delay for
`server-unreachable` is this schedule, so a reader following either arrives at an
issue rather than a value.

Written from a call site instead, the schedule gets whichever shape the first
implementation of the unreachable state needed, and the two parts that go wrong are
predictable. The bound is the one that is left out, because nothing about an
unbounded probe fails a test: the suite runs for seconds and the defect is a phone
that was warm in the morning. And the recovery grows a refresh, because refetching
on reconnect is what makes a demonstration look good, and it is the behaviour that
turns a thirty second outage into a cold start on a metered connection.

The third, probing in step across eleven clients, is invisible on any single device
and is only ever seen by whoever is looking at the server.

None of this has happened here, because there is nothing in this tree that opens a
connection.

## Alternatives, and what each cost

Probing for as long as the process lives. Nothing to bound, and the core always
notices eventually. It costs a radio on a device in somebody's bag, all night, for
a server that is switched off, and the cost is invisible to whoever would have to
diagnose it.

A fixed interval. One number, easy to reason about, and it recovers a brief outage
as fast as the interval. Any single value is either too eager for a server that has
been off for hours or too slow for a router that blinked, and choosing it means
choosing which of those to be bad at.

Doubling with no ceiling. It is the standard shape and it is cheapest at the tail.
It reaches intervals where the core is not really probing any more, so a server
fixed at midday is noticed in the evening, and the person concludes the application
does not recover at all.

Leaving recovery to the client to poll. It is the least code here and the client
knows when somebody is looking. 0007 already refuses it, and the reason is that
polling is then implemented eleven times, at eleven intervals, and the client that
forgets is the one whose users report that it never comes back.

Refetching the last request on recovery. It makes the return feel immediate and it
is what somebody watching a demonstration expects. It spends the connection of a
person who has just regained one, on data nobody may be looking at, and on a
metered connection it is a cost they did not ask for.

Treating a recovery as a reason to invalidate the cache. It is the safe-sounding
answer, since something may have changed while the server was away. It turns every
outage into a cold start and it discards entries whose freshness rules in 0043
already say when they stop being trusted.

Attempting recovery only when a client says the network changed, with no schedule
at all. Cheapest of all and it is the most accurate signal available. It costs
every client that does not implement it, and it costs the case where the network
never changed and the server was simply restarted.

## What would reverse this

An hour is measured and found either to be spent on servers that came back later,
or to have cost something on a real device, on the harness in #65 rather than
estimated. The number moves, and a record that only moves numbers supersedes this
one so its reasoning stays beside what replaced it.

Two clients are found not to send the attempt-now signal, so that the schedule is
in practice the only route back. That is evidence the seam is in the wrong place,
and the replacement either drops it or makes the schedule what a client can rely on
alone.

#46's cold-start measurement shows that not refetching on recovery leaves a client
showing stale content long enough to be reported as a defect. Then the rule that a
recovery is a report rather than a refresh is wrong at the edges, and what replaces
it says what the core refetches and how it knows anybody wants it.

A platform is found on which a probe cannot be made while the application is in the
background at all, so the schedule stops silently rather than at its bound. The
bound is then describing something that does not happen, and the replacement says
what the core does on that platform instead.
