# 0111. Which source is played, and what the handover carries

Date: 2026-08-17

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #111

## The decision

Asking what to play is one call that carries the client's own capability
description rather than resting on the one the server stored at the last sign-in,
the answer is read against that description by a ladder that takes the source the
device can play untouched, then the same streams in a different container, then
the fewest converted streams, with a tie broken by the order the server listed,
the handover that call returns holds an address the platform's own player opens,
the start position 0058 decides and the audio and subtitle streams that were
chosen, and where nothing the server offers or will convert is playable the call
fails with `request-refused` rather than returning an empty answer.

## One call rather than two

0008 owns the interval this call sits inside and fixed its endpoints before this
record existed: the interval is opened by the call that asks the core what to play
for an item and closed by that call returning the playable handover, and it is one
call so that the endpoints need no further definition. An interval spanning two
calls has a gap between them that belongs to nobody, and the gap is where a client
does its own work.

So the negotiation with the server and the handover are one call from the caller's
side, whatever the core does with the server behind it. Splitting them reads as
the more honest shape, because two things are happening, and it puts the time a
client spends between the two calls inside the two seconds the number is about and
inside nobody's budget. That is 0008 superseded rather than implemented, and the
supersession would be invisible: both halves would still work.

Nothing here says the core makes one request. It may make several, and 0046
already places this call among the ones a cold start cannot answer, which is a
statement that it reaches the server. What is fixed is the number of calls a
client makes.

## Which description the answer is computed from

0036 fixed who supplies the capability description and when it is sent, and left
one question here by name: the server accepts a description on a playback request
as well as reading the stored one, and choosing between them belongs to this
issue. It is not a detail beside the source rule. It is the input the source rule
reads, and a request carrying its own description and a server answering from the
stored one produce different sources for the same item on the same device.

The core sends the description on every play call, and the answer is computed from
that one. The stored description is not made pointless by this and 0036's
requirement to send it on every sign-in is untouched: it is what the server holds
for anything the core did not ask for on this call.

Two things the stored description cannot carry are why.

It goes stale between sign-ins, and a session outlives a great many of them. 0036
requires the description on every sign-in precisely because a client that changed
what it can decode after an update would otherwise be described by its previous
version indefinitely, and that argument does not stop at the sign-in boundary. A
platform update that removes a decoder, a television whose output changes when a
different display is attached, and a client that ships a new binding all move the
description without moving the session. Between two sign-ins the stored one
describes what the device could do then.

It collides between two clients on one device. 0036 stores the description against
the device identifier, states that the server keys a live session on the client
name joined to that identifier, and says a client is not required to keep its
identifier distinct from another application's on the same device. Two clients are
therefore two sessions and one stored description, and the last of them to sign in
decides what the other is offered. 0036 answers the neighbouring case, two people
on one television describing one device once, and that answer holds because it is
one client. This one is not the same case and is not covered by it.

What it costs is bytes on every play call rather than bytes on every sign-in.
The play call happens once per item somebody starts rather than once per tile, so
the cost lands in the place that can carry it. That is an argument from where the
call sits rather than a measurement, no run has been made, and #65 is where a
measured one would come from.

One property comes out of this that is worth having in view. A server that ignored
the description on the call and answered from the stored one would hand back a
source chosen against the wrong description, and the core cannot see that from the
answer. What it can see is the answer failing the ladder below, because the ladder
reads the same description the core sent. The mismatch therefore arrives as
nothing playable, from the core, rather than as a stream that fails inside the
platform's decoder, which is the one place the core has no visibility and a client
has no error to map.

## The rule that picks a source

Four rungs, and the core takes the highest one the description admits.

    played as it stands       the bytes the server holds, sent unchanged
    container changed         the same streams, repackaged
    some streams converted    the ones the description does not admit, no others
    every stream converted

Highest first because each rung down costs somebody something the rung above does
not. Playing a file as it stands costs nothing at all: the picture and the sound
are the ones in the file, and the server sends bytes it already has. Changing the
container re-encodes nothing, so the picture and the sound are still the file's
own, and it costs the server a repackage. Converting a stream costs picture or
sound quality that cannot be recovered afterwards, and it costs the operator's own
machine, which on self-hosted software is frequently a small one doing this while
somebody else is watching something. Converting every stream costs both, on every
stream, including the ones that did not need it.

A source is on a rung only where the description covers what will actually be
played there: the container, the video stream, and the audio and subtitle streams
chosen by the rule below. That ordering matters and is decided here rather than
left to the code. Judging a source before its streams are chosen tests it against
streams nobody will play, so a file whose second audio track is undecodable is
refused for a track the person was never going to hear, and the person gets a
conversion instead.

Where two candidates sit on one rung, the one the server listed first is taken.
The order is the server's own, it is the same order for two clients asking the same
question, and it costs nothing to follow. A tie broken on something the core
computed would need a number the description does not carry, and inventing one
here would be the core holding an opinion about a file it has not seen.

What this rule buys is the sentence the issue asks for: two clients given the same
item start it the same way. It buys it exactly where the two agree. Two clients
with different descriptions land on different rungs and that is the rule working
rather than failing, because they are different devices. Two clients with the same
description and the same preferences below land on the same source, and where they
did not, the rule is what makes the difference visible instead of leaving it to
whichever caller asked first.

## Which audio and subtitle streams are chosen

The caller may supply an ordered list of languages it prefers, and the core takes
the first stream matching the earliest entry it can satisfy. Where the caller
supplies none, or none of them matches, the core takes the stream the source marks
as its own default, and where the source marks none, the first one listed.

The language belongs to the caller for the same reason the capability description
does. What a person wants to hear and read is a setting a client holds, in a
screen the core does not draw, and 0003 keeps the core out of both. A core that
guessed would guess from something it holds, and the only candidates are the
server's own default and a locale it would have to be handed anyway.

Subtitles are chosen and not turned on. What the handover names is which subtitle
stream was chosen if one is shown, and whether to show it at the start is a client
setting the core has no view on. 0112 places subtitle rendering on this side of
its line, which is a statement about where that work is written rather than a
statement that this call decides it.

## What the handover carries

An address the platform's own player opens, with whatever the player needs in
order to open it. That wording is 0003's and is quoted rather than improved on,
because it is the boundary this issue is most likely to erode.

The position to start from, which is 0058's rule applied rather than restated: the
rewind, the two ends of an item, and the case where the device holds a position it
has not yet delivered are all decided there and none of them is re-decided here.

The audio and subtitle streams chosen by the rule above, so that a client can show
what is playing without asking a second question and without inspecting the
address.

Which rung the source came from, because a client that wants to tell a person the
file is being converted has no other way to know, and the alternative is each of
eleven clients inferring it from the shape of the address.

Nothing else. Not a player, not a decoder, not a frame, not a duration the client
should count against, and nothing beyond the point 0112 places outside. The core
opens no player and holds no opinion about what the client does with the address
it was given.

## When nothing is playable

The call fails with `request-refused` from 0004's vocabulary, and where the server
supplied an error code of its own it rides in that kind's payload as the opaque
string 0004 already defines it to carry.

That kind rather than an empty answer, because an empty result is the shape a
client forgets to handle, and it is indistinguishable from the item having no
sources at all.

That kind rather than a new one, because two conditions arrive at this point and
both are refusals of the same request. The server may answer that it will serve
nothing, which is a refusal it made. Or the server may offer sources and
conversions and none of them clear the ladder, which is a refusal the core made on
the description the client supplied. In both the request was understood and
answered, which is what 0004 puts in that row's `Means` column.

The cost of using one kind for both is that the payload cannot say which of the
two happened. 0004 fixes that payload as the server's error code and nothing more,
so a client sees an absent code where the core refused and where the server
refused without one. What tells them apart is the diagnostic event under 0100,
which is where somebody diagnosing this looks, and it is not what a client shows a
person. Where a client is found needing the difference in front of a person, the
repair is a field on 0004's payload rather than a sixteenth kind, and it is a
supersession of 0004 rather than of this record.

## A slow connection, and what this rule does not do about it

The ladder reads what the device can decode. It does not read what the link can
carry, and those are different questions with different answers.

A device that decodes everything, on a link that cannot carry the file at the rate
it was made at, plays as it stands under the rule above and stalls. The core sees
that as a slow server, which 0007 already names and 0027 already bounds, and the
person sees a film that stops. Asking for a conversion at a lower rate is the
repair, and the core cannot decide it here: this call is the first request for that
item, so there is no measurement of the link to read, and a rate the core guessed
would convert files on a fast connection for nothing.

What is decided is that the caller may supply a ceiling and the core carries it
into the request, narrowing the candidate set before the ladder runs. A client
holds what the core does not: whether the device is on a metered connection, what
a person set, and what the platform says about the network it is on. The core
invents no ceiling and applies none where the caller supplies none, which is the
same division 0036 draws for the capability description.

The residual is stated rather than solved. A person on a slow link with no ceiling
set gets the stall rather than the conversion. Solving it needs the core to act on
a measurement of the link, which is a decision about when to abandon and retry a
play that is already in flight, and that is a record of its own rather than a
sentence here.

## What this record does not decide

Which endpoint carries the play call, and which carries the description. This
record decides that the description travels with the call. Naming the request it
travels on needs the surface enumerated, which is #10, and 0036 already sends that
half there.

Which server versions accept a description on a play call. Everything 0036 states
about the server was read at one commit, named where it is quoted there, and the
version range is #10's once entry 3 of #1 says what it is.

What the vocabulary of a capability description is: which containers, codecs,
profiles and limits it can express. 0036 gives the core the shape and the client
the contents, and the shape itself is written where the interface is, with #75 and
#76 as where a client's description is checked against it.

Whether a play already in flight is moved to another source. Everything here is
about one call and the answer it returns. 0005 guarantees that playback already
started is not interrupted by the core, and a mid-play switch is a second
negotiation on a stream the core does not hold.

What a client does with the address. 0112 draws that line and this record stops at
it.

## Why this is written down before the code

Five landed records already stand on this handover and none of them builds it.
0003 stops the boundary at it. 0005 and 0114 both guarantee that a stream in
flight is the platform's player reading the address handed over here. 0008 measures
the interval that ends at it. 0112 names it as the thing that could move its own
line. So the first playback code answers five decisions at once, and whatever it
does becomes all five of their answers.

The specific thing that gets decided by accident is which source is taken. The
shortest correct-looking code takes the first one the server listed, because that
compiles, it works, and on the machine where it was written every file is already
playable. It ships a stream the device cannot decode to the first device with a
narrower description, and the failure arrives from inside the platform's decoder,
which 0004 has no kind for and the core no visibility into. 0036 already names
that direction for a wrong default and names it as the expensive one.

The second is sending no description on the call, because one is already stored
and it works. That is invisible until a device's capabilities change between two
sign-ins, or until a second client on the same device overwrites the first's, and
both of those are reported as one device sometimes getting conversions it does not
need.

Neither is discoverable from the code afterwards. A ladder that was never written
does not appear in a diff as a missing ladder, it appears as a line taking the
first element of a list.

## Alternatives, and what each cost

The caller picks the source. Cheapest for the core, and a client author knows
their platform better than a rule does. It costs the thing this issue exists for:
eleven clients answer it eleven ways, two clients on one device start the same
item differently, and the contract in #75 then has to carry the whole rule as
prose for a person to implement by hand, which is the duplication this repository
exists to remove.

The server picks and the core takes what it is given. Nothing to write, and the
server has the fuller picture of what it can convert. It costs the preference for
playing a file untouched, which is the one choice that costs nobody anything: what
the server prefers is a server-side default that differs between versions and
between deployments, so the same item on the same device would be converted or not
depending on what the operator upgraded to. It also leaves the core with no answer
at all for a server that ignored the description, because it would have nothing to
compare the answer against.

Ask for a conversion always. One path, no ladder, no ambiguity about which rung
anything is on, and it works everywhere. It costs picture and sound quality on
every play where the file was already playable, it costs the operator's machine a
conversion for every stream in the household, and on a slow link it costs the
start it was meant to protect, because a conversion begins after the server has
started producing bytes rather than before.

Play as it stands always, and convert never. Cheapest on the server, best picture,
and no ladder. It costs the item that cannot be decoded at all, which becomes
unplayable on that device with nothing the core will do about it, and the person is
told to go and re-encode their own library.

Compute the answer from the stored description alone. Fewer bytes on the call and
the server already holds one. It costs staleness between sign-ins and the collision
between two clients sharing a device identifier, and both are failures nobody
reports as this, because what they look like is a device that sometimes gets
conversions it does not need.

Two calls, one to negotiate and one to hand over. It reads as the more honest
shape, and it lets a client show something between them. It costs 0008's interval,
which is superseded rather than implemented, and the gap between the two calls
belongs to nobody while sitting inside the number a person feels.

## What would reverse this

A supported server version is found computing its answer from the stored
description and ignoring one sent on the play call. The rule above then chooses
against a description the server did not read, and this record is superseded by
one saying what the core does with a description it cannot make the server use.

The server stops storing the capability description against the device identifier
alone. Half the argument for sending it on the call rests on that, it was read at
the commit 0036 names, and the check is a re-run of the command quoted there
against a later one.

The ladder is found taking a conversion where the device could have played the
file as it stands, twice. Once is a description that was wrong, which is the
client's. Twice means the ladder is asking the description for something it cannot
answer, and the record is superseded by one that says what the extra input is.

A client is found needing to tell a server refusal from a core refusal in what it
shows a person. The repair is a field on 0004's payload, so that record is
superseded and this one is amended by the pointer rather than replaced.

0112's line moves. That record already names this handover as the thing that could
move it, and a handover needing something from inside the decode is evidence the
line was drawn in the wrong place. Everything here about what the handover carries
is written against the line where it stands today.
