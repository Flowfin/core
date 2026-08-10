# 0033. The secret store interface, and a core with no secret store

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #33

## The decision

The session token rests only in a store the client supplies, whose whole surface
is keeping one secret under one name, reading it back, and forgetting it; every
one of those is asynchronous, an absent secret is an absence rather than a
failure, the only failure it may report is `storage-unavailable`, what comes back
is validated before it is used and a value that does not validate signs the
session out rather than being repaired, and a core handed no secret store holds
the token in memory for the life of the process and never falls back to a file it
chose.

## The surface, and why it is this small

Three operations.

Keep a secret under a name, replacing whatever was there.

Read the secret for a name, answering with it or with the fact that there is none.

Forget the secret for a name. Forgetting one that is not there succeeds, for
0040's reason: a caller that has to ask first has a race to lose.

There is no listing, no enumeration and no iteration, and unlike the byte store
this is the operation somebody will reach for first, so it is worth saying what
replaces it. A client that wants to restore a session at start already knows which
sessions it configured. 0005 makes the whole of a session except the token
ordinary data that may be cached, shown and logged, so the server, the account
identifier and the device identity are the client's to hold in ordinary storage,
and the core reads the secret for a session the caller names. The keychain is
asked one question about one name and is never asked what it holds.

What that costs is an orphan. A client that loses its own list has secrets in the
platform store that nothing will ever name again, and nothing in the core can find
them, because finding them is the operation this surface does not have. Signing
out removes the one it names, which is #114, and a client that lost the list has
to clear the platform's own store itself. That is a real cost and it is smaller
than its alternative: enumeration over a platform keychain is the operation whose
behaviour differs most between platforms, several of them return items other
applications wrote, and a core that enumerated would be reading entries it did not
create in order to decide which are its own.

## What a name is

The name handed to the store identifies one session, which 0005 fixes as the
server, the account and the device together, and it is derived the way #41 derives
a cache key rather than being written out.

The reason is 0101's rule that nothing the core writes carries a person's name or
a server address in a readable form, and it applies here more sharply than to the
cache. A keychain item is protected in its value and frequently not in its label:
the label appears in the platform's own listing, in whatever a device backup
holds, and in the view a person opens when they want to know what an application
has stored. A label reading as an address and an account is the sentence 0072 says
a federation list would leak, written by the one part of the system that is
supposed to be the careful one.

The derivation is #41's and is not restated here, with one thing added that the
cache does not need: a secret store name and a cache key are separate spaces and
must not be able to collide, so whatever tag #41 puts at the front of a key
distinguishes them.

## What may fail, and what is not a failure

An absent secret is not a failure. A first run, a person who has never signed in,
and a session whose secret was forgotten all produce the same absence, and the
answer is that there is no session to restore.

The only failure a store may report is `storage-unavailable` from 0004, which
already carries which store it was and whether the failure was a read or a write.
A locked device, a keychain the platform closed while the application was in the
background, a user who declined the permission, an item the platform refused to
write. The core does not tell those apart, because its answer is the same for all
of them, and the answer is stated here rather than left to a call site: a write
that fails does not fail the sign-in that produced the token, and the session
continues in memory as though no store had been supplied. A read that fails is not
an absence, and the difference matters: an absence means sign in again, a failure
means the secret may still be there and must not be overwritten by a new one under
the same name until a read succeeds.

That last rule is the one worth stating explicitly, because the convenient
handling of a failed read is to treat it as empty and carry on, and on a device
locked at the moment of a background start that quietly replaces a working
session's secret with a second one.

## What comes back is untrusted

0101 puts every byte read back out of this store on the untrusted side and fixes
the outcome: a token that comes back malformed produces a signed-out session
rather than a parse of an attacker-shaped value. What validation means concretely
is this record's.

A secret is accepted only if it is non-empty, within a stated length bound, and
composed entirely of bytes that can be placed in the header it is going into
without escaping. The last of the three is the one that is not decoration: a value
carrying a line ending is a request the core would assemble on somebody else's
behalf, and the store is on a device 0101 assumes is shared and may be lost.

A value that fails any of the three is not repaired, not trimmed and not escaped.
The session is signed out, the value is forgotten, and the client is told through
the interface in #100. Repairing it would mean the core deciding what somebody
else's token was meant to be.

## Threads

0009 already fixes this and an implementer needs two consequences of it.

The store is called from the waiting lane only, and never concurrently for one
session, so an implementation may be written without locking. That is the
deliberate opposite of the byte store in #40, and 0009 gives the reason: a
keychain call is rare, and a platform keychain is the place a client is most
likely to write something naive.

An implementation may block, and the core holds no lock of its own across the
call. A keychain that puts up the platform's own authentication is therefore a
slow store rather than a stopped core.

## A core with no secret store

It works. Sign-in succeeds, every call that needs a token has one, and nothing
starts failing. What changes is that the session ends with the process, and the
person signs in again next time.

It says so, through a capability a client can ask about, which is a call that
cannot wait in the terms of 0009. A client that cannot ask cannot tell an operator
why they sign in every morning, and it cannot decide to prompt for sign-in at a
moment of its own choosing rather than at the moment somebody presses play.

There is no fallback and there will not be one. 0005 and 0101 both refuse a file
the core chose the location of, with or without obfuscation, and the reason is
that a key the core manages lives on the same device as what it protects. This
record adds nothing to that argument and does not weaken it.

The absence of a store is not the in-memory double the suite runs against. 0040
makes that distinction for the byte store and the same argument holds here without
being made twice: the double is a supplied store under test as a store, and it can
be made to fail, to be locked, or to answer slowly, which is how the failure rules
above are reached at all.

## What each platform family offers

The property is the test, and it is one sentence: the facility whose contents are
protected by the platform rather than by file permissions the application chose. A
client author who has one of those uses it, and a client author who has to decide
between two applies that sentence.

The facilities themselves are named here as a claim rather than as a measurement,
because nothing in this tree reads a platform and no command in this repository
produces the list. On Apple platforms the keychain. On Android the platform
keystore, reached directly or through the storage facility that is backed by it.
On Windows the credential manager. On desktop Linux the freedesktop secret
service, where a session provides one. On a television the answer differs per
vendor and the honest position is that a client author checks rather than assumes,
which is the case this whole interface exists for: the core does not have to know,
and does not.

A client with none of these supplies no store, which is the case above, rather
than inventing one.

## What this does not decide

Holding more than one session, and what signing out removes. #114, expressed in
the naming this record takes from #41.

Establishing the device identity that is part of a name. #36.

When a token is renewed and what a rejection does. #34, on the model in 0005.

Whether the secret is the only thing worth storing. It is, because 0005 fixes the
token as the only secret, and a record that widened that would be superseding 0005
rather than extending this one.

Proving that nothing secret reaches the cache. #48, which reads the separation
this record and 0006 keep.

## Why this is written down before the code

The interface is already promised by four landed records and defined by none of
them. 0005 makes storage a client responsibility behind "the named interface in
#33". 0006 says the secret store is a different interface with a different
implementation and gives the reason the two are not one store with a flag. 0009
places its calls on the waiting lane. 0101 calls it the largest single
concentration of trust in the design. Those are the ones quoted here rather than
all of them, and the set moves, so it is derived:

    $ git grep -l '#33' -- docs/decisions

So the first thing that needs a token across a restart finds the interface named
everywhere and specified nowhere, and invents one. The shape it invents is a
synchronous read and write over a file path, because that is what a developer's
own machine makes easiest and because a token is small enough that the file looks
harmless. That single default breaks 0005's rule about where a secret rests, 0009's
rule about the calling thread, and 0101's placement of this seam, and it does so at
the seam where a mistake is a credential at rest on every device every client that
linked the core has ever run on. Deciding afterwards that it should not have been
written does not remove it from any of them.

This has not happened here, because there is no code in this tree. That is what
makes the record one file today.

## Alternatives, and what each cost

The core reaching the platform's protected storage itself. Removes an interface
and a thing every client supplies. 0005 already refuses it and gives the reasons,
and nothing new was found against it; what is new is that this record makes the
refusal concrete, since the interface it names is the whole of what the core would
otherwise have had to build per platform.

One store for secrets and cached bytes, with a flag or a naming convention marking
which is which. One interface instead of two and one place to clear on sign-out.
0006 already refuses it and the reason is the proof in #48: with two interfaces the
proof is that the cache store never receives the token, which a test can watch,
and with one it is a property of a flag at every call site.

A store that can enumerate what it holds. It would let a client offer a list of
signed-in accounts without keeping one itself, and it would let the core clean up
after a client that lost its list. It costs the platform differences named above,
and it costs a rule about what the core does with an item it did not write, which
is a rule with no safe answer.

Storing the whole session rather than the token. Fewer moving parts, and restoring
a session becomes one read. It costs the size, since a keychain item is the wrong
place for capability answers and an address, and it costs 0005's distinction
between the one secret and the ordinary data around it, which is the distinction
that keeps the rule about what may be logged simple.

A synchronous surface, on the argument that a keychain read is fast. It reads well
at a call site and most reads are fast. The reads that are not fast are the ones
where the platform puts up its own authentication, which is exactly the
configuration an operator who cares about this has chosen, and a synchronous
surface puts that wait on whichever thread called.

Refusing to run with no secret store, so a client cannot get the degraded
behaviour by accident. It removes the case that is hardest to test and it is
honest about what the core cannot promise without one. It costs the first thing
anybody does with a new library, which is to construct it and sign in before
writing a keychain implementation, and it costs a client that deliberately wants
nothing kept.

Handing the store a value the core has already wrapped, so that what rests is not
the token itself. It sounds like defence in depth. The wrapping key has to live
somewhere the same process can read, which is the secret store, so it is 0101's
own argument arriving one layer further in with nothing added.

## What would reverse this

Two client implementations are found whose protected storage does not in fact
survive an application update, or is not in fact protected by the platform. 0005
already names this as its own reversal, and the honest replacement is a record
saying the core holds sessions in memory only.

A supported platform is found with no facility meeting the property above. 0101's
reversal names this case, and what replaces this record says what the core does
there rather than implying a protection it cannot have.

A token is met that does not fit a platform's item size bound. The validation rule
above then refuses a working session on that platform, which is the wrong
direction, and the replacement says where a value too large for the store goes
instead.

Three operations turn out not to be enough, named with which call could not be
expressed. The likeliest candidate is a client needing to know whether a secret
exists without reading it, on a platform where reading prompts a person. Then the
surface grows and this record is superseded rather than extended, because the
argument for three is the argument against a fourth.
