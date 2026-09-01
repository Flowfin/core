# 0052. The shape reserved before the bytes

Date: 2026-09-01

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #52

## The decision

The shape the core supplies before an image's bytes is an aspect ratio and never
a pair of pixel dimensions; it is read from item metadata alone, it arrives as
the decimal text the answer carried rather than as a number something else
already made of it, and both supported server lines state one for the `Primary`
kind of [0049](0049-the-artwork-address-and-the-size-asked-for.md)'s five and for
no other; where nothing is stated - the other four kinds, a `Primary` the server
left unset, and an item that has no image of the kind at all - the answer says so
and the rectangle a client reserves is the whole box it asked to draw in, which
is a shape rather than a guess and cannot move under a late image.

## What the two supported lines state, and what they do not

Read against the two commits
[0272](0272-the-route-next-up-is-read-from.md) names, which are the two lines
entry 3 of #1 answered with, so this record and the surface record are written
against one pair of tips:

    $ A=1fbd8739292cce610231be93daf43368733edf63    # the 10.11 line
    $ B=c3ed1407ca698b0905de99da87b67415e6a62dbd    # the 12.0 line
    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/SharedVersion.cs?ref=$r" --jq '.content' \
    >     | base64 -d | grep AssemblyVersion | head -1
    > done
    [assembly: AssemblyVersion("10.11.11")]
    [assembly: AssemblyVersion("12.0.0")]

**One field carries an image's shape, it is a nullable double, and it is the same
line of the same file on both.** The item type the paged reads and the item
detail row answer with is where a client would look for one:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/MediaBrowser.Model/Dto/BaseItemDto.cs?ref=$r" \
    >     --jq '.content' | base64 -d | grep -n 'public double? PrimaryImageAspectRatio'
    > done
    395:        public double? PrimaryImageAspectRatio { get; set; }
    395:        public double? PrimaryImageAspectRatio { get; set; }

**Nothing on that type carries a shape for the other four kinds**, on either
line:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/MediaBrowser.Model/Dto/BaseItemDto.cs?ref=$r" \
    >     --jq '.content' | base64 -d \
    >     | grep -cE 'public .*(Backdrop|Thumb|Logo|Banner).*(AspectRatio|Width|Height)'
    > done
    0
    0

So a backdrop, a thumb, a logo and a banner have no stated shape to read, and
that is a property of the surface rather than of what has been built here.

**The field is asked for rather than always sent.** It is attached only where the
caller named it among the fields it wants:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Emby.Server.Implementations/Dto/DtoService.cs?ref=$r" \
    >     --jq '.content' | base64 -d | grep -n 'ItemFields.PrimaryImageAspectRatio'
    > done
    235:            if (options.ContainsField(ItemFields.PrimaryImageAspectRatio))
    372:            if (options.ContainsField(ItemFields.PrimaryImageAspectRatio))

A read that does not name it is answered with the field unset, which is
indistinguishable at this end from an item that has no primary image. So the core
names it on every read whose answers a client may reserve space for, and until it
does, a client finding nothing stated has been told nothing about the item rather
than told the item has nothing.

**What a stated value is.** Null where the item has no primary image at all; the
measured ratio of the file where the server could read one; and the item type's
own default in the two cases in between:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Emby.Server.Implementations/Dto/DtoService.cs?ref=$r" \
    >     --jq '.content' | base64 -d \
    >     | sed -n '/double? GetPrimaryImageAspectRatio/,/^        }$/p' \
    >     | grep -nE 'imageInfo is null|return null|IsLocalFile|GetDefaultPrimaryImageAspectRatio|return .double.width / height'
    >   echo "--"
    > done
    5:            if (imageInfo is null)
    7:                return null;
    10:            if (!imageInfo.IsLocalFile)
    12:                return item.GetDefaultPrimaryImageAspectRatio();
    22:                    return (double)width / height;
    30:            return item.GetDefaultPrimaryImageAspectRatio();
    --
    5:            if (imageInfo is null)
    7:                return null;
    10:            if (!imageInfo.IsLocalFile)
    12:                return item.GetDefaultPrimaryImageAspectRatio();
    22:                    return (double)width / height;
    30:            return item.GetDefaultPrimaryImageAspectRatio();
    --

THE CORE DOES NOT SEPARATE THOSE THREE AND CANNOT. A ratio measured from the file
and an item type's default arrive in one field with nothing beside them saying
which. That is stated here so that a stated ratio is never later read as a
measurement of the bytes a fetch will return.

## The three shape-shaped fields on one item, and the two this record is not about

The same type carries two other fields a hurried mapping reaches for, and both
are about the item's own media rather than about any image of it. This is the
trap this record exists to close, because all three read as "the shape" in a
sentence.

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Emby.Server.Implementations/Dto/DtoService.cs?ref=$r" \
    >     --jq '.content' | base64 -d | grep -n 'dto.AspectRatio = \|var width = item.Width;'
    >   echo "--"
    > done
    847:                dto.AspectRatio = hasAspectRatio.AspectRatio;
    1301:            var width = item.Width;
    --
    1031:                dto.AspectRatio = hasAspectRatio.AspectRatio;
    1534:            var width = item.Width;
    --

`AspectRatio` there is a string taken from the item's video, and `Width` and
`Height` are the item's own pixel dimensions where it has any. A poster is not a
video frame and a photograph's own dimensions are not its cover's, so a client
handed any of the three as the shape of a `Backdrop` would reserve a rectangle
derived from something else entirely. Nothing in the field names says so.

So the core takes a stated ratio for `Primary` and refuses one offered for any
other kind, rather than accepting whatever a caller mapped. The refusal is over a
runtime value because the loop that walks an item's five kinds holds the kind in
a variable, which is where the mix-up happens.

## Why a ratio, and never a pair of pixel dimensions

The quantity a layout needs before the bytes is a shape, and the pixels are
already decided at this point by the caller:
[0049](0049-the-artwork-address-and-the-size-asked-for.md) makes the maximum
width and the maximum height both required and rounds each onto a rung, so the
box is fixed before any of this is asked. A second pair of numbers beside it
would be either the same box again or a disagreement with it, and the second is
the defect.

Pixel dimensions are also the quantity that does not survive the request. The
ladder rounds up and the server scales down inside the box, so what comes back
carries the image's shape and the box's bound, and the only one of the three
numbers that is the same before and after is the ratio.

THIS IS A DIFFERENT QUANTITY FROM THE ONE
[0054](0054-artworks-own-tier.md) PLACES IN THE METADATA TIER, and the two read
as one in a sentence. [0006](0006-the-cache-contract.md) fixes the decoded
dimensions' life as the same as the bytes they describe, so an entry of that kind
exists only where an image was fetched and decoded. Every tile on a first scroll,
and every tile on the cold start
[0046](0046-what-is-served-before-a-session-is-restored.md) describes, has no such
entry, which is exactly the moment the layout is first built. The quantity here
comes out of item metadata, which [0006](0006-the-cache-contract.md) caches with
an age of its own, so it is present on a cold start where a decoded dimension is
not.

## Why it arrives as the text the answer carried

The value on the wire is a decimal number inside an answer body. Turning it into
a machine number is a decision - how many digits are kept, what an exponent does,
what a magnitude no image has does - and a decision made by whatever decodes the
answer is one made where nobody argued it.

So the seam is the text, on the shape
[0049](0049-the-artwork-address-and-the-size-asked-for.md) already uses for an
item identifier and a content tag: the value exactly as the server sent it, and
the rules here are what turn it into something usable or refuse it. It also makes
every refusal provable without a decoder, a socket or a server, which is what
lets these rules land before the transport does.

The grammar admitted is digits, optionally a point and more digits, and nothing
else. A sign and an exponent are both shapes a number may legally take in an
answer body and both are refused here, unparsed. Inside the range an aspect ratio
can occupy nothing is written that way, so refusing the shape costs a well
behaved server nothing and takes the magnitudes that are not shapes - a negative,
a zero spelled `-0.0`, `1e300` - out before any arithmetic sees them.

## The bound a stated ratio is inside, and where it comes from

The ratio is kept as ten-thousandths of a width per unit of height, in whole
numbers, so the arithmetic that reserves a rectangle carries no floating point
and no rounding anybody has to reason about. Four fraction digits is finer than a
rung can express: at the top rung of 3840 the fifth digit moves an edge by less
than half a pixel.

The bound is derived from the ladder rather than chosen. A box built from
[0049](0049-the-artwork-address-and-the-size-asked-for.md)'s ladder has both
edges between its lowest and its highest rung, so the widest shape any box can
have is the top rung over the bottom one and the narrowest is the bottom over the
top. A stated ratio outside that pair cannot be drawn tightly inside any box this
core will ever ask for, so it is a server describing something no image in a
library is, and it is refused by name rather than clamped. Both numbers are
computed from the ladder in one place, so a ladder that changes carries them with
it.

## What a client reserves, in each answer

Where a ratio is stated, the rectangle is the largest one of that shape fitting
inside the box, computed in whole numbers that round down, so it never exceeds
the box on either edge. An edge is never reserved as zero: a ratio at the far end
of the bound inside the smallest box would round to nothing, and a rectangle of
no width is not a rectangle a layout can hold.

Where nothing is stated, and where a stated ratio was refused, the rectangle is
the whole box. That is the stated answer for a server that knows nothing: a
client reserves the space it already asked to draw in, whatever arrives is drawn
inside it, and no layout moves. It looks worse than a tight rectangle and it is
not a guess, which is the trade taken here deliberately.

An item with no image of the kind reserves the same rectangle as one whose shape
is unstated. #51 already fixes that an absence is an answer rather than a failure,
and an empty tile occupies a rectangle exactly as a full one does.

THE TWO ANSWERS ARE NOT MERGED EVEN THOUGH THEY RESERVE THE SAME RECTANGLE. A
server that stated nothing and a server that stated something the core refused
are different statements, and a client reporting on its own library can tell them
apart. Collapsing them would make a server sending nonsense look like a server
being quiet, which is the collapse #51 refuses one module over.

## Why this is written down before the code

Without it the first client to draw a tile wall decides it, and it decides it in
the direction that is cheapest at that moment: ask for the bytes, read the
dimensions out of them, lay out afterwards. That is the published rule inverted
rather than implemented, and it is unenforceable in every client at once, because
a rule that needs the bytes cannot be honoured before the bytes.

The second failure is quieter, and it is why this record reads the server rather
than only stating a rule. Three fields on one item look like the shape, two of
them are about the item's own media, and a mapping that takes the wrong one
produces rectangles that are plausible, stable and wrong for a whole library.
Nothing reports it, and a reviewer comparing a field name against a variable name
sees a match.

The third is the one this record is written against by name: a per-kind table of
conventional ratios typed into a call site because something had to be returned.
Five numbers nobody here can derive from anything, taken from whatever a client's
stylesheet used, wrong for the library whose covers are square, and
indistinguishable from measurements once they are in the tree.

## Alternatives, and what each cost

**A per-kind table of default ratios in the core.** A tight rectangle for every
kind on the first paint, and it is what a client author asks for. It costs five
numbers this repository cannot derive: neither supported line states a shape for
four of the five kinds, so the table would be a claim wearing the clothes of a
reading, and a library of square covers or of 4:3 stills would be laid out
wrongly and consistently. The record admitting it would have no reversal
condition anybody could check, which is
[0001](0001-decision-records.md)'s own test for a decision that should not be
taken this way.

**Pixel dimensions rather than a ratio.** Directly usable, with no arithmetic at
the client. There are none to give: neither line carries a width or a height for
any image of an item, and the two fields that do carry pixels are the item's own
media. Deriving them would mean fetching, which is the thing this record exists
to happen after.

**Take the ratio as a machine number the answer decoder already made.** One less
parse and one less type. It moves the decision about digits, exponents and
magnitudes into whatever decodes the answer, where it is settled by a library's
defaults rather than argued, and it makes every rule here need a decoder before
it can be proven at all.

**Answer nothing where the server states nothing, and let the client decide.**
Honest, and the smallest possible core. It is the published rule handed to eleven
clients to each get wrong differently, which is what a shared core exists
against. It also loses the case this record most wants held: an image the server
knows nothing about still occupies a rectangle, and that rectangle is already
known.

**Fetch a header per image and read the dimensions before laying out.** Exact,
and it works for every kind. It is a request per tile before the first paint,
which is two hundred requests on a tile wall to avoid a layout shift, and the
cold start it would sit inside has no network at all.

## What would reverse this

Either supported line states a shape for a kind other than `Primary` in the item
type a client reads. The third command in this record's first section returns
something other than `0` for one of the two, and this record is superseded by one
naming what is stated and for which kinds.

A ratio refused by the bound derived from the ladder is seen arriving from a
server somebody runs, rather than from a fixture. The bound is then wrong rather
than the server, and the record replacing this one carries the reading that found
it.

[0049](0049-the-artwork-address-and-the-size-asked-for.md)'s ladder is superseded
by one that has no lowest and highest rung for the bound to be computed from. The
bound is derived rather than typed, so a ladder whose rungs merely move carries
it along; what would need a new record is a ladder that stops having ends.

A client is measured drawing a wall of tiles with the whole box reserved for
every unstated kind, and the space that leaves is shown to cost more than a table
of conventional ratios would have cost in wrong rectangles. That is a measurement
rather than a preference, and it is what would buy the first alternative above.
