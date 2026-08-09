# 0068. The data locality position

Date: 2026-08-09

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #68

## The decision

Personal data stays on the operator's own host and on the devices they use, the
core opens no connection to a host the operator did not configure, there is no
diagnostics, statistics or improvement route out of a device at all, and reaching
a second server is a per-server act a person performs rather than a default that
can be turned off.

## What counts as personal data here

Listed concretely, because a reference to a statute does not tell a contributor
whether the field they are adding is in the set.

The server address as typed, and the resolved identity of the server behind it.
Both say where a person's library lives, and a self-hosted address frequently
says where the person lives.

The account identifier and the account name.

The session token, which is the only secret in #5 and is also the credential that
reaches everything else in this list.

The device identity and the device profile from #36. The profile says what
hardware somebody has.

Library contents. Titles, identifiers, artwork, and any metadata that came back
about what is on that server.

Playback positions, what was played, when it was played, how far it got, and
whatever #60 decides counts as watched. This is the entry in the list closest to
the sensitive kind, because a viewing history says a great deal about a person.

The queue of actions taken while the server was gone, in #47, because it is the
same data with a delay on it.

Diagnostic events and measurement spans, wherever their fields carry any of the
above. A span naming an item identifier is personal data in the same way the item
is.

A contributor deciding about a field this list does not name asks one question.
Could two people running the same build against the same server hold different
values here? If yes, it is personal data under this record, and the rules below
apply to it without further argument.

## The rule, in a shape a check can refuse

An outbound connection to a host the operator did not configure is a defect.

Not a policy, not a preference, and not a thing that is acceptable behind a
setting. The set of hosts the core may contact is enumerated in #69, it is built
from what the operator supplied, and everything outside it is refused. #70 is the
test that fails when the core reaches a host nobody configured, and #73 is the
proof that there is no telemetry, no analytics and no crash reporting to reach
one with. Those two are the mechanisms this position rests on. Until they exist,
this record is a rule nothing refuses, and that is a statement about today rather
than an intention about later.

Two consequences worth stating so they are not rediscovered as bugs.

There is no host in the set by default. A core with no configuration reaches
nothing, which is what makes #70's failure unambiguous: any host at all in a run
that configured none is the defect.

Nothing in the core resolves a name that came out of a response body. A server
answering with an address somewhere else does not thereby become a route out of
the device, and an artwork address pointing at a third host is not followed.

## Federation is an act, not a setting

Reaching a second server is #72, and it is deliberate in the strong sense: a
person adds that server, with its address and its credentials, the same way they
added the first. Nothing is enabled by a default, by a discovery protocol, or by
a server telling the core about another server.

An opt-out default would not satisfy this position and it is worth saying why
rather than treating it as obvious. The person most exposed by data leaving their
host is the person least likely to change a default, because changing it requires
knowing it exists. A default that can be turned off protects whoever already
understood the risk, which is the population that needed the protection least. It
also makes the property unprovable: a check can show that the core reaches no
unconfigured host, and it cannot show that every operator made the choice
knowingly.

## What is kept on the device, for how long, and what removes it

The core writes nothing to a location it chose. Every byte goes through a store
the client supplied, which is #40 for the cache and #33 for the secret, so this
section is about what the core asks to be kept rather than about files it owns.

The session token is held in the secret store and is removed when the session
ends. Signing out removes it, which is #114, and that removal is expressed in the
keying from #41 so that ending one session leaves the others alone.

Cache entries are kept until they are evicted under the bound in #42 or until
something invalidates them, and they survive a sign-out. That is deliberate:
signing back in on a device somebody already used should be fast, and the entries
are already keyed per server, per account and per device, so leaving them costs
nobody else's privacy. A caller that wants them gone asks for that, and the core
removes every entry under the key space for that server and account.

The offline queue in #47 is kept until it is delivered or until its own bound is
reached, and never longer, because an undelivered action is the one piece of this
data with no copy on the server.

Diagnostic events and measurement spans are not retained by the core at all. They
are handed to whatever the client supplied under #100 and #61 and are forgotten
in the same call. The core keeps no ring buffer, no file and no history of them.

What an uninstall leaves behind is not the core's to promise, and this record
will not claim otherwise. The core does not choose where the client's stores put
their bytes, so it cannot say whether removing an application removes them. A
client can promise this and the core cannot, and stating it the other way round
would be the kind of assurance this whole position exists to avoid.

## What this position gives up

Crash reporting. A build that phoned home with a stack and the state around it
would find defects faster and would find the ones that only happen on somebody
else's hardware, which are the expensive ones. Without it, a defect is found when
somebody reports it in words, and #71 further restricts what they can be asked to
send. This is a real cost paid deliberately.

Usage measurement. Nothing here will ever say how many operators run this, which
platforms they are on, which endpoints matter, or which feature nobody uses.
Every one of those is a question worth answering when deciding what to build, and
they will be answered by asking people rather than by counting them, which is
slower and biased towards whoever speaks up.

Remote diagnosis of a performance number. The speed budget in #8 is measured
where the software runs, and no run on an operator's device reaches a build
here, so the numbers this project publishes are measured on machines this project
controls and never in the field.

None of the three is offered back behind a switch. A switch is a code path, a
code path is a host in the set on some build, and the property in #70 stops being
provable the moment one exists.

## Why this is written down before the code

A position adopted after the code exists is a position that has to be proven
against code that was not built for it, and the proof is an audit rather than a
check. Every place that reaches the network has to be found, and finding all of
them is the part nobody can promise.

The direction of the mistake is also one-way. Removing a route out of a device is
easy while no client depends on it and impossible once one does, because by then
somebody is reading the data it produced and the argument is about their
dashboard rather than about the position.

There is a specific failure this stops. A library that offers optional telemetry
puts the choice in the hands of the client author, and there are eleven of them.
An operator installing a client cannot tell which of them enabled it, and the
person whose viewing history it is has no relationship with any of them. Deciding
here that the route does not exist is the only version of this that an operator
can verify without reading eleven codebases.

## Alternatives, and what each cost

Telemetry that is off by default and can be turned on. The usual answer, it
respects the person who cares, and it produces data from the people willing to
give it. It costs the property outright: the code path exists on every build, the
default is one line of configuration away from being the other value, and #73 has
nothing left to prove. It also puts the decision with the client author rather
than with the operator.

Crash reporting only, with no usage data. Much easier to defend, and crashes are
where the real defects are. It costs the same property for the same reason, and a
crash report is among the worst carriers, because it captures the state around
the failure, which is the data that was being handled at the time.

Anonymised or aggregated measurement. Nothing identifying leaves, so the
objection appears to be answered. Aggregation happens on a host somebody runs,
which is a host the operator did not configure, so the rule above is already
broken before the anonymisation is examined. A viewing history is also among the
easiest things to re-identify, so the anonymisation is doing less work than it
appears to.

Leaving the position to each client. Honest about where the user relationship
actually is, and a client can make promises about its own platform that the core
cannot. It costs the operator any way of checking, and it costs this repository
the ability to say anything true about what the shared code does.

Federation enabled by default, with a way to switch it off. Better for the person
who wants their two servers to work together and never reads a setting. It costs
the population that needed the protection, as set out above, and it makes a
second host reachable on a device whose operator never chose one.

## What would reverse this

#70 turns out not to be writable as stated, because the host set cannot be
decided ahead of a run once federation is in it. The rule would then be prose
with no mechanism, and this record is superseded by one that either states the
rule in a form a check can reach or admits plainly that nothing refuses a
violation.

An operator asks for a route out and configures the host themselves. That is not
a reversal and is worth naming so it is not mistaken for one: a host the operator
configured is inside the rule, and #69 is where it is enumerated.

A jurisdiction this software runs in requires something this position forbids,
for a class of operator that actually exists. The record is then superseded by
one that names the requirement and holds it to its smallest surface, rather than
this one being quietly widened.

The list of what counts as personal data is found to have missed a field that
already shipped, twice. One is an omission. Two means the question in that
section does not place things reliably, and the test is replaced by one that
does.
