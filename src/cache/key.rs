//! How a cache key is built, and why a collision here is a disclosure.
//!
//! `docs/decisions/0041-how-a-cache-key-is-built.md` is the record and this is
//! the construction it fixes: a cryptographic digest of a version tag and the
//! four parts 0006 names, each written as its length in a fixed-width field
//! followed by its bytes, in one order.
//!
//! # What a collision costs, which is what fixes the construction
//!
//! 0101 places it: two accounts on one device must not be able to read each
//! other's entries, and a collision between them is a disclosure rather than a
//! stale answer. Part of the input is chosen by the server, because an item
//! identifier in a request came from an answer the server sent and 0101 treats
//! every byte from a server as untrusted whether the server is healthy or not.
//! So the construction has to hold against an input somebody chose to make it
//! fail, which is why the digest is cryptographic and why the joining is by
//! length rather than by a separator.
//!
//! # What the digest does not buy
//!
//! It removes readable values from anywhere a store puts a key, so a directory
//! listing, a shared backup or a screenshot stops saying which servers a person
//! uses. It hides nothing from somebody who can guess the input: the parts are
//! low entropy, and anyone holding the device who suspects a particular server
//! and account can compute a candidate and look for it. Separation between
//! accounts is what this provides. Confidentiality against whoever holds the
//! device is not, and 0101 already says the core does not encrypt what it
//! writes.

use sha2::{Digest, Sha256};

use super::EntryKey;

/// The tag that opens a cache entry key.
///
/// Inside the digest rather than beside it, which is what makes an old key space
/// unreachable rather than misread when the construction changes: nothing has to
/// migrate it and the bytes are ordinary garbage the bound in #42 evicts. The
/// trailing number is this construction's version and moves when any part of the
/// writing below moves.
///
/// It is also what separates this space from the secret store's. 0033 requires a
/// secret name and a cache key to be distinguishable, and 0041 says the tag at
/// the front is what carries that, so the two spaces differ in their first
/// written part and cannot collide however equal the other four are. The secret
/// store's own tag arrives with #33's naming and is not invented here.
const CACHE_ENTRY_SPACE: &[u8] = b"flowfin/cache-entry/1";

/// The width of the length field written in front of every part.
///
/// Eight bytes, big-endian, of a [`u64`]. Fixed-width rather than variable so
/// that the length itself cannot be read as part of what follows it, and wide
/// enough that no length this core can produce needs a second rule.
const LENGTH_WIDTH: usize = 8;

/// The width above is the width of what is actually written, rather than a
/// number beside it that a later edit could leave behind.
const _: () = assert!(LENGTH_WIDTH == size_of::<u64>());

/// Which of the two things the server part of a key space is.
///
/// 0041 fixes the server as the resolved identity rather than the address a
/// person typed, and fixes a fallback where a server offers no identity of its
/// own. 0010 read both supported lines and found `GET /System/Info/Public` on
/// each, so [`ServerPart::Reported`] is the ordinary case and
/// [`ServerPart::BaseAddress`] is the exception.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerPart<'a> {
    /// The identifier the server reports about itself.
    Reported(&'a str),
    /// The base address from 0028, where the server offers no identifier.
    ///
    /// The cost 0041 states rather than leaves to be discovered: two addresses
    /// that reach one server become two key spaces, so entries are fetched twice
    /// and never mixed. That is the safe direction, since the unsafe one is a
    /// single key space for two servers.
    BaseAddress(&'a str),
}

impl<'a> ServerPart<'a> {
    /// Which of the two this is, as bytes written into the key.
    ///
    /// WRITTEN AS ITS OWN PART, WHICH THE RECORD DOES NOT ASK FOR AND WHICH THE
    /// CONVENIENT VERSION LEAVES OUT. Without it, a server reporting the
    /// identifier `https://films.example` and a different server with no
    /// identifier reached at the base address `https://films.example` produce
    /// one key space between them. That is the direction 0041 calls unsafe, and
    /// nothing about either input looks wrong at the call site. Writing the kind
    /// costs one part and fails the other way, into two key spaces for one
    /// server, which 0041 already accepts as the price of the fallback.
    const fn kind(self) -> &'static [u8] {
        match self {
            Self::Reported(_) => b"reported",
            Self::BaseAddress(_) => b"base-address",
        }
    }

    /// The value, whichever of the two it is.
    const fn value(self) -> &'a str {
        match self {
            Self::Reported(value) | Self::BaseAddress(value) => value,
        }
    }
}

/// The three parts of a key that do not change between two requests.
///
/// The server, the account as the identifier the server gave back at sign-in
/// rather than the username, and the device identity from #36. 0068 promises a
/// caller that signing out removes every entry under one of these, and 0041
/// makes that set well defined without making it reachable: 0040 gives the store
/// no listing and a digest does not reverse, so the removal in #114 needs the
/// core's own record of which keys it wrote, which is the bookkeeping 0040 hands
/// to #42.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpace<'a> {
    /// The server, as 0041 fixes it.
    pub server: ServerPart<'a>,
    /// The account, as the identifier the server gave back at sign-in.
    pub account: &'a str,
    /// The device identity from #36.
    pub device: &'a str,
}

/// One parameter of a request, and whether it was there at all.
///
/// [`None`] is a parameter that was absent and [`Some("")`] is one that was
/// present and empty. 0041 requires the two to be written differently, because
/// they are different requests and the server may answer them differently.
pub type Parameter<'a> = (&'a str, Option<&'a str>);

/// The part of a key that changes between two requests.
///
/// The endpoint and the parameters that change the answer. Which parameters
/// change an answer at all is a per-endpoint fact 0006 leaves with the code, and
/// 0041 adds only that a parameter the core decided not to include is excluded
/// by a written decision at that endpoint rather than by being forgotten.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestKey<'a> {
    /// The endpoint, as 0010 writes one.
    pub endpoint: &'a str,
    /// The parameters that change the answer, in any order.
    ///
    /// THE ORDER DOES NOT REACH THE KEY, WHICH THE RECORD LEAVES OPEN. The
    /// parameters are sorted by name before they are written, so one request
    /// written two ways is one entry. Order-dependent, the same library query
    /// assembled by two call sites is two entries holding the same bytes, which
    /// is a silent duplication rather than a wrong answer, and it costs a fetch
    /// and a slot in the bound every time. Two parameters carrying one name keep
    /// the order they were given in, so a request naming `ids` twice with two
    /// values is not the same request as the one naming them the other way
    /// round.
    pub parameters: &'a [Parameter<'a>],
}

/// Writes parts, each as its length in a fixed-width field followed by its
/// bytes.
///
/// THE LENGTH PREFIX IS THE WHOLE OF THE AMBIGUITY ARGUMENT, and what it
/// prevents is invisible by inspection. With plain concatenation an account
/// called `ab` followed by a request starting `c` produces the same bytes as an
/// account called `a` followed by a request starting `bc`, and one of those two
/// is a person who is not supposed to be reading it. A separator character does
/// not fix that; it moves the problem to whichever part is allowed to contain
/// the separator, and an item identifier that came from a server the core does
/// not trust is exactly such a part.
struct Written {
    bytes: Vec<u8>,
}

impl Written {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Writes one part.
    ///
    /// The length is written as a [`u64`] whatever the pointer width, so a key
    /// derived on a 32-bit television and one derived on a 64-bit desktop are
    /// the same key for the same input.
    fn part(&mut self, bytes: &[u8]) -> &mut Self {
        let length = u64::try_from(bytes.len()).expect("a part is shorter than a 64-bit length");
        self.bytes.extend_from_slice(&length.to_be_bytes());
        self.bytes.extend_from_slice(bytes);
        self
    }

    /// Writes a part whose own content is a sequence of parts.
    ///
    /// The nested writing is length-prefixed as one part, so the same argument
    /// holds one level down: no value inside a nested part can be made to look
    /// like the start of the part after it.
    fn nested(&mut self, inner: &Written) -> &mut Self {
        self.part(&inner.bytes)
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// Whether a parameter was present, written so that absent and present-and-empty
/// are different bytes.
fn parameter_value(value: Option<&str>) -> Vec<u8> {
    let mut written = Written::new();
    match value {
        None => written.part(b"absent"),
        Some(present) => written.part(b"present").part(present.as_bytes()),
    };
    written.finish()
}

/// The five parts, in 0041's order, ready to be digested.
fn written_parts(space_tag: &[u8], space: &KeySpace<'_>, request: &RequestKey<'_>) -> Vec<u8> {
    let mut server = Written::new();
    server
        .part(space.server.kind())
        .part(space.server.value().as_bytes());

    let mut sorted: Vec<Parameter<'_>> = request.parameters.to_vec();
    sorted.sort_by_key(|(name, _)| *name);

    let mut endpoint = Written::new();
    endpoint.part(request.endpoint.as_bytes());
    for (name, value) in &sorted {
        endpoint
            .part(name.as_bytes())
            .part(&parameter_value(*value));
    }

    let mut all = Written::new();
    all.part(space_tag)
        .nested(&server)
        .part(space.account.as_bytes())
        .part(space.device.as_bytes())
        .nested(&endpoint);
    all.finish()
}

/// The digest, at its full width, as lower-case hexadecimal.
///
/// SHA-256, and the reason the choice is here rather than in the record is that
/// 0041 says so: it requires a cryptographic digest used at full width and
/// leaves which function and what width to the means. What decides it is
/// collision and second-preimage resistance against an input the adversary
/// partly controls, which SHA-256 has and no shortened output of it does, so
/// nothing here truncates.
///
/// Hexadecimal rather than the raw bytes because 0040 lets a store use a key as
/// a filename, a column or a label, and a key that survives all three without a
/// rule about encoding is one a client cannot get wrong. The cost is two
/// characters per byte and it is paid once per key.
fn digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(Sha256::output_size() * 2);
    for byte in Sha256::digest(bytes) {
        out.push_str(HEX[usize::from(byte)]);
    }
    out
}

/// The two characters each byte becomes, built once.
static HEX: [&str; 256] = build_hex();

/// Builds the table above at compile time, so the hot path holds no formatting.
const fn build_hex() -> [&'static str; 256] {
    // A literal table rather than a loop over `format!`, which is not available
    // in a constant. The pairs are generated in order, so an index is its own
    // value.
    const PAIRS: &str = "\
000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\
202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f\
606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f\
a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf\
c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf\
e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff";
    let mut table: [&'static str; 256] = [""; 256];
    let bytes = PAIRS.as_bytes();
    let mut i = 0;
    while i < 256 {
        // Safe by construction: the table above is exactly two ASCII characters
        // per index, so every slice taken here is on a character boundary.
        let (_, rest) = bytes.split_at(i * 2);
        let (pair, _) = rest.split_at(2);
        table[i] = match core::str::from_utf8(pair) {
            Ok(text) => text,
            Err(_) => "",
        };
        i += 1;
    }
    table
}

/// A DELIBERATE DEFECT, WRITTEN TO MEASURE WHETHER THE CODE-SCANNING ANALYSIS
/// REPORTS IT, AND REVERTED IN THE NEXT COMMIT ON THIS BRANCH (#81).
///
/// #81's second condition is that a deliberately introduced finding of the kind
/// the analysis is meant to catch reddens the gate. Five were tried in #195 and
/// the query set fired on none of them, and the reading recorded there was about
/// the graph rather than about the queries: the crate declared no dependency, so
/// nothing in it was a sink any query recognised. The graph has moved, `sha2` is
/// in it, and three of the sixteen loaded queries are named for exactly that.
///
/// Two shapes in one function, both of them what those queries look for: key
/// material written into the source, and a value a caller supplies as a
/// credential hashed with a bare digest. Nothing calls it, nothing merges it,
/// and the run it exists to produce is read into #81.
#[must_use]
pub fn the_deliberate_defect_for_issue_81(password: &str) -> String {
    const SECRET_KEY: [u8; 32] = [
        0x3f, 0x8c, 0x1b, 0x0a, 0x9d, 0x2e, 0x4f, 0x67, 0x08, 0x19, 0x2a, 0x3b, 0x4c, 0x5d, 0x6e,
        0x7f, 0x90, 0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c, 0x6d,
        0x7e, 0x8f,
    ];
    let mut hasher = Sha256::new();
    hasher.update(SECRET_KEY);
    hasher.update(password.as_bytes());
    let mut out = String::with_capacity(Sha256::output_size() * 2);
    for byte in hasher.finalize() {
        out.push_str(HEX[usize::from(byte)]);
    }
    out
}

impl EntryKey {
    /// Derives the key one cache entry is kept under.
    ///
    /// The construction is 0041's, in its order: a version tag for this key
    /// space, the server, the account, the device identity, and the request.
    /// Each part is written as its length in a fixed-width field followed by its
    /// bytes, and the digest of the result is the key.
    ///
    /// The artwork tier in #54 keys the same way. It is a separate tier with its
    /// own bound rather than a separate scheme, which 0101 names as the case
    /// that would otherwise have no rule at all.
    #[must_use]
    pub fn derive(space: &KeySpace<'_>, request: &RequestKey<'_>) -> Self {
        Self::from_derived_key(digest(&written_parts(CACHE_ENTRY_SPACE, space, request)))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CACHE_ENTRY_SPACE, EntryKey, KeySpace, LENGTH_WIDTH, Parameter, RequestKey, ServerPart,
        digest, written_parts,
    };

    fn space<'a>(server: ServerPart<'a>, account: &'a str) -> KeySpace<'a> {
        KeySpace {
            server,
            account,
            device: "device-9f2c",
        }
    }

    fn request<'a>(endpoint: &'a str, parameters: &'a [Parameter<'a>]) -> RequestKey<'a> {
        RequestKey {
            endpoint,
            parameters,
        }
    }

    fn key(server: ServerPart<'_>, account: &str, endpoint: &str) -> String {
        EntryKey::derive(&space(server, account), &request(endpoint, &[]))
            .as_str()
            .to_owned()
    }

    /// The first of this issue's three conditions that a derivation can answer
    /// on its own: two servers holding the same item identifier do not collide.
    /// Everything else about the two requests is equal, so the server part is
    /// the only thing separating them.
    #[test]
    fn two_servers_holding_one_item_identifier_do_not_collide() {
        let first = key(
            ServerPart::Reported("server-a"),
            "account-1",
            "/Items/the-same-item",
        );
        let second = key(
            ServerPart::Reported("server-b"),
            "account-1",
            "/Items/the-same-item",
        );
        assert_ne!(first, second);
    }

    /// The failure 0006 calls the one that cannot be corrected afterwards at any
    /// reasonable price: two accounts on one device reaching each other's
    /// entries. A shared television is where it shows and a developer's own
    /// device is where it never does.
    #[test]
    fn two_accounts_on_one_device_do_not_collide() {
        let first = key(
            ServerPart::Reported("server-a"),
            "account-1",
            "/Items/the-same-item",
        );
        let second = key(
            ServerPart::Reported("server-a"),
            "account-2",
            "/Items/the-same-item",
        );
        assert_ne!(first, second);
    }

    /// The second condition a derivation can answer alone. A store may use the
    /// key as a filename, a column or a label, so 0101 requires that nothing the
    /// core writes carries a person's name or a server address in a readable
    /// form.
    #[test]
    fn no_key_material_carries_a_readable_name_or_address() {
        let derived = EntryKey::derive(
            &KeySpace {
                server: ServerPart::BaseAddress("https://films.example:8920"),
                account: "ada.lovelace",
                device: "the-television-in-the-front-room",
            },
            &request("/Items/44f1", &[("userId", Some("ada.lovelace"))]),
        );
        let written = derived.as_str();

        for readable in [
            "films.example",
            "https",
            "8920",
            "ada.lovelace",
            "television",
            "Items",
            "userId",
        ] {
            assert!(
                !written.contains(readable),
                "the key carries {readable} in a form somebody reading a directory listing could see"
            );
        }

        assert_eq!(written.len(), 64, "the digest is used at its full width");
        assert!(
            written
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "a key is lower-case hexadecimal and nothing else"
        );
    }

    /// The defect the length prefix exists against, written as the record writes
    /// it: an account called `ab` followed by a device called `c` produces the
    /// same bytes as an account called `a` followed by a device called `bc`, and
    /// one of the two is a person who is not supposed to be reading the other's
    /// entries. Deleting the length field in `Written::part` is what turns this
    /// red.
    ///
    /// THE TWO PARTS HAVE TO BE ADJACENT AND THIS TEST NAMED TWO THAT ARE NOT
    /// UNTIL IT WAS WATCHED FAILING. It varied the account and the endpoint,
    /// which have the device identity written between them, so the concatenation
    /// separated them whatever the prefix did and the assertion held with the
    /// prefix deleted. A guard nobody watched fail is a guard nobody knows the
    /// direction of, and this one was pointing at nothing.
    #[test]
    fn a_part_cannot_be_made_to_look_like_the_start_of_the_next_one() {
        let first = EntryKey::derive(
            &KeySpace {
                server: ServerPart::Reported("s"),
                account: "ab",
                device: "c",
            },
            &request("/Items", &[]),
        );
        let second = EntryKey::derive(
            &KeySpace {
                server: ServerPart::Reported("s"),
                account: "a",
                device: "bc",
            },
            &request("/Items", &[]),
        );
        assert_ne!(first.as_str(), second.as_str());
    }

    /// The same argument one level down, where the request is written. The
    /// endpoint and the first parameter name are adjacent inside that nested
    /// part, so an endpoint that swallows the start of a parameter name is the
    /// same defect one level in.
    #[test]
    fn a_part_inside_the_request_cannot_be_made_to_look_like_the_next_one() {
        let first = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request("ab", &[("c", None)]),
        );
        let second = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request("a", &[("bc", None)]),
        );
        assert_ne!(first.as_str(), second.as_str());
    }

    /// 0041 states this one because the convenient handling collapses the two:
    /// a parameter that is absent and one that is present and empty are
    /// different requests, and the server may answer them differently.
    #[test]
    fn an_absent_parameter_is_not_a_present_and_empty_one() {
        let absent = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request("/Items", &[("searchTerm", None)]),
        );
        let empty = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request("/Items", &[("searchTerm", Some(""))]),
        );
        assert_ne!(absent.as_str(), empty.as_str());
    }

    /// The part of the fallback the record does not write out. A reported
    /// identifier and a base address that read the same are two servers, and
    /// collapsing them is the direction 0041 calls unsafe.
    #[test]
    fn a_reported_identity_is_not_a_base_address_that_reads_the_same() {
        let reported = key(
            ServerPart::Reported("https://films.example"),
            "account-1",
            "/Items",
        );
        let fallback = key(
            ServerPart::BaseAddress("https://films.example"),
            "account-1",
            "/Items",
        );
        assert_ne!(reported, fallback);
    }

    /// The tag is inside the digest rather than beside it, so a change to the
    /// construction makes an old key space unreachable instead of misread.
    /// Nothing outside this module can move the tag, so the assertion is made
    /// against the writing rather than against the public call.
    #[test]
    fn the_version_tag_is_inside_the_digest() {
        let space = space(ServerPart::Reported("s"), "account-1");
        let request = request("/Items", &[]);

        let current = digest(&written_parts(CACHE_ENTRY_SPACE, &space, &request));
        let next = digest(&written_parts(b"flowfin/cache-entry/2", &space, &request));
        let other_space = digest(&written_parts(b"flowfin/secret-name/1", &space, &request));

        assert_ne!(current, next, "a new version reaches a new key space");
        assert_ne!(
            current, other_space,
            "0033 requires a secret name and a cache key to be distinguishable"
        );
    }

    /// The decision this module took that 0041 left open. Order-dependent, one
    /// request assembled by two call sites is two entries holding the same
    /// bytes.
    #[test]
    fn the_order_the_parameters_were_given_in_does_not_reach_the_key() {
        let one = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request(
                "/Items",
                &[("sortBy", Some("name")), ("parentId", Some("44f1"))],
            ),
        );
        let other = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request(
                "/Items",
                &[("parentId", Some("44f1")), ("sortBy", Some("name"))],
            ),
        );
        assert_eq!(one.as_str(), other.as_str());
    }

    /// Two parameters carrying one name are not reordered against each other, so
    /// a request naming `ids` twice is not the request naming them the other way
    /// round.
    #[test]
    fn two_parameters_carrying_one_name_keep_the_order_they_were_given_in() {
        let one = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request("/Items", &[("ids", Some("a")), ("ids", Some("b"))]),
        );
        let other = EntryKey::derive(
            &space(ServerPart::Reported("s"), "account-1"),
            &request("/Items", &[("ids", Some("b")), ("ids", Some("a"))]),
        );
        assert_ne!(one.as_str(), other.as_str());
    }

    /// A key is a function of its input and of nothing else, which is what makes
    /// a second run of the core read what the first one wrote.
    #[test]
    fn the_same_input_produces_the_same_key() {
        let first = key(ServerPart::Reported("s"), "account-1", "/Items");
        let second = key(ServerPart::Reported("s"), "account-1", "/Items");
        assert_eq!(first, second);
    }

    /// The device identity is one of the five and is the one no other test above
    /// varies on its own.
    #[test]
    fn two_devices_under_one_account_do_not_collide() {
        let first = EntryKey::derive(
            &KeySpace {
                server: ServerPart::Reported("s"),
                account: "account-1",
                device: "device-a",
            },
            &request("/Items", &[]),
        );
        let second = EntryKey::derive(
            &KeySpace {
                server: ServerPart::Reported("s"),
                account: "account-1",
                device: "device-b",
            },
            &request("/Items", &[]),
        );
        assert_ne!(first.as_str(), second.as_str());
    }

    /// The length field is fixed-width and is written whatever the pointer width
    /// of the machine deriving the key, so a television and a desktop agree.
    #[test]
    fn every_part_is_written_behind_a_fixed_width_length() {
        let written = written_parts(
            b"t",
            &KeySpace {
                server: ServerPart::Reported("s"),
                account: "a",
                device: "d",
            },
            &RequestKey {
                endpoint: "e",
                parameters: &[],
            },
        );

        // The tag, one byte, behind an eight-byte length that reads as one.
        assert_eq!(&written[..LENGTH_WIDTH], &1u64.to_be_bytes());
        assert_eq!(written[LENGTH_WIDTH], b't');
    }
}
