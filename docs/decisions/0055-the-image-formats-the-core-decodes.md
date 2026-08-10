# 0055. The image formats the core decodes, and how the rest are refused

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #55

## The decision

The core decodes JPEG, PNG and WebP and nothing else, it decides which of the
three a response is by reading the bytes rather than by believing the content
type the server declared, it refuses anything outside the set before a decoder is
reached and refuses an image whose declared dimensions exceed a stated bound
before pixels are allocated for it, and every one of those refusals is
`answer-not-understood` from 0004, which is a different answer from the item
having no image of that kind.

## The accepted set, and why each is in it

JPEG, because it is what a photographic still is stored and served as, and
because refusing it would refuse artwork rather than a format.

PNG, because it is what anything with flat colour or transparency is stored as,
and because a server producing a placeholder or a logo produces one of these two.

WebP, because it is the format a server's own resizing produces when it has one
available, and #49 asks the server for the size that will actually be drawn, so
the resized answer is the ordinary answer on this board rather than the unusual
one. It is the entry with the highest cost in this record: a third decoder is a
third parser on the untrusted side of 0101 and a third target for #86. It is
taken because the alternative is either refusing the response a server gives to
the request #49 is built to make, or asking for a format the server then has to
convert away from, which spends the server's processor on every tile in a wall of
two hundred.

Which formats a given Jellyfin server actually serves is a claim here rather than
a measurement. Nothing in this tree contacts a server, and no command in this
repository produces that list. #104 is where recorded fixtures are held honest
against a real server, and it is the route by which this claim becomes a
measurement or is contradicted.

## What is refused, and the reason for each

The refusals are named rather than left as the remainder, because a reader
deciding whether to add one needs the argument that kept it out.

AVIF and HEIC are refused for a reason specific to this repository rather than a
general one. Both are still images carried in a video codec's coding tools, so
decoding one reaches a decoder of the class 0112 places outside the core and
inside the platform. Accepting them would either pull a video decoder into the
core, which is the surface 0003 and 0112 both keep out, or reach for the
platform's decoder for images, which 0003 refuses by name because it produces
eleven answers to which formats are accepted. Neither is a small change, and
neither is a thing to arrive at by adding a format to a list.

SVG is refused because it is not an image format in the sense this record is
about. It is a document with a parser that resolves references, and the
well-known failures of accepting one are that it fetches, that it scripts, and
that it reads local entities, each of which overturns a decision this board took
somewhere else. 0069 says the core reaches one origin, and a format whose parser
can issue a request is that rule undone inside a decoder.

GIF is refused because what it offers over PNG is animation, artwork is a still,
and its decoder carries the historical defect class of the whole family. There is
no case on this board where a poster has to move.

BMP, TIFF and ICO are refused together. Each is a container rather than a
compressed image, each has more shapes than anything on this board would produce,
and no route in #49 asks for one.

Anything not listed at all is refused by the same rule as the named ones. The
list of accepted formats is the rule, and the refusals above are the ones worth
their reasons rather than the whole of what is refused.

## Refused on content, never on the claim

0101 puts a declared content type on its untrusted list and names this record as
the place the consequence is taken. It is taken here.

The format is decided by reading a fixed prefix of the bytes and matching it
against the signature of each accepted format. Nothing else decides it. The
`Content-Type` header the server sent does not decide it, the file extension in
the address does not decide it, and a mismatch between what the server declared
and what the bytes are is not an error in itself: the bytes win and the
declaration is a field on a diagnostic event, at most.

A prefix that matches none of the three is refused before any decoder is reached.
That is the part of this record that keeps the surface small, and it is why the
refusal is by list rather than by attempting a decode and seeing what happens.
Attempting a decode to find out what a thing is means the parser has already been
handed the bytes, which is the whole of what was supposed to be prevented.

The signature is checked against the accepted set alone. There is no detection
step that identifies a wider set of formats and then refuses the ones outside the
list, because such a step is itself a parser over untrusted bytes and it grows
every time somebody adds a format to its table.

## Bounds enforced before anything is allocated

Two bounds, in order, both before pixels exist.

A bound on the encoded length, applied while the response is being read rather
than after it is complete, so a server sending without end is refused during the
transfer rather than after the device has held all of it. Sixteen megabytes.

A bound on the declared dimensions, read out of the format's own header after the
signature matched and before a buffer for the decoded image is allocated. Eight
thousand one hundred and ninety two in each axis, and sixteen million pixels in
total. Both, not either: a per-axis bound alone admits an image that is inside
both axes and enormous in area, and a total alone admits one that is a single row
long enough to overflow an index somewhere downstream.

Each of the three numbers is chosen rather than measured, and the harness in #65
is where measured replacements would come from. The reasoning behind them is the
same in each case and it is not about how large a picture looks. It is about what
an attacker can make the core allocate with a short response, which is the shape
these bounds exist against: a header declaring enormous dimensions costs the
sender nothing and costs the device a buffer. Sixteen million pixels at four
bytes each is sixty four megabytes, which is already far beyond any tile a client
draws and is at the edge of what a television will give a single allocation
before the platform kills the process.

An image that passes both bounds is not therefore trusted. It is decoded under
every other rule 0101 sets for an untrusted parse, and the decoder is in #86's
fuzzing target set for the same reason.

This bound is not the memory budget in #50, and the two are different quantities
that would otherwise be confused. This one is per image and it is enforced before
a decode, so that no single response can make the core allocate more than it
says. #50's is the total of decoded bytes the core holds at once across every
image, and it is enforced against a working set rather than against a hostile
input. Neither substitutes for the other: a total budget is reached by two
hundred legitimate tiles as easily as by one hostile image, and by then the
allocation this record refuses has already happened.

## What a refusal is called

`answer-not-understood` from 0004, for a format outside the set, for bytes whose
signature matches nothing, for declared dimensions past a bound, and for an image
whose decode fails part way through.

The fit is imperfect and saying so is part of the decision. A refused format is a
shape the core recognised and declined on purpose, and the word in the kind says
the opposite. It is taken anyway because a sixteenth kind is a change to 0004 and
to every client that handles the fifteen exhaustively, and because 0069 already
took the same kind for a cross-origin redirect on the same reasoning. That is
twice. 0004 names the measurement that would overturn it, which is
`answer-not-understood` becoming the kind an operator sees most, read off the
events in #100, and this record is one more thing pushed under that kind rather
than an argument that the kind is right.

An image that is refused is distinct from an item that has no image, which is
#51's answer and is not an error at all. One says the server has nothing and the
other says the server sent something wrong, and a client that cannot tell them
apart shows a failure sentence over a library that is fine.

A refused image is not cached, in either direction. 0006 already says a response
to a request that failed is never cached, and the reason applies here with force:
caching the refusal would mean a server that fixed a broken image stays broken
for that client until something evicts the entry. #51's negative answer is a
different thing, because the server said there is no image, which is an answer
rather than a failure.

## Why this is written down before the code

The decoder is the most attacked surface this repository will carry, and 0003 and
0112 both place it inside the core deliberately, on the grounds that one answer
to which formats are accepted is safer than eleven. That argument only pays if
the one answer exists. Without this record the core inherits whichever formats
the first decoding library happens to support, which is a set nobody chose and
which grows when the library is updated.

The specific failure is the accept-then-see shape. Written at a call site, the
natural code hands the bytes to a decoder and handles the error, because that is
what a decoding library's interface invites. It works, it passes every test
written against real artwork, and the surface it exposes is every format the
library was built with, including the ones nobody on this board has ever asked
for. Nothing about it looks wrong in review, since the error is handled.

Written afterwards, this is a narrowing rather than a decision, and a narrowing
is argued against by whoever finds an image that stops working. Written now,
before an image has been decoded in this tree, the accepted set costs one list.

## Alternatives, and what each cost

Accepting whatever the chosen decoding library supports and handling failures.
The least work, and it never refuses an image a server can produce. It costs the
surface outright: the accepted set becomes a property of a dependency rather than
a decision, it changes when the dependency is updated without anybody deciding,
and #86's fuzzing target set is then derived from somebody else's feature list.

Deciding the format from the declared content type, and refusing on that. Simple,
cheap, and it reads as though it is doing the same job. It costs everything,
because 0101's whole position is that the declaration is chosen by whoever sent
the bytes, so a hostile server declares JPEG and sends something else.

Sniffing into a wide table of formats and refusing everything outside the
accepted set. It gives a better diagnostic, since the refusal can name what the
thing actually was. It costs a second parser over untrusted bytes whose only
purpose is to produce a nicer message, and it is a parser that grows.

Adding AVIF for the bandwidth. It is genuinely smaller on the wire, which matters
on a wall of two hundred tiles over a home connection. It costs a decoder of the
class 0112 places outside the core, which is not a format decision but a
supersession of two records, and it should be argued as one if it is wanted.

Refusing WebP as well, leaving two formats. The smallest surface available and
the easiest to defend. It costs the response a server gives to the request #49 is
built to make, and the fallback is asking the server to convert to JPEG on every
tile, which moves the cost onto the operator's machine at the moment two hundred
tiles are being scrolled.

Bounding only the encoded length, on the grounds that a short response cannot
carry a large image. It removes the header read before the allocation. It costs
the case the bound exists for, since a few hundred bytes of header can declare
any dimensions at all and the allocation happens before the rest of the bytes are
needed.

A sixteenth error kind for a refused image. It says what happened, and a client
could show something specific for it. It costs a change to 0004 and to every
client's exhaustive handling, for a case a client's behaviour does not actually
differ on: whatever the reason, there is no image to draw and there is nothing
the person can do about it.

## What would reverse this

A format outside the set is found to be what a server serves for ordinary
artwork, observed against a real server under #104 rather than supposed. The set
then widens by one, with its own reasoning in a superseding record, and the
reasoning has to answer the parser cost rather than the convenience.

A bound refuses an image a server legitimately produced, twice. One is a server
configured oddly. Two means a number here is wrong, and the record is superseded
by one carrying numbers from the harness in #65 rather than chosen ones.

`answer-not-understood` becomes the kind an operator sees most, which is the
measurement 0004 already names for itself. This record is one of the two places
that pushed a refusal under it, so a supersession of 0004's vocabulary supersedes
this record's last section with it.

An advisory lands against the decoder for one of the three accepted formats, in
the chosen toolchain, with no fixed version available. The set narrows for as
long as that holds, and the narrowing is a record rather than a configuration,
because a format silently dropped presents as artwork that stopped working.
