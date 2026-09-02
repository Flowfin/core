# 0010. The server surface, and what an absence does

Date: 2026-08-27

Status: accepted. Supersedes nothing. Superseded by 0272.

Issue: #10

## The decision

The core reaches a Jellyfin server through the sixteen named capabilities
enumerated below and through no other part of its interface; a capability name is
what `capability-absent` carries in 0004, a 404 means an absent route on a path
this record lists that carries no caller-supplied identifier and a missing
resource on one that does, and the core establishes no capability by probing
except where a landed record already required a read before the call for a reason
of its own.

## What was read, and where

Entry 3 of #1 was answered on 2026-08-24 with the two plugin lines, 10.11 and
12.0, so this record is written against those two and against nothing else.

Both lines were read in the public server repository, at one commit each, named
here so that a reader gets the same bytes:

    $ git clone https://github.com/jellyfin/jellyfin
    $ A=1fbd8739292cce610231be93daf43368733edf63    # the 10.11 line
    $ B=c3ed1407ca698b0905de99da87b67415e6a62dbd    # the 12.0 line
    $ for r in $A $B; do
    >   git -C jellyfin show $r:SharedVersion.cs | grep AssemblyVersion | head -1
    > done
    [assembly: AssemblyVersion("10.11.11")]
    [assembly: AssemblyVersion("12.0.0")]

Those are the tips of the two lines as one clone held them, which is a statement
about one fetch rather than about the lines. A later commit on either is a later
commit, and the reversal condition at the end of this record is written against
that rather than against a promise that it will not happen.

The whole route surface of both lines was compared before anything below was
written, so that a difference is found rather than looked for:

    $ for r in $A $B; do
    >   git -C jellyfin grep -hE '^ *\[(Route|Http(Get|Post|Delete|Put|Head))' $r \
    >     -- Jellyfin.Api/Controllers | sed -E 's/^ +//; s/\]$//' | sort -u > $r.routes
    > done
    $ comm -23 $A.routes $B.routes
    [HttpGet("Initiate")
    [HttpGet("Items/{itemId}/CriticReviews")
    [HttpGet("NetworkShares")
    [HttpGet("Recordings/Groups/{groupId}")
    [HttpPost("{userId}/EasyPassword")
    [HttpPost("MediaEncoder/Path")
    $ comm -13 $A.routes $B.routes
    [HttpGet("Items/{itemId}/Collections")

Seven route attributes differ across the two lines out of the several hundred
each carries, and exactly one of the seven is inside the surface below. The other
six are administrative, live-television or metadata routes that no record on this
board reaches.

## The sixteen capabilities, and the paths that carry them

Every path is joined to the base address 0028 resolved, by the rule that record
already fixes. Every authenticated call carries the four parts 0036 names in the
authorization value, and both lines read the same five values out of it:

    $ for r in $A $B; do
    >   git -C jellyfin grep -c 'auth.TryGetValue' $r \
    >     -- Jellyfin.Server.Implementations/Security/AuthorizationContext.cs
    > done
    1fbd8739292cce610231be93daf43368733edf63:Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:5
    c3ed1407ca698b0905de99da87b67415e6a62dbd:Jellyfin.Server.Implementations/Security/AuthorizationContext.cs:5

so 0036's reading, taken at one commit, holds across the range this record fixes,
which is the thing that record sent here.

The column headed `repeat` says what a second identical call leaves behind: an
`assertion` leaves the server in the state the first call intended, and an
`accumulation` leaves something the first one did not. It is here because 0038
says in so many words that which calls change server state is this record's and
not that one's. 0038 retries by error kind and never reads this column; what
reads it is #47, deciding what a queue may replay.

| Capability | Method and path | Purpose | repeat |
| --- | --- | --- | --- |
| `server-identity` | `GET /System/Info/Public` | The identifier and version the server reports about itself. 0041's server part. | assertion |
| `password-sign-in` | `POST /Users/AuthenticateByName` | 0030's route. | accumulation |
| `quick-connect` | `GET /QuickConnect/Enabled` | Whether the operator has this route on. Read before an exchange is started. | assertion |
| | `POST /QuickConnect/Initiate` | Begin an exchange and receive a code and a secret. | accumulation |
| | `GET /QuickConnect/Connect` | Ask whether the exchange was approved. 0031's poll. | assertion |
| | `POST /Users/AuthenticateWithQuickConnect` | Exchange an approved secret for a session. | accumulation |
| `delegated-sign-in` | none on either line | 0032's route. | not applicable |
| `token-renewal` | none on either line | 0034's exchange of a live token for a fresh one. | not applicable |
| `sign-out` | `POST /Sessions/Logout` | 0114's ending of one session. | assertion |
| `device-capabilities` | `POST /Sessions/Capabilities/Full` | The description 0036 says a client supplies. | assertion |
| `library-query` | `GET /UserViews` | The top of the library, which is a first screen's first request. | assertion |
| | `GET /Items` | The query surface #39 builds on. | assertion |
| `item-detail` | `GET /Items/{itemId}` | One item, in full. | assertion |
| `resume-list` | `GET /UserItems/Resume` | What 0058 resumes into. | assertion |
| `item-user-data` | `GET /UserItems/{itemId}/UserData` | The position and the watched mark, read. 0056 and 0060. | assertion |
| | `POST /UserItems/{itemId}/UserData` | The same, written. | assertion |
| `artwork` | `GET /Items/{itemId}/Images/{imageType}` | The bytes #49 builds an address for. `HEAD` on the same path answers the same. | assertion |
| `playback-selection` | `POST /Items/{itemId}/PlaybackInfo` | 0111's choice of source, carrying 0036's description in the body. | accumulation |
| `playback-progress` | `POST /Sessions/Playing` | Playback started. | accumulation |
| | `POST /Sessions/Playing/Progress` | 0057's cadence. | assertion |
| | `POST /Sessions/Playing/Stopped` | Playback ended. | accumulation |
| `played-marking` | `POST /UserPlayedItems/{itemId}` | 0060's mark, set. | assertion |
| | `DELETE /UserPlayedItems/{itemId}` | 0060's mark, cleared. | assertion |
| `change-notification` | A WebSocket upgrade on the resolved origin | 0116's connection. | assertion |

`POST /Items/{itemId}/PlaybackInfo` is an accumulation rather than an assertion
because its body carries a field that makes the call open a live stream, so a
repeat can open a second one:

    $ git -C jellyfin show $B:Jellyfin.Api/Models/MediaInfoDtos/PlaybackInfoDto.cs \
    >   | grep -n 'AutoOpenLiveStream'
    84:    public bool? AutoOpenLiveStream { get; set; }

The `GET` form of the same path exists on both lines and is not in the table.
0111 decides that the description travels with the call, and a description does
not fit in a query string, so the `POST` form is the one this record names.

`POST /Sessions/Playing/Ping` exists on both lines and is deliberately not in the
table. 0057 fixes a cadence of reports rather than a keep-alive, and naming a
call no record asks for is how a surface grows by accident.

Where a path exists in a bare form and again under a `Users/{userId}` prefix, the
bare form is the one above. The prefixed spellings are marked obsolete by the
server on both lines, and the marking is what predicts removal rather than
anybody's judgement:

    $ for r in $A $B; do
    >   git -C jellyfin grep -c -E '^ *\[Obsolete' $r -- Jellyfin.Api/Controllers \
    >     | awk -F: '{s+=$NF} END {print s}'
    > done
    46
    50

## Where the two lines differ inside this surface

One route in the table's own area is on 10.11 and gone from 12.0, and it is an
alias the server had already marked obsolete:

    $ git -C jellyfin show $A:Jellyfin.Api/Controllers/QuickConnectController.cs \
    >   | sed -n '68,76p'
        /// <summary>
        /// Old version of <see cref="InitiateQuickConnect" /> using a GET method.
        /// Still available to avoid breaking compatibility.
        /// </summary>
        /// <returns>The result of <see cref="InitiateQuickConnect" />.</returns>
        [Obsolete("Use POST request instead")]
        [HttpGet("Initiate")]
        [ApiExplorerSettings(IgnoreApi = true)]
        public Task<ActionResult<QuickConnectResult>> InitiateQuickConnectLegacy() => InitiateQuickConnect();

`POST /QuickConnect/Initiate` is on both, so the table names it and nothing in
the core is affected by the removal. What the removal is worth is the evidence
for the paragraph above: on this interface an obsolete alias is a route with a
date on it, and the one that has already been removed between two supported lines
is one of those aliases.

One difference is in what the interface DECLARES rather than in what the server
does, and it would read as a behaviour change to anybody comparing generated
descriptions. On 12.0 `POST /QuickConnect/Initiate` declares a 401 response and
on 10.11 it does not, while both implementations return exactly that:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/QuickConnectController.cs \
    >     | grep -c 'Status401Unauthorized'
    > done
    0
    1
    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/QuickConnectController.cs \
    >     | grep -c 'Unauthorized("Quick connect is disabled")'
    > done
    3
    3

Everything else in the table is identical in route and in response shape across
the two lines. The two response shapes the core reads most were compared field by
field:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:MediaBrowser.Model/System/PublicSystemInfo.cs \
    >     | grep -c 'public .* { get'
    > done
    7
    7
    $ for r in $A $B; do
    >   git -C jellyfin show $r:MediaBrowser.Model/Session/PlaybackProgressInfo.cs \
    >     | grep -c 'public .* { get'
    > done
    21
    21

and the message vocabulary 0116's connection carries is the same set on both:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:MediaBrowser.Model/Session/SessionMessageType.cs \
    >     | grep -cE '^ +[A-Z][A-Za-z]+,?$'
    > done
    34
    34
    $ git -C jellyfin show $A:MediaBrowser.Model/Session/SessionMessageType.cs \
    >   | grep -cE '^ +(LibraryChanged|UserDataChanged),?$'
    2

That connection is authenticated on both lines and is refused without a token,
which is why it is a capability of a session rather than of a server:

    $ for r in $A $B; do
    >   git -C jellyfin grep -c 'Token is required' $r \
    >     -- Emby.Server.Implementations/HttpServer/WebSocketManager.cs
    > done
    1fbd8739292cce610231be93daf43368733edf63:Emby.Server.Implementations/HttpServer/WebSocketManager.cs:1
    c3ed1407ca698b0905de99da87b67415e6a62dbd:Emby.Server.Implementations/HttpServer/WebSocketManager.cs:1

## The two capabilities no supported line offers

`delegated-sign-in` and `token-renewal` are in the set and have no path on either
line. They are named rather than left out, because a capability with no name
cannot be reported and a client asking for it would get silence.

There is no route on either line that exchanges a live token for a fresh one:

    $ for r in $A $B; do
    >   echo "== ${r:0:7}"
    >   git -C jellyfin grep -hE '^ *\[Http' $r -- Jellyfin.Api/Controllers \
    >     | grep -iE 'token|renew|refresh' | sed -E 's/^ +//' | sort -u
    > done
    == 1fbd873
    [HttpPost("{itemId}/Refresh")]
    [HttpPost("Library/Refresh")]
    == c3ed140
    [HttpPost("{itemId}/Refresh")]
    [HttpPost("Library/Refresh")]

Two routes per line carry any of those three words, and both spellings ask the
server to re-scan library metadata. Neither is about a session token. The search
prints what it found rather than counting, because the count is two rather than
zero, and a reader shown a zero here would be reading a pattern that did not match
rather than an absence.

The answer a sign-in produces states nothing about how long the token is good for:

    $ git -C jellyfin show $A:MediaBrowser.Controller/Authentication/AuthenticationResult.cs \
    >   | grep -E 'public .* { get'
        public UserDto User { get; set; }
        public SessionInfoDto SessionInfo { get; set; }
        public string AccessToken { get; set; }
        public string ServerId { get; set; }
    $ git -C jellyfin show $B:MediaBrowser.Controller/Authentication/AuthenticationResult.cs \
    >   | grep -cE 'public .* { get'
    4

Three landed records were written for both answers and this is the answer they
get, so what follows is theirs rather than new.

0005 says a session holds whatever the server said about the token's validity. On
both supported lines that is nothing, so the field is empty rather than absent,
and a client reading it learns that the server made no statement.

0034 schedules a renewal ahead of a stated expiry. There is no stated expiry on
either line, so that schedule never fires and renewal on these two lines is
entirely rejection-driven: the generation number that record fixes is what does
the work, and the timer has nothing to fire against until a line states an expiry.
0034 already decides what happens where there is no renewal route, and that branch
is the whole behaviour here rather than the exception it is written as.

0032 says the core asks the configured server and never guesses whether it
delegates. The server's own interface offers no such route on either line, so the
answer is `capability-absent` before an exchange starts, on every supported
server, and a client offering that route offers something nothing will complete.
Where an operator has installed a plugin that adds one, it is a plugin's
interface, which the plan for this repository puts out of scope, so this record
does not reach it.

## The fallback rule, and why it is one rule

One rule, applied to every path in the table, decided before any call is made.

A 404 on a path this record lists that carries no caller-supplied identifier is
`capability-absent`. A 404 on a path that carries one is `not-found`. A 405, a 410
and a 501 are `capability-absent` on any path, which is 0004's table unchanged.

The reason is that the two kinds of path differ in what can be missing. Nothing
the caller named can be absent from `GET /UserViews` or `POST /Sessions/Logout`,
so the only thing a 404 can be about is the route. On `GET /Items/{itemId}` the
item is a thing that can be gone, the route is present on both supported lines,
and reporting an absent film as an absent capability tells an operator to upgrade
their server over a deleted item.

The split is therefore decided by this table and by nothing in the answer, which
is what 0004 asks for when it says the thing the split depends on is the core's
own list of what the interface holds rather than anything in the response. It
also leaves 0004's other rule intact: a body may add payload and may never change
the kind.

`GET /QuickConnect/Connect` is the one path where a caller-supplied value that is
not an item decides the answer, and it belongs on the `not-found` side. The server
answers 404 there for a secret it does not hold, which is an exchange that expired
or was never started, and that is one of 0031's three endings rather than a
statement about the server:

    $ git -C jellyfin show $B:Jellyfin.Api/Controllers/QuickConnectController.cs \
    >   | sed -n '82,93p'
            try
            {
                return _quickConnect.CheckRequestStatus(secret);
            }
            catch (ResourceNotFoundException)
            {
                return NotFound("Unknown secret");
            }
            catch (AuthenticationException)
            {
                return Unauthorized("Quick connect is disabled");
            }

One rule rather than a rule per endpoint, because a rule per endpoint is a table
of exceptions that is read once when it is written and never again, and the first
endpoint somebody adds afterwards gets whichever row was nearest.

## What a poll on a route that was turned off costs, stated rather than repaired

The same code above answers 401 where the operator turned quick connect off while
an exchange was in flight. 0004's table maps 401 to `not-authenticated` and
forbids the body from changing the kind, so a client polling an exchange on a
server whose operator has just turned the route off is told that it is not
authenticated, which reads as a dead token.

`GET /QuickConnect/Enabled` is what makes that rare rather than ordinary, since
0031 already requires the read before a person is shown a code. It does not make
it impossible, because the setting can move between the read and the poll.

This record states the cost and does not repair it. The repair would be a rule
letting something other than the status decide a kind, which is a change to 0004
and belongs in a record that supersedes 0004 rather than in an edit here.

## What is read before a call, and what is not

Nothing in the table is probed to find out whether it may be called. A call is
made, and its own answer says whether the capability is there. That is the whole
rule.

Two reads happen before a call and neither is a probe of this surface.

`GET /System/Info/Public` is read once per server, unauthenticated, and both lines
answer it without a token. 0041 needs the identifier out of it for the server part
of a cache key, and 0028 is already at that address. Its `Version` field is
recorded and reported under 0071 and is never used to decide whether a call may be
made.

`GET /QuickConnect/Enabled` is read before an exchange is started, because 0031
requires it: a person shown a code that nothing will ever approve is the failure
that record exists against. It reads an operator's setting rather than a version,
which is why this table cannot replace it.

Per call rather than per session, for two reasons that both cost something in the
other direction. A probe per session is a round trip spent before anything is
shown, on the cold start 0046 is already trying to fill, and it answers a question
that on these two lines has one answer for every capability but two. And a version
string is not an oracle for this interface: a route can be present and marked
obsolete, as the prefixed spellings above are, and a self-hosted build
carries whatever version its packager wrote.

WHAT THIS DOES NOT DO IS MAKE 0004's RESERVATION COME TRUE, and the distinction is
worth stating because the words are close. That record is superseded if the 404
split turns out not to be decidable from the core's own capability list, which it
says is possible if this issue answers that capability is probed per call. The
split above is decided from the list, before any call, by the shape of the path.
What is per call is which kind a call ENDS in, which is a different sentence, and
0004's condition is not met by it.

## What this record does not decide

Which endpoints an operator's plugins add. The plan for this repository puts a
plugin's interface out of scope and this record keeps it there.

Whether the core should hold a second surface for a server line older than 10.11.
Entry 3 of #1 named two lines and this record is written against those two.

What the query parameters on `GET /Items` are, which fields a first screen asks
for, and what an artwork address carries. Those are #39 and #49, and putting them
here would decide two issues inside a third.

What the vocabulary of the description in `device-capabilities` is. 0036 gives the
core the shape and the client fills it, and 0111 names the case where a playback
call carries its own.

Whether a write may be replayed after the server has already acted on it. The
`repeat` column is the input to that; #47 is where it is decided. Neither
supported line offers an identity a caller can attach to a write so that the
server can refuse a repeat, which is the promise 0047 says would be needed and
does not have:

    $ for r in $A $B; do
    >   git -C jellyfin grep -ci 'idempot' $r -- Jellyfin.Api \
    >     | awk -F: '{s+=$NF} END {print s+0}'
    > done
    0
    0

Nothing here is a statement about what a running server returns. Every line above
was read out of the server's source at two commits, and no request was made to any
server. The comparison against a real server is #104 and is not this record's.

## Why this is written down before the code

Without it the surface is discovered by the first caller that needs an endpoint,
and the list of what the core depends on then exists only as the set of string
literals scattered through whatever code grew first. Half of that has already
happened on this board in the other direction: 0004 wrote a 404 split whose
deciding input is a list that did not exist, 0032 and 0034 were written for both
answers of a question nobody had asked the server, and 0041 named a fallback for a
server identity that turns out to be present on every supported line. Four records
were each carrying a conditional, and the condition was one file away from being
read.

The specific failure the absence produces is an operator on a supported server
line being told that a film is not found when the route is gone, or being told to
upgrade when their film is gone, because whichever caller wrote the first 404
branch decided the split for the whole core. That is 0004's own sentence about the
report that is hardest to act on, and this list is what stops it being decided by
accident.

The second failure is quieter and is why the two absent capabilities are named
rather than omitted. A client that offers a delegated sign-in on a server whose
interface has no such route sends a person to a control that does nothing, and
without a name in the set the core has no way to say why.

## Alternatives, and what each cost

**A capability probe once per session.** The core asks the server what it offers,
once, and every call afterwards is decided from the answer. It costs a round trip
on the cold start 0046 is trying to fill, and it needs the server to offer such an
answer. Neither line does: `GET /System/Info/Public` reports a version and an
identifier and no capability list, and the only per-capability read on either line
is `GET /QuickConnect/Enabled`, which reports an operator's setting. Building it
would mean deriving capability from the version string, which is the next
alternative.

**Capability derived from the version the server reports.** Cheap, one field, no
extra call. It is wrong in both directions on this interface. A route can be
present and obsolete, which is a removal with a date on it that no version
comparison sees until the removal lands, and the one route that has already been
removed between the two supported lines was in exactly that state. In the other
direction a self-hosted build carries whatever version its packager wrote, so the
core would refuse a call that would have worked. It also puts a version table into
the core that has to be edited for every server release.

**A rule per endpoint for what an absence means.** More precise than one rule, and
it is the shape that decays: a table of exceptions is read once when it is written,
and the endpoint somebody adds in a year gets the row that was nearest. The rule
above is one sentence about the shape of a path, which somebody applying it to a
new endpoint cannot get wrong by not having read this record.

**Naming no capabilities and reporting a bare 404.** No set to keep, nothing to
name in a payload, and one fewer thing for a client to switch on. It gives up the
report an operator can act on, which is the whole reason 0004 split the row, and it
makes the two absent capabilities unreportable rather than absent.

**Waiting for a real server rather than reading the source.** Stronger evidence for
what a server returns, and it needs a server on each line, which nothing on this
board has yet. It also answers a different question: what one operator's deployment
does today, rather than what the two lines carry. #104 is where the comparison
against a real server lives, and it is written to run against this record rather
than instead of it.

## What would reverse this

A third server line is added to what the core supports, or either of the two named
here moves far enough that the comparison above no longer reproduces. The condition
is checkable: re-run the route comparison in the first section at the current tips
of both lines, and a difference inside the sixteen capabilities is this record
being out of date rather than a server being wrong.

A supported line gains a route that exchanges a live token for a fresh one, or
states a token's validity in the sign-in answer. Either one turns 0034's schedule
from something with nothing to fire against into the ordinary path, and the two
records move together.

A supported line offers a capability answer of its own, so that one read replaces
the per-call rule for more than the single setting `GET /QuickConnect/Enabled`
carries. The first alternative above then becomes cheaper than what was chosen,
and this record is superseded rather than amended.

The 404 split is measured producing the wrong answer against a real server, on the
diagnostic events from #100 rather than in argument. The shape of a path is a
proxy for what can be missing, and a route answering 404 for a resource on a path
with no identifier in it would be that proxy failing.

A client is found reporting `not-authenticated` to a person because an operator
turned quick connect off mid-exchange, twice. One is the cost this record states.
Two is evidence that the read before the exchange does not narrow it enough, and
what follows is a record that supersedes 0004 rather than an edit here.
