# 0272. The route next up is read from

Date: 2026-09-01

Status: accepted. Supersedes 0010. Superseded by nothing.

Issue: #272

## The decision

The core reaches a Jellyfin server through the seventeen named capabilities
enumerated below and through no other part of its interface; a capability name is
what `capability-absent` carries in
[0004](0004-the-error-vocabulary.md), a 404 means an absent route on a path this
record lists that carries no caller-supplied identifier and a missing resource on
one that does, and the core establishes no capability by probing except where a
landed record already required a read before the call for a reason of its own.

The seventeenth is `next-up`, read from `GET /Shows/NextUp`.

## What this record changes, and what it does not

One row is added to
[0010](0010-the-server-surface-and-what-an-absence-does.md)'s table. Every other
row, the fallback rule, the two capabilities no supported line offers, what is
read before a call, and what that record does not decide are carried unchanged.

**The whole table is carried here because that is the only shape supersession
has.** [0001](0001-decision-records.md) offers a record that is added or
superseded and three edits that are neither, and a partial supersession is not
among them. A row added to
[0010](0010-the-server-surface-and-what-an-absence-does.md)'s own text would be an
edit of the kind that record forbids, and a record naming only the new row would
leave the other sixteen capabilities with no live authority. #267 is where the
shape of a partial supersession is asked for and this record does not invent one.

**THE READINGS BEHIND THE SIXTEEN CARRIED ROWS WERE NOT RETAKEN HERE.** They were
taken in [0010](0010-the-server-surface-and-what-an-absence-does.md) against the
two commits that record names, that record keeps its text, and a reader who wants
the derivation of a carried row reads it there. What is retaken below is what this
record asserts on its own account: that the two commits are the two lines they are
said to be, and everything about the route being added. A reader who takes the
carried rows for readings made today is reading this record for something it does
not claim.

## What was read, and where

The two lines are the ones entry 3 of #1 answered with, and this record is written
against the same two commits
[0010](0010-the-server-surface-and-what-an-absence-does.md) names, so that the row
added below sits beside the sixteen carried ones rather than beside a different
pair of tips:

    $ A=1fbd8739292cce610231be93daf43368733edf63    # the 10.11 line
    $ B=c3ed1407ca698b0905de99da87b67415e6a62dbd    # the 12.0 line
    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/SharedVersion.cs?ref=$r" --jq '.content' \
    >     | base64 -d | grep AssemblyVersion | head -1
    > done
    [assembly: AssemblyVersion("10.11.11")]
    [assembly: AssemblyVersion("12.0.0")]

The route reached through the server's own repository over the network rather than
through a clone, which is a second way to the same two commits rather than a
second reading: a commit identifier names one tree, and both routes are asking for
that tree by name.

## The seventeen capabilities, and the paths that carry them

Every path is joined to the base address
[0028](0028-the-address-a-person-typed.md) resolved, by the rule that record
already fixes. Every authenticated call carries the four parts
[0036](0036-the-device-identity-and-who-supplies-it.md) names in the authorization
value.

The column headed `repeat` says what a second identical call leaves behind: an
`assertion` leaves the server in the state the first call intended, and an
`accumulation` leaves something the first one did not. It is here because
[0038](0038-retry-and-backoff.md) says in so many words that which calls change
server state is this record's and not that one's.
[0038](0038-retry-and-backoff.md) retries by error kind and never reads this
column; what reads it is #47, deciding what a queue may replay.


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
| `next-up` | `GET /Shows/NextUp` | The episode a series is continued at, which is a first screen's second shelf. 0039's page and 0039's item type. | assertion |
| `item-user-data` | `GET /UserItems/{itemId}/UserData` | The position and the watched mark, read. 0056 and 0060. | assertion |
| | `POST /UserItems/{itemId}/UserData` | The same, written. | assertion |
| `artwork` | `GET /Items/{itemId}/Images/{imageType}` | The bytes #49 builds an address for. `HEAD` on the same path answers the same. | assertion |
| `playback-selection` | `POST /Items/{itemId}/PlaybackInfo` | 0111's choice of source, carrying 0036's description in the body. | accumulation |
| `playback-progress` | `POST /Sessions/Playing` | Playback started. | accumulation |
| | `POST /Sessions/Playing/Progress` | 0057's cadence. | assertion |
| | `POST /Sessions/Playing/Stopped` | Playback ended. | accumulation |
| `played-marking` | `POST /UserPlayedItems/{itemId}` | 0060's mark, set. | assertion |

`POST /Items/{itemId}/PlaybackInfo` is an accumulation rather than an assertion
because its body carries a field that makes the call open a live stream, so a
repeat can open a second one, which is
[0010](0010-the-server-surface-and-what-an-absence-does.md)'s reading and is not
retaken here. The `GET` form of the same path, `POST /Sessions/Playing/Ping`, and
the spellings the server marks obsolete under a `Users/{userId}` prefix are out of
the table for the reasons that record gives.

## The row this record adds

### The route exists on both lines and answers with the type 0039 already fixes

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Jellyfin.Api/Controllers/TvShowsController.cs?ref=$r" \
    >     --jq '.content' | base64 -d | grep -n -A2 'HttpGet("NextUp")' | grep 'HttpGet\|ActionResult'
    > done
    76:    [HttpGet("NextUp")]
    78-    public ActionResult<QueryResult<BaseItemDto>> GetNextUp(
    75:    [HttpGet("NextUp")]
    77-    public ActionResult<QueryResult<BaseItemDto>> GetNextUp(

That answers the question #272 asks last, and the answer is the ordinary one:
next up is the item type
[0039](0039-the-page-the-item-and-what-next-up-is-not.md) fixes, in the page that
record fixes, and not a fourth shape. The envelope is
`QueryResult<BaseItemDto>` - the same one `GET /Items` and `GET /UserItems/Resume`
answer with - so the core's promise that a client written against one read can
display the result of another survives this row untouched, because nothing new is
being mapped.

The two paging parameters and the total flag are on both lines:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Jellyfin.Api/Controllers/TvShowsController.cs?ref=$r" \
    >     --jq '.content' | base64 -d | sed -n '/HttpGet("NextUp")/,/^    {/p' \
    >     | grep -cE 'FromQuery\] int\? startIndex|FromQuery\] int\? limit|FromQuery\] bool enableTotalRecordCount = true'
    > done
    3
    3

so this is a paged library read like the other three, and
[0039](0039-the-page-the-item-and-what-next-up-is-not.md)'s offset, its total that
may be a page length, and its rule that the core always asks for the total apply
to it with nothing added.

The route is under a base of its own and carries the authorization every
authenticated call in the table carries, so it is a capability of a session rather
than of a server. `GET /System/Info/Public` is the one row that is not, and it is
not this one:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Jellyfin.Api/Controllers/TvShowsController.cs?ref=$r" \
    >     --jq '.content' | base64 -d | grep -nE '^\[Route|^\[Authorize'
    > done
    29:[Route("Shows")]
    30:[Authorize]
    28:[Route("Shows")]
    29:[Authorize]

### `assertion`, for the reason every other read in the table carries it

A second identical `GET /Shows/NextUp` leaves the server in the state the first one
intended. Nothing in it is written, and #47 may replay it.

### The absence semantics are the ones the fallback rule already gives

`GET /Shows/NextUp` carries no caller-supplied identifier in its path, so a 404 on
it is `capability-absent` and never `not-found`, by the same sentence that decides
`GET /UserViews` and `POST /Sessions/Logout`. Nothing in this row is an exception
to that rule, and the row is in the table rather than beside it for exactly that
reason.

## What a first screen loses without the row

The kickoff names next up on the first screen a television shows, and
[0039](0039-the-page-the-item-and-what-next-up-is-not.md) took it out of #39's
scope rather than out of the plan. Without a row, a client that wants the shelf
has three moves and each one costs something this board has already paid to avoid.

It reaches the route itself, outside the core. The path then exists in a client
rather than in the enumeration, the core's list of what it depends on is wrong by
one, and #70's test - which fails when the core reaches a host nobody configured -
is judging a smaller surface than the client actually uses.

It draws the shelf out of `GET /UserItems/Resume`. That is the resume list, which
is a different question: resume answers what was left part-watched, and next up
answers what a series is continued at, including a series whose last episode was
finished. On both supported lines the finished-series case is the route's ordinary
answer and the resume list has nothing to say about it.

It draws no shelf. That is a product decision the kickoff already took the other
way, and #272 says in its own words that what is open here is the route and its
cost rather than the feature.

## What the route costs against both supported lines

**One request, on the cold start #62 is measuring.** It is a second paged read
beside the resume list on the first screen, and
[0009](0009-the-concurrency-model.md) already lets the two be in flight together,
so the cost is a request rather than a round trip added to the critical path. That
is a statement about the shape of the calls and not a measurement: nothing in this
tree makes a request, and what the shelf costs in milliseconds is #62's to measure
rather than this record's to assert.

**One parameter the core must send, and this is the part that is easy to get
wrong.** On both lines the route includes resumable episodes unless it is told not
to:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Jellyfin.Api/Controllers/TvShowsController.cs?ref=$r" \
    >     --jq '.content' | base64 -d | sed -n '/HttpGet("NextUp")/,/^    {/p' \
    >     | grep -E 'enableResumable|enableRewatching' | sed -E 's/^ +//'
    >   echo "--"
    > done
    [FromQuery] bool enableResumable = true,
    [FromQuery] bool enableRewatching = false)
    --
    [FromQuery] bool enableResumable = true,
    [FromQuery] bool enableRewatching = false)
    --

A core that sends nothing therefore asks for a set that overlaps the resume list,
and a first screen drawing both shelves shows the same episode in both of them.
This record states that cost and does not fix the parameter. What a call asks for
is #39's, by the same sentence
[0010](0010-the-server-surface-and-what-an-absence-does.md) already applies to
`GET /Items`, and fixing it here would decide one issue inside another. What is
this record's is that the cost exists and is a default rather than a server
behaviour: a caller who sends nothing gets the duplicate, so the value has to be
chosen rather than left to whoever writes the first request.

**One parameter the two lines disagree about, and it is the harmless direction.**
10.11 carries a parameter the server has already marked obsolete and 12.0 has
removed it:

    $ for r in $A $B; do
    >   gh api "repos/jellyfin/jellyfin/contents/Jellyfin.Api/Controllers/TvShowsController.cs?ref=$r" \
    >     --jq '.content' | base64 -d | sed -n '/HttpGet("NextUp")/,/^    {/p' \
    >     | grep -n 'disableFirstEpisode' | sed -E 's/^ +//'
    >   echo "--"
    > done
    16:        [FromQuery][ParameterObsolete] bool disableFirstEpisode = false,
    --
    --

The core sends neither spelling, so the removal reaches nothing here. It is
recorded because it is the second instance of the pattern
[0010](0010-the-server-surface-and-what-an-absence-does.md) already names once: on
this interface a thing marked obsolete is a thing with a date on it, and the marked
thing is what disappears between two supported lines. A core that had sent it would
have been sending a parameter one supported line ignores.

## What an absence means for a client that has already drawn the shelf

The core reports `capability-absent` carrying the name `next-up`, on the call,
because nothing in this table is probed and the answer to a call is what says
whether the capability is there. A client that reserved a rectangle for the shelf
has therefore already reserved it when it learns, and that is the cost rather than
a defect: the alternative is a probe per session, which
[0010](0010-the-server-surface-and-what-an-absence-does.md) priced against the
cold start and refused for every capability rather than for this one.

What the name buys is the difference between an empty shelf and an unexplained
one. A client that gets a named capability back can say why the shelf has nothing
in it and can stop drawing it on the next start; a client that gets a bare 404
learns that something was not found and has to guess whether the server has no
such route or the person has nothing to continue. That is
[0004](0004-the-error-vocabulary.md)'s own reason for splitting the row, applied
to the row this record adds.

**Neither supported line is that case**, and the sentence above is written for a
case that does not arise on 10.11 or 12.0. The route is on both, read above at both
commits. What it is written for is a self-hosted build that removed it and a third
line nobody has read yet, and it is written now because the alternative is the
first caller deciding it at a call site.

An empty answer is not an absence and the two must not be collapsed. A person with
no series in progress gets a page whose items are empty and whose total is zero,
which is a successful read, and a shelf drawn for it is a shelf with nothing in it
rather than a capability that is gone.

## What this record does not decide

Everything [0010](0010-the-server-surface-and-what-an-absence-does.md) leaves
open, unchanged: which endpoints an operator's plugins add, whether the core holds
a surface for a line older than 10.11, what the query parameters on `GET /Items`
are, what the vocabulary of the description in `device-capabilities` is, and
whether a write may be replayed.

Whether the shelf is drawn at all, and where. The kickoff named next up and #272
says the feature is not what is open here.
[0003](0003-what-the-core-does-not-do.md) puts the drawing outside in any case.

Which fields the call asks for, what the page size is, what value
`enableResumable` is sent with, and whether the shelf is fetched at the same moment
as the resume list. Those are #39's and #62's, and deciding them here would decide
two issues inside a third, which is the sentence
[0010](0010-the-server-surface-and-what-an-absence-does.md) already applies to the
same pair. The overlap the parameter produces is stated above as a cost of the
route, which is this record's, and the value is not chosen here.

Nothing here is a statement about what a running server returns. Every line above
was read out of the server's source at two commits, and no request was made to any
server. The comparison against a real server is #104.

## Why this is written down before the code

Nothing in this tree makes a request, so this is the last moment the route is a
decision rather than a discovery. #39 met it as a discovery already: its body asked
for resume and next up together, the resume row was in the table and the next-up
row was not, and the record it produced had to refuse half its own scope and say so.
That is the cheap version of the failure, caught in a record. The expensive version
is a call site: whoever writes the first next-up request writes a path that is in
no enumeration, and #70's test then passes over a host and a route the core really
reaches, which is a check that has stopped being about the thing it names.

The parameter is the second half and it is the half that would not have been caught
at all. `enableResumable` defaults to including what the resume list already
returns, so a first caller who sends nothing gets a working shelf with a duplicate
in it, on every server, and the duplicate reads as a server behaviour rather than
as a default nobody chose.

## Alternatives, and what each cost

**Leaving the route out and drawing the shelf from `GET /UserItems/Resume`.** No
row, no new call, and one fewer thing on the cold start. It answers a different
question: a series whose last episode was watched to the end is not in the resume
list and is exactly what next up is for. It also quietly makes the core's answer
depend on what a client asked for rather than on what it is called.

**A row that names the route and fixes `enableResumable` here as well.** One
record instead of two, and the duplicate above closed in the same change. It
decides what a call asks for, which
[0010](0010-the-server-surface-and-what-an-absence-does.md) places with #39 for
every other row in this table, and a surface record that starts choosing query
parameters for one row is the shape the next row inherits.

**Adding the row to
[0010](0010-the-server-surface-and-what-an-absence-does.md)'s own text.** One line
changed, no reproduction of a table, nothing to keep in step. It is the edit
[0001](0001-decision-records.md) forbids: a reader afterwards finds a record that
was always right about a route it did not carry when #39 needed it, and the
refusal that record produced reads as a mistake rather than as the state of the
surface at the time.

**Waiting for #267 to decide the shape of a partial supersession**, so that the
sixteen carried rows do not have to be reproduced. It is the cheaper shape and it
does not exist yet; taking it now would mean inventing it, which
[0243](0243-the-means-a-certificate-is-validated-with.md) declined to do for the
same reason on the same open question. What this record pays instead is a
reproduced table and a paragraph saying which readings it did not retake, and
#267 landing is what retires that cost rather than reversing this decision.

## What would reverse this

The route stops answering on a supported line, or its answer stops being
`QueryResult<BaseItemDto>`. The core then owes a mapping rather than a row, which
is the condition
[0039](0039-the-page-the-item-and-what-next-up-is-not.md) already states for the
other three reads, and the two records move together.

`enableResumable` changes its default on a supported line, or the resume list
begins answering the finished-series case. The overlap this record states as a cost
is then gone, and #39 is deciding a parameter against a reading that no longer
reproduces. It is checkable the same way it was read: at the two commits a
superseding record names.

A third server line is added, or either of the two named here moves far enough that
the readings above no longer reproduce. That is
[0010](0010-the-server-surface-and-what-an-absence-does.md)'s own reversal
condition carried forward and it now covers seventeen rows rather than sixteen.

#267 decides a shape that lets a record add a row without carrying the whole table.
This record is then the instance to re-shape, and what changes is how the addition
is written rather than what was decided about the route.
