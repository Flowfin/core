# 0056. The unit a playback position is expressed in, and its three edges

Date: 2026-08-16

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #56

## The decision

A position is a whole number of ticks of one hundred nanoseconds held in a signed
sixty four bit integer inside one named type that also carries a duration, it is
never negative and never more than a stated duration, both bounds are applied
where a value enters the core rather than at each use, conversions out of the
type truncate toward zero while conversions into it are exact, and the core
always states a position on the wire because an absent one means finished to the
server.

## The unit is the server's, and the server's is ticks

The server carries a position, a duration and a reported position in the same
unit, and names all three for it. Read at
`ae8723026d97b6d0f926638803edef338919b794` in the public server repository:

    $ git clone https://github.com/jellyfin/jellyfin
    $ J=ae8723026d97b6d0f926638803edef338919b794
    $ git -C jellyfin grep -n "PositionTicks\|RunTimeTicks" "$J" \
        -- MediaBrowser.Model/Session/PlaybackProgressInfo.cs \
           MediaBrowser.Model/Dto/UserItemDataDto.cs \
           MediaBrowser.Model/Dto/BaseItemDto.cs | sed "s/^$J://" | grep -v Cumulative
    MediaBrowser.Model/Dto/BaseItemDto.cs:193:        public long? RunTimeTicks { get; set; }
    MediaBrowser.Model/Dto/UserItemDataDto.cs:32:        public long PlaybackPositionTicks { get; set; }
    MediaBrowser.Model/Session/PlaybackProgressInfo.cs:72:        public long? PositionTicks { get; set; }

A tick is one hundred nanoseconds, which the server's own source says where it
converts from a unit that is not:

    $ git -C jellyfin grep -n "1 tick == 100 ns" "$J" \
        -- src/Jellyfin.MediaEncoding.Keyframes/Matroska/MatroskaKeyframeExtractor.cs \
        | sed "s/^$J://"
    src/Jellyfin.MediaEncoding.Keyframes/Matroska/MatroskaKeyframeExtractor.cs:78:        // TimestampScale is in nanoseconds, scale it to get the value in ticks, 1 tick == 100 ns
    src/Jellyfin.MediaEncoding.Keyframes/Matroska/MatroskaKeyframeExtractor.cs:84:        // TimestampScale is in nanoseconds, scale it to get the value in ticks, 1 tick == 100 ns

and which it states again as a count, in the place a reader is most likely to
meet it:

    $ git -C jellyfin show "$J":MediaBrowser.Controller/Session/SessionInfo.cs | sed -n '22,23p'
            // 1 second
            private const long ProgressIncrement = 10000000;

Taking the unit rather than converting to a friendlier one is the whole of this
decision, and the reason is not that conversion is expensive. It is that the
conversion is not the identity in both directions. Ten million ticks is a second
exactly, so seconds convert in and out without loss, but a position the server
gave back is not a whole number of seconds and is not a whole number of
milliseconds either. Convert it in, hold it, convert it out, and the value that
goes back is not the value that came, which makes the comparison 0058 rests on
into a comparison of two numbers that were never in the same unit.

The width is the server's too. A signed sixty four bit integer is what the server
declares, and matching it means no value the server can state is unrepresentable
here and no value stated here overflows on the way back.

## One type for a position and a duration

They are the same quantity measured from the same origin, they arrive in the same
unit, and every rule that uses one uses the other: 0058 compares a position
against a duration and subtracts one from the other in both of its boundary
tests, and 0060 does the same. Two types in one unit would mean two sets of
conversion helpers, and the first caller needing to subtract a position from a
duration writes the conversion between them by hand, which is the arithmetic this
record exists to remove from call sites.

Subtracting two of them gives one of them, saturating at zero. That is the
rewind in 0058 rather than a general-purpose arithmetic, and saturation is what
makes the rewind correct with no test at the call site: three seconds into an
item less a ten second rewind is the beginning, and it does not need a caller to
remember that.

## The three edges

A position is never negative. Below zero is not a value the type holds, so a
caller cannot construct one, and arithmetic inside the core saturates at zero
rather than producing one. That removes the failure this issue names, a signed
value going negative on a seek to the start, at the type rather than at every
place a seek is handled.

A position is never past a stated duration. Where a duration is known, a larger
position is clamped to it. Refusing instead would throw away a real position over
a metadata inaccuracy, and the inaccuracy is ordinary: a stated duration comes
from a probe of the file and the stream can run slightly past it. The server
tolerates the same drift in the same direction, treating anything within one
second of the end as the end, which is the quoted `UpdatePlayState` below.

So the clamp is silent within one second and reported through 0100 beyond it,
once per item. A stream running a minute past its stated duration is a library
that says something untrue about a file, which is an operator's to fix and
nobody's to notice unless somebody says it.

Where the duration is not stated there is no upper bound and only the zero floor
applies. `RunTimeTicks` is nullable above, so this is a case the server produces
rather than one this record invents, and 0058 and 0060 already fix that such an
item keeps a position from its first moment and is never treated as finished on
its own.

Both bounds are applied where a value enters the core, from a caller or from a
server, and never at each use. Applied at each use they are a rule somebody has
to remember at every call site, and the sites that forget are the ones nobody
reaches in a test.

That is a departure from the shape 0101 uses for a declared length or a declared
dimension, which is refused rather than clamped, and it is deliberate rather than
an oversight. Those are numbers something is allocated or indexed against, so a
wrong one is a memory fault and refusing is the only safe answer. Nothing is
allocated against a position. A wrong one costs a person the wrong place in a
film, and refusing the answer that carried it would cost them the library.

## Absent is not zero, and the core never sends absent

The field the core writes is optional and the field it reads back is not:
`PositionTicks` above is nullable and `PlaybackPositionTicks` is not. That
asymmetry is not cosmetic, because of what the server does with the absence:

    $ git -C jellyfin show "$J":Emby.Server.Implementations/Library/UserDataManager.cs \
        | sed -n '445,461p'
                var positionTicks = reportedPositionTicks ?? runtimeTicks;
                var hasRuntime = runtimeTicks > 0;

                // If a position has been reported, and if we know the duration
                if (positionTicks > 0 && hasRuntime && item is not AudioBook && item is not Book)
                {
                    var pctIn = decimal.Divide(positionTicks, runtimeTicks) * 100;

                    if (pctIn < _config.Configuration.MinResumePct)
                    {
                        // ignore progress during the beginning
                        positionTicks = 0;
                    }
                    else if (pctIn > _config.Configuration.MaxResumePct || positionTicks >= (runtimeTicks - TimeSpan.TicksPerSecond))
                    {
                        // mark as completed close to the end
                        positionTicks = 0;

A report carrying no position is read as the whole duration, which is the item
finished. So an optional field left unset does not mean "no news", it means "they
watched all of it", and a client that omitted the position on a report it sent
for some other reason would mark things watched that nobody watched.

This record therefore fixes that the core always states the position, on every
report 0057's cadence produces and on every one of its five immediate events. The
type has no absent value for the same reason: a position of zero is a position,
and a person who has never opened an item has no entry rather than an entry
holding nothing.

Reading the other direction, the core cannot tell an item nobody has watched from
one somebody stopped in the first seconds, because the field it reads back cannot
express the difference. That costs nothing, because 0058 already keeps no resume
position below the first sixty seconds or the first five per cent, so both cases
resume at the beginning by that record's own rule rather than by this absence.

## What the core reads back is not what it sent

The same quotation shows the server applying its own boundaries before it stores
anything. Below its own threshold the position becomes zero; above its other one
the position becomes zero and the item is marked played. Those thresholds are an
operator's setting rather than a constant:

    $ git -C jellyfin grep -n "MinResumePct\|MaxResumePct\|MinResumeDurationSeconds" "$J" \
        -- MediaBrowser.Model/Configuration/ServerConfiguration.cs | sed "s/^$J://"
    MediaBrowser.Model/Configuration/ServerConfiguration.cs:133:    public int MinResumePct { get; set; } = 5;
    MediaBrowser.Model/Configuration/ServerConfiguration.cs:139:    public int MaxResumePct { get; set; } = 90;
    MediaBrowser.Model/Configuration/ServerConfiguration.cs:145:    public int MinResumeDurationSeconds { get; set; } = 300;

What this record takes from that is one rule and no more: a position the core
reads back is a value the server decided, not the value the core sent, so nothing
here may be written as though the two round-trip. A report is not a write whose
success can be checked by reading it again, and a core that compared them would
find them differing on ordinary items and would have to guess why.

What follows for the boundaries themselves is 0058's rather than this record's,
and it is written into #58 rather than settled here, because those numbers are
that record's and a record is superseded rather than edited.

## Conversions, and the direction each loses in

Into the type, from whole seconds and from whole milliseconds, exact in both
cases, because both divide ten million evenly.

Out of the type, to whole seconds and to whole milliseconds, truncating toward
zero and never rounding to nearest. Rounding up is what a reader expects and it
is wrong at exactly one place: the last tick of an item rounds to a whole second
past the stated duration, and 0058's finished test then fires on an item that is
one tick short of its end. Truncation is wrong by less than a unit at the other
end, where nothing tests a boundary.

The tick count itself is reachable, through a name that says the unit, because
something has to write it on the wire. What is refused is a plain number of
unstated unit anywhere a position is meant on the core's own interface, which is
this issue's fourth condition and is the only one of the four that is a property
of the interface rather than of a test.

Overflow is not reachable and the arithmetic behind that is worth writing down
rather than assuming. The largest signed sixty four bit value is
9223372036854775807, which at ten million ticks a second is 922337203685 seconds
and a fraction, and that is about twenty nine thousand years:

    $ awk 'BEGIN { printf "%.5f\n", 9223372036854775807 / 10000000 / 86400 / 365.25 }'
    29227.10230

So no duration a server can state and no sum of positions inside an item comes
near it, and this record states that bound rather than asking for a check nothing
could reach.

## What this record does not decide

Where playback resumes, what counts as the two ends of an item, and whose
position wins a disagreement. 0058 and 0060, both landed, both expressed in this
unit now that there is one.

How often a position is reported, and through what. 0057 and 0047.

Which endpoints carry a position and in which shapes across server versions. #10,
with the caveat this record repeats rather than resolves: everything above was
read at one commit of the public server repository, named where it is quoted.

Whether a client sees the type at all or a platform binding converts it. #75 and
#76, where the contract and its checks are.

## Why this is written down before the code

Three landed records defer to this issue by number and cannot be read without it.
0057 says the position a report carries has its unit and precision here. 0058
says its three durations and two proportions are expressed in whatever unit this
record chooses. 0060 says the same. So the unit is already load bearing for
decisions that are made, and the first code that has a position to hold supplies
it for all three.

What that code will do is predictable. It will use whatever duration type the
runtime chosen in #11 offers, because that is what is to hand and it reads as
correct. Every runtime has one and they do not agree: some hold nanoseconds in a
signed sixty four bit integer, some hold ticks of a hundred nanoseconds, some
hold a pair of a second count and a sub-second count. The unit on the wire then
becomes whichever the runtime picked, and the wire unit is a property of the
server rather than of the language, so the conversion at the boundary is
introduced silently on the first day and is the source of the drift this issue
names.

The second thing it will do is treat the optional field as optional. That is a
one-line saving that marks items watched, and it is invisible in a test against a
fake server unless the fake server implements the sentence quoted above, which
nobody would think to do without having read it.

## Alternatives, and what each cost

Whole milliseconds. It is the unit most clients already speak, it is small enough
for any seek a person makes, and the arithmetic is readable. It costs an inexact
conversion in one direction: a tick count from the server is not a whole number
of milliseconds, so every value that arrives loses up to nine thousand nine
hundred and ninety nine ticks, and the loss is applied again on the way back. One
round trip of a position through the core would move it, which is under a
millisecond and is invisible until 0058 compares two positions that took
different routes.

Whole seconds. The coarsest unit anybody would defend, and it makes every number
in 0057 and 0058 readable without conversion. It costs a seek accuracy nobody
would accept and it costs the same round-trip inexactness as milliseconds, at a
thousand times the size.

A floating point number of seconds. The unit a player hands you, and the one that
needs no thought at all. It costs exactness in a way that is worse than losing a
fixed amount: the values are not representable, so two positions that should be
equal compare unequal, and 0058's disagreement rule and 0060's boundary tests are
both equality-adjacent comparisons. It also accumulates, so a position built by
adding intervals drifts from one that was reported directly.

The duration type of whatever runtime #11 chooses. The obvious answer, free, and
it comes with conversion helpers already written. It costs the decision being
made by a language choice that has not happened, it moves when the language
moves, and it makes the wire representation depend on a detail of a runtime
rather than on the server. Where the runtime's own precision is finer than a
tick, it also invites a position with a sub-tick part that cannot be sent.

Separate types for a position and a duration. It is the more careful modelling
and it would refuse subtracting a duration from a position in the wrong order. It
costs two sets of conversions and a conversion between the two types, which is
arithmetic at a call site, which is what this record removes. The mistakes it
would catch are ones 0058's rules do not admit anyway, since both of that
record's boundary tests are a position against the duration of the same item.

Refusing a position past a stated duration instead of clamping it. It is the
shape 0101 uses everywhere else and it is consistent. It costs a person their
resume point over a stated duration that was slightly short, which is a normal
state of a library rather than an attack, and the server itself tolerates the
same drift.

## What would reverse this

The server states a position in a second unit on any endpoint the core uses. The
type then has to carry which unit a value came in, or the core has to convert,
and either is a change to what this record decided rather than an addition.

A supported server version is found where a report carrying no position does not
mean the whole duration. The rule that the core always states the position stops
being load bearing there, and the record is superseded by one written against the
range #10 fixes rather than the one commit quoted here.

The clamp against a stated duration is reported through 0100 more often than it
is silent, on a real library. That is evidence that stated durations and streams
disagree as a rule rather than as an exception, and the tolerance is then a
measurement rather than the one second borrowed from the server.

A caller is found that legitimately needs a position finer than a tick. Nothing
in a media file is stated that finely today, so this is a change in what the
server carries rather than a request the core can grant, and it lands as a record
that names this one.
