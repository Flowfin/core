# 0061. The span facility, and what a span costs when nobody is listening

Date: 2026-08-12

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #61

## The decision

Measurement is a second client-supplied interface beside the diagnostics one in
0100, and a span on it carries a name from a set declared in one place in the
tree, an identifier meaningful only inside one run, a parent handed to it
explicitly rather than inferred, an interval read from the single injected
`steady` source in 0102, and an outcome from three values; it is delivered once,
when it closes, to a subscriber supplied when the core was created, and where no
subscriber was supplied a span reads no clock, allocates nothing and takes no
lock, which is the bound this record states in place of a time nobody has
measured.

## What a span carries

A name, from the set described in the next section, never composed where it is
emitted.

An identifier. Unique inside one run of the core and meaningless outside it. It
exists for one purpose, which is the join in 0008: a client reporting its own
half of a published number names the core interval it joins to, and without an
identifier the client's duration and the core's duration are two numbers that
cannot be shown not to overlap.

It is allocated in sequence from a counter the core owns, rather than drawn at
random from a space wide enough to be unique everywhere. A globally unique
identifier is a value stable enough to correlate two reports from one device,
and 0068 places that kind of correlation outside what the core hands anybody. A
counter that starts again with the process cannot do it, and it does everything
0008 asks of the identifier.

A parent, or nothing. The section below says why it is handed over rather than
inferred.

An interval, as two readings of `steady` from the one injected source in 0102.
0008 requires both endpoints of an interval to come from one clock and requires
an interval whose endpoints did not to be discarded rather than reported. With
one source, and one clock named for spans, that cannot arise from a choice made
where a span is emitted. It can still arise from a span closed by something
other than the work it was measuring, which is the stop case under Delivery.

An outcome, from three values and no fourth: completed, failed, cancelled.

Three because 0009 already requires the distinction on the way out of the core,
where a cancelled call has its own outcome distinct from every failure and #37
maps nothing onto it. Two values would put a withdrawn tile in the same bucket
as a decode that went wrong, and the tile wall in #53 is the case where most
spans are cancelled, so the ordinary case would be the one that spoiled the
numbers. A failed span carries no kind from 0004: the kind is already on the
failure the caller received and on the event 0100 emits for it, and a third copy
is a third thing to keep in step.

Nothing else, and the section on fields below is where that is argued rather
than asserted.

## The names, and where they are written down

The three intervals a build is gated on are named in 0008 and are not restated
here.

Names are lower case and dotted, and a sub-interval's name begins with the name
of the interval it sits inside. So the set reads as the nesting does, and a
reader holding a name knows which of the three numbers it is inside without a
table.

Every name is declared in one place in the tree rather than written as a literal
where it is emitted, for two reasons. #67 has to publish each measurement with
the command that produced it, and a set derivable by reading one file is a set a
run can print rather than one somebody keeps a list of. And 0008's argument
about naming the endpoints applies to the names themselves: a literal at an emit
site is renamed by whoever is working in that file, and a renamed span is a
number that stops arriving with nothing failing anywhere.

Which subsystem emits which span, and what its sub-intervals are called, belongs
to the issue that builds the subsystem. That is the same placement 0100 makes
for the identity of a diagnostic event, and for the same reason: a span emitted
with no name written down before it is emitted is that issue's defect rather
than this record's. The six sub-intervals 0008 names in prose take their
identifiers that way, under the rule above.

Adding a name is therefore not a change to this record. Changing one of the
three in 0008 is a change to 0008, because those three are what a build is gated
on, and renaming one detaches the gate from what it was gating. That is the same
failure #113 describes for a check-run name, arriving through a different door.

## A parent is handed over, never inferred

A span that sits inside another is given its parent when it is opened. There is
no ambient context, no thread-local or task-local carrier, and nothing the
facility works out from where the code is running.

0009 is the reason. Work in this core moves between two lanes by design: a
request waits on the waiting lane and a body large enough to matter is parsed on
the processing lane, and a completion may run in a sink the client supplied. An
ambient context has to be carried across each of those handovers by hand to stay
correct, which is the explicit parent with more machinery around it, and
wherever it is not carried it is wrong at exactly the handovers the
sub-intervals exist to measure. A parse attributed to whichever lane picked it
up is a plausible number that sends somebody to the wrong subsystem.

What this costs is that a subsystem which cannot reach the parent cannot open a
child. That is the case worth meeting rather than hiding, because an interval
nobody can place is one 0008 cannot add up.

## Delivery

Once, when the span closes. Not at open, and not at both.

A subscriber that saw opens would have to hold every open span to pair it with
its close, which is state with a bound somebody has to choose and a rule for
what happens when the bound is reached. The case that would set that bound is
the tile wall in #53, where two hundred spans are opened and most are
withdrawn. Delivering once means the subscriber holds nothing.

Everything about threads, locks and retention is 0100's, stated once there for
both facilities and not restated here: the subscriber is called on the thread
the work happened on, the core holds no lock of its own across the call, nothing
is retained, and a subscriber that blocks blocks a lane.

Where the subscriber is supplied is 0100's answer as well, which is where the
core is created in #115, and it is not changed afterwards. That matters more
here than it does for events, and the next section is why.

A span still open when the core has stopped is discarded rather than delivered.
0009 has the stop call cancel every outstanding call before it waits, so the
ordinary path closes those spans as cancelled and delivers them. Anything still
open after that was not closed by the work it was measuring, and an interval
ended by a stop is a measurement of the stop.

## What it costs when nobody is listening

With no subscriber supplied, opening a span tests one reference and does nothing
else. No clock is read, nothing is allocated, and no lock is taken. Closing it
does the same.

That is the bound this record states, and it is a property rather than a time.
Nothing in this tree can be measured: there is no language chosen, no build and
no test command, and the harness that would produce a number is #65. #61 asks
for a bound proven by a test, and a time written here would be a number with no
command behind it, which is what this repository refuses everywhere else. The
property is what a test can assert as soon as there is code, and a number is
what #65 adds to it.

Not reading the clock is the part that is easy to lose. The natural
implementation opens the span, reads the clock, and decides at close whether to
hand anything over. It is close to free in the ordinary sense and it still reads
a clock twice per span, on a wall of two hundred tiles with six sub-intervals
inside each. The decision belongs at open, and that is the whole of what close
to nothing means here.

Because the subscriber is fixed when the core is created, the answer at open is
the answer at close. A facility whose subscriber could arrive part way through a
run would have spans opened without a clock reading and closed with one, and the
honest thing to do with those is discard them, which gives a client that
subscribed at the wrong moment a run with a hole in it and no statement about
where the hole is. Fixing the subscriber at creation removes that case instead
of handling it.

## What a span never carries

No fields. Not an item identifier, not a server address, not a byte count, not a
status code.

0068 places an item identifier under its personal data list, and 0100's rule
that an event carries fields rather than a sentence exists so that #71 can
redact by reading field names. A facility with no fields does not need that rule
to reach it at all, which is one fewer place for #71 to be applied
approximately.

What is given up is attribution. A span says that an artwork decode inside a
cold first tile took what it took, and it does not say which image. That is the
right trade for what 0008 asks of this facility, which is arithmetic over many
intervals, and it is the wrong trade for looking into one image, which is what
the events in 0100 are for.

## What the suite may do

0102 fixes this and it is not restated here beyond the line this record depends
on: all three clocks reach the core through one injected source, a test may
advance `steady` by any amount, and it may not move it backwards.

So a timing test opens a span, advances the source, closes it and asserts an
interval, in microseconds of real time and with the same answer on a loaded
machine as on an idle one. That is what #61 asks of the controlled clock, and it
is already decided.

## What this does not decide

The three allocations, what a build fails on, and how many repetitions a number
is taken over. 0008, and #66 is where that gate is built.

Which subsystem emits which sub-interval, and what it is called. The issue that
builds the subsystem.

Whether a number is any good. #65 produces one and #67 publishes it with the
command that produced it.

What a client does with a span once it has one. 0003 places that outside the
core, in the same way it places the wording of an error.

## Why this is written down before the code

Instrumentation is the thing that gets added per subsystem, which is the
sentence #61 opens with. What this record adds is that three of the decisions
above cannot be made per subsystem at all, because the second subsystem to make
one differently produces nothing anybody sees.

The parent is the first. A subsystem that works out its parent from the lane it
is running on produces sub-intervals that add up, and they add up to the wrong
thing, and nothing about the number says so.

The clock read at open is the second. It is a decision inside the facility with
no visible surface, so it gets made once by whoever writes the facility and is
never looked at again, and the property it costs is the one #53 needs most.

The identifier is the third, and it is the one that cannot be repaired
afterwards. A globally unique identifier handed out with every span goes into
whatever a client reports, and a value that correlates two reports from one
device has left before this repository hears about it. Narrowing the identifier
later does not reach what was already sent.

## Alternatives, and what each cost

An existing tracing library, with its span model, its context propagation and
its exporters. Nothing to design, a shape many readers already know, and the
nesting comes with it. It costs a dependency in the graph eleven clients embed,
which is what #103 is being written against; it costs an exporter that reaches
the network from inside the core, which 0069 refuses; and its spans carry
attributes, so #71's redaction rule would have to reach a second facility rather
than none.

Ambient context propagation, in the manner most such libraries use. Emit sites
get shorter and a subsystem does not thread a parent through its own calls. It
costs correctness at every handover between the two lanes in 0009 and at every
completion that runs in a client's sink, and it costs it quietly, because a
misparented sub-interval is a plausible number rather than a missing one.

One facility carrying both events and spans. Refused in 0100, where the argument
is, rather than here.

Fields on spans. A span could say which item, which size and how many bytes,
which is what somebody wants the first time a number is surprising. It costs #71
a second facility to redact and 0068 a second route by which an item identifier
leaves, and the thing it is wanted for is a single occurrence, which is what an
event already is.

Timing every span unconditionally and letting the subscriber discard what it
does not want. One code path, no branch at open, and the numbers are there
whenever anybody asks. It costs the property in #61 that instrumentation is
close to free when nobody is listening, which is the property the tile wall in
#53 has the least room for.

Delivering at open and at close. A subscriber sees work start, which is what a
live view wants. It costs two deliveries per span and a correlation table in the
subscriber, and it lands on #53 before anywhere else.

Sampling, so that only some spans are delivered. The standard answer to a
facility that costs too much, and it stays available later. It costs the join in
0008, since a client's half cannot be paired with a core interval that was never
delivered, and it buys nothing in the harness in #65, where a run is twenty
repetitions taken deliberately.

Random identifiers wide enough to be unique across devices and runs. They let a
span identifier be carried into any store without a collision. It costs 0068
what the section above describes, and it buys a property nothing here asks for,
since the join in 0008 is inside one run.

## What would reverse this

The harness in #65 measures the cost, with a subscriber present, of the
sub-intervals inside a cold first tile at a share of that interval's allocation
in 0008 large enough that the instrumentation is inside the number it reports.
The nesting is then the wrong shape, and this record is superseded by one built
on that measurement rather than by removing spans one at a time until the number
comes back.

A sub-interval genuinely cannot reach its parent where it is opened, twice. One
is a subsystem structured badly. Two means the explicit parent is the wrong
mechanism for this core, and the record is superseded by one naming what carries
the context instead, with what it costs at the lane handovers.

A client is found to need a field on a span to compute its own half of a number
in 0008. The rule against fields then costs the whole number it was written
beside, and it is bought back with a named field and a redaction rule that
reaches this facility, in a record rather than at an emit site.

The subscriber has to become settable after creation, for a client whose
measurement is turned on by a person rather than at start. The fixed-at-creation
property is then unavailable, and with it the guarantee that a span opened
without a clock reading cannot be closed with one, so this record is superseded
by one saying what happens to a span that was open across the change.
