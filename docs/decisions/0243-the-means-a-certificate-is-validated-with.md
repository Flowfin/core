# 0243. The means a certificate is validated with, and what it costs

Date: 2026-08-31

Status: accepted. Supersedes nothing. Superseded by nothing.

Narrows: 0103, on the fourth behaviour refused outright, a dependency that writes to a log

Issue: #243

## The decision

The core validates a certificate through `rustls` driving
`rustls-platform-verifier`, which dispatches to each platform's own verifier,
admitted under [0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s
clause for a dependency a landed record already requires - the record being
[0029](0029-certificate-validation-and-the-self-signed-server.md), which requires
the platform's own trust store and the platform's own path building and refuses a
client-supplied evaluation by name - with that record's fourth refused behaviour
narrowed here to writing to a log rather than to linking a logging facade, on the
standing condition that the core installs no logger and states that it installs
none.

## What was actually in the way, and it was not the choice of package

Two landed records refused each other's answer, and that is what #243 turned out
to be about. Every candidate that satisfies
[0029](0029-certificate-validation-and-the-self-signed-server.md) carries a
logging facade, which
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) refuses outright as
its fourth behaviour. In the verifier crate it is ten call sites and a
non-optional entry in the manifest, so it is not a feature that can be switched
off:

    grep -rn 'log::' src/verification/ | grep -c '!'
    10
    grep -n -A2 'dependencies.log' Cargo.toml
    74:[dependencies.log]
    75-version = "0.4"
    76-

The one shape
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) admits - `rustls`
with `rustls-native-certs` for the roots and `rustls-webpki` for the path
building - treats every root equally regardless of its status, which is the
platform's store without the platform's decisions, and
[0029](0029-certificate-validation-and-the-self-signed-server.md) refuses that.

[0011](0011-the-language-the-toolchain-and-the-binding-layer.md) wrote this exact
case down as its own reversal condition before there was a graph to measure it
against, and
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) says the same from
its own side: where the rule refuses what a landed record requires, which of the
two moves is a decision above both rather than an exception written into either.
That decision was taken on #243 on 2026-08-30, and this record executes it.

The reason it went the way it did is what each record is protecting.
[0029](0029-certificate-validation-and-the-self-signed-server.md) protects the
platform's own trust decisions and its revocation state, and no rewording
preserves that if the core stops asking the platform.
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s fourth behaviour
protects against a second exit for the values
[0071](0071-what-may-leave-through-a-diagnostic-event.md) classifies field by
field, and the facade's own default sink writes nothing. Read out of the facade's
own source at `log v0.4.34`, the version the graph below resolves:

    grep -n '' src/lib.rs | sed -n '456p;1318,1327p'
    456:static mut LOGGER: &dyn Log = &NopLogger;
    1318:struct NopLogger;
    1319:
    1320:impl Log for NopLogger {
    1321:    fn enabled(&self, _: &Metadata) -> bool {
    1322:        false
    1323:    }
    1324:
    1325:    fn log(&self, _: &Record) {}
    1326:    fn flush(&self) {}
    1327:}

So a facade with no logger installed is not an exit, and the property
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) exists to guarantee
is untouched by linking one.

## The narrowed rule, stated in full

A dependency that WRITES to a log is refused, exactly as before. A dependency
that links a logging facade whose sink is absent is not refused, provided the
core installs no logger and states that it installs none.

The condition is not decoration. A facade is harmless because nothing is
registered behind it, and the moment the core registers something the fourth
behaviour is back in force with nothing to have caught the change. The core
installs no logger today, and the two readings that say so are this:

    git grep -n 'set_logger\|set_boxed_logger' origin/main -- src/ ; echo "exit=$?"
    exit=1
    git grep -n '^log =\|^tracing' origin/main -- Cargo.toml ; echo "exit=$?"
    exit=1

Nothing refuses a logger installed tomorrow. `no-text-output` in
`.github/invariants/rules` refuses `println!` and its neighbours under `src/` and
reaches no logger installation, so the condition above is carried by this record
and by review. #266 is where a rule that refuses it is asked for.

## What this record does to 0103, and what it does not

The narrowing is written here rather than into
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s own text, and the
reason is that record's neighbour.
[0001](0001-decision-records.md) permits three edits to a landed record and a
change to what it decided is not among them; a narrowing is a change to what it
decided. What
[0001](0001-decision-records.md) does permit is a pointer to a later record that
goes further on a case the earlier one already names, where the pointer changes
no sentence's meaning and takes no reason away, and that is what
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) and
[0029](0029-certificate-validation-and-the-self-signed-server.md) each receive.

THE RESIDUAL IS REAL AND IT IS STATED RATHER THAN SOFTENED.
[0001](0001-decision-records.md) offers whole-record supersession and no partial
one, and superseding
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) entirely would
discard a licence set, a worth test and four other grounds that are unchanged. So
a reader who opens
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) and does not follow
its pointer reads its fourth behaviour one clause wider than the rule in force.
Nothing refuses that reader. #267 is where the shape of a partial supersession is
asked for, and this record does not invent one.

## How the readings below were taken

Every `cargo` reading in this record was taken in a scratch crate outside this
tree, on a Windows machine, with the toolchain `rust-toolchain.toml` pins. The
manifest of that crate declares two dependencies and nothing else:

    rustls = "0.23"
    rustls-platform-verifier = "0.7"

resolving to `rustls v0.23.43`, `rustls-platform-verifier v0.7.0` and the crypto
provider `rustls` names first in its own `default` list. The gate runs on
`ubuntu-latest`, so the cross-compile results in particular are this machine's and
not the runner's. That bound is stated once and applies to every number here.

The tree these are measured against:

    git rev-parse origin/main
    ded6d1fcbd9ad65b5a04fbf58c0a7d0e34dd2c10

## What the graph is, counted rather than supposed

Per triple, the shipping graph beside the crate itself:

    cargo tree -e normal --target TRIPLE --prefix none | sed 's/ (\*)$//' \
      | sort -u | grep -v '^verifier-probe' | grep -v '^$' | wc -l
    aarch64-linux-android        29
    armv7-linux-androideabi      29
    aarch64-apple-ios            17
    aarch64-apple-tvos           17
    aarch64-apple-darwin         17
    x86_64-pc-windows-msvc       13
    x86_64-unknown-linux-gnu     13

For scale, what this tree carries today is nine packages beside its own:

    cargo metadata --format-version 1 --locked | jq -r '.packages[].name' | wc -l
    10

The union across all seven triples is thirty-nine distinct packages, and that is
the number to hold rather than any single triple's, because every client links one
of the seven and the eleven of them link all of them between them.

## The licence, read rather than assumed

Every licence expression carried by the thirty-nine, derived rather than eyeballed:

    cargo metadata --format-version 1 --locked \
      | jq -r '.packages[] | "\(.name) v\(.version)\t\(.license)"' | sort > licences
    for t in TRIPLE...; do
      cargo tree -e normal --target "$t" --prefix none | sed 's/ (\*)$//;s/ (proc-macro)$//'
    done | grep -v '^verifier-probe' | grep -v '^$' | sort -u > shipping
    awk -F'\t' 'NR==FNR{l[$1]=$2;next}{print l[$0]}' licences shipping | sort | uniq -c | sort -rn
         26 MIT OR Apache-2.0
          2 MIT
          2 ISC
          2 Apache-2.0 OR MIT
          2 Apache-2.0 OR ISC OR MIT
          1 Unlicense OR MIT
          1 ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0)
          1 ISC AND (Apache-2.0 OR ISC)
          1 BSD-3-Clause
          1 (MIT OR Apache-2.0) AND Unicode-3.0

Nine of the ten expressions are satisfied by
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s admitted set,
including the two conjunctive ones, because every member of each conjunction is
in the set and each dual offer inside them includes an admitted member. `MIT-0`
appears only inside a dual offer beside `Apache-2.0` and `ISC`, so it is admitted
on those.

THE TENTH IS NOT, AND IT IS THE ONE COLLISION THIS RECORD LEAVES OPEN.
`unicode-ident` is offered as `(MIT OR Apache-2.0) AND Unicode-3.0`. The term is
conjunctive rather than a dual offer, so
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s sentence about a
dual offer does not reach it, and `Unicode-3.0` is named in neither the admitted
half of that set nor the refused half. It arrives through a proc-macro rather
than through anything linked into a client, and it is present on exactly two of
the seven triples:

    cargo tree -e normal --target TRIPLE --prefix none | grep -c '^unicode-ident'
    aarch64-linux-android     3
    armv7-linux-androideabi   3
    aarch64-apple-ios         0
    aarch64-apple-tvos        0
    aarch64-apple-darwin      0
    x86_64-pc-windows-msvc    0
    x86_64-unknown-linux-gnu  0

This record does not admit that term and does not refuse it. Extending an
enumerated licence set is a change to
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) of the kind this
record's own section above says it may not make, and it is a different question
from the one #243 asked. #268 is where it is asked, and until it is answered the
Android half of this means is a dependency this board has not licensed.

## Where the platform's own path building stops

This is the first thing #243 asks and the answer is not uniform. The crate's own
README states its dispatch per platform, and the row that matters is the second
from last:

    sed -n '20,27p' README.md
    | OS             | Certificate Store                             | Verification Method                  | Revocation Support |
    |----------------|-----------------------------------------------|--------------------------------------|--------------------|
    | Windows        | Windows platform certificate store            | Windows API certificate verification | Yes                |
    | macOS (10.14+) | macOS platform roots and keychain certificate | macOS `Security.framework`           | Yes                |
    | iOS            | iOS platform roots and keychain certificates  | iOS `Security.framework`             | Yes                |
    | Android        | Android System Trust Store                    | Android Trust Manager                | Sometimes[^1]      |
    | Linux          | System CA bundle, or user-provided certs[^3]  | webpki                               | No[^2]             |
    | WASM           | webpki roots                                  | webpki                               | No[^2]             |

The last row is outside
[0113](0113-the-target-triples-the-gate-compiles-for.md)'s set and is not a
platform this decision covers.

On six of the seven triples the path building is the platform's. On the Linux
desktop client it is `rustls-webpki` over whatever root bundle the system happens
to hold, with no revocation at all, which the graph shows directly - that is the
one triple whose shipping set carries `rustls-native-certs` and `openssl-probe`
and no platform framework.

So [0029](0029-certificate-validation-and-the-self-signed-server.md)'s sentence
that the core does not reimplement the platform's path building is met on six
triples and has nothing to meet on the seventh, because that platform has no path
building of its own to use. Every candidate does its own there, so this is a
departure the means does not cause and cannot avoid, and it is recorded rather
than left for somebody to discover in a refusal.

## Whether 0029's six reason classes are derivable

They are not, and this record does not pretend the question is settled by a
reading of source. What the crate maps, at the version measured, is narrower than
six classes on every backend, and the crate says so itself above each of the two
mapping tables:

    grep -rn 'Only map' src/
    src/verification/apple.rs:220:                // Only map the errors we need for tests.
    src/verification/windows.rs:677:        // Only map the errors we have tests for.

What each of the three backends reaches, and the Android one is the third of them:

    grep -n '=> InvalidCertificate' src/verification/windows.rs
    684:            CRYPT_E_REVOKED => InvalidCertificate(CertificateError::Revoked),
    685:            CERT_E_EXPIRED => InvalidCertificate(CertificateError::Expired),
    686:            CERT_E_UNTRUSTEDROOT => InvalidCertificate(CertificateError::UnknownIssuer),
    687:            CERT_E_WRONG_USAGE => InvalidCertificate(CertificateError::InvalidPurpose),

    grep -n 'errors::errSec' src/verification/apple.rs
    222:                    errors::errSecHostNameMismatch => Ok(TlsError::InvalidCertificate(
    225:                    errors::errSecCreateChainFailed => Ok(TlsError::InvalidCertificate(
    228:                    errors::errSecInvalidExtendedKeyUsage => Ok(TlsError::InvalidCertificate(
    231:                    errors::errSecCertificateRevoked => {

    grep -n 'VerifierStatus::' src/verification/android.rs | sed -n '1,5p'
    216:                    VerifierStatus::Expired => Err(InvalidCertificate(CertificateError::Expired)),
    219:                        Err(InvalidCertificate(CertificateError::UnknownIssuer))
    223:                        Err(InvalidCertificate(CertificateError::Revoked))
    226:                        Err(InvalidCertificate(CertificateError::BadEncoding))
    228:                    VerifierStatus::InvalidExtension => Err(InvalidCertificate(

Three consequences for
[0029](0029-certificate-validation-and-the-self-signed-server.md):

`self-signed` and `issuer-unknown` arrive as one value on all three backends: the
one that reaches an unknown issuer is `UnknownIssuer` and there is no second value
beside it for a certificate that signed itself.

`not-yet-valid` is not separated from `expired` on Windows, on Apple or on
Android, which is the pair
[0029](0029-certificate-validation-and-the-self-signed-server.md) names
separately on purpose for the television that came up believing it is 1970.

`revoked` is a class the platforms report and
[0029](0029-certificate-validation-and-the-self-signed-server.md) does not carry.
It would land in `chain-unusable`, and a client then cannot tell a revoked
certificate from an unusable chain, which is the one difference an operator can
act on.

THAT IS A READING OF SOURCE AND NOT A REFUSAL ON A WIRE, so it does not discharge
[0029](0029-certificate-validation-and-the-self-signed-server.md)'s own reversal
condition, which requires two platforms producing different classes for one
certificate measured on a real refusal. What this record gives that condition is
the thing it did not have: something to take the measurement with. The
measurement itself belongs to #29 and #21.

## What it costs the gate, which is the largest cost and is not the graph

`rustls` needs a crypto provider, and both providers it offers are C. Its own
`default` list names the first, and the only other is the second. Read out of
`rustls v0.23.43`'s own manifest:

    grep -n '' Cargo.toml | sed -n '70,76p;88,91p'
    70:default = [
    71:    "aws_lc_rs",
    72:    "logging",
    73:    "prefer-post-quantum",
    74:    "std",
    75:    "tls12",
    76:]
    88:ring = [
    89:    "dep:ring",
    90:    "webpki/ring",
    91:]

Both carry a native build:

    cargo metadata --format-version 1 --locked | jq -r '.packages[]
      | select(.name=="aws-lc-sys" or .name=="ring")
      | "\(.name) \(.version) links=\(.links) build=\(.dependencies|map(select(.kind=="build"))|map(.name)|join(","))"'
    aws-lc-sys 0.44.0 links=aws_lc_0_44_0 build=bindgen,cc,cmake,dunce,fs_extra,pkg-config
    ring 0.17.14 links=ring_core_0_17_14_ build=cc

Today the target set compiles the library for all seven triples on one runner and
needs no C toolchain at all, because the graph is pure Rust. Taking either
provider ends that. On this machine, with all seven target standard libraries
installed and the pinned toolchain, six of the seven triples fail inside
`aws-lc-sys`'s build script rather than anywhere in Rust:

    for t in TRIPLE...; do cargo check --locked --target "$t"; done

The first Android triple, as one exact line of what the six look like:

    warning: aws-lc-sys@0.44.0: Compiler family detection failed due to error: ToolNotFound: failed to find tool "aarch64-linux-android-clang": program not found

What the other five reported, summarised rather than pasted, because each of the
Apple lines carries a page of quoted argument vector. Four of them are the same
`ToolNotFound` with a different tool name - `arm-linux-androideabi-clang` on the
second Android triple, `cc` on `aarch64-apple-darwin` and `x86_64-linux-gnu-gcc`
on `x86_64-unknown-linux-gnu`. The two remaining are the Apple device triples,
where `clang` is found and fails, reporting `ToolExecError` on the compilation of
`c11.c` with `--target=arm64-apple-ios` and `--target=arm64-apple-tvos`
respectively and a Windows SDK where a platform sysroot should be.

The seventh is the host and it builds:

    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 41s

That is this machine and not the runner, and what it establishes is the shape
rather than the runner's verdict: the C cross-toolchain per triple is a real
prerequisite that the target leg does not have today, which on a Linux runner
means an Android NDK and an Apple SDK. The 4m 41s is one machine's cold build of
the provider and is the order of the cost rather than a number to hold the runner
to.

The pure-Rust provider that would avoid all of it states its own maturity in its
version string:

    cargo search rustls-rustcrypto --limit 1
    rustls-rustcrypto = "0.0.2-alpha"    # Pure Rust cryptography provider for the Rustls TLS library…

so it is not a candidate today and is named here as the thing that would retire
this cost.

## What the test harness gains

Proving a refusal needs a fake that speaks TLS and can be made to present a
certificate the core will refuse. The fake today is deliberately not that, and
says so in its own header:

    git show origin/main:tests/fake_server/mod.rs | sed -n '29p'
    //! here. This is `std::net` and `std::thread` and nothing else: an HTTP server

Making certificates inside the suite is a test-tree dependency, which
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) relaxes the worth
test for and relaxes nothing else for. The expensive half is that the fake then
needs a server-side TLS implementation too, and that is the same question as the
shipping side rather than a separate one: the provider it would use is the
provider decided above. So this record does not add a second answer for the
suite. #21 and #29 own what the harness becomes; what they gain from here is that
the answer is already chosen for them.

## What this record does not do

It does not add a dependency. `Cargo.toml` is unchanged by it, and the line
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) requires beside a
manifest entry - the clause that admitted it and what would retire it - is
written when the entry arrives, which is #27's and #29's change rather than this
one's.

It does not decide the transport, which is #27, and it does not touch the pin
register, which landed under #29 and needs none of this.

It settles nothing about the Android licence term, which is #268, and it builds
no mechanism for the no-logger condition, which is #266.

## Why this is written down before the code

The core reaches no network at all today:

    git grep -n 'std::net' origin/main -- src/ ; echo "exit=$?"
    exit=1

which is the only moment this can be decided rather than discovered. A means
chosen at a call site is chosen by whoever meets the call site first, and the
thing they will choose is the package that compiles, which on this question is
the one whose error type carries no reason class at all.

The specific failure is narrower and it is the one this board already came within
one decision of. #29's second condition sat behind an absence nobody held, #29
sits in front of the transport in #27, and #27 sits in front of every call any
other issue on this board makes. One decision was holding a milestone, and the
thing holding the decision was not a preference between packages but two landed
records refusing each other. Written afterwards, that collision is discovered as
a red gate on somebody's branch, and the cheapest way out of a red gate is to
weaken whichever record is nearer to hand.

## Alternatives, and what each cost

`native-tls`, which is SChannel on Windows, Security.framework on Apple and
OpenSSL everywhere else. The smallest graph of the four, measured the same way as
the one above:

    cargo tree -e normal --target TRIPLE --prefix none | sed 's/ (\*)$//' \
      | sort -u | grep -v '^nativetls-probe' | grep -v '^$' | wc -l
    aarch64-linux-android        15
    armv7-linux-androideabi      15
    aarch64-apple-ios             7
    aarch64-apple-tvos            7
    aarch64-apple-darwin         14
    x86_64-pc-windows-msvc        4
    x86_64-unknown-linux-gnu     15

It fails [0029](0029-certificate-validation-and-the-self-signed-server.md) on that
record's own terms before
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) is reached. That
record requires the presented chain, the fingerprint, the reason class and the
subject, issuer and validity window to be handed to the client as data, and what
this one publishes is a newtype over the platform's own error with none of them
on it:

    grep -n 'pub struct Error' native-tls-0.2.18/src/lib.rs
    119:pub struct Error(imp::Error);

It is refused by
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) twice over as well.
It carries the same logging facade on its OpenSSL targets, which are five of the
seven, and on Apple it pulls `tempfile`, which is that record's third behaviour, a
dependency that reads or writes the filesystem without being told where. Both are
in the trees above. Its Android target reaches OpenSSL rather than the Android
Trust Manager, so it does not meet
[0029](0029-certificate-validation-and-the-self-signed-server.md)'s platform
requirement there either.

`rustls` with `rustls-native-certs` and `rustls-webpki`, and the logging feature
off. The shape that trips none of the five behaviours - the roots crate declares
one dependency and it is not a logger:

    grep -n -A3 '^\[dependencies' rustls-native-certs-0.8.4/Cargo.toml
    52:[dependencies.pki-types]
    53-version = "1.10"
    54-features = ["std"]
    55-package = "rustls-pki-types"

so it costs no narrowing of any record. What it costs is the platform's
decisions, which the crate's own comparison table states in the row for it:

    sed -n '56,57p' README.md | awk -F'|' '{print $2 "|" $4}'
     `rustls-platform-verifier` (non-Linux/BSD)      | System store, with full (dis)trust decisions from every source available.
     `rustls-native-certs` + `webpki`                | System store, with no (dis)trust decisions. All roots are treated equally regardless of their status.

That is the platform's store without the platform's judgement about it, and
[0029](0029-certificate-validation-and-the-self-signed-server.md) refuses exactly
it. It is what the Linux triple gets anyway, and the difference is that there it
is forced and here it would be chosen for six platforms that have something
better.

The platform facilities reached directly from this tree, over `schannel`,
`security-framework`, `jni-sys` and a roots crate. The narrowest graph, none of
those reaches a logger, and it needs no record narrowed. It costs the largest
amount of security-critical code written here, which is the class
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) answers by name -
a protocol, a parser of somebody else's format, or a cryptographic primitive -
and it is the shape
[0029](0029-certificate-validation-and-the-self-signed-server.md)'s own
alternatives section warns about, the one that feels more careful than the
ordinary answer.

Writing the validation here.
[0103](0103-what-admits-a-dependency-and-what-is-refused.md) answers this by
class in one sentence, and the sentence is about exactly this case: a wrong
implementation of a protocol, a parser or a cryptographic primitive is a defect
nobody sees until it is exploited.

Dropping a platform family from the target set until it can be served. It costs
the least code of any option and it is the only one that makes the Android
licence term and the Android NDK both go away. It costs the platform, and
[0113](0113-the-target-triples-the-gate-compiles-for.md) is where a triple leaves
that set rather than here.

## What would reverse this

A pure-Rust crypto provider for `rustls` reaches a stable release. The C
cross-toolchain per triple then buys nothing, the target leg goes back to one
runner with no NDK and no SDK, and this record is superseded by one naming that
provider. The version string above is the thing to re-read.

The verifier crate's platform dispatch stops reaching a platform's own verifier
on any triple in
[0113](0113-the-target-triples-the-gate-compiles-for.md)'s set, so that what is
carried is roots plus `webpki` there. The whole reason this means was chosen over
the smaller graph is gone for that platform, and the choice is retaken against
the alternatives above rather than inherited.

The facade stops being harmless: a logger is installed anywhere in the core, or
the crate begins writing through a sink it registers itself.
[0103](0103-what-admits-a-dependency-and-what-is-refused.md)'s fourth behaviour
is then in force unnarrowed, and this record is superseded by one that says what
replaces the means rather than by an exception written into either record.

The Android licence term is refused under #268. The two Android triples then
carry a package this board may not carry, and what moves is the means on those
triples, the licence set, or the triples themselves - and this record is
superseded by the one that says which.

Two platforms are measured producing different reason classes for one
certificate, on a real refusal. That is
[0029](0029-certificate-validation-and-the-self-signed-server.md)'s reversal
condition rather than this record's, and it reverses the class set rather than
the means - but the measurement is only possible because of this record, so it is
named here as the thing to go and take.
