# 0042. The cache bound, and what is evicted when it is reached

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #42

## The decision

The cache is bounded by bytes the core itself counted as it handed them to the
store, defaulting to two hundred and fifty six mebibytes which is the sum of the
tier bounds in #54 rather than a setting of its own; the entry evicted at the bound
is the least recently used one within its own tier, where used means read out or
written in, and by kind is expressed as the tiers rather than as an order inside
one; eviction waits for an outstanding read of the key it chose and never cancels
one; and a store that refuses a write because the device is full is not a signal
that the bound was wrong, so the core suspends writing for a stated interval rather
than evicting its own entries to make room for whatever filled the device.

## What the bound is counted against

The core counts the bytes it hands to the store, entry by entry, and holds the
total itself. It does not ask the store how much it holds in order to decide
whether to evict.

The reason is that the two numbers answer different questions. A store over a
filesystem reports what the filesystem says, which includes block rounding, and on
a device with sixteen-kibibyte blocks a cache of ten thousand two-kibibyte metadata
entries occupies eight times what the core wrote. A bound enforced against that
number would evict entries the core never accounted for, differently on every
platform, which is one rule per client where 0006 promises one.

The store's own count is read, once, and for one purpose, which is the section on
the index below.

The bound counts payload bytes as the core produced them, before the envelope 0105
wraps every entry in. The envelope is a fixed overhead per entry rather than a
proportion of one, so counting it would make the bound depend on how many entries
happen to be small, and the number that is easy to reason about is the one that
says how much of the server's answers are held.

## The default, and the arithmetic behind it

Two hundred and fifty six mebibytes. It is chosen rather than measured, like every
number on this board that is not accompanied by a command, and #65 is where a
measured replacement would come from.

The arithmetic that produced it is arithmetic on two more chosen numbers, and both
are stated so the reasoning can be checked rather than trusted. An artwork entry
fetched at the size that will actually be drawn under #49 is taken as forty
kibibytes. An item's metadata is taken as two kibibytes.

    224 MiB / 40 KiB   =  5734 artwork entries
     32 MiB /  2 KiB   = 16384 metadata entries

Those two tiers are #54's and their sum is this number. A client that changes one
of them changes the total, because the total is not a third setting that has to be
kept in agreement with the other two.

What makes it the right order of magnitude is the pair of failures on either side.
Too small and the cold start in #46 has nothing to serve, and the wall of two
hundred tiles in #53 refetches on every scroll, which is the case this cache exists
for. Too large and the core is holding a person's library in a directory somebody
else's platform may reclaim without telling it, in a backup that grows, on a device
whose storage the application does not own. Five thousand posters is roughly thirty
times a full screen of them, so a person returns to what they were looking at last
week from the cache and to what they have never opened from the server.

A client sets each tier's bound at creation under #115. A bound below the floor in
#54 is refused there, with the floor named, rather than accepted and quietly
raised. There is no way to express a bound of zero: a client that wants nothing
kept supplies no store, which 0040 already defines, and the difference between the
two is that the second still holds what it can in memory for the life of the
process.

## What is evicted

The least recently used entry in the tier that is over its bound. Used means read
out of the cache or written into it, and both move the entry to the end of the
order.

Eviction happens on the write that would exceed the bound, before that write, and
it removes entries until the write fits. Not on a timer and not on a background
sweep, because a sweep is a thing that runs when nothing else is happening, which on
a television is never.

The three options #42 names are all real and this is why the answer is the third
one expressed through the tiers rather than the first two.

Least recently fetched, which is age order, evicts the library list a person opens
every evening because it was first written a fortnight ago. It is correct for a
cache whose entries are equally valuable and it is wrong here, where what is read
often is what the next screen is built from.

By kind, on its own, is not an order. It says artwork goes before metadata and then
needs a second rule to choose among the artwork, so it is half a decision that
arrives looking like a whole one.

Least recently used inside a tier is the answer, and by kind is what the tiers are.
That is a better place for it than a comparator, because a comparator that prefers
one kind is a rule somebody has to hold in their head at every call site, and two
bounds are a rule the arithmetic keeps on its own. #54 owns the tiers and the split.

Age plays no part in eviction. 0043 already fixes that age marks an entry stale and
never withholds it, and an entry a month past its threshold is worth more than
nothing to a person on a train. Both eviction and invalidation produce `absent`
under 0043, and this record adds no fourth state.

## An entry a caller is holding

Eviction never removes an entry with a read outstanding, and it never cancels one.
It picks the next entry in the order instead, and comes back to that key only after
the read has finished.

Bytes already handed to a caller are not affected by eviction at all, and that is
0009 rather than a rule this record adds: what the core hands over belongs to the
caller from that moment. Evicting the stored copy of an artwork entry does not
reach into an image a client is drawing.

So the property #42 asks for is held in two different ways for the two cases it
covers, and it is worth saying which is which. For bytes already handed over it is
structural and there is nothing to get wrong. For a read in flight it is a rule
somebody has to implement, and it is the one that will be forgotten, because the
window is the length of one call into client code and every test on a developer's
machine closes it before the eviction is chosen.

## The index, and what it costs

0040 gives the store four operations and no listing, and says the bookkeeping
eviction needs is therefore the core's. This is that bookkeeping.

The core holds an index: for every entry, its key, the length the core counted, its
tier, and its position in the use order. Nothing else, and in particular no part of
the entry's value, because an index that held values would be a second cache with
no bound of its own.

The index does not survive a restart on its own, so it is written through the store
like any other entry, under a key reserved for it and inside the envelope 0105
defines. It is written when it has changed and no more often than once every ten
seconds on the `elapsed` clock, and once more when the core is stopped under #115.
Writing it on every cache write would double every cache write, which is the cost
that would make the whole design worse than the listing operation 0040 refused.

That cadence means a core that was killed rather than stopped loses up to ten
seconds of index. What that costs is entries in the store the index does not know
about, which are unreachable and unremovable, because without listing the core
cannot find a key it does not hold.

This is where the store's own count is read, exactly once, at start. Where the
store says it holds more than the index accounts for, the difference is bytes the
core cannot reach, and the core reduces its own budget by that difference rather
than pretending the space is free. The cache is then smaller than its bound by
whatever was orphaned, which is honest, and the alternative is a bound that is
quietly exceeded on every device that has ever lost power.

An index that does not parse, or that 0105's envelope refuses, is an absent index.
The core starts with an empty one, the store's count becomes the whole orphaned
difference, and the cache is effectively unusable until the store is cleared by the
client. That is the worst outcome in this record and it is stated rather than
smoothed over: #105 owns what a damaged entry does, this is the one entry whose
damage is not local, and a route by which a client can clear a store it supplied is
something #105 and #114 reach and this record does not decide.

## A device that is full

A write refused by the store is `storage-unavailable` under 0004, the call that
caused it does not fail, and the entry is simply not cached. That is 0040 and this
record does not change it.

What it adds is what happens on the third one. After three consecutive write
refusals, the core stops attempting cache writes for five minutes on the `elapsed`
clock, reports it once through 0100, and then attempts one write again. Reads
continue throughout, because a device with no room can still be read, and the
entries already held are the ones that matter most at exactly this moment. Three is
chosen because one refusal is a transient and a run of them is a condition, and
five minutes is chosen because it is long enough that a full device is not being
asked hundreds of times and short enough that a person who deleted something gets
their cache back inside one sitting.

The core does not evict its own entries in response. Its bound is its own, and the
device being full is not evidence that the bound was wrong: it is far more likely
to be evidence that something else on the device is large. Evicting here spends a
person's cache to make room for whatever filled the disk, and it does it silently,
and then the space is taken by the other thing anyway and the cache is both empty
and still unable to write.

The one place where the core does evict on a refusal is a metadata write, and that
is #54's rule rather than this one, because it is about artwork giving way to
metadata rather than about the device.

## Why this is written down before the code

An unbounded cache is not a decision anybody takes. It is what exists until
somebody adds a bound, and the moment it is added is after the first report from an
operator whose television stopped working, at which point the answer has to be
shipped through eleven clients to devices that are already full.

The eviction rule is the half that gets decided by whichever data structure was
nearest. A map with no order gives arbitrary eviction, which behaves differently on
every run and cannot be tested; a list gives insertion order, which is age order,
which is the option this record refuses with a reason. Neither is chosen, and
neither is visible in review as a choice.

The index is the part that cannot be added afterwards at any price. A cache written
for a year with no index has entries whose keys nothing recorded, and there is no
listing operation to find them with. The repair is to discard every entry on every
device, once, on the day it is noticed, and to ship the code that discards them.

None of these has happened in this tree, because there is no code in it, and that
is what makes the record cheap now.

## Alternatives, and what each cost

No bound, with the client responsible for clearing the store. Nothing to count,
nothing to evict, no index, and the client already owns the storage. It puts the
decision in front of eleven client authors who will each discover it at a different
time, and the ones who never discover it ship the television that stops working.

A bound in entries rather than bytes. Trivial to count and it needs no length
bookkeeping. It is wrong by three orders of magnitude between a metadata entry and
an artwork one, so the bound would have to be per kind anyway, and it would still
say nothing about the device's actual occupancy, which is the thing that runs out.

Enforcing the bound against the store's own reported size. One number, no index of
lengths, and it is the number that is actually true of the device. It costs the
same bound meaning different things on different platforms, because of block
rounding and whatever the store adds, and it costs a read of the store on the path
of every write.

A listing operation on the store, so the store owns enumeration and the core needs
no index. 0040 already refuses it and gives the reason: it cannot be implemented
cheaply over every platform's key-value facility and it enumerates in a different
order on each, so the single eviction rule becomes one per client.

Least recently used across the whole cache with no tiers. One order, one bound, and
no split to keep in agreement. 0006 already refuses it, and the failure is precise:
a scroll through a large library is thousands of artwork reads and writes, which
pushes every metadata entry to the front of the eviction order, so the library
listing that made the scroll possible is the first thing removed.

Evicting the core's own entries when the device is full. It is the reflex, it makes
the next write succeed, and it is measurable as a higher write success rate. It
spends the cache on whatever else filled the device, and it leaves nothing to serve
at exactly the moment the cache is the only thing that can serve anything.

Writing the index on every change. No window in which entries can be orphaned, and
the start-up reconciliation would not be needed. It doubles the number of store
writes the core makes, on the waiting lane, which is the cost 0040 weighed the
whole no-listing design against.

## What would reverse this

The index costs more in memory the core holds than a listing operation would have
cost every client. 0040 already names this as its own reversal condition and #65 is
where the measurement would come from. The store then grows an operation and both
records are superseded.

The orphan reconciliation is found to trigger on a device that never lost power,
which would mean the store's count and the core's are not comparable at all. 0040
names that case too, and the answer there is that the store's number becomes
reporting only, which removes this record's only defence against an unbounded
store.

Two hundred and fifty six mebibytes is measured against the cold-start number in
#62 and found to be either far more than is ever used or too little to hold a
first screen. The number moves in a record that supersedes this one, so that the
arithmetic above stays readable beside what replaced it.

An eviction is observed removing an entry a caller was holding, once. There is no
second instance to wait for: the rule above is either implemented or it is not, and
one occurrence means it is not.
