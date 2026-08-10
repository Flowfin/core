# 0040. The cache store interface, and a core with no store

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #40

## The decision

Cached bytes reach durable storage only through a store the client supplies, whose
whole surface is reading one entry, writing one entry, removing one entry and
saying how much it holds; every one of those is asynchronous, a read that finds
nothing is an absence rather than a failure, the only failure it may report is
`storage-unavailable`, and a core handed no store at all keeps what it can in
memory for the life of the process and says so through a capability a client can
ask about rather than behaving as though it had a cache.

## The surface, and why it is this small

Four operations.

Read an entry by key, answering with its bytes or with the fact that there are
none.

Write an entry: a key and bytes, replacing whatever was there.

Remove an entry by key. Removing one that is not there succeeds, because a caller
that has to ask first has a race to lose.

Say how much is held, in bytes, as the store counts them.

There is no listing, no iteration, no prefix scan, no transaction and no
expiry. 0006 already says the store is an interface over entries rather than over
files, and this is what that costs a client: four operations somebody implements
once over a directory, a database, or whatever their platform supplies, with
nothing in the list that only some of those can do.

The one that is missed first is listing, so it is worth saying what pays for its
absence. Eviction in #42 needs to know what is there and in what order, and with
no way to ask the store, that is the core's own bookkeeping. It is a real cost and
it lands on #42 rather than being spread across every client. The alternative
spreads it: a store that can enumerate on a filesystem, cannot enumerate cheaply
over a platform key-value facility, and enumerates in a different order on each,
so the eviction rule 0006 promises is one rule becomes a rule per client. That is
the drift this repository exists against, arriving through a method signature.

## What may fail, and what is not a failure

An absent entry is not a failure. 0006 already gives a caller three states, fresh,
stale and absent, and a read that finds nothing produces the third. A store that
reported absence as an error would make the ordinary cold-start path in #46 look
like something going wrong.

The only failure a store may report is `storage-unavailable` from the vocabulary
in 0004. A full device, a store the platform closed underneath the core, a
permission withdrawn while the application was in the background. The core does
not need to tell those apart, because its answer is the same for all of them, and
what that answer is belongs to #42.

A failed write never fails the call that caused it. Somebody asking for a library
list gets the library list; the cache is an accelerator, and a device that has run
out of room is not a reason to show an empty screen in front of a working server
and a valid session. A failed read is the same shape: the entry is absent and the
network answers.

Bytes that come back changed, truncated or written by another version of the core
are not this record's. #105 decides what such an entry is allowed to do, and until
it does, everything here describes entries this version wrote completely.

## What the store never sees

No secret. The token and anything else that authenticates go through the separate
interface in #33, and 0006 states the reason the two are separate rather than one
store with a convention: with two interfaces the proof is that the cache store
never receives the token, which is a thing a test can watch, and #48 is where it
is watched.

The store sees keys and bytes and learns nothing else. What a key is made of, and
therefore the guarantee that two servers and two people on one device cannot read
each other, is #41. A store implementation is not asked to understand a key,
parse it, or keep any structure that follows from it.

## Threads

0009 already puts every read and write through this interface off the thread that
called into the core, on the waiting lane. Two consequences an implementer needs
and would otherwise have to infer.

An implementation may block. It is never called on a thread a client draws on,
because it is never called on a thread a client called in on. A store over a
filesystem does not need to find an asynchronous file interface on every platform
to be correct here.

The core holds no lock of its own across the call, which is 0009's rule for
calling out to client-supplied code. That is what makes a slow store a slow store
rather than a stopped core.

## A core with no store

It works. Nothing refuses to run, no call starts failing, and the guarantee in
0006 that a cached answer says whether it is fresh, stale or absent holds
unchanged. What changes is that everything is absent again after the process ends.

It says so. There is a capability a client can ask about, and it is a call that
cannot wait in the terms of 0009, so a client can ask it while deciding what to
draw. A core that quietly behaved as though it had a cache would let a client
measure the cold start in #46 against a number that never happens on a real
device, and would leave the operator documentation in #74 unable to answer what is
stored on the device, which in this configuration is nothing.

What it holds in the meantime is what it can, in memory, for the life of the
process, under the same bound and the same eviction as a store-backed cache. There
is one cache in the core and one set of rules for it; the store is where entries
survive a restart, and its absence is not a second cache with second behaviour.

This is not the same thing as the in-memory store the suite runs against, and the
two are worth keeping apart because they look identical from outside. The suite's
store is a supplied store that happens to keep its bytes in memory, and it is
under test as a store: it can be made to fail, made to be full, made to answer
slowly. A core with no store is the absence of one, and what is under test there
is the absence. Collapsing them would leave the second case with no test at all
while every run appeared to cover it.

## What this does not decide

The bound on total size and what is evicted when it is reached. #42.

Artwork's separate tier and its own bound. #54.

What a key is made of. #41.

What an entry written by another version, or half written, may do. #105.

What is cached at all, what never is, and when an entry stops being trusted. 0006.

Where the bytes go. The client's, and 0003 and 0006 both refuse the core choosing.

## Why this is written down before the code

Landed records already depend on this interface and none of them describes it.
0006 sends the core's bytes through "the interface in #40" and says it never
looks for a path. 0009 names "the byte store in #40" among the four things that
never run on a caller's thread. 0101 puts it outside the boundary in both
directions and treats every byte read back out of it as untrusted. 0003 refuses
the core a filesystem of its own on the grounds that a client supplies the
location. Those are the ones quoted here rather than all of them, and the set
moves, so it is derived:

    $ git grep -l '#40' -- docs/decisions

So the first subsystem with something to cache finds the interface named wherever
it matters and defined nowhere, and it invents one. The shape it invents is
predictable, because it is the shape a developer's own machine makes easiest: a
synchronous get and put over a file path, with a not-found returned as an error.
Each of those three defaults is wrong here for a different reason already written
down. A path decides the platform question 0006 refused. A synchronous call breaks
the guarantee in 0009 at the call site least likely to be tested, because a read
from a warm page cache on a developer's machine is fast enough that nothing looks
wrong. A not-found error makes the cold-start path in #46 indistinguishable from a
storage fault.

This has not happened in this tree, because there is no code in it. That is what
makes the record cheap now: one file today against every cache call site later.

## Alternatives, and what each cost

A richer store: listing, iteration, expiry, a transaction. Eviction and ageing
could then be handed to the store, and #42 would be a configuration rather than an
implementation. It costs every client author all of it, on four platforms, and it
costs the single behaviour: expiry implemented eleven times is eleven answers to
when an entry stops being trusted, which 0006 decides once on purpose.

A store keyed by file path rather than by opaque key. Familiar, and it makes a
directory implementation trivial. It costs the platforms where the answer is not a
filesystem, which then fake a path layout the core does not use, and it invites
the core to reason about a path, which is the thing 0006 removed.

The core owning a cache directory directly, with the client supplying only a
location. One interface fewer, and everything about eviction becomes ordinary file
work. Refused by 0003 and 0006 for the platform reason, and it also costs the
birth requirement in #20, because the suite would then need a filesystem to test
anything about the cache.

A synchronous surface, on the argument that a key-value read is usually fast.
Simplest to implement and it reads well at a call site. The word doing the work is
"usually": a cold read on a television's storage under load is not fast, and a
synchronous surface puts that wait on whichever thread called, which is exactly
what 0009 exists to refuse.

Failing the caller's call when a write fails. Consistent in a narrow sense, and it
surfaces a full device immediately rather than quietly. It costs the case it is
supposed to help: a full device would stop a library list from being shown even
though the server answered, and the person then has a working setup and an empty
screen.

Refusing to run with no store supplied, so that a client cannot get the degraded
behaviour by accident. Honest, and it removes the case that is hardest to test. It
costs the first thing anybody does with a new library, which is to construct the
core and call something before writing a store, and it costs a client that
deliberately wants no persistence.

## What would reverse this

A platform on which these four operations genuinely cannot be implemented, named
with which of them failed and why. Then the surface changes and this record is
superseded by one describing the smaller or different one.

Two client implementations whose answers to how much is held cannot be compared,
so that the bound in #42 means different things on two devices. Then the bound is
measured on bytes the core handed over instead, and this record is superseded by
one that says the store's own number is for reporting only.

#42's bookkeeping for eviction turns out to cost more, in memory the core holds
for its own index, than a listing operation would have cost every client. That is
a measurement rather than a feeling, and the harness in #65 is where it would come
from.

#105 decides that a half-written entry has to be recognised by something only the
store can know. Then the store carries one more operation and this record is
superseded rather than extended, because the argument for four is the argument
against a fifth.
