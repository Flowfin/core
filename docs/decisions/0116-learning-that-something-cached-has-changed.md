# 0116. Learning that something cached has changed

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #116

## The decision

The core listens on the connection the server offers for change notifications and
uses what arrives to shorten 0043's freshness rather than to add a second
mechanism beside it: a named item is invalidated and becomes `absent`, everything
whose containing queries the core cannot identify has its threshold shortened to
zero and becomes `stale` by age, artwork and capability answers are untouched
because 0006 and 0043 already answer for them, and the listener is an accelerator
whose absence, refusal or silence changes nothing about what a read may be
trusted for.

## What the server offers

Two message kinds on the session connection, read at
`ae8723026d97b6d0f926638803edef338919b794` in the public server repository:

    $ git clone https://github.com/jellyfin/jellyfin
    $ J=ae8723026d97b6d0f926638803edef338919b794
    $ git -C jellyfin grep -n "LibraryChanged\|UserDataChanged" "$J" \
        -- MediaBrowser.Model/Session/SessionMessageType.cs | sed "s/^$J://"
    MediaBrowser.Model/Session/SessionMessageType.cs:13:        UserDataChanged,
    MediaBrowser.Model/Session/SessionMessageType.cs:22:        LibraryChanged,

The first carries identifiers rather than a bare signal, which is what makes a
targeted rule possible at all:

    $ git -C jellyfin show "$J":MediaBrowser.Model/Entities/LibraryUpdateInfo.cs \
        | grep -n "public string\[\]"
    29:        public string[] FoldersAddedTo { get; set; }
    35:        public string[] FoldersRemovedFrom { get; set; }
    41:        public string[] ItemsAdded { get; set; }
    47:        public string[] ItemsRemoved { get; set; }
    53:        public string[] ItemsUpdated { get; set; }
    55:        public string[] CollectionFolders { get; set; }

It is also already batched by the server, on an operator's setting:

    $ git -C jellyfin grep -n "LibraryUpdateDuration" "$J" \
        -- MediaBrowser.Model/Configuration/ServerConfiguration.cs | sed "s/^$J://"
    MediaBrowser.Model/Configuration/ServerConfiguration.cs:178:    public int LibraryUpdateDuration { get; set; } = 30;

And the connection states its own liveness rule rather than leaving the core to
invent one:

    $ git -C jellyfin show "$J":Emby.Server.Implementations/Session/SessionWebSocketListener.cs \
        | sed -n '26p;36p'
            private const int WebSocketLostTimeout = 60;
            private const float ForceKeepAliveFactor = 0.75f;

The server drops a connection it has heard nothing on for that many seconds, and
tells the client the number ahead of time at three quarters of it. That is the
one thing the alternative below is most expensive without.

## Why listening, and what it does not buy

It is the only one of the three that answers the question this issue leads with.
Somebody adds a film from a laptop and the television shows it without anybody
restarting anything, and somebody marks something watched on a phone and the
television agrees.

It buys that at the cost of a held connection per session, which is real and is
the reason the alternatives below are not dismissed. What makes the cost payable
here rather than in general is that the expensive half of a held connection is
usually knowing whether it is alive, and this server states the timeout and sends
a keepalive ahead of it, so the core is reading a rule rather than guessing at a
heuristic.

What it does not buy is a route by which the core wakes a client. Nothing here
pushes anything upward. The core marks its own cache and a client learns on its
next read, which is what this issue's own condition asks for and is 0043's
stale-then-fresh path arriving sooner. Whether the core offers a client a signal
that something moved is a question about the query surface in #39 and is not
decided here.

## The rule per kind

Item metadata for a named identifier is invalidated. The core knows that item's
answer changed, so what it holds is wrong rather than old, and 0043 already fixes
that this produces `absent` rather than `stale`. That covers `ItemsUpdated` and
`ItemsRemoved`, and `ItemsAdded` names nothing the cache holds.

Library query results are shortened to zero rather than invalidated, and this is
the part that would otherwise be got wrong in the expensive direction. A query
result is keyed by 0041's digest over the request, and a digest cannot be asked
whether the answer under it contained a given item. So the core does not know
which queries changed, only that some may have, and invalidating all of them
would empty the tile wall on every library change, which during a scan is every
thirty seconds by the setting quoted above. Shortening the threshold to zero is
0043's own permission used on a different route: the entry is served immediately,
marked `stale` with its age, the screen fills at once and corrects itself, and
0046's cold start is not paid again on every notification.

Playback positions and watched marks arrive as `UserDataChanged`, which carries
the new values as well as the identifiers. The values are not written into the
cache. What it carries is one part of an item rather than an item, and merging a
part into a cached whole produces an entry that is half of one moment and half of
another, which is the state nothing can later tell from a correct one. So it
invalidates the item metadata entries it names, exactly as `ItemsUpdated` does.

That message carries the account it is about, and 0041 keys the cache per
account, so a notification for a different account signed in on the same device
reaches none of this session's entries.

Artwork bytes have no rule here. 0006 makes a changed image a different key
because the address is content-tagged, and 0043 says so in the row where its
thirty day age is explained, so there is nothing for a notification to
invalidate.

Capability answers have no rule here either. What changes one is a server
upgrade, which neither message reports, and 0043's own row already says the cost
of a wrong one is recoverable on the next day.

## The degradation, and the rule that keeps it honest

No freshness window is ever extended because a listener is connected.

That is the whole of the degradation rule and it is stated as a prohibition
because the tempting change is the other direction. A core that is being told
about every change could hold a library list for a day instead of five minutes,
and the cache hit rate would improve visibly. It would also mean that a listener
which is connected and silent for a reason nobody noticed produces a cache that
is confidently wrong for a day, and a connection that is up and not delivering is
the failure mode this issue names.

So every entry lives under 0043's table whether or not anything is listening. The
listener can only make an entry stale or absent sooner than the table would. A
connection that was refused, never established, dropped, or that the server
stopped talking on leaves every threshold exactly where the table put it, and no
state anywhere records that the cache is being kept current by a listener.

Reconnection takes no schedule of its own. 0038 bounds the attempts of a request
and 0045 is the recovery schedule for a server that is gone, and a third timetable
here would be a third answer to when the core tries again.

The connection goes to the origin 0028 resolved and to no other host, so 0069's
list is unchanged by this record.

## What the bound actually is

This issue asks that a cached read reflect a change within a stated bound, and
the honest answer is that the core cannot state it as a number.

The core's own contribution is delivery: the moment the message is read, the
entries it names are absent or stale, and the next read reflects it. Ahead of
that sits the server's own batching, which is the operator's setting quoted
above and is thirty seconds only until somebody changes it.

So what this record can promise is that the core adds no delay of its own, and
what a test can assert is that a change made on the fake server is reflected on
the next read with no interval of the core's in between. A number covering the
whole path would be a claim about somebody else's configuration.

## Alternatives, and what each cost

Revalidating on the next read and nothing else. The cheapest possible, it needs
no connection, no reconnection policy and no liveness question, and it refreshes
exactly what somebody asked for. It is kept as the floor rather than rejected;
what it cannot do is the case this issue exists for, because a tile wall nobody
scrolls is never re-read, so the film somebody added from a laptop does not
appear until a person navigates away and back. It also makes the freshness
thresholds carry the whole load, which means shortening them is the only lever,
and shortening them is paid by every device on every read.

Polling. Predictable, easy to bound, and it needs nothing held open. It is wrong
on almost every run it makes, and the cost lands on a machine in somebody's
house rather than on a service. The server's own change notifier is batched at an
operator's setting, so a poll faster than that setting cannot learn anything the
previous one did not, and a poll slower than it is worse than the listener at
every point on the curve. It is the option that looks cheapest until it is
multiplied by the number of devices in a household.

Listening and treating a connected listener as a guarantee, so that thresholds
lengthen while it is up. The best cache hit rate of any option here, and the
reason it is refused is written above: it converts a silent listener into a cache
that is confidently wrong, and silent is exactly what a listener is when
something has gone wrong with it.

Writing the values `UserDataChanged` carries straight into the cache. It saves a
round trip for the one kind where the server sends the new value. It costs an
entry assembled from two moments, and 0101 treats a pushed value as untrusted on
the same ground as any other thing the server said, so the saving is a merge of
partially trusted data into a whole that nothing afterwards can tell from an
answer the core asked for.

Invalidating every query result on a notification instead of shortening it. It is
the more obviously correct reading of "the answer may have changed", and it never
serves something out of date. It costs the tile wall on every notification, which
during a library scan is once per batching interval, and what a person sees is
the screen emptying and refilling repeatedly while a scan runs.

## What would reverse this

The server stops naming identifiers in its change notification. The targeted half
of the rule is then not available and the whole of it collapses to the shortening
rule, which is a different decision rather than a smaller one.

A held connection is measured as a visible share of what a device spends on
power or on a metered connection. The listener is then something a client turns
on rather than something the core does, and the record is superseded by one
carrying that measurement and the command that produced it.

A library scan is observed making the tile wall unusable through the shortening
rule, because every read during a scan finds every entry stale and refetches.
That is evidence the shortening needs a floor of its own, and the floor is a
number with a reason rather than an adjustment.

#10 answers that a supported server version offers no such connection at all. The
floor above is then the whole behaviour on that version, and what a core does
across a range where one half has it and the other does not is a decision that
names this record.
