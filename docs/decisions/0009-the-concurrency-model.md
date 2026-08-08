# 0009. The concurrency model, and what the core promises about threads

Date: 2026-08-08

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #9

## The decision

There are two kinds of call in this core and no third kind: a call that can wait is
asynchronous, cancellable and never runs on the thread that made it, and a call
that cannot wait returns from memory the core already holds and is safe from any
thread. The core owns its own threads, in two named lanes created when the core is
created and stopped when it is stopped, so that the promise about never decoding
where a client draws is the core's to keep and to prove rather than eleven clients'
to get right.

The spelling of all of this depends on the language, which is #11. What is decided
here are the properties, and a language that cannot carry them fails the choice in
#11 rather than weakening this record.

## The public interface

Every call that can wait on something outside the process is asynchronous. It
returns a handle at once, delivers its outcome later, and can be cancelled. This
covers every request to a server, every decode, every read or write through the
byte store in #40, and every read or write through the secret store in #33.

Every call that cannot wait returns its answer immediately from state the core is
already holding. This covers reading a session's identity, reading what the core
last observed about a server's reachability, reading the age of a cached entry the
core has already loaded, and cancelling. These calls take no cancellation handle,
which is how a reader tells the two kinds apart without a document: the presence of
a cancellation handle is the mark of a call that can wait.

No call is both. There is no synchronous variant of an asynchronous call and no
blocking wrapper in the core, because a blocking wrapper is the thing a client calls
on the thread it draws on.

### A client whose platform has a different concurrency model

The core's asynchronous shape is completion-based: a handle, an outcome delivered
once, and a cancellation that is itself immediate. A client on a platform with
coroutines, with promises, with an event loop, or with nothing but threads adapts by
wrapping that one shape, and the wrapper is small because there is one shape rather
than one per call.

What a client must supply for the adaptation to be correct is a completion sink: an
object the core hands outcomes to. The core will not guess a platform's main thread,
because it cannot know one exists. Where a client needs outcomes on a particular
thread, the sink is where the marshalling happens, and it is written once rather
than at every call site. A client that supplies no sink receives outcomes on the
core's waiting lane, which is correct for a client that does no drawing in its
completions.

A client that genuinely has only threads may block its own thread on a handle. The
core provides the handle; it does not provide the blocking call, because a blocking
call in the core's own surface is one a client will reach for from the wrong thread.

## The core owns threads

Two lanes, and nothing else. No thread is created outside them, no work is posted to
a platform's shared pool, and no timer thread exists on its own.

The waiting lane carries everything that is waiting on a server, and it runs the
completions the core delivers. It is sized to the connection limit the transport in
#27 sets, one waiter per permitted connection, so that no request ever waits for a
lane before it waits for a connection. Sizing it any larger buys nothing, because
the thing being waited on is already bounded.

The processing lane carries everything that costs a processor: image decoding, any
parse over a body large enough to matter, and the hashing that produces a cache key.
It is sized to the host's reported usable processor count less one, with a floor of
one. The subtraction is there so that a fully occupied processing lane still leaves a
processor for whoever is drawing, which is the only reason this lane exists as a
separate thing. A client may set the size and may not set it to zero.

Who starts them. The core-creation call in #115, and nothing else. A core that has
not been created owns no threads, which means a client can construct one, decide
against it, and discard it without having started anything.

What happens when the host wants them stopped. The stop call in #115 does not
return until both lanes have stopped. It cancels every outstanding call first, so
the guarantee below about no further delivery applies to all of them, then it waits.
It is the one place in the core where something waits on a thread, and it is
deliberate: a stop that returned before its threads had stopped would let a host
unload the process underneath them, and a host that suspends the core needs to know
that when the call came back, nothing of the core's is still running.

The stop call is bounded. If a lane has not stopped by the bound, the core reports
that it could not stop rather than waiting indefinitely or claiming success. What
the bound is belongs to #115, and this record requires only that a failure to stop
is reported as one and never as a stop.

## The guarantee about the calling thread

No call the core exposes performs a network wait, a decode, a read or write through
the byte store in #40, or a cache-key hash on the thread that called it. Those four
are the whole list, and the list is short on purpose: a guarantee with exceptions is
one a client cannot rely on at a call site.

Which calls may block. The stop call in #115, for the reason above. Nothing else. A
call that cannot wait may take a lock the core holds, and the core never holds one of
its own locks while calling out to client-supplied code, so the longest such call is
bounded by the core's own work rather than by anybody else's.

How a client verifies this rather than trusting it. Two routes, and neither is a
sentence in a document.

The core's own suite runs under a detector that reddens when a concurrency claim is
broken, which is #117. What it is given to detect is the thread identity every stage
of every call ran on, recorded by the core in its own test configuration, and an
assertion that no identity on which one of the four kinds of work ran is ever an
identity a call entered on.

The conformance suite in #76 asks the same question from the client's side. A client
runs it, the suite calls the core from a thread it names, and it fails if the core
does any of the four on that thread. This is the route that matters, because it is
the one a client author can run against the core they actually linked rather than
against the one the suite was written beside.

## Cancellation

Every asynchronous call is cancellable. A tile that scrolled off a screen is work
nobody wants finished, and a wall of two hundred tiles in #53 is the case that makes
this structural rather than a nicety.

Cancelling is one of the calls that cannot wait. It returns at once. So the honest
answer to what has stopped when cancellation returns is: nothing necessarily has, and
the guarantee at that moment is about delivery rather than about work.

### What a caller may assume the moment cancel returns

No outcome for that call will ever be delivered. Not a success, not a failure, not a
progress report. A completion already posted to the sink and not yet run is
discarded.

Nothing that call would have produced will reach the cache. A response body already
received is dropped rather than written, and a decoded image already produced is
freed rather than stored.

Cancelling again is a no-op, and cancelling a call that has already completed is a
no-op. Neither is an error and neither races: the outcome is delivered exactly once
or not at all, and cancel is what decides which.

A cancelled call is not a failure of the thing it was doing. It has its own outcome,
distinct from every failure, and #37 maps nothing onto it. Whether the vocabulary in
#4 carries a member for cancellation is #4's to decide; what this record requires is
that a cancelled request is never reported as a request that failed, because a
client that shows an error for work it cancelled itself is showing an error nobody
caused.

### What a caller may not assume the moment cancel returns

That the work stopped. Each of the following continues, and each is deliberate.

A request already sent to the server may be received and acted on. A cancelled write
is not an undone write: if the core had already told the server a position, that
position is recorded there. Cancellation is a statement about what the caller wants
back, never about what the server has already been told.

Bytes already in flight are read and discarded rather than abandoned in the socket,
because a connection left with unread bytes cannot be reused, and reuse is what #27
exists for.

A decode already inside a decode step runs to that step's end. A decoder is not
interruptible at an arbitrary instruction, so the core cancels between steps and not
inside one. The bound is therefore one decode step, not zero.

A read already begun through the byte store in #40 completes, because the store is
the client's code and the core does not get to interrupt it.

That anything the caller lent the call is free to reuse. Buffers, sinks and stores
the caller supplied may still be touched.

### What a caller may assume once it waits on the handle

The handle can be waited on after cancelling, and when that wait returns the core is
no longer touching anything: not the caller's buffers, not the byte store, not the
secret store, not the sink. This is the point at which a caller may free what it
lent, and it is a different point from the one above.

Splitting the two is the whole of what this section is for. A single cancel that
claimed both would be claiming that a decode step and a client's own store call had
stopped, which is not something the core can make true.

## Reentrancy and thread safety, by kind

No type exists in this repository yet, so this is stated per kind of object. The
names arrive with #11 and #13, and each type carries the statement for its kind
where a reader will meet it.

The core handle is safe from any thread, always, including while it is being stopped.
It is the only object with no conditions on it.

A session handle is safe from any thread. Calling on a session while another thread
signs it out is defined rather than racing: the call either goes out under a valid
token or fails with the signed-out outcome, and never goes out under a token that has
been discarded.

A query result is immutable once it has been handed back. There is no shared mutable
state to protect, and the core keeps no reference through which it could change one.

A decoded image is immutable, and its bytes belong to the caller from the moment they
are handed over. The core does not read them again.

The byte store the client supplies in #40 may be called from either lane and
concurrently, including for two entries at once. Its interface says so, because a
client that assumed single-threaded access would corrupt its own storage rather than
producing a failure the core could report.

The secret store the client supplies in #33 is called from the waiting lane only, and
never concurrently for one session. A client may implement it without locking. This is
the deliberate opposite of the byte store, and the reason is that a keychain call is
rare and a platform keychain is the place a client is most likely to write something
naive.

The diagnostics sink the client supplies in #100 may be called from any lane, at any
time, and concurrently. It must be safe for that, it must not block, and it must not
call back into the core. The last of the three is the deadlock, so the interface
forbids it rather than documenting it.

A completion running in the sink may call back into the core, because the core holds
no lock of its own while calling out. It must not block, because while it runs it
occupies a lane.

## Why this is written down before the code

This is the decision that reaches every interface in the repository, so it is the one
that gets made four times if it is not made once. The four are predictable: the first
call site is blocking because it was easy to test, the second takes a callback because
it needed one, the third takes both because a reviewer asked, and the fourth takes a
runtime handle because by then there was one to take. The interface is then
un-unifiable without changing every client that had started.

The guarantee about the calling thread has to exist before the code for a sharper
reason. It is not a property that can be added: a core that decoded on the calling
thread for a year has clients whose completions assume they are on their own thread,
and moving the work moves the assumption underneath them.

## Alternatives, and what each cost

A blocking interface only, with each client wrapping it. The smallest core, the
easiest to test, and no lanes to own or stop. It hands the whole question in this
record to eleven clients, which is eleven answers, and it makes the promise about
never decoding where a client draws unenforceable by anything the core can run. It
also makes the core's share of the number in #62 depend on how each client wrapped
it.

A callback interface with no lanes of its own, running completions on whichever
thread the work finished on. Nothing to size and nothing to stop. Completions then
arrive on a thread the client cannot predict, which in practice means every client
marshals defensively at every call site, and the ones that forget are the bug that
only appears on one platform.

A runtime the client must host, with the core scheduling onto it. Cheapest inside the
core and the most expensive at the boundary: it decides part of the client's
architecture, it needs the runtime present on every target including a television,
and it turns the binding layer into something with its own scheduling semantics to
test.

Taking an executor from the client instead of owning lanes. Attractive because it
lets a host that owns its scheduling stay in charge, and it is the option to revisit
first if the reversal below happens. Its cost is that the calling-thread guarantee
becomes conditional on somebody else's executor, so the core can no longer prove it,
and the proof is the reason the guarantee is worth stating.

Offering both a blocking and an asynchronous surface. Meets every client where it is,
and doubles the surface the conformance suite in #76 has to cover, doubles what #117
has to detect, and guarantees that the blocking surface is the one a hurried client
calls from the thread it draws on.

## What would reverse this

A target platform forbids an application creating its own threads, or bounds them so
tightly that two lanes is not affordable. The core then cannot own lanes, and the
executor option above becomes the answer, with the calling-thread guarantee weakened
to a requirement on the supplied executor and stated as a requirement rather than a
promise.

The detector in #117 cannot be run on the toolchain chosen in #11. The calling-thread
guarantee then has no route to verification and becomes an unproven claim, which is
not a state this record accepts: either the means changes or the guarantee is
withdrawn and written as a claim.

The measurement in #62 or #63 shows the handover between lanes costing more than the
guarantee is worth on the slowest supported target. The subtraction that sizes the
processing lane, or the separation of the lanes altogether, would then be the thing
paying for a property nobody can perceive.

The delivery guarantee on cancel turns out to be unimplementable without holding a
lock across a call into client code. The rule against holding a lock while calling out
is the more important of the two, so the cancel guarantee would be the one that
weakens, and it would say plainly that a completion may still arrive after cancel
returns rather than leaving a caller to discover it.
