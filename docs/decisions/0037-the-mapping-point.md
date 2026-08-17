# 0037. The one point a failure becomes a kind, and what tells three populations apart

Date: 2026-08-17

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #37

## The decision

Every value of the closed set in 0004 is constructed at one point in the core and
at no other, that point is reached by the three sources 0004 maps and by the two
conditions the core raises about itself, and where it produces
`answer-not-understood` it names the reading site out of a set declared in one
place in the tree, so that the reversal condition 0004, 0069 and 0055 all rest on
can be read as three populations rather than as one count.

## One point, and what reaches it

0004 decides which kind a failure becomes. This record decides where that
decision is taken, and the answer is once.

Three sources arrive at it as inputs and are 0004's own: a transport failure
classified before any HTTP exists, an HTTP status read through 0004's table, and
a server-supplied error body that may add payload and may never change the kind.
A shape none of the three produces a kind for arrives as the fourth rule, which
is the catch-all that keeps the set closed.

Two more reach the same point and are mapped from nothing. `storage-unavailable`
is a store the client supplied answering badly, and `internal-fault` is a defect
here. Neither is a reading of somebody else's answer, so there is no mapping rule
for either, and they are constructed at this point anyway. The reason is the
property rather than tidiness: what #37 owes is that no value of the set can be
produced anywhere else, and a second construction site is a second place a
sixteenth thing can be invented, whether or not anything is being mapped there.

`cancelled` is the one that has to be said out loud, because it looks like an
exception and is not. 0009 separates a cancelled call from every failure and 0061
gives a span a third outcome for the same reason, so cancellation is not a
failure being classified. The value a caller receives is still constructed here,
because a caller holding an outcome should not be able to tell from its shape
which part of the core built it.

Nothing that is not one of the fifteen leaves. A raw status, a runtime's own
exception type, a store's own error, and a sentence a server wrote all stop here.
That is 0004's rule about what a client gets, arriving at the place that has to
hold it.

## The reading site, and the measurement that needs it

0004 already fixes what `answer-not-understood` carries: what the core was
reading, what it expected, and where in the answer it stopped. What this record
adds is the form of the first of those three. It adds no field and changes no
kind.

The site is a value from a set declared in one place in the tree, and never a
string written where the failure is raised. That is the rule 0061 already takes
for span names, for a different reason there and this one here: a set declared in
one place can be printed, counted and grouped, and a literal written at a call
site can only be read by whoever is looking at that line.

What it buys is the measurement three landed records rest on. 0004's reversal
condition is `answer-not-understood` becoming the kind an operator sees most,
measured on the diagnostic events in #100. 0069 pushes a refused cross-origin
redirect under the same kind and names the same measurement as what would
overturn it. 0055 pushes a refused image format under it, counts itself as the
second, and names the same measurement again. Three records, one number, and
three different repairs behind it.

A count of one kind cannot choose between those repairs. It says the vocabulary
is too small and does not say which of the three made it so, and the three are
not near each other: a redirect the core understood and refused, a format the
core recognised and declined, and an answer the core genuinely could not read.
Grouping the events by the site the kind was raised at is what turns one number
into three, and it is the only thing on this board that would.

So the site set has a value for a refused cross-origin redirect and a value for a
refused image format, distinct from every value naming something the core was
trying to parse. Adding a site is not a change to this record, in the same way
that adding a span name is not a change to 0061. Removing one, or collapsing two
into one, is, because that is the granularity the measurement is read at.

`internal-fault` carries a stable identifier for the site that produced it, which
0004 already requires, and it is declared the same way and for the same reason.
Two sets rather than one, because the two answer different questions and a shared
set would be a set where half the values can never appear on half the kinds.

## What a payload may not carry, and where redaction is not

0004's rule is that `answer-not-understood` never carries the answer itself,
because an answer holds library contents and may hold a token, and that no
payload field anywhere holds a session token. This point is where both hold or do
not.

Where a payload says where reading stopped, it is an offset and the field that
was being read, and not the bytes at that offset. The convenient shape is a
fragment of the answer, because that is what a person debugging wants, and it is
the shape that carries whatever happened to be at that offset into every route a
payload travels.

The redaction rule is not applied here, and getting that backwards is the mistake
worth naming. 0071 draws its boundary at the sink rather than at the type, and
says in as many words that an error returned to a caller in the same process is
not redacted. A client asked for the call, the caller is inside the process, and
a core that reduced a caller's own payload to correlators would be answering a
question nobody asked with a value nobody can use. What leaves through 0100 to a
diagnostic sink is where 0071 bites, and the point here hands the same values to
both routes rather than pre-chewing one of them.

## Proving the absence rather than asserting it

0004 says there is no default branch anywhere that produces something else, and
names #37 as where that absence is proven rather than asserted. What this record
decides is the shape of the proof rather than the proof, because there is no
language chosen, no build command and no test command in this tree.

Two things are owed, and they are not the same thing. That every failure a caller
receives is one of the fifteen is a property of the type a caller holds, and in a
means that can refuse a non-exhaustive match it is refused rather than tested,
which is 0004's own reversal condition about the language. That every failure the
core produces went through this point is a property of the source, and it is not
refusable by a type at all: a second construction site produces perfectly
well-typed values.

So the second one is what the proof has to reach, and the honest form of it is a
check over the tree for construction outside the one point rather than a test that
drives failures and looks at what comes out. A test proves the sites it reached.
The condition here is about the site nobody reached, which is the one that was
written last week for a case the suite does not have a fixture for.

That check is owed by #37 and does not exist, because there is nothing to check.
Naming it here is not a mechanism, and this paragraph is not one either.

## Why this is written down before the code

The mapping point is the thing that does not get built. Nobody sits down to write
a mapping point; somebody writes the first request, it fails in two ways, and the
two ways are handled where they happened because that is where the information
is. By the time there are forty call sites the mapping is in forty places, each
correct on its own, and the fifteen kinds are a document rather than a property.

The version of that which survives review is worse, because it looks like the
right thing. One helper is written, most call sites use it, and three do not,
because those three had something the helper could not carry. Nothing reports the
three. They are the sites whose failures were never counted in the measurement
above.

The reading site is the second, and it is lost in a way that is not recoverable
afterwards. A literal typed where the failure is raised works, reads fine, and
tells a person reading that line exactly what they want. It stops being useful at
the moment somebody wants to count, which is a year later and in a different
repository, and by then the values are spelled eleven ways across the core and
whatever an operator has already sent cannot be regrouped.

## Alternatives, and what each cost

Mapping at each call site, with the vocabulary as a shared type. Every failure is
classified where the most is known about it, which is genuinely more information
than a single point has. It costs the closure: nothing then forces a new
condition to be named rather than approximated, the fourth rule is written
independently at each site, and the sites that get it wrong are invisible because
their values are the right type.

A layered mapping, with the transport classifying its own failures and the
request layer classifying statuses. It follows where the knowledge lives and each
layer stays honest about what it knows. It costs one place to look, and it makes
the catch-all ambiguous: a shape the transport did not recognise and a body the
parser did not recognise arrive as the same kind from two different points with
two different notions of where reading stopped.

Free text for the reading site, so that whoever raises the failure can say
precisely what was being read. It is the most informative thing at the moment it
is written. It cannot be grouped, so the measurement three records rest on stays
a single count, and it is a field somebody will eventually put a value from the
answer into.

A sixteenth kind separating a refusal the core made from an answer it could not
read, so that no site set is needed to tell 0069's and 0055's populations from
the rest. It is the honest naming, and both records say the fit they took is
imperfect. It costs what 0004 prices a sixteenth kind at, a change to that record
and to every client, and it buys a distinction that a field on an existing kind
already carries.

Counting nothing, and revisiting the vocabulary when somebody complains. It costs
nothing today. It makes the reversal condition three landed records name
unreadable at the moment somebody wants to read it, which is the point at which
the vocabulary is already wrong.

## What would reverse this

The site set turns out to need a value per call site rather than per reading, so
that a count over it says how many places exist rather than what happened. The
grouping is then noise and the replacement decides a coarser set with the
measurement stated against it.

A means chosen in #11 that cannot refuse a value of the set being constructed
outside one point, and in which no check over the tree can find one either. The
single point is then a convention, and this record is superseded by one saying
that rather than claiming a property nothing keeps, which is the same shape
0004's own reversal condition takes about exhaustive matching.

A failure arrives that has to carry a fragment of the answer for anybody to act
on it, measured as a condition reported through #100 that nobody could diagnose
without one. The rule against carrying the answer is then costing more than it
saves, and the replacement decides what may be carried, from where, and with what
treatment at the sink under 0071.
