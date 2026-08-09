# 0101. What the core trusts, and what it is built to survive

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #101

## The decision

The core trusts two parties and each for one narrow thing, the client that hosts
it for the seams it supplies and for passing on what the operator typed, and the
operator for the single decision of which server may be talked to; every byte
that arrived over a network and every byte read back out of a store is untrusted
input whatever its origin, the device is treated as shared with another person,
losable and holding a second account, and everything else the core meets is
untrusted by default rather than by omission.

## The one question

The list below is finite and the inputs are not, so the rule comes first and the
list is worked out from it.

An input is trusted only if the core produced it inside this process and it has
not left the process since. Everything else is untrusted.

That is the whole test, and it is deliberately harsher than it needs to be for
most inputs, because the cost of applying it to something harmless is a bounds
check nobody needed and the cost of missing something is a parser reading
attacker-chosen bytes.

Two clarifications, because both have already been got wrong elsewhere.

Trusting a party is not trusting their bytes. The operator is trusted to choose a
server. That makes the address the operator typed a legitimate destination. It
makes nothing at all true about what arrives from it. A client is trusted to
implement a byte store. That makes the store a legitimate place to put bytes. It
makes the bytes that come back out of it no more trustworthy than any other bytes
on a disk.

Leaving the process and coming back is a boundary crossing. A cache entry the
core wrote an hour ago is not remembered state. It is a file on a disk that other
programs can write, on a device that may have been someone else's since, and it
is read as hostile input on the way back in. This is the one the rule exists to
catch, because it is the one that looks like memory.

A contributor holding an input the rest of this record never mentions applies the
paragraph above and is done. If the answer comes out as trusted and the input
crossed anything, the answer is wrong.

## What the client is trusted for

The core is a library inside the client's own process. It holds no privilege the
client does not already hold, and it cannot defend anything against the process
it lives in. So what the client is trusted for is not a concession, it is a
description of where the core stops being able to have an opinion.

The client is trusted to supply seams that behave as their interfaces say. A byte
store that says it can be called from two threads is called from two threads. A
secret store that says it is never called concurrently for one session is relied
on to the extent #9 already relies on it. Where a seam misbehaves the core
reports a failure rather than corrupting itself, and that is a robustness
property rather than a security one.

The client is trusted to pass on the server address the operator gave it, without
substituting one. A client that substitutes an address has already replaced the
operator's decision, and it did not need the core's help to do it.

The client is not trusted for the correctness of what it passes at a call site.
Arguments are checked, and a bad one produces a named failure from the vocabulary
in #4. The reason is not defence, since a client that wants to break itself can,
it is that a client author debugging their own mistake at four in the morning
deserves a named answer rather than a corrupted cache.

The trust reaches to the process boundary and no further. Nothing about the
client's own network use, its own storage, or the other libraries it links is
something this core knows or relies on.

## What the operator is trusted for

Exactly one decision, which server this core may talk to. The operator names it,
and under #29 the operator is also the one who may accept a certificate the core
would otherwise refuse. Under #72 the operator is the one who turns a second
server on. That is the whole list.

Naming a server is a statement about intent. It says the operator wants their
library from that host. It does not say the host is healthy, that it is running
the software the operator installed, that no one else answers at that address
today, or that the bytes it serves were made by it at all. Artwork is the
standing counterexample: a server hands on images it fetched from somewhere else,
so even a perfectly healthy server is a pipe for bytes nobody in this
relationship chose.

The consequence for #69 is direct. The set of hosts the core may contact is the
set the operator configured, everything else is refused, and the fact that a
configured host asked the core to go somewhere else does not add to that set.

## What is untrusted

Every byte that arrived over a network, including from the operator's own server.
The population this core is written for is self-hosted servers on home networks,
which is the population least likely to be patched on the day an advisory lands,
and a server compromised last week looks exactly like one that is not.

Everything a payload says about itself. A declared content type, a declared image
dimension, a declared length, a count, an offset. Each of those is a number
chosen by whoever sent it, and every one of them is a number something is about
to allocate or index against. #55 refuses image formats on content rather than on
the claim for this reason, and it bounds declared dimensions before a decode is
attempted rather than after.

Every byte read back out of the byte store in #40. See the one question above.
#105 is where a cache written by another version, or half written, is decided,
and it inherits from here that the answer may never be to believe it.

Every byte read back out of the secret store in #33. A token that comes back
malformed produces a signed-out session, not a parse of an attacker-shaped value.

What follows is the same in every case. A parser reachable from any of these is a
parser #86 fuzzes. A bound is enforced before an allocation and not after it. A
failure is a named kind from #4 rather than a crash, because a crash in a library
is a crash of somebody's client.

## The device

Three assumptions, and the core holds all three at once rather than choosing the
convenient one.

The device is shared. Another person may use it, and on most platforms may read
files this core wrote.

The device may be lost, which means the disk is read at leisure by somebody who
has no hurry and no need to guess a passphrase.

The device holds more than one account, possibly on more than one server. #41 is
the mechanism, and its rule is a confidentiality rule rather than a cache
correctness rule: two accounts on one device must not be able to read each
other's entries, and a key collision between them is a disclosure and not a stale
answer.

What follows for anything written to disk is decided here rather than separately
in every place that writes.

Nothing secret is written by the core to its own storage at all. Not a token, not
a password, not a device identifier that acts as one. Secrets go to the secret
store in #33 and nowhere else, and where a client supplies no secret store the
session lives in memory for as long as the process does. #48 is the test that
this is true rather than intended.

Everything the core does write is treated, when deciding what may go into it, as
though anyone holding the device will read it. That is not a claim that it is
readable on every platform. It is the assumption used when deciding what to put
there, so that the answer does not change per platform.

Nothing written carries a person's name or a server address in a readable form,
which is #41's own condition and #71's for the diagnostic side.

The core does not encrypt what it writes, and this is a decision rather than an
omission. A key the core manages lives on the same device as the ciphertext, so
it defends against a person reading a file and not against a person holding the
device, which is the case the second assumption above is about. Where a platform
offers storage protection tied to the device lock, switching it on is the
client's to do inside the byte store in #40, where the platform's own mechanism
is reachable and the core's would not be.

## What is out of scope

Named, because the reader who assumes cover assumes it here.

A client that deliberately misuses the interface. The core runs in that client's
process with that client's privileges. Anything the core could refuse the client
can do without asking.

A platform whose own protected storage is already compromised. If the keychain
can be read, the token can be read, and no arrangement inside a library above it
changes that.

A compromised operating system, a device with a debugger attached, and a device
whose owner has removed the platform's own restrictions. Each of those is below
the layer this core occupies.

The server's own security. The core cannot tell a healthy server from a
compromised one and does not try. What it promises against a hostile server is
narrow and worth stating exactly: it does not crash, it does not corrupt its
stores, it does not let one account's data reach another, and it does not open a
connection to a host the operator did not configure. It does not promise that the
titles shown are real or that the watch state is honest. A server that lies about
its own contents is telling the truth as far as anything in this repository can
tell.

Whoever runs the network seeing that a connection happened, how large it was and
when. Transport confidentiality is #29's subject and traffic analysis is outside
what a client library can address.

A server that will not answer. That is the absent case in #7 and the recovery in
#45, handled as a condition rather than defended against as an attack.

## The seams a client supplies

Three seams, and they are not the same kind of thing. Each is placed here so that
the issue that builds it inherits the placement rather than deciding it again.

The byte store in #40 sits outside the boundary in both directions. The core
trusts it to hold bytes and hand them back, and trusts nothing about the bytes it
hands back. Nothing secret goes in, per the device section above. A store that
returns nothing is a cache miss and not a failure. A store that returns something
unexpected is untrusted input and is refused the same way a network payload would
be.

The secret store in #33 sits inside the boundary for confidentiality and outside
it for correctness, and it is the only seam the core relies on for a security
property. It is relied on because a token has to rest somewhere and a platform's
protected storage is the only place on a device that can hold one, so this is the
largest single concentration of trust in the design and is named as one rather
than left to be discovered. The correctness half is separate: what comes back is
validated before use, and a value that does not validate is a signed-out session.
The absence of an implementation may never degrade to a file, which is #33's own
condition and is a consequence of this record rather than a preference.

The diagnostics sink in #100 sits outside the boundary, and the direction that
matters is outward rather than inward. Everything handed to it is treated as
though it will be published, because a client may write it to a file, print it to
a console, or paste it into a bug report with a screenshot. So the redaction rule
in #71 is a property of what the core produces, before the sink is called. The
core does not trust the sink to redact and does not ask it to.

## What five issues on this board take from here

Each of these already carried an assumption of its own, which is the condition
that opened #101.

#29 takes that a server the operator named is a legitimate destination and not a
trusted source, so certificate validation is about reaching the intended host and
not about believing what it says.

#41 takes that key separation between two accounts on one device is
confidentiality rather than cache correctness, which decides what a collision
costs.

#48 takes the rule that nothing secret is written by the core to its own storage,
which is what its tests search for.

#55 takes that a declared content type and a declared dimension are attacker
chosen, which is why the refusal is on content and before the decode.

#86 takes that every parser reachable from the untrusted list is in its target
set, which is how the set is derived rather than listed.

## Why this is written down before the code

A trust assumption that is found after the code exists is found as a defect, and
usually as somebody else's.

The specific failure this is against is five slightly different boundaries. Read
on its own, #41 is a rule about cache keys, #48 is a rule about hygiene and #55
is a rule about image parsers. They are the same sentence about a shared device
and an untrusted server, written three times in three vocabularies, and the sixth
case has no rule at all. The artwork cache tier in #54 is the sixth case already
waiting: nothing in #41 or #48 obviously reaches it, and without this record the
person who builds it has no reason to think either applies.

There is a second reason, and it is about direction. A byte store that has been
given a secret once has that secret at rest on every device every client that
linked the core has ever run on. Deciding afterwards that it should not have been
does not remove it from any of them. The rule has to be in place before the first
write, not before the first release.

## Alternatives, and what each cost

No record, with each issue answering its own case as it arrives. Cheapest, and it
is the state this record ends. It costs the sixth case, which has no answer, and
it costs the reader who wants to know whether a rule they are reading is local to
that issue or general.

Trusting the operator's server, on the grounds that the operator chose it. This
is a real position and it buys a great deal: no bounds before decode, no fuzzing
of response parsers, no refusing an image by content, and a much smaller test
surface. It fails on artwork, where the server is passing on bytes it did not
make, and it fails on the population this core is for, since a self-hosted server
is patched when its operator gets round to it. It also fails quietly, because the
day it is wrong is the day nobody is watching.

Trusting nothing, including the client. Consistent, and it would mean validating
every argument as hostile and refusing to rely on the secret store for anything.
It costs the token, which then has nowhere to rest at all, and it buys nothing,
because a library cannot defend against the process it runs inside. The result
would be more work and a weaker position.

Encrypting everything the core writes with a key the core manages. It looks like
the strong answer. The key lives on the device beside the ciphertext, so it
defends against a person reading a file and not against a person holding the
device; it needs somewhere to keep the key, which is the secret store again, so a
client without one would lose the cache as well as the session; and it makes the
cache unreadable to the operator, who is the person #68's position exists for.

A threat model in the usual shape, with attackers, assets and rated risks. It is
the shape a reader from a security background expects and it would be recognised
immediately. It says what to be afraid of rather than what to believe, the
ratings need a likelihood nobody here can measure, and a contributor holding one
unfamiliar field cannot look it up in a risk table. The one question at the top of
this record is what that contributor actually needs, and it is what the table
would not have given them.

## What would reverse this

The core needs to keep something at rest that a platform's protected storage
cannot hold, for instance a value larger than a keychain item bound on a
supported platform. The rule that the core writes nothing secret then has an
exception, and an exception to that rule is a new record with a key management
section in it rather than a sentence added to this one.

#33 lands naming a supported platform for which it can name no protected storage.
The placement of the secret store inside the boundary is then false on that
platform, and the record that replaces this one says what the core does there
instead of implying a property it cannot have.

A measurement under #62 or #63 shows bounds-before-decode, content sniffing and
per-account key separation together costing more than a tenth of the core's share
of either number on the slowest supported target. That is a real price for a
property, and it is the point at which which parts of it are worth keeping
becomes a question with evidence behind it rather than a preference.

The core is run somewhere other than inside the client's process, as a separate
service or a shared daemon. Every sentence here that reads "the client could do
it without asking" stops being true at that moment, the client becomes a remote
caller that has to be authenticated, and this record is superseded by one written
for that shape rather than stretched to cover it.
