# 0030. The password route, and what the password is allowed to touch

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #30

## The decision

The password route takes the base address 0028 produced, an account name and a
password, presents the password to the configured server inside one call and
holds it nowhere else and for no longer than that call, turns the answer into the
session 0005 fixes or into one of 0004's kinds by 0004's own mapping with no
error of its own, and is never repeated on anybody's behalf afterwards, because
the core keeps nothing it could repeat it with.

## What the call takes, and what it answers with

Four things go in. The base address, already resolved by 0028, so that this route
never sees what a person typed and never repairs an address. The account name, as
the person typed it. The password. The device identity from #36, because 0005
makes it part of what identifies the session the server is about to issue.

One of two things comes out. A session as 0005 describes it, holding the resolved
address, the account identifier the server gave back, the device identity, the
token, whatever the server said about validity, and the capability answers #10
fixes. Or one of the fifteen kinds in 0004, with the payload that kind already
carries.

Nothing else comes out. There is no third answer meaning "signed in but", and
there is no field on the session saying how it was acquired. 0005 fixes that all
three routes converge on one shape, and a field naming the route would be the
thing every later caller branches on, which is the convergence undone one
condition at a time.

## The password's whole life

It exists in the core between the moment the caller hands it over and the moment
the request carrying it has been written. That is the whole of it.

It is not held after the call. Not for a renewal, which 0034 drives from a
generation number and a rejection rather than from a credential. Not for a
retried sign-in after the session ends, which 0034 refuses by name. Not for a
second server, and not for the same server on a second attempt after a rejection,
where the caller supplies it again because the person typed it again.

It is not written anywhere. Not into the secret store, which 0033 fixes as
holding the token and nothing else. Not into the cache, which 0006 and 0041
already keep the token out of and which the proof in #48 covers. Not into a
diagnostic event, which 0071 excludes outright with no severity that admits it.
Not into an error payload: 0004 lists what each of the fifteen kinds carries, no
row of that list holds a credential, and the row this route reaches most often,
`not-authenticated`, carries whether a token was presented and nothing else.

It is not part of anything derived. Not a cache key, not a secret store name, not
a correlator, not an identifier for a diagnostic event. 0041 builds a key from
the server, the account identifier and the device, and that is the whole of the
input.

The account name is a different thing and is treated differently. 0068 places it
on the personal data list, so it reaches a diagnostic event under 0071's rule for
a named field rather than under the password's exclusion, and 0006 already keeps
it out of the cache key in favour of the identifier the server gave back. It is
data about a person. The password is not data at all in that sense: nothing in
the core has a reason to carry it one step past the request.

## The three answers this route has to keep apart

The issue names three and the mapping for all three is 0004's, arriving here
unchanged. That is the point worth recording: this route adds no vocabulary and
no route-specific error, because a sign-in that reported failures in its own
words would be the first of eleven clients' worth of drift, inside the call every
client makes first.

A credential the server did not accept is `not-authenticated`, with the payload
saying there was no token to present. 0034 already reads it that way from the
other side: a rejection carrying "a token was presented" is a session that has
ended and starts a renewal, and a rejection carrying "there was none" is this
route being told the name and the password do not go together. One kind, two
payloads, and the difference is the thing #34 and #35 act on.

A server that did not answer is `server-unreachable` or `timed-out`, and neither
says anything about the credential. 0007 decides which: a name that did not
resolve, a refused connection and an absent route are evidence about the server,
and a deadline reached with no answer is the other. Collapsing either into a
rejected credential is how a person retypes a correct password at a server that
is switched off, which is the failure the issue names.

An answer the core could not read is `answer-not-understood`, carrying what was
being read and where reading stopped and never the answer itself. 0004 puts a 2xx
whose body does not parse there, and 0004's closure rule puts a body that parses
but omits the token, the account identifier or the validity statement there as
well. There is no branch in this route that returns a session with a field
missing.

Two more answers exist at this door and are worth naming so that nobody adds a
fourth kind for them later. A server that accepted the credential and will not
let this account sign in is `not-permitted`, which is 0004's 403 row and carries
nothing. A server line with no password route at all is `capability-absent`
carrying the name of the capability from #10's set, which is 0004's row for a 404
on a resource the interface says should exist, reaching the one call where an
operator will actually meet it.

## Whether this call is retried

It is, under 0038, unchanged and with no exception for carrying a credential.
`timed-out`, `server-busy` and `server-failed` are retried inside the call, at
most three attempts inside 0007's five second deadline, and the rest are not.
`not-authenticated` is not retried, which is 0038's rule and is the one that
matters here: a password the server refused is not going to be accepted a moment
later, and a loop against an authentication endpoint is the shape that gets an
address blocked.

Retrying inside the call costs nothing this record has to protect against,
because the password is already in memory for the duration of that call and a
second attempt does not extend its life by a millisecond.

It costs one thing that is worth writing down rather than discovering. A
`timed-out` whose payload says bytes reached the server may be a sign-in that
succeeded, with a token the core never received, and the retry that follows it
may produce a second one. The residue is a session in the server's own device
list that nothing here will ever end, because the core holds no token for it and
0033's store was never given one. The core does not attempt to clean that up and
does not pretend it can. An operator can see it and end it from the server's own
side, which is a thing they can act on only if somebody said it happens.

The alternative, a route that refuses to retry because it carries a credential,
buys nothing against that. It converts one lost packet into a person retyping a
password, and on a television that is an on-screen keyboard.

## What this route does not decide

Whether the address is usable at all, and what becomes of one with no scheme or
with credentials inside it. 0028, before this route is reached.

What happens when the machine that answered did not prove it is the one the
address named. 0029, and it happens before any byte of this request is sent.

Where the token rests afterwards, and what a core with no secret store does.
0033.

What a rejection later in the session does. 0034, on 0005's model.

The other two acquisition routes. #31 for the code a person approves elsewhere,
and #32 for a server that delegates. 0005 fixes that all three converge, and the
difference between them stays inside each route.

## Why this is written down before the code

The password's lifetime is decided by whoever writes the first sign-in call, and
the natural shape of that call keeps it. A credentials object held on the client
of the core so that a retry is easy. A field on the session so that a renewal can
re-authenticate. A parameter threaded through a helper so that a test can sign in
twice. Each of those is a reasonable line to write and each puts a password
somewhere that outlives the request, and the one that gets written is the one
that makes a test shorter.

None of them looks wrong in review, because the wrongness is the lifetime rather
than the line, and a lifetime is not visible at a call site. It becomes visible
in a crash report, in a diagnostic bundle an operator was asked to send, or in
whatever a platform writes when a process is suspended, which are the three
places nobody is looking when the line is written.

The second half is the failure mapping, and it fails in the other direction. A
sign-in call is written first, before there is a mapping to reach for, so it
returns whatever the first author found expressive. By the time 0004's table
exists in code, sign-in has its own three errors and every client has already
branched on them. Writing the mapping down now costs a section. Writing it after
one client has shipped costs that client, and the mapping is the part a client
author is least willing to change, because it is the screen a person sees when
nothing else works.

Neither has happened here, because there is no code in this tree and no language
in which to write any.

## Alternatives, and what each cost

The core keeping the password for the life of the session, so that a rejection
can be answered by signing in again rather than by renewing. It removes the
renewal path entirely for this route, and on a server line with no renewal route
it is the only way a session survives an expiry without a person. It costs the
whole of the rule above: a credential at rest in memory for as long as a
television is on, in a process that is suspended, dumped and inspected by the
platform, and 0101 already treats that device as shared with another person and
losable. 0034 refuses the same thing from its own side and gives the same reason.

A route-specific error type, so that a client handling sign-in can match on
exactly the conditions sign-in produces. It reads better at the one call site
where a client author is paying most attention. It costs 0004's closure, which is
the property that makes a client's handling provably exhaustive, and it costs it
at the call every client writes first, so the exception becomes the example the
next route is written from.

Refusing to retry anything on this route, on the argument that a credential
should be sent once. It is the intuition, and it is why it is worth answering
rather than ignoring. It buys nothing, because the password's exposure is the
call and not the attempt, and the orphan session above is a consequence of the
network rather than of the retry: a single attempt that timed out after bytes
reached the server leaves exactly the same residue and additionally makes the
person retype. It also puts a second retry policy in the tree beside 0038's, and
two policies disagree the first time either is edited.

Distinguishing a wrong name from a wrong password. Every person who has typed a
name wrong wants this, and a server that answered them separately would let the
core say it. It costs an account enumeration oracle at an unauthenticated
endpoint, which is the standard reason no server offers the distinction, and a
core that inferred it from response timings or from a second call would be
building the oracle the server declined to build.

Taking the password as a plain string rather than as something the caller can
clear. A string is what every client has, and asking for anything else puts work
on eleven client authors for a property the runtime may not offer. It costs
whatever the chosen means in #11 leaves in memory after the reference is dropped,
which is a real cost and an unmeasurable one here, and this record takes the
string and states the residual rather than claiming a scrub it cannot perform.

## What would reverse this

A supported server line answers a wrong name and a wrong password differently, on
its own, without the core inferring anything. The distinction is then available
rather than manufactured, and the payload on `not-authenticated` grows a field
rather than the vocabulary growing a kind.

The chosen means in #11 offers a credential type whose bytes can be cleared on a
schedule the runtime guarantees. The residual named in the last alternative above
stops being a residual, and the record is superseded by one that takes that type
and says what it promises.

A measurement on the diagnostic events in #100 shows the orphan session above
happening often enough that operators are meeting it. Then the retry on this
route is costing something real rather than something stated, and the replacement
decides between a single attempt and a route that asks the server to end a
session it issued to nobody.

`not-authenticated` with no token presented turns out to be reached by something
other than a refused credential on a supported server line, twice. Once is a
mapping mistake. Twice means the payload is not the distinction #34 and #35 think
they are reading, and the record that replaces this one says what else lands
there.
