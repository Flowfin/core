//! The address a person typed, and the one routine every path is joined by.
//!
//! `docs/decisions/0028-the-address-a-person-typed.md` is the record. It fixes
//! two things that are easy to get wrong quietly, and both of them are here
//! rather than in a caller: what each shape a person types becomes, and how a
//! request path is appended to the result.
//!
//! # Why a joining routine rather than a URL type's own
//!
//! Reference resolution is the operation a URL type offers and the one that
//! reads as correct, and it replaces everything after the base's last separator.
//! `Users/Me` resolved against `https://example.com/jellyfin` is
//! `https://example.com/Users/Me`, so an operator behind a reverse proxy at a
//! sub-path loses the sub-path on every request, the server answers 404, and a
//! 404 at sign-in reads as a wrong password. [`BaseAddress::join`] concatenates.
//!
//! # What a caller can and cannot do with a base address
//!
//! [`BaseAddress`] holds its parts privately and hands out no string except
//! through [`BaseAddress::join`], so the "one routine" half of 0028 is a
//! property of this type rather than a convention callers keep. A caller that
//! needs the host on its own - the list of hosts the core may contact, #69 - adds
//! the accessor for it deliberately, against the parts already parsed here, and
//! not by parsing a base string a second time.
//!
//! # One rule of 0028 is not implemented here
//!
//! 0028 says a host with characters outside ASCII is converted to its ASCII form
//! once, on the way in. This module REFUSES such a host instead, as
//! [`UnusablePart::Host`], and that is a departure rather than an omission
//! nobody noticed.
//!
//! The conversion is IDNA, which needs the Unicode mapping and normalisation
//! tables that decide it. The standard library carries neither, and
//! `docs/decisions/0011-the-language-the-toolchain-and-the-binding-layer.md`
//! measures five absences without measuring this one:
//!
//! ```text
//! $ cat idna.rs
//! fn main() { let _ = "münchen.example".to_ascii_idna(); }
//! $ rustc --edition 2024 -o idna idna.rs
//! error[E0599]: no method named `to_ascii_idna` found for reference `&'static str`
//! ```
//!
//! So the two ways to close it are a dependency admitted under
//! `docs/decisions/0103-what-admits-a-dependency-and-what-is-refused.md` or a
//! record superseding 0028, and both are decisions rather than code. Until one
//! of them is taken, refusing is the direction that cannot send a request
//! somewhere the person did not name: a refused address reaches no server, and
//! the host is never held in a form #69's list would compare wrongly against.

/// The two schemes 0028 accepts, and there is no third.
///
/// A scheme outside this set is refused by name rather than by omission, which
/// is the shape 0055 uses for image formats. Holding the accepted set as a type
/// is what makes the refusal a property of the value: there is no way to build a
/// [`BaseAddress`] carrying anything else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// What a person typed as `http://`, and never what an absent scheme
    /// becomes.
    Http,
    /// What a person typed as `https://`, and what an absent scheme becomes.
    Https,
}

impl Scheme {
    /// The scheme as it appears in a request, lowered in case.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Which part of what was typed could not be used.
///
/// 0004 fixes the payload of `address-not-usable` as the address exactly as it
/// was given together with the part that could not be used, and this is that
/// second half. The set is closed and exhaustive, for 0004's reason: a caller
/// matching on it is told by the compiler when a case appears rather than
/// falling into a branch somebody wrote for something else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnusablePart {
    /// Nothing was typed, or nothing but whitespace was.
    TheWholeAddress,
    /// A scheme that is neither `http` nor `https`.
    Scheme,
    /// A user name or a password inside the address. 0028 refuses these rather
    /// than stripping them: stripping connects with less than the person
    /// supplied, and keeping puts a password inside a value that is stored and
    /// shown.
    Credentials,
    /// A host that is empty, that carries a character a host may not, or that
    /// is outside ASCII. The last of those is this module's departure from
    /// 0028 and the module documentation says why.
    Host,
    /// Something after the host separator that is not a port number a request
    /// could be sent to.
    Port,
    /// A sub-path this core will not hold: one carrying an empty segment, a
    /// space, or a control character. None of the three is repaired, because
    /// each repair produces a path the person did not type, and a wrong path is
    /// the 404 at sign-in this record exists to prevent.
    Path,
}

/// What a person typed could not be turned into somewhere to send a request.
///
/// This is not a value of the failure vocabulary. 0037 requires every value of
/// that set to be built at one mapping point and nowhere else. THIS SENTENCE
/// SAID THAT POINT DOES NOT EXIST IN THIS TREE YET. It does, and this type is
/// what it reads: [`crate::failure::Failure::address_not_usable`] turns what
/// this module found into 0004's `address-not-usable`, keeping what was typed
/// unmodified and the part alongside it.
///
/// It carries what was typed unmodified, including the surrounding whitespace,
/// because a client shows a person what they typed rather than what the core
/// made of it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressNotUsable {
    typed: String,
    part: UnusablePart,
}

impl AddressNotUsable {
    /// The address exactly as it was given, with nothing removed or repaired.
    #[must_use]
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// Which part of it could not be used.
    #[must_use]
    pub const fn part(&self) -> UnusablePart {
        self.part
    }
}

/// A usable base address, and the only thing downstream of a typed address.
///
/// 0028 applies its rules once, where an address enters the core, and what is
/// stored afterwards is the result. No call site re-reads what a person typed,
/// and nothing downstream sees the original except the payload of an
/// [`AddressNotUsable`].
///
/// Thread safety, from 0009: immutable once parsed. There is no shared mutable
/// state to protect and no method that changes one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseAddress {
    scheme: Scheme,
    /// Lowered in case, ASCII, and holding its brackets where the host is an
    /// IPv6 address, because the brackets are what separate the address from
    /// the port.
    host: String,
    port: Option<u16>,
    /// The sub-path an operator typed, case preserved, with no trailing
    /// separator. Empty where there is none.
    path: String,
}

impl BaseAddress {
    /// Turns what a person typed into a base address.
    ///
    /// The rules are 0028's and the module documentation says which one of them
    /// this does not implement.
    ///
    /// # Errors
    ///
    /// [`AddressNotUsable`], carrying the address as it was given and the part
    /// that could not be used. Nothing is guessed and nothing is repaired: a
    /// repaired address is a request sent to a destination the person did not
    /// name.
    pub fn parse(typed: &str) -> Result<Self, AddressNotUsable> {
        let refuse = |part| {
            Err(AddressNotUsable {
                typed: typed.to_owned(),
                part,
            })
        };

        // Surrounding whitespace first. It arrives from a paste out of a message
        // and a person cannot see it.
        let trimmed = typed.trim();
        if trimmed.is_empty() {
            return refuse(UnusablePart::TheWholeAddress);
        }

        let (scheme, authority_and_path) = match split_scheme(trimmed) {
            Some(("https", rest)) => (Scheme::Https, rest),
            Some(("http", rest)) => (Scheme::Http, rest),
            // A scheme the person did type and this core does not accept.
            Some(_) => return refuse(UnusablePart::Scheme),
            // No scheme becomes https, never http: the first thing that travels
            // over a scheme-less address is a password.
            None => (Scheme::Https, trimmed),
        };

        // A query string and a fragment arrive from a paste out of a browser,
        // neither can mean anything for a base address, and carried into the
        // base they would ride on every request the core ever makes. The
        // fragment goes first because it may hold a question mark.
        let without_fragment = authority_and_path
            .split_once('#')
            .map_or(authority_and_path, |(before, _)| before);
        let without_query = without_fragment
            .split_once('?')
            .map_or(without_fragment, |(before, _)| before);

        let (authority, path) = match without_query.find('/') {
            Some(at) => (&without_query[..at], &without_query[at..]),
            None => (without_query, ""),
        };

        if authority.contains('@') {
            return refuse(UnusablePart::Credentials);
        }

        let Some((host, port_text)) = split_host_and_port(authority) else {
            return refuse(UnusablePart::Host);
        };
        if !host_is_usable(host) {
            return refuse(UnusablePart::Host);
        }

        let port = match port_text {
            None => None,
            Some(text) => match text.parse::<u16>() {
                // Port zero names no port a request could be sent to, and a
                // parser accepting it produces an address that fails at connect
                // rather than here.
                Ok(0) | Err(_) => return refuse(UnusablePart::Port),
                Ok(number) => Some(number),
            },
        };

        let kept_path = path.trim_end_matches('/').to_owned();
        if !path_is_usable(&kept_path) {
            return refuse(UnusablePart::Path);
        }

        Ok(Self {
            scheme,
            // A host is case-insensitive, so it is lowered here and compared
            // nowhere else in two forms.
            host: host.to_ascii_lowercase(),
            port,
            // A path is not case-insensitive, and lowering one turns a working
            // sub-path into a 404 on any server whose filesystem cares. The
            // trailing separator goes so that a join adds exactly one and a
            // double slash never reaches a server.
            path: kept_path,
        })
    }

    /// Appends a request path to this base address.
    ///
    /// This is the one routine that builds a request address, and it
    /// concatenates. Exactly one separator goes between the base and the path,
    /// whether or not the path was written with a leading one, so a sub-path
    /// survives and a double slash never reaches a server.
    ///
    /// Joining nothing yields the base itself, with no trailing separator.
    #[must_use]
    pub fn join(&self, path: &str) -> String {
        let mut joined = String::new();
        joined.push_str(self.scheme.as_str());
        joined.push_str("://");
        joined.push_str(&self.host);
        if let Some(port) = self.port {
            joined.push(':');
            joined.push_str(&port.to_string());
        }
        joined.push_str(&self.path);

        let appended = path.trim_start_matches('/');
        if !appended.is_empty() {
            joined.push('/');
            joined.push_str(appended);
        }
        joined
    }
}

/// Splits a scheme off the front, where one was typed.
///
/// A scheme is recognised ONLY when the separator that follows it is `//`, and
/// that is the rule 0028's `host:port` case rests on: `example.com:8096` handed
/// to a parser is otherwise read as the scheme `example.com` with `8096` as
/// everything after it, which is the shape most likely to be typed on a home
/// network and the shape most likely to be parsed wrongly. Under this rule it
/// carries no scheme, `https` is supplied, and the port is a port.
fn split_scheme(address: &str) -> Option<(&str, &str)> {
    let (candidate, rest) = address.split_once("://")?;
    let mut characters = candidate.chars();
    let first = characters.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !characters.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
        return None;
    }
    // Lowered so that a scheme typed in capitals is the same scheme.
    Some(match candidate.to_ascii_lowercase().as_str() {
        "http" => ("http", rest),
        "https" => ("https", rest),
        _ => ("", rest),
    })
}

/// Splits an authority into a host and the text after the host separator.
///
/// `None` where the shape is not one a host can be read out of at all: an
/// unclosed bracket, a bracketed form with something after the bracket that is
/// not a port, or an unbracketed host carrying more than one separator, which is
/// an IPv6 address somebody typed without the brackets that make it readable.
fn split_host_and_port(authority: &str) -> Option<(&str, Option<&str>)> {
    if let Some(rest) = authority.strip_prefix('[') {
        let closing = rest.find(']')?;
        let host = &authority[..=closing + 1];
        return match &authority[closing + 2..] {
            "" => Some((host, None)),
            after => Some((host, Some(after.strip_prefix(':')?))),
        };
    }
    match authority.split_once(':') {
        None => Some((authority, None)),
        Some((host, port)) if !port.contains(':') => Some((host, Some(port))),
        Some(_) => None,
    }
}

/// Whether a host is one this core will hold.
///
/// A bracketed host holds an IPv6 address: hexadecimal, separators and the
/// dotted form an IPv4-mapped address ends with. Everything else is a registered
/// name, which is letters, digits, `-`, `.` and `_`. The underscore is not legal
/// in a hostname and is present on internal networks, and refusing it would
/// refuse an address that works.
///
/// A character outside ASCII is refused here, which is this module's departure
/// from 0028 and is argued at the top of this file.
fn host_is_usable(host: &str) -> bool {
    if let Some(inner) = host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
        return !inner.is_empty()
            && inner
                .chars()
                .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.');
    }
    !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
}

/// Whether a sub-path is one this core will hold.
///
/// An empty segment is refused rather than collapsed. `example.com//jellyfin`
/// and an address typed with the scheme's separator missing, `https//example.com`,
/// both arrive here, and each has more than one plausible repair - which is the
/// definition of a guess. A space and a control character are refused for the
/// same reason: a path carrying either has to be encoded before it can be sent,
/// and encoding it here would send a request to a path the person did not type.
fn path_is_usable(path: &str) -> bool {
    !path.contains("//") && !path.chars().any(|c| c == ' ' || c.is_control())
}

#[cfg(test)]
mod tests {
    //! The table 0028 is read against, and the joins that prove a sub-path
    //! survives.
    //!
    //! Every case names the shape of 0028 it stands for. A base address is read
    //! back through [`BaseAddress::join`] with nothing appended, because that is
    //! the only routine this type hands a string out through, so the table
    //! exercises the same path a request does.

    use super::{AddressNotUsable, BaseAddress, UnusablePart};

    /// What a typed address is expected to become.
    enum Expected {
        /// A usable base, written as the string a join with nothing appended
        /// produces.
        Base(&'static str),
        /// A refusal naming a part.
        Refused(UnusablePart),
    }

    struct Case {
        /// The shape of 0028 this case stands for.
        shape: &'static str,
        typed: &'static str,
        expected: Expected,
    }

    const CASES: &[Case] = &[
        Case {
            shape: "a complete address is unchanged",
            typed: "https://example.com",
            expected: Expected::Base("https://example.com"),
        },
        Case {
            shape: "no scheme becomes https, never http",
            typed: "example.com",
            expected: Expected::Base("https://example.com"),
        },
        Case {
            shape: "a scheme the person typed is honoured and never changed",
            typed: "http://example.com",
            expected: Expected::Base("http://example.com"),
        },
        Case {
            shape: "a trailing slash is removed",
            typed: "https://example.com/",
            expected: Expected::Base("https://example.com"),
        },
        Case {
            shape: "a base path is kept",
            typed: "https://example.com/jellyfin",
            expected: Expected::Base("https://example.com/jellyfin"),
        },
        Case {
            shape: "a base path with a trailing slash keeps one and loses the other",
            typed: "https://example.com/jellyfin/",
            expected: Expected::Base("https://example.com/jellyfin"),
        },
        Case {
            shape: "several trailing slashes leave no separator behind",
            typed: "https://example.com/jellyfin//",
            expected: Expected::Base("https://example.com/jellyfin"),
        },
        Case {
            shape: "a port with no host separator is read as a port",
            typed: "example.com:8096",
            expected: Expected::Base("https://example.com:8096"),
        },
        Case {
            shape: "a scheme-less address with a port and a base path",
            typed: "example.com:8096/jellyfin/",
            expected: Expected::Base("https://example.com:8096/jellyfin"),
        },
        Case {
            shape: "surrounding whitespace is removed before anything else",
            typed: "  example.com/jellyfin  ",
            expected: Expected::Base("https://example.com/jellyfin"),
        },
        Case {
            shape: "a query string is removed",
            typed: "https://example.com/jellyfin?redirect=1",
            expected: Expected::Base("https://example.com/jellyfin"),
        },
        Case {
            shape: "a fragment is removed, and it may hold a question mark",
            typed: "https://example.com/jellyfin#what?",
            expected: Expected::Base("https://example.com/jellyfin"),
        },
        Case {
            shape: "the bracketed form keeps its brackets",
            typed: "[::1]:8096",
            expected: Expected::Base("https://[::1]:8096"),
        },
        Case {
            shape: "an IP address is a host like any other",
            typed: "192.168.1.10:8096",
            expected: Expected::Base("https://192.168.1.10:8096"),
        },
        Case {
            shape: "the scheme and the host are lowered in case; the path is not",
            typed: "HTTPS://Example.COM/Jellyfin/Media",
            expected: Expected::Base("https://example.com/Jellyfin/Media"),
        },
        Case {
            shape: "a scheme outside the accepted set is refused by name",
            typed: "ftp://example.com",
            expected: Expected::Refused(UnusablePart::Scheme),
        },
        Case {
            shape: "credentials are refused, not stripped",
            typed: "https://name:secret@example.com",
            expected: Expected::Refused(UnusablePart::Credentials),
        },
        Case {
            shape: "a user name with no password is still credentials",
            typed: "https://name@example.com/jellyfin",
            expected: Expected::Refused(UnusablePart::Credentials),
        },
        Case {
            shape: "nothing was typed",
            typed: "",
            expected: Expected::Refused(UnusablePart::TheWholeAddress),
        },
        Case {
            shape: "nothing but whitespace was typed",
            typed: "   ",
            expected: Expected::Refused(UnusablePart::TheWholeAddress),
        },
        Case {
            shape: "something after the host separator that is not a number",
            typed: "https://example.com:jellyfin",
            expected: Expected::Refused(UnusablePart::Port),
        },
        Case {
            shape: "a port outside the range a port has",
            typed: "https://example.com:99999",
            expected: Expected::Refused(UnusablePart::Port),
        },
        Case {
            shape: "an empty host",
            typed: "https:///jellyfin",
            expected: Expected::Refused(UnusablePart::Host),
        },
        Case {
            shape: "whitespace inside the address is not repaired",
            typed: "https://ex ample.com",
            expected: Expected::Refused(UnusablePart::Host),
        },
        Case {
            shape: "an IPv6 address typed without its brackets",
            typed: "https://::1:8096",
            expected: Expected::Refused(UnusablePart::Host),
        },
        Case {
            shape: "an address typed with the scheme separator missing is not a sub-path",
            typed: "https//example.com",
            expected: Expected::Refused(UnusablePart::Path),
        },
        Case {
            shape: "an empty segment inside a sub-path is refused rather than collapsed",
            typed: "example.com//jellyfin",
            expected: Expected::Refused(UnusablePart::Path),
        },
        Case {
            shape: "a host outside ASCII, which this module refuses rather than converting",
            typed: "https://münchen.example",
            expected: Expected::Refused(UnusablePart::Host),
        },
    ];

    #[test]
    fn every_shape_becomes_what_the_record_says_it_becomes() {
        for case in CASES {
            match (BaseAddress::parse(case.typed), &case.expected) {
                (Ok(base), Expected::Base(expected)) => {
                    assert_eq!(base.join(""), *expected, "{}: {:?}", case.shape, case.typed);
                }
                (Err(refusal), Expected::Refused(part)) => {
                    assert_eq!(refusal.part(), *part, "{}: {:?}", case.shape, case.typed);
                }
                (produced, _) => panic!("{}: {:?} produced {produced:?}", case.shape, case.typed),
            }
        }
    }

    #[test]
    fn the_table_covers_at_least_the_fifteen_inputs_the_issue_asks_for() {
        assert!(CASES.len() >= 15, "{} cases", CASES.len());
    }

    /// The payload 0004 fixes: what was typed, unmodified, including the
    /// whitespace a person cannot see, so a client shows them what they typed.
    #[test]
    fn a_refusal_carries_what_was_typed_rather_than_what_the_core_made_of_it() {
        let refusal: AddressNotUsable =
            BaseAddress::parse("  ftp://example.com/jellyfin  ").expect_err("refused");
        assert_eq!(refusal.typed(), "  ftp://example.com/jellyfin  ");
        assert_eq!(refusal.part(), UnusablePart::Scheme);
    }

    /// The failure this whole record exists for. Reference resolution would
    /// produce `https://example.com/Users/Me` and the sub-path would be gone.
    #[test]
    fn a_base_path_survives_a_join() {
        let base = BaseAddress::parse("example.com/jellyfin").expect("usable");
        assert_eq!(
            base.join("Users/Me"),
            "https://example.com/jellyfin/Users/Me"
        );
    }

    /// The sub-path is the case that is absent on the machine the code was
    /// written on and present on the operator's, so it is asked of every shape
    /// of request path rather than of one.
    #[test]
    fn a_base_path_survives_every_shape_of_request_path() {
        let base = BaseAddress::parse("https://example.com:8096/jellyfin").expect("usable");
        for path in [
            "Users/Me",
            "/Users/Me",
            "System/Info/Public",
            "Items/abc123/Images/Primary",
            "Sessions/Playing/Progress",
            "QuickConnect/Initiate",
        ] {
            let joined = base.join(path);
            assert!(
                joined.starts_with("https://example.com:8096/jellyfin/"),
                "{path}: {joined}"
            );
            assert!(!joined.contains("/jellyfin//"), "{path}: {joined}");
        }
    }

    #[test]
    fn a_leading_separator_on_the_request_path_does_not_double_the_one_the_join_adds() {
        let base = BaseAddress::parse("https://example.com").expect("usable");
        assert_eq!(base.join("/Users/Me"), base.join("Users/Me"));
        assert_eq!(base.join("///Users/Me"), "https://example.com/Users/Me");
    }

    #[test]
    fn joining_nothing_is_the_base_with_no_separator_after_it() {
        let base = BaseAddress::parse("https://example.com/jellyfin/").expect("usable");
        assert_eq!(base.join(""), "https://example.com/jellyfin");
        assert_eq!(base.join("/"), "https://example.com/jellyfin");
    }
}
