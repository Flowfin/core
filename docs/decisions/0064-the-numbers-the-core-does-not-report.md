# 0064. The two numbers the core does not report, and what a client gets instead

Date: 2026-08-13

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #64

## The decision

Focus change and dropped frames get no number, no proxy and no span of their own,
which is 0008's refusal carried one door further; what the core offers in their
place is three obligations that keep it from being the reason either number is
missed, each of them already decided in 0009 and answerable only by the two
routes in #117 and #76, together with the report shape in 0008 used for a number
the core has no interval in, so that eleven clients measuring these two measure
them the same way.

## What 0008 already settled, and the one thing it did not

0008 decided that the core measures neither number, claims neither, and offers no
proxy for either. That table and its two refusals are not restated here.

What it did not close is the span. Its argument is about a value reported as the
number, and a span is not reported as anything: it is an interval delivered to a
subscriber, which reads as data rather than as a claim. So the refusal 0008
makes at the level of a published number leaves the cheaper version of the same
mistake available, and this record closes it. There is no span named for focus
change and none for dropped frames, and the name set 0061 declares in one place
in the tree carries neither.

The reason is that a span for either would have to be opened somewhere the core
runs, and on both paths that is the wrong place. Nothing of the core is on the
focus-change path at all, which is 0008's first row. On the frame path the core
is present, but what it is doing there is decoding and holding a lane, and those
are already spans under 0008's sub-intervals, named for what they are. A second
span named for the frame would be the same interval under a name that says it
measured a frame.

## The three obligations, and where each one is decided

None of the three is new here. Each is stated in the form that is true, because
two of the three are written in #64 in a form that is not quite.

A core call on a drawing path does not block, with one call excepted rather than
zero. 0009 admits exactly one blocking call, the stop call in #115, and says so.
Stopping the core is not something a client does while drawing, so the obligation
holds on every path this number is about, and it holds by an exception that is
named rather than by a rule with none. Written without the exception it is a
sentence that is false about the core as recorded, and the first person to find
the stop call would be entitled to read the whole obligation as approximate.

Decode does not run on the caller's thread. 0009 lists four kinds of work that
never run on the thread that called in, and a decode is one of the four. The list
is closed, so this obligation is that list rather than a promise beside it.

Prefetch does not hold a lock a caller waits on, up to a bound nothing states.
0009 says a call that cannot wait may take a lock the core holds, and that the
core never holds one of its own locks while calling out to client-supplied code,
so such a wait is bounded by the core's own work and not by anybody else's. That
is the property, and it is weaker than the sentence in #64, which reads as no
wait at all. The wall of two hundred tiles in #53 is where a duration for it
would come from, and #53 is where it belongs, because the number depends on what
prefetch does under a lock rather than on the lock rule.

## What refuses each of the three, and what refuses none of them today

0009 names two routes and neither is a sentence in a document. #117 runs the
core's own suite under a detector that reddens when a concurrency claim is
broken. #76 asks the same question from the client's side, against the core a
client actually linked.

Both apply to all three obligations without being extended, which is worth
stating because the alternative is a third route written for this record. The
detector reads the thread identity every stage ran on, so it sees the decode
obligation directly and sees the blocking one as a lane occupied by a call that
entered from outside. A lock a caller waits on is the same observation with a
duration attached.

Nothing in this tree refuses any of them. There is no language chosen, no build
and no test command, so neither route exists, and #64's condition that each
obligation has a test that fails when the obligation is broken is unmet on all
three. What this record adds against that day is that the three tests are two
routes rather than three tests, so a client that ran #76 has asked all three
questions and not one of them.

## What a client reports for a number the core has no interval in

0008 fixes what a client sends back for a whole number, and its join is the span
identifier. For these two there is nothing to join to, and the shape is used
anyway, with the span field empty.

    number          focus-change, or dropped-frames
    span            empty, and empty means the core had no interval in this
    client-start    the moment the client began, on its own clock
    client-end      the moment the client considers the thing done
    clock           which clock those two came from
    platform        what it ran on, at whatever granularity the client has
    build           what was running, so a number can be attributed

An empty span field rather than a second shape, because a client reporting four
numbers through two shapes writes the second one badly, and because the empty
field is the statement this record exists to make, arriving where somebody
reading the data will meet it. A reader holding a report for focus change can see
that no core interval was claimed, without having read 0008.

What this does not do is make the number comparable across clients. Two clients
timing focus change from different events produce two numbers under one name, and
nothing here can fix that, because the events are theirs. The shape fixes what is
reported and the client contract in #75 is the only place the endpoints could be
fixed. Saying so is the honest half of "so that every client measures them the
same way", which this record delivers in part and not in whole.

## Where the statement is published, and why not yet

#64 asks for the statement in the documentation an operator and a client author
both read. Neither document exists. #95 is the documentation an operator reads
before installing, #75 is the client contract, and both are open with nothing
landed.

So the publication condition is unmet, and this record is not a way of meeting
it. A decision record is read by somebody deciding to reopen a decision, which is
not the operator deciding whether to run this and not the client author deciding
what to measure. Both of those documents reference this record when they land
rather than restating its sentences, because a summary of a refusal drifts into a
softer refusal, and the softer version of this one is that the core does not
measure those numbers yet.

`README.md` is not that place either, and not only for the reason above. It is
where #13, #23 and #74 also write, and a paragraph added here to close a
condition would be the fourth hand in one file.

## Why this is written down before the code

0008 argued the proxy that arrives as a number. This record is about the one that
arrives as a span, and the difference is what makes it a second record rather
than a sentence in the first.

The shape is predictable. Once there is code and a subscriber, somebody wanting
to know why frames are dropping opens a span around the work that seems to cause
it and names the span for the symptom, because that is the name they were
thinking about. It costs nothing at the emit site, it needs no store record and
no build gate, and it is never reported as the published number by anybody. Then
the harness in #65 sums the set of names it finds, #67 publishes each measurement
with the command that produced it, and a value under a name containing the words
of a published number is in a document. Nothing lied at any step.

The second failure is the publication condition read down to what could be
delivered. With no operator page and no client contract, the cheapest way to
close #64's first condition is to write the statement into a record and call it
published. That is a condition met in a document rather than in the world, and it
is the shape this repository refuses everywhere else.

The third is the obligations arriving one subsystem at a time. Three sentences
that read as separate promises get three separate proofs, and the third one is
the one nobody writes. Placing all three on the two routes 0009 already names is
what stops that, and it is cheap to do now and awkward once two of them have
their own test.

## Alternatives, and what each cost

Offering a span for the core's contribution to a dropped frame, named for the
frame. It is the thing somebody actually wants at the moment a frame is dropped,
and the data exists already under other names. It costs exactly what the section
above describes, and it costs it through a facility that has no gate on it, so
nothing would refuse the name once it was in the set.

Putting the statement in `README.md` now and closing the first condition. It is
the one document that exists and that both readers reach, so the condition would
be met in the world rather than on paper. It costs a fourth hand in a file three
other issues are written against, and it costs a summary that will disagree with
#95 and #75 on the day either lands, which is the drift this repository counts as
a defect class rather than an accident.

Saying nothing until #95 and #75 exist. Nothing is written twice and the sentence
is composed once by whoever writes the document it belongs in. It costs the three
obligations their placement, so each subsystem meets them as a fresh question,
and it costs the span refusal its timing, since the cheapest moment to refuse a
name is before anybody has a reason to add it.

Superseding 0008 with a record covering both halves. One record for the whole
speed budget and one place to look. It costs the supersession rule its meaning:
0008 is not wrong and nothing in it stops holding, so the new record would be
0008 with a section added, which is an edit wearing more ceremony.

Reporting both numbers from the harness in #65 against a client written for the
purpose. 0008 prices this one already, under gating on the whole published number
rather than on the core's share, and the price is that the harness is what gets
measured.

## What would reverse this

A client is found to call the core on the focus-change path, on any platform.
That is 0008's own reversal condition reached through this record: the first row
of its table becomes wrong, the boundary in #3 is not where both records assumed
it is, and a core interval exists to be named.

A dropped frame is attributed to the core twice, and both times the attribution
needed something the core does not hand over. Once is an investigation done the
hard way. Twice means the refusal above costs more than the proxy it prevents,
and it is bought back with a named span, a redaction rule that reaches it under
#71, and a line in 0008 saying what a build does with it.

#95 or #75 lands and its author finds the sentence here does not fit what its
reader needs. The record that replaces this one is written from that document
rather than the document being written from this record, since the reader it
failed is the one it was for.

A route arrives that can refuse one of the three obligations and not the other
two. The claim that the three are two routes is then wrong, and this record is
superseded by one placing each obligation against what actually refuses it.
