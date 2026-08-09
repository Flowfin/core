# 0102. The clocks every deadline is measured against

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #102

## The decision

The core reads three clocks and never a fourth: a steady clock that only moves
forward and only measures intervals inside one run, an elapsed clock that does the
same across a suspension, and the device wall clock, which is read only to
interpret a moment the server also believes in; a duration is never measured on
the wall clock, a token is never refused on the device's own reading of it, and
every rule below fails towards stale and towards asking the server rather than
towards trusting a number the device produced.

## The three clocks

    steady     Moves forward only, at a rate nothing corrects, from an origin
               with no meaning. Measures an interval between two events in one
               run of the core. Not comparable across runs, not comparable
               between devices, and on several platforms it stops while the
               device is suspended.

    elapsed    The same properties, and it keeps counting while the device is
               suspended. Measures an interval that has to survive a device
               going to sleep. Still meaningless as a moment, and still reset by
               a restart.

    wall       What the device believes the time is. Moves in both directions,
               by a correction, by a person setting it, and by a television
               coming up from a power cut believing it is 1970. The only clock
               that names a moment two machines can talk about.

A duration is measured on `steady` or on `elapsed`, never on `wall`. A moment is
named on `wall`, and only where something outside the device also has an opinion
about it.

The split between `steady` and `elapsed` is which side of a suspension the
interval has to survive. A request timeout is `steady`: a device that suspended
mid-request has no request left, and counting the sleep against the timeout would
turn every wake into a timeout. A queued action waiting for a server to come back
is `elapsed`, because the wait is real time and a suspension does not shorten it.
#115 owns what suspension does to the core and is the issue this distinction
exists for.

Where a duration must survive a restart of the process, none of the three works,
because both monotonic clocks reset. That is the cache case and it is answered
below by anchoring on the server rather than by finding a fourth clock.

## What is on which clock

| What expires | Clock | Why |
| --- | --- | --- |
| A request timeout, and the read timeout inside it | `steady` | An interval in one run, and a suspension should not consume it. |
| Retry backoff between attempts | `steady` | Same. |
| The interval a measurement span reports | `steady` | The spans in #61 are in-run durations, and a correction mid-span would otherwise produce a negative one. |
| A queued action waiting for the server to return | `elapsed` | The wait is real time and continues while the device sleeps. |
| The cadence progress is reported on | `elapsed` | Same, and a device that slept through three cadence ticks owes one report rather than three. |
| A cache entry's age | anchored on the server, read below | It has to survive a restart, which no monotonic clock does. |
| A token's stated expiry | `wall`, and never as an authority | The server has an opinion about this moment. The device's reading of it is a hint. |
| Clock skew between device and server | both, read below | A moment compared against a moment, bounded by an interval. |

A contributor taking a new expiry that this table does not name asks two
questions, in this order. Does anything outside this device have an opinion about
when it happens? If yes, it is a moment, it is on `wall`, and the paragraph on
tokens below says how much weight the device's reading carries. If no, it is a
duration; does it have to survive a suspension? Yes is `elapsed`, no is `steady`.
Nothing needs a fourth answer, and a case that seems to is the cache case, which
is answered by anchoring rather than by a clock.

## What the core does when the device and the server disagree

Skew is measured on every response that carries the server's own time, not once
at sign-in. A device clock can be corrected in the middle of a session, and that
correction is the case this whole record is about, so a single measurement at
sign-in would be a measurement taken before the interesting thing happened.

It is measured with its uncertainty, because a bare subtraction is wrong by the
round trip. A `steady` reading is taken immediately before the request goes out
and immediately after the answer comes back, which gives the interval the server's
stated moment lies somewhere inside. The offset is therefore known to within that
round trip and is kept as a value with a bound, not as a number.

Nothing acts on a skew smaller than its own uncertainty. A measured offset of
200 ms on a request that took 400 ms is not evidence of anything.

Two thresholds, and both only cause a report:

Above 60 seconds, the core reports the offset and its bound as a diagnostic event
under #100. Sixty seconds is chosen because it is comfortably above any plausible
round trip on the paths this core takes, so a report at that level is about a
clock rather than about a slow network.

Above 24 hours, the core reports the same thing under a second identity, because
that magnitude is not a drifting clock. It is a clock that was never set, and it
is worth being able to find in a diagnostic stream without reading values.

The core changes no behaviour at either threshold. It does not refuse to run, does
not correct the device clock, and does not start rewriting moments. It reports,
and everything else in this record is already written so that a wrong device clock
cannot produce a wrong outcome. The skew value is available to a client that wants
to say something about it, which is where a sentence about a wrong clock belongs
under #3.

## A token is never refused on the device's own clock

The core never treats a stated expiry as grounds to refuse a call. The session
ends when the server says so, which arrives as `not-authenticated` from #4 and is
handled by #34 and #35.

The stated expiry is used for one thing: scheduling a renewal before the token is
likely to be rejected, so that the rejection usually does not happen during
something a person is watching. Where the device clock is wrong, the schedule is
wrong, and the cost of that is one extra round trip or one recoverable rejection.

The other direction has no such cost. A device that comes up believing it is 1970
sees every token as not yet valid; a device set forward sees every token as
expired. On either, a core that refused locally would throw away a working session
on every start, and the person would see a sign-in prompt for a server that would
have accepted them. That failure is silent about its cause, because nothing in the
trace says a clock was compared, which is the exact shape #102 was opened against.

Where the stated expiry has passed by more than the skew bound, the core may renew
before making the call rather than after being rejected. That is an optimisation
in the same direction as the schedule, and it still never produces a failure: if
the renewal is refused, the original token is still used and the server still
decides.

## What a cache entry's age is measured against

An entry stores two moments at write: the server's own stated time from the
response, and the device's `wall` reading at the same instant. The difference
between them is the skew at write, kept with the entry.

Age at read is the device's `wall` now, minus the entry's stored device moment,
corrected by the difference between the skew now and the skew at write. When the
device clock moves between the write and the read, that correction removes the
movement, because both readings moved with it and the server's did not.

When there is no current skew measurement, which is the offline case, the
correction is unavailable and the age is the uncorrected difference.

Two guards make the failure direction one-way, and they are the point of this
section:

An age that computes as negative is treated as an age past every threshold, not as
a fresh entry. A clock that moved backwards therefore makes entries stale early,
never fresh forever.

An age larger than a sanity bound is treated the same way. A device that jumped
forward by a decade has no entries worth trusting, and the cheap answer is to
treat them all as stale and let the server confirm.

Both guards fail towards asking the server. That is the correct direction because
a needless request costs a round trip while a permanently fresh entry costs a
person seeing something that is not there any more, with nothing in the system
that will ever correct it. #43 owns the thresholds and reads this record for what
the age is measured against.

## What a test may assume

All three clocks reach the core through one injected source. Nothing in the core
reads a platform clock directly, which is what makes a timeout test take
microseconds rather than seconds and is part of the birth requirement in #20.

A test may set and move `wall` freely, in both directions and by any amount. That
is the whole point: the 1970 television and the person setting the date forward are
cases that have to be reachable in a suite.

A test may advance `steady` and `elapsed` by any amount, and may advance them
independently, which is how a suspension is expressed: `elapsed` moves, `steady`
does not.

A test may not move `steady` or `elapsed` backwards. The fake source refuses it
rather than allowing it, and that refusal is itself tested. The shipping code
treats both as never going backwards and has no branch for it, so a suite able to
move them backwards would be proving behaviour that does not exist in anything
that ships, and the proof would look exactly like a real one.

No test waits on real time. A test that sleeps is measuring the machine it ran on,
and #21 supplies the controlled clock so that it does not have to.

## Why this is written down before the code

Four separate things on this board expire, and every one of them is a comparison
between two times that nobody wrote down. Left to the code, each gets whichever
clock was nearest when it was written, which is the wall clock, because that is
the one every platform makes easiest to reach.

The failures that follow are not read as clock failures. A valid token thrown away
on every start reads as a sign-in bug. A stale entry that never expires reads as a
caching bug or as a server that did not update. An interval measured across a
correction reads as a performance anomaly, and it is the one that will be
investigated longest, because a span that reports minus four seconds is not
something anybody looks for.

None of it can be repaired by inspection afterwards. A trace does not say which
clock produced a number, so the evidence that would identify the cause is exactly
what is missing. Writing the assignment down first means a wrong one is a wrong
line in a table rather than an archaeology exercise.

The cache anchoring is the part that cannot be retrofitted at all. Entries written
without the server's stated time cannot have their ages corrected later, because
the information needed for the correction was never stored. A cache that has to be
discarded wholesale on the day this is noticed is the cheap version of that
mistake.

## Alternatives, and what each cost

One clock for everything, the wall clock. Simplest to write, simplest to read in a
log, and correct on any device whose clock is right. It costs every case in this
record, and the devices it is wrong on are televisions and handhelds, which is
what this project runs on.

Monotonic everywhere, with no wall clock at all. Immune to every correction, and
nothing can jump. It costs the ability to talk to the server about a moment at
all, and it costs the cache, since a monotonic clock resets at every restart and a
cache that forgets its ages on restart is a cache that revalidates everything on
every start, which is the cold-start number in #46.

Refusing a token on the device's reading of its expiry. Saves a round trip on
every expired token and is what most clients do. It costs the wrong-clock device
completely, and it costs it in the direction of a sign-in prompt, which is the
most expensive thing to show a person who did nothing wrong.

Correcting the device clock, or keeping a corrected clock inside the core and
using it everywhere. It removes the skew problem at the source and every
comparison becomes simple again. It costs a core that has an opinion about the
device's time, which is a platform concern the boundary in #3 places outside, and
it makes every duration depend on a correction that can itself be wrong.

Measuring skew once at sign-in. One measurement, no per-response cost, and it
catches the clock that was already wrong. It costs the correction that happens
mid-session, which is the one that produces the confusing report, since everything
before it looked right.

Storing the age as a countdown decremented while the core runs. No clocks at all
in the read path, and immune to any jump. It costs every moment the core is not
running, which is most of them, and it makes the cache's behaviour depend on how
often the client was started.

## What would reverse this

The oldest server line chosen in #1 entry 3 turns out not to state its own time on
responses, or to state it at a granularity coarser than the thresholds here. The
skew measurement then has no input, the cache anchoring loses its reference, and
this record is superseded by one built on whatever that server actually supplies.

A platform is found where `elapsed` cannot be read at all, so a suspension is
invisible to the core. The two-monotonic-clock split is then unimplementable
there, and the record is superseded by one saying what the core does on a platform
that cannot tell it how long it was asleep, rather than leaving that to whoever
hits it.

A case appears that is neither a duration nor a moment under the two questions
above, twice. One is a case placed wrongly. Two means the question pair is the
wrong tool and something more than three clocks is being asked for.

The renewal scheduling in #34 is measured to fire so far from the right moment on
skewed devices that rejections during playback become common, which is the
condition #35 exists for. The stated expiry would then be doing harm as a hint,
and this record is superseded by one that either drops the schedule or takes the
skew correction into it.
