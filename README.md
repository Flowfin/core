# core

Every Flowfin client needs the same things and none of them should write those twice: talking to a Jellyfin server, holding a session, caching what was fetched, decoding artwork, tracking playback position, and measuring whether the speed budget was met. Eleven clients written independently drift in what they cache, in when they give up on a slow server, and in what they call fast. The speed budget is written as numbers a build can miss, and a number nothing measures is a wish, so this is where those numbers are instrumented. What shared means technically is decided: one Rust library reaching each client through a foreign function interface generated per platform, recorded with its costs in [0011](docs/decisions/0011-the-language-the-toolchain-and-the-binding-layer.md). The core draws nothing: a core that knows about widgets stops being shared the first time two platforms disagree about a list.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

## Building it

A fresh clone needs a Rust toolchain and nothing else. `cargo`, the formatter and
the analyser all arrive with it, and there is no dependency to fetch: the manifest
declares none, and what may ever be added to it is
[0103](docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md).
The version is pinned in [rust-toolchain.toml](rust-toolchain.toml), which the
toolchain manager reads by itself, so a fresh clone gets the right compiler
without being told to install one. A compiler that arrived some other way is
told which version this tree expects, by number, rather than meeting a compile
error.

Two commands, and they are the two the gate runs rather than variants of them:

    cargo build --locked --all-targets
    cargo test --locked

`--locked` is there in both so that a build which would rewrite `Cargo.lock`
fails instead of proceeding quietly. `--all-targets` is there so that the first
command builds the tests as well as the library.

THIS PARAGRAPH SAID THAT MADE IT A BUILD OF EVERYTHING RATHER THAN OF HALF OF
IT, AND IT IS NOT. `--all-targets` selects the test targets carrying
`test = true`, so the two targets `Cargo.toml` declares with `test = false` are
not compiled by it, and a file in either of them that stops compiling leaves this
command green. Measured rather than reasoned about:

    printf '\nthis is not rust and will not compile;\n' >> tests/needs_a_real_server_or_real_hardware.rs
    cargo build --locked --all-targets ; echo "exit=$?"
    exit=0
    cargo build --locked --test needs_a_real_server_or_real_hardware ; echo "exit=$?"
    exit=101

One of the two has a leg that builds and runs it on every pull request, which is
`.github/workflows/thread-detector.yml`. The other has none, and `Cargo.toml`
says so beside it.

## How the tree is arranged

One directory under `src/` per thing
[0003](docs/decisions/0003-what-the-core-does-not-do.md) says the core owns, so
that the boundary is visible in the tree and not only in a document:
`src/server/`, `src/session/`, `src/cache/`, `src/artwork/`, `src/playback/` and
`src/measurement/`.

Two directories beside those six are not concerns from that record and say so in
their own first paragraph. `src/failure/` holds the error vocabulary the six map
onto, and `src/diagnostics/` holds the sink a client supplies.

There is no behaviour in any of them yet. What each type is, and the statement
about which thread a client may call it from, is written where a reader meets the
type; `tests/thread_statements.rs` is what refuses a change that breaks one of
those statements.

## What this core sends, and to whom

Nothing, other than to the server an operator configured. There is no telemetry,
no analytics and no crash reporting here: no data about a person, a device or a
failure leaves for anybody but that server, and there is no setting that turns
such a route on, because there is no route to turn on.

That is the position in
[0068](docs/decisions/0068-the-data-locality-position.md) and it is checked
rather than promised. The `invariants` check refuses a telemetry, analytics or
crash-reporting package in the resolved dependency graph, which is the way one
of them usually arrives - as a dependency rather than as a decision, so that the
decision never gets made. The refused names are data rather than code:

    git grep -A1 '^id: no-reporting-dependency' -- .github/invariants/rules

**Two bounds on that, stated rather than left to be discovered.** The check is a
name list and not a purpose test, so a reporter published under a name nobody has
written there is not refused by it. And it reads the dependency graph, so it
cannot see a few lines written directly in this tree that kept something and sent
it; what stands against that is `no-network-outside-the-transport` in the same
register, which refuses a socket opened anywhere in `src/` outside the one
transport, and the review.

### Sending a crash report by hand

The position is that nothing is sent automatically, not that nothing may ever be
sent. If you want me to see a crash, open an issue with it. Strip it first: the
server address, the account name, the token, the device identity, and any title
or identifier out of a library are the fields
[0068](docs/decisions/0068-the-data-locality-position.md) lists as personal, and
an issue here is public from the moment it is submitted.

Where the crash is a security problem, [SECURITY.md](SECURITY.md) is the route
instead, and it says plainly whether that route is open today.

**There is no private destination for an ordinary crash report.** Which address
this repository publishes, for a vulnerability or for anything else, is entry 5
of #1 and is undecided, so a public issue with the fields above removed is the
whole of the by-hand route today.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

See [SECURITY.md](SECURITY.md) for how to report a security problem, what
this repository treats as one, and what a reporter gets back.

## License

AGPL-3.0, copyright 2026 Nils Lehnen.

The full text is in [LICENSE](LICENSE).
