# 0069. Every host the core may contact

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #69

## The decision

The set of destinations is exactly the origins of the servers the operator
configured, one entry per configured server and no entry the core adds on its
own; a redirect is followed only where it stays inside the origin that was
already in the set, a redirect out of it is refused before a request is sent
there, and the second host in a delegated sign-in is not in the set because the
core never contacts it.

## The list

One row per configured server, and the whole list is derived rather than written
down, because a list in a document drifts against the thing it describes.

| Destination | Who chose it | What reaches it |
| --- | --- | --- |
| The origin of a server the operator configured | The operator, by typing its address | Every request the core makes: sign-in, capability answers, library queries, artwork, playback and progress reports |

That is the whole table, and the number of rows in a running core is the number of
servers the operator added. The first is added at sign-in. A second is the
deliberate act in 0072, performed the same way, by the same person, and it adds a
row rather than widening one.

The set is empty before anything is configured. 0068 already fixes that and it is
what makes the check in #70 unambiguous: any host at all in a run that configured
none is the defect, with no allowance to reason about.

## What an origin is here

The origin is the scheme, the host and the port of the base address 0028 produced
from what the person typed. Not the host alone.

Not the host alone, because a server behind a reverse proxy on one machine may
share that machine with something else on another port, and because 0028 already
fixes that a typed `http://` is honoured rather than upgraded, so scheme is part
of what the operator chose rather than a detail.

A path is not part of the origin. 0028 joins every path to the base by
concatenation, and a server at a sub-path is still one origin.

The comparison is made against the origin as it was resolved on the way in, not
against the address as it was typed, for the same reason 0028 gives: a repaired
address pointing somewhere the person did not name is the failure that record
exists against.

## Redirects

A redirect inside the origin already in the set is followed. That case is
ordinary: a server correcting a path, a trailing slash, or a version prefix.

A redirect to anywhere else is refused, and it is refused before a request is sent
to the new location. Nothing is sent there: not a credential, not a device
identity, not the query. This is the same rule 0029 states for a connection whose
certificate was not accepted, and for the same reason, which is that a failure
discovered after the request went out is a request already delivered.

The reason is 0101's, arriving here as a consequence rather than as a new
argument. The operator is trusted for exactly one decision, which server the core
may talk to, and a configured host asking the core to go somewhere else does not
add to that set. A server that redirects artwork to a public content network is
sending an address and a user agent to a host the operator did not choose, and on
a self-hosted server the address frequently says where the person lives, which is
the first item in 0068's list.

The refusal is `answer-not-understood` from 0004, and no sixteenth kind is added
for it. 0004's own table already routes a 3xx that surfaces to the caller there,
and its payload shape carries what was being read and where reading stopped, which
is where the refused location is named.

That kind is an imperfect fit and it is better to say so than to grow the
vocabulary. A cross-origin redirect is a shape the core understood and refused
rather than one it could not read, and the word in the kind says the opposite.
What keeps it as the right answer is that the alternative costs a change to every
client for a condition that should never occur against a correctly deployed
server. 0004 already names the measurement that would overturn that:
`answer-not-understood` becoming the kind an operator sees most, read off the
diagnostic events in #100. If cross-origin redirects are what that turns out to
be made of, the vocabulary was too small and this is where it shows.

## Two hosts that are not in the set, and why

An identity provider in a delegated sign-in. The second host is genuinely
involved and the operator's own server named it, which is what makes it worth
naming here rather than leaving to be assumed. It is still not in the set,
because the core never contacts it. 0005 fixes the shape: the core hands back
what the client must open, the platform opens it, the person authenticates
somewhere the core has no visibility into, and the core receives only the result
the server issues at the end. Every request the core makes on that route goes to
the configured server. What the platform's own browser reaches is the platform's
business and the person can see it.

The place artwork actually came from. A server hands on images it fetched
elsewhere, which 0101 calls the standing counterexample to trusting a configured
server's bytes. The core fetches artwork from the configured server and never
from whatever that server fetched it from, so the third host is a fact about the
operator's server rather than a destination. 0068 already says an artwork address
pointing at a third host is not followed, and this record adds only that it is
also not resolved.

## What is refused before it is a connection

Name resolution is part of the set rather than something that happens before it.
A name is resolved only for a host in the set, because a resolution is itself a
request to somebody, it carries the name being looked up, and on a home network
the somebody is frequently the operator's own router and afterwards their
provider. A core that resolved first and checked afterwards would have already
told a third party which server the person uses.

Nothing resolves a name that arrived in a response body. 0068 fixes this and it is
repeated here only because a redirect is the case that looks different and is not:
a `Location` header is a response saying where to go next, and the rule is the
same as for an address in a body.

## What this leaves for other issues

The test that fails when the core reaches a host nobody configured is #70. This
record is its input and does not perform it, so until #70 exists this is a rule
nothing refuses, which is 0068's own sentence about itself and is a statement
about today.

That there is no telemetry, analytics or crash reporting to reach an unconfigured
host with is #73, and it is a different proof over a different subject: this
record bounds where the core may go, and #73 bounds what is in the tree that would
want to go anywhere.

Which certificate is acceptable once the core is at a destination is 0029. A pin
says nothing about the set: it does not add a host, and a host in the set does not
get its certificate accepted.

Adding a second configured server, and what revoking that act does, is 0072.

## Why this is written down before the code

The list has one row, which is what makes it worth writing. A rule that is
obviously satisfied today is a rule nobody notices being broken, and the breakage
does not arrive as somebody deciding to contact a second host. It arrives as a
redirect being followed because following redirects is the default in every HTTP
client anybody would reach for, and the default follows them anywhere.

That is the specific failure this record is against, and it is one line of
configuration on the transport in #27. It looks like nothing, it is invisible in
every test against a fake server that does not redirect, and the first time it
matters is an operator whose server is behind something that redirects artwork to
a content network, at which point the core has been sending an address to a third
party for as long as that deployment has existed.

The second failure is the resolution order. Checking the host after resolving it
reads as equivalent and is not, and the difference is only visible to somebody
watching the network rather than reading the code.

Neither has happened here, because there is no transport in this tree.

## Alternatives, and what each cost

Following redirects anywhere, as an ordinary HTTP client does. Nothing to
implement, and it is what every deployment expects to work. It costs 0068's
position outright, and it costs it silently, since the operator's server is the
thing that asked and nothing in the core would report that it had been asked.

Following redirects anywhere, with a report to the client each time. Honest,
cheap, and it keeps unusual deployments working. It moves a decision 0101 places
with the operator onto a client author writing a handler, and the handler that
gets written is the one that allows it, because the alternative is a deployment
that does not work.

An allowed set the operator can extend, so that a person with a content network in
front of their library can name it. It is the flexible answer and it has a real
use. It costs the property that the set is derived from what was configured, which
is what makes #70's failure unambiguous, and it adds a second kind of
configuration whose wrong value is a data disclosure rather than a broken
connection.

Refusing every redirect, including inside the origin. Simpler still, and there is
nothing to compare. It costs ordinary deployments, since a server correcting a
path is a redirect, and the failure would read as the server being broken.

Comparing hosts rather than origins. Shorter, and it allows a server that
redirects from a port to another port on the same machine, which happens. It costs
the scheme, so an `https` origin redirecting to `http` on the same host would be
followed, which sends the session token in the clear on a rule that was written to
protect the address.

A sixteenth error kind naming a refused redirect exactly. It says what happened
rather than approximately what happened, and a client could show something
specific. 0004 fixes the cost of a sixteenth kind as a change to that record and
to every client, which is deliberately high, and this condition should not occur
against a correctly deployed server at all.

## What would reverse this

A supported server line is found to redirect out of its own origin as part of
normal operation, on a deployment the operator did not misconfigure. Then the
refusal is refusing something ordinary rather than something dangerous, and the
replacement says what is followed and what the operator is told.

`answer-not-understood` turns out to be dominated by refused redirects, measured
on the events in #100. That is 0004's reversal reached through this record, and
the answer is a kind that says what happened rather than a rule change here.

#70 turns out not to be able to observe the set at all, for example because the
chosen toolchain in #11 offers no way to see a connection before it is made. Then
this record is a rule with no possible mechanism rather than one whose mechanism
is pending, which is a different statement and belongs in a record that says so.

The core is asked to fetch something the operator's server genuinely cannot serve,
which is the shape a subtitle or a metadata provider takes if either ever becomes
the core's. Then the set has a second kind of row, whoever chose that destination
has to be named in it, and the table above is the wrong shape rather than merely
short.
