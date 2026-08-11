# 0027. The transport's timeouts, its connection limit, and connection reuse

Date: 2026-08-11

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #27

## The decision

Every request the core makes goes through one transport, which bounds an
attempt with two per-attempt timeouts, two seconds to reach a connection and
two seconds to the first byte of a response, inside the five second call
deadline record 0007 already sets rather than beside it; it holds at most
six connections to one server and twelve across all servers, keeps an idle
one for sixty seconds, and reuses it for any session against that server;
and it reads the remainder of a cancelled response only while reading is
cheaper than a handshake, closing the connection after sixty-four kilobytes
or one second rather than spending a person's link on bytes nobody wants.

## The two timeouts, and the third number that is not one

The connect timeout is two seconds. It runs from the moment an attempt begins
and covers resolving the name, reaching the machine, and the handshake through
the point where 0029 decides whether the certificate is acceptable. Resolution
is inside it rather than beside it because a resolver that does not answer
and a server that does not answer are the same thing from the caller's side,
and splitting them would be a fourth number with no caller able to tell
the two apart. A name that resolves to nothing is not this timeout at all:
0007 reports `unreachable` at once on a name that did not resolve, a refused
connection and an absent route, and none of those involves waiting.

The first-byte timeout is two seconds. It runs from the moment the request
has been written to the moment the first byte of the response arrives. This
is the one 0007 says a single overall timeout handles worst, because a server
that accepts a connection and then says nothing is otherwise indistinguishable
from one that is merely slow until the whole deadline has gone.

The whole-request timeout is the five seconds in 0007 and is not decided
again here. It belongs to the call rather than to the attempt, which is
0038's sentence, so an attempt is bounded by whichever of the two arrives
first: the two seconds above, or what is left of the caller's five.

Neither per-attempt bound is a smaller deadline in disguise. Their purpose
is to end a stalled attempt early enough that 0038 can spend another one
inside the same five seconds, which a single overall bound cannot do.

### The arithmetic, so that a later change is mechanical

0038 allows three attempts, waits drawn from zero to 250 ms and then zero to
500 ms, and refuses to start an attempt with less than 500 ms of deadline left.

Two stalled attempts cost four seconds of the five, plus the two waits, which
leaves between 250 ms and one second. So a third attempt starts only where
the two waits together drew under 500 ms, and where it does start it runs
against what is left of the deadline rather than against the two seconds
here. That is the intended shape. The third attempt exists for failures
that arrive fast, which is what a status code is, and a stall is expected
to consume the budget rather than to be repeated three times.

The first-byte bound is above 0007's 400 ms and by a distance. A bound at or
below it would abandon attempts before the core had ever reported `late`, which
would make the `late` state unreachable in exactly the case it was written for.

Every number here is chosen rather than measured, in the same way as the
thresholds in 0007 and the waits in 0038, and for the same reason: there is
no code in this repository to measure. #65 is the harness that would replace
a choice with a number.

## What they are measured against

All three are durations between two events inside one run, so 0102 puts them
on the steady clock, and its table already names the request timeout and the
read timeout inside it. A device that suspended mid-attempt has no attempt
left to time out, which is why they are not on the elapsed clock.

The controlled source in 0102 is what makes each of them provable in
microseconds. A test advances the steady clock past a bound and asserts
the outcome, and none of that waits on real time.

## Nothing bounds the rate of a body

There is no bytes-per-second rule and no separate bound on the time between two
bytes after the first. The call deadline is what ends a body that trickles,
and 0055 bounds an image body at sixteen megabytes of encoded length while
it is being read rather than after it is complete.

The residual is worth stating rather than discovering. A response that
genuinely needs more than five seconds is abandoned, and on a slow link
a large artwork body is such a response. That is deliberate at this size:
#49 asks the server for the size that will actually be drawn, so an ordinary
tile is far below the refusal bound, and a tile that cannot arrive inside the
deadline is one no client should be holding a screen for. If ordinary artwork
on a supported link is measured exceeding it, the answer is a deadline per
kind of request, which supersedes 0007 rather than this record.

A media stream is not one of these requests. 0005 and 0112 put the stream in
the platform's own player against an address the core hands over, so nothing
long-lived is carried here.

## The connection limit

Six connections to one server. Twelve across all servers.

Six, because it bounds the two hundred tiles in #53 into a queue rather
than into two hundred sockets, and because a server people run at home is
frequently a small machine, where the sixth concurrent request is already
competing with the one somebody is waiting for. Browsers settled on six per
host for the same protocol version, which is a claim here rather than
anything this repository measured, and it is worth knowing before choosing
a different number rather than an argument on its own.

Twelve, because the limit has to be a fixed number at the moment the core
is created. 0009 creates its lanes then and sizes the waiting lane from this
limit, so a per-server limit multiplied by a server count that changes when
a session is added would mean a lane that has to grow, which 0009 does not
do. Twelve is two servers each able to hold their six, which is #114's ordinary
case, and it fixes the waiting lane at twelve waiters for the life of the core.

The per-server six is a ceiling and not a reservation, and the cost of that
is the third server. Where three or more are active at once they contend for
the twelve, and a stalled server can hold six of them for as long as 0007's
deadline. That is named rather than solved: reserving slots per server would
idle most of them on the ordinary single-server device, and what frees a
stalled slot is the abandonment 0007 already performs, which is the reason
it gives for abandoning at all.

The count is over requests outstanding against one server, whatever number of
sockets carries them. 0009 sizes its waiting lane at one waiter per permitted
connection and that stays true, because the number that sizes the lane is
this count. Stating it over requests rather than over sockets is what keeps
it meaning the same thing if the protocol version turns out to multiplex.

## What a connection is reused for, and what ends one

A connection is reused for any request to the same origin, including for
a different session against that server. Identity travels in the request,
as a header carrying the token from 0005 that the store in 0033 handed over,
and the core presents no certificate of its own, so a connection carries no
identity that two sessions could confuse. That is what makes several sessions
against one server in #114 cost no more connections than one.

A connection is never reused across origins. 0028 fixes what an origin is,
0069 fixes which ones may be reached at all, and 0029 evaluates its pin per
connection rather than per request precisely so that a reused connection is
one whose certificate was already accepted.

Five things end a connection, and each of them is a case where reusing it
would be either wrong or slower than a handshake.

Sixty seconds idle. A connection nothing has used for a minute is one whose
survival the core cannot know, and writing on a dead one is not free: it costs
the two second first-byte bound above and then a retry, which is worse than
the handshake it was avoiding. Sixty seconds is chosen rather than measured,
and #65 is where a measurement would replace it.

The server closing it, which needs no rule and is named here only so the
list is the whole list.

A response the core did not read to its end. 0009 already says bytes in flight
are read and discarded rather than left in the socket, because a connection
with unread bytes on it cannot be reused. Where that reading stops short,
the connection goes with it.

Cancellation past the bound in the next section.

The pin for that server changing, under 0029. Every connection to it is
closed, because each was accepted under the pin that has just been replaced,
and a connection that outlived the decision that admitted it is the one case
where reuse would carry an old answer forward.

## A cancelled request, and the bytes still coming

0009 decides what cancellation guarantees and this record does not restate
it. What 0009 leaves open is how far the reading it describes goes, and an
unbounded answer would let a cancelled call hold a connection out of the
limit above for as long as the deadline, which is the resource cancellation
was supposed to release.

The core reads the remainder of a cancelled response until sixty-four
kilobytes have been read or one second has passed, whichever comes first,
and then closes the connection instead.

Both numbers are the same argument. Reading on is worth doing only while it is
cheaper than the handshake it saves, and beyond that it is spending somebody's
link on bytes that have already been discarded, which is the cost 0045 refuses
when it declines to refetch on recovery. Sixty-four kilobytes is the order of
a body the core would have taken in a read or two anyway. A cancelled image
is the case that decides the shape: at up to sixteen megabytes under 0055,
an artwork body is far past the bound, so a tile scrolled off the screen
closes its connection rather than downloading in full to save a handshake,
and that is the right way round for the wall in #53.

## What this record does not decide

The protocol version. It follows from the means in #11 and from what
#10 records about the server surface, and the limit above is stated over
outstanding requests so that it means the same thing either way.

Redirects. 0069 decides them, inside the origin and nowhere else, and names
this transport as the one line of configuration where the default would have
been to follow them anywhere.

Which failures are retried, how often, and after what wait. That is 0038,
and the per-attempt bounds here exist to serve it.

What the states are called and when each is reported. That is 0007.

What cancellation promises to a caller. That is 0009.

## Why this is written down before the code

A transport is the artefact that gets its behaviour by default rather than
by decision. Every client library in every language ships with a timeout,
a pool size and a reuse policy already set, and the first request written
against it inherits all three without anybody choosing them. The values
are then invisible: they appear in no line of this repository, so a reader
looking for the connection limit finds nothing and concludes there is not one.

Two records already depend on a number that would have been arrived at that
way. 0007 abandons a request at five seconds to protect the connection limit,
and says so as the reason for abandoning at all, which is an argument that
evaporates if the limit turns out to be a default nobody set. 0009 sizes its
waiting lane at one waiter per permitted connection, so a lane that is created
once at core creation needs the number to exist before it is created. Both
are pointing here, and until this record neither could be checked.

The two per-attempt bounds have to exist before the retry loop for a sharper
reason. 0038 spends three attempts inside one five second deadline, which
is only possible if a stalled attempt can be ended early. A transport with
a single overall timeout makes the second attempt unreachable in exactly
the case 0038 was written for, and nothing about that failure looks like a
missing timeout: it reads as a retry policy that does not retry.

## Alternatives, and what each cost

One overall timeout per request and nothing else. The smallest thing that
can be written, and it is what a caller thinks a timeout is. It costs the
case 0007 names first, a server that accepts a connection and then says
nothing, where the caller waits the entire deadline for information that
was available in the first two seconds. It also costs 0038 its retries,
since there is no attempt to end early.

A timeout per attempt with the retries added on top. The common shape, and
each attempt gets a fair chance rather than inheriting what is left. It costs
the bound outright, which 0038 has already rejected in the same words: three
attempts at five seconds each is a caller waiting fifteen seconds while 0007
says the answer should have been given up on at five.

A bound on the time between two bytes rather than a bound on the first
one. Better against the server that stops answering half way through a body,
which the first-byte bound does not catch at all. It costs a number that
has to hold for both a small answer and an artwork body, and the deadline
already ends the case, so what it buys is confined to the seconds between
the first byte and the five.

Client-supplied timeouts and a client-supplied limit, tuned per platform. A
real argument on a television, and the same one 0007 rejected for its
thresholds and 0038 for its waits. It costs the drift this repository exists
to remove, and it makes the core's share of the number in #62 unmeasurable,
because the share would depend on values the core does not know.

No limit on connections, letting the platform's own pool decide. Nothing to
size, and on a platform that pools well it is the better answer. It costs
0009 the number that sizes its lane, it costs 0007 the reason it abandons a
request, and it hands the wall of two hundred tiles in #53 to whatever the
default happens to be on the slowest supported target.

A per-server reservation rather than a shared total. It removes the
third-server contention named above outright, and it is the honest answer
if several servers at once turn out to be ordinary rather than unusual. It
costs idle waiters on the single-server device, which is nearly every device,
and it makes the waiting lane a thing that grows when a session is added,
which is a change to 0009 rather than a number here.

Reading a cancelled body to its end, however long it is. Every connection
stays reusable and there is no bound to argue about. It costs a person's link,
and it costs it worst on the case that is most common, since a cancelled
tile is cancelled because nobody is looking at it.

Closing a cancelled connection at once, with no reading at all. Simplest,
and it never spends a byte on discarded work. It costs a handshake on every
cancellation, and on a wall of tiles the ordinary case is a cancellation of
something small that had almost finished.

## What would reverse this

The five seconds in 0007 changes. Everything here is fitted inside it, and
the arithmetic above is written so the refitting is mechanical rather than
a fresh argument.

The harness in #65 measures a real round trip against a supported server
and finds that two seconds does not reach a connection, or that a first
byte routinely takes longer than two seconds on a supported link. That is
the measurement this record is waiting for, and it replaces a chosen number
with one.

Ordinary artwork on a supported link is measured exceeding the call
deadline. The answer is then a deadline per kind of request rather than a
change here, and it supersedes 0007.

Three or more servers active at once turns out to be ordinary rather than
unusual, and the contention named above is observed starving one of them. The
limit then becomes a reservation, the waiting lane becomes a thing that is
sized differently, and 0009 is superseded alongside this record rather than
left pointing at a number that has changed meaning.

The means chosen in #11 offers no way to bound a connection attempt separately
from a read, or no way to see a connection before it is made. The first makes
the split above unimplementable and this record is superseded by one saying
what can be bounded. The second is already 0069's reversal condition and is
the more serious of the two.

A cancelled body is measured to be small enough, in ordinary use, that the
sixty-four kilobyte bound never decides anything. The bound is then ceremony,
and the record that replaces it says either that a cancelled connection is
always closed or that it is always drained, rather than keeping a number
that does nothing.
