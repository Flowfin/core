# 0029. Certificate validation, and the server an operator signed themselves

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #29

## The decision

Certificate validation is always on and nothing the core reads on its own can
weaken it; the single exception is one exact certificate pinned by the operator
through the client, for one server, after the client has shown them what the core
refused, stored where the client already stores bytes, removable, and readable
back so it can be shown; the core itself never decides to trust anything, never
remembers a refusal as an acceptance, and sends no request and no credential over
a connection whose certificate it did not accept.

## What is validated, and what a refusal is

The core validates that the machine which answered is the one the address named,
using the platform's own trust store and the platform's own path building. What
that means in detail is the platform's, and the core does not reimplement it.

0243 names the means this requirement is met through, per platform, and records
that the logging facade that means carries is admitted under 0103's fourth
behaviour as narrowed there. Nothing in this record moves.

A refusal is `certificate-rejected` from 0004, which already fixes its payload as
the address, a reason class, and the presented certificate's fingerprint, and
already fixes its retry property as no until a person decides. This record fixes
the reason classes, because a class nobody wrote down becomes a string a client
switches on.

    self-signed          The certificate signed itself, and nothing else vouches
                         for it.
    issuer-unknown       The chain ends at something the platform's trust store
                         does not hold.
    name-mismatch        The chain is trusted and names a machine other than the
                         one the address named.
    expired              The chain is trusted and its validity window has ended.
    not-yet-valid        The chain is trusted and its validity window has not
                         begun, which is a device clock as often as a server.
    chain-unusable       Anything else the platform refused: a signature that
                         does not verify, an algorithm it will not accept, a
                         constraint the chain violates.

Six classes, and `chain-unusable` is the one that keeps the set closed, in the
same shape and for the same reason as `answer-not-understood` in 0004. A class
carries no sentence, which is 0003 applied here: a client writes six sentences and
decides for itself whether to name the server in them.

The two clock-shaped classes are named separately from the rest on purpose.
`not-yet-valid` on a television that came up believing it is 1970 is the case 0102
describes, and a client that can tell it apart from `expired` can say something
useful rather than telling somebody their server is broken.

What the core hands over so a client can show the refusal: the presented chain as
it arrived, the fingerprint of the certificate at its end, the reason class, and
the subject, the issuer and the validity window as data. Not as a rendered
sentence, for 0004's reason. The fingerprint is the field that matters, because it
is the one an operator can compare against what their own server prints.

## The exception, and what an operator is actually agreeing to

There is one, it is a pin, and it is nothing else.

A pin names one certificate, identified by a fingerprint over the whole of its
encoding rather than over its public key alone. The whole encoding, because that
is what a server prints when its operator asks it what certificate it is serving,
and a pin an operator cannot check by comparing two strings is a pin they will
accept without checking.

A pin belongs to one server and reaches no other. The server is the resolved
identity in 0006, and where the core does not yet have one, because the pin is
being taken during the first connection to that server, the pin is held against
the base address from 0028 and no other address inherits it.

A pin is offered only by the client, only after a refusal, and only with the
material above in hand. The core has no route to a person, so it cannot ask, and
0003 refuses it the words it would ask with. What the core exposes is the refusal
and a call that says this exact fingerprint, for this server, is acceptable from
now on.

What a pin replaces is the chain, the name and the validity window, and nothing
else. A pinned certificate is accepted whether it signed itself, names a different
machine, or expired last month, because the operator asserted that this exact
certificate is their server's and each of those three properties is one they
overrode by asserting it. Everything about the connection that is not the
certificate still holds.

What a pin never becomes is an issuer. A certificate signed by the pinned
certificate is not accepted, and a chain that merely contains it is not accepted.
Only the exact certificate is, and only by fingerprint. Installing the operator's
certificate as a trust anchor for that connection is the shape this refuses by
name, because it turns one server's key into something that can vouch for any
name, including the ones the person did not type.

A pin is visible and removable. A client can read back which servers carry a pin
and what fingerprint each holds, and can remove one, which is what makes it a
decision an operator can revisit rather than one they took once in the dark.

## Where a pin lives, and what it is keyed by

A certificate is public, so a pin is not a secret and does not go to the secret
store in #33. It is bytes, so it goes to the store in #40 under a key built the
way #41 builds one.

It is keyed by the server and the device, and not by the account. A certificate is
a property of a machine rather than of who signed in to it, and asking the second
person on a shared television to compare a fingerprint they have no way to check
produces an agreement with no meaning behind it. The cost is real and is stated
rather than hidden: a second account on that device inherits a decision the first
account's operator took. That is the same person in the case this repository is
written for, a household with a server in it, and where it is not, the pin is
visible to any client that asks.

With no store supplied, the pin lives as long as the process, which is 0040's
answer to everything else and is not softened here. An operator on a client with
no byte store pins once per run.

## What the core never does

No configuration the core reads on its own turns validation off. Not a field on a
call, not an environment variable, not a build flag, not a debug mode. The core
has no such switch to find, so there is nothing for a client to set, for a
tutorial to copy, or for a person to be talked into.

No trust on first use. Accepting whatever answers the first time and pinning it
silently would move the decision from the operator to whoever is on the network at
that moment, which is precisely the moment a home network is least trustworthy.

No accept-once. A client may retry after a refusal, and the retry is refused
again, because an exception that lasts one call is an exception nobody can see
afterwards and it is indistinguishable from a person clicking through.

No request over an unaccepted connection. The handshake gets far enough to see
what was presented and stops there. Nothing is sent: not a credential, not a
device identity, not a query. This is the part that is worth a test on its own,
because a validation failure discovered after the request went out is a
credential already delivered.

No opinion about a scheme. 0028 fixes that a typed `http://` is honoured and that
an absent scheme becomes `https`. A connection with no certificate at all has
nothing for this record to validate, and telling an operator anything about that
choice is a sentence, which is the client's under 0003.

## What this leaves for other issues

The list of hosts the core may reach at all, and what happens to a redirect out of
the configured origin, is #69. A pin says nothing about where the core may go; it
says which certificate is acceptable once it is there.

The transport that carries this, its timeouts and its connection reuse, is #27. A
pinned connection is reused like any other, which is why the pin is evaluated per
connection rather than per request.

Whether a refusal reaches the client as a diagnostic event as well as a failure is
#100's shape and #71's redaction rule. A fingerprint is not personal data; the
address next to it is, and 0068 already places it.

Proving any of this is #21's, because every test named in this record needs a
server that can be made to present a certificate the core will refuse.

## Why this is written down before the code

The exception is the part that gets built in a hurry, and it has exactly one
convenient shape: a boolean that skips validation for a connection. It is three
lines, it makes the self-hosted case work immediately, and it is wrong in a way
that is invisible in every test anybody writes, because a test with a hostile
certificate is a test somebody has to think of. Once that boolean exists it is
also a field on a public interface, and eleven clients set it, and at least one
sets it from a configuration file an operator can edit.

The second shape is the trust anchor. It feels more careful than a boolean, it
appears in a great deal of published advice, and it is worse than a pin in the one
direction that matters: a certificate installed as an anchor can vouch for any
name at all, so an operator who wanted to reach their own server has quietly
authorised that key to answer for everything else too.

Neither has happened here, because there is no transport in this tree. That is
what makes the record cost one file rather than a change to a public interface
eleven clients already call.

## Alternatives, and what each cost

Refusing every certificate the platform refuses, with no exception at all. The
smallest surface, nothing to store, nothing to show, and no way for a person to
make a mistake. It costs a large share of the population this repository exists
for, since a self-hosted server on a home network usually has a certificate its
operator issued, and the cost is not the operator's alone: a core that cannot
reach those servers gets a client that ships its own transport around it, which
moves the whole decision somewhere nobody reviews.

Trust on first use, remembering whatever answered the first time. It asks nobody
anything, it protects every connection after the first, and it is what a great
deal of self-hosted software does. It costs the first connection, which is the one
made on whatever network the person happened to be on, and it costs the property
that the operator decided: the core would have decided, silently, and the record
of what it decided reads afterwards exactly like a pin somebody checked.

Pinning the public key rather than the whole certificate. It survives the operator
reissuing a certificate for the same key, which removes a re-pin every year. It
costs the comparison: what a server prints, and what a person can read off their
own machine, is usually a certificate fingerprint, and a pin an operator cannot
verify by comparing two strings is a pin taken on faith.

Installing the operator's certificate as a trust anchor scoped to that connection.
Familiar, and it reuses the platform's own path building rather than adding a
comparison. It costs the name check, because an anchor vouches for whatever name
the chain under it claims, so the operator's own server key becomes able to answer
for any address the core is ever pointed at.

Letting the client supply its own trust evaluation. Honest about where platform
knowledge lives, and it would let a client with an unusual deployment do something
this record does not cover. It costs the one seam the core cannot afford to hand
over: 0101 places the operator's single decision here, and a client-supplied
evaluator is a client deciding it instead, with no way for the core to tell a
careful implementation from one that returns yes.

Allowing a pin to expire on the certificate's own validity window. Tidier, and it
forces a periodic look at something an operator agreed to. It costs a person
access to their own library on a date they did not choose and cannot predict, for
a certificate that is exactly as trustworthy on that morning as it was the night
before, since what made it acceptable was the operator's assertion rather than its
dates.

## What would reverse this

A pinned certificate's key is found to have been taken, on a real installation,
with no route to withdraw the pin except the operator removing it by hand. The
residual this record accepts is that a pin has no revocation, and one real
instance turns that from an accepted residual into a missing mechanism, which is a
new record naming what withdraws a pin.

Two clients are found to have shipped a way to accept a certificate that is not a
pin, whether through their own transport or through a configuration the core
cannot see. That is evidence the pin is too narrow to be used rather than too
wide, and the replacement records what those clients actually needed instead of
restating a rule they went around.

A supported platform is found to expose no way to see the presented chain after a
refusal. The exception then cannot be offered there at all, because the client has
nothing to show, and this record is superseded by one that says so for that
platform rather than implying a route it does not have.

The reason classes above turn out not to be derivable from what a platform reports,
so that two platforms produce different classes for the same certificate. Measured
on a real refusal rather than assumed. The set is then wrong, and it is superseded
by one built from what the platforms actually say.
