# 0100. The diagnostics interface, and its relation to measurement spans

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #100

## The decision

Everything the core has to say while it runs leaves it as a diagnostic event
handed to an interface the client supplies, carrying a severity from a closed set
of four, a stable identity, structured fields and a moment, and never a sentence
written for a person; the measurement spans in #61 are a second interface rather
than a second kind of event on this one, so that a client can subscribe to either
without paying for the other; and a core given neither runs unchanged.

## What an event carries

A severity, from four values and no fifth.

`fault` is a defect in the core, the same subject as `internal-fault` in the
vocabulary in 0004. Nothing about a server or a network is being claimed and the
event is worth sending to this repository.

`failure` is something the core was asked to do that did not happen. A kind from
0004 was returned to the caller for it, and the event says which, so that a report
and the client's own error handling describe one occurrence rather than two.

`notice` is something that happened, nothing failed, and somebody supporting an
installation would want it in front of them. A token renewed, a server declared
unreachable, an entry served stale, a tier evicted.

`detail` is everything else. It is the level somebody turns on to answer a
question and turns off afterwards, and a shipped client is expected to be running
with it off.

Four, and the set is closed for the reason the fifteen error kinds in 0004 are
closed. A client's filter is written against these values exhaustively, so a fifth
is a change to this record and to every client that filters, which is the cost
that stops the set growing by accident. Severity graded on an open scale of
numbers is the alternative this refuses, and it is refused in the section below.

An identity. A stable name for what happened, decided once by whichever subsystem
emits it, never renamed without a record, and never a number. It is what a client
filters on, what a report is grouped by, and what somebody searches for when the
same thing has been reported twice.

Fields, as name and value pairs of data. A count, an identifier, a duration, a
kind from 0004, an address. Not a rendered sentence, and not a sentence with
values already substituted into it.

The reason is #71 rather than tidiness. The redaction rule there has to be
applied by something that reads field names and decides per name, because 0068
places a field carrying an item identifier under its personal data list while the
count beside it is not. A rule applied by searching rendered text for things that
look personal is a rule that misses the field it was written for and removes a
title that happened to look like an address.

A moment, read on `wall` through the single injected clock source in 0102. It is
carried for one purpose, which is lining an event up against something outside
this device, and the thing it is lined up against is a server's own log. The core
never reads it back and no core behaviour depends on it, so a device with a wrong
clock produces events that are hard to correlate rather than events that are
wrong.

Nothing else. No stack, no thread identity, no process detail. Those describe a
platform the core does not know, and a client that wants them adds them at the
sink, where the platform is known.

## What emitting one costs, and on which thread it happens

Emitting is a call that cannot wait in the terms of 0009. It returns from state
the core already holds and it is safe from any thread.

The client's implementation is called on the thread the event happened on, inside
the core's own lanes. So an implementation that blocks blocks that lane, and the
one thing this interface asks of a client is that it does not. Handing events to
a queue drained by another thread is the alternative, and it costs a bound
somebody has to pick, a rule for what happens when the bound is reached, and
events that arrive out of order with respect to the work that produced them,
which is the property that makes a report readable.

The core holds no lock of its own across the call. That is 0009's rule for
calling out to client-supplied code rather than a new one here, and it is what
makes an implementation that does something slow a client's problem rather than a
deadlock.

Nothing is retained. 0068 already states this for both facilities: an event is
handed over and forgotten in the same call, and the core keeps no ring buffer, no
file and no history. The diagnostics bundle in #71 is assembled from what a client
kept, not from something the core held on to.

With no implementation supplied, an event costs the test of one reference at the
site that would have emitted it. That is a property rather than a measurement.
Nothing in this tree can be measured, so the condition asking for a stated bound
proven by a test stays unmet, and it is unmet in the issue this record came from.

The level below which events are not produced at all is separate from the sink
and can be changed while the core is running. That call cannot wait either.
Supplying the sink happens where the core is created, in #115, and is not changed
afterwards, so no single run has two halves observed by different things; the
level moves because turning `detail` on to answer a question and off again
afterwards is the ordinary reason anybody touches this at all.

## Why this is a second interface and not a second kind of event

Two facilities, and one line for why: a client that wants diagnostics and not
measurement can have exactly that, and one facility makes that impossible.

The longer version is the property #61 asks for. Spans have to cost close to
nothing when nobody is listening, and a subscriber is what makes them cost
something. Carried on one facility, the moment a client supplies a sink because it
wants `failure` events in its own log, every span in the core has a subscriber and
has to be materialised and handed over for that client to discard. The cheap half
becomes unreachable for anybody who wants the other half, which is the whole
value of the property.

They also stop being the same shape as soon as either is used. A span is a pair of
moments whose reason for existing is arithmetic over many of them, consumed by the
harness in #65 and by a client computing the two numbers it owns in #64. An event
is a single occurrence whose reason for existing is that a person eventually reads
something a client wrote from it. Filtering, grouping and retention differ for
both, and a subscriber to one has to reject everything from the other on every
delivery.

What this does not become is two clocks. There is one injected clock source in the
core and 0102 says so; the spans in #61 report intervals on `steady` and an event
carries a moment on `wall`, and that is the same table in 0102 answering two
questions rather than two facilities each choosing.

Nor is it two subscription models. Both are an interface the client supplies where
the core is created, both are absent by default, both are called on the thread the
work happened on, both hold no lock, and both are forgotten in the same call.
Everything in the paragraphs above about threads, locks and retention is stated
once and holds for both.

## What this does not decide

The wording of anything. The core produces an identity and fields; the sentence a
person reads is the client's, which is the same rule 0003 and 0004 already state
for errors, applied here so that the client owns the wording in both places rather
than one of them.

Where events go, whether they are written down, and whether anything is shown to
anybody. Also the client's, and 0003 is where a core that opens its own log file
is refused.

The redaction rule and the diagnostics bundle a person is asked to send. #71.

The span facility itself, its names, its subscription and its measured overhead.
#61, which takes its names from 0008.

Which subsystem emits which event. That belongs to the issue that builds the
subsystem, and an event with no identity written down before it is emitted is that
issue's defect rather than this record's.

## Why this is written down before the code

This seam is already assumed by landed records and by open work, and it is
specified nowhere. 0003 sends diagnostics out of the core "behind the interface in
#100". 0005 has the core tell a client "through the diagnostics interface in #100"
at the moment it discards a token. 0068 promises an operator that events are
handed to "whatever the client supplied under #100 and #61" and retained nowhere.
#71 plans a redaction rule over fields that no record says exist. Those are the
ones quoted here rather than all of them, and the set moves, so it is derived:

    $ git grep -l '#100' -- docs/decisions

So the next subsystem that has something to report finds the interface named
wherever it matters and no interface behind the name. What it invents is
predictable: a formatted string, emitted from wherever it was convenient, because
that is the shape every runtime makes easiest. Each of those decisions is
individually reasonable and each one removes something. A string cannot be
redacted by field name, so #71 becomes pattern matching over text. A string has
wording in it, which 0003 gave to the client. An emit from a caller's thread
breaks 0009 in the place least likely to be tested.

None of that has happened here yet, because there is no code in this tree to have
done it. That is the whole reason this is cheap today: the record costs one file
now and costs a rewrite of every emit site later.

## Alternatives, and what each cost

One facility carrying both events and spans. One interface, one subscription, one
place a client implements, and the two kinds are genuinely similar in retention
and in their personal data rules, which 0068 already treats together. It costs the
property in #61 that instrumentation is close to free when nobody is listening,
because with one facility a subscriber to either is a subscriber to both. That
cost lands on the tile wall in #53, which is the case with the most spans and the
least room.

The core writing its own log, with the client giving a destination. One interface
fewer and every subsystem gets to report without a client doing anything. Refused
in 0003: a core that logs has chosen a place on a platform it does not know, and
on a shared device it has written to somewhere it cannot promise is private.

A rendered sentence per event, with the fields already substituted in. The
simplest possible sink, because a client that only wants to print has nothing to
write. It costs the wording rule in 0003, it costs translation, and it costs #71
the ability to redact by name, which is the difference between a rule that is
applied and a rule that is approximately applied.

Severity as an open integer, in the manner of the levels most logging libraries
carry. Familiar, needs no decision, and every client already has a mapping for it.
It costs agreement: no two subsystems settle on the same meaning for the middle
values, a client's filter is then written against a number nobody defined, and the
exhaustive handling that makes the closed set in 0004 useful is not available.

A ring buffer inside the core that a client can ask for, which would make the
bundle in #71 a single call. Convenient, and it is what somebody will ask for the
first time a defect is reported without one. It costs 0068's statement that the
core retains none of this, and it puts personal data in a buffer with no rule for
its lifetime, no key, and no place in the sign-out removal in #114.

## What would reverse this

The two facilities are measured, by the harness in #65, to cost more together than
one facility carrying both would, by an amount a client can observe at the tile
wall in #53. Then they become one and the cheap-subscription property is bought
some other way, and this record is superseded by one that says how.

Two events, on two occasions, that genuinely cannot be expressed as a severity, an
identity and fields. One awkward fit is an event written badly. Two is a shape
this record does not cover, and the shape is widened by a new record rather than
by a field named `message`.

#71 lands a redaction rule that cannot be applied by reading field names. Then the
field-and-not-a-sentence rule bought nothing it was written for, and the argument
for it has to be made again on whatever grounds are left.
