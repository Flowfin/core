# 0105. An entry this version did not write, and one that was not finished

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #105

## The decision

Everything the core writes through the store in 0040 is wrapped in a small
envelope of the core's own carrying a format version, the kind of the payload,
the payload's length and a digest over it, and an entry whose envelope does not
parse, whose version is not the one this build writes, or whose length or digest
does not match what was read back is dropped where it was found and treated as
absent, without being read, without touching any neighbour, and with the drop
reported through 0100 rather than performed in silence.

## The envelope, and why the store is not asked

0040 gives the store four operations and no fifth: read one entry, write one
entry, remove one entry, say how much is held. None of them says whether a write
finished, and none of them can be made to. The store is the client's, it may be a
directory, a database or whatever a platform supplies, and the atomicity of a
write differs across all three. So the question of whether the bytes that came
back are the bytes that went in is answered by the bytes themselves.

The envelope holds four things and is written in front of the payload.

A format version, being the version of the envelope and of the payload shape
inside it, as a number this build writes and compares against.

The kind, from the list 0006 gives of what is cached, so that a payload is never
handed to a reader for a different kind. A key collision cannot produce this
under 0041, and a defect on the write path can, which is the case the field is
for.

The length of the payload in bytes.

A digest over the payload, at the width 0041 already requires of a key.

A read parses the envelope first, checks the version, checks that the payload is
the stated length, and checks the digest, in that order, before any part of the
payload is looked at. Any of the four failing produces absence.

This adds no operation to the store and the record for 0040 named that as a
reversal condition, so it is worth saying plainly that the condition has not
happened. A fifth operation would have been the store telling the core whether a
write completed, and the reason it is not needed is that the answer is derivable
from what a read already returns.

## A version that does not match

The entry is discarded. It is not read, not partially read, and not repaired.

The issue this record comes from says that discarding is a legitimate answer and
reading it anyway is not, and asks for the reason rather than the choice. The
reason is that the alternative has no stopping point. Reading an older entry
means knowing the older shape, which means the build carries every shape it has
ever written, and each of those has to be tested against a fixture nobody can
regenerate because the version that wrote it is gone. What is actually built in
that situation is a reader for the last shape and a hope about the ones before
it, and the failure mode is the one this issue opens with: a misread field rather
than a missing file, so somebody is shown the wrong thing instead of an error.

A newer version than this build writes is discarded by the same rule and not by a
separate one. Downgrading a client is ordinary, and a build that tried to read a
shape defined after it was written would be guessing.

What makes discarding cheap here is that a cache entry can be fetched again.
Every kind in 0006's table is a copy of something the server holds. The one thing
in the store that is not a copy is the queue in 0047, and it is answered below
under its own rule rather than inheriting this one.

There is a neighbouring version already decided elsewhere and the two must not be
confused. 0041 puts a version tag inside the digest that forms the key, so
changing the key construction makes an old key space unreachable rather than
misread. That covers a change to how a key is built. This record covers a change
to what is stored under a key that is still reachable, which is the case 0041
explicitly leaves here. Both can move in one release and they move independently:
a key-space change hides the old entries from every lookup, and an envelope
version change means the lookups that still land on an old entry drop it.

## A write that did not finish

A device losing power, a process the platform killed for using too much memory,
and a store implementation whose own write was partial all leave an entry that
exists and is not what it claims to be. On a television the power case is the
normal one rather than the exotic one.

The length and the digest together are what tell a complete entry from a
truncated one, and each is doing a different job. The length catches the ordinary
truncation, cheaply and before anything is allocated for the payload. The digest
catches the case the length misses, which is a write that was replaced in place
by a shorter one and left the tail of the previous entry behind it, so the bytes
are the stated length and are two entries end to end.

What the digest does not buy is stated here rather than left to be assumed, and
it is the sentence that would otherwise be quoted back as a property this
repository has. It is not authentication. Anything that can write the store can
write a matching digest, and 0101 already treats every byte read back out of the
store as untrusted for exactly that reason. The envelope detects an entry that
was damaged, not one that was chosen. Parsing a payload whose digest matched is
still parsing untrusted input, with every bound 0101 puts on it, and nothing in
this record moves the store inside a trust boundary.

## What a drop takes with it, and what it does not

One entry. The entry that failed is removed through the store's own remove
operation and the read answers absent, so the caller fetches it the way it would
fetch anything that was never cached. Nothing else is examined, nothing else is
removed, and no scan of the store is started.

Clearing the cache on a bad entry is the shape this refuses, and it is worth
naming because it is the shortest implementation and it reads as the careful one.
One truncated file then becomes a cold start, which is the number #46 exists to
protect, and it becomes a cold start on exactly the device where power was lost,
which is the device least likely to have a fast network. Worse, it is
self-concealing: the cache is empty, so nothing is left to diagnose from, and the
symptom is a slow application rather than a broken file.

A drop is silent to the person and never silent to a report. Nothing is shown, no
error reaches the caller, and the fetch that follows is an ordinary fetch. What
the caller sees is absence, which 0006 already gives it as one of three states.

## What is reported

Two things, for the reason 0047 gives for reporting a dropped queue entry twice
over: an event reaches a client that was listening at that moment, and a count
reaches one that was not.

An event through 0100 at each drop, at `notice`, with a stable identity and
fields naming the kind of the entry, which of the three checks failed, and the
version that was found where a version was the thing that failed. The key is not
a field. What identifies the entry is the correlator 0071 defines, for the same
reason 0047 carries a correlator rather than a target.

A standing count the core keeps for the run, readable through a call that cannot
wait in the terms of 0009, separated by which check failed. A cache that empties
itself on every start presents as a slow network and can stay that way for a long
time, and the difference between one drop after a power cut and four hundred
drops on every start is the whole diagnosis. One event per drop is what makes
that visible in a report; the count is what makes it visible without one.

`notice` rather than `failure` for a cache entry, because nothing the caller
asked for failed. The severity for a dropped queue entry is different and is
below.

## The queue is not a cache, and its rule differs

0047's entries are things a person did. They cannot be fetched again, and the
issue this record comes from asks whether the queue inherits the rule written for
something cheaper. It does not.

The envelope is the same. Each queue entry carries one, and so does whatever the
queue keeps that is not an entry, which under 0047 is the counter that fixes the
order.

Three things differ.

A dropped queue entry is reported at `failure` rather than at `notice`, and it
counts into the standing drop count 0047 already keeps for entries displaced at
the bound. Silently discarding a person's action is the failure 0047 exists to
prevent, and a drop for a bad envelope is that same discard arriving through a
different door. It reaches the same counter so that a client reporting what was
lost reports one number rather than two that have to be added up by whoever reads
it.

The drain steps over a bad entry rather than stopping at it. 0047 drains in
counter order and stops at the first entry that could not be delivered, because
continuing would deliver a later action ahead of an earlier one. A corrupt entry
is not an undelivered entry. It is gone, waiting will not recover it, and
stopping there would strand every later action behind something that will never
succeed. So it is dropped, reported, and the drain continues.

A counter that fails its own envelope does not empty the queue. It is rebuilt as
one above the highest counter among the entries that survived. The only property
the counter has to hold is order among what is there, and 0047 already fixes that
it is a counter rather than a timestamp precisely so that it is not a claim about
time. Discarding a queue because its counter was truncated would throw away
somebody's actions to protect a number that can be recomputed from them.

## What this does not decide

The bound on the store and what is evicted when it is reached. #42, which already
says in its own words that an entry dropped for a version or a completeness
failure is this record rather than an eviction.

Which digest function and which width. That follows the toolchain in #11 in the
same way 0041's does, and this record states the requirement rather than naming a
function that does not exist yet.

What is cached at all and when an entry stops being trusted for age. 0006 and
0043. An entry that fails a check here is absent, not stale, which keeps 0043's
three states at three.

What a store may report as a failure. 0040, and `storage-unavailable` is not what
a failed check produces, because the store worked and returned what it held.

## Why this is written down before the code

Two landed records send this question here by name and neither can answer it.
0006 says its contract describes only entries this version wrote completely and
names this issue for the rest. 0040 says the same of its interface. So the first
piece of code with something to read back finds the question named in both places
and decided in neither, and it answers it at a call site.

The answer a call site produces is predictable, because it is the one that is
invisible while it is being written. Bytes come back, they parse, and they are
used. There is no envelope, because nothing on a developer's machine ever
truncates a write, and the check that would have caught it has no failure to
demonstrate itself against. The defect ships and surfaces on a television that
lost power, as a wrong field rather than as an error, and it is reported by an
operator as something looking odd rather than as a fault anybody can reproduce.

Written afterwards, this is a migration rather than a record. Every entry already
in every operator's store was written without an envelope, so the reader has to
accept both shapes, which is the version-tolerant reader this record refuses, and
it has to accept it permanently because nothing can tell the two apart except by
guessing at the first bytes.

The cost of writing it now is one envelope on the write path before there is a
write path. That is the cheapest this decision will ever be.

## Alternatives, and what each cost

A migration path per version, reading an older shape and rewriting it in the
current one. It keeps a warm cache across an upgrade, which is real: the upgrade
is the moment a person notices, because they just did something and are looking
at the result. It costs a reader per historical shape, kept forever, tested
against fixtures that can only be written by hand once the version that produced
them is gone, and it puts the whole of that surface on the untrusted side of
0101. The thing it protects is a cache that refills itself in one session.

A tolerant reader with no version at all, using whatever fields it recognises and
ignoring the rest. Common, cheap, and it survives most changes without anybody
thinking about them. It costs the one case that matters: a field whose meaning
changed rather than whose name changed is read confidently and wrongly, and there
is nothing in the entry that would have said otherwise.

Relying on the store to write atomically, by writing to a temporary name and
renaming. It is genuinely correct on a filesystem, and it removes the digest. It
costs every platform where the store is not a filesystem, where there is no
rename with those properties and a client would have to fake one, and 0040 gives
the core no way to require it or to check that a client did. It also puts a
correctness property of the core into eleven implementations, which is the drift
this repository exists against.

A length and no digest. Smaller, and it catches the truncation everybody pictures
when they think about this. It costs the replaced-in-place case, where a shorter
write leaves the previous entry's tail behind and the total length is whatever it
was before, which is the one shape a length check cannot see.

Clearing the whole cache when any entry fails a check, on the grounds that one
bad entry means the store is suspect. It is simple and it is defensible on a
filesystem that lost a directory. It costs a cold start for one file, on the
device that just lost power, and the cost is paid at every start until the
underlying problem is found, which is hard because the evidence was cleared.

Dropping a bad entry with no report. Everything works, the entry refetches,
nobody has to handle anything. It costs the diagnosis outright. A store whose
writes never complete produces an application that is slow for reasons nothing
records, and the shape of that report is a person saying it feels slower than it
used to.

Treating a corrupt queue entry the way a corrupt cache entry is treated, at
`notice` and outside the drop count. One rule instead of two, and the queue code
gets shorter. It costs the promise 0047 made, which is that a discarded action is
reported, and it hides the discard in the severity a client is expected to filter
out.

## What would reverse this

An envelope check fires in normal operation on a device that lost nothing,
meaning the digest or the length is being tripped by something other than damage.
That is evidence the write path or the store interface is wrong rather than the
entry, and this record is superseded by one written against whatever was actually
found.

The measured cost of digesting every payload on the write path is a visible share
of the cold start in #46, taken from the harness in #65 rather than supposed. The
envelope then narrows to a version and a length, with the replaced-in-place case
stated as an accepted residual, and this record is superseded by one that says so
in the open.

A payload shape changes in a way that would genuinely have been readable by the
previous build, twice. One is a change that should have been additive and was
not. Two means the discard rule is throwing away entries it could have kept, and
the record is superseded by one that states which changes are additive and how a
reader is allowed to depend on that.

A store implementation is found that can report whether a write completed, on
every platform this repository targets. The envelope's completeness half is then
duplicating something the interface could carry, 0040's fifth operation becomes
worth its cost, and both records are superseded together rather than one of them
being bent around the other.
