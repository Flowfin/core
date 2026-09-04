# 0292. The means the core speaks HTTP with, and what it costs

Date: 2026-09-03

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #292

## The decision

The core writes and reads HTTP/1.1 through `ureq-proto`, taken with
`default-features = false` and the `client` feature alone, driven over a socket
the core opens and a TLS stream 0243 already decided; it is admitted under
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s clause for a
dependency a landed record already requires - the records being
[0011](0011-the-language-the-toolchain-and-the-binding-layer.md), which measures
that the standard library reaches TCP and stops, and
[0027](0027-the-transports-timeouts-and-connections.md), which requires a
transport - and it is taken as a protocol rather than as a client because every
bound, every connection and every destination in 0027, 0069 and 0102 is a value
this tree already holds and a client would hold instead.

## What the choice was actually between, and it was not the size of a graph

The set of candidates and the first readings against them are on #292, taken on
2026-09-03 and not repeated here. What that reading leaves is two shapes rather
than eight: a client configured to 0027, which is `ureq`, and a protocol driven
over the core's own socket, which is `ureq-proto`. Everything below re-derives
what separates those two, because a decision made from somebody else's reading
is the failure this repository's first rule is about.

The number that is easiest to reach for does not separate them. Counted below,
`ureq` brings three packages more than `ureq-proto` does, and three packages is
not a reason to write a protocol by hand or to refuse to. What separates them is
which artefact holds 0027's numbers, which clock 0102's bounds are measured
against, and whether a destination 0069 did not admit can be reached at all.

## How the readings below were taken

Every `cargo` reading in this record was taken in scratch crates outside this
tree, on a Windows machine, with the toolchain `rust-toolchain.toml` pins, which
resolve `cargo 1.98.0 (797e8a9bc 2026-08-05)`. Every source reading was taken
against the crate sources `cargo vendor` wrote for those crates, at the versions
each lockfile resolved. The gate runs on `ubuntu-latest`, so what is measured
here is the resolver's answer and each crate's own source, and never the runner's
verdict. Nothing below was compiled for any triple but this machine's.

The tree these are measured against:

    git rev-parse origin/main
    d9de3c7fb18237d4e3456c6504a01819d82c55d5

## The baseline, so that every count is a difference

0243 already decided that a certificate is validated through `rustls` driving
`rustls-platform-verifier`, and that entry arrives with the socket rather than
with this record. So a count of a candidate's graph that includes those two is a
count of two decisions at once. One scratch crate declares exactly 0243's two
lines and nothing else:

    rustls = "0.23"
    rustls-platform-verifier = "0.7"

    cargo tree -e normal --target TRIPLE --prefix none \
      | sed 's/ (\*)$//;s/ (proc-macro)$//' | sort -u | grep -v '^probe-' | wc -l
    aarch64-linux-android        29
    armv7-linux-androideabi      29
    aarch64-apple-ios            17
    aarch64-apple-tvos           17
    aarch64-apple-darwin         17
    x86_64-pc-windows-msvc       13
    x86_64-unknown-linux-gnu     13

Those are 0243's own seven numbers, re-derived rather than copied, so the counts
below are comparable with that record.

## What each shape costs, counted rather than supposed

`ureq-proto` on its own, with `default-features = false` and `features =
["client"]`, resolving `ureq-proto v0.6.1`:

    cargo tree -e normal --target TRIPLE --prefix none \
      | sed 's/ (\*)$//;s/ (proc-macro)$//' | sort -u | grep -v '^probe-' | wc -l
    aarch64-linux-android        7
    armv7-linux-androideabi      7
    aarch64-apple-ios            7
    aarch64-apple-tvos           7
    aarch64-apple-darwin         7
    x86_64-pc-windows-msvc       7
    x86_64-unknown-linux-gnu     7

Seven on every triple, and the same seven, which is the property worth having
rather than the size: nothing in this graph is conditional on a platform, so
there is no triple on which a client links something the other six do not.

    base64 v0.23.1
    bytes v1.12.1
    http v1.5.0
    httparse v1.10.1
    itoa v1.0.18
    log v0.4.34
    ureq-proto v0.6.1

`ureq` with `rustls-no-provider` and `platform-verifier`, which is the feature
pair that takes 0243's means in, resolving `ureq v3.4.0`:

    aarch64-linux-android        35
    armv7-linux-androideabi      35
    aarch64-apple-ios            24
    aarch64-apple-tvos           24
    aarch64-apple-darwin         24
    x86_64-pc-windows-msvc       20
    x86_64-unknown-linux-gnu     20

The second set of counts includes 0243's own graph and the first does not, because
a client cannot be declared without its TLS and the protocol crate does not touch
it. So the comparable reading is what each adds over the baseline above, taken as
a set difference per triple:

    comm -13 baseline.TRIPLE ureq-proto.TRIPLE | wc -l
    aarch64-linux-android        5
    armv7-linux-androideabi      5
    aarch64-apple-ios            6
    aarch64-apple-tvos           6
    aarch64-apple-darwin         6
    x86_64-pc-windows-msvc       6
    x86_64-unknown-linux-gnu     6

    comm -13 baseline.TRIPLE ureq.TRIPLE | wc -l
    aarch64-linux-android        8
    armv7-linux-androideabi      8
    aarch64-apple-ios            9
    aarch64-apple-tvos           9
    aarch64-apple-darwin         9
    x86_64-pc-windows-msvc       9
    x86_64-unknown-linux-gnu     9

Three packages on every triple. Compared directly rather than through the
baseline, the two listings differ by thirteen, and ten of the thirteen are 0243's
own entry arriving with the client rather than beside it:

    comm -13 ureq-proto.x86_64-unknown-linux-gnu ureq.x86_64-unknown-linux-gnu
    once_cell v1.21.4
    openssl-probe v0.2.1
    percent-encoding v2.3.2
    rustls v0.23.43
    rustls-native-certs v0.8.4
    rustls-pki-types v1.15.1
    rustls-platform-verifier v0.7.0
    rustls-webpki v0.103.15
    subtle v2.6.1
    untrusted v0.9.0
    ureq v3.4.0
    utf8-zero v0.8.1
    zeroize v1.9.0

So the whole of the graph argument between the two shapes is `ureq`,
`percent-encoding` and `utf8-zero`, and this record does not rest on it.

## The licence, read rather than assumed

Every expression carried by the seven, derived rather than eyeballed:

    cargo metadata --format-version 1 --locked \
      | jq -r '.packages[] | "\(.name) v\(.version)\t\(.license)"' | sort
    base64 v0.23.1      MIT OR Apache-2.0
    bytes v1.12.1       MIT
    http v1.5.0         MIT OR Apache-2.0
    httparse v1.10.1    MIT OR Apache-2.0
    itoa v1.0.18        MIT OR Apache-2.0
    log v0.4.34         MIT OR Apache-2.0
    ureq-proto v0.6.1   MIT OR Apache-2.0

Six dual offers and one bare `MIT`. Every member of every expression is in
0103's admitted half, so nothing here reaches the third state
[0268](0268-a-conjunctive-licence-expression.md) named, where a term the set
places in neither half leaves an expression undecided. No conjunction appears in
this graph at all, so 0268's rule is not exercised by it rather than satisfied by
it, and that distinction is worth keeping: a later version of any of the seven
could introduce one, and the reading that would catch it is the command above
rather than this paragraph.

## The four behaviours, read against each package's own source

0103 refuses four behaviours outright and 0061 supplied a fifth. Each is read
over the vendored sources of all seven packages rather than over the crate this
record names.

**It reaches no network of its own.** No package in the graph opens a socket or
resolves a name:

    grep -rnE 'TcpStream|UdpSocket|ToSocketAddrs' */src ; echo "exit=$?"
    exit=1

The wider pattern answers in one file, and the hits there are not a reach:

    grep -rlE 'std::net' */src
    log/src/kv/value.rs

which is a block of `impl_to_value_from_display!` entries for the standard
library's own address types - a value somebody hands the facade, not an address
anything here connects to. `ureq-proto` says the same of itself, in its own scope
statement: opening and closing sockets is out of scope.

**It touches no filesystem.** One hit, and it is a documentation example:

    grep -rnE 'std::fs|OpenOptions|File::(open|create)' */src
    bytes/src/bytes.rs:240:    /// let file = File::open("upload_bundle.tar.gz")?;

**It starts no thread.** Five hits, one of them a documentation example and the
other four inside two `#[cfg(all(test, loom))]` modules, which are a concurrency
proof rather than a worker:

    grep -rnE 'thread::spawn|thread::Builder' */src
    bytes/src/bytes.rs:1643:            let t1 = thread::spawn(move || {
    bytes/src/bytes.rs:1648:            let t2 = thread::spawn(move || {
    bytes/src/bytes_mut.rs:246:    /// let th = thread::spawn(move || {
    bytes/src/bytes_mut.rs:2015:            let t1 = thread::spawn(move || {
    bytes/src/bytes_mut.rs:2020:            let t2 = thread::spawn(move || {

    grep -n 'cfg(all(test, loom))' bytes/src/bytes.rs bytes/src/bytes_mut.rs
    bytes/src/bytes.rs:1627:#[cfg(all(test, loom))]
    bytes/src/bytes_mut.rs:1997:#[cfg(all(test, loom))]

**It writes to no log.** One package links a logging facade, and it is `log`
itself, which 0243 already narrowed this behaviour for and which is already in
the graph 0243 brings. `ureq-proto` writes through that facade and installs no
sink. The condition 0243 attached to its narrowing is that the core installs no
logger and states that it installs none, and this tree already refuses one:

    grep -n -A1 '^id: no-logger-installed' .github/invariants/rules

**It carries no field-bearing surface.** Nothing in the graph is a tracing
library, and no package in it declares one. 0061's ground is not reached.

Two further properties, neither of which 0103 asks for and both of which cost
nothing to record now and something to discover later. `ureq-proto` declares
`#![forbid(unsafe_code)]`, which is the strongest statement a parser of untrusted
bytes can make about itself in this language. And exactly one package in the
graph carries a build script:

    ls */build.rs
    httparse/build.rs

That script runs `rustc --version` and reads `CARGO_CFG_*` variables to decide
which SIMD paths to enable. It launches a process at build time, which is a cost
worth naming because no other package here does; it compiles no C, declares no
`links` key, and reaches no network. So the target leg's problem in #291, which is
a C cross-toolchain per triple, does not arrive through this entry.

## Why the protocol and not the client, stated as records rather than as taste

0027's numbers are already values in this tree, and the file says so of itself:

    git show origin/main:src/server/transport.rs | sed -n '1,8p'

`REACHING_A_CONNECTION`, `REACHING_THE_FIRST_BYTE`, `AN_IDLE_CONNECTION_IS_KEPT_FOR`,
`REQUESTS_OUTSTANDING_TO_ONE_SERVER`, `REQUESTS_OUTSTANDING_ACROSS_ALL_SERVERS`,
`A_CANCELLED_BODY_IS_READ_FOR` and `A_CANCELLED_BODY_IS_READ_TO` are constants
there, with `CallDeadline`, `AttemptBound`, `Outstanding`, `IdleConnections` and
`ACancelledBody` deciding against them. A client that holds the same seven
quantities as its own settings does not remove that module; it makes two copies
of one decision, and the question of which is authoritative is asked at every
later change. Four readings decide the direction, and each is a landed record
rather than a preference.

**0102's clock cannot reach a client's bounds.** 0102 says all three clocks reach
the core through one injected source and that nothing in the core reads a
platform clock directly, which is what makes a timeout test take microseconds.
`ureq` holds its clock in a type the crate does not export:

    grep -n 'pub(crate) struct CurrentTime' ureq/src/timings.rs
    177:pub(crate) struct CurrentTime(Arc<dyn Fn() -> Instant + Send + Sync + 'static>);

    grep -n -A3 'impl Default for CurrentTime' ureq/src/timings.rs
    215:impl Default for CurrentTime {
    216-    fn default() -> Self {
    217-        Self(Arc::new(Instant::now))
    218-    }

The seam exists and is `pub(crate)`, so the injection 0102 requires is available
to that crate and not to a caller. Every bound `ureq` enforces is therefore real
time, and a test of one waits on it.

**Nothing in this repository would report that.** The register that refuses a
platform clock reads `src/` and judges no dependency:

    grep -n -A2 '^id: no-platform-clock' .github/invariants/rules

so a tree that moved its bounds into a client keeps a green `invariants` leg
while 0102's promise stops being provable. That is the reason this is decided in
a record rather than left to whoever writes the socket: the failure has no red
gate anywhere.

**0069's set can be left by a route the core did not choose.** 0069 fixes the
destinations as exactly the origins the operator configured, with no entry the
core adds on its own. `ureq`'s default configuration adds one:

    grep -n 'proxy: Proxy::try_from_env()' ureq/src/config.rs
    872:            proxy: Proxy::try_from_env(),
    grep -rn 'std::env::var' ureq/src/
    ureq/src/proxy.rs:234:            if let Ok(env) = std::env::var(attempt) {
    ureq/src/proxy.rs:545:            if let Ok(env) = std::env::var(attempt) {

`ALL_PROXY`, `HTTPS_PROXY`, `HTTP_PROXY` and `NO_PROXY` are read from the
environment, so on a device where one of those is set every request the core
makes goes to a host the operator never named. It is switched off with one call,
and that is the shape of the problem rather than the answer to it: a destination
rule enforced by remembering to make a call is one that survives until the day
somebody constructs the client differently, and nothing here would refuse that
day. `ureq-proto` never sees an address at all, so the set 0069 admits is passed
before anything can be reached, by construction rather than by configuration.

**0027's ending for a cancelled body is not what a client does.** 0027 reads the
remainder of a cancelled response to sixty-four kilobytes or one second and then
closes, because reading on is worth doing only while it is cheaper than the
handshake it saves. `ureq` has no `Drop` implementation anywhere in its source:

    grep -rn 'impl Drop' ureq/src ; echo "exit=$?"
    exit=1

so an abandoned body drops its connection rather than reading it to a bound, and
the connection is not returned to the pool. That is the alternative 0027 priced
and declined. In the same direction, 0027's fifth ending - every connection to
one server closed when 0029's pin for it changes - has no call. The pool type
carries four methods and none of them is that one:

    awk '/^impl /{c=$2} /^ *pub fn /{split($0,a,"("); sub(/^ *pub fn /,"",a[1]);
         print c" "a[1]}' ureq/src/pool.rs
    ConnectionPool new
    ConnectionPool connect
    ConnectionPool run_connector
    ConnectionPool pool_count
    Connection buffers
    Connection transmit_output
    Connection maybe_await_input
    Connection consume_input
    Connection close
    Connection reuse
    Connection is_tls

`close` and `reuse` are `Connection`'s and act on one connection the caller is
holding, not on every connection to an origin. Dropping the whole client closes
every origin at once, which is a different act.

None of the four is a defect in `ureq`. Each is a decision that crate made for
its own callers, and this core has already made the opposite one in a record.

## What the protocol shape hands back, and what it costs

`ureq-proto`'s own scope statement is the interface this record is choosing:

    grep -n -A 12 '# In scope' ureq-proto/src/lib.rs

In scope is the HTTP/1.1 protocol, indication of connection states, chunked
transfer encoding and 100-continue handling. Out of scope is opening and closing
sockets, TLS, request routing and body transformations. So the core writes a
request into its own buffer, reads a response out of its own buffer, and is told
whether the connection may be reused and why:

    grep -n 'pub fn must_close_connection' ureq-proto/src/client/mod.rs
    508:    pub fn must_close_connection(&self) -> bool {
    grep -n 'pub enum CloseReason' -A 8 ureq-proto/src/close_reason.rs

Every one of 0027's seven numbers is then spent where `src/server/transport.rs`
already holds it, every bound is read off 0102's injected source, and a redirect
is a state handed back rather than a request sent, so 0069's refusal is a match
arm.

What it costs is the read-write loop over
`rustls::StreamOwned<ClientConnection, TcpStream>`, written here and tested here.
That is real work and this record does not pretend otherwise. What it is not is
the thing 0103 refuses: chunked framing, content-length framing, connection
semantics and 100-continue are the protocol, and they stay in a package whose
whole subject they are. The line between the two is where `httparse` alone falls,
and that is why the floor of this set is the protocol crate and not the parser -
0103's own sentence is that a protocol, a parser of somebody else's format, or a
cryptographic primitive is a defect nobody sees until it is exploited.

The second cost is a release cadence. `ureq-proto` is `0.x`, so a minor version
is a breaking change under this ecosystem's own convention, and the two lines in
front of the board are `0.5.3` and `0.6.1`. The manifest pins `0.6`, which admits
patch releases and refuses `0.7`, so an upgrade is a deliberate act with a diff
to read. It is worth noting that the seams `ureq` itself exposes for driving its
own transport sit in a module that crate declares outside its version promise, so
the client shape does not buy stability at this layer either.

## The `client` feature, and what turning `server` off removes

`ureq-proto`'s default feature list is `client` and `server`, and this entry takes
`client` alone. The resolved graph is identical either way - seven packages on
every triple, measured both ways - so this buys no package. What it removes is
code: the `server` feature compiles a request parser, which is a second parser of
untrusted bytes in a core that will never accept a connection. 0101 treats every
byte that arrived over a network as untrusted and #86 is the corpus over the
parsers this core exposes; a parser that is compiled and unreachable is neither
covered by that corpus nor removable from a binary somebody audits.

## What this record does not do

It does not add the entry `rustls` and `rustls-platform-verifier` need. 0243
decided that and the socket in #27 is what puts it in the manifest; nothing here
moves it, and #291 still stands in front of it, because every graph that includes
0243's two lines carries a C build that the target leg cannot compile today. The
entry this record lands is pure Rust and does not reach that question.

It does not decide the protocol version. 0027 leaves that to the means and to
0010, and `ureq-proto` speaks HTTP/1.1, so the version follows from the entry
rather than being decided beside it. 0027 states its connection limit over
outstanding requests rather than over sockets precisely so that the limit means
the same thing if a later version multiplexes, and that sentence is unaffected.

It does not write the transport. Nothing in this change opens a socket, and the
module that would says of itself why it does not. #27 is where the loop is
written, and this record is what its socket is written against.

It does not decide what a request body is, how a response body is handed to a
caller, or where the buffers live. Those are the transport's shape and belong
with the code that has them.

## Why this is written down before the code

Because the alternative is the failure 0103 was written against, one layer up. A
person at a call site takes the first package that compiles, and the first package
that compiles for HTTP in this ecosystem is a client, which arrives holding
opinions about timeouts, pools, redirects and proxies that four landed records
here have already decided differently. Every one of those opinions is a default
rather than a call, so taking the client is taking four reversals in a diff whose
only visible line is a manifest entry.

## Alternatives, and what each cost

`ureq`, configured to 0027. The cheapest to start and the one a reader will ask
about first, since it is `ureq-proto`'s own caller and it takes 0243's means in
whole. Its costs are the four above, and the one that decides it is 0102: through
this client every bound is real time, the injection point is not exported, and no
route in this repository would report the loss. The other three are individually
answerable - one call for the proxy, a second copy of the numbers for the bounds,
an accepted difference for the cancelled body - and answering each of them is
where the second copy of 0027 gets written.

`hyper`. The most capable and the only candidate with no thread of its own. Its
shape is the refusal: `handshake` is an `async fn` over poll-based `Read + Write`,
the connection is a future that needs an `Executor`, and every bound needs a
`Timer` implementation.

    grep -n 'pub trait Executor' -A 3 hyper/src/rt/mod.rs
    grep -n 'pub trait Timer' -A 3 hyper/src/rt/timer.rs
    grep -n 'pub async fn handshake' hyper/src/client/conn/http1.rs

Over 0009's two owned lanes that is an executor and a readiness adapter written
here, or a runtime taken in - and "a runtime the client must host" is an
alternative 0009 names and declines. It is also fourteen packages rather than
five, with `tokio` in the graph regardless.

`reqwest` and `isahc`. Refused by 0009 and by 0103's third behaviour, on their own
source rather than on reputation: `reqwest`'s blocking client starts a thread and
runs a current-thread runtime inside it, and `isahc` runs a background thread
driving libcurl.

    grep -rn 'thread::Builder' reqwest/src/blocking/client.rs isahc/src/agent/mod.rs
    reqwest/src/blocking/client.rs:1414:        let handle = thread::Builder::new()
    isahc/src/agent/mod.rs:144:                thread::Builder::new()

    sed -n '1414,1419p' reqwest/src/blocking/client.rs
            let handle = thread::Builder::new()
                .name("reqwest-internal-sync-runtime".into())
                .spawn(move || {
                    use tokio::runtime;
                    let rt = match runtime::Builder::new_current_thread()
                        .enable_all()

`isahc` additionally adds `curl-sys` and `libz-sys`, which is #291's C
cross-toolchain cost paid a second time on every triple, and it carries `tracing`
as a dependency with no `optional` line beside it, which is 0061's fifth ground
arriving unconditionally:

    grep -n -A2 'dependencies.tracing]' isahc/Cargo.toml
    332:[dependencies.tracing]
    333-version = "0.1.17"
    334-

`attohttpc`. Refused by 0243 before 0103 is reached: it offers no
platform-verifier route at all, which is what 0029 requires and what 0243
recorded as refused in the other direction.

    grep -rl 'platform.verifier' attohttpc/ ; echo "exit=$?"
    exit=1

It also spawns a thread per resolved address in its connect race and a shutdown
thread per request whenever a deadline is set, and 0009 says no timer thread
exists on its own:

    grep -n 'thread::spawn' attohttpc/src/happy.rs attohttpc/src/streams.rs
    attohttpc/src/happy.rs:64:        thread::spawn(move || {
    attohttpc/src/streams.rs:148:                thread::spawn(move || {

`minreq`. The smallest full client, and refused on the same third behaviour: it
enforces its one deadline by parking the caller on a spawned thread.

    grep -n 'std::thread::spawn' minreq/src/connection.rs
    365:            let thread = std::thread::spawn(move || {

Its rustls configuration is a process-wide `static` built from
`ClientConfig::builder()`, which takes the process default provider rather than
one the caller hands over, so 0243's own seam is not available through it:

    grep -n 'static CONFIG' minreq/src/connection/rustls_stream.rs
    22:static CONFIG: LazyLock<Result<Arc<ClientConfig>, rustls::Error>> = ...

And it pins the verifier a minor version behind 0243's, which would put two
copies of that crate in one graph:

    grep -n -A2 'dependencies.rustls-platform-verifier]' minreq/Cargo.toml
    131:[dependencies.rustls-platform-verifier]
    132-version = "0.6.2"
    133-optional = true

`httparse` with `http`, three packages and the smallest graph of all. Refused by
0103's own sentence: chunked framing, content-length framing, connection
semantics and 100-continue would then be written in `src/`, and that is a protocol
implemented here.

Writing HTTP/1.1 with no dependency at all. The same refusal one step further,
and it also gives up `http`'s types, which are what the ecosystem's own tooling
and every later reader expect.

## What would reverse this

The loop this shape requires is measured to be larger or more defect-prone than
the four reversals a client would cost. That is a judgement a reader of #27's
diff can make and nobody can make today, and it is the honest reversal condition
rather than a number.

`ureq` exports its clock seam, so that 0102's injected source can drive its
bounds, and gains a per-origin close. Two of the four objections then go, the
proxy default is one call, and the second copy of 0027's numbers is the whole of
what is left to argue about.

`ureq-proto` reaches `1.0` with a stability promise, which removes the cadence
cost, or stops being released at all, which is the removal condition written
beside the manifest entry.

The core stops speaking HTTP/1.1 - because 0010's server surface moves, or
because a later protocol version is required - and the entry is then judged again
against whatever speaks the new one.

The standard library gains an HTTP client. 0011 names this shape as what retires
a dependency taken for one of its five measured absences, and this is the absence
this entry was taken for.
