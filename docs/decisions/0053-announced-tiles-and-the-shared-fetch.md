# 0053. Announced tiles, the order they are started in, and the fetch two of them share

Date: 2026-08-13

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #53

## The decision

A client announces the tiles it expects to draw as an ordered window and
withdraws one by cancelling it, so the order of the window is the whole of the
priority the core accepts and no tile carries a number beside it; an announcement
is advisory, so what it can cost is a fetch that started late and never an image
that does not arrive; announcements that resolve to one artwork entry under 0041
share one fetch, which is abandoned when the last caller holding it has
withdrawn and not before; and a withdrawal reaches the transport and the decoder
through the cancellation 0009 already defines, so what it stops is bounded by
0027's drain and one decode step rather than being immediate.

## What a client announces, and what the order means

A client hands over an ordered list of the artwork it expects to draw, for one
session. The order is the order the core starts the work in. Announcing again
replaces the previous window rather than adding to it, because a wall being
scrolled produces a new window several times a second and a surface that
accumulated would need a second call to say what is no longer coming.

Announcing is a call that cannot wait in the terms of 0009. It returns from state
the core is already holding, it takes no cancellation handle, and it is safe from
any thread. Nothing about it reaches a server, so there is nothing for it to wait
on.

The order is a statement about starting and not about finishing. Two tiles
started in order complete in whatever order their bodies and the decode budget in
0050 produce, and a client that draws in announcement order regardless is a
client waiting on the slowest of the two. What a client gets from the order is
that the tile nearest the screen is the one the core spends its next connection
on.

There is no priority number, and that is a refusal rather than an omission. 0050
already declines a client-supplied priority on the decode admission queue and
gives the reason, which is that it is a scheduling interface the core would then
owe every client. A priority here and an ask order there would be two orderings
over the same work, and they disagree the first time a tile announced second
carries the higher number. One order that both read is the same decision taken
once.

## An announcement is advisory

A tile that was announced is fetched ahead of being asked for. A tile that was
never announced is fetched when the client asks for it, at the moment it draws.
Announcing is therefore a way of being early and never the way of being served,
and every consequence below rests on that.

It is what makes the bound in the next section affordable. It is what lets the
core drop a window it cannot hold without anything failing, because dropping the
tail of a window costs the time a fetch would have saved. And it is why nothing
here appears in 0004: no announcement produces a failure, so there is no kind for
one and none is asked for.

## The bound on what may be announced

Two hundred and fifty six entries per session. A window longer than that is kept
to its first two hundred and fifty six in the order it was given, the remainder
is not announced, and the core reports through 0100 that a window was cut and by
how much.

Chosen rather than measured, and #65 is where a measured replacement would come
from. The arithmetic is the wall this issue is named for and one screen of
lookahead beyond it:

    200 tiles announced for the wall
     56 further entries, which is more than a screen of any tile size on any
        supported target

The case the bound is against is not a client that announces two hundred and
one. It is a client that announces its library. A listing of twenty thousand
items announced as one window is a queue the core holds proportional to somebody
else's data, and it is the shape a client arrives at by announcing everything it
has rather than everything it will draw. Cutting the tail is the right response
to it because the tail is the part furthest from the screen, and because of the
section above: the tiles that were cut are still fetched when they are asked for.

The remainder is reported rather than dropped in silence. A client whose windows
are being cut is a client whose prefetch is doing nothing, and there is no other
way for its author to find that out.

## What is outstanding, and what is only announced

The cap on outstanding requests this issue asks for is not a number this record
adds. 0027 holds at most six connections to one server, and it names this wall as
the reason for the figure.

What this record fixes is where the other announced tiles sit while those six are
busy. They are held as data in announcement order, and they are not outstanding
requests and not waiters on a lane. 0009 sizes the waiting lane at one waiter per
permitted connection and creates it once when the core is created, so two hundred
announced tiles cannot each hold a waiter without that lane being a thing that
grows, which 0009 does not do.

So the announced set is a queue in front of the transport rather than a queue
inside it, and the number that decides how many of its entries are in flight is
0027's, unchanged.

## What a withdrawal stops

A tile is withdrawn by cancelling it, and cancelling is 0009's, not a second
mechanism. What the caller may assume the moment it returns, what it may not, and
what it may assume once it waits on the handle are that record's three lists, and
they hold here without amendment.

Two of them are worth following through to this case, because the wall is where
they bite hardest.

Bytes still coming are read no further than 0027 allows, which is sixty-four
kilobytes or one second before the connection is closed instead. An artwork body
is far past that bound, so a tile that scrolled off closes its connection rather
than finishing the download to save a handshake.

A decode already inside a step runs to the end of that step and its buffer is
then released to whatever is waiting under 0050. The bound is one step and not
zero.

The third consequence is the one that surprises. 0009 requires that nothing a
cancelled call would have produced reaches the cache, so a withdrawn tile whose
body had already arrived has those bytes dropped rather than written to the
artwork tier in 0054. Scrolling back to it fetches again. That is the cost of the
guarantee rather than an oversight in it, and the alternative is a cancelled call
that goes on doing work with a visible effect, which is what a caller cancels in
order to stop.

That rule is about a call, so it does not reach a fetch that another caller is
still holding. The next section is where that case is settled.

## One fetch behind two tiles

Two announcements that resolve to the same artwork entry share one fetch and one
decode. The identity they are compared on is the cache key 0041 builds for that
entry, which is per server and per account, so two tiles naming one poster join
and two tiles on two servers never do.

Taking the key rather than the address is what makes the rule survive #49. That
issue decides how a requested size becomes an address, and its own condition
requires two nearby requested sizes to resolve to one cache entry. Coalescing on
the address would fetch those twice, and it would do it for the case #49 exists
to prevent. Coalescing on the key inherits whatever #49 decides instead of
deciding it here.

A real ask joins an announcement already in flight rather than starting beside
it. This is the ordinary case and not an unusual one: the client announced the
tile a moment before it drew it, which is the whole point of announcing.

Each caller keeps its own handle and its own outcome, which 0009 delivers exactly
once per call. So a fetch shared by three callers is abandoned when the third has
withdrawn, and never when the first has. Written the other way round, the first
tile leaving the screen cancels the fetch the tile still on it is waiting for,
and what a person sees is a poster that never arrives, on the item that shares
its artwork with another, at a moment nothing records.

This is not a cache and it does not become one. What it covers is a second
request while the first is in flight. Afterwards an artwork entry is in the tier
0054 describes and is served from there, and an absence is kept by nothing,
because 0006 lists what may be kept and 0043 closes that list. The comment on #51
sets out the two readings of that issue's second-request condition. This record
is the narrower one and gives it a home: a second request inside one screenful is
answered without a second network call because the first is still in flight.
Whether an absence is kept at all, and for how long, is #51 and reaches 0006 and
0043. Nothing here decides it.

## Prefetch, and the lock a caller could wait on

0064 states the third of the three obligations in #64 in the form that is true,
which is that prefetch does not hold a lock a caller waits on up to a bound
nothing states, and it places the duration here.

The property this record fixes is the shape of that bound rather than the number.
Announcing takes the core's own lock over the announced set, and while holding it
the core compares the new window against the previous one and calls out to
nothing: not to the client's stores, not to the diagnostics sink in 0100, not
into the transport. The work under the lock is therefore proportional to the
window, which the section above bounds, and it is the core's own work rather than
anybody else's, which is the condition 0009 states for every lock it admits.

The duration is not stated. A number here would be a number with no command
behind it, and #65 is the harness that produces one. What is owed is the
measurement rather than the property.

## What this record does not decide

How a requested size becomes an address, and which sizes share an entry. That is
#49, and the coalescing rule above reads its answer.

Supplying the dimensions before the bytes so a late image cannot move the layout.
That is #52. It changes what a client is handed and not what is fetched or in
which order.

How many decodes may be held at once and what happens at that bound. That is
0050, which also fixes that waiting decodes start in the order they were asked
for, and the announcement order is what that order is on this path.

How many connections are open and what a cancelled body costs. That is 0027.

What a span carries and what it costs when nobody is listening. That is 0061,
which already names this wall as the case where most spans are withdrawn and
where the cheap-subscription property is bought.

## Why this is written down before the code

Prefetch is added at a call site, in the sitting where somebody notices that a
scroll is slow, and it arrives as a loop that starts the next twenty images. What
that produces is not a bug anybody can see on the machine it was written on. A
fast scroll spends every one of 0027's six connections on tiles that have already
left the screen, the tiles arriving on the screen queue behind them, and the
report is that scrolling is slower the faster you scroll. Nothing in it names
prefetch.

The withdrawal rule is the part that cannot be repaired quietly. A core that
cancels a shared fetch on the first caller's withdrawal produces one missing
image, on the items that share artwork, under scrolling, and it produces it on
one device in ten. Every route by which a person would report it describes
something else: a server that returned nothing, a network that dropped, a wall
that sometimes has a hole in it. #51 has already made a hole in the wall a
first-class answer, so the client showing it is behaving correctly and there is
nothing for anybody to notice.

The ordering surface is the part that cannot be added later. A core whose artwork
calls have no announcement for a year has eleven clients that call in draw order
and cancel on scroll, which is the interface this record replaces, and adding an
order afterwards means every one of them decides separately whether to use it.
Deciding it now costs one call and a list.

## Alternatives, and what each cost

No announcement at all, with the client asking for what it draws and cancelling
what it does not. The smallest surface, nothing to bound, and the cancellation
half already exists in 0009. It costs working ahead entirely: the core hears
about a tile at the moment somebody is waiting for it, so every tile on a scroll
is a cold fetch, and the order the core starts work in is whatever order the
client's draw loop happened to call in.

A priority number on each announced tile. More expressive, and it lets a client
raise one tile without restating the window. It costs the second ordering
described above, against a decode queue 0050 has already decided is served in ask
order, and it is a scheduling interface every client then has to have an opinion
about.

A count rather than a list, with the client asking the core to work ahead by
twenty. Much cheaper for the client to call. The core cannot honour it: it does
not hold the client's list and 0003 refuses it the knowledge of what is being
drawn, so it does not know what the next twenty are.

Coalescing on the address rather than on the cache key. One fewer concept, and it
is what the transport could do on its own. It fetches twice for two nearby sizes
that #49 resolves to one entry, which is the case that issue is written for.

Abandoning a shared fetch as soon as any caller withdraws. No counting, and it is
what a single-holder implementation does by default. It costs the caller still
waiting its image, in the way described above, which is invisible everywhere it
is reported.

Keeping the bytes of a withdrawn tile that had already arrived. Better for
scrolling back, and the bytes are paid for. It costs 0009 its cancellation rule,
which says a cancelled call produces nothing that reaches the cache, and that
rule is worth more than the fetch it saves because a client is entitled to rely
on it at every other call site too.

An unbounded announced set. No number to defend and no window ever cut. It makes
the core hold a queue the size of somebody's library, and the client that
produces it is not doing anything obviously wrong.

## What would reverse this

The harness in #65 measures a wall of two hundred tiles and finds windows being
cut in ordinary use. The bound is then too small for the case it was chosen for,
and it moves in a record that supersedes this one.

#49 answers that two nearby requested sizes resolve to separate cache entries.
The identity coalescing is compared on is then wrong rather than merely awkward,
and it is restated over whatever that record makes the identity of an artwork
request.

0050 grows a client-supplied priority on its decode queue, for its own reasons.
The two-orderings argument above disappears with it, and the refusal of a
priority here is superseded alongside that record rather than kept as a rule
whose reason has gone.

A client is found that needs a tile started out of announcement order, twice.
One is a client that should be announcing a different window. Two is an ordering
surface that is the wrong shape.

0009's guarantee that a cancelled call delivers no outcome weakens, which is a
reversal that record already names for itself. The rule about the last caller
holding a shared fetch is written on top of it and is restated over whatever
replaces it.

A dependency or a platform is found where a fetch cannot be joined after it has
started, so a real ask cannot attach to an announcement already in flight. The
ordinary case then costs two fetches, and the answer is either that announcing
and asking are one call or that this record is superseded by one that says a
prefetch is abandoned when the ask arrives.
