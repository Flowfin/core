# 0049. The artwork address, the size asked for, and the rounding that shares an entry

Date: 2026-08-29

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #49

## The decision

An artwork address is the path 0010 fixes with the item and one of five image
kinds written into it, carrying the content tag the item's metadata gave for that
kind and a maximum width and height, both required; the size a caller asks for is
rounded up on each edge independently to the next value of one fixed ladder of
fifteen edge lengths before it reaches the address, so two tiles that differ by a
few pixels ask for one image and share one cache entry; the item identifier and
the tag are refused unless every byte of them is an ASCII letter, digit, hyphen
or underscore, because both arrived from a server and both are written into a
request; and there is no way to build an address that asks for the original.

## What goes wrong without it

Two things, and the second is the one that is invisible.

A full-resolution poster fetched for a tile 180 pixels wide spends bandwidth,
decode time and memory on pixels nobody sees, two hundred times over on one
screen. 0050 counts a poster at three hundred by four hundred and fifty and 0054
counts an artwork entry at forty kibibytes encoded, and both of those numbers are
arithmetic that a full-resolution fetch makes wrong by an order of magnitude.

Two tiles that differ by three pixels fetching twice. Nothing reports it, both
answers are correct, the cache holds two entries where one would do, and the
bound in 0042 evicts twice as fast as its arithmetic says. A size that reaches
the address unrounded is the ordinary way that happens, because a caller
computing a tile size from a screen width gets a different number on every device
and sometimes on every row.

## The five kinds, and why each is in the set

The set is closed and named, the way 0055 names the formats and 0028 names the
schemes, so a kind outside it cannot be constructed rather than being caught at a
call site.

`Primary`, the cover a tile wall is made of. It is the one this record's
arithmetic is written against.

`Backdrop`, the full-bleed still behind a detail screen. It is the largest of the
five and it is why the ladder reaches as far as it does.

`Thumb`, the wide still an episode row and a continue-watching row are drawn
from.

`Logo`, the title treatment drawn over a backdrop. It is usually the one with
transparency, which is 0055's reason for admitting PNG.

`Banner`, the wide strip a series row uses where a client draws one.

WHICH KINDS A GIVEN SERVER ACTUALLY HOLDS IS A CLAIM HERE RATHER THAN A
MEASUREMENT. Nothing in this tree contacts a server and no command in this
repository produces that list, which is the same position 0055 takes about
formats. #104 is the route by which this becomes a measurement or is
contradicted, and a kind found to be served under a name this record does not
carry widens the set in a superseding record rather than in a call site.

## The address carries no index, and that is 0010's doing rather than a choice here

0010 fixes the artwork row as `GET /Items/{itemId}/Images/{imageType}` and that
template carries no index segment. So an item holding more than one image of one
kind is reachable at one address here, and choosing among several is not
something this record can express. Widening the path to carry an index is a
supersession of 0010 rather than an addition here, because the path is 0010's
decision and not this one's.

## The ladder, and why rounding goes up

Fifteen edge lengths, applied to width and to height independently:

    90  120  180  240  300  360  450  600  720  900  1080  1440  1920  2560  3840

A requested edge becomes the smallest ladder value that is not below it. An edge
of zero is refused, and so is an edge above the top of the ladder, because both
of them are a caller asking for something other than the size that will be drawn
and the second is the request this issue exists to make impossible.

Upwards rather than to the nearest. An image smaller than the box it is drawn in
is enlarged by whoever draws it, and that is visible in a way that drawing a
slightly larger image at a smaller size is not. Rounding to the nearest would
send half of all requests to the visible failure to save bandwidth on the
invisible one.

Three hundred and four hundred and fifty are both rungs, and that is not a
coincidence to be tidied away. 0050 takes a poster on a tile wall as three
hundred by four hundred and fifty and computes the decoded budget from it, and
0054 and 0042 take an artwork entry at forty kibibytes encoded at the size this
record asks for. A ladder whose nearest poster rung was far from that pair would
make three landed records' arithmetic wrong, so the rungs were chosen against
those records rather than freely.

WHAT THE ROUNDING DOES NOT BUY, and it is the sentence a reader will otherwise
supply for themselves. Two nearby sizes share an entry only when they fall on one
rung. Three hundred and three hundred and one straddle a rung and are two
requests, and no rounding rule of this shape can be otherwise: any ladder has
boundaries and any two sizes can sit either side of one. What the ladder buys is
that the number of distinct requests is bounded by the ladder rather than by the
number of distinct screens, and that is the quantity 0042's arithmetic depends
on.

## Why the tag is in the address rather than beside it

0006 fixes artwork as the one cached kind that is not revalidated against the
server, on the ground that it is addressed by a tag that changes when the image
changes, so a changed image is a different key rather than a stale entry. That
sentence is a requirement on this record and its reversal condition names this
issue by number.

So the tag is a required input. An item whose metadata carries no tag for a kind
has no address here at all, which is a value a caller has to handle rather than
an empty string that produces a request. What that answer is called is #51, and
this record deliberately stops at not producing an address.

## Why the identifier and the tag are refused on their bytes

Both arrive from a server. 0101 treats every byte from a server as untrusted
whether the server is healthy or not, and both of these are written into a
request the core then sends: the identifier into the path and the tag into a
query parameter.

An identifier carrying a separator, a question mark, a fragment marker or a
percent sequence is a server choosing a path other than the one 0010 fixes,
against a host the core already resolved. That is a request the core assembled on
somebody else's behalf, and refusing on the character set rather than escaping is
the direction that cannot be got wrong quietly: an escaping routine that misses a
case produces a request that works until it does not, and a refusal produces
nothing.

ASCII letters, digits, hyphen and underscore. It is narrower than what a URL
permits and that is deliberate, because the set has to be judged against what
these two fields are rather than against what a path can hold.

## What the parameters are, and why they are the ones the key sees

Three: the maximum width, the maximum height, and the tag. Each is a parameter
0041 calls one that changes the answer, so each is written into the request part
of a cache key.

Maximum rather than fill. A fill parameter crops to a box, and cropping decides
what a person sees, which 0003 places outside this core by name. A maximum bounds
the answer and preserves the shape of the image, so the decoded dimensions are
the ones 0050's four-bytes-a-pixel arithmetic assumes.

The tag is in the key for the same reason it is in the address. Without it a
changed image is a stale entry under an unchanged key, which is precisely the row
0006 wrote artwork's exception into.

## Why this is written down before the code

The rounding set is not a tuning constant. Three landed records compute with the
size this record asks for, and one of them, 0006, carries a reversal condition
that fires on the addressing. A number chosen in a call site would be a number
those records have no way to find, and the first person to notice would be
whoever re-derived 0042's entry count and got a different one.

The character refusal is the other half. It reads like input validation somebody
added defensively, and it is a boundary decision: it is the point at which a
value a server sent stops being a value and becomes part of a request. Written
here, a later widening has to answer 0101; written in the code, it is one
condition somebody relaxes to make a test pass.

## Alternatives, and what each cost

An optional size, with the original fetched when none is given. The interface is
smaller and the caller that knows what it wants is unaffected. It costs the whole
of this issue: the accidental full-resolution fetch is the failure being
prevented, and an optional parameter is how it happens.

Rounding to the nearest rung. Half the requests are closer to what will be drawn.
It costs an enlarged image on half of all tiles, which is the visible half of the
two errors.

A continuous rule, such as rounding up to a multiple of sixty. Fewer numbers to
argue about and no ladder to maintain. It costs the property 0042 needs: the
number of distinct sizes grows with the range rather than being bounded, and the
poster pair 0050 computes with is not a value of the rule unless the modulus is
chosen to make it one, which is the ladder with extra steps.

Escaping the identifier and the tag rather than refusing them. Nothing is refused
that a server legitimately sent. It costs a routine that has to be right about
every case forever, on the untrusted side of 0101, in exchange for accepting
values no server on either supported line has been observed to send.

Putting the size outside the cache key and keying on the item and kind alone. One
entry per image rather than one per size, which is fewer entries. It costs the
answer being wrong: two callers asking for two sizes would read each other's
bytes, and the second one would draw a poster at the wrong size or enlarge a
thumbnail.

## What would reverse this

A server on a supported line is observed under #104 to serve a kind this record
does not name, for ordinary artwork rather than for a special case. The set
widens by one in a superseding record with its reasoning.

A real measurement under #65 puts an artwork entry at the poster rung far from
forty kibibytes, or a decoded poster far from 0050's figure. The rungs are then
chosen again against measured numbers rather than assumed ones, and 0042, 0050
and 0054 are re-derived with them rather than left standing.

The character set refuses an identifier or a tag a server legitimately sent,
twice. One is a server configured oddly. Two means the set here is wrong, and it
is widened in a record that answers 0101 rather than in the code.

A server is found to offer change notification for artwork under #116. 0006's own
reversal condition then fires, that record is superseded in a row, and the tag's
place in the address is argued again rather than kept because it was decided
first.
