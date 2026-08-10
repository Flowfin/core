# 0115. Creating the core, stopping it, and a host that suspends it

Date: 2026-08-10

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #115

## The decision

Creation is one call that takes every implementation the core wants from a client,
any of which may be absent, and it reaches no network and no store while it runs;
stopping is one call that cancels everything, waits for both lanes, and is true
when it returns or reports within a bound that it could not stop, after which the
core is finished rather than restartable; and suspend and resume are a separate
pair that keeps what a stop discards, where a resume assumes neither that the
token is valid, nor that the network is the one it left, nor that a connection it
held is still usable.

## Creation

One call, and everything the core wants from a client is an argument to it. The
byte store in 0040, the secret store in 0033, the diagnostics sink in 0100. Each
may be absent, and what each absence means is decided in its own record and is not
repeated here.

A capability answer says which are present. That is one call that cannot wait in
the terms of 0009, and it exists because three separate absences produce a core
that works in three different reduced ways, and a client that cannot ask is a
client that cannot explain any of them to an operator.

Creation reaches nothing. It opens no connection, resolves no name, reads no
entry out of the byte store, and reads no secret. A core that has just been
created has started its two lanes, as 0009 fixes, and has done nothing else.

That is the part worth deciding rather than inheriting, because a creation call
that restored the last session would be convenient in exactly one place and wrong
in four. It could fail for a reason that has nothing to do with construction, so a
client would need a failure path around a call that is not supposed to have one.
It would put a store read and a network wait on whichever thread constructed the
core, which is the guarantee 0009 exists to make. It would make the cold start
measured in #46 depend on where the constructor happened to be called. And it
would make a core that a client constructed and then decided against into
something that has already touched a keychain.

Restoring a session is therefore a call a client makes, naming the session it
wants, which is the shape 0033 already needs since its store cannot be asked what
it holds.

## Stopping

One call, and it is the one place in the core that waits on a thread. 0009 fixes
the shape: it cancels every outstanding call first, so nothing further is
delivered, then it waits for both lanes, and it does not return until they have
stopped. This record fixes what 0009 left here.

The bound is a value the client may set, and its default is two seconds. Two is
chosen so that a stop called from inside a platform's own termination callback
returns while that callback still has time left. The windows those platforms
actually allow are a claim rather than a measurement: nothing in this tree runs on
a platform, and no command in this repository produces them. A client that knows
its own window sets the bound rather than accepting a default chosen against a
claim.

The bound has a floor it cannot be set below, and the floor is not a preference.
0009 already fixes two things the core cannot interrupt: a decode runs to the end
of its current step, because a decoder is not interruptible at an arbitrary
instruction, and a read already begun through the byte store completes, because
the store is the client's own code. So a bound shorter than those cannot be met by
any implementation, and a bound a caller can set below what is achievable produces
a stop that always reports failure, which teaches a client to ignore the report.

On expiry the core reports that it could not stop and names which lane did not.
0009 already refuses reporting that as a stop, and this record adds what the core
is afterwards: finished. It accepts no new work. Every call made after a stop was
requested fails with `cancelled` from 0004, whether the stop succeeded or timed
out.

`cancelled` is an imperfect fit and the record says so rather than growing the
vocabulary. Its meaning in 0004 is that the caller asked for this to stop and it
stopped, and a call made after the caller asked for a stop is close enough to that
to not be worth a sixteenth kind, which 0004 fixes as a change to that record and
to every client. What makes it tolerable is that the condition is entirely the
caller's own doing: nothing but the client's own stop puts a core into this state.

A stop is idempotent. A second one returns at once with the same outcome.

A stopped core is not restartable, and creating a second one is the answer. A
restartable core would need a rule for what survives a stop, and the honest list of
things that would need one is long: the lanes in 0009, the capability answers, the
correlator salt in 0071, whatever the transport in #27 was holding. Each of those
is a decision, none of them has a reason to exist, and together they are a second
lifetime that would be tested far less than the first.

## What a stop owes work that was never delivered

Nothing beyond leaving it where it already is. The durable queue in #47 survives
the process by construction, so a stop neither drains it nor discards it.

A stop that tried to deliver would be a stop that waits on a server, which the
bound above cannot hold and which contradicts what 0009 makes the stop wait on,
namely its own lanes. A person closing an application on a train would then wait
for a network that is not there.

A stop that discarded the queue would throw away somebody's actions at the moment
they are least recoverable, which is the failure #47 exists against.

So there is one mechanism and it is #47's, which is 0005's rule about two
mechanisms for one promise applied here: the second one is always the one that
gets tested less.

## Suspend and resume

A separate pair, and the difference from a stop is what is kept. A stop ends the
core. A suspend is the host saying the process is about to be set aside and will
come back.

On suspend the core stops its own scheduled work: the recovery schedule in #45 and
the reporting cadence in #57. Outstanding calls are not cancelled, because the
host has not asked for that and the caller still wants its answer. Nothing is
flushed, because 0068 and 0100 already fix that the core retains nothing to flush.

On resume, three things the core may not assume, and each of them has a concrete
consequence rather than being a caution.

Not that the token is still valid. The core does not refuse it either, which is
0102's rule: a session ends when the server says so, and the device's own reading
of an expiry is a hint. So a resume proceeds and lets the first call find out,
with 0005's rejection path already written for exactly that.

Not that the network is the one it left. Every connection the transport in #27 was
holding is discarded on resume rather than reused. A socket that survived a
suspension onto a different network is one that fails on its first byte, and 0007
would read that failure as a slow server and start counting toward an abandonment
that describes nothing. Discarding costs one handshake on the first request after
a wake, which is inside what #62 allows for a first screen.

Not that elapsed time is what the device's clock makes it look like. 0102 fixes
which clock every interval is on, and the split between `steady` and `elapsed` is
this pair: a request timeout is `steady` so that a suspension does not consume it,
and a queued action's wait is `elapsed` so that a suspension does not shorten it.
This record adds nothing to that and names it so the consequence is not
rediscovered as a defect.

A resume performs no work of its own beyond restarting the schedules it stopped.
It does not refresh, does not revalidate the cache, and does not renew a token
ahead of a call, because each of those spends a person's connection at the moment
they have just picked the device up and are waiting to see something.

## What this does not decide

Which implementations exist at all and what each absence costs. 0033, 0040 and
0100, one each.

What a sign-out removes and how several sessions are held. #114.

The connection limit, the timeouts, and how connections are held between requests.
#27.

What the queue does with a restored entry's wait, which its own comment already
raises. #47.

The bound on the recovery schedule that a suspend pauses. #45.

## Why this is written down before the code

0009 already carries most of the shape and deliberately left three numbers and one
question here: the bound, what a failed stop leaves behind, what a resume may
assume, and where undelivered work goes. Each of those is answered by whichever
call site meets it first if it is not answered now, and three of the four answers
that arrive that way are wrong in the same direction, which is towards making the
call look like it succeeded.

A stop with no bound waits forever on a client store that is not going to return,
and it does so inside a platform's termination callback, so the symptom is an
application the platform killed rather than one that stopped. A stop that reports
success after a timeout is the negative disclosure turned positive, which 0009
already refuses in its own words. A resume that reuses connections produces a
first request after every wake that fails for a reason nothing in the trace
explains, and it presents as a slow server, which is the report that gets
investigated longest.

The fourth, a stop that flushes the queue, is wrong in the opposite direction and
is the one that looks most responsible. It is a person waiting on a network
because they closed an application.

None of this has happened here, because there is no core to create in this tree.

## Alternatives, and what each cost

Creation that restores the last session. It is what a client author expects and it
removes a step from every client. It costs a construction call that can fail for
network reasons, a store read on the constructing thread, a cold-start measurement
that depends on where the constructor was called, and a keychain touched by a core
somebody constructed and discarded.

A stop that returns immediately and reports later. It never blocks, which reads as
the modern answer, and it removes the one blocking call from the interface. 0009
already refuses it and gives the reason: a host that unloads the process on that
report has no way to know whether anything of the core's is still running.

An unbounded stop. It is simpler and it is always eventually true. It hands a
client's own slow store implementation the power to hang the host's termination
path, at which point the platform kills the process and the operator sees a crash.

A restartable core, so that a suspend on a platform that kills threads can be
expressed as stop and start. It matches what a couple of platforms actually do to
an application. It costs a rule for what survives a stop, per piece of state, and
a second lifetime that every test would have to cover twice or leave uncovered.

Suspend and resume folded into stop and create. One pair instead of two, and
nothing to explain. It costs everything a resume keeps: the sessions, the
transport's knowledge of what is reachable, the cache the core has already loaded,
and it turns an ordinary phone backgrounding into a cold start, which is the
number #46 exists to protect.

Keeping connections across a suspension. It saves a handshake on every wake and is
correct whenever the network did not change. The case it fails is the one it was
kept for, a device that moved, and it fails as a slow server rather than as a
network change.

A sixteenth error kind meaning the core has been stopped. It would say exactly
what happened. 0004 fixes the price of a sixteenth kind as a change to that record
and to every client, and the condition is one only the caller's own stop can
produce.

## What would reverse this

A platform is found whose termination window is shorter than the floor the bound
cannot go below. Then the stop this record describes cannot be completed there at
all, and the replacement says what the core does on that platform instead of
implying a stop it cannot finish.

A client is found needing a core to outlive a stop, twice, with what it was trying
to keep. One is a client that should create a second core. Two is evidence that
creation is expensive enough to be worth avoiding, which is a measurement from the
harness in #65 rather than a preference, and the replacement describes a restart.

Discarding connections on resume is measured against #62 and found to cost more
than the failures it prevents. That is a real trade with numbers on both sides, and
the harness in #65 is where they would come from.

#47 decides that a restored queue entry's wait is anchored on something a stop has
to write. Then a stop does owe the queue an act, this record's answer of leaving it
alone is wrong, and what replaces this record says what a stop writes and what a
stop that timed out leaves behind.
