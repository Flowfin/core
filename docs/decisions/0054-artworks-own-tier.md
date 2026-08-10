# 0054. Artwork's own tier, the split, and what gives way

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #54

## The decision

The cache is two tiers under the one store interface in 0040, artwork in one and
everything else in the other, each with its own bound and its own use order so
that neither can evict the other, defaulting to two hundred and twenty four
mebibytes of artwork beside thirty two of metadata whose sum is the total 0042
states; the client sets each bound and the total is their sum rather than a third
number that has to agree with them; and the one place where artwork gives way to
metadata is a metadata write the device refused, where the core frees artwork and
tries once more, never the other way round.

## Which entries are in which tier

0006 lists five kinds of entry and the split follows that list rather than
inventing a category.

The artwork tier holds artwork bytes.

The metadata tier holds library query results, item metadata, capability answers
for a server, and the decoded dimensions of an image. Dimensions are in the small
tier and not beside the bytes they describe, which is worth stating because it
looks like a mistake: they are tens of bytes each, they are what #52 uses to
reserve space before an image arrives, and losing them costs a layout that shifts
under a person rather than a picture that is briefly missing.

A kind that is in neither is not a cache entry. Positions recorded while the server
was gone are the queue in 0047, and 0043 already says why: they are a person's
action rather than a copy of a server's answer, and nothing about them expires.

## The split, and the arithmetic behind it

Two hundred and twenty four mebibytes of artwork, thirty two of metadata. Both
chosen rather than measured, on the same two per-entry figures 0042 states, and #65
is where measured replacements would come from.

    224 MiB / 40 KiB   =  5734 artwork entries
     32 MiB /  2 KiB   = 16384 metadata entries
    224 + 32           =   256 MiB, which is the total in 0042

Seven eighths to artwork and one eighth to metadata, and the ratio rather than
either number is what the reasoning is about. An artwork entry is about twenty
times the size of a metadata one, and a library has one or two images per item, so
a cache that held artwork and metadata for the same set of items in the same
proportion would be spending roughly ninety five per cent of itself on pictures.
The split gives metadata more than that proportion on purpose: sixteen thousand
items of metadata is a larger library than the artwork tier holds pictures for, so
the tier that decides whether there is a screen at all runs out last.

That is the whole reason the tiers are the sizes they are. Metadata is small, it is
what the interface is built from, and losing it costs a person a blank screen in
front of a server that is not there. Artwork is large, numerous, and cheap to lose,
because it can be fetched again and because #51 already makes a missing image a
first-class answer rather than a failure.

A client sets both at creation under #115.

The metadata tier has a floor of four mebibytes, which is about two thousand items
on the figure above, and a bound below it is refused there with the floor named.
Below that the tier holds less than a large library's listing, so a single scroll
evicts what the scroll started from and the cache costs more than it returns.

The artwork tier may be set to zero, and that is not the same as the floor being
zero. A client that draws no artwork exists, and the probe an operator runs against
their own server in #92 is one. A client that holds no metadata does not exist,
because there is nothing left for the core to serve from the cache at all.

The total is not a setting. 0042 states two hundred and fifty six mebibytes as the
default, and that number is what these two defaults add to rather than a third
value the client can set. Three numbers where two would do is three numbers that
can disagree, and the disagreement would be discovered by whichever of them the
code happened to check first.

## Each tier evicts only itself

0042 fixes the rule, which is least recently used, and this record fixes what it is
applied to: each tier has its own bound and its own order, and eviction in one
never considers an entry in the other.

That is what makes the promise in #54 structural rather than a preference. A scroll
through a large library is thousands of artwork reads and writes and it cannot
evict a library listing, because there is no order in which the two appear
together. Nothing has to prefer metadata at eviction time, because nothing ever
chooses between them.

It also means neither tier can borrow from the other. An artwork tier that is full
while the metadata tier is half empty evicts artwork, and the free space in the
other tier stays free. Borrowing sounds like an improvement and it is the same
mistake in slower motion: a tier that can grow into its neighbour's space is one
bound again, and the first large scroll takes the space and does not give it back.

## The one place artwork gives way

Everything above is about the cache reaching its own bound. The case where the two
tiers genuinely compete is the device running out of room, because there they are
competing for something neither bound describes.

When a write to the metadata tier is refused with `storage-unavailable`, the core
evicts from the artwork tier until it has released at least eight times the length
of the refused write, or one mebibyte, whichever is more, and then attempts that
write once again. If it is refused again, 0042's suspension takes over and applies
to both tiers.

Eight times, because freeing exactly what was needed buys one write and then the
next metadata write does the same work again. One mebibyte as a floor, because a
run of two-kibibyte writes would otherwise trigger an eviction round each, and a
round is a walk of the artwork order and a call into the client's store.

A write to the artwork tier that is refused never evicts metadata. It is not
retried, the entry is not cached, and #51 already makes that a first-class answer.
This is the asymmetry #54 asks for, written at the one place in the design where it
has anything to bite on.

## What is not covered

Where the two bounds a client set add up to more than the device can hold, this
record does nothing for the metadata tier that 0042's suspension does not already
do. The artwork eviction above frees room once and the device fills again, and
after three refusals the core stops writing for five minutes and says so. The core
does not shrink its own bounds in response, for 0042's reason: the device being full
is not evidence that the bounds were wrong, and it is more often evidence that
something else on the device is large.

So the guarantee that artwork gives way to metadata holds inside the cache
completely and against a full device only for as long as there is artwork left to
release. That second half is a mitigation rather than a property, and stating it as
one is the difference between this record and a record that claims the problem is
solved.

## Why this is written down before the code

One bound over both kinds is what exists until somebody splits it, and it is not
visible as a decision in review, because a single bound and a single order is the
shape any cache starts as. Its failure is precise and slow: a person scrolls a large
library, every tile written pushes the library listing further down one shared use
order, and the listing is evicted. What they see is that going back to the top of
their library needs the network, on a device where the whole point of the cache was
that it did not. Nothing in that report says the word artwork.

The split cannot be added afterwards without deciding what the entries already
written belong to, which is a migration over data the core cannot enumerate,
because 0040 gives the store no listing. Deciding it now costs two numbers and two
orders.

The direction of the giving-way rule is the other half. Written the natural way
round, a failed artwork write frees metadata to make room for a picture, and it
does it during a scroll, which is when the metadata is most likely to be needed
next.

Neither has happened in this tree. There is no code here for it to happen in.

## Alternatives, and what each cost

One bound over both kinds, with a comparator that prefers metadata. Fewer numbers,
one order, and it does express the preference. It costs the guarantee, because a
preference is a rule that holds until the metadata is the least recently used thing
in a cache full of artwork, which after a long scroll it is.

Tiers per kind, all five of them. More precise, and each kind's bound could be
reasoned about on its own. It costs four more numbers that nobody has evidence for,
and the four small kinds behave the same way under pressure, so the split would be
recording a distinction that changes no behaviour.

A proportional split rather than absolute bounds, for instance one eighth of
whatever total the client set. One number for the client instead of two, and the
ratio stays right at every size. It costs the floor: a client setting a small total
gets a metadata tier below what a listing needs, and the failure appears only on the
devices that set the total low, which are the constrained ones this matters most on.

Letting a tier borrow unused space from the other. Better occupancy, and it is what
a single bound gives for free. It is a single bound with extra steps, because the
first sustained scroll takes the borrowed space and there is no moment at which
giving it back is triggered.

Artwork not cached at all, on the grounds that it is refetchable. No tier, no split,
no eviction interaction. It costs the wall of two hundred tiles in #53 on every
scroll and every cold start, which is the case this repository names as the one
where artwork is won or lost.

Evicting artwork on any storage refusal, including a refused artwork write. Uniform,
and one rule instead of two. It makes a full device spend the artwork tier on
artwork, evicting pictures to store pictures, which converges on an empty tier and
no more room than it started with.

## What would reverse this

The measurement in #62 shows the metadata tier being evicted from during ordinary
use at its default size. That means sixteen thousand items is not a large library
and the split is wrong, and it moves in a record that supersedes this one.

The artwork eviction on a refused metadata write is observed to fire repeatedly on
devices that are not full, which would mean `storage-unavailable` is reaching the
core for reasons that have nothing to do with room. The response above would then be
spending the artwork tier on a condition it cannot fix, and the rule is withdrawn
rather than retuned.

A client is found that needs a third tier, with what it holds and why the two above
could not carry it. One is a client using the cache in a way this board did not plan
for. Two is a split that is the wrong shape.

0040 grows a listing operation, for the reason that record names as its own
reversal. Tiering could then be a property of an entry the store can filter on
rather than two separate accountings in the core, and the whole of this record's
mechanism is superseded even where its numbers survive.
