# 0028. The address a person typed, and how every path is joined to it

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #28

## The decision

What a person typed is turned into a base address exactly once, on the way in,
by rules written here rather than by whatever a platform's URL type happens to
do; every path the core requests is appended to that base by one routine that
concatenates rather than resolves; and anything the rules do not cover is
`address-not-usable` from 0004, carrying what was typed unmodified, rather than
a repaired address pointing somewhere the person did not name.

## What arrives, and what each shape becomes

Surrounding whitespace is removed before anything else. It arrives from a paste
out of a message or an email and a person cannot see it.

No scheme becomes `https`. Never `http`, because the first thing that travels
over a scheme-less address is a password, and a default that downgrades sends it
in the clear on the one request where it exists. The case a plain default would
serve is a server on a home network, and that case is a certificate rather than a
scheme: what the core does about a certificate an operator would accept is #29.

A scheme the person did type is honoured and never changed. Typing `http://` is a
choice somebody made, and quietly upgrading it produces a connection failure at a
server that is running, which reads as the server being down. Whether an operator
is told anything about that choice is #29's, not this record's.

A scheme that is neither `http` nor `https` is `address-not-usable`. The accepted
set is named rather than left as whatever a parser will accept, which is the same
shape #55 uses for image formats and the same reason: refusing by name rather
than by omission.

A trailing slash is removed. The base is held with no trailing slash and every
join adds exactly one separator, so a double slash never reaches a server. A
server behind a reverse proxy can answer a double slash with a 404 while
answering the single-slash form, and that 404 arrives at sign-in.

A base path is kept. An operator whose server sits at `example.com/jellyfin`
typed the sub-path because it is part of the address, and dropping it is the
failure this record exists for: every request lands on a path that does not
exist, the server answers 404, and 404 at sign-in reads as a wrong password.

A port with no host separator is read as a port. `example.com:8096`, typed
without a scheme, is the shape most likely to be typed on a home network and the
shape most likely to be parsed wrongly, because a parser handed it first sees
`example.com` as a scheme and `8096` as everything after it. The scheme is
therefore supplied before the string is parsed and never after.

A query string or a fragment is removed. They arrive from a paste out of a
browser and neither can mean anything for a base address, but carried into the
base they would ride on every request the core ever makes.

Credentials in the address are refused, not stripped. `https://name:secret@host`
becomes `address-not-usable` naming the part that could not be used. Stripping
them silently would connect somewhere with less than the person supplied and
leave them wondering why sign-in failed. Keeping them would put a password inside
a value that is stored, shown and put in front of an operator, which is what 0006
and 0068 are for.

An IP address is a host like any other, including the bracketed form. `[::1]:8096`
keeps its brackets, because they are what separates the address from the port.

The scheme and the host are lowered in case; the path is not. A host is
case-insensitive and a path is not, and lowering a path would turn a working
sub-path into a 404 on any server whose filesystem cares.

A host with characters outside ASCII is converted to its ASCII form here, once.
Not for tidiness: the list of hosts the core may contact in #69 compares strings,
and a host held in one form and compared in another is a comparison that passes
or fails for a reason nobody intended.

Anything else is `address-not-usable`, carrying the address as it was given,
unmodified, and which part of it could not be used, which is the payload 0004
already fixes for that kind. Nothing is guessed. A repaired address is a request
sent to a destination the person did not name, and the operator in 0101 is
trusted for exactly one decision, which is which server may be talked to.

## Joining, and why it is not URL resolution

Every path the core requests is appended to the base by one routine. The base
path and the request path are concatenated with exactly one separator between
them.

This is deliberately not reference resolution, which is the operation a URL type
offers and the one that looks correct. Resolving a relative reference against a
base replaces everything after the base's last separator, so resolving `Users/Me`
against `https://example.com/jellyfin` produces `https://example.com/Users/Me`
and the sub-path is gone. Every request then goes to a path that does not exist
on a server that is working, which is the 404 that reads as a wrong password.

One routine rather than one per caller, so that the sub-path case is handled once.
A base path is the case that is absent on the machine the code was written on and
present on the operator's, so a second joining site is a site nobody tests.

## Where the rules are applied

Once, where an address enters the core, and what is stored afterwards is the
result. No call site re-reads what a person typed, and nothing downstream sees the
original except the payload of an `address-not-usable`, which carries it so that a
client can show a person what they typed rather than what the core made of it.

## What this does not decide

Whether anything is at that address, and whether what answers proved it is the
machine the address named. #29 for the certificate, and 0004's
`server-unreachable` and `certificate-rejected` for the two answers.

Whether the core may contact that host at all. #69 holds the list and #70 is the
test that refuses a host nobody configured.

How the request is made, its timeouts and its connection reuse. #27.

What a client shows a person for any of this. 0003 and 0004 both give the wording
to the client, and this record produces a kind and a payload rather than a
sentence.

## Why this is written down before the code

The failure is already named in the issue this record comes from: joining a URL
badly produces a 404, and a 404 at sign-in reads as a wrong password. It is
expected rather than observed here, because there is no code in this tree to have
produced it, and it is the ordinary outcome elsewhere.

What makes it worth a record rather than a careful function is that each of these
shapes has a plausible answer that is wrong in a way nothing local reveals. An
`http` default is invisible until somebody is on a network with an observer. A
resolved join is invisible until an operator has a reverse proxy at a sub-path.
A bare `host:port` misparsed as a scheme is invisible until somebody types the
thing most people on a home network type. All three are found by an operator
rather than by whoever wrote the join, and the person reporting it has a wrong
password in front of them and no reason to mention their address.

Written afterwards, the rules would also be written twice. The first caller that
needs a path joins it, the second finds the first inconvenient and joins its own,
and the sub-path case is then correct in one of them.

## Alternatives, and what each cost

Hand the string to whatever URL type the language ships and use the result. The
cheapest possible, and it is what gets written when nobody has decided. It costs
the choice: those types disagree with each other on almost every shape above, they
offer resolution rather than concatenation as the obvious join, and the behaviour
the core ends up with is chosen by a library rather than by anyone. It is not
wrong on the shapes it handles; it is unstated, and an unstated behaviour is one
that changes when the library does.

Require a complete address with a scheme and refuse everything else. One rule,
nothing guessed, and every case above disappears. It costs the moment it lands:
somebody deciding whether this software works at all types what they type
everywhere else and is refused, and a refusal at that moment is not read as
strictness.

Try `https` and fall back to `http` when it fails. Helpful on a home network and
it is what a person would do by hand. It costs a password in the clear after a
failure that may have been a certificate the operator would have accepted under
#29, and it costs an extra request on every first sign-in to learn something the
operator already knows.

Repair more: remove whitespace inside the string, correct a scheme that is nearly
right, drop credentials rather than refusing them. Each one turns a refusal into a
success and feels like kindness. Each one also produces a destination the person
did not name, and the cost of being wrong is a request sent somewhere else, which
is the one failure this record cannot allow itself.

Keep the base exactly as typed and let each caller join as it sees fit. No
normalisation, no lost information. It costs the trailing slash and the sub-path
in every caller rather than in one, and it guarantees that the two are handled
differently in at least one of them.

## What would reverse this

A shape a person plausibly types that these rules turn into the wrong destination
rather than into a refusal, twice. One is a rule written badly and is fixed by a
new record covering it. Two is a sign that normalising on the way in is the wrong
place for this, and the record is superseded by one that says where else it goes.

The probe in #92, or an operator report, shows a server line on which a
scheme-less address has to reach `http` for a first sign-in to be possible at all.
Then the `https` assumption is costing installations rather than protecting them,
and what replaces it is argued in a new record rather than added here as an
exception.

#69 lands a host list compared in a form this normalisation does not produce.
Then the two are settled together, in whichever record survives, because a host
held one way and compared another is worse than either choice made twice.
