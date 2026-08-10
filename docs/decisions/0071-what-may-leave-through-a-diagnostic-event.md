# 0071. What may leave through a diagnostic event

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #71

## The decision

Redaction is decided per field name, by the core, before the client's sink is
called, against the personal data list in 0068 rather than against a copy of it;
each named field is excluded outright, reduced to a correlator that means nothing
outside the run that produced it, or carried whole, and which of the three it gets
does not vary with severity; an error returned to a caller in the same process is
not redacted, because the boundary this record is about is the sink and not the
type; and the core retains nothing, so a bundle is the client's to assemble and
the core's half of it is a machine-readable statement of what was excluded and
what was reduced.

## The list is 0068's

The set of fields this rule is about is the one 0068 states, and it is not copied
here. A second copy drifts against the first, and the two documents would then
disagree about whether a field is personal in the one situation where somebody is
reading a document to find out.

0068 also carries the question a contributor asks about a field neither document
names: could two people running the same build against the same server hold
different values here. That question is what makes the list finite without making
it complete, and repeating the list without the question would be copying the
weaker half.

#109 scrubs recorded fixtures against the same set and reads 0068 for it directly.

## Three treatments, and nothing else

Excluded. The field never appears in an event at any severity, with no
configuration that admits it. The session token and anything derived from it,
which is 0005's rule and 0004's, arriving here without change. A password, which
the core holds for the length of one request on one sign-in route and never
writes. Whatever the secret store in 0033 handed back.

Reduced. The field appears as a correlator: a fixed-width prefix of a digest of
the value, salted with a value created when the core is created, held in memory
for its lifetime, never written and never emitted. The account identifier, the
item and other server-supplied identifiers, the device identity, the server
identity and the address it came from.

Carried whole. Counts, durations, an error kind from 0004, a capability name from
#10, an HTTP status, a severity, an event identity, a boolean. None of them can
differ between two people running the same build against the same server, which
is 0068's own test applied field by field.

A field nobody has classified is excluded. That is the direction the default has
to fail in, and it is the opposite of the direction a default falls in on its own,
because an event is written by somebody who wants to see the value.

## What the correlator buys and what it costs

Within one run, two events about the same item carry the same correlator, so a
report shows that the same thing failed eleven times rather than that eleven
things failed once. That is the whole of what an identifier was in the event for.

Across runs it means nothing, and across devices it means nothing, because the
salt differs. Two reports from two people about the same film do not line up, and
neither do two reports from one person a day apart. The first of those is the
point. The second is the cost, it is real, and somebody chasing an intermittent
fault across restarts pays it.

Nothing in a report lets a reader confirm a guess about what an identifier was.
The salt never leaves the process, so a digest over a small input space stays
opaque, which is the property 0041 explicitly does not claim for a cache key and
this one does. The two are different because the salt is different: a cache key
has to be the same on the next run, and a correlator must not be.

## The rule does not relax at the verbose level

`detail` in 0100 is the level somebody turns on to answer a question and turns off
afterwards. It is also, and for the same reason, the level running at the moment
somebody is about to send what it produced to a stranger. A rule that carried more
at `detail` would carry the most exactly when the output is most likely to leave
the device.

So what varies with severity is how much is emitted, not how much is redacted. A
`detail` event may exist where a `notice` event would not, may carry more fields,
and may fire per request rather than per session. Every field in it is treated the
same way it would be treated in a `fault`.

This is the answer to the question the issue asks as a rule per level, and it is a
different answer from the one the question expects. A per-level rule sounds
careful and its levels are chosen by whoever is debugging, which puts the
redaction decision in the hands of the person with the strongest reason to turn it
off.

## An error returned to a caller is not this

0004's payloads carry an address in three kinds and an identifier in a fourth, and
they keep them.

The caller is the client, in the same process, holding the address it handed the
core a moment earlier. Redacting it would remove nothing from anybody and would
make the payload useless for the thing it exists for, which is a client deciding
what to do and what to say. 0004 already fixes the two hard limits that hold in
both directions: no payload anywhere holds a token, and `answer-not-understood`
never carries the answer itself, because an answer holds library contents.

What is redacted is the copy that leaves through the sink. A `failure` event names
the kind that was returned and carries the reduced fields, and never the payload
whole. 0101 places the sink outside the boundary and says everything handed to it
is treated as though it will be published, which is what makes the sink the
boundary rather than the error type.

The measurement spans in #61 are on the sink's side of that line and inherit this
rule with nothing added, which 0068 already implies by putting a span naming an
item identifier under its list.

## The bundle is the client's

#71 asks for a diagnostics bundle whose exclusions are stated in the bundle
itself, and the core cannot produce one. 0068 and 0100 both fix that an event is
handed over and forgotten in the same call, with no ring buffer and nothing
retained, so there is no store of past events for the core to assemble.

What the core supplies instead is the rule as data: for each field name it has
ever emitted, which of the three treatments it applies, available through a call
that cannot wait in the terms of 0009. A client assembling a bundle out of what
its own sink kept includes that statement verbatim, so whoever is about to send it
can read what is not in it.

The statement is about what the core did, not about what the client's sink did
afterwards. A client that renders events into a log file of its own has made its
own decisions, and the bundle says so rather than implying a guarantee across a
boundary the core cannot see.

## What this does not decide

The check that proves any of this. #71's own condition is a test that drives a
full session at the most verbose level and searches the output for every named
field, and it needs the fake server in #21, so until that exists this record is a
rule nothing refuses.

What is in a recorded fixture. #109, against 0068's list.

That nothing secret reaches the cache. #48, which is the same rule on a different
route.

The wording of anything. 0003 and 0004 leave every sentence to the client, and a
redacted field is data with a different value rather than a phrase.

## Why this is written down before the code

The failure the issue names is a person attaching a log to a bug report, and the
thing that makes it likely is that nothing about it looks like a disclosure while
it is being built. An event carrying an item identifier is written by somebody
debugging item handling, at the moment they need the identifier, and it passes
review because the alternative is an event that says nothing.

The specific defect this record is against is the one that is cheapest to write:
redaction applied to a rendered string by searching it for things that look
personal. 0100 already refuses a rendered sentence for this reason and this is
where the consequence lands. Such a rule misses the field it was written for,
because an opaque identifier looks like nothing in particular, and it removes a
film title that happened to contain something matching an address.

The second defect is the per-level relaxation, and it is worse because it reads as
the responsible design. It puts the strongest redaction on the events nobody
looks at and the weakest on the ones about to be sent to a stranger.

Neither has happened here, because nothing in this tree emits an event. The rule
has to exist before the first one, because a field emitted whole is in every log
every client wrote from the day it shipped, and deciding afterwards that it should
not have been does not remove it from any of them.

## Alternatives, and what each cost

Redaction at the sink, as the client's job. The client knows where the output
goes, and a client that never writes to a file needs none of this. 0101 refuses it
in one sentence: the core does not trust the sink to redact and does not ask it
to. It would also put one rule in eleven implementations, where the tenth is the
one that ships without it.

Rendering events as sentences and redacting the text. It is what most logging
does, and it needs no per-field classification. 0100 already refuses the rendered
sentence, and the cost here is the one named above: a text rule cannot see a field
boundary, so it misses opaque values and removes innocent ones.

Carrying identifiers whole, on the grounds that a diagnostic sink is the client's
own and stays on the device. True most of the time. The case it fails is the only
case that matters, which is the log that was asked for and sent.

An unsalted digest, so that correlation works across runs and across devices. It
would let two reports about the same item be recognised as such, which is
genuinely useful when a fault is rare. It costs the property that the correlator
means nothing outside its run: an unsalted digest over an identifier is a value
anybody can compute from a guess, so it identifies rather than correlates.

A per-run counter rather than a digest, assigning 1, 2, 3 to values as they are
first seen. It leaks nothing at all, not even a fixed mapping. It costs a table
the core holds for the life of the run, which grows with the number of distinct
identifiers seen, on a wall of two hundred tiles, with no bound anybody has a
reason to pick.

Dropping identifiers entirely rather than reducing them. The strongest position
and the simplest to prove. It costs the ability to tell one recurring failure from
several distinct ones, which is the first question anybody asks of a report, and
the reply to it would be that the core cannot say.

A configuration that turns redaction off for support. It is what somebody will ask
for, and it would make a hard fault easier to chase. It costs the property
outright, and the switch would be set by whoever is already frustrated, on a
device belonging to somebody who is not in the conversation.

## What would reverse this

A fault is investigated and closed as not diagnosable, twice, with the missing
information named and shown to be something the treatments above removed. One is
an unlucky report. Two is evidence the reduction is too strong, and the
replacement says what is carried instead and what that costs.

The correlator turns out to be reconstructable from a bundle, for instance because
something else in the same event narrows the input to a handful of candidates.
That is this record being wrong rather than needing tuning, and the replacement
says what an event may carry beside a correlator.

#48 or #109 lands a mechanism that classifies fields, and its classification and
this record's three treatments cannot be expressed in each other. Then one of them
is the authority and the other is a description, and which is which is a record
rather than a convention.

The chosen means in #11 makes per-field classification unenforceable, so that a
new field reaches the sink with no treatment and nothing refuses it. The default
above says such a field is excluded, and if that default cannot be made to hold in
the code, this record is claiming a property nothing keeps and is superseded by one
that says so.
