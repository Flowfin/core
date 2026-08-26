# 0113. The target triples the gate compiles for

Date: 2026-08-26

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #113

## The decision

The gate compiles the library for one target triple per client platform family,
held as a register beside the check rather than written into a workflow, runs the
suite on the runner's own host and on nothing else, and reports all of it under
one check-run name that does not move when a platform is added or removed.

## Where the set lives, and why it is not in this record

`.github/targets/targets` holds it, one triple per line with the reason it is
there. This record does not copy that list, and a reader who wants it runs the
command rather than trusting a paragraph:

    grep -v '^#' .github/targets/targets | grep .

A list here would be a second declaration of the set, and the two would disagree
on the first day somebody adds a platform, which is the one day this record is
worth opening.

What the register is for is the argument rather than the names. A platform enters
this gate by somebody writing a sentence saying which client family it serves,
and `.github/targets/targets.sh` refuses an entry that carries a triple and no
sentence. A triple somebody added and nobody argued for is a leg that runs on
every pull request forever without anybody knowing what it protects, and it is
never removed, because removing it would require the argument that was never
made.

## What decided the shape of the set

One entry per client platform family rather than one per triple the compiler
knows. The compiler's own number is not the gate's:

    rustc --print target-list | wc -l
    322

Two entries differ from that rule and both earn it. The Android platform gets
two, because its second ABI is the only 32-bit entry in the whole set and a
pointer-width or alignment assumption written into the core compiles cleanly on
every 64-bit target. The Apple platform gets three, because a phone, a desktop
and a television are three client families under one vendor rather than one.

Where a family offers two architectures on one operating system, one of them is
enough. What differs between them is codegen for a machine, which is the
compiler's own concern, and what this leg is looking for is source that does not
compile. The exception is the width, which is why the 32-bit entry exists and why
it is the entry to keep if the set is ever cut.

## What a compile buys, and what it does not

It catches the class that stops a build: a construct one toolchain accepts and
another does not, a pointer-width or alignment assumption, a conditional
compilation arm that exists on one platform and not on the next. That class is
real here and was measured rather than imagined, on this tree, by writing one
line into `src/lib.rs`:

    const _: () = assert!(size_of::<usize>() == 8);

    cargo build --locked --all-targets ; echo "exit=$?"
    exit=0
    cargo test --locked ; echo "exit=$?"
    exit=0
    bash .github/targets/targets.sh check ; echo "exit=$?"
    ::error::the library did not compile for armv7-linux-androideabi.
    exit=1

Both commands `README.md` gives a contributor pass that line. Before this leg
existed, so did the whole gate.

It buys nothing about behaviour. Not one line of the suite executes on any triple
in the register except the runner's own, so a green run says the core builds for
a platform and says nothing about how it behaves on one. The check prints that
sentence on every run, beside its verdict, rather than leaving a tick to be read
past.

## Why the suite is one run on one host

0011 already states it and this record is where the gate is made to match: the
core's own tests test the library rather than the binding, and the binding is
what differs per platform. The conformance suite in #76 is per client by
construction and is where a platform's own behaviour is asked about.

Running the suite on three host operating systems instead would buy the standard
library's own platform differences, which are real, and it would buy them by
tripling the wall-clock of the leg that a contributor waits on most often. That
is the trade to revisit on the day the core first reaches a platform facility,
and #27 is the earliest such day.

## The naming answer, which #26 takes from this issue

`build` and `test` keep their literal names, exactly as #15 and #16 wrote them.
Nothing in this decision turns either into a matrix, so neither is renamed and
the two strings #26 was written against are the two strings that exist.

The new leg reports under one name, `targets`, and that name does not move when
the register changes. The alternative the issue names is a matrix, which produces
`targets (aarch64-linux-android)` and one sibling per entry. Both are defensible
and they fail differently, so what decided it is which failure is silent.

A matrix makes every platform change a ruleset edit. Somebody adding a television
to the register would have to know that a rule on the default branch names the
legs one by one, and nothing in the register, the script or the workflow would
tell them. What they would get instead is a required context that no longer
exists, which is `pending` forever, and a required context nobody added, which is
a platform whose failure does not block anything. Neither state announces itself.

What a matrix would have bought is a leg that vanished being visible in the
ruleset. That is bought here instead, inside the check, where the person editing
the register meets it: the set derived from the register is counted against the
raw lines it came from, a disagreement is refused, and a register naming nothing
is refused rather than passing as a set with nothing to do. A reader that stopped
matching reports an empty set and prints a page indistinguishable from a run that
compiled every platform, and that is the failure the count exists against.

This is a decision about one name and not a rule that every check takes one. It
is worth saying because the board already carries a name that reads like a matrix
leg and is not one, so a reader comparing the two would otherwise take this as a
departure from a convention:

    grep -n 'name: Analyze' .github/workflows/codeql.yml
    61:    name: Analyze (rust)

That parenthesis is a literal somebody wrote, so the language it names is part of
the string a ruleset would match rather than a leg the runner generated. Which
shape a later check should take is that check's argument.

## What no run here covers

Named one by one, because a client author reads silence as coverage.

**Every platform outside the register.** A television running webOS or Tizen is
not in it: a client for either is hosted in a JavaScript runtime, no binding for
either exists, and adding a triple for one would be a leg protecting nothing. A
desktop on Windows on ARM is not in it. The Android emulator ABIs are not in it.

**The second architecture of a family that has one entry.** macOS on Intel,
Linux on ARM and 32-bit Windows are each the same operating system and the same
standard library as an entry that is here, and none of the three is compiled.

**Behaviour anywhere but the host.** Every triple in the register is compiled and
none is run. There is no device in this gate, no emulator and no simulator.

**The binding layer.** 0011 puts a generated foreign function interface between
this library and every client. No such artefact is in this tree, nothing here
generates one, and nothing here compiles or links one. A green run over seven
triples says nothing about the layer eleven clients will actually call.

**The optimised build.** Each triple is compiled once, unoptimised. A difference
that appears only under the optimiser is outside every run this leg makes.

**Two platforms the detector does not reach.** 0011 measured that the thread
sanitiser is available on the Linux and Apple targets and absent on the Android
and Windows ones, and #117 states that bound on its own leg. This record adds
nothing to it and does not narrow it: compiling for Android and for Windows here
is not the detector reaching them.

## What the leg costs, measured

Read off the first run of the leg, at
`b3090f025358f1905dee6ec95d52f48d50659076`:

    gh api repos/Flowfin/core/actions/jobs/98332650994 \
      --jq '"\(.name) \(.started_at) \(.completed_at) \(.conclusion)"'
    targets 2026-08-26T21:28:41Z 2026-08-26T21:29:12Z success

Thirty-one seconds, of which the step that installs seven standard libraries and
compiles seven times took seventeen:

    gh api repos/Flowfin/core/actions/jobs/98332650994 \
      --jq '.steps[] | select(.number == 4) | "\(.started_at) \(.completed_at)"'
    2026-08-26T21:28:53Z 2026-08-26T21:29:10Z

What it adds to the gate a contributor waits on is nothing, because the legs run
beside each other and this one is not the slowest. At the same commit:

    gh api repos/Flowfin/core/commits/b3090f025358f1905dee6ec95d52f48d50659076/check-runs \
      --paginate --jq '.check_runs[] | select(.completed_at != null)
        | "\(((.completed_at | fromdateiso8601) - (.started_at | fromdateiso8601)))\t\(.name)"' \
      | sort -rn | head -3
    116     Analyze (rust)
    42      thread-detector
    31      targets

So the leg costs thirty-one seconds of runner time per pull request and finishes
eighty-five seconds before the slowest leg on the same head. Both numbers move
with the register and with the tree, so they are re-read rather than cited.

## Why this is written down before the code

There is almost no code yet, and that is the only moment this set can be chosen
on its reasons. A gate acquires its platform set the way a repository acquires a
language: the first person who needs a build adds the runner in front of them,
the next person adds theirs, and the set afterwards is a history of who was
blocked rather than a statement about who the clients are.

The specific failure is narrower and this board has the shape of it already
written down twice. #113's own body names it: a matrix renames every required
check, and turning one leg into three detaches a ruleset from what it was
requiring without anything going red. The second is the one this leg is against,
and it is measured above: a defect that only appears on a platform nothing builds
for leaves the whole gate green, and the report arrives from a client repository
at the moment somebody first tries to ship. Both are cheap to decide now and
expensive to decide after eleven clients exist.

## Alternatives, and what each cost

**One leg, the runner's own triple, which is the state this replaces.** Cheapest,
and its cost is the measurement above: two commands green on a line that stops an
Android build.

**A matrix over host runners, building and testing on Linux, macOS and Windows.**
It buys the standard library's real platform differences under a running suite,
which is more than a compile buys. It costs the renaming of `build` and `test`,
which are the two strings #15, #16 and #26 are written against; it triples the
wall-clock of the leg a contributor waits on; and it still covers no phone and no
television, which is four of the client families and the two that are hardest to
reach any other way.

**Both: a matrix over host runners and a compile over every client triple.** The
most coverage available and the one to take on the day the core reaches a
platform facility. Today it pays for three runners to compile a library with no
dependency, no unsafe code and no conditional compilation arm anywhere in it, and
the three would agree by construction.

**Cross-compiling every triple the compiler knows.** Three hundred and twenty-two
legs, most of them for a platform no client will ever run on, and a red one on a
tier nobody supports would be a gate people learn to ignore. The register exists
so that the set is the clients' rather than the compiler's.

**One check-run name per matrix leg, which is the naming alternative.** Argued
above. It makes a vanished leg visible in the ruleset and makes every platform
change a ruleset edit somebody has to know about, and this record takes the first
of those inside the check instead.

## What would reverse this

The core acquires code that differs per platform - a conditional compilation arm,
a platform facility behind #27, or a dependency admitted under 0103 that carries
its own per-target build. A compile then stops being the interesting half, the
suite has to run somewhere other than the host, and this record is superseded by
one that says where.

A client family arrives that no triple in the register serves, or a family in the
register stops being a client. Either is a change to the set rather than to this
record, and it is made by editing `.github/targets/targets` with the reason on
the same line.

The leg's own cost crosses the slowest leg on the same head, so that adding a
platform starts adding wall-clock to what a contributor waits on rather than
runner time beside it. The two commands above are what say whether that has
happened, and the answer today is eighty-five seconds of headroom.

#26 writes required contexts into the ruleset on the default branch and takes a
shape this record did not anticipate. The ruleset wins, because a name a ruleset
matches is the name that decides a merge, and this record is superseded by one
describing what is required rather than what is reported.
