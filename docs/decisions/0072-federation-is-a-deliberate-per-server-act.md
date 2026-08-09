# 0072. Federation is a deliberate per-server act

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #72

## The decision

A second server becomes reachable only when a person adds it themselves, one
server at a time, after being shown what that act will share, and the act is
written down on the device, limited to what was named at the time, and revocable
without touching any other server.

## The four parts of deliberate

`docs/decisions/0068-the-data-locality-position.md` allows exactly one route for
data to reach a second host and calls it deliberate. That word carries the whole
position, so it is defined here as four properties, each of which an
implementation either has or does not have.

Off unless switched on. A configuration nobody edited federates with nothing.
There is no default partner, no server discovered on the network, and no server
that becomes reachable because another server mentioned it. A core that has been
given one server address reaches one host.

Per second server. The act names one server and affects that server alone.
Adding a second partner does not widen the first, and there is no setting whose
value is "federate", only a set of servers each of which was added on its own
occasion. Where the set is empty the feature is absent rather than idle.

Named before shared. What the act will share is enumerated at the moment it is
performed, in terms of the items in the personal data list in 0068, and the
enumeration is what the core is then bound by. A description in general terms is
not this. "Your library will be shared" names nothing a person can check
afterwards, and it is the sentence that makes every later widening invisible.

Reversible. Every act can be undone by the person who performed it, from the
device, without the second server agreeing and without the first server being
reachable. Revocation that requires a working network is revocation that fails
in exactly the situation somebody reaches for it.

## Scope, and why one act cannot stand for another

Federating for one purpose does not federate for another. The enumeration made
at the time of the act is the boundary, and a later feature that wants something
outside it asks again rather than inheriting the earlier answer.

The failure this stops is the one that never looks like a decision. A pairing
taken for a single purpose is the cheapest thing in the world to widen: the
consent already exists, the connection already exists, and the widening is one
field added to a request that was already being sent. Nobody is asked, nothing
in the interface changes, and the person who agreed to one thing is now sharing
another. Each step is small and the sum is a position nobody would have agreed
to as a single question.

So the core carries what was named alongside the partner rather than treating
the partner as a permission. Where a request would carry something the
enumeration does not include, that is a defect and not a case to handle at run
time, in the same way that reaching an unconfigured host is a defect under 0068
rather than a condition to report.

Widening an existing act is a new act. It names what is being added, it is
performed by a person, and it is recorded separately, so that the record below
shows two entries and not one entry that changed shape.

## What the device records

The answer to "what have I shared, and with whom" is a fact held on the device
rather than something reconstructed from memory or from a second server.

Each act writes an entry: which server, when, what was enumerated, and, once it
happens, when it was revoked. Entries are appended and not rewritten, so a
revoked act stays visible as an act that was performed and then revoked. A list
that showed only what is currently active would answer a different and less
useful question, since the person asking has usually just remembered something
they did once.

The entry is readable by the operator through the client, which means the core
exposes it rather than rendering it. The wording belongs to the client for the
same reason the error vocabulary in #4 and the diagnostics in #100 leave wording
to the client.

Where the client supplied no store under #40, the entries live for the process
and are gone afterwards, and the core says so through the same capability that
reports the absent cache. A record of what was shared that quietly disappears is
worse than one that was never promised, so the absence is reported rather than
inferred.

The entry is personal data under 0068 and is keyed the way everything else is
under #41. It says which servers a person uses, which is the first item on that
list.

## Revocation, and what it cannot undo

Revoking an act stops further sharing under it. From the moment the core accepts
the revocation, no request carries anything under that act, whether or not the
second server can be reached, and whether or not it was reached before. This is
the half that is genuinely in the core's hands, and it is where the proof in
this issue's condition lands.

What is already at the second host is not in the core's hands, and the core says
so plainly rather than offering something that sounds better. Data that was sent
was sent. The core does not know what the other host retained, cannot delete it,
and will not claim a deletion it cannot observe. Where a client shows the
operator what revocation does, the sentence it is given to show says that
sharing stops and that recovering what has already gone is a matter between the
operator and whoever runs the other server.

An offer to request deletion from the other server is the tempting alternative
and it is refused here. A request the core cannot verify was honoured is an
assurance about somebody else's host, and the difference between "we asked" and
"it is gone" is invisible in the interface at the moment it matters most.

Revoking one act leaves every other act untouched, including a second act
against the same server. This falls out of the per-server rule rather than
being an extra property, and it is worth stating because the shortest
implementation of revocation is to drop the partner entirely, which is right
only when the partner has one act against it.

## Why this is written down before the code

The direction of this mistake is one-way. A default that shares can be turned
off in a later version and the data it already sent stays sent, on hosts nobody
here controls, belonging to operators who never made a choice. There is no
release that repairs that, which is why the property is decided before the first
line that reaches a second host exists.

There is also a specific failure that this repository is unusually exposed to.
The core is linked into clients written by other people. If the definition of
deliberate is left to them, then the same shared code produces one client that
asks per server and another that offers a single switch, an operator cannot tell
which they installed without reading it, and the position in 0068 becomes a
statement about a library rather than about anything a person actually runs.
Deciding it here is the only version that survives eleven clients.

Written after the code, this becomes an audit. Every place that carries data
towards a partner has to be found and checked against an enumeration that did
not exist when it was written, and nobody can promise that the search was
complete.

## Alternatives, and what each cost

A single federation setting, off by default, covering every partner at once.
Much less to build and much less to explain, and the person who wants two
servers to work together turns it on once. It costs the per-server property
outright: the second partner arrives under a consent given for the first, and
the operator's answer to "what have I shared" becomes a setting's value rather
than a list of acts. It also makes the switch a target, since one line of
configuration then governs every host.

Federation on by default, discoverable, with a way to switch it off. Best for
the person who never opens a settings screen and expects their two servers to
find each other. 0068 already argues why this fails, and the argument is not
repeated here beyond the part specific to this record: with a default there is
no act, so there is nothing to enumerate, nothing to record and nothing to
revoke, and all four properties above collapse together rather than one at a
time.

Consent per request rather than per server. The strongest version of naming what
will be shared, since nothing is ever sent without an answer about that exact
thing. It costs usability so heavily that the real outcome is a person who
approves without reading, which is a worse position than the one it replaces
because it looks like stronger consent. A prompt answered reflexively is a
weaker record of intent than an act performed once and written down.

Delegating the whole question to the second server, which states what it wants
and receives it. This is how a good deal of federation between servers is built,
and it puts the enumeration where the knowledge is. It costs the property that
matters most here, since the set of what is shared would then be decided by a
host the operator does not run, and 0068 already refuses to let a response body
decide what the core does.

Recording only the currently active partners rather than an append-only list.
Smaller, and it answers the question a settings screen asks. It costs the
question people actually arrive with, which is about something they did once and
half remember, and a revoked act would leave no trace that it ever happened.

## What would reverse this

An implementation of the enumeration turns out not to be checkable, because what
a request carries cannot be compared against what was named without duplicating
the whole request surface. The property in that case is prose with no mechanism,
and this record is superseded by one that either states it in a form something
can refuse or says plainly that nothing refuses it.

A person performs the act, gets the four properties, and still cannot answer
what they have shared, observed twice on real use rather than supposed. That is
evidence the record on the device answers the wrong question, and the record is
superseded by one describing the question it should answer.

A second act against a server the operator already federated with is found to be
the normal case rather than the exception, and asking again each time is
ceremony that people learn to click through. The per-purpose boundary is then
doing harm rather than work, and it is replaced by a scheme that keeps the
enumeration while asking less often.

Federation is dropped from this repository's scope. The core reaching one server
at a time is a coherent product, and if no client asks for a second server, this
record covers a feature that does not exist and is superseded by one that says
so.
