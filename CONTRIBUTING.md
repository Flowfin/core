# Contributing

## Before you push

Two commands. They are the ones the gate runs, character for character, so a
green run here and a green run there are the same run:

    cargo build --locked --all-targets
    cargo test --locked

A fresh clone needs a Rust toolchain and nothing else. There is no restore step
to run first, and that is a fact about this tree rather than a convenience: the
manifest declares no dependencies, so `cargo build` fetches nothing. When that
changes, `--locked` is what makes the restore refuse to rewrite `Cargo.lock`
rather than doing it quietly, and it is already in both commands for that reason.

The toolchain version is pinned in `rust-toolchain.toml`, in one place, and no
workflow carries a copy of the number. The toolchain manager reads that file
itself, so a fresh clone compiles with the pinned version without being told to
install anything. Where a compiler arrived some other way, the `build` check says
which version this tree expects and which one ran, by number:

    bash .github/toolchain/toolchain.sh check

The gate still prints the compiler it used on every run, so a verdict can be read
against a version rather than against a promise about one.

`README.md` carries the same two commands and the arrangement of the tree. This
document is the one that says what happens around them.

## No work without an issue

Every change starts as an issue and lands as a pull request. Direct pushes to
`main` are refused.

An issue says what is wrong, what the evidence is, and what "done" means. If the
evidence is a number, it carries the command that produced it. A sentence with a
number and no command is a claim, and the difference between a claim and evidence
is one line.

The same holds in a pull request body. Every number in it carries the command that
produced it, run at the commit being pushed rather than in a working tree that has
something else in it. Where a claim cannot be backed by a command, write it as a
claim and say so.

A negative statement stays negative. If something was not done, not measured, or
not covered by any run here, say so plainly and do not soften it in a later edit.

## Sign your work

Every commit carries a `Signed-off-by` trailer matching its author, and the check
named `DCO sign-off` refuses a pull request where one does not:

    git commit -s

Retroactively, for a branch that already has commits:

    git rebase --signoff <base>

What the trailer refers to is the developer certificate of origin in the file
named DCO at the root of this repository. Signing off is a statement about the
provenance of what you wrote, not a formality.

## What the gate runs, and what each thing refuses

The list below goes stale the day a check is added. Derive it rather than trusting
it:

    gh api repos/Flowfin/core/commits/main/check-runs --jq '.check_runs[].name' | sort

**`build`** compiles the tree with every compiler warning an error, using the
first command above. Before it compiles anything it compares the compiler that
ran against the version `rust-toolchain.toml` pins, and refuses a mismatch by
number rather than letting it arrive as a compile error.
`.github/toolchain/toolchain.sh` holds that comparison and the fixtures that
prove it. `.github/workflows/build.yml`.

**`test`** runs the suite with the second command above and refuses a run that
collected nothing. A harness that ran no test exits zero and prints a page that
reads like a clean run, so the count is read out of the run, printed, and refused
when it is zero; a non-zero filtered-out total is refused for the same reason.
The number of tests executed is written to the step summary.
`.github/test/test.sh` holds the counting and the fixtures that prove it.

**`lint`** runs the analyser the language ships, denying its default, pedantic and
manifest sets, with every remaining warning an error. The lints it does not refuse
are in `.github/lint/excluded-lints`, one per line with the reason, and a line
carrying a name and no reason is itself refused. `.github/lint/lint.sh` holds the
settings and the fixtures that prove them.

**`format`** runs the formatter the language ships over every tracked source file,
in `--check` mode, so a carriage return is refused rather than converted. The
subject is `git ls-files` rather than the crate, because `cargo fmt` walks the
module graph and never opens a tracked file no `mod` declares. The files it does
not ask about are in `.github/format/unformatted-paths`, one per line with the
reason, and a line carrying a path and no reason, or a path the tree no longer
carries, is itself refused. `.github/format/format.sh` holds the settings and the
fixtures that prove them.

**`DCO sign-off`** refuses a commit whose trailer does not match its author.

**`Deterministic PR-hygiene checks`** refuses a pull request that names no issue
and one whose body says nothing the template did not. Where an issue it names
declares a `Scope:` line at column zero, the changed paths are compared against
it; where none does, the run prints that the comparison was not made.
`.github/pr-hygiene/hygiene.sh`.

**`Documents name paths that resolve`** refuses a Markdown link target or a code
span that names a path not tracked in this tree. `.github/doc-paths/doc-paths.sh`
carries what it reads and, on every run, the list of what it does not.

**`Analyse the shell the gate runs (shellcheck)`** analyses every tracked shell
file. The rules it does not refuse are in `.github/shell-analysis/excluded-rules`
with the reason for each.

**`Audit workflows (zizmor)`** audits the workflow files themselves.

**`Reject Trojan Source Unicode`** refuses bidirectional and invisible Unicode
control characters in tracked text.

**`dependency-review`** reads the dependency diff of a pull request against the
advisory database.

Two runs report and refuse nothing, which is deliberate rather than an oversight.
**`External addresses in documents`** requests the addresses documents name and
prints what answered; an address outside this repository that is down for an hour
is not a defect here, and a gate that reddens for it teaches people that red means
nothing. **`Scorecard analysis`** scores the repository and writes to the
code-scanning surface.

## Which of these is a gate, and which is a sentence

Read this before treating a green tick as a merge condition.

**No check is required to merge.** The ruleset on `main` requires a pull request
and refuses a deletion and a rewrite, and it names no status check at all:

    gh api repos/Flowfin/core/rulesets/20572113 --jq '[.rules[].type]'
    ["deletion","non_fast_forward","pull_request"]

So a red `build` blocks nothing today. #26 is where the names are written into
that ruleset, and it waits on #113 deciding what the names will be under a build
matrix. Until then, whether a red check stops a merge is a person's judgement, and
the rule is that it does.

**The two commands above are prose.** Nothing compares what a workflow invokes
against what this document and `README.md` say a contributor runs. The three are
kept in step by whoever edits one of them.

**Formatting is checked, and `cargo fmt` is not what checks it.** Run `cargo fmt`
before pushing and the gate will agree with you for every file the module graph
reaches. It reaches every source file in this tree today, and it is not what the
`format` check runs: that reads `git ls-files`, so a source file added without a
`mod` declaring it is judged here and would pass `cargo fmt --all --check`
untouched. `bash .github/format/format.sh check` is the run the gate makes, and
it needs no network.

**A test that needs real hardware or a real server does not run here.** It has a
harness of its own, `tests/needs_a_real_server_or_real_hardware.rs`, declared in
`Cargo.toml` with `test = false` so that `cargo test --locked` never invokes it
and nothing it needs can make the headless suite conditional. Running it is
deliberate:

    cargo test --locked --test needs_a_real_server_or_real_hardware

What it covers that the headless suite cannot: a genuine TLS handshake against a
genuine certificate chain, a real server's behaviour under a real sign-in, and
image decoding at real sizes on real hardware. A fake proves the core's reaction
to a shape; it cannot prove that the shape is what a real server sends or that
real hardware decodes what was handed to it.

It refuses rather than skipping. An absent prerequisite prints every missing
variable by name and exits non-zero, and so does a run that carries no case,
because a skip exits zero and cannot be told from a run that proved something.

**It carries no case today**, so every path in the list above is untested rather
than tested elsewhere, and a pull request touching one says so in its own body.
The first case needs behaviour that does not exist yet; #27 is where the earliest
of it arrives.

**Every test in this repository runs headless**, which is #20's rule. Every test
runs with no display server present and as a non-elevated user. A test that needs
either is a defect in the test rather than a step to document.

The reason it is a rule rather than a preference is that it cannot be added later.
A suite that grew up assuming a display, a keychain prompt or an administrator is
not made headless afterwards without rewriting the tests that matter most, and by
then somebody will argue that the ones needing a display are the important ones.

One concrete case, written here so nobody rediscovers it. Binding a socket to a
machine's own interface address rather than to loopback raises a firewall consent
dialog on Windows. The dialog is answered by an administrator, and its subject is
the executable's full path, so answering it settles nothing for the next build
directory. A test that needs that bind belongs in the separate harness above.

Nothing enforces any of this today, and the `test` check arriving did not change
that. #20's other condition is that the check runs on a runner with no display
server, as a non-elevated user, and with no network access to anything but a
loopback address. That check exists now, from #16, and it does none of the three:
it runs the suite and counts what the suite collected. A test that opened a
display, requested elevation or bound to a non-loopback address passes it. #20 is
where those three land.

**Nothing reads the prose of an issue, a commit message or a pull request body.**
`Scope:` at column zero is the only line any route takes out of an issue. Whether
a body says what changed, what failure it prevents, or what was not covered is
read by a person.

## Fixtures

A fixture exists to prove an exact sequence of bytes, so nothing under
`tests/fixtures/` is translated on the way into the tree or out of it.
`.gitattributes` carries that rule, and `tests/fixture_bytes.rs` is what goes red
when the rule is removed.

Read a fixture as bytes. Every convenience for reading lines treats a carriage
return and a line feed as the same thing, which is exactly the difference a
fixture in that directory exists to hold.

## Decisions

A decision that shapes this repository is a file under `docs/decisions/`, in the
shape `docs/decisions/0001-decision-records.md` fixes: what was decided in one
sentence, what would have gone wrong without the record, the alternatives with
what each one cost, and a reversal condition somebody can check against the world.
A record is added or superseded and never edited in place.

The number of a record is the number of the issue whose decision it records, and
`docs/decisions/README.md` lists every record. Adding a record without its index
line leaves the index wrong and nothing here refuses that.

## Adding a check to the gate

Where a check needs logic rather than one command, the logic goes in a script
beside the workflow rather than in steps inside it, and the script carries
fixtures proving each rule bites. `.github/lint/lint.sh`,
`.github/format/format.sh`, `.github/test/test.sh`,
`.github/toolchain/toolchain.sh`, `.github/doc-paths/doc-paths.sh` and
`.github/shell-analysis/shell-analysis.sh` are the ones that exist, and each runs its own fixtures before it judges
anything, so a rule cannot pass its fixture and refuse something else in the
gate. The count is left out of that sentence on purpose, because a number in a
document drifts against the directory it describes. Derive it:

    git ls-files -- '.github/**/*.sh'

A rule that is turned off is turned off in a register beside the script, one entry
per line with the reason on the same line, and the run refuses an entry that
carries no reason. An exclusion is a debt rather than a dispensation, so the
reason says what would retire it.

The check-run name matters as much as the check. GitHub takes it from the job's
`name:` and falls back to the job id, a ruleset matches that literal string, and a
renamed job silently detaches a requirement from the thing it was requiring.

## What a pull request body carries

The template prompts for it and no check reads it. What a reader needs: what
changed in the words somebody will use in a year, the specific failure it
prevents, the evidence with its commands, what a guard here refuses together with
the run that watched it fail, what the change does not cover, and whether anybody
other than the author has read it.

Where a change adds or edits a guard, show it refusing: the deliberate violation,
the run that went red on it, and the run that stays green without it. A guard
nobody watched fail is a guard nobody knows the direction of.
