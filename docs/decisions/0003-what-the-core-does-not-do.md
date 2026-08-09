# 0003. What the core does not do

Date: 2026-08-08

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #3

## The decision

The core owns everything between a server address and a decoded byte, and it owns
nothing a person can see; a capability belongs in the core only if all eleven
clients would otherwise implement it and could disagree about the result, and it
belongs outside the core if the answer depends on what is on screen, on the
platform, or on the language a person reads.

## What the core owns

Reaching a server. Address parsing, transport, timeouts, certificate validation,
retry and the mapping of every failure onto one error vocabulary.

Holding a session. Acquiring it, renewing it, holding more than one at a time, and
handing the secret to a store the client supplies rather than to one the core
chose.

Caching what was fetched. The keys, the bound, the eviction, the age of an entry
and whether an entry may be served while a request for a fresher one is
outstanding.

Fetching and decoding artwork. Address construction, the requested size, the
accepted format set, the bounds enforced before a decode, and the decoded bitmap
with its dimensions.

Tracking playback position. The unit, the precision, the cadence a position is
reported on, what happens to a position recorded while the server was gone, and
what counts as watched.

Producing measurements. Named spans, their values, the spread across repeated
runs, and a statement of what a run did not measure.

## What the core never owns

Drawing. Nothing in the core puts a pixel anywhere. The nearest it comes is
handing over a decoded bitmap and its dimensions.

Knowing that a list exists, or a tile, or focus, or a page, or a scroll offset.
These are the words a view layer thinks in, and a core that learns any of them
has learned the shape of one client's screen.

Deciding what a screen shows. Which items appear, in what grouping, at what
moment, and what happens when a person presses something.

Platform packaging. Bundles, manifests, signing, store metadata, permissions
prompts, installers and update channels.

The wording of anything a person reads. No sentence the core produces is written
for a person. This includes error text, empty-state text, unit labels and dates
formatted for a locale.

The location of storage on disk. The core is told where to write and never asks
the platform, because where an application may write is a platform question with
a different answer on each of them.

The lifetime of the host process. The core is created and stopped by the client
and never decides on its own that the process should keep running.

## The cases that sit on the line

### Image decoding is inside

It sounds like drawing and it is not. Turning bytes into pixels is a parse of
untrusted input that arrived over a network, which puts it in the same class as
parsing a response body, and it is the most attacked surface this repository will
carry. Eleven clients each reaching for a platform decoder is eleven different
answers to which formats are accepted, eleven different bounds on a declared
dimension, and eleven chances that one of them decodes whatever it is handed.
The core decodes, refuses by name what is not on its list, and stops at the
bitmap. Putting that bitmap on a surface is the client's, and so is any scaling
done for a particular display.

Video decoding is the opposite placement for a different reason, and the record
for #112 is where that line is drawn.

### Video decoding is outside

Placed outside for the reason recorded against #112, which is where the argument
belongs and is not repeated here. The consequence for this boundary is that the
core stops at the handover in #111: an address the platform's own player opens,
with whatever the player needs in order to open it, and nothing beyond that
point.

### Sort and filter are inside

They sound like view concerns because a person changes them from a screen. The
result is not a view concern. Two clients sorting the same library locally
disagree the first time a title starts with a lower-case letter, an article, a
digit or a character outside the ASCII range, and a person who sees one order on a
television and another on a phone reads that as a bug in whichever they saw
second. Ordering and filtering are asked of the server where the server can answer
them, and where the core must do it locally it does it one way, stated once. Which
control a person turns it with is the client's.

### Error identity is inside, error wording is outside

The core owns the identity of a failure, meaning which member of the vocabulary in
#4 it is, what it carries with it, and the guarantee that nothing falls through to
a default. The core owns no sentence about it. A sentence has a language, a tone,
a length that fits a particular screen, and a decision about whether to name the
server at all, none of which the core can know. A client that wants one sentence
per error identity writes eleven sentences once, which is cheaper than a core that
ships strings it cannot translate and cannot shorten.

### The playable handover is inside up to the address

Deciding what to play, asking the server for it, and producing something the
platform can open are the core's. Opening it is not. The line is the address plus
whatever accompanies it, and the core does not learn what a player did with it
beyond the positions reported back through #57.

### Where bytes are stored is outside, how they are keyed is inside

The client supplies a location and the core never looks for one. The key under
which anything is written, and therefore the guarantee that two servers and two
people on one device cannot read each other, is the core's and is decided in #41.
A client supplying a location is not thereby deciding a layout inside it.

### Diagnostics leave the core as events, not as text

The core produces diagnostic events with identities and fields. Where they go,
whether they are written down, and whether anything is shown to a person are the
client's, behind the interface in #100. A core that logs has chosen a destination
on a platform it does not know, and on a shared device it has written to a place
it cannot promise is private.

## Placing something the record does not mention

Three questions, in this order. If the first answer is no, the capability is
outside regardless of the other two.

Would all eleven clients implement this, and could two of them produce different
answers a person would notice? If not, it is one client's feature and belongs
there.

Does the answer depend on what is on screen, on which platform is running, or on
the language a person reads? If yes, it is outside, however convenient it would be
to have it once.

Can the answer be tested without a display, without a real server and without
elevation? If not, either it is outside or the part that can be tested that way is
the part that comes in.

## Why this is written down before the code

Without the boundary written first, the boundary becomes whatever the code turned
out to do, and the code is then the argument for keeping it. The failure runs in
one direction and in small steps: a convenience that takes a list, a helper that
formats a date, a dependency that brings a windowing library with it, and by the
third platform the shared core is a framework two clients are arguing over.
Nothing about any single step looks like the mistake.

## Alternatives, and what each cost

A boundary described only as "no user interface". One sentence, nothing to
maintain, and it decides none of the cases above. Image decoding, sort order and
error text all read as user interface to somebody and as core to somebody else,
which is how a one-sentence boundary produces the drift it was written to stop.

A boundary drawn later, once two clients exist and the overlap is visible.
Genuinely more informed, and it arrives after the interfaces that would have to
change. Every client that started early pays for the correction, and the argument
at that point is about their code rather than about the boundary.

A wider core that owns layout-neutral view state, such as a page of results with
its position. Saves real work in each client, and it is the first step of exactly
the failure described above: a page implies a size, a size implies a screen, and
the core now has an opinion about one.

A narrower core that stops at the transport and hands back raw response bodies.
Very little to disagree about, and it leaves the caching, the artwork bounds and
the position accounting to be written eleven times, which is the duplication this
repository exists to remove.

## What would reverse this

A capability this record places outside the core turns out to be implemented
identically in three or more clients, with the implementations having converged
rather than been copied. That is evidence the line was drawn one step too tight,
and the capability moves in with its own record.

A capability this record places inside the core is overridden or bypassed by two
or more clients, because the core's single answer did not fit their platform. That
is evidence the line was drawn one step too loose, and the capability moves out.

The check in #77 cannot be written against this boundary, because the forbidden
side cannot be expressed as data. A boundary no machine can refuse a crossing of
is a boundary that will be crossed, and this record would then be replaced by one
drawn where a check can see it.
