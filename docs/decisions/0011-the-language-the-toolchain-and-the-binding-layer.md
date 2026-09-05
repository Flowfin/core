# 0011. The language, the toolchain, and the binding layer

Date: 2026-08-24

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #11

## The decision

The core is one Rust library, built and tested with the stable `cargo` toolchain
pinned in the tree, reaching each client through a foreign function interface
generated per platform, chosen because the properties the records already
standing depend on are ones a compiler refuses rather than ones a reviewer
remembers.

## Where the answer came from, and what this record adds

Entry 2 of #1 was answered on 2026-08-24, and the answer is the first of the four
candidates that entry priced:

    gh issue view 1 --repo Flowfin/core --json comments --jq '.comments[-1].body' | grep -A2 '^Entry 2:'
    Entry 2: Rust with a foreign function interface per platform. One implementation
    for eleven clients, memory-safe by default - which this issue itself names the
    property that costs the most to buy any other way. The binding layer is generated

That is the choice. What this record adds is the part an answer to a question
cannot carry: what the choice costs in this repository's own terms, what it
forecloses, what would reverse it, and whether the means meets each condition the
records that landed while nobody had chosen a language wrote against it.

Eleven of those records name this issue:

    git grep -l '#11\b' -- docs/decisions | wc -l
    11

## What was measured, and on what

Every answer below about the language was produced by compiling or running a
program against the toolchain named here, on this host:

    rustc -vV
    rustc 1.97.0 (2d8144b78 2026-07-07)
    binary: rustc
    commit-hash: 2d8144b7880597b6e6d3dfd63a9a9efae3f533d3
    commit-date: 2026-07-07
    host: x86_64-pc-windows-msvc
    release: 1.97.0
    LLVM version: 22.1.6

A reader reproduces a block below by saving the program in it under the file name
its first line carries and running the command beneath it. The version is stated
because several of the answers are version-dependent, and one of them is a feature
expected to stabilise.

## The toolchain

`cargo` is the build tool, the test runner and the dependency resolver, and it
arrives with the compiler rather than being a second thing to install:

    cargo --version
    cargo 1.97.0 (c980f4866 2026-06-30)

Two of the gate's check names in M2 are toolchain components rather than new
tools. `rustfmt` answers #18 and `clippy` answers #17:

    rustup component list --installed
    cargo-x86_64-pc-windows-msvc
    clippy-x86_64-pc-windows-msvc
    rust-docs-x86_64-pc-windows-msvc
    rust-std-x86_64-pc-windows-msvc
    rustc-x86_64-pc-windows-msvc
    rustfmt-x86_64-pc-windows-msvc

#19 asks for a committed lockfile and a restore that refuses to change it, and the
flag that refuses is the build tool's own:

    cargo build --help | grep -E '^ +--(locked|offline|frozen)\b'
          --locked                Assert that `Cargo.lock` will remain unchanged
          --offline               Run without accessing the network
          --frozen                Equivalent to specifying both --locked and --offline

The edition is 2024. Every program in this record compiled under `--edition 2024`
on the compiler above.

Which exact version is pinned, and in which file, is #14 and is not decided here.
A version written into this record would be a second declaration of the same fact,
and the two would disagree at the first upgrade.

## What the choice costs

**One build leg per target triple.** The library is compiled for each platform a
client runs on, and the compiler knows this many of them:

    rustc --print target-list | wc -l
    322

That number is not the gate's; which triples the gate builds and tests is #113,
and this record says only that the count is a per-target count rather than one.
The suite is the other half: the core's own tests are one run on one host, because
they test the library rather than the binding, while the conformance suite in #76
is per-client by construction.

**One runtime a contributor installs, and a second one for one gate leg.** The
toolchain above is the whole requirement for building and testing the core. The
exception is the detector in #117, which is measured below and needs a nightly
compiler.

**A binding layer that is code.** An interface generated per platform is a
generated artefact crossing a boundary the compiler stops checking, so it is
tested rather than assumed, which is what the answer to entry 2 itself says. Which
generator produces it is not decided here: a generator is a dependency, the rule
that admits a dependency is #103, and choosing one before that rule exists is the
case #103 was opened against.

**The licence reaches the clients through linking.** Entry 1 of #1 is answered by
the fleet-wide AGPL-3.0-or-later answer, and the repository already publishes
under it:

    gh api repos/Flowfin/core --jq '.license.spdx_id'
    AGPL-3.0

A client embeds this library, so entry 2's answer is what carries entry 1's answer
to eleven clients rather than stopping at this repository. That follows from the
pair and is not a decision of this record.

Entry 1 was answered a second time and the answer above is the one that was
superseded. The answer that stands is
[0303](0303-the-licence-the-core-is-offered-under.md).

## What the standard library supplies, and what it does not

This section exists because the difference decides how large the dependency graph
in #103 has to be, and because several records already standing rest on one side
of it or the other.

**A duration type, unsigned and in nanoseconds.**

    // dur.rs
    use std::time::Duration;
    fn main() {
        println!("size_of::<Duration>() = {}", std::mem::size_of::<Duration>());
        println!("Duration::MAX = {:?}", Duration::MAX);
        println!("as_nanos(1s) = {}", Duration::from_secs(1).as_nanos());
    }

    rustc --edition 2024 -O -o dur dur.rs && ./dur
    size_of::<Duration>() = 16
    Duration::MAX = 18446744073709551615.999999999s
    as_nanos(1s) = 1000000000

It cannot hold a negative value, and what refuses one is the type rather than a
convention:

    // durneg.rs
    use std::time::Duration;
    fn main() { let _ = Duration::new(-1i64, 0); }

    rustc --edition 2024 -o durneg durneg.rs
    error[E0308]: mismatched types
     --> durneg.rs:2:35
      |
    2 | fn main() { let _ = Duration::new(-1i64, 0); }
      |                     ------------- ^^^^^ expected `u64`, found `i64`
      = note: `-1i64` cannot fit into type `u64`

**A connection attempt bounded separately from a read, against an address that
exists before the connection does.**

    // net.rs
    use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
    use std::time::Duration;
    fn main() {
        let addrs: Vec<SocketAddr> = "localhost:9".to_socket_addrs().unwrap().collect();
        println!("resolved before any connect: {addrs:?}");
        let f: fn(&SocketAddr, Duration) -> std::io::Result<TcpStream> = TcpStream::connect_timeout;
        let s: fn(&TcpStream, Option<Duration>) -> std::io::Result<()> = TcpStream::set_read_timeout;
        println!("connect_timeout takes a resolved address: {}", f as usize != 0);
        println!("set_read_timeout is a separate bound: {}", s as usize != 0);
    }

    rustc --edition 2024 -o net net.rs && ./net
    resolved before any connect: [[::1]:9, 127.0.0.1:9]
    connect_timeout takes a resolved address: true
    set_read_timeout is a separate bound: true

**A processor count whose floor is in the type.**

    // par.rs
    fn main() { println!("{}", std::thread::available_parallelism().unwrap()); }

    rustc --edition 2024 -o par par.rs && ./par
    32

The return type is a non-zero integer, so the floor of one that 0009 requires for
the processing lane cannot be lost in the subtraction that sizes it.

**No source of unpredictable bytes on the stable compiler.** There is one in the
standard library and a stable build cannot reach it:

    // rnd.rs
    fn main() { let x: u128 = std::random::random(); println!("{x}"); }

    rustc --edition 2024 -o rnd rnd.rs
    error[E0658]: use of unstable library feature `random`
     --> rnd.rs:1:27
      = note: see issue #130703 <https://github.com/rust-lang/rust/issues/130703> for more information

**No cryptographic digest.** The standard library offers one hasher, it is sixty
four bits wide, and it is not a digest:

    // dig.rs
    use std::hash::{DefaultHasher, Hasher};
    fn main() { let mut h = DefaultHasher::new(); h.write(b"a"); println!("{:016x}", h.finish()); }

    rustc --edition 2024 -o dig dig.rs && ./dig
    407448d2b89b1813

    // dig2.rs
    fn main() { let _ = std::hash::Sha256::new(); }

    rustc --edition 2024 -o dig2 dig2.rs
    error[E0433]: cannot find `Sha256` in `hash`

**No transport security and no HTTP.** The networking module reaches TCP and
stops:

    // tls.rs
    fn main() { let _ = std::net::TlsStream::connect("example:443"); }

    rustc --edition 2024 -o tls tls.rs
    error[E0433]: cannot find `TlsStream` in `net`

So certificate validation in #29, the transport in #27 and every request the core
makes rest on something outside the standard library, and what may be taken is
#103's rule. That is the largest single cost of this choice, and it is named here
rather than discovered at the first request.

## The conditions the standing records put on the means

Each record below wrote a condition against a language nobody had chosen. Each is
answered here with what was measured, or with the statement that it is not met.

**0009, the concurrency model.** Two lanes the core owns, completion-based calls,
and no runtime hosted by the client. The lanes are operating-system threads, the
sizing input is the processor count above, and no scheduler outside the standard
library is required, so the model is carried without an executor and without the
hosted runtime that record's third alternative priced. **Met.**

**0009's reversal condition, the detector in #117.** That record says the
calling-thread guarantee becomes an unproven claim if the detector cannot be run
on the toolchain chosen here. It can be run, and the answer carries two bounds
worth having in one place. It is a nightly compiler flag:

    // tsan.rs
    fn main() { println!("x"); }

    rustc --edition 2024 -Zsanitizer=thread -o tsan tsan.rs
    error: the option `Z` is only accepted on the nightly compiler

and it is supported on some targets and not on others:

    for t in x86_64-unknown-linux-gnu aarch64-apple-darwin aarch64-apple-ios aarch64-linux-android x86_64-pc-windows-msvc; do
      printf '== %s\n' "$t"
      rustup run nightly rustc -Zunstable-options --print target-spec-json --target "$t" \
        | sed -n '/supported-sanitizers/,/]/p' | grep '"thread"' || echo '   no thread sanitizer'
    done
    == x86_64-unknown-linux-gnu
        "thread",
    == aarch64-apple-darwin
        "thread",
    == aarch64-apple-ios
        "thread",
    == aarch64-linux-android
       no thread sanitizer
    == x86_64-pc-windows-msvc
       no thread sanitizer

**Met, on a second toolchain and not on every target.** The claims 0009 makes are
about the core's own code rather than about a platform, so a run on a host that
supports the detector verifies them. What stays out of reach is a race that
manifests only on Android or on Windows, and #117 states that bound rather than
reporting a clean run over a set it did not cover.

**0027 and 0069, a bounded connection attempt and a connection seen before it is
made.** Both are the networking measurement above: the attempt is bounded by one
call and the read by another, and the destination is a resolved address in hand
before anything is dialled. 0069's more serious reversal condition is not reached,
and #70 has something to observe. **Met.**

**0030, a credential whose bytes can be cleared on a schedule the runtime
guarantees.** The point at which a value is dropped is fixed by the language
rather than by a collector, so the timing half exists. The overwriting half does
not: nothing in the standard library promises to erase the bytes of a string
before its allocation is released, and the compiler is free to have copied them
first. This is a claim rather than a measurement, because the reading that would
prove it is a read of freed memory. **Not met**, so 0030's residual stands exactly
as that record wrote it and its reversal condition is not triggered.

**0032 and 0036, a source of unpredictable bytes of at least 128 bits.** Measured
above: the standard library's source is unstable on the stable compiler. **Not met
on the toolchain as chosen**, so the seam 0032 already named is what is used, the
client supplies the bytes, and 0036 pays for it a second time on the device
identity. Neither record is superseded, because both wrote this outcome as a case
rather than as a failure. It is not the state 0032's reversal condition describes
either: what that condition refuses is no client on any platform being able to
supply the bytes, and every platform in view has such a source.

**0037, one construction point for the failure set.** A value of the set cannot be
built outside the module that owns it, and the compiler is what refuses:

    // errset.rs, built as a library
    pub struct Failure(Kind);
    #[non_exhaustive]
    pub enum Kind { NotAuthenticated, Unreachable }
    impl Failure {
        pub fn map(io: &std::io::Error) -> Failure {
            match io.kind() {
                std::io::ErrorKind::PermissionDenied => Failure(Kind::NotAuthenticated),
                _ => Failure(Kind::Unreachable),
            }
        }
    }

    // caller.rs, built against it
    extern crate errset;
    use errset::{Failure, Kind};
    fn main() { let _ = Failure(Kind::Unreachable); }

    rustc --edition 2024 --crate-type lib --crate-name errset -o liberrset.rlib errset.rs
    rustc --edition 2024 --extern errset=liberrset.rlib -o caller caller.rs
    error[E0423]: cannot initialize a tuple struct which contains private fields
     --> caller.rs:4:13
      |
    4 |     let _ = Failure(Kind::Unreachable);
      |             ^^^^^^^
      = note: constructor is not visible here due to private fields

**Met**, and by a refusal rather than by a check over the tree, which is the
stronger of the two routes 0037 names.

**0041 and 0105, a cryptographic digest without a dependency #103 refuses.** The
standard library has none, measured above. The requirement is therefore met by a
dependency or stated as unmet, and which of the two is #103's to answer rather
than this record's. **Not met by the toolchain alone**, and this is the sentence
0041's reversal condition asked for: that condition is reached, and what it calls
for is a new record about the digest rather than a line added to 0041.

**0056, the duration type the first position-holding code will reach for.** The
measurement above is the case 0056 predicted: the type is unsigned, its resolution
is nanoseconds, and it does not agree with a tick. 0056 fixes the wire unit against
the server rather than against this type, so the conversion at the boundary is the
one that record already requires, now written against a measured shape rather than
against an unknown one. **Met, in the sense 0056 asked for**, which is that the
runtime's type does not decide the unit.

**0071, per-field classification that is enforced rather than remembered.** A field
reaches the sink only through a trait it must implement, and a field whose
treatment nobody chose does not compile:

    // cls.rs
    pub trait Classified { fn rendered(&self) -> String; }
    pub struct ServerId(pub u32);
    impl Classified for ServerId { fn rendered(&self) -> String { format!("server {}", self.0) } }
    pub struct Password(pub String); // no impl: no treatment was chosen
    pub fn emit(field: &dyn Classified) { println!("{}", field.rendered()); }
    fn main() { emit(&ServerId(7)); emit(&Password("hunter2".into())); }

    rustc --edition 2024 -o cls cls.rs
    error[E0277]: the trait bound `Password: Classified` is not satisfied
      --> cls.rs:13:10
       |
    13 |     emit(&Password("hunter2".into()));
       |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound

**Met.** 0071's default, that a field with no treatment is excluded, is a
compilation failure here rather than a convention, so that record is not claiming
a property nothing keeps.

## What this record does not decide

The exact toolchain version and where it is pinned. #14.

The directory layout, the crate names, and the two commands a fresh clone runs.
#13. 0009 expects the type names for its per-kind statements to arrive with one of
those two issues, and #13 is the one with names in it.

Which target triples the gate builds and tests, and what no run covers. #113.

What a dependency has to be worth, which licences may appear in the graph, and
what is refused outright. #103. Three of the absences measured above are questions
that record answers and this one does not.

Which digest function and which width. 0041 and 0105 already say this follows the
toolchain; what follows from the measurement here is that it cannot follow from
the toolchain alone.

Which generator produces the binding layer. It is a dependency, so it is #103's
rule applied once that rule exists.

## Why this is written down before the code

The failure this record is against has a shape, and it is not that somebody picks
the wrong language. It is that the language gets picked by the first file. A
repository with no recorded means acquires one the first time somebody needs to
compile something, the choice is then defended by the work already done in it, and
the conditions above are met or missed by accident rather than checked.

The second failure is narrower and more expensive. Five of the answers above are
absences: no unpredictable bytes on a stable build, no digest, no transport
security, no HTTP, and no promise about clearing a credential's bytes. Each is a
dependency-shaped hole, and a hole met at a call site is filled with whatever the
person at that call site reached for. Writing them down together, before the first
call site exists, is what makes them one question for #103 instead of five answers
nobody compared.

## Alternatives, and what each cost

The four candidates are entry 2 of #1's, and what follows is what each would have
cost against the conditions measured above rather than in general.

Kotlin Multiplatform. Cheap on Android and on a Java desktop, and it carries a
cryptographic digest and a source of unpredictable bytes in its own standard
library, so two of the absences above would not exist. It costs an extra runtime
on Apple platforms and on a television, it decides the language of every client
rather than leaving that open, and 0030's residual gets worse rather than better,
because a collected runtime fixes neither the timing nor the overwriting of a
credential's bytes.

C++. Reaches every target with the least ceremony and has the largest supply of
libraries for the five absences. Every memory-safety property then has to be
bought with tooling, review and sanitiser runs that are themselves gate legs
somebody maintains, and 0037's single construction point and 0071's per-field
classification become conventions a reviewer checks rather than refusals a
compiler makes. The property that decided against it is the one #1 itself names as
the most expensive to buy any other way.

No shared code, a specification plus a conformance suite. It costs eleven
implementations of every condition above and makes the specification the artefact
that has to be perfect, because nothing else is shared. It also empties this
record: there is no means to choose, and 0009's guarantee about the calling thread
becomes eleven promises nobody can verify in one place.

Rust with each client building the source rather than linking a built library. Not
in entry 2's list, and worth naming because it is the shape somebody proposes
next. It removes the binding layer and replaces it with a build of the core inside
every client's build system, which is eleven build integrations instead of one
generated interface, and it makes the version a client runs a property of that
client's checkout rather than of a released artefact.

## What would reverse this

A target the eleven clients need turns out not to be reachable by this compiler at
all, so that a client cannot link the core rather than paying a cost to. The
target list is what says whether this has happened, and it is a comparison against
a named triple rather than a judgement.

The detector in #117 stops being available on every target that carries it today,
or is measured to be unusable against the core's own suite. 0009's reversal
condition is then reached through this record, the calling-thread guarantee has no
route to verification, and either the means changes or that guarantee is withdrawn
and written as a claim. This is the condition to watch, because it is the one this
record answers with a second toolchain rather than with the pinned one.

#103's rule, once written, refuses every candidate for transport security, so that
the core cannot make a request at all under the licence and safety conditions that
rule sets. That is not a reason to change the language, and it is written here so
it is not read as one: it would mean the rule and the transport are in conflict and
one of the two records is wrong, which is a decision above both.

The stable compiler gains a source of unpredictable bytes, which is the one absence
above that is expected to move; the tracking issue in the error text is where that
is decided. When it lands in a pinned stable version, the seam 0032 and 0036
describe stops being necessary, and both of those records are superseded by one
that takes the source directly rather than this record being edited.
