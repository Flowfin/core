# 0006. The cache contract

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #6

## The decision

The cache holds only what a server sent and never anything secret, every entry is
keyed by the server, the account and the device together so that no two of them
can read each other's entries, an entry stops being trusted by explicit
invalidation, by the server reporting a change, or by age, with which of the three
applies fixed per kind of entry rather than per call, a read may return a stale
entry and always says that it did, and the bytes live wherever the client put
them because the core is handed a store and never chooses a place.

## The one guarantee

A client that reads nothing else can rely on this: anything the cache returns was
sent by the server this session is against, for the account this session is for,
and the answer says how old it is and whether it is still trusted.

Everything below is how that is held. Nothing below weakens it.

## What is cached, and what never is

Cached: responses to library queries, the item metadata inside them, the
capability answers a server gave under #10, artwork bytes and their decoded
dimensions, and playback positions recorded while the server was gone, which is
the queue in #47 rather than a cache of the server's own answer.

Never cached: the session token, and anything derived from it. The token is the
only secret in #5, so it is the only thing this rule has to be about, and it is
not weakened by being small. It is not a cache key, not part of a cache key, not
a value, not a field inside a cached response, and not present in whatever the
store writes alongside an entry. The secret store in #33 is a different interface
with a different implementation, and the reason the two are separate interfaces
rather than one with a flag is that a flag is a thing somebody sets wrongly once.

Also never cached: anything the server marked as not to be stored, and any
response to a request that failed. A failure is not an answer, and a cached
failure is how a server that recovered stays broken for a client until something
evicts it. The error kinds in #4 that a client might reasonably want to remember,
the stable ones, are the client's to remember and not the core's to write down.

`no-store` on a response is obeyed. Where the server says a thing may not be
kept, it is not kept, whatever this record would otherwise allow, and there is no
setting that turns that off.

#48 is where the absence of secrets is proven rather than asserted, and until it
lands this section is a rule with no mechanism behind it.

## How an entry is keyed

The key is built from four parts, in this order, and all four are always present.

The server, as the resolved identity of the server rather than the address a
person typed. Two addresses that reach one server are one server, and one address
that reaches two servers over time is two.

The account, as the identifier the server gave back at sign-in. Not the username,
which a person can change and which two servers can reuse for two people.

The device identity from #36, because a cache written on one device is never read
on another and including it makes that structural rather than incidental.

The request, as the endpoint and the parameters that change the answer, in a form
where two requests that differ produce two keys. Parameters that do not change
the answer are not in the key, and which those are is a per-endpoint fact that
belongs with the code rather than in this record.

Two consequences follow and both are the point. Two servers cannot collide,
because the first part differs. A fresh sign-in as a different account cannot
read the previous account's entries, because the second part differs, and the
previous account's entries are not deleted by the new sign-in either, which is
what makes switching back cheap. Signing out is #114 and does not by itself
destroy the cache; what a sign-out removes is the token, from the store that
holds tokens.

#41 owns the exact construction, including how the parts are joined so that no
value of one can be made to look like the start of another.

## When an entry stops being trusted

Three routes, and each kind of entry uses a fixed subset of them.

Explicit invalidation. The core drops an entry because it knows the answer
changed, most often because the client asked for something that changes it. This
applies to every kind.

The server reporting a change. A validator the server supplied, or a change
notification where the server offers one, which is #116. Applies to library
queries and item metadata. It does not apply to artwork, which is addressed by a
tag that changes when the image changes, so a changed image is a different key
rather than a stale entry.

Age. Every kind has one, because the first two routes both depend on a server
being reachable and the whole reason a cache exists here is the case where it is
not.

Which route applies, by kind:

| Kind | Explicit | Server-reported | Age |
| --- | --- | --- | --- |
| Library query results | yes | yes | yes |
| Item metadata | yes | yes | yes |
| Capability answers for a server | yes | no | yes, long |
| Artwork bytes | yes | no, the address changes instead | yes, long |
| Decoded dimensions | yes | no | same as their bytes |

The numeric ages are not in this record. They are a tuning decision that will
move with measurement, and a number written here becomes the number two other
documents restate. #43 owns them, alongside what a stale read looks like.

## What a caller may assume

A read may return a stale entry. This is not a fallback that only happens when
the server is gone; it is the normal path for the first screen, because the
published cold-start number is spent before the network can answer, which is #46.

Every answer says which it is. Three states, and a client can act on all three:
fresh, meaning within its age and not invalidated; stale, meaning past its age
and returned anyway; and absent, meaning nothing was cached. A stale answer also
carries how old it is, so that a client can decide between showing it and showing
nothing without needing a rule from the core about how old is too old.

A caller can demand a fresh one. A read that requires freshness never returns a
stale entry; it either returns a fresh answer or one of the failure kinds in #4.
This is the mode a client uses after a person asked for a refresh, and it is the
mode nothing else should use, because a client that demands freshness everywhere
has turned the cache off and will find out at the cold-start measurement.

A caller cannot ask for a stale entry only. There is no read that refuses a fresh
answer, because a caller wanting the old value wants a value it already has.

A read never blocks on the network in order to answer from the cache. Serving a
cached entry while a request for a fresher one is outstanding is the behaviour in
#7, and this contract is what that behaviour reads.

An entry written by another version of the core, or half written, is not covered
here. #105 decides what such an entry is allowed to do, and until it does, this
contract describes only entries this version wrote completely.

## Where the bytes live

The core is handed a store through the interface in #40 and never looks for a
place to write. It does not read an environment variable, does not ask the
platform for a cache directory, and has no default.

Where an application may write differs on every platform this repository targets,
and on several of them the answer also depends on how the application was
packaged. A core that guessed would be wrong on some of them, and wrong in the
way that is discovered by an operator rather than by a test.

The store is an interface over entries rather than over files. The core hands it
a key and bytes and asks for bytes back. Whether that is a directory, a database
or something the platform supplies is the client's, and the core learns nothing
about it. This is also what keeps the core testable without a disk, which is the
birth requirement in #20.

The bound on total size and what is evicted when it is reached are #42, and
artwork gets its own tier with its own bound in #54, because one bound shared
between a hundred kilobytes of metadata and two hundred images is a bound that
evicts the metadata every time.

## Why this is written down before the code

A cache with no written contract is a cache each client learns by experiment, and
what it learns is whatever the implementation happens to do that month. Every
such observation becomes a client's assumption, and the assumptions are
discovered later, one at a time, by changing the implementation and finding out
which client broke.

Two of the five answers above cannot be corrected afterwards at any reasonable
price. The keying is one: a cache keyed without the account works perfectly until
the day a second person signs in on a shared television, and by then the entries
are already written under the wrong keys, so the correction has to invalidate
everything and has to be shipped to every client at once. The other is the store
being supplied rather than chosen. A core that picked a path on disk has that
path in every client's data directory and in every operator's backup, and moving
it later is a migration with no owner.

The stale-and-says-so answer is the third thing, and it is unrepairable in a
different way. A cache that returns stale entries silently is indistinguishable
from a working one right up until somebody sees a film they deleted last week,
and by then no client has anywhere to put the words that would have explained it,
because none of them asked for a freshness field.

## Alternatives, and what each cost

A cache the core places on disk itself. One less interface, and the core can be
sure the layout is what it expects. It costs the platform question having a
different answer on every target, and it puts a path the core invented into every
operator's machine permanently.

Keying on the server only. Simpler keys, and correct for the single-account case
that is most installations. It costs the shared device, which is a television,
which is the case this project cares about most, and the failure is one person
seeing another person's library rather than an error.

Freshness as a single time-to-live for everything. One number, easy to reason
about and easy to tune. It costs the difference between an artwork blob that is
immutable at its address and a library query whose answer changes when anybody
adds a file, and any single number is badly wrong for one of them.

A read that returns stale entries without saying so. The interface is a plain get
and every caller is simpler. It costs the client the ability to say anything
about what it is showing, and it makes the offline behaviour in #45 impossible to
present honestly, because the client cannot tell the two cases apart.

No cache, and a request every time. Nothing to invalidate, nothing to key, and
never wrong. It costs the cold-start number outright, and it costs the offline
behaviour completely, which are two of the things this repository exists to
provide.

Caching the token alongside everything else, in one store. One interface instead
of two, one place to clear on sign-out. It costs the property that makes #48
provable: with two stores, the proof is that the cache store never receives the
token, which is a thing a test can watch. With one store, the proof is about a
flag on a call, which is a thing a test can only watch at the call sites it knows
about.

## What would reverse this

A client is found to have needed a read mode this contract does not offer, twice.
One is a client doing something unusual. Two means the three-state answer is the
wrong shape, and this record is superseded by one describing the shape that was
actually needed.

#105 decides that an entry written by another version may be served rather than
discarded. That widens what the one guarantee above covers, since a served entry
would then have come from a version whose keying rules may differ, and this
record is superseded by one that states the guarantee against a cache with a
history rather than against a cache one version wrote.

The measurement in #62 shows the cold-start number is met without serving stale
entries at all. The stale path is then complexity bought for nothing, and it is
removed with its own record rather than left in place because it was written
here first.

Server-reported change under #116 turns out to be available for artwork as well,
which would happen if the addressing in #49 stops being content-tagged. The table
above is then wrong in a row, and the record is superseded rather than corrected,
because the row is a decision and not a typing error.
