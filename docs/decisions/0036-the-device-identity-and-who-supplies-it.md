# 0036. The device identity, and who supplies each part of it

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #36

## The decision

The client supplies the device identifier and the device name and keeps both in
its own ordinary storage; the core generates an identifier only when a client
asks it to and hands it straight back rather than keeping a copy; the identifier
is an opaque string that is stable for the life of an installation and is derived
from nothing about the hardware or the person; and the core fixes the shape of
the capability description while the client supplies everything in it.

## The three parts, and who holds each

The identifier. A string that names this installation of this client on this
device to a server. The client holds it. Where the client has no notion of one,
it asks the core for a value once, at any moment, and stores what it gets back.
The core keeps nothing.

The name. What a person sees in the server's own session list. The client holds
it and the core never invents one, because only the client knows whether it is a
television in a living room or a handset, and a core that guessed would put the
same word in front of every operator running any of the eleven clients.

The capability description. What this client can actually play. The core owns its
shape, so that eleven clients do not describe the same thing eleven ways, and the
client supplies the contents, because what a platform can decode is a fact about
the platform and 0003 keeps platform knowledge out of the core.

## Why the client holds the identifier rather than the core

0033 already decided this and it is worth saying why rather than only that it
follows. The secret store has no listing, so a client restoring a session at
start has to name the session it wants, and 0005 fixes a session as one server,
one account and one device. The identifier is therefore an input the client must
already hold before the core has read anything, and a core that stored it would
have to read a store in order to build the name of the entry in that store.

0041 puts the same identifier inside the cache key, and 0046 turns that into a
property this record has to keep: a cold start serves from cache before a session
is restored precisely because every part of a key is in the client's hands
already. An identifier the core generated at start and had not yet persisted
would produce a different key space on every run, so the first start after every
crash would serve nothing and the store would fill with entries nothing will ever
name again. That failure is invisible on a developer's machine, where the process
is stopped cleanly, and it is ordinary on a television.

So the core generating one and remembering it is not a smaller version of this
decision. It is a second durable store the core would own, and 0040 and 0033 are
the only two the core has.

## What the identifier may not be derived from

Not a hardware serial, a network address, an advertising identifier, or anything
else the platform hands out that also names the device to somebody else. Not a
hash of any of those, because a hash of a stable identifier is a stable
identifier with a longer name and it links the same device across every server it
contacts.

Not anything about the person. The account name is on 0068's personal data list,
and an identifier built from it would put that value into every request header,
into 0041's key construction, and into 0033's item label, which is the one place
0101 requires nothing readable to appear.

What it is instead is a value with no meaning: enough unpredictable bytes from
the runtime that two installations do not collide, in the shape 0032 already
established for the value tying a delegated sign-in to its answer, including that
record's requirement that a client supplies the bytes where the means chosen in
#11 offers no source of its own.

Unpredictability here buys collision resistance and nothing else. 0041 already
says plainly that anyone holding the device can compute a cache key, because the
device identity is one of its low-entropy inputs, and nothing in this record
should be read as making that identifier a secret. It is not stored through 0033,
it is not a credential, and a client is free to show it to a person who asks what
their device is called.

## What the server does with it, and what a changed identifier costs

The Jellyfin server reads four named parts out of the authorization value a
client sends, and the identifier and the name are two of them. Read at
`ae8723026d97b6d0f926638803edef338919b794` in the public server repository:

    $ git clone https://github.com/jellyfin/jellyfin
    $ J=ae8723026d97b6d0f926638803edef338919b794
    $ git -C jellyfin grep -n "auth.TryGetValue" "$J" \
        -- Jellyfin.Server.Implementations/Security/AuthorizationContext.cs | sed "s/^$J://"
    Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:86:                auth.TryGetValue("DeviceId", out deviceId);
    Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:87:                auth.TryGetValue("Device", out deviceName);
    Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:88:                auth.TryGetValue("Client", out client);
    Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:89:                auth.TryGetValue("Version", out version);
    Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:90:                auth.TryGetValue("Token", out token);

So the client name and its version travel beside the device identity rather than
separately, and this record puts both on the same footing as the device name: the
client supplies them and the core sends what it was given.

The server keys a live session on the client name joined to the device
identifier:

    $ git -C jellyfin show "$J":Emby.Server.Implementations/Session/SessionManager.cs \
        | sed -n '478,479p'
            private static string GetSessionKey(string appName, string deviceId)
                => appName + deviceId;

That is what the stability requirement is actually for, and it is a stronger
reason than the cache. An identifier that changes between starts produces a fresh
session for every start, so the operator's own session list fills with entries
for one device that nobody can tell apart, every one of them holding a token that
nothing will ever sign out. 0030 already names one route to that residue, a
sign-in whose answer never arrived, and calls it a thing an operator can only act
on if somebody said it happens. This is the same residue produced continuously
rather than occasionally, and the identifier is what prevents it.

It also means the key is the client name and the device together. Two clients on
one device are two sessions on the server whether or not they share an
identifier, so a client is not required to keep its identifier distinct from
another application's on the same device. The core does not attempt to make it
so, because a value the core made distinct per client would have to be derived
from something naming the client, and the core does not hold that either.

## The capability description

The server stores the description against the device identifier rather than
against a session:

    $ git -C jellyfin grep -n "GetCapabilities(info.DeviceId)\|SaveCapabilities" "$J" \
        -- Emby.Server.Implementations/Session/SessionManager.cs | sed "s/^$J://"
    Emby.Server.Implementations/Session/SessionManager.cs:188:                var capabilities = _deviceManager.GetCapabilities(info.DeviceId);
    Emby.Server.Implementations/Session/SessionManager.cs:1833:                _deviceManager.SaveCapabilities(session.DeviceId, capabilities);

Two consequences follow and both are decided here rather than left to whoever
writes the first call. A description sent once outlives the session that sent it,
so it is stated by this record to be sent on every sign-in rather than once per
installation, since a client that changed what it can decode after an update
would otherwise be described to the server by its previous version indefinitely.
And a description is per device rather than per account, so two people signed in
on one television describe one device once and the second sign-in does not
contradict the first.

The core owns the shape and refuses to fill it in. A client states what it can
decode, at what sizes, over what containers, and the core carries that to the
server unchanged and unaugmented. The core adding a default would be the core
claiming a platform fact it cannot check, which is 0003's line, and a default
that is wrong produces a stream the device cannot play with an error that arrives
from the decoder rather than from the core.

The shape being the core's is the half that is worth the argument. Eleven clients
each writing their own would be eleven answers to what a profile contains, and
the drift would show up as one platform silently getting transcoded streams it
did not need while another gets a container it cannot open. The conformance suite
in #76 is where a client's description is checked against the shape, and there is
nothing to check it with until #75 exists.

## What this record does not decide

Which endpoint carries the description, and when a playback request supplies its
own. The server accepts one on a playback request as well as reading the stored
one, and choosing between them belongs to #111 with the endpoint list in #10.

The server versions this behaviour holds across. Everything above was read at one
commit of the public server repository, named where it is quoted, and #10 is
where the surface is enumerated against a version range once entry 3 of #1 says
what that range is.

What the identifier is stored under on the client's side. That is the client's
ordinary storage, which 0033 already places outside the core, and this record
adds no requirement to it beyond keeping the value.

Whether an operator can rename a device from the server. The name the core sends
is the client's, and what the server does with a name it was given afterwards is
the server's business.

## Why this is written down before the code

Three landed records already depend on an identity that does not exist. 0005
lists the device identity as part of what a session is. 0033 names it as part of
the name a secret is stored under and points at this issue for it. 0041 writes it
into the cache key and points here too. 0030 takes it as an argument to sign-in.
So the first code that needs a device identifier reaches four other decisions at
once, and whatever it does becomes all four of their answers.

The specific thing that gets decided by accident is who generates it. The
shortest correct-looking code generates a value in the core at first use and
keeps it in memory, because that compiles and works and every test passes. It
produces a new key space on every process start, which nothing observes, because
the failure is a cache that is empty when it should be warm, and an empty cache
is indistinguishable from a cold one at the point where somebody would notice.
The second shortest reaches for a platform identifier, because the platform
offers one and it is obviously stable, and that is the version that ships an
advertising identifier or a hardware serial to every server a person contacts.

Written afterwards, neither is discoverable from the code. There is nothing in a
diff that says which values were deliberately not used.

## Alternatives, and what each cost

The core generates the identifier and persists it through 0040's byte store. It
removes a thing a client has to remember, and it is the answer that reads as the
core doing its job. It costs the ordering 0046 depends on: the identifier is part
of 0041's key, so the core would have to read the store in order to build the key
of the entry it needs from the store, and the special-cased first read is exactly
the kind of exception that later gets removed by somebody tidying the key
construction. It also makes the identifier disappear when a client clears its
cache, which is the one operation a person is told to try when anything is wrong.

The core generates it and persists it through 0033's secret store. It survives a
cache clear, and the keychain is the store that is meant to survive. It costs the
one property 0033 is built around, that the store holds the token alone, and it
puts a value that is not a secret behind a prompt on a locked device, so a cold
start would wait on a keychain to learn its own name. 0046 refuses exactly that
ordering.

A platform-supplied identifier, used directly. Stable by construction, free, and
already unique. It costs the property this record spends the most words on: those
identifiers are issued to name the device to other parties as well, several of
them are resettable by a person in a settings screen, which turns a reset into
the changed-identifier failure above, and on the platforms where they are not
resettable they are a permanent link between one device and every server it ever
contacted. 0068's position is what that runs against.

A value derived from the account or the server. It needs nothing stored at all,
since both are already to hand. It costs the separation 0041 exists for, because
a device identity that is a function of the account is not a device identity, and
two accounts on one device would then produce keys that differ for a reason that
has nothing to do with the device.

The core owning the capability description's contents as well as its shape, with
a client overriding what it disagrees with. One correct-looking default and less
for a client author to write. It costs 0003's line and it costs it in the
direction that is hardest to debug: a default that overstates what a device can
decode produces a stream that fails inside a decoder, which is the one place the
core has no visibility and the client has no error to map.

## What would reverse this

A client is found that genuinely cannot keep a value across restarts. The core
then owes a durable place for it, and this record is superseded by one that says
which of the two stores holds it and how 0046's ordering survives, rather than by
a special case added to 0041.

The server stops keying a live session on the client name and the device
identifier together. The stability argument above rests on that join, and a
server that keyed on something else would change what a changed identifier costs.
The commit the join was read at is named above, so the check is a re-run of that
command against a later one.

Two installations are observed colliding on an identifier the core generated.
That is evidence the byte source or its width is wrong, and the record is
superseded by one carrying the measurement rather than the width being adjusted
in place.

#10 answers that a supported server version does not read the four parts above
out of the authorization value. The identity then has more than one shape on the
wire, which is a different decision from this one and lands as a record that
names this one.
