# 0004. The error vocabulary every client shares

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #4

## The decision

Every failure the core reports is exactly one of a closed set of fifteen kinds,
each answering three questions a client can ask without knowing what went wrong
underneath, each carrying a stated payload and nothing else, and none of them
carrying a sentence written for a person; a transport failure, an HTTP status and
a server-supplied error body are all mapped onto that set by rules written here,
and a shape none of the rules recognise becomes the named kind
`answer-not-understood` rather than escaping as something else.

## The vocabulary

Three questions are asked of every kind. Retry says whether repeating the same
call can produce a different answer without anything else changing. Stable says
whether the same call repeated now gives the same answer, which is what a client
needs in order to decide whether to cache the failure or to hide it. Means is
what the condition is about, in the words of the situation rather than of the
socket.

| Kind | Retry | Stable | Means |
| --- | --- | --- | --- |
| `address-not-usable` | no | yes | What was typed as a server address cannot be turned into somewhere to send a request. |
| `server-unreachable` | yes, after a delay | no | Nothing at that address answered. The network, the name, or the machine. |
| `timed-out` | yes, after a delay | no | Something answered too slowly, or stopped answering part way. |
| `certificate-rejected` | no, until a person decides | yes | The machine that answered did not prove it is the one the address named. |
| `not-authenticated` | no, until a session changes | yes | There is no session the server accepts. It expired, it was ended, or it was never valid. |
| `not-permitted` | no | yes | The session is valid and this account may not do this. |
| `not-found` | no | yes | The thing asked for is not on that server. |
| `request-refused` | no | yes | The server understood the request and rejected it as wrong. |
| `server-busy` | yes, after the hint | no | The server is refusing load, not this request. |
| `server-failed` | yes, after a delay | no | The server broke while handling a request that was not wrong. |
| `answer-not-understood` | no | yes | Something arrived and it is not a shape the core knows how to read. |
| `capability-absent` | no | yes | This server does not offer the part of its interface this call needs. |
| `cancelled` | caller's choice | no | The caller asked for this to stop, and it stopped. |
| `storage-unavailable` | yes, after a delay | no | A store the client supplied could not be read or written. |
| `internal-fault` | no | yes | A defect in the core. Nothing about the server or the network is being claimed. |

Fifteen, and the set is closed. A client handles it exhaustively, and a
sixteenth kind is a change to this record and to every client, which is the cost
that keeps the set from growing by accident.

## What each kind carries

A payload is what a client cannot recover on its own and would otherwise have to
parse out of something. Everything else is left out, because a field nobody
needs is a field somebody displays.

`address-not-usable` carries the address as it was given, unmodified, and which
part of it could not be used.

`server-unreachable` carries the address that was contacted and whether any
bytes reached the server before it failed. The second half is what separates a
call that certainly did not happen from one that may have.

`timed-out` carries which deadline expired, the elapsed time it expired after,
and the same did-anything-reach-the-server flag. Which clock that elapsed time
came from is #102.

`certificate-rejected` carries the address, the reason class, and the presented
certificate's fingerprint. The fingerprint is what a person compares against
their own server. What the core does about the self-signed case is #29.

`not-authenticated` carries whether a token was presented and rejected, or
whether there was none to present. #34 and #35 act on that difference.

`not-permitted` carries nothing.

`not-found` carries the identifier that was asked for.

`request-refused` carries the server-supplied error code where the server gave
one, as an opaque string, and nothing more.

`server-busy` carries the retry-after hint where the server gave one, as a
duration, and a flag saying whether it was given or assumed.

`server-failed` carries the HTTP status and the server-supplied error code where
there is one, both opaque.

`answer-not-understood` carries what the core was reading, what it expected, and
where in the answer it stopped. It never carries the answer itself, because an
answer holds library contents and may hold a token.

`capability-absent` carries the name of the capability, from the set #10 fixes.

`cancelled` carries nothing.

`storage-unavailable` carries which store it was, the cache store from #40 or the
secret store from #33, and whether the failure was a read or a write.

`internal-fault` carries a stable identifier for the site that produced it, and
nothing derived from the data being handled.

No payload field anywhere holds a session token. That is the rule in #5 applied
here, and the same rule keeps a token out of `answer-not-understood`.

## How a real failure becomes one of the fifteen

Three sources, three rules, and a fourth rule for everything the first three do
not catch.

A transport failure is classified before any HTTP exists. A name that does not
resolve, a refused connection, an unreachable network and a connection dropped
mid-body are `server-unreachable`. A deadline reached with no answer, or an
answer that stalls mid-body, is `timed-out`. A refusal to trust the peer is
`certificate-rejected`. An address that never produced a connection attempt
because it could not be parsed is `address-not-usable`.

An HTTP status decides the kind, by this table, and the body may not override it.

| Status | Kind |
| --- | --- |
| 401 | `not-authenticated` |
| 403 | `not-permitted` |
| 404 on a resource the interface says should exist | `capability-absent` |
| 404 otherwise | `not-found` |
| 405, 410, 501 | `capability-absent` |
| 429 | `server-busy` |
| 503 | `server-busy` |
| any other 4xx | `request-refused` |
| any other 5xx | `server-failed` |
| 1xx or 3xx surfacing to the caller | `answer-not-understood` |
| 2xx whose body does not parse | `answer-not-understood` |

The two 404 rows are the only place the kind depends on something other than the
status, and the thing it depends on is the core's own list of what the server
interface holds rather than anything in the response. #10 owns that list. Without
the split, an operator on an older server sees "not found" for a library that
exists, which is the report that is hardest to act on.

A server-supplied error body may add payload and may never change the kind.
Bodies differ between server versions and between whatever sits in front of the
server, and a proxy returning a themed error page for a 503 would otherwise turn
a retryable condition into an unrecognised one. The status is the part of the
answer that is defined by a protocol rather than by a deployment.

Anything the three rules above do not produce a kind for is
`answer-not-understood`. This is the rule that keeps the set closed. A status
outside every row, a body that parses but omits a field the core needs, a field
holding a value outside the set the core knows, an artwork payload in a format
#55 does not accept: all of them arrive as the same named kind, with a payload
saying what was being read and where reading stopped. There is no default branch
anywhere that produces something else, and #37 is where that absence is proven
rather than asserted.

## The core writes no sentences

Nothing the core produces is written for a person to read. Not error text, not a
"reason" string, not a summary field a client might be tempted to show. The
reason is in #3 and does not need repeating beyond its consequence: a sentence
has a language, a length that fits a particular screen, and a decision about
whether to name the server at all, and the core knows none of the three.

What a client gets instead is the kind, which is a stable identifier from a set
of fifteen, and the payload fields listed above, which are data rather than
prose. A client writes fifteen sentences once and can prove it wrote all of them,
because the set is closed and exhaustive matching is a thing a compiler can
check. Fifteen sentences per client is cheaper than a core that ships strings it
cannot translate, cannot shorten, and cannot stop a client from showing to a
person.

The opaque strings in the payloads are the one place this needs care. A
server-supplied error code is passed through as an opaque identifier precisely so
that a client can match on it, and it is not a sentence even where the server
sent one. Where the server sends prose, it is dropped rather than forwarded,
because a forwarded sentence is a sentence some client will display, in whatever
language the server chose, about a server the person may not have known they were
talking to.

Diagnostics are the other route out, and they are not an exception. A diagnostic
event carries identities and fields under #100, goes to the client's own sink,
and is never the thing a person is shown.

## Why this is written down before the code

Eleven clients naming errors eleven ways is the failure, and it does not arrive
as a disagreement. It arrives as one person seeing "cannot connect" on a
television and "check your network settings" on a phone for a server that
returned 503, and reporting a bug against whichever they saw second.

The part that cannot be repaired later is the closure. A vocabulary that starts
open, with a catch-all for the unrecognised, never closes afterwards, because by
then every client has a branch that handles the catch-all and each of them
handles it differently. Closing it now costs a table. Closing it after two
clients ship costs a change to both, and the change is invisible in a compiler
that already accepted a default branch.

The second unrepairable part is the retry property. A client that guesses which
failures are worth retrying will retry `not-permitted` and give up on
`server-busy`, and both mistakes look like the server misbehaving. Once that
guessing is spread across eleven clients, correcting it means finding it eleven
times.

## Alternatives, and what each cost

An open set with a catch-all kind. Cheap today, and every unrecognised condition
has somewhere to go without a decision. It costs exactly the property this record
is for: with a catch-all, nothing forces a new condition to be named, so the
catch-all becomes the common case and each client invents its own handling of it.
The closed set makes a new condition a visible change.

Passing the HTTP status through and letting each client decide. Honest about
where the information came from, and no mapping to maintain. It puts the mapping
in eleven places instead of one, which is the duplication this repository exists
to remove, and it leaks a transport detail into an interface that also has to
describe failures with no HTTP in them at all, such as `storage-unavailable`.

Kinds carrying a message string for the client to show. Immediately useful and it
is what most libraries do. It costs the boundary in #3, and it costs it in the
direction that cannot be walked back: the moment a string is available, a client
displays it, and after that the string's wording is an interface with eleven
consumers and no translations.

A deep hierarchy, with kinds nesting under kinds. More expressive, and a client
can handle at whichever depth it likes. It costs exhaustiveness, because the
compiler check that makes the closed set worth having works on a flat set, and
handling at a parent level is how a client silently stops distinguishing the
cases this record separated on purpose.

Letting a server-supplied error body choose the kind. It uses the most specific
information available, and where the server is a current Jellyfin it would be
right more often. It costs the deployment: bodies vary between versions and
anything in front of the server can replace them, and it hands the choice of kind
to the least trustworthy part of the answer.

## What would reverse this

A condition appears that genuinely fits none of the fifteen and is not a defect,
twice. One is a mapping mistake in this record. Two is a set that is the wrong
size, and it is superseded by a set that covers them, with the cost of changing
every client paid deliberately.

`answer-not-understood` becomes the kind an operator sees most, measured on the
diagnostic events from #100 rather than assumed. That means the mapping rules are
missing a case that happens in the field, and the record is superseded by one
that names it.

The 404 split in the table above turns out not to be decidable from the core's
own capability list, which is possible if #10 answers that capability is probed
per call rather than per session. The split is then wrong rather than merely
awkward, and this record is superseded by one that places the distinction where
the answer actually is.

A client is found handling the vocabulary non-exhaustively without the compiler
having refused it, which is a property of whatever #1 entry 2 chooses as the
language. If the chosen means cannot refuse a missing case, the closed set is a
convention rather than a guarantee, and this record is superseded by one that
says so plainly instead of claiming a property nothing enforces.
