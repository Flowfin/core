//! The device identity, and who supplies each part of it.
//!
//! `docs/decisions/0036-the-device-identity-and-who-supplies-it.md` is the
//! record and #36 is the issue. It fixes three parts and gives the core the
//! smallest role in each: the client holds the identifier and the name, the core
//! owns the shape of the capability description and fills none of it in, and the
//! core generates an identifier only when a client asks for one and hands it
//! straight back rather than keeping a copy.
//!
//! # Why the core keeps nothing, which is what shapes every type here
//!
//! 0041 puts the identifier inside the cache key and 0046 serves from cache
//! before a session is restored, so every part of a key has to be in the
//! client's hands before the core has read anything. A core that stored the
//! identifier would have to read a store in order to build the name of the entry
//! it wants out of that store. So there is no field anywhere below that outlives
//! a call, and [`DeviceIdentifier::generate`] is an associated function rather
//! than a method on anything.
//!
//! # What a changed identifier costs, which is why the width is refused
//!
//! 0036 reads the server keying a live session on the client name joined to the
//! device identifier. An identifier that moves between starts leaves a fresh
//! session and a token behind on every start, in the operator's own session
//! list, that nothing will ever sign out. The two ways to arrive there are a
//! client that did not keep the value and a value that was never distinct in the
//! first place, and the second is the one this module can refuse.
//!
//! # Where the bytes come from
//!
//! 0011 measured the toolchain and found no source of unpredictable bytes on a
//! stable build, so the seam 0032 named is what is used and the client supplies
//! them. That is a decision taken in those two records rather than here, and the
//! only thing this module adds is the width below and a refusal of anything
//! narrower.

/// Why a part of a device identity or a capability description was refused.
///
/// A local answer rather than a value of the failure vocabulary. 0037 requires
/// every value of that vocabulary to be built at one mapping point. THIS
/// SENTENCE SAID [`crate::failure`] HOLDS NO TYPE TODAY; it holds one since #37
/// landed, and nothing here maps onto it, because a refused part of an identity
/// is a client handing the core something wrong rather than an answer being
/// read. This is the same shape as [`crate::session::SecretStoreUnavailable`]
/// for the same reason.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartNotUsable {
    /// The client supplied nothing where it owes a value.
    ///
    /// An empty string, an empty list, or a size of zero. The core has exactly
    /// two answers available to it here and 0036 removes one of them: it may not
    /// invent a value, because a name the core guessed would put the same word
    /// in front of every operator running any of the eleven clients, and a
    /// description the core guessed would be a platform claim it cannot check.
    /// What is left is to refuse.
    Absent,
    /// Fewer unpredictable bytes than [`LEAST_UNPREDICTABLE_BYTES`].
    ///
    /// Carries what was supplied, because the caller's own count is the thing it
    /// has to change, and a refusal that does not say the number sends somebody
    /// to read this file.
    FewerBytesThanTheWidth {
        /// How many bytes reached [`DeviceIdentifier::generate`].
        supplied: usize,
    },
}

/// The fewest unpredictable bytes an identifier may be generated from.
///
/// 0032 fixes at least 128 bits for the value tying a delegated sign-in to its
/// answer, and 0036 takes the same shape for the device identity. Sixteen bytes
/// is that width, and it buys collision resistance and nothing else: 0041 says
/// plainly that anybody holding the device can compute a cache key, so this
/// value is not a secret, is not stored through 0033, and a client may show it
/// to a person who asks what their device is called.
pub const LEAST_UNPREDICTABLE_BYTES: usize = 16;

/// The sixteen digits an identifier is written in.
///
/// A table rather than a conversion call, so that writing out a value the width
/// above admits has no failing case at all and nothing here can panic. Every
/// index into it comes from four bits.
const HEXADECIMAL: [u8; 16] = *b"0123456789abcdef";

/// Refuses an empty string where the client owes a value.
fn supplied_text(value: &str) -> Result<String, PartNotUsable> {
    if value.is_empty() {
        return Err(PartNotUsable::Absent);
    }
    Ok(value.to_owned())
}

/// What names this installation of this client on this device to a server.
///
/// An opaque string with no meaning. 0036 says what it may not be built from and
/// says it with the reasons: not a hardware serial, a network address or an
/// advertising identifier; not a hash of one, because a hash of a stable
/// identifier is a stable identifier with a longer name; and not anything about
/// the person, because 0068 puts the account name on its personal data list and
/// a derived identifier would carry it into every request header, into 0041's
/// key construction and into 0033's item label.
///
/// WHAT THIS TYPE ENFORCES OF THAT AND WHAT IT CANNOT. It takes a byte slice and
/// nothing else, so the core adds no input of its own, and there is nothing in
/// this crate for it to reach: `no-platform-clock` and `no-filesystem-access` in
/// `.github/invariants/rules` refuse a reading of the machine anywhere under
/// `src/`. Whether the bytes a client supplied were themselves a serial number
/// is not decidable here and no test below claims it is. That obligation is the
/// client's, it is written into 0036, and the conformance suite in #76 is where
/// a client is asked about it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceIdentifier {
    value: String,
}

impl DeviceIdentifier {
    /// Produces an identifier from unpredictable bytes the client supplied, and
    /// keeps no copy of it.
    ///
    /// The bytes are written out as lower-case hexadecimal and nothing else
    /// happens to them. A digest here would be free to add and would say
    /// something untrue: it would read as though the core had made the value
    /// harder to guess, when what it would actually do is fix the width of a
    /// value whose unpredictability is entirely the client's doing. 0036 says
    /// the identifier is not a secret, so there is nothing for a digest to
    /// protect, and the honest construction is the one that adds nothing.
    ///
    /// The client stores what comes back. Calling this a second time produces a
    /// second identifier rather than the first one again, which is what "hands
    /// it straight back rather than keeping a copy" means at the level of a
    /// signature: there is nowhere in this crate for the first one to have been
    /// kept.
    ///
    /// # Errors
    ///
    /// [`PartNotUsable::FewerBytesThanTheWidth`] where fewer than
    /// [`LEAST_UNPREDICTABLE_BYTES`] were supplied. Refused rather than accepted
    /// and padded, because a narrow value collides between installations, and a
    /// collision here is two devices sharing one session on the server and one
    /// key space in the cache.
    pub fn generate(unpredictable_bytes: &[u8]) -> Result<Self, PartNotUsable> {
        let supplied = unpredictable_bytes.len();
        if supplied < LEAST_UNPREDICTABLE_BYTES {
            return Err(PartNotUsable::FewerBytesThanTheWidth { supplied });
        }
        let mut value = String::with_capacity(supplied * 2);
        for byte in unpredictable_bytes {
            for nibble in [byte >> 4, byte & 0x0f] {
                value.push(char::from(HEXADECIMAL[usize::from(nibble)]));
            }
        }
        Ok(Self { value })
    }

    /// Takes the identifier a client kept across a restart.
    ///
    /// This is the ordinary way in. Generation is the exception, for a client
    /// whose platform has no notion of its own, and both produce the same type
    /// because the server cannot tell them apart either.
    ///
    /// # Errors
    ///
    /// [`PartNotUsable::Absent`] for an empty string. The server joins the
    /// client name to the identifier to key a live session, so an empty
    /// identifier collapses every device running that client into one session,
    /// and the operator's session list stops being able to name a device at all.
    pub fn kept(value: &str) -> Result<Self, PartNotUsable> {
        Ok(Self {
            value: supplied_text(value)?,
        })
    }

    /// The value, as the client keeps it and as the server reads it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// What a person sees in the server's own session list.
///
/// The client holds it and the core never invents one, because only the client
/// knows whether it is a television in a living room or a handset.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceName {
    value: String,
}

/// What the client calls itself to a server.
///
/// 0036 reads the client name and its version travelling beside the device
/// identity in the same authorization value, and puts both on the same footing
/// as the device name: the client supplies them and the core sends what it was
/// given. It is a separate type from [`DeviceName`] because the server reads the
/// two into different fields and keys a session on this one joined to the
/// identifier, so two strings that swapped places at a call site are two wrong
/// answers rather than none.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientName {
    value: String,
}

/// Which version of the client this is.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientVersion {
    value: String,
}

/// Writes the two calls a part the client owns carries, and nothing else.
///
/// The three types above differ in what they mean to a server and not at all in
/// what they do here. Written out three times, the risk is not the repetition:
/// it is that one of the three quietly stops refusing an empty string, which is
/// the direction that fails silently, because the value ends up in a header
/// rather than in a crash.
macro_rules! supplied_string {
    ($type:ty, $what:literal) => {
        impl $type {
            #[doc = concat!("Takes the ", $what, " the client supplied.")]
            ///
            /// # Errors
            ///
            /// [`PartNotUsable::Absent`] for an empty string. 0036 leaves the
            /// core no second answer: inventing one is what that record refuses
            /// by name, so refusing is what is left.
            pub fn supplied(value: &str) -> Result<Self, PartNotUsable> {
                Ok(Self {
                    value: supplied_text(value)?,
                })
            }

            /// The value, as the client wrote it and as the server reads it.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.value
            }
        }
    };
}

supplied_string!(DeviceName, "device name");
supplied_string!(ClientName, "client name");
supplied_string!(ClientVersion, "client version");

/// The parts of the authorization value that are not the token.
///
/// 0036 reads the server taking named parts out of that value, and this type
/// holds the ones a session does not own. The token is the session's and is not
/// here.
///
/// Every part is supplied. There is no constructor that fills one in, no
/// [`Default`], and no method that changes one afterwards, so a client that has
/// not decided what to call itself cannot reach a core that decided for it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIdentity {
    identifier: DeviceIdentifier,
    device_name: DeviceName,
    client_name: ClientName,
    client_version: ClientVersion,
}

impl DeviceIdentity {
    /// Assembles the parts the client holds.
    #[must_use]
    pub const fn of(
        identifier: DeviceIdentifier,
        device_name: DeviceName,
        client_name: ClientName,
        client_version: ClientVersion,
    ) -> Self {
        Self {
            identifier,
            device_name,
            client_name,
            client_version,
        }
    }

    /// The identifier, which is also 0041's device part of a cache key.
    #[must_use]
    pub const fn identifier(&self) -> &DeviceIdentifier {
        &self.identifier
    }

    /// The name a person sees in the server's session list.
    #[must_use]
    pub const fn device_name(&self) -> &DeviceName {
        &self.device_name
    }

    /// The client's own name, which the server joins to the identifier.
    #[must_use]
    pub const fn client_name(&self) -> &ClientName {
        &self.client_name
    }

    /// The client's version.
    #[must_use]
    pub const fn client_version(&self) -> &ClientVersion {
        &self.client_version
    }
}

/// The largest picture the client says it can decode.
///
/// 0036's "at what sizes", as two counts of pixels. Not a bitrate and not a
/// quality: what the core carries is what the client stated it can decode, and
/// whether the server holds anything that size is the server's answer to give.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LargestPicture {
    across: u32,
    down: u32,
}

impl LargestPicture {
    /// Takes the two counts the client supplied.
    ///
    /// # Errors
    ///
    /// [`PartNotUsable::Absent`] where either count is zero, which describes a
    /// client that can decode nothing rather than a client that did not say.
    pub const fn supplied(across: u32, down: u32) -> Result<Self, PartNotUsable> {
        if across == 0 || down == 0 {
            return Err(PartNotUsable::Absent);
        }
        Ok(Self { across, down })
    }

    /// Pixels across.
    #[must_use]
    pub const fn across(self) -> u32 {
        self.across
    }

    /// Pixels down.
    #[must_use]
    pub const fn down(self) -> u32 {
        self.down
    }
}

/// What this client can actually play, in the shape the core owns.
///
/// 0036 splits this in two and the split is the whole point of the type. The
/// core owns the shape, so that eleven clients do not describe the same thing
/// eleven ways and the drift does not show up as one platform silently getting
/// transcoded streams it did not need while another gets a container it cannot
/// open. The client supplies the contents, because what a platform can decode is
/// a fact about the platform and 0003 keeps platform knowledge out of the core.
///
/// # What the shape is, and what fixes it
///
/// The three parts 0036 names and no fourth: what the client can decode, at what
/// sizes, over what containers. A fourth part is a change to that record rather
/// than a field added here, because a shape that grows at a call site is eleven
/// shapes again by the second client.
///
/// # What is not decided here
///
/// Which values are legal. A container name and a codec name are platform facts,
/// the core carries the strings it was given, and nothing below reads them.
///
/// How the description is written on the wire and which call carries it. 0010
/// names one endpoint for the stored description and another for the one a
/// playback call carries, and choosing between them is #111. Neither call exists
/// in this tree: the transport is #27 and is not built.
///
/// That a description is sent on every sign-in rather than once per installation
/// is 0036's rule and it is a rule about a call. Nothing here sends anything, so
/// nothing here keeps it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    containers: Vec<String>,
    codecs: Vec<String>,
    largest_picture: LargestPicture,
}

impl Capabilities {
    /// Takes the description the client supplied, unchanged and unaugmented.
    ///
    /// The lists are kept in the order they arrived and nothing is added to
    /// them. A core that sorted them would be describing a preference the client
    /// did not state, and a core that added an entry would be claiming a
    /// platform fact it cannot check. 0036 fixes the second as the failure that
    /// is hardest to debug: an overstated description produces a stream that
    /// fails inside a decoder, which is the one place the core has no visibility
    /// and the client has no error to map.
    ///
    /// # Errors
    ///
    /// [`PartNotUsable::Absent`] where either list is empty or carries an empty
    /// entry. A client that names no container describes a device that can open
    /// nothing, and the core may not answer that with a default of its own.
    pub fn supplied(
        containers: &[&str],
        codecs: &[&str],
        largest_picture: LargestPicture,
    ) -> Result<Self, PartNotUsable> {
        Ok(Self {
            containers: each_supplied(containers)?,
            codecs: each_supplied(codecs)?,
            largest_picture,
        })
    }

    /// The containers the client said it can open, in the order it gave them.
    #[must_use]
    pub fn containers(&self) -> &[String] {
        &self.containers
    }

    /// The codecs the client said it can decode, in the order it gave them.
    #[must_use]
    pub fn codecs(&self) -> &[String] {
        &self.codecs
    }

    /// The largest picture the client said it can decode.
    #[must_use]
    pub const fn largest_picture(&self) -> LargestPicture {
        self.largest_picture
    }
}

/// Refuses an empty list and an empty entry inside one.
fn each_supplied(values: &[&str]) -> Result<Vec<String>, PartNotUsable> {
    if values.is_empty() {
        return Err(PartNotUsable::Absent);
    }
    values.iter().map(|value| supplied_text(value)).collect()
}

#[cfg(test)]
mod tests {
    //! #36's three conditions, and what each test can and cannot say.
    //!
    //! The first condition is that the identifier is stable across restarts. In
    //! 0036's arrangement the client is what holds it, so a restart is modelled
    //! the way a client meets one: the value is written out, everything the core
    //! built is dropped, and the value is read back. There is no process here
    //! and no store, and that is the point rather than a shortcut, since a core
    //! holding either would be the arrangement 0036 refused.
    //!
    //! The second is that the identifier is not derived from a hardware
    //! identifier or from personal data. What is provable is that the core adds
    //! no input of its own: the same bytes produce the same value, and the value
    //! is those bytes and nothing else. Whether what a client supplied was
    //! itself a serial number is outside every reading available here, and no
    //! test below pretends otherwise.
    //!
    //! The third is that the capability description comes from the client rather
    //! than being assumed by the core. That one is provable in both directions,
    //! and both are below: what the client gave comes back unchanged and
    //! unaugmented, and what the client left out is refused rather than filled
    //! in.

    use super::{
        Capabilities, ClientName, ClientVersion, DeviceIdentifier, DeviceIdentity, DeviceName,
        LEAST_UNPREDICTABLE_BYTES, LargestPicture, PartNotUsable,
    };

    /// Sixteen bytes, which is the width, written out so that the expected
    /// hexadecimal below can be read against them by eye.
    const BYTES: [u8; 16] = [
        0x00, 0x01, 0x0f, 0x10, 0x7f, 0x80, 0xa5, 0xff, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x71, 0x82,
        0x93,
    ];

    fn an_identity(identifier: DeviceIdentifier) -> DeviceIdentity {
        DeviceIdentity::of(
            identifier,
            DeviceName::supplied("the television in the front room").expect("a name was supplied"),
            ClientName::supplied("flowfin-tv").expect("a client name was supplied"),
            ClientVersion::supplied("0.0.0").expect("a version was supplied"),
        )
    }

    fn a_picture() -> LargestPicture {
        LargestPicture::supplied(3840, 2160).expect("both counts were supplied")
    }

    /// The first condition. A client generates once, keeps the string, and comes
    /// back with it after a restart; what it comes back with is the identity it
    /// had. The identifier is the one part of it a server keys a live session
    /// on, so this is the property that stops an operator's session list filling
    /// with entries for one device that nobody can tell apart.
    #[test]
    fn an_identifier_the_client_kept_survives_a_restart() {
        let first_run =
            an_identity(DeviceIdentifier::generate(&BYTES).expect("the width was supplied"));
        let written_out = first_run.identifier().as_str().to_owned();
        drop(first_run);

        let second_run =
            an_identity(DeviceIdentifier::kept(&written_out).expect("the client kept a value"));
        assert_eq!(second_run.identifier().as_str(), written_out);
    }

    /// The half of "hands it straight back rather than keeping a copy" that a
    /// test can observe. A core that had kept the first value would have it to
    /// hand out again, and the second call would answer with it.
    #[test]
    fn generating_a_second_identifier_does_not_return_the_first() {
        let first = DeviceIdentifier::generate(&BYTES).expect("the width was supplied");
        let mut other = BYTES;
        other[15] = 0x94;
        let second = DeviceIdentifier::generate(&other).expect("the width was supplied");
        assert_ne!(first, second);
    }

    /// The second condition, in the only direction a reading here reaches: the
    /// bytes the client supplied are the whole of the input. Two calls with the
    /// same bytes agree, so nothing that changes between calls reaches the value.
    #[test]
    fn the_supplied_bytes_are_the_whole_of_the_input() {
        let first = DeviceIdentifier::generate(&BYTES).expect("the width was supplied");
        let second = DeviceIdentifier::generate(&BYTES).expect("the width was supplied");
        assert_eq!(first, second);
        assert_eq!(
            first.as_str(),
            "00010f107f80a5ff2c3d4e5f60718293",
            "the value is the bytes and nothing else"
        );
    }

    /// The refusal the width buys, at the one-byte boundary rather than at a
    /// number that could not have passed.
    #[test]
    fn one_byte_short_of_the_width_is_refused_and_says_what_arrived() {
        let short = [0x11_u8; LEAST_UNPREDICTABLE_BYTES - 1];
        assert_eq!(
            DeviceIdentifier::generate(&short),
            Err(PartNotUsable::FewerBytesThanTheWidth {
                supplied: LEAST_UNPREDICTABLE_BYTES - 1
            })
        );
        assert!(DeviceIdentifier::generate(&[0x11_u8; LEAST_UNPREDICTABLE_BYTES]).is_ok());
    }

    /// An empty identifier collapses every device running one client into a
    /// single session on the server, because the key is the client name joined
    /// to this value.
    #[test]
    fn an_empty_identifier_is_refused_rather_than_kept() {
        assert_eq!(DeviceIdentifier::kept(""), Err(PartNotUsable::Absent));
    }

    /// The three parts a client owns and the core may not invent. Each is
    /// refused separately, because a helper shared by three types is a helper
    /// one of them can quietly stop calling.
    #[test]
    fn a_part_the_client_owes_is_refused_rather_than_invented() {
        assert_eq!(DeviceName::supplied(""), Err(PartNotUsable::Absent));
        assert_eq!(ClientName::supplied(""), Err(PartNotUsable::Absent));
        assert_eq!(ClientVersion::supplied(""), Err(PartNotUsable::Absent));
    }

    /// What the client wrote is what the identity carries, in every part. The
    /// server reads them into different fields, so a type that swapped two of
    /// them would be wrong in a way nothing on the wire would report.
    #[test]
    fn every_part_of_an_identity_is_the_string_the_client_wrote() {
        let identity = an_identity(DeviceIdentifier::kept("kept-9f2c").expect("a value was kept"));
        assert_eq!(identity.identifier().as_str(), "kept-9f2c");
        assert_eq!(
            identity.device_name().as_str(),
            "the television in the front room"
        );
        assert_eq!(identity.client_name().as_str(), "flowfin-tv");
        assert_eq!(identity.client_version().as_str(), "0.0.0");
    }

    /// The third condition, in the direction that says the core added nothing.
    /// Order is part of it: a core that sorted either list would be stating a
    /// preference the client did not.
    #[test]
    fn a_description_comes_back_unchanged_and_unaugmented() {
        let description =
            Capabilities::supplied(&["mkv", "mp4"], &["hevc", "h264", "av1"], a_picture())
                .expect("every part was supplied");

        assert_eq!(description.containers(), ["mkv", "mp4"]);
        assert_eq!(description.codecs(), ["hevc", "h264", "av1"]);
        assert_eq!(description.largest_picture(), a_picture());
        assert_eq!(description.largest_picture().across(), 3840);
        assert_eq!(description.largest_picture().down(), 2160);
    }

    /// The third condition in the other direction. A part the client left out is
    /// refused, so there is no route to a description the core wrote any of.
    #[test]
    fn a_part_the_client_left_out_is_refused_rather_than_filled_in() {
        assert_eq!(
            Capabilities::supplied(&[], &["hevc"], a_picture()),
            Err(PartNotUsable::Absent)
        );
        assert_eq!(
            Capabilities::supplied(&["mkv"], &[], a_picture()),
            Err(PartNotUsable::Absent)
        );
        assert_eq!(
            Capabilities::supplied(&["mkv", ""], &["hevc"], a_picture()),
            Err(PartNotUsable::Absent)
        );
        assert_eq!(
            LargestPicture::supplied(0, 2160),
            Err(PartNotUsable::Absent)
        );
        assert_eq!(
            LargestPicture::supplied(3840, 0),
            Err(PartNotUsable::Absent)
        );
    }
}
