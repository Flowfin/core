# 0109. What a recorded fixture may carry, and why the check asks about membership

Date: 2026-08-29

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #109

## The decision

A recorded fixture in this repository carries, for every personal value a rule
can locate, either nothing at all or one of a small set of obviously synthetic
values declared in `.github/fixture-scrub/values`, and the check that holds this
asks whether a value it found is IN that set rather than whether it has the shape
of a personal one.

## Why the property is membership and not a shape

#109 asks for a check refusing a value that matches a named shape. Taken
literally that check refuses every recording the suite exists to serve, and the
reason is in the same issue two paragraphs above: the fields are not removed, they
are replaced with values that are obviously synthetic and stable. A synthetic
account name has the shape of an account name. A synthetic identifier has the
shape of an identifier. A shape alone therefore refuses both the real value and
its replacement, or it refuses neither, and there is no third setting.

The shape is still what finds the candidate, and that is why each rule carries
two patterns rather than one. `find` locates something that might be a personal
value; the judgement is then made against `allow`, which admits the whole of a
declared value and nothing else. The two are different questions and collapsing
them is what makes this look impossible.

The same split is what makes a refusal readable. A rule that refuses on a shape
can only say that something here looks like an identifier. A rule that refuses on
membership says that this file carries an identifier the recording procedure did
not put there, which is the sentence somebody acts on.

## What is absent rather than synthetic

Two of the five values carry no synthetic form at all: the session token and a
password.

`docs/decisions/0071-what-may-leave-through-a-diagnostic-event.md` excludes the
token outright rather than reducing it, and this record takes the same line for a
file on disk. A synthetic token would be a value that reads as a credential in a
tree anybody can clone, and the first person to copy one out of a fixture into a
request to their own server has learned nothing about whether it works until it
does. There is no case a recording needs one for: what a fixture proves is the
framing of an answer, and an answer's framing does not depend on the bytes of the
credential that asked for it.

A password never reaches a response at all, so its only route into a recording is
the request half of a recorded exchange, and
`docs/decisions/0030-the-password-route.md` fixes what a password may touch. A
file in a public tree is not in that set at any value.

## The vocabulary is not 0071's, and the two are about different things

0071 has three treatments, `excluded`, `reduced` and `carried whole`, and this
record has two, `absent` and `synthetic`. That is deliberate rather than drift.

0071 judges a field on its way out through a client's sink, at runtime, where a
correlator that is stable within one run and meaningless across runs is exactly
what a reader of a diagnostic report needs. A fixture is read by a person who is
comparing it against a server answer, and a salted digest in it would be a value
nobody can compare against anything, produced by a salt no recording could carry.
So the reduced treatment has no meaning here, and giving this register 0071's
three words with one of them unusable would be the worse mistake: a reader would
take the two registers for one rule stated twice.

## Where the values live, and why they are stable

The declared set is in `.github/fixture-scrub/values`, beside the check, one
block per rule, with the record it comes from and the failure it prevents on the
same block. `tests/recorded/README.md` is the same set written as a procedure,
where somebody about to record will meet it, and it names this record rather than
restating the argument.

An identifier that changed on every recording would turn each re-recording into a
diff nobody reads, and a real value that slipped through would arrive inside
exactly that noise. So the synthetic identifiers are a fixed short list, the
address is one reserved name and loopback, and the device identity is one value.

`server.invalid` is the address because RFC 6761 reserves the `.invalid`
top-level domain and no resolver answers for it. A recording that escapes into a
run therefore cannot reach a machine belonging to anybody, which a plausible
example domain does not promise.

## What this leaves uncovered, said plainly

The list in `docs/decisions/0068-the-data-locality-position.md` is closed by a
question a contributor answers rather than by a set of shapes. Its own words: a
contributor deciding about a field the list does not name asks whether two people
running the same build against the same server could hold different values there.
No reading of a file makes that judgement.

So a title, an account name, and a viewing history are personal data under 0068
and no rule here locates any of them. The check is a floor: it holds the values
whose shape can be written down, and the last reading is a person's. That is
stated on every run under what the run did not read, because a leg whose bound is
invisible is read as covering more than it does.

The history is the other bound. The check judges the tree at the commit it was
handed, so a value committed and then deleted is still in the history and no run
here reports it. That is the whole reason this record exists before the first
recording rather than after it.

## Why this is written down before the code

A scrubbing rule is the artefact that gets written after the incident. The order
this repository has already taken for line endings in #99 is the argument, and
this is the same order for a more expensive failure: a fixture committed once is
in the history whether or not a later change deletes the file, and a public
repository carrying one real library breaks the position in
`docs/decisions/0068-the-data-locality-position.md` through the test suite,
before a client exists, in the place nobody looks.

Without the record the check gets written the day somebody notices, against
whatever the fixtures already in the tree happen to contain, which is a rule
fitted to the defect rather than to the position. The synthetic values would then
be chosen at a keyboard by whoever recorded first, and the second person to
record would choose differently, which is the drift the stability rule above
exists against.

## Alternatives, and what each cost

**A shape alone, as #109 asks literally.** One pattern per rule, no declared set,
nothing to keep in step. It costs the whole subject: it refuses a scrubbed
recording exactly as readily as an unscrubbed one, so the first honest recording
turns the leg red and the leg is then turned off.

**No recorded fixtures at all, and a fake written by hand forever.** Nothing to
scrub, and it is the state of the tree today. It costs #104, which exists because
a hand-written fake proves the framing of an answer and never that a real server
still sends one of that shape, and it costs it permanently rather than for now.

**Scrubbing at the recording tool and no check.** The procedure alone, trusted.
Cheapest, and it is what most repositories do. It costs the property: whoever is
recording at eleven at night is exactly the reader who skips step 2, and a rule
nothing refuses is a rule this repository does not call a rule.

**A check over the history rather than over the tree.** It would catch the case
this record is most afraid of, a value committed and later deleted. It costs a
scan whose subject grows without bound and whose refusal has no legal repair: a
finding in a landed commit is fixed by rewriting history, which the ruleset on
`main` refuses. The order in the procedure is what stands in for it.

**One list of forbidden values rather than a list of admitted ones.** The
denylist shape, and it reads as the obvious one. It costs the direction a default
falls in: a value nobody thought to forbid is admitted, and the values nobody
thinks to forbid are the ones a real server invented.

## What would reverse this

A recording turns out to need a value under one of the two absent names for a
reason a framing test cannot avoid. The treatment for that name becomes
`synthetic` with a declared value, and this record is superseded rather than the
register quietly gaining an `allow`.

The synthetic identifiers collide with something a server sends, so a recording
carrying an honest server answer is refused. The set moves and the record moves
with it, and the procedure's stability rule is what makes the move one edit
rather than a re-recording.

A reading of what 0068 calls personal data is found that a pattern can hold for a
title or an account name. The floor above stops being a floor, the uncovered list
shrinks, and the sentence the run prints about what it did not read is corrected
rather than left standing.

The re-recording run in #104 lands and turns out to be the better place for the
rule, because it holds the bytes before they reach a file. The check here stays as
the thing that judges what is already tracked, and the record that supersedes this
one says which of the two is the authority.
