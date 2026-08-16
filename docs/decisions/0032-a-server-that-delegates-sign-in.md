# 0032. A server that delegates sign-in, and the value that ties an attempt to its answer

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #32

## The decision

Where the configured server says it delegates sign-in, the core hands the client
one address to open and generates for that attempt an unpredictable value it
keeps in memory, ties the attempt to the call the caller made, accepts an answer
only where it names an attempt this process started and has not already finished,
and treats everything that comes back through the client as untrusted input, so
that the second host is one the core never contacts and an answer nobody started
is a refusal rather than a session.

## Knowing that this is the route

The core asks the configured server, and never guesses. Whether a server
delegates is part of the surface #10 owns, so this record is written for both of
that issue's answers rather than waiting for one.

Where the server states its sign-in routes, the core reports what it stated and a
client offers what will actually work. Where it states nothing, the route is
reported as `capability-absent` from 0004, carrying the capability name from
#10's set, rather than being offered and failing at the moment a person presses
it. Where #10 answers that capability is probed per call rather than per session,
the probe is this call and the answer is the same kind, and nothing else in this
record moves.

What the core never does is infer the route from a failure. A password rejected
is `not-authenticated` under 0030 and means the name and the password do not go
together; reading it as evidence that the server wanted a different route is how
a person with a mistyped password is sent to an identity provider they do not
have an account with.

## What the core hands over, and what it never touches

One address, which the client opens with the platform's own browser. 0003 keeps
the core out of drawing anything and 0005 fixes the shape: the person
authenticates somewhere the core has no visibility into, and the core receives
only the result the configured server issues at the end.

The address is the configured server's own. 0069 already states the consequence
and this record is where it is spent: the identity provider is a second host that
is genuinely involved, the operator's own server named it, and it is still not in
the set of destinations because the core never sends anything to it. Every
request this route makes goes to the origin 0028 resolved. What the platform's
browser reaches afterwards is the platform's business and the person can see it
in their own address bar.

Where the route needs an address for the answer to come back to, the client
supplies it at the time of the call and the core carries it through unchanged.
Only the client knows what its platform can receive, and a core that invented one
would be inventing it per platform, which is the platform knowledge #3 keeps out.
It is part of what the attempt is tied to, so an answer arriving for a different
return address is an answer for a different attempt.

The core does not require the platform's own browser and cannot. What it does is
hand over an address rather than a rendered surface, so a client that opens it
inside itself has made that choice visibly. The cost of that choice is the
client's to carry and is worth naming once: a surface the client controls is a
surface the client can read, and what a person types into it is their provider
credential rather than anything this repository ever holds.

## The value that ties an attempt to its answer

Every attempt carries a value the core generates. Not the client, and not the
server, because the whole use of the value is to answer the question "did I start
this", and a value somebody else chose cannot answer it.

It is drawn from the runtime's source of unpredictable bytes, and never from a
clock, a counter, an identifier the core already holds, or anything derived from
the session being established. It is at least 128 bits. Where the means chosen in
#11 offers no such source, it is a fourth seam the client supplies, on the shape
0033 and 0040 already use, rather than a weaker value drawn from what is to hand.
That is a statement about a language nobody has chosen yet, and it is written
here so that the answer is not decided by whichever call site needs bytes first.

It is a secret in the same sense the token is, for as long as the attempt is
open. It is excluded from a diagnostic event by 0071's rule for anything derived
from a credential, it never reaches the cache, and it is never written through
0033, because it does not outlive the process. Comparison is over the whole value
and does not stop at the first byte that differs.

An attempt is the call the caller made. It ends when it is answered, when the
caller cancels it, or when the core stops under 0115, and it ends in exactly one
of those. There is no separate expiry and no number here for how long a person
may take, which is 0005's treatment of the wait in #31 arriving unchanged: a
person authenticating somewhere else is doing something the core cannot see and
has no business timing. The set of open attempts is therefore bounded by the
calls the caller is holding rather than by a rule of the core's, which is the
same bound every other outstanding call in the core has.

An answer is accepted at most once. The attempt is finished by the first answer
that matches it, and a second answer naming the same value finds nothing started.

## What an answer that matches nothing becomes

`not-authenticated` from 0004, with the payload saying there was no token to
present. Nothing is sent to the server, and no session is produced.

That is the same kind a refused password reaches in 0030, and the sameness is
deliberate. What a client does about it is identical in both cases, which is to
offer sign-in again, and the two conditions are not distinguishable to anybody
outside the core in a way that would be safe to publish: a client that lost its
own attempt and an answer somebody else injected produce the same failure, and
telling them apart in the answer would be telling the second one which of the two
it was.

The vocabulary does not grow a kind for it. 0069 took the same decision for a
cross-origin redirect and wrote down why, and the argument is the same one: a
sixteenth kind is a change to 0004 and to every client, deliberately expensive,
and this condition should not occur at all against a correctly deployed server
and an honest answer.

A refusal from the identity provider is a different condition and gets a
different kind, which is what the issue asks for. It is `request-refused`,
carrying the provider's own error identifier as the opaque string 0004 already
fixes for that payload, and never the sentence the provider wrote. It means the
person got as far as the provider and did not come back with an approval, which
is a thing a client says differently from "that did not work, try again", and it
is separable from the case above precisely because it arrives on an attempt the
core did start.

A refusal by the configured server of an answer the core relayed to it is 0004's
mapping unchanged, with no special reading here. A 401 is `not-authenticated`, a
403 is `not-permitted`, and a body that does not parse is
`answer-not-understood`.

## Everything that comes back is untrusted

0101's rule reaches this route twice, and it is worth stating both, because the
second one is the one that reads as safe.

What the client hands back left this process, so it is untrusted whatever it
claims about itself. It is validated for shape and bounded in length before
anything is done with it, and it is not used to build a request until it has
matched an attempt. Matching first is the order that matters: a core that
exchanged the answer with the server and then checked which attempt it belonged
to has already sent an attacker's value to the operator's server.

The token the server issues at the end of the exchange is untrusted in the same
way as any other token, and 0033's validation is what it meets. A value that does
not validate signs the session out rather than being repaired, and this route
adds nothing to that.

The exchange itself is an ordinary request to the configured server. 0007's
deadline, 0038's retry policy and 0069's destination set all apply with no
exception, and this record adds no number of its own.

## Where this route stops

The three routes converge at a token, an account identifier and whatever the
server said about validity, which is 0005's rule. Everything in the core after
that point is written once and does not know which route produced the session. A
session established here renews under 0034 exactly as any other, and 0034 already
states what happens where a renewal is refused: this route is not re-run on a
person's behalf, because re-running it needs a person at a browser.

Signing out of a session established here is #114 under 0114, which ends the
token and the secret and leaves the other sessions alone. Nothing about the
provider is ended by it, because the core has nothing at the provider to end and
never did.

## Why this is written down before the code

Two of the properties here are the kind that get written correctly by accident
and then lost, and one is the kind that never gets written at all.

The order is the first. Checking that an answer belongs to a started attempt
after using it reads identically to checking before, and it is only visible to
somebody reading the sequence rather than the call. The version that gets written
is whichever one the code around it makes shorter, and the wrong one is the one
where the exchange happens first, because that is where the answer's own fields
are already to hand.

The value's source is the second. A value drawn from a clock, a counter or a
hash of the account is unpredictable to a reader and not to anybody else, it
passes every test that checks the value is present and matches, and nothing about
it fails until somebody is looking for it. It is also the line a second author
changes for a reason that sounds good, which is that a derived value is easier to
reproduce in a test.

The third is the second host. There is no request to write for the provider, so
there is nothing to review, and the property that the core never contacts it is
one nothing announces. It stops being true the first time somebody follows a
redirect out of the origin to be helpful, which 0069 already refuses, or the
first time a route is written that fetches something the provider published
because it was convenient. Writing here that the second host stays out of 0069's
set means that addition would be a change to a record rather than a line in a
file.

None of the three has happened here, because there is no code in this tree and no
language in which to write any.

## Alternatives, and what each cost

The client generating the value that ties an attempt to its answer. It is where
the return address comes from, so it is the natural place, and it removes a
requirement on the runtime. It costs the question the value exists to answer: the
core would then be checking that an answer matches something the client said,
which is a check the client could already make and which proves nothing about
whether the core started anything. It also puts the strength of the value in
eleven places instead of one.

A value derived from the session being established, so that no source of
unpredictable bytes is needed at all. Nothing to supply, nothing to seed, and it
is reproducible in a test. Everything it is derived from is known to whoever is
in a position to inject an answer, which is the whole of what the value is
against.

An expiry on an attempt, so that the set of open attempts cannot grow. It bounds
something and it sounds careful. The number would be invented rather than
measured, it is a number about how long a person takes at somebody else's
website, and 0005 already refuses the same number for #31 for the same reason. An
attempt tied to the caller's call is bounded by something the caller already
controls.

Accepting an answer more than once, on the argument that a client may deliver it
twice through a platform that redelivers. It would make a real platform behaviour
harmless. It makes a replayed answer harmless too, in the direction nobody wants,
and a client that can receive an answer twice can also remember that it did.

A sixteenth kind naming an answer that matches no attempt. It says what happened
rather than approximately what happened. 0004 fixes the price of a sixteenth kind
as a change to that record and to every client, and here it would additionally
publish the difference between a client's own defect and an injected answer,
which is the one place that difference should not be published.

Embedding the provider exchange in the core, so that a client gets a session
without opening anything. It removes a seam and it is what a client author would
ask for. It puts a browser inside a core that draws nothing, and it puts the
person's provider credential on the inside of this repository, which is the one
thing this route currently guarantees is never here.

## What would reverse this

A supported server line delegates in a way that requires the core to contact the
provider directly. 0069's set then grows a row with a party who chose it, and
this record is superseded by one that says what reaches that host and what the
operator is told before it does.

The means chosen in #11 turns out to have no source of unpredictable bytes and no
client on any platform can supply one. The seam named above is then a seam nobody
can fill, and the honest replacement says the route cannot offer the property
rather than offering a value that does not have it.

An answer that matches no attempt turns out to be reached routinely by a correct
client on a real platform, for example because the platform redelivers. The
refusal is then refusing something ordinary, and the replacement decides between
a remembered answer and a client obligation, with the replay cost paid
deliberately.

`request-refused` on this route turns out to carry an error identifier that
differs per provider deployment for the same condition, measured on the events in
#100. The opaque passthrough is then giving a client something it cannot match
on, and the replacement decides which conditions are worth naming in the core.
