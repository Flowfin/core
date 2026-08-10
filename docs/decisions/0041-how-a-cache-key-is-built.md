# 0041. How a cache key is built, and why a collision is a disclosure

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #41

## The decision

A key is a cryptographic digest of a version tag and the four parts 0006 names,
each part length-prefixed and written in a fixed order so that no value of one can
be made to look like part of another, so that the key a store receives carries no
readable name and no readable address, and so that two accounts on one device, or
two servers behind one address, cannot reach each other's entries by any input
either of them controls.

## What a collision costs, which is what fixes the construction

0101 already places this: two accounts on one device must not be able to read each
other's entries, and a collision between them is a disclosure and not a stale
answer. Everything below follows from that one sentence rather than from a
preference about hashing.

It decides the failure direction. A key scheme that occasionally treats two
different requests as one is a caching defect if the two belong to the same
person, and it is somebody seeing another person's library if they do not.

It decides who the adversary is. Part of the input is chosen by the server, since
an item identifier in a request comes from an answer the server sent, and 0101
treats every byte from a server as untrusted whether the server is healthy or not.
So the construction has to hold against an input somebody chose to make it fail,
which rules out a scheme that is merely unlikely to collide on ordinary data.

## The construction

Five things are written, in this order, and the digest of the result is the key.

A version tag for this construction.

The server, as the resolved identity 0006 requires rather than the address a
person typed.

The account, as the identifier the server gave back at sign-in, never the
username.

The device identity from #36.

The request, as the endpoint and the parameters that change the answer.

Each of the five is written as its length in a fixed-width field followed by its
bytes. The length prefix is the whole of the ambiguity argument and it is worth
being explicit about what it prevents, because the defect it prevents is invisible
by inspection: with plain concatenation, an account called `ab` with a request
starting `c` produces the same string as an account called `a` with a request
starting `bc`, and one of those two is a person who is not supposed to be reading
it. A separator character does not fix this, it moves it to whichever part is
allowed to contain the separator.

The digest is cryptographic and its output is used at full width. What is required
of it is collision and second-preimage resistance against an input the adversary
partly controls, which is the paragraph above rather than a preference for strong
primitives. A non-cryptographic hash is refused by name, because it is the default
a developer reaches for when the word in front of them is "key" and it is chosen
for speed against inputs nobody is choosing.

Which function, and its exact width, is a means decision that depends on the
toolchain in #11, so it is named where the code is rather than here, and the
choice is recorded there against this record's requirement.

## What the digest does and does not buy

It removes readable values from anywhere a store puts a key. A store may use the
key as a filename, a column, or a label, and 0101 requires that nothing the core
writes carries a person's name or a server address in a readable form. A directory
listing, a backup somebody shares, and a screenshot of a file browser all stop
saying which servers a person uses, which 0072 names as the first thing a
federation list would leak.

It does not hide anything from somebody who can guess the input. The parts are a
server, an account identifier, a device identity and a request, and anyone holding
the device who already suspects a particular server and account can compute the
digest and look for it. That is a confirmation of a guess rather than a
disclosure, and it is the honest limit of what a digest over low-entropy input can
do. Nothing in this record should be read as making a cache directory opaque to
its holder.

The distinction matters because the two get conflated, and the conflated version
is the one that gets quoted later as a security property this repository does not
have. What the keying provides is separation between accounts and servers on one
device. What it does not provide is confidentiality of the cache against the
person holding the device, and 0101 already says the core does not encrypt what it
writes and why.

## Two parts that need a rule of their own

The server part is the resolved identity, which is the identifier the server
reports about itself and which the session already holds under 0005 among the
capability answers from #10. Where a server offers no such identifier, the part is
the base address from 0028 instead, and the cost is stated rather than discovered:
two addresses that reach one server then become two key spaces, so entries are
fetched twice and never mixed. That is the safe direction of the two, since the
unsafe direction is one key space for two servers.

The request part has one rule the endpoint and parameters do not give on their
own. A parameter that is absent and a parameter that is present and empty are
written differently, because they are different requests and the server may answer
them differently. Which parameters change an answer at all is a per-endpoint fact
that 0006 already leaves with the code, and this record adds only that a parameter
the core decided not to include is excluded by a written decision at that endpoint
rather than by being forgotten.

## The version tag, and what it is for

Changing any of the above changes what a key means. Without a tag, a core built
after such a change reads entries an earlier core wrote as though they were its
own, under keys that happen to match, and the failure is a wrong answer rather
than a miss.

With the tag at the front, an old key space is simply unreachable. Nothing
misreads it, nothing has to migrate it, and the bytes are ordinary garbage that
the bound in #42 evicts. That is the cheap answer and it is available only because
the tag is inside the digest rather than beside it.

This is not the same question as an entry written by another version of the core
carrying a different payload shape under the same construction, which is #105's,
and the two answers are deliberately separate: the tag protects the meaning of the
key, and #105 protects the meaning of what is behind it.

## What this makes possible and what it does not

Signing out is #114, and 0068 already promises that a caller who wants a session's
entries gone gets them removed under the key space for that server and account.
This record makes that set well defined: every entry whose first three parts are
those values, and no other.

It does not make that set reachable. 0040 gives the store no listing, on purpose,
and a digest is not reversible, so nothing can be recovered from the keys
themselves. Removing a key space therefore requires the core's own record of which
keys it wrote, which is the bookkeeping 0040 hands to #42 for eviction. The same
index serves both, and naming that here is the point: without it, #114 cannot
complete a removal it has already been promised, and the shortest route to
discovering that is somebody implementing sign-out and finding nothing to
enumerate.

The artwork tier in #54 keys the same way. It is a separate tier with its own
bound rather than a separate scheme, and 0101 names it as the case that would
otherwise have no rule at all.

The secret store in #33 names its entries through this construction as well,
because the same argument about readable labels applies more sharply to a
keychain. Its names and cache keys occupy separate spaces and cannot collide,
which the version tag at the front carries.

## Why this is written down before the code

0006 already decided the four parts and said this record owns the exact
construction, including how the parts are joined so that no value of one can be
made to look like the start of another. That sentence is the whole of what is left
to get wrong, and it is the kind of thing that is got wrong in a line of code that
reads correctly.

The specific defect is the one named above: parts joined without lengths, or with
a separator one part may contain. It produces no test failure, because the inputs
that collide are not the inputs anybody writes a fixture for, and it produces no
symptom on a device with one account, which is every developer's device. It
surfaces on a shared television, as one person seeing another person's library,
which 0006 already says is the failure that cannot be corrected afterwards at any
reasonable price: the entries are already written under the wrong keys, so the
repair invalidates everything and has to reach every client at once.

The second thing that cannot be corrected afterwards is a readable key. A store
that has been handed an address and a username as a filename has them in every
backup taken since, and a later change to the construction does not remove them
from any of those.

Neither has happened in this tree, because nothing here writes a cache entry. That
is what makes this one file now.

## Alternatives, and what each cost

Keying on the request alone, or on the server and the request. Simpler, shorter
keys, and correct for the single-account installation that most of them are. 0006
already refuses it, and the cost is the case this project cares about most, a
shared television, with the failure being a disclosure rather than an error.

Joining the parts with a separator character and no lengths. It reads far better
at the call site and it is what most caching code does. It is correct exactly
while no part can contain the separator, which is a property of today's inputs
rather than of the scheme, and the input that breaks it is an item identifier that
came from a server the core does not trust.

A structured key handed to the store as a tuple rather than a string, leaving the
joining to the store. It removes the ambiguity question from the core entirely. It
costs 0040's whole argument for four operations over an opaque key, and it moves a
confidentiality property into eleven implementations, where the joining is done
eleven ways and one of them uses a separator.

A non-cryptographic hash, chosen for speed. Cache keys are computed on every read,
and this is the standard advice for a cache. It costs the adversarial half: the
input is partly server-chosen, and a fast hash with published collisions turns a
disclosure into something an unhealthy server can aim.

Keeping the parts readable in the key, on the argument that a cache directory is
already readable and that debugging is easier. It is genuinely easier to debug, and
an operator can see what their own device holds, which is a thing 0068's position
values. It costs 0101's rule outright, and it costs it in the direction where the
data has already been copied into backups by the time anybody reconsiders.

Encrypting the key rather than digesting it, so the core can recover the parts and
enumerate a key space. It would give #114 its removal without an index. The key
has to live on the device beside the data, which is 0101's argument against
encrypting the cache at all, and it buys enumeration that #42 needs an index for
anyway.

## What would reverse this

The index #42 keeps for eviction turns out to be unaffordable in memory on the
smallest supported device, measured on the harness in #65 rather than assumed.
Then removal by key space needs something the store can answer, and the store
grows an operation, which is 0040's own reversal and this record's at the same
time.

A collision is observed between two accounts on one device. That is not a tuning
problem, it is this record being wrong, and what replaces it says which part of
the construction failed and why.

#10 answers that a server offers no stable identity of its own on a supported
server line, so the fallback above becomes the ordinary case rather than the
exception. Two addresses for one server then duplicate every entry in the common
case, which is a cost worth re-deciding rather than inheriting.

The chosen toolchain in #11 offers no cryptographic digest without adding a
dependency that #103's rule refuses. The requirement above then has to be met some
other way or stated as unmet, and either is a new record rather than a sentence
added to this one.
