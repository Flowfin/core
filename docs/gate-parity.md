# Parity with the gate on the sign-on plugin

The sign-on plugin in this organisation is the only repository here with a gate
worth copying, and M10 is measured against it. Parity does not mean copying its
workflow files. It means that for every check it runs, this board has a decision,
and that where the decision is anything other than adopting the check, the reason
is written down. Silence is what lets a gate quietly end up a third of the size of
the one it claims parity with.

This table is the record of those decisions. A row naming an issue is a plan and
not a run: nothing here asserts that a check exists in this repository, and the
`satisfied` rows are the only ones that say a check runs here today.

The first column names a file inside that repository's workflow directory rather
than a path in this tree, so no name in it is expected to resolve against this
repository's own paths.

## What was read, and when

That repository, at the commit this table was written against:

    git clone --depth 1 https://github.com/Flowfin/jellyfin-plugin-sso ssoclone
    cd ssoclone && git rev-parse HEAD
    54c5bf468a4f8719309fd59aa7f448ab17bfbbf8

The check runs on the head of its default branch:

    gh api repos/Flowfin/jellyfin-plugin-sso/commits/main/check-runs --jq '.check_runs[].name' | sort -u
    ABI floor build
    Analyze (actions)
    Analyze (csharp)
    Analyze (javascript-typescript)
    Audit workflows (zizmor)
    build
    Enforce greppable invariants
    Package (JPRM) / Build package
    Package (JPRM) / Generate SBOM
    prettier
    Reject Trojan Source Unicode
    Report any workflow that concluded non-success on the default branch
    Scorecard analysis
    submit-nuget
    wiki-lint

That output is one commit's worth and it moves, so re-run it rather than reading
the paste. The deduplication is not cosmetic. One name is on that commit six
times:

    gh api repos/Flowfin/jellyfin-plugin-sso/commits/main/check-runs --jq '.check_runs[].name' | sort | uniq -c | sort -rn | head -3
          6 Report any workflow that concluded non-success on the default branch
          1 wiki-lint
          1 submit-nuget

It also under-reports in two ways, so it is the wrong list to measure parity
against on its own. A workflow that runs only on a pull request never lands on the
default branch at all. A workflow that runs only on a schedule lands on whichever
commit was head when it last ran, so it appears or does not depending on when the
list is taken. The set that decides parity is therefore the workflow files and
their triggers, which do not move between two readings of one commit:

    awk 'FNR==1{if(NR>1)print f" | "t" | "nm; f=FILENAME; sub(/.*\//,"",f); t=""; nm=""; p=0} /^name:/&&nm==""{nm=substr($0,7)} /^on:/{p=1;next} /^[a-zA-Z]/{p=0} p&&/^  [a-z_]+:/{g=$0; sub(/:.*/,"",g); gsub(/ /,"",g); t=t g ","} END{print f" | "t" | "nm}' .github/workflows/*.yml
    build.yml | workflow_call, | Build
    codeql.yml | push,pull_request,schedule, | CodeQL
    dco.yml | pull_request, | DCO
    dependency-review.yml | pull_request, | Dependency review
    dotnet.yml | push,pull_request, | .NET
    e2e-login.yml | schedule,pull_request,release,workflow_dispatch, | E2E Login Harness
    fuzz.yml | schedule,workflow_dispatch, | Fuzz (SharpFuzz)
    manifest-freshness.yml | schedule,workflow_dispatch, | Manifest freshness
    nightly-betas.yml | schedule,workflow_dispatch, | Nightly betas
    opengrep.yml | push,pull_request, | Repo Invariant Lint (Opengrep)
    perf-baseline.yml | schedule,workflow_dispatch, | Performance baseline (login latency)
    prettier.yml | push,pull_request, | Prettier Lint
    pr-hygiene.yml | pull_request, | PR Hygiene
    publish.yml | push,workflow_dispatch, | Publish Release
    publish-beta.yml | workflow_dispatch, | Publish Beta
    publish-failure-alert.yml | schedule,workflow_dispatch, | Publish failure alert
    publish-jf12-beta.yml | workflow_dispatch, | Publish JF12 Beta
    publish-jf12-stable.yml | push,workflow_dispatch, | Publish JF12 Stable
    regenerate-manifest.yml | workflow_dispatch, | Regenerate manifest
    scorecard.yml | branch_protection_rule,schedule,push, | Scorecard supply-chain security
    stryker-mutation.yml | schedule,workflow_dispatch, | Stryker mutation testing
    unicode-guard.yml | push,pull_request, | unicode-guard
    wiki-lint.yml | schedule,workflow_dispatch,push, | Wiki Lint
    zizmor.yml | push,pull_request, | Workflow Security Analysis

The count in that output moves too. #80 read 23 files at `2b37832` and this table
has one row per file at `54c5bf4`, so the number is derived from the command
rather than restated here.

One check-run name in the first output belongs to no workflow file, and it is the
one a reader would otherwise go looking for:

    gh api repos/Flowfin/jellyfin-plugin-sso/commits/main/check-runs --jq '.check_runs[] | select(.name == "submit-nuget") | .html_url'
    https://github.com/Flowfin/jellyfin-plugin-sso/actions/runs/31521364131/job/93878890123
    gh api repos/Flowfin/jellyfin-plugin-sso/actions/runs/31521364131 --jq '{name: .name, path: .path}'
    {"name":"Automatic Dependency Submission (NuGet)","path":"dynamic/dependency-graph/auto-submission"}

`submit-nuget` is GitHub's own dependency submission rather than a file anybody
wrote, so it has no row. It is named here so that a reader comparing the two lists
does not conclude a workflow went missing.

What this repository runs today, which is what a `satisfied` row points at:

    gh api repos/Flowfin/core/contents/.github/workflows --jq '.[].name'
    dco.yml
    dependency-review.yml
    doc-paths.yml
    pr-hygiene.yml
    scorecard.yml
    shell-analysis.yml
    unicode-guard.yml
    zizmor.yml

Read at `7a8b187e8e1fb61ee769f57ba2d6bf8389ee2fb3` on this repository's default
branch. #80 named four of these as already satisfied and the sign-off gate as a
fifth. The hygiene check and the document check landed after the first version of
this table was written, through #83 and #110, so each of their rows moved from
naming a plan to naming a run. The shell analysis under #81 is the third, and its
row was written as a run in the change that added it while this paste could not
be, for the reason the next paragraph gives. This is the drift the table is most exposed to: a
row is written while a check is still an issue, the check lands, and the row goes
on describing the plan. It has happened twice now, and both repairs came after
the merge rather than inside it, because the paste above can only be read once
the file is on the default branch. The command above is what says how many rows
are `satisfied`, and a count written into this paragraph would be the next thing
to go stale.

## What the verdicts mean

`satisfied` is a check that runs here today, and the row names the file that runs
it.

`adopted` is taken in the same shape, and the row names the issue that lands it.

`adapted` is taken in a changed shape, and the row says what changed and why.

`satisfied` answers a different question from the two above it. Those two say what
shape a check is taken in, and `satisfied` says whether it has landed, so a row
can be both. Where a check runs here in a changed shape, the verdict is
`satisfied` and the row still says what changed and why, because losing that
sentence at the moment the check lands is losing it at the moment somebody starts
relying on the check.

`declined` does not apply here, and the row says why.

`waiting` is a row whose verdict cannot be taken yet, because the answer to a
named entry of #1 decides whether the row exists at all. The row says which entry
and what each answer would make of it. Waiting and declined are different states
and collapsing them is how a gap stops being visible.

Almost every issue named below produces code, and no code can be written before
entry 2 of #1 is answered. That is true of the whole board rather than of
particular rows, so it is stated once here and not repeated per row. A row is
`waiting` only where the verdict itself is undecidable, never merely because the
work is.

## The table

| Workflow file | Check-run name on the default branch | What it protects against | Verdict | What lands it here, and why |
| --- | --- | --- | --- | --- |
| `build.yml` | `Package (JPRM) / Build package`, `Package (JPRM) / Generate SBOM` | A release artifact built somewhere other than the release path, and one shipped without a bill of materials or a provenance attestation | waiting | Entry 2 of #1. One answer produces a library per target triple and another produces a specification and a suite, and only the first has an artifact to build. Where there is one, #94 builds it and #87 attaches the bill of materials and the attestation. Its two check runs reach the default branch through `dotnet.yml`, which calls this file. |
| `codeql.yml` | `Analyze (actions)`, `Analyze (csharp)`, `Analyze (javascript-typescript)` | A defect a compiler and a review both pass, found by semantic analysis rather than by a pattern | adapted | #81. `codeql.yml` runs here as one analysis rather than three, because this repository has one language, and its check-run name is `Analyze (rust)`. The deviation and its reason: fewer analyses, because there are fewer languages. The analysis over the workflow files that the third name there covers is not adopted, because `zizmor.yml` already reads those same files here and putting a second analyser over them is an argument nobody has made; that sentence is in `.github/workflows/zizmor.yml` too. The verdict on a finding is this repository's rather than the action's: the action uploads and does not fail a build, so `.github/codeql/codeql.sh` reads the file the analysis wrote and refuses a finding the register in `.github/codeql/excluded-rules` does not excuse by name, and refuses a file carrying no run or no loaded rule, because a query set that never loaded reports zero findings and reads exactly like a clean tree. #81 also reaches a third body of code neither of those covers, the shell this gate is written in, and that half has landed under its own row at the foot of this table rather than here. |
| `dco.yml` | none, it runs only on a pull request | A contribution nobody certified they had the right to make | satisfied | `dco.yml` runs here. The text its refusal message points a contributor at landed through #106, and the second file that message names is #23. |
| `dependency-review.yml` | none, it runs only on a pull request | A pull request that adds a dependency with a known advisory against it | satisfied | `dependency-review.yml` runs here. It reads the dependency graph of a change, and this tree has no manifest for it to read yet, so what it reviews arrives with #19. |
| `dotnet.yml` | `build`, `ABI floor build`, and the two `Package (JPRM)` runs through `build.yml` | A change that does not compile, does not pass its tests, drops coverage on the surface that decides security outcomes, restores a dependency graph that drifted from the lockfile, or uses something the oldest supported line does not have | adapted | One workflow there is several things here, because this board separates them by check name: `build` is #15, the test run is #16, the locked restore is #19, the coverage bar is #84, and the packaging job is `build.yml`'s row. `ABI floor build` is adapted rather than adopted, since the core talks to a server over its interface instead of linking against it, so the floor is a server interface version and the leg runs the suite against the oldest line's fixtures, which is #88. |
| `e2e-login.yml` | none on this commit; its job name comes from a matrix and it runs on a schedule, a pull request, a release and on demand | A sign-in that works against every fake and fails against a real server and a real identity provider | adapted | #22 and #92. A real round trip lives in the separate harness that refuses to run rather than skipping when what it needs is absent, because #20 forbids the headless suite from depending on a server or a display, and #92 is the same route in an operator's hands rather than in the gate. |
| `fuzz.yml` | none, it runs on a schedule and on demand | A parser that throws something nobody named when it is handed input somebody chose | adapted | #86, which copies both halves of that posture, the scheduled coverage-guided run and the seed corpus replayed inside the gating build, and widens the target set. This repository decodes images that arrived over a network as well as parsing responses, and 0101 treats a declared image dimension as untrusted, so the image path is fuzzed here and is not there. |
| `manifest-freshness.yml` | none, it runs on a schedule and on demand | A publish that reported success and left the channel with nothing installable in it | waiting | Entry 6 of #1. Where a release is published to a channel somebody installs from, a green publish run is not proof the channel moved and the same defence is owed, in #96. Where nothing is published anywhere until a client exists, there is no channel and the row is declined instead. |
| `nightly-betas.yml` | none, it runs on a schedule and on demand | A pre-release channel that either builds on every push or stops building without anybody noticing | waiting | Entry 6 of #1. A cadence for pre-release builds is only a question once there is somewhere to publish them, and what a first release contains is #91 with #96 as the workflow that would carry it. |
| `opengrep.yml` | `Enforce greppable invariants` | The mistake a compiler and a type system cannot see and a pattern can, made a second time | adapted | `invariants.yml` runs here, landed through #82 under this board's own naming. The rule set is data rather than code and every rule carries the record it comes from and the failure it prevents, which is what makes a rule with no stated failure refusable by the loader. The rules are taken from decisions already recorded here rather than from that repository's; the four that landed under #82 come from 0068, 0027, 0003 and 0102, #77 added three more, all three grounded in 0003, and #73 added a fourth reader of the lockfile grounded in 0068. Two of the four seeds #82's body names could not be written as a pattern and #82 carries the reason; a third could not either, because 0003 says of the drawing boundary in so many words that the forbidden side cannot be expressed as data. THAT SENTENCE IS UNCHANGED BY THE THREE #77 ADDED, and a reader who takes them for the boundary is making the mistake the sentence exists to stop: each of the three holds a list of what somebody has named - a set of dependency names, a set of view words, one trait - so a crossing written in a name nobody listed passes all three, and the run prints that bound beside its verdict. The count is not written into this cell for the reason the table gives elsewhere. Derive it:<br><br>`git grep -c '^id: ' -- .github/invariants/rules` |
| `perf-baseline.yml` | none, it runs on a schedule and on demand | A number published on a page with no run behind it, and a slow change nobody measured | adapted | #65 for the harness, #67 for publishing each measurement with the command that produced it, and #66 for the part that differs: the speed budget here is published as numbers a build can miss, so a missed number reddens a build rather than being archived for somebody to read. |
| `prettier.yml` | `prettier` | An argument about formatting, and a diff nobody can read for the whitespace in it | adapted | #18. The formatter is whichever one the toolchain pinned in #14 brings rather than that one, and #18 carries a trap that repository does not have: a formatter defaulting to one line ending, run in a checkout made with `core.autocrlf=true`, reports every tracked file as failing on a tree with no modifications. |
| `pr-hygiene.yml` | none, it runs only on a pull request | A change that arrives without the things a reader needs, judged by a person's patience rather than by a rule | satisfied | `pr-hygiene.yml` runs here, landed through #83 in a changed shape. It keeps the word that matters, deterministic, so the check refuses only what can be decided by reading the pull request and the issues it names. Its rule set is the smaller one, because this board has fewer conventions so far and a hygiene check that refuses conventions nobody has written yet is one that gets bypassed in its first week. Its rules sit in `.github/pr-hygiene/hygiene.sh` rather than in the workflow file, and each run proves them against their own fixtures before it judges anything, so a rule cannot pass a fixture and refuse something else in the gate. |
| `publish.yml` | none on this commit; it runs on a stable tag push and on demand | A release built by hand on somebody's machine, and a tag that ships what a pull request could not merge | waiting | Entry 6 of #1 decides whether there is a publish at all and to where. Where there is, #96 runs the full gate before anything is published and #94 produces the bytes. |
| `publish-beta.yml` | none, it runs only on demand | A pre-release nobody can install, so nothing is tested before it is released | waiting | Entry 6 of #1. A channel that publishes an installable build on every push to the default branch is a promise to installers, and no entry of #1 has made one yet. |
| `publish-failure-alert.yml` | `Report any workflow that concluded non-success on the default branch` | A scheduled or post-merge workflow that starts failing with nobody waiting on it, and stays red for weeks | adopted | #90, which takes the property rather than a list: the run derives what it watches from the runs on the default branch, opens an issue rather than sending a notification, and prints the full list of what it examined so a workflow it never heard of cannot read as one it checked. |
| `publish-jf12-beta.yml` | none, it runs only on demand | An upgrader on a newer server generation left with a build that will not load there | waiting | Entries 3 and 6 of #1 together. A second publishing leg exists only where more than one server line is supported and something is published, and the testing half of that question is #88 rather than this row. |
| `publish-jf12-stable.yml` | none on this commit; it runs on a generation-specific tag push and on demand | Two release lines firing on each other's tags, so a release goes out for the wrong generation | waiting | Entries 3 and 6 of #1, as the row above. What is worth keeping whatever the answer is the tag shape that keeps two lines from matching each other's glob, and #93 is where a version scheme is decided. |
| `regenerate-manifest.yml` | none, it runs only on demand | A publish that half succeeded and cannot be finished, because re-running it re-enters a step against a release that is already sealed | waiting | Entry 6 of #1. Any publishing route reachable from this board owes a recovery path that does not rebuild or re-release, and #96's dry run is the nearest thing on this board to one today. |
| `scorecard.yml` | `Scorecard analysis` | A supply-chain regression in the repository's own configuration, found by somebody else | satisfied | `scorecard.yml` runs here. |
| `stryker-mutation.yml` | none, it runs on a schedule and on demand | A suite that covers a line without asserting anything about it | adapted | #85, with no deviation in placement: scheduled, reported and gating nothing, because the question it answers is far too slow to sit in front of a merge. What differs is the scope list, since the modules differ, and here it is the surface named in #84. |
| `unicode-guard.yml` | `Reject Trojan Source Unicode` | Source that reads one way to a person and another way to a compiler | satisfied | `unicode-guard.yml` runs here. |
| `wiki-lint.yml` | `wiki-lint` | Prose that names a path or a command that no longer exists, so the first thing a reader follows is the first thing that is wrong | satisfied | `doc-paths.yml` runs here, landed through #110 in a changed shape, and it covers the path half of that protection and not the command half. There the documentation is a separate repository with no gate of its own, so the check cannot run on a pull request without a typo already in the wiki reddening every unrelated change. Here the documents and the code are in one tree and move in one commit, so the check runs on the pull request, which is where the change that moves a file can fix the sentence naming it. Its rules sit in `.github/doc-paths/doc-paths.sh` rather than in the workflow file, and each run proves them against their own fixtures before it judges anything. A command a document tells a reader to run is not checked: that needs a verb in the tree or a pinned toolchain to look in and this repository has neither, #14 pins one, and the run prints the absence on every pull request rather than leaving it to be assumed. The external addresses are a second job that returns zero whatever it finds, since an address outside this repository that is down for an hour is not a defect here. |
| `zizmor.yml` | `Audit workflows (zizmor)` | Workflow YAML treated as configuration when it is release-critical attack surface | satisfied | `zizmor.yml` runs here. |
| none on that gate | `thread-detector` | A concurrency claim broken under a load nobody reproduces by hand, found as a rare wrong answer rather than as a failure | satisfied | #117, and it is added here rather than adopted: parity is a floor rather than a ceiling, and this is the first row added above it. This repository is a library hosted inside other people's processes rather than one process on a server, and 0009 makes promises about which thread a caller is left on, so a detector that reddens when one of those promises is broken is a check this board needs and that gate does not run. `.github/workflows/thread-detector.yml` runs here on every pull request and on every push to the default branch, and `.github/thread-detector/thread-detector.sh` is the verdict: the suite runs under the detector and a report against it fails the run, and then a target holding a data race written on purpose runs under the same detector and a run of it that reports nothing fails the run too, because a detector that was never switched on reports nothing and prints a page indistinguishable from a clean tree. A finding it does not refuse is written in `.github/thread-detector/suppressions` with the reason, an entry there carrying no reason is itself refused, and the detector's own suppression file is derived from that register rather than being a second file. Two bounds it prints on every run: the detector reaches neither the Windows nor the Android target, so a race that appears only there is outside every run this leg makes, and it reports an interleaving it observed rather than one that is possible. |
| none on that gate | `Analyse the shell the gate runs (shellcheck)` | A defect in the language this gate's own checks are written in, and in particular an expansion left unquoted, which word-splits its input and points a rule at something nobody named | satisfied | #81. `.github/workflows/shell-analysis.yml` runs here, and its settings and its fixtures are in `.github/shell-analysis/shell-analysis.sh`. A rule it does not refuse is written in `.github/shell-analysis/excluded-rules` with the reason it is not refused, an identifier written there with no reason is itself refused, and every run prints that file beside its verdict. Its findings reach the code-scanning tab as well as the job log: the same script writes them as SARIF, the workflow uploads that file under its own category, and the upload is skipped on a pull request from a fork, where the token cannot write to that surface and the gate still refuses. Parity is a floor rather than a ceiling and this is the second row added above it. That gate runs no shell analyser: the single mention of shellcheck in that repository is a disable directive inside a composite action rather than a run, and no workflow file there invokes it. Here two of this gate's own legs are shell scripts that read paths and pull-request bodies, so the language the checks are written in was the one body of executable code in this tree that nothing read. It covers the shell and not the core's own language, which is the half of #81 that waits on #11. |

## What this table does not cover

The configuration that proposes dependency updates sits beside those workflows
rather than inside one, so no row here reaches it and #108 holds it instead. This
table is built from workflow files, which is exactly the gap #108 was opened
against.

Nothing here says whether a check that runs on this repository is passing, on
either repository. The rows record what exists and what is decided, and a run's
conclusion is a different question read at a different time.

No row was verified by running that gate's check against this tree. Every verdict
is a reading of what the check protects against, taken from the workflow file and
its own comments, against what this board has already decided. Where a verdict is
wrong, it will be wrong in the direction of assuming a check transfers.
