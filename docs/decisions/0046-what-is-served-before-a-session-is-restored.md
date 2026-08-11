# 0046. What is served before a session is restored

Date: 2026-08-11

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #46

## The decision

A cache read is answerable from the moment the core is created, because 0033
already makes a session's identity the client's to hold and 0041 builds a key from
that identity alone, so the start path reads the byte store for a session the
client names without waiting on the secret store and without any network call;
what it can serve that way is every kind 0006 caches, each marked with one of
0043's three states; and what it cannot serve is every call that needs a token,
which is listed here rather than discovered one call site at a time.

## Why the cache is reachable before the secret is

The reason this works at all is a property two earlier records already fixed for
their own reasons, and it is worth naming because it is the thing that would be
given up by accident.

0005 makes a session the server, the account and the device together, and makes
everything in it except the token ordinary data. 0033 has no listing, so a client
that wants to restore a session at start already holds those three parts itself
and hands them to the core to name the secret it wants read. 0041 builds a cache
key out of a version tag and those same parts plus the request, and the token is
not among them, because 0006 forbids the token from being a key, part of a key, a
value or a field inside one.

So every input a cache key needs is in the client's hands before the core has read
anything. Nothing in the key comes back from a sign-in. The account identifier is
the one part that looks as though it should, since 0006 defines it as the
identifier the server gave back at sign-in, and it is a value the client kept from
the last one rather than a value this start has to earn.

That is the whole mechanism. A cold start serves from cache because the key space
is addressable without a secret, not because anything is skipped or assumed.

## The start path

Creation reaches nothing, which 0115 fixes and which this record depends on rather
than restates: a core that has just been created has started its two lanes and has
done nothing else.

The client then names a session. From that point three things are in flight and
none of them waits on another.

The cache read for whatever the client asked for. It is an ordinary read through
the byte store, so it is asynchronous under 0009 and it answers with 0043's state
and age.

The secret read, which 0033 makes a read of one name. It either produces a token,
produces the absence that means sign in again, or fails as `storage-unavailable`.

The request to the server, once there is a token to send it with, reported in
0007's stages.

The order matters in one direction only. The cache read must not be sequenced
behind the secret read, and that is the sentence this record exists to make
refusable. 0033 already names the case that makes it concrete: a device locked at
the moment of a background start fails the secret read rather than answering it,
and a start path that read the secret first would then show nothing at all while a
complete answer sat in the store, on the one device where the person is most
likely to be somewhere with no network either.

Nothing here weakens 0006's guarantee. An answer served this early was still sent
by the server this session is against, for the account this session is for, and it
still says how old it is.

## What is servable, and what is not

Everything 0006 lists as cached is servable before a session is restored, under
0043's states and thresholds, with no exception carved for this path:

| Kind | Servable before the session is restored |
| --- | --- |
| Library query results | yes |
| Item metadata | yes |
| Capability answers for a server | yes |
| Artwork bytes | yes |
| Decoded dimensions | yes |

What is not servable is not a list of entries. It is a list of calls, and the
distinction is the useful half of this section:

A playback handover, which is #111 and is a call to the server rather than a
lookup.

Any report or write toward the server, including a position report. Those go onto
the queue in #47, which a start does not drain until there is a token to drain it
with.

A read that demands freshness, which 0006 fixes as returning a fresh answer or a
named failure and never a stale one. Before there is a token the failure is what
it returns.

Sign-in itself, on any of 0005's three routes.

The case #46 names as the one to be careful about, a cached playback
authorisation, turns out to have been answered before this record: 0006 does not
cache one, because it is derived from the token and the token is the only secret.
So there is no entry to withhold and nothing here has to remember to withhold it.
That is the good outcome, and it is worth writing down that it came from the cache
contract rather than from a rule on this path, because a later change that started
caching anything derived from a token would break this section without touching
it.

## A cached answer is not a signed-in state

The failure this section is against is a screen that looks signed in because it is
full.

A cache read never moves a session's state. A session whose secret has not been
read yet is not restored, a session whose token the server has since ended is not
restored, and neither becomes restored by the core answering a library query out
of the store. The session state is a call that cannot wait under 0009 and it is
read on its own, so a client that wants to know has one place to ask and does not
infer it from having received data.

The core also does not decide who is holding the device. Naming a session is the
act that exposes that session's cache, and that act is the client's. A client that
wants a person to prove who they are before their library appears does that before
naming the session, not after, and the core cannot do it for them because it has
no way to ask anybody anything. 0041 keeps two accounts on one device out of each
other's entries; it does not and cannot keep one account's entries from whoever
picked the device up.

Both halves of that are stated because the convenient reading of a cold start that
works offline is that the core has decided the person is allowed to see this. It
has decided only that the entries under the named key were written by this server
for this account on this device.

## Where the measurement points are

No new endpoints. 0008 already opens the core's interval at the call that issues
the first library query after the core was created and closes it at the return of
the first decoded artwork bitmap belonging to an item in that answer, and it
already separates an empty-cache variant from a warm-cache variant. This path is
that warm-cache variant, and #46 asking for measurement points at both ends of it
is answered by pointing at them rather than by naming a second pair.

A second pair would be the more obvious thing to do and it is the thing this
record refuses. Two names for one interval is how two numbers for one question get
published, and the one that is quoted afterwards is whichever was measured most
recently.

What the interval does not separate is the store read from the work around it. A
client whose supplied store is slow and a core that is slow produce the same
number, and telling them apart needs the span data in #61 rather than this
interval. Nothing in this record is measured. There is no language chosen, no
build and no test command in this tree, and the workflows it carries are

    $ ls .github/workflows/
    dco.yml
    dependency-review.yml
    scorecard.yml
    unicode-guard.yml
    zizmor.yml

none of which runs anything described here. #62 is where this path becomes a
number and #65 is the harness that produces it.

## Why this is written down before the code

Without it the ordering gets chosen by whoever writes the first start path, and
the natural way to write one is the wrong way round. Restoring a session reads as
the first step because everything else needs a session, and a cache read placed
after it is correct on every machine a developer tests it on, where the keychain
is open and the network is there. It is wrong on a locked device, wrong on a train
and wrong on the measurement the number in #62 is taken from, and all three arrive
at once because they are the same start.

The second thing it prevents is quieter. A start path that has to work before a
session exists is under steady pressure to make the cache readable without naming
an account, since that would remove the one input it has to be given. That change
is the disclosure 0041 and 0101 are written against, and it would arrive as a
simplification of this path rather than as a change to a key.

## Alternatives, and what each cost

Restore the session first and read the cache afterwards. One ordering, no
concurrency at the start, and the smallest possible amount to reason about. It
costs the case the cache exists for. The secret read can fail or hang on exactly
the device where the network is also absent, and the answer already on disk is
then withheld for a reason unrelated to it.

Read the cache but hold the answer until the token is known good. Produces a
screen that is correct and late, which is the shape #7 already refuses for a slow
server, and it makes a cached answer depend on a network round trip that the cache
exists to survive. It also produces the worst version of the honesty problem in
the section above, because the answer shown is then genuinely gated on
authentication and the next change that removes the gate for speed removes a
property nobody wrote down.

An offline mode the client switches on, with a separate path through the core.
Every offline behaviour in one place, easy to explain, and it is two mechanisms for
one promise, which 0005 already refuses for the same reason: the second one is
tested less, and here the second one is the path that runs on every start where
something is wrong.

Key the cache without the account so that a read needs nothing named at all. This
is the cheapest start path that can exist and it is refused outright. 0006 and
0041 place a collision between two accounts as a disclosure rather than a stale
answer, and dropping the account from the key is not a collision risk, it is the
disclosure by construction.

Serve nothing before the session is restored and publish a slower cold start. The
honest version of the first alternative, and it costs the number in #62 and the
kickoff's reason for having a cache at all.

## What would reverse this

A change to 0033 or 0041 that makes a session's identity or a cache key depend on
something only a sign-in produces. The mechanism in the first section is then gone
and this record is superseded by one describing what is reachable instead, rather
than being kept alive by an exception.

#114 deciding that naming a session requires a token, for a reason about who is at
the device. That would be a defensible answer to the second half of the honesty
section and it removes this path as written.

A measurement from #65 showing that the cache-first ordering and the
restore-first ordering produce the same warm-cache number on every platform the
answer to #113 covers. The ordering is then buying nothing, and the simpler one
wins. The measurement has to include a locked secret store, because that is the
case the ordering is for and a run that omits it would show exactly this result.
