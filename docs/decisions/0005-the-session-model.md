# 0005. The session model, its lifetime, and a token that dies mid-playback

Date: 2026-08-08

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #5

## The decision

A session is the pairing of one server with one account on one device, identified
by the three together rather than by the token it holds; the core keeps as many as
a client asks for and treats exactly one of them as current for any given call;
the token is the only secret, the core never writes it anywhere except a store the
client supplies, and the whole lifetime is driven by a server rejecting a request
rather than by the core trusting a stated expiry.

## What a session is

A session is identified by three things: the server it was established against,
the account it was established for, and the device identity established in #36.
Two of the three matching is not a match. The same account on the same server from
a second device is a second session, because the server issued a second token and
may end either one without ending the other.

A session holds the resolved server address, the account identifier the server
gave back, the device identity, the token, whatever the server said about the
token's validity, and the capability answers from #10 for that server. Everything
in that list except the token is ordinary data and may be cached, logged and shown.

The token is the only secret. It is the whole of what an attacker needs, so it is
the whole of what is protected. It never appears in a cache key, in a cache value,
in a diagnostic event, in an error, or in anything a person is asked to send.
The proof that this holds for the cache is #48.

## How many sessions at once, and what current means

The core holds any number. One device carrying sessions for two servers, and one
server carrying sessions for two people, are both ordinary and are already assumed
by the cache keying in #41. Holding several is #114 together with sign out.

Every call that reaches a server is made in the context of exactly one session,
and the caller says which. There is no ambient current session that a second call
can change underneath the first, because that is the shape in which one person's
request is answered with another person's token.

Moving between sessions does not disturb what the core is holding for the one
being left. Requests already outstanding against the leaving session run to
completion or are cancelled by the caller, and their answers are attributed to the
session they were made in. Cached entries stay, because they are keyed per session
in #41 and are not the arriving session's to read. Positions recorded but not yet
reported stay queued against their own session, and the queue in #47 is per
session for the same reason.

Signing out of one session ends that session's token at the server, discards its
secret, and leaves every other session untouched. What happens to its cached
bytes and its unsent positions is #114's to decide, and this record only requires
that the answer be the same whether the sign-out was asked for or forced by a
rejection.

## Who stores the secret

While a session is in use the core holds the token in memory, because it has to
put it on a request.

At rest, the only acceptable place is the platform's own secret store: a keychain,
a keystore, a credential locker, or whatever the platform calls the facility whose
contents are protected by the platform rather than by file permissions the
application chose. Reaching that facility is platform work, so the core does not
do it. It is a client responsibility behind the named interface in #33.

The core never writes a token into its own cache, its own diagnostics, or any
file it chose the location of. It has no fallback for a client that does not supply
a secret store: a session then lives only as long as the core does, and the person
signs in again next time. A file the core wrote itself, with or without
obfuscation, is not a fallback, it is the same secret in a worse place.

## How a session is acquired

Three routes, and they converge at one point.

Username and password, in #30. The core sends what the person typed and receives a
token, the account identifier and the server's statement about validity. This is
the only route where the core touches a password, and it holds it for the length of
one request and never stores it.

Quick Connect, in #31. The core asks the server to begin an exchange, receives a
short code for the client to show, and then waits for the server to say the
exchange was approved elsewhere. The core never sees a credential on this route.
What it must handle that the first route does not: a wait with no upper bound
imposed by the core, a code that expires, an exchange a person abandons, and the
fact that the person approving is doing so somewhere the client cannot see.

A server that delegates to an external identity provider, in #32. The core hands
back what the client must open, the platform opens it, the person authenticates
somewhere the core has no visibility into, and the core receives the result the
server issues at the end. The core is never in the middle of that exchange and
never sees the provider's own credentials or its tokens. It knows only that the
server accepted the outcome and issued a session.

All three converge at one place: a token, an account identifier, and whatever the
server said about validity. Everything after that point in the core is written once
and does not know which route produced the session. Where a route needs something
the others do not, that difference stays inside the route.

## Token lifetime

The ground truth is a server rejecting a request. The core treats an
authentication rejection on any call as the authoritative statement that the token
is no longer usable, and its renewal path is driven from there. Renewal ahead of a
stated expiry is an optimisation layered on that path, never a replacement for it,
and #34 is where both are implemented.

The rejection path is built first because it is the only one that cannot be
avoided. A token can be ended at the server before the moment it said it would
expire, by a sign-out elsewhere, by an administrator, or by the server restarting
with different state, and none of those are visible to a core that is watching a
clock. A core that only renewed ahead of expiry would still meet a rejection it had
no path for, at which point the path gets written in a hurry inside whichever call
met it first.

Renewing ahead of expiry costs a clock the core can trust, and which clock a
deadline is measured against is #102's to name. It is worth paying for on one
count: it moves the renewal off the path of a request a person is waiting on. The
cost of not paying it is one extra round trip on the first request after a token
lapses, which is inside the budget in #62 for everything except the case below.

What the core does on a rejection, in order: it stops sending on that session; it
attempts renewal once; on success it retries the rejected request itself, once,
transparently to the caller; on failure it moves the session to a signed-out state,
discards the token, tells the client through the diagnostics interface in #100, and
maps the original call to the vocabulary member in #4 that means the person has to
sign in again. It does not retry renewal in a loop, because a server that refused a
renewal is not going to accept the same one a second later, and a loop against an
authentication endpoint is the shape that gets an address blocked.

## A token that dies mid-playback

This is the case that decides whether the model is any good, because it is the one
where the person is not looking at a sign-in screen and has not asked for
anything. The sequence, and #35 is where it is implemented.

Playback already in flight is not interrupted. The stream is being read by the
platform's player against the address handed over in #111, and the core does not
have a way to stop it, does not want one, and would be wrong to use one. A person
watching something does not stop watching because a background report failed. This
is the guarantee, and every step below is arranged so as not to break it.

The position report that was due when the rejection arrived is not lost and is not
retried immediately. It goes onto the same queue that holds positions recorded
while the server was gone, in #47, keyed to the session it belongs to. Reports
already queued stay queued and unchanged. Nothing in the queue is discarded because
of an authentication failure, because the failure says nothing about whether the
positions are correct.

The core then attempts renewal once, exactly as it would for any other call. On
success it drains the queue in order, reports the current position, and playback
continues having never noticed. Nothing is told to the client except through
diagnostics, because nothing happened that a person needs to know about.

On failure the session moves to signed-out. The queue is kept rather than dropped:
it belongs to the session, and a person who signs in again to the same account on
the same server gets those positions reported, in order, before anything else is
sent. The core stops attempting to report while the session is signed out, and
does not retry on a schedule, because every attempt would fail for the same
reason.

What the client is told, and when. At the moment renewal fails, through #100, that
this session can no longer report and that positions are being held. Not as an
error on any call the client made, because it made none. The client decides whether
to show anything, and a client that shows nothing until playback ends is behaving
correctly.

What is guaranteed about the position that was reached. The last position the core
observed before the rejection is held with the precision decided in #56, and it is
reported once reporting is possible again. What is not guaranteed: any position
reached after the token died. The core learns positions on the cadence in #57, and
once it can no longer report it also stops being told, so the recorded position is
the last one on that cadence rather than the moment playback actually stopped.
The gap is therefore bounded by the cadence in #57 and by nothing else, and #58 is
where the rewind on resume absorbs it. A core that claimed the exact stopping point
here would be claiming something it never observed.

## Why this is written down before the code

The mid-playback case is the one that gets designed last, inside whichever call
first met it, and by then the session is whatever four call sites assumed. Two of
them will have retried in a loop, one will have dropped the queued positions
because dropping them made a test pass, and the fourth will have stopped playback,
which is the only outcome a person actually notices. None of that is recoverable by
review of a single change, because each change looks reasonable where it sits.

The rest follows from the same problem one step earlier. A core with an ambient
current session is a core in which two accounts on one device eventually see each
other's data, and that defect is found by a person, in public, rather than by a
test.

## Alternatives, and what each cost

One session at a time, switched by signing out and in. Considerably less to hold
and to key, and it refuses the case in #41 that this board has already committed
to. It also makes moving between two servers a re-authentication, which on a
television with an on-screen keyboard is the difference between a feature and one
nobody uses.

The core reaching the platform secret store itself. Removes an interface and a
thing every client has to supply. It puts platform-specific code inside a core
whose whole claim is that it has none, needs a conditional per platform, and cannot
be tested without the platform, which loses the headless property in #20 at the
one seam where a mistake is a leaked credential.

The core keeping its own encrypted file when no secret store is supplied. Looks
like a safe default and is not one. The key has to live somewhere the same process
can read, which means an attacker who can read the file can read the key, and the
outcome is a token at rest with a claim of protection over it. Signing in again is
a real cost paid by a person; this is an unreal protection paid for by nobody.

Renewing only ahead of a stated expiry. One fewer round trip on a lapsed token and
no rejection handling on the ordinary path. It has no answer for a token ended
early, so the rejection path gets written anyway, later, in the wrong place.

Stopping playback when a token dies. Simple, consistent, and the single most
visible way to fail. It converts a background problem the core can usually fix
without anybody noticing into an interruption in the middle of something a person
chose to watch.

Reporting positions on a signed-out session by retrying on a schedule. Would
eventually deliver them if the server changed its mind. Servers do not change their
mind about a rejected token, and the schedule is a stream of failing requests
against an authentication endpoint for as long as the process lives.

## What would reverse this

A server line the core must support issues tokens whose validity cannot be
established from a rejection, for instance by answering a call with a success that
silently returns nothing. The rejection-driven model has no signal there and would
be replaced by one that renews on a schedule.

The queue in #47 turns out to be unable to hold a position across a sign-in,
because #114 decides that signing out discards a session's queued work. The
guarantee in this record about positions surviving a failed renewal would then be
wrong and would have to be withdrawn rather than quietly weakened.

Two or more clients supply a secret store whose contents do not in fact survive an
application update or are not in fact protected by the platform. The interface in
#33 would then be a guarantee nothing keeps, and the honest replacement is a
record that says the core holds sessions in memory only.

The cadence in #57 is measured and found so coarse that the position lost on a
mid-playback failure is one a person notices. The bound this record states would
then be unacceptable rather than merely stated, and the answer is a position the
core observes on its own rather than a tighter cadence.
