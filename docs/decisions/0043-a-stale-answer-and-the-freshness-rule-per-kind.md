# 0043. A stale answer, and the freshness rule per kind of entry

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #43

## The decision

Every cache read answers with the value, one of the three states 0006 fixes, and
the age that produced the state, the age being the quantity 0102 anchors on the
server rather than on any device clock; the age is read against one threshold per
kind of entry from the table below rather than per call site; a server may shorten
a threshold and may never lengthen one; age alone never withholds an entry, only
marks it; and a caller that demands freshness gets a fresh answer or the
transport's own named failure and never a stale one.

## What a read answers with

Three states, from 0006, and this record adds what each carries.

`fresh` carries the value and its age. The age is carried even when the entry is
fresh, because a client showing "updated a minute ago" needs it in the case where
nothing is wrong, and an interface that supplies it only on the unhappy path
teaches every client to ask twice.

`stale` carries the value and its age. Nothing else. There is no reason field
saying which threshold it passed, because a threshold is the core's and a client
that acted on it would be reimplementing this table.

`absent` carries nothing. Absent means the cache holds nothing usable under that
key, and it is the answer for an entry that was never written, one that eviction
removed under #42, and one that explicit invalidation removed.

An invalidated entry is absent rather than stale, and that is a rule rather than
an implementation detail. Invalidation happens because the core knows the answer
changed, so what is held is wrong rather than old, and handing it back marked
`stale` would let a client show it. Age marks; invalidation removes. The three
states stay three because of that split.

Reading the age of an entry the core has already loaded is a call that cannot wait
under 0009, which is what lets a client decide what to draw without a round trip
through its own async machinery.

## What the age is

0102 fixes it and this record does not restate the mechanism: two moments are
stored at write, the age at read is corrected by the difference between the skew
now and the skew at write, the correction is unavailable offline, and both a
negative age and an age past a sanity bound are treated as past every threshold.

What this record adds is that those two guards are the only route by which an
entry becomes stale without a threshold being reached, and that the direction is
deliberate. Both fail towards asking the server. A device that came up believing
it is 1970 shows its whole cache as stale and refetches, which costs round trips
on one start. The opposite failure costs somebody seeing a film they deleted last
week, with nothing in the system that will ever correct it.

The age is not a claim that the value was correct at that moment. It is how long
ago the server sent it. A server that was wrong then is still wrong now, which
0101 already says plainly about a server that lies about its own contents.

## The thresholds

None of these numbers is a measurement. They are chosen, with the reasoning
beside each one, and this is stated first because a number in a table is read as
measured unless it says otherwise. What would produce measured replacements is
named at the end of this record.

| Kind | Stale after | Why this one |
| --- | --- | --- |
| Library query results | 5 minutes | The answer changes when anybody adds a file, and the person most likely to notice a stale list is the one who just added something. Five minutes is short enough that a person who adds a film and goes back to the list sees it after one ordinary navigation, and long enough that scrolling a wall of tiles does not refetch the list behind it. |
| Item metadata | 1 hour | It changes when a metadata provider is re-run or somebody edits a field, which is a deliberate act rather than a background one, and it is read far more often than it changes. |
| Capability answers for a server | 24 hours | 0006 gives this kind a long age. What changes it is a server upgrade, and a wrong answer here is a `capability-absent` on a call the server would now accept, which is recoverable on the next day rather than damaging. |
| Artwork bytes | 30 days | 0006 makes a changed image a different key rather than a stale entry, because the address is content-tagged. The age is therefore not tracking change at all; it bounds the case where a server reuses a tag for different bytes, which is a server defect rather than a normal event. |
| Decoded dimensions | same as their bytes | 0006 already fixes this, and the reason is that dimensions describing bytes the cache no longer holds are the shape that makes a layout reserve the wrong space under #52. |

Five kinds, which is the set 0006 lists, and a sixth kind of entry is a change to
that record and to this one rather than a row somebody adds.

Playback positions recorded while the server was gone are not in this table and
are not cache entries. They are the queue in #47, they are a person's action
rather than a copy of a server's answer, and nothing about them goes stale with
age.

## What a server may say about a threshold

A server may shorten one and may never lengthen one.

Where a response carries the server's own statement that it may be kept for less
time than the table allows, that shorter time is used. The server knows things the
core does not, most obviously that a library is being actively written to.

Where it states a longer time, the table wins. The server knows nothing about the
device, and the thresholds above are about a person in front of a screen rather
than about how often the server's data changes. A server that could lengthen them
could make a device hold a library list for a week, and the operator's own server
is not the only thing that answers at that address over the life of a cache entry.

`no-store` is absolute and is not part of this trade. 0006 already fixes that a
response the server said may not be kept is not kept, whatever else would allow
it, and that there is no setting which turns that off.

## Age never withholds

There is no upper age at which the core refuses to serve an entry. An entry that
is a month past its threshold is served, marked stale, with its age, and the
client decides.

This is 0006's rule and it is worth restating exactly once here, in the record
that owns the numbers, because a table of thresholds is the place somebody adds a
second column for "and after this, refuse". The reason not to is that the core has
no way to know what showing a very old library costs the person in front of it,
and a client that would rather show nothing already can: it has the age.

What removes an entry is eviction under #42 or invalidation, both of which produce
`absent`. Age alone never does.

## The demand for freshness

0007 fixes that a caller has three ways to ask, and the third is the server's
answer only, with the cache used for nothing. This record fixes what that returns
when the server cannot be reached: the transport's own failure kind from 0004,
`server-unreachable` or `timed-out` as the case was, and never a stale entry and
never a cache-specific failure.

There is no sixteenth kind for "would have been stale", and the vocabulary does
not need one. The call failed because the server did not answer, which is what the
kind says, and a caller that wanted a value rather than a fresh one would have
asked the ordinary way.

A caller cannot ask for stale only. 0006 already refuses it: a caller wanting the
old value wants one it already has.

## What this does not decide

The bound on total size and what is evicted at it, which is the other route to
`absent`. #42.

Artwork's separate tier and its own bound. #54.

What the core does when a server reports that an entry changed, which shortens an
entry's life without touching this table. #116.

What an entry written by another version, or half written, is allowed to do. #105,
and until it lands every threshold above describes entries this version wrote
completely.

What is served while a request is outstanding, and what a client sees at each
stage of a slow server. 0007, which reads this record for whether the entry may be
served at all.

## Why this is written down before the code

0006 sent the numbers here deliberately, on the grounds that a number written in
that record becomes the number two other documents restate. What it did not send
here, and what this record exists for more than the table, is the pair of rules
around the table: that a server may shorten and not lengthen, and that age never
withholds.

Both get decided by accident otherwise, and in opposite directions. The freshness
statement on a response is the nearest thing to hand at the moment somebody writes
the read path, so the code honours it in both directions without anybody deciding
that a server may extend how long a device holds something. And the upper bound
gets added the first time somebody sees a very old entry served, as one line in
the read path, at which point the offline behaviour in #45 quietly stops working
for anybody who was away longer than that line allows.

The five numbers themselves are the least valuable part of this record and are the
part that will move. What has to exist before the first read path is written is
one table rather than five call sites, because the alternative is not five wrong
numbers, it is five places to look when the sixth caller wants to know what the
rule is.

## Alternatives, and what each cost

One threshold for everything. One number to reason about and to tune. 0006 already
refuses it and gives the reason: an artwork blob that is immutable at its address
and a library query that changes when anybody adds a file cannot share a number
without it being badly wrong for one of them.

Thresholds supplied by the client. It puts the number next to the person who knows
what the screen shows, and a television could hold things longer than a phone. It
costs the single behaviour this repository exists for, since eleven clients would
pick eleven sets, and the first support report about a stale list would have to
begin by asking which client and what it configured.

Honouring the server's stated freshness in both directions. Consistent, standard,
and it is what an HTTP cache does. It costs the direction where a server, or
whatever sits in front of one, can decide how long somebody's device holds their
library, which 0101 places outside what the operator's trust covers.

An upper age past which a stale entry is refused. It stops anybody seeing anything
very old, which is the failure that sounds worst when described. It costs the
offline case outright, since the entries a person needs on a train are exactly the
old ones, and it hands the core a decision about a screen it cannot see.

Serving stale entries with no age attached. The read interface becomes a plain
get. 0006 already refuses it, and this record adds only that the age is what the
demand-for-freshness path exists to let a caller avoid, so removing it would leave
that path as the only way to express a preference.

No stale serving at all, so that a cache answers only when fresh. Nothing to mark,
nothing to explain, and every answer is current. It costs the cold-start number in
#46 and the offline behaviour in #45, which are two of the things this repository
exists to provide.

## What would reverse this

The benchmark harness in #65 measures a threshold against the cold-start number in
#62 and finds one of them wrong. The numbers move, and a record that only moves
numbers supersedes this one rather than editing it, so that the reasoning above
stays readable next to what replaced it.

A server line is found that states freshness in a way the shorten-only rule cannot
read, so that honouring it at all requires honouring it in both directions. Then
the rule is not implementable as written and the replacement says what the core
does instead.

Someone is found to have needed the refused upper bound, twice, with what they
were showing and to whom. One is a client that could have used the age. Two is
evidence that the age is not enough to act on, and the replacement says what the
core withholds and why.

`stale` turns out to be the state a client sees most, measured on the diagnostic
events in #100 rather than assumed. That means the thresholds are describing a
cache that is never fresh in practice, which is a different failure from a number
being slightly wrong, and it is a record about what is being cached rather than
about how long it lasts.
