# 0112. Where the platform decoder begins

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #112

## The decision

Video decoding and the presentation of decoded frames are reached through each
platform's own interface and are written nowhere in this project, while
everything above that line, meaning timing, audio and video synchronisation,
subtitle rendering, tone mapping, prefetch, cache, focus handling, the tile
pipeline and all playback policy, is written here.

## The line

    decode and hardware presentation   platform interface, per client
    everything else                    written here, per platform

## What is written rather than taken from a library

Everything above the decoder. Timing, presentation, audio and video
synchronisation, subtitle rendering, tone mapping, prefetch, cache, focus
handling, the tile pipeline, and all playback policy.

That is where the published speed numbers are won or lost, and it is where
clients that already exist miss them. A general-purpose media library will not
meet those numbers on this project's behalf, because meeting them means knowing
what is about to be shown, and that knowledge is exactly what a library does not
have. A library sees a request for the next thing; this project's own layers see
the wall of tiles a person is moving across and what they will reach next.

## What is not written here, and why that is not a compromise

Video decoding. Hardware decode is reachable on no platform except through that
platform's own interface, and those interfaces are not libraries that could be
routed around. They are the interface to the decode block.

| Target | The interface that reaches its decode block |
| --- | --- |
| Android, including Android TV and Fire TV | MediaCodec |
| iOS, tvOS, macOS | VideoToolbox |
| Windows | Direct3D 11 Video Acceleration, through Media Foundation |
| Linux desktop | VA-API, with VDPAU where only that is present |
| webOS | the platform media pipeline, reached through its media element |
| Tizen | the platform media pipeline, reached through AVPlay or its media element |
| Roku | the Video node in the platform's own application framework |

A decoder written here would run in software. Decoding modern 4K material in
software needs several times the compute a television has, and the outcome is
dropped frames, which is the number this project publishes as zero at 60 frames
per second. On a handheld it also ends the battery. This paragraph is a claim
rather than a measurement: no run of a software decoder on a target device has
been made in this repository, there is no code here to make one with, and the
claim rests on the published capabilities of the devices rather than on something
measured here.

The platform interface is therefore not the weaker option. It is the only one
that reaches the target, and choosing it costs nothing that was available.

## Placing a piece of work on one side

One question. Does this work produce or present decoded frames? If yes, it is the
platform's interface, in the client, and nothing about it is written here. If no,
it is written here, once, and used by every client.

The question is deliberately narrow. Work that merely happens near the decoder,
such as deciding which stream to ask for, when to start prefetching the next
item, or what to do when a stream stalls, is not decoding and is not presenting,
so it is written here. That is most of the work, and it is where the difference
between clients would otherwise appear.

Image decoding is on the other side of this line and is inside the core, for the
reason in the record for #3: turning image bytes into pixels is a parse of
untrusted input from a network, which is a security surface rather than a
performance one, and it stops at a bitmap rather than at a display. The two
records name each other so that a reader who finds one finds the other.

## What this record does not decide

Which binding each client uses to reach the interface named for its platform, and
what its capability description declares. Those are per-client decisions and there
is no client board to hold them.

One consequence belongs beside them rather than inside them. A capability
description that understates what the client can play makes the server convert a
file it could have sent untouched: worse picture, higher load on the server, and a
slower start. Playback quality is lost at that description more often than at the
decoder, and it is cheap to get right. #36 owns its shape and says the client
supplies what it can actually decode.

It also does not decide what the handover across the line looks like. That is
#111, and the interval that ends at it is measured under #63.

## Why this is written down before the code

The cheap reading of "share what can be shared" and the cheap reading of "write it
properly for each platform" point in opposite directions here, and both sound
right. Whichever is nearest when the first playback code is written becomes the
answer, and the answer is then defended with the code rather than with an
argument.

Placed wrongly in one direction, a decoder is written here and the project spends
a year discovering per device that it cannot meet a number it published. Placed
wrongly in the other, everything above the decoder is written per platform too,
which is eleven timing implementations and eleven answers to when a stream is
considered started, and the number in #63 then means something different on each
of them.

The written record is also what stops the line moving by accident. A single piece
of work that sits just above the decoder is always easier to put on the platform
side while somebody is already there, and after three of those the line is
somewhere nobody chose.

## Alternatives, and what each cost

A software decoder written here, shared by every client. One implementation, one
set of behaviours, and every platform behaves identically. It costs the compute
argument above: it does not reach the published frame number on the hardware this
project targets, and it ends a handheld's battery. Identical behaviour on every
platform is worth nothing when the behaviour is dropped frames.

A cross-platform media framework taken as a dependency, wrapping each platform's
decoder. It reaches hardware decode, it is maintained by somebody else, and it
covers the platforms. It costs the layer above the decoder, which such a framework
also supplies and which is where the numbers are won, so the choice is either to
accept its timing and prefetch behaviour or to fight it. It is also a large
dependency on every target, which is the question #103 exists to ask.

Everything above the decoder written per client too, with only a specification
shared. Each client uses whatever its platform makes natural, and nothing has to
cross a boundary. It costs the numbers being comparable at all, and it is the
duplication this repository exists to remove.

The line drawn at the container parser instead, with demuxing on the platform
side. Fewer moving parts on this side and the platform interfaces often want a
container anyway. It costs the ability to decide what to prefetch and when to
start, because those decisions need to know what is in the stream, and it hands
the most attacked parse on the path to whichever component each platform happens
to use.

## What would reverse this

A target appears whose only route to its decode block is a library rather than a
platform interface. The table above then has a row that is not a platform
interface, and the argument that these are not libraries that could be routed
around stops being true everywhere. The record is superseded by one that says
where a library is accepted and what it is held to.

Software decoding becomes adequate on the lowest target this project supports,
measured on that device rather than assumed. The compute argument above is the
whole of the reason, so if it stops holding, the placement stops being forced and
is worth retaking.

Work that is neither producing nor presenting decoded frames turns out to be
unwritable on this side for a platform reason, twice. One is a design mistake in
that piece of work. Two means the one question above is placing things it cannot
place, and the line is drawn somewhere a question can actually reach.

The handover in #111 turns out to need something from inside the decode that the
platform interface will not give up, for instance an exact presentation timestamp
for the first frame. The measurement in #63 would then end at a point nothing can
observe, and this record is superseded by one that moves the line to where the
observation is possible.
