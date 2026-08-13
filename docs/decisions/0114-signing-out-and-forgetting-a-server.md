# 0114. Signing out, forgetting a server, and holding several sessions

Date: 2026-08-13

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #114

## The decision

Signing out and forgetting a server are two acts rather than one: a sign-out ends
one session, drops its token from memory, forgets its secret through 0033,
resolves that session's outstanding work into named kinds from 0004, leaves every
other session running and completes on a device with no network; forgetting a
server is a second act a client asks for, which does everything a sign-out does
and then removes every cache entry and every queued action under the key space
0041 fixes for that server and that account; and the queue survives a sign-out,
because an undelivered action is the one thing a person did that has no copy
anywhere else.

## Two acts, and why they are not one

Every client that was not told they differ collapses them into one button, and
which of the two that button does is then decided by whoever wrote it.

The two answer different questions. Somebody signing out is saying that the next
person to pick this device up must not be them. Somebody forgetting a server is
saying that this device should stop holding anything about that server at all.
The first is ordinary and happens on a television in a house every evening. The
second is what a person does before they sell the device, or when they have
finished with a server they were only visiting.

If sign-out did both, then signing back in on a device somebody already used
refetches the whole library over whatever connection they have, and 0006's
argument for keeping entries across a sign-in is lost. If forgetting a server did
only what a sign-out does, then there is no act at all that removes what the core
asked to be kept, and 0068's promise to an operator that they can have it removed
has nothing behind it.

So both exist, they are named separately in the interface, and the second is
strictly the first plus removal.

## What a sign-out does

It takes a session, which 0005 fixes as one server, one account and one device
identity together, and it does five things.

The token leaves memory. Nothing further is sent under it by this core.

The secret is forgotten through the interface in 0033, under the name that
record derives from 0041's construction. Forgetting one that is not there
succeeds, so a sign-out on a session whose secret was never written is not a
failure.

The session's outstanding work is resolved, in the terms of the next section.

The session moves to signed out and stays addressable. A signed-out session is
still a server, an account and a device identity the client can name, which is
what a later sign-in and a later forget both need, and 0033 already requires the
client rather than the core to be holding that triple.

The server is told, as one ordinary request under the bounds 0007 and 0038
already fix, and it is the only half that may fail. A token this core stopped
using is still a token the server accepts until the server decides otherwise, so
telling it is worth attempting and worth reporting. Where the attempt did not
succeed the client is told through 0100, with the fact that the token may still
be live at the server, because that is a thing an operator can act on from the
server's own side and cannot act on if nobody said it.

The first four happen whatever the network is doing. A sign-out on a train is a
sign-out, and it reports that the server was not told rather than refusing to
complete. Reversing that order, so that the local half waits on the server half,
produces the failure this whole act exists against: a person hands the device to
somebody else believing they signed out, and the token is still in memory because
a request timed out.

A sign-out of a session that is already signed out succeeds and changes nothing.
A sign-out forced by a refused renewal in 0034 reaches exactly this state, with
the server half skipped because the server has already refused that token, which
is what 0005 requires when it says the answer must be the same whether the
sign-out was asked for or forced.

## What a sign-out does not do

It does not touch any other session. Not another account on the same server, not
the same account on another server, and not the same account on the same server
from a second device, which 0005 already fixes as a second session because the
server issued a second token and may end either one alone.

It does not remove cache entries. 0006 and 0068 both landed this direction with
their reasons, and this record follows them rather than reopening them: the
entries are keyed per server, per account and per device, so leaving them costs
nobody else's privacy, and signing back in on a device somebody already used is
the case the cache exists for. This issue's own list of the parts of a sign-out
puts cache removal inside it, and the two landed records are the later answer;
what removes those entries is the second act below.

The consequence of that is stated here rather than left to be found. A cache read
needs no token, which is what 0046's cold start depends on, so a signed-out
account's library is still servable to whoever names that session on that device.
0046 already says as much, in the sentence that naming a session is the act that
exposes its cache and that 0041 keeps two accounts apart while doing nothing to
keep one account's entries from whoever picked the device up. This record does
not weaken that and does not improve it: it is the reason the second act exists,
and a client whose sign-out button has to mean the stronger thing calls the
second one.

It does not stop playback. 0005 guarantees that a token dying mid-playback does
not interrupt what somebody is watching, and an asked-for sign-out is not a
reason to break that guarantee either. The stream is being read by the platform's
own player against the address handed over in #111, and this core does not have a
way to stop it. What may happen is that the address stops working, because the
server was told the token is finished; that is the server's answer to the player
rather than something the core did, and the record says so here rather than
leaving a client to discover it as a defect.

It does not end sessions elsewhere. What a sign-out reaches is this device.

## Work in flight

Four kinds of work can be running when a sign-out arrives, and each ends in a
named state rather than in silence. 0009 already refuses reporting a cancelled or
undelivered thing as something it is not, and this is that rule applied to one
moment.

A request already sent on that session is cancelled and ends as `cancelled` from
0004. An answer that arrives afterwards is discarded rather than delivered, and
it is not written to the cache, because it was fetched under a session that has
ended.

A decode running for that session ends the same way, at the step boundary 0009
and 0115 already fix, since a decoder is not interruptible at an arbitrary
instruction. Bytes already handed to a caller are the caller's and are not
reached, which is the same boundary 0042 draws for eviction.

A read or a write already begun through the byte store completes. That is 0115's
rule and its reason is that the store is the client's own code, and it holds here
without exception: a sign-out does not abandon a call inside somebody else's
implementation.

A queued action stays queued, which is the section below.

Cancelling by session is what makes this possible, and it is the part that will
be got wrong. 0009 gives the core two lanes shared by every session, so the unit
of work on a lane carries the session it belongs to and the sign-out cancels that
set and no other. A sign-out that cancelled a lane's work would stop the other
account's tile wall on a television with two people signed in, and the person who
lost their screen did nothing.

## The queue survives, and what that costs

0047 holds one durable queue per session, and its contents are the actions
somebody took that the server has not been told about yet. 0068 names it as the
one piece of this data with no copy on the server.

So a sign-out keeps it. A later sign-in to the same server, account and device
finds the queue where it was, in its own order, and drains it before anything
else is sent, which is exactly what 0005 already promises for the sign-out that a
refused renewal forces. Keeping the two answers identical is what stops the
queue's guarantee from depending on which of the two ended the session.

Before the server is told, and only while there is a session to send under, the
sign-out attempts a drain. It is an ordinary attempt under 0007 and 0038 and it
adds no number of its own: what drains, drains, and what does not stays queued.
A sign-out is not allowed to become slow because somebody has a fortnight of
positions waiting, and it is not allowed to discard them either.

What that costs is a queue nobody comes back for. A person who signs out of a
server and never signs in again leaves entries under 0047's bound that will not
be delivered and will not expire, because 0047 refuses to expire an entry by age
and gives the reason. That cost is real, it is bounded by the count 0047 fixes,
and the act that removes it is the next section. Naming it here is the point: the
alternative is discovering it as a store that holds something forever for a
session that is finished.

## Forgetting a server

Everything a sign-out does, and then removal.

Every cache entry under the key space for that server and that account, in both
tiers of 0054, which 0041 makes a well defined set: every entry whose first three
parts are that server, that account and that device identity, and no other.

The queue for that session in 0047, including its standing count of what it
dropped.

Whatever the core keeps to find those entries, which is the index 0042 holds for
eviction. 0040 gives the store no listing and 0041's keys are digests, so that
index is the only route to the set, and a removal that left it holding rows for
entries it has removed would leave the bound counting bytes that are gone.

Removal goes through the same store interface as everything else. The core
removes what it wrote, through 0040's remove operation, one entry at a time. It
does not reach past the interface, it does not ask the client to delete a
directory, and it makes no claim about what an uninstall leaves behind, which
0068 already refuses to promise and this record does not promise either.

The act is per server and account rather than per server, because two people on
one television are two key spaces and one of them forgetting a server is not the
other one asking for anything.

## When a removal cannot be completed

The index is the single point of failure and the record says so rather than
assuming it is there.

Where the index is absent or was refused by 0105, the set is not reachable, and
the core reports that it removed what it could and names how many entries it
could not reach. It does not report a removal it did not make. This is 0068's
promise meeting its limit in the open, and the honest report is worth more than a
success value, because an operator who asked for their data to be removed and was
told it was gone has no reason ever to ask again.

Where a removal fails part way, because the store answered `storage-unavailable`
from 0004, what has been removed stays removed and the act reports how far it
reached. Repeating it is safe, since removing an entry that is not there succeeds
under 0040's rule and the index says what is left.

The local half of the sign-out inside a forget still completes in both cases. A
device that cannot reach its own store is not a device where somebody should
remain signed in.

## Holding several at once

0005 already fixes that the core holds any number, that every call names its
session, and that there is no ambient current session a second call can change
underneath the first. This record adds what holding several requires of the acts
above, and one thing about the set itself.

The durable list of sessions is the client's, not the core's. 0033's store cannot
be asked what it holds and 0040's cannot either, so nothing in the core can
enumerate what was signed in on a previous run, and 0115 already makes restoring
a session a call the client makes naming the triple it wants. The core knows the
sessions it was told about during this run and no others.

The consequence is that a client which loses its own list leaves both a secret
and a key space that nothing will ever name again. 0033 already states that cost
for the secret. It is the same cost for the cache, it is the reason forgetting a
server is a call rather than something the core does on its own, and a client
that lost the list clears the platform's own stores itself.

## Why this is written down before the code

Sign-out is written by whoever needs it first, and that is #41's own test, which
signs in, signs out and signs in as somebody else. A sign-out written to make
that test pass removes the cache, because the test is about the cache, and the
result is 0006's argument reversed by a fixture rather than by an argument.

Each of the other three parts has a cheap default that is wrong in a way nothing
notices for a long time. A sign-out that waits on the server before dropping the
token in memory leaves a token in memory whenever a network is bad, which is the
one condition under which somebody is most likely to hand the device to somebody
else. A sign-out that discards the queue loses actions a person took, silently,
which is the failure 0047 exists against, arriving through the one act where
throwing things away looks like the point. A sign-out that cancels a lane rather
than a session stops the other person's screen, and on a developer's machine with
one account it is invisible.

Two of the four cannot be corrected afterwards. An action discarded at sign-out
is gone from the only device that held it. A token left live at a server because
nobody reported that it was not ended stays live until the server decides, and no
later change to the core reaches it.

None of this has happened here, because there is no session, no queue and no
store in this tree. The records quoted above are not all of the ones that named
this issue as the one that would answer them, and the set moves, so it is derived
rather than counted here:

    $ git grep -l '#114' -- docs/decisions

## Alternatives, and what each cost

One act, where signing out removes everything. It is the answer this issue's own
list assumes, it needs no second name in the interface, and it is what a person
who has just read a privacy page expects. It costs 0006's whole argument for
keeping entries across a sign-in, so every sign-in on a shared television refetches
a library over the connection the cache exists to avoid, and it makes the cheap
act destructive in a way that cannot be undone by signing back in.

One act, where signing out removes nothing but the token, with no second act at
all. The smallest surface, and it is what the core can do without an index. It
costs 0068's promise to an operator outright, since nothing then removes what the
core asked to be kept, and the first client to want it builds its own removal
against a key space it cannot compute.

Removing the cache lazily, by marking the key space dead and letting 0042's bound
evict it in time. No index walk, no partial removal to report, and the entries do
go eventually. It costs the promise its meaning: data removed when the cache
happens to fill is data still on the device for as long as somebody does not use
the application, which is exactly the period after they stopped using it.

Discarding the queue at sign-out, on the argument that a session that ended has no
business holding work. Simpler, and it removes the queue-nobody-comes-back-for
cost named above. 0005 already refuses it by naming it as its own reversal
condition, and what it costs is a person's actions, silently, in the case where
they signed out on a train and back in the next morning.

Waiting for the server to confirm before completing the local half. It makes the
report simple, since either the token is ended everywhere or the sign-out did not
happen. It costs the case the act is for, because a device with no network then
cannot sign out at all.

Letting the core keep its own list of sessions so that it can offer one and clean
up after a client that lost theirs. It removes the orphan cost above. It costs a
durable list of servers and accounts written by the core somewhere it chose,
which 0101 refuses, and it needs an enumeration over stores that 0033 and 0040
both deliberately do not have.

## What would reverse this

A client is measured, on the harness in #65, to spend longer walking the index at
a forget than an operator will wait on the smallest supported device. Removal by
key space then needs something the store can answer, the store grows an
operation, and that is 0040's reversal and this record's together.

A supported server line offers no way to end a token from the device. The server
half above is then not merely best effort, it is absent, and what replaces this
record says what an operator is told instead, since the current text promises a
report about an attempt that would never be made.

A second act turns out not to be enough, named with the case that cannot be
expressed. The candidate is a person wanting their playback positions removed
from the device while staying signed in, which is neither of the two acts here.
Then the removal is expressed per kind rather than per session and this record is
superseded rather than extended, because the argument for two acts is the
argument against a third.

#105 refuses an index in ordinary use rather than after a version change, so the
report about a removal that could not be completed becomes the common answer
instead of the edge. That is evidence the index is the wrong place to hold the
only route to a key space, and what replaces this record says what holds it
instead.
