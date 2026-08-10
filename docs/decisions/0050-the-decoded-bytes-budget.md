# 0050. The decoded bytes the core holds at once

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #50

## The decision

The core holds at most sixty four mebibytes of decoded image at any moment,
counted as the pixel buffers it has allocated and not yet handed to a caller,
enforced by not starting a decode that would exceed it rather than by failing one,
with the floor a client may set it to being the per-image bound in 0055 because a
budget smaller than the largest image that record admits would make a legal image
undecodable; there is no decoded-image cache anywhere in the core, so this budget
is a working set and never a store.

## What is being counted

0009 fixes that a decoded image belongs to the caller from the moment it is handed
over and that the core does not read it again. 0006 fixes that what the cache holds
for artwork is the encoded bytes and the decoded dimensions, and not the pixels.
Between them there is no decoded image anywhere in the core that somebody is not
waiting for.

So the quantity this record bounds is small and precise: the pixel buffers of
decodes that are running, plus the pixel buffers of decodes that have finished
whose outcome has not yet reached the caller. Nothing else. A number that sounded
like a cache size is a working set, and saying so is most of the value of writing
it down, because the other reading produces an eviction policy for something that
is never stored.

It is counted in bytes of the buffer the core allocated, at the decoded dimensions,
which is four bytes a pixel. Not the encoded length, which 0055 bounds separately
during the transfer, and not what the client does with the image afterwards, which
is the client's memory from the moment it is handed over.

## The number

Sixty four mebibytes. Chosen rather than measured, and #65 is where a measured
replacement would come from.

The arithmetic, on one more chosen number. A poster drawn on a tile wall is taken
as three hundred by four hundred and fifty pixels, which is the size #49 asks the
server for rather than a full-resolution original.

    300 x 450 x 4        =    540000 bytes for one decoded poster
    64 MiB               =  67108864 bytes
    67108864 / 540000    =       124 posters held at once
    200 x 540000         = 108000000 bytes, which is 103 MiB

The last line is the point of the number rather than an accident of it. A wall of
two hundred tiles does not fit, and it is not supposed to. What is on a screen at
any moment is a fraction of two hundred, #53 cancels the decode for a tile that
scrolled off before it finished, and a budget large enough to hold every tile a
person scrolled past is a budget that holds a library in pixels on a device that
has one screen.

The other end is fixed by 0055 rather than chosen here. That record admits an image
of sixteen million pixels, which is sixty four million bytes decoded, so the
largest image the core will ever decode fits inside this budget alone with about
three mebibytes to spare and every other decode waits behind it. The two numbers
are within four per cent of each other and that is deliberate: a budget below the
per-image bound would mean a decode that 0055 accepts and this record can never
run, and a budget far above it would let several maximal images be allocated at
once, which is the allocation 0055 exists to refuse arriving by a different route.

## What happens at the bound

A decode that would take the total past the budget does not start. It waits until
enough has been handed over, and then it starts.

It does not fail. There is no sixteenth kind in 0004 for this and none is asked
for: nothing has gone wrong, the caller asked for more work than the device can
hold at once, and the answer to that is to do it in an order rather than to refuse
part of it. A failure here would also be a failure whose occurrence depends on what
other callers were doing, which is the least reproducible thing a client can be
handed.

Waiting decodes are started in the order they were asked for. Not by size, which
would starve the large ones, and not by a client-supplied priority, which is a
scheduling interface the core would then owe every client. #53 is where a caller
expresses that it no longer wants a tile, and it expresses it by cancelling.

A cancelled decode releases its buffer at the end of the step it is inside, which
is 0009's bound on cancellation rather than a new one here, and the room is given to
whatever is waiting.

The one case that stalls is a client that asks for a hundred and fifty decodes and
consumes none of the completions. Its own hundred and twenty fifth decode waits
until it takes one. That is correct, it is the client's backlog rather than the
core's, and the core cannot tell it apart from a client that is consuming slowly.
What the core does is say so: after five seconds with a decode waiting for room, it
reports through 0100 how much is held and how many decodes are waiting. Not as an
error on any call, because nothing failed, and not as a sentence, because 0004 and
0100 both fix that the core writes none.

## What this does not add

A second bound on how many decodes run at once. 0009 already sizes the processing
lane to the host's usable processor count less one with a floor of one, and puts
image decoding on that lane, so the number that can run at once is that size. A
second number for the same question is how two answers get written down, and the
comment on #50 that asked for this record said so first.

The two bounds interact and neither replaces the other. The lane size limits
decodes that are running; this budget also counts decodes that have finished and
are waiting to be handed over, which is the larger number on a client that is slow
to consume. On a four-processor television the lane is three, so three maximal
images cannot be decoded at once under this budget and the third waits, which is
this record binding before the lane size rather than after it.

A decoder supplied by the platform. 0003 refuses it by name, 0112 puts image
decoding on the core's side of its line, and the comment on #50 records that the
bullet in the issue asking for one is answered in the opposite direction. Building
it would mean superseding 0003 with an argument that answers the untrusted-input
reason rather than the performance one.

Anything about the artwork tier in the cache. That is #54, it holds encoded bytes,
and the two quantities are not comparable: a tier of two hundred and twenty four
mebibytes of encoded artwork is several gigabytes of pixels if it were ever decoded
at once, which is why the budget here is a working set and the tier there is a
store.

## What a client may set

A client sets the budget at creation under #115.

The floor is the per-image bound in 0055, which is sixteen million pixels at four
bytes each. Below that, an image 0055 admits could never be decoded, and the
refusal a person would see would depend on a memory setting rather than on the
image, which is exactly the client-dependent accept set 0055 exists against.

A client that wants a smaller ceiling than that floor is asking for 0055's
dimension bound to be lowered. That is a change to 0055, with its own record and
its own argument about what an attacker can make the core allocate, and it is not a
number a client sets.

There is no ceiling on what a client may set it to. A desktop with memory to spare
holding a full wall of two hundred is a legitimate configuration, and nothing in
the core is harmed by it.

## Why this is written down before the code

The failure this prevents does not look like a memory bug when it arrives. Two
hundred decodes started together on a television produce a process the operating
system kills, and what a person reports is that the application closes when they
scroll. Nothing in that report points at decoding, and the same code is fine on
every machine anybody develops on, because a desktop absorbs a hundred megabytes
without noticing.

The part that cannot be added later is the shape of the answer rather than the
number. A core that decoded without a budget for a year has callers written against
a decode that always starts immediately, and adding admission control afterwards
introduces a wait at a call site nobody expected one at. Adding it now costs a queue
and a counter.

The definition is the other half, and it is the half that would have been got wrong.
Somebody reading "a bound on decoded bytes held" with no record in front of them
builds a cache of decoded images, because that is what holding bytes sounds like,
and it is a second cache with a second eviction policy for data 0006 deliberately
does not keep. That is not a small mistake to unwind: the memory it occupies is the
memory this budget exists to bound.

## Alternatives, and what each cost

No total budget, with only the per-image bound in 0055. One number instead of two,
and no queue. 0055 already says why it is not enough: a total is reached by two
hundred legitimate tiles as easily as by one hostile image, and the per-image bound
has nothing to say about the two hundredth.

Failing a decode that does not fit, rather than queueing it. Simpler, and the caller
learns immediately that it asked for too much. It costs reproducibility, since
whether a call fails depends on what other callers are doing, and it costs a
sixteenth kind in 0004 or a misuse of an existing one.

Bounding the number of decoded images instead of their bytes. Easy to count and easy
to explain. A poster and a backdrop differ by an order of magnitude in area, so a
count is a byte bound that is wrong by that factor in whichever direction the screen
happens to be showing.

A decoded-image cache with its own eviction, so that scrolling back is instant. It
is the feature this number is most likely to be mistaken for, and it is genuinely
useful. It costs the ownership rule in 0009, which is what lets the core hand pixels
over and forget them, and it costs a second store with a second policy for something
that can be reproduced from bytes the cache already holds.

Sizing the budget from the device's reported memory. It would be right on every
device rather than chosen once, and it is the option to revisit first. It costs a
number that differs on every device, so a client's own testing says nothing about
what another device will do, and a reported figure on a television is not the figure
the application is allowed to use.

Making the budget per caller rather than for the core. One client's backlog would
then not delay another's decodes. There is one processing lane and one device
memory, so a per-caller budget bounds nothing that matters, and the sum of them is
the number that would have to be bounded anyway.

## What would reverse this

The harness in #65 measures the decode working set on a wall of two hundred tiles
and finds the queue is entered on an ordinary scroll rather than on an extreme one.
That means the number is too small for the case it was chosen for, and it moves in a
record that supersedes this one.

0055's per-image bound moves. This record's floor is derived from it, so a change
there changes the floor here, and the derivation is stated above so that the
dependency is visible rather than discovered.

A decoded-image cache is added to the core with its own record and its own argument.
The quantity this record bounds is then no longer the whole of what the core holds
in pixels, and the budget has to be restated over both.

A client is found whose decodes wait behind another client's backlog in a way that
matters, twice. One is a client that should be cancelling. Two means the shared
budget is the wrong shape and the per-caller option above is what replaces it.
