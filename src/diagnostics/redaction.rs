//! What may leave through a diagnostic event, decided per field name and applied
//! before the client's sink is called.
//!
//! 0071 is the record and #71 is the issue. Three treatments and nothing else:
//! a field is excluded outright, reduced to a correlator that means nothing
//! outside the run that produced it, or carried whole. Which of the three a field
//! gets does not vary with severity.
//!
//! # Where the decision lives
//!
//! On the name, at the site the name is written. 0071's decision sentence is that
//! redaction is decided per field NAME rather than per value shape, and that is
//! what [`FieldName`] carries. A name is constructed through one of three calls,
//! each of which is a treatment, so there is no way to write a field whose
//! treatment nobody chose, and the compiler is what refuses one rather than a
//! review.
//!
//! 0071's default, that a field nobody has classified is excluded, is therefore a
//! rule with nothing to apply to here. THAT IS THE STRONGER FORM RATHER THAN THE
//! RULE MISSING: the direction the default exists to fail in is kept by making
//! the unclassified field unwritable, and a default that never fires cannot fire
//! the wrong way. What it costs is that the rule is not readable as one line of
//! code somebody can point at, which is why it is written here.
//!
//! WHAT THAT DOES NOT REACH is a field classified wrongly. The compiler asks
//! whether somebody chose, never whether they chose correctly, and 0068's
//! question - could two people running the same build against the same server
//! hold different values here - is a judgement a reading of this tree does not
//! make. The review is where a wrong choice is caught, and #71's own condition,
//! a test that drives a full session at the most verbose level and searches the
//! output for every named field, is what would catch one afterwards. That test
//! needs the fake server in #21 and is not here.
//!
//! # Where the treatment is applied
//!
//! In [`crate::diagnostics::Diagnostics::emit`], before the sink is called. 0101
//! places the sink outside the boundary and says everything handed to it is
//! treated as though it will be published, which is what makes the sink the
//! boundary rather than the error type. An error returned to a caller in the same
//! process is not redacted and is not this module's subject.
//!
//! # What is not here
//!
//! No list of field names. 0068 holds the set of data this rule is about and
//! 0071 refuses to copy it. A field name is written where the event that carries
//! it is written, with its treatment on it, which is the placement 0100 already
//! takes for an event's identity. [`crate::cache::bound`] is where the names
//! this tree emits today live, and there are five.
//!
//! No machine-readable statement of what was excluded and what was reduced. 0071
//! asks the core for one, over the field names it can emit. Assembling it needs
//! a set gathered across every subsystem, and the placement above deliberately
//! puts the names with their subsystems, so the gathering happens where the core
//! is created rather than in a list here that would drift against the emit
//! sites. Creating the core is #115. #71 stays open on that half.
//!
//! No source for the salt. 0071 has it created when the core is created, and the
//! core reads no platform, so it arrives from outside the way 0036 has the device
//! identity arrive. Creating and stopping the core is #115, and nothing in this
//! tree creates one.

use super::FieldValue;
use sha2::{Digest, Sha256};

/// What happens to a field on its way to a sink.
///
/// Three and no fourth. A fourth would be a change to 0071, because the three are
/// the whole of what that record admits and a client reading the core's statement
/// about a field is reading one of these.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Treatment {
    /// The field never appears in an event at any severity, with no
    /// configuration that admits it. The session token and anything derived from
    /// it, a password, and whatever the secret store handed back.
    Excluded,
    /// The field appears as a [`Correlator`]. The account identifier, the item
    /// and other server-supplied identifiers, the device identity, the server
    /// identity and the address it came from.
    Reduced,
    /// The field appears unchanged. Counts, intervals, an error kind, a
    /// capability name, a status, a severity, an event identity, a truth. None of
    /// them can differ between two people running the same build against the same
    /// server, which is 0068's own test applied field by field.
    CarriedWhole,
}

/// The name of one field, carrying the treatment it gets.
///
/// The two are one value on purpose. A name in one place and a treatment in
/// another is a pair that comes apart the first time somebody adds a field
/// without reading the other file, and that is 0071's own reversal condition
/// rather than a worry invented here.
///
/// The name is `&'static str` for the reason an event identity is: it is written
/// at the emit site and is never assembled out of what arrived from somewhere, so
/// what the rule reads is a fixed name rather than whatever a server put in an
/// answer.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldName {
    name: &'static str,
    treatment: Treatment,
}

impl FieldName {
    /// A name whose value is carried unchanged.
    ///
    /// The question to answer before writing this rather than one of the two
    /// below is 0068's: could two people running the same build against the same
    /// server hold different values here. Where they could, this is the wrong
    /// call.
    #[must_use]
    pub const fn carried_whole(name: &'static str) -> Self {
        Self {
            name,
            treatment: Treatment::CarriedWhole,
        }
    }

    /// A name whose value appears as a correlator and never as itself.
    #[must_use]
    pub const fn reduced(name: &'static str) -> Self {
        Self {
            name,
            treatment: Treatment::Reduced,
        }
    }

    /// A name whose value never leaves at all.
    ///
    /// Writing a field under one of these is not pointless and is not the same as
    /// leaving the field out. It says at the emit site that somebody considered
    /// carrying the value and that the answer is no, which is the sentence the
    /// next person to want it there reads.
    #[must_use]
    pub const fn excluded(name: &'static str) -> Self {
        Self {
            name,
            treatment: Treatment::Excluded,
        }
    }

    /// The name, as a sink receives it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.name
    }

    /// Which of the three it gets.
    #[must_use]
    pub const fn treatment(self) -> Treatment {
        self.treatment
    }
}

/// The value a correlator is computed with, created when the core is created.
///
/// It is held in memory for the life of the core, never written and never
/// emitted, which is why nothing here reads it back out. A type with no accessor
/// is what keeps that promise rather than a sentence asking callers not to, and
/// the debug shape below is written out by hand for the same reason.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// It arrives from outside because the core reads no platform and has no source
/// of its own, which is the same reason 0036 has the device identity supplied.
/// Two cores created in one process hold two salts and their correlators do not
/// line up, which is the behaviour rather than a defect: 0071's property is that
/// a correlator means nothing outside the run that produced it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CorrelatorSalt {
    bytes: [u8; Self::WIDTH],
}

/// Written out rather than derived, so that the one value this type exists to
/// keep off every output cannot reach one through the trait every type in this
/// crate carries.
impl core::fmt::Debug for CorrelatorSalt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CorrelatorSalt").finish_non_exhaustive()
    }
}

impl CorrelatorSalt {
    /// How many bytes a salt is.
    ///
    /// Thirty-two, which is the width of the digest the correlator is taken from.
    /// A salt narrower than the digest is the part of the construction an
    /// attacker would attack first, and there is no reason here to make it
    /// narrower.
    pub const WIDTH: usize = 32;

    /// The salt, from bytes whoever created the core supplied.
    ///
    /// What those bytes have to be is not decided here and cannot be checked
    /// here: a value that is unpredictable to anybody reading a report. A salt
    /// assembled from something a reader of the report also holds gives a
    /// correlator that identifies rather than correlates, which is 0071's
    /// argument against an unsalted digest arriving one step later.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; Self::WIDTH]) -> Self {
        Self { bytes }
    }
}

/// How many characters of the digest a correlator carries.
///
/// Sixteen, which is sixty-four bits. It is chosen rather than measured and the
/// two directions it is chosen between are: wide enough that two distinct values
/// in one run do not collide, and narrow enough to be read and compared by
/// somebody looking at a report by eye. A collision costs a reader thinking two
/// failures were about one thing, which is the mistake the correlator exists to
/// prevent the opposite of, and sixty-four bits is far past the number of
/// distinct identifiers one run of a client sees.
pub const CORRELATOR_WIDTH: usize = 16;

/// What a reduced field carries instead of its value.
///
/// Within one run, two events about the same value carry the same correlator, so
/// a report shows that one thing failed eleven times rather than that eleven
/// things failed once. Across runs and across devices it means nothing, because
/// the salt differs. The second is a real cost and somebody chasing an
/// intermittent fault across restarts pays it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correlator {
    text: String,
}

impl Correlator {
    /// The correlator for one value under one salt.
    ///
    /// The salt goes into the digest before the value, so that the construction
    /// is not one an attacker can extend from a digest of a prefix. Nothing here
    /// depends on that today, because no correlator is ever published beside the
    /// value it came from, and it is the order that costs nothing to get right.
    ///
    /// A value that is not text is reduced too, through a fixed encoding of its
    /// bytes. 0071 places identifiers and addresses under this treatment and all
    /// of those are text, so nothing in this tree needs the other three; they are
    /// here because a treatment that answered only for one shape would put the
    /// decision back on the shape rather than on the name, which is the thing
    /// 0071 and 0100 both refuse.
    #[must_use]
    pub fn of(salt: &CorrelatorSalt, value: FieldValue<'_>) -> Self {
        let mut digest = Sha256::new();
        digest.update(salt.bytes);
        match value {
            FieldValue::Count(count) => {
                digest.update(*b"c");
                digest.update(count.to_be_bytes());
            }
            FieldValue::Interval(interval) => {
                digest.update(*b"i");
                digest.update(interval.as_nanos().to_be_bytes());
            }
            FieldValue::Text(text) => {
                digest.update(*b"t");
                digest.update(text.as_bytes());
            }
            FieldValue::Truth(truth) => {
                digest.update(*b"b");
                digest.update([u8::from(truth)]);
            }
        }
        let mut text = String::with_capacity(CORRELATOR_WIDTH);
        for byte in digest.finalize().into_iter().take(CORRELATOR_WIDTH / 2) {
            text.push(HEX[usize::from(byte >> 4)]);
            text.push(HEX[usize::from(byte & 0x0F)]);
        }
        Self { text }
    }

    /// The correlator, as a sink receives it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// The character each half of a byte becomes.
///
/// A second table beside the one in [`crate::cache::key`] rather than one shared
/// between them, because the two are not the same subject: that one is inside the
/// key derivation 0041 fixes and this one is inside the correlator 0071 fixes,
/// and a shared helper would tie two records' constructions together for the sake
/// of sixteen characters.
static HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[cfg(test)]
mod tests {
    //! What the three treatments, the correlator and the salt are proven with
    //! here, and what is proven one module up instead.
    //!
    //! The conditions in [`crate::diagnostics`] drive whole events through the
    //! facility and read what a sink was handed, which is where the rule meets
    //! its boundary. These are about the pieces underneath: that a name carries
    //! the treatment it was built with wherever it is built, that every shape a
    //! field value can take is reduced rather than only the one this tree emits
    //! today, and that the salt cannot be read back out of the type that holds
    //! it.

    use super::{CORRELATOR_WIDTH, Correlator, CorrelatorSalt, FieldName, Treatment};
    use crate::diagnostics::FieldValue;
    use core::time::Duration;

    fn a_salt() -> CorrelatorSalt {
        CorrelatorSalt::from_bytes([0x5a; CorrelatorSalt::WIDTH])
    }

    fn another_salt() -> CorrelatorSalt {
        CorrelatorSalt::from_bytes([0xa5; CorrelatorSalt::WIDTH])
    }

    /// Each of the three calls is the treatment, and the name comes back out
    /// unchanged. Built here rather than as a constant so the calls are made at
    /// a moment a run can see.
    #[test]
    fn a_name_carries_the_treatment_the_call_that_built_it_names() {
        let whole = FieldName::carried_whole("attempts");
        let reduced = FieldName::reduced("item");
        let excluded = FieldName::excluded("token");

        assert_eq!(whole.treatment(), Treatment::CarriedWhole);
        assert_eq!(reduced.treatment(), Treatment::Reduced);
        assert_eq!(excluded.treatment(), Treatment::Excluded);

        assert_eq!(whole.as_str(), "attempts");
        assert_eq!(reduced.as_str(), "item");
        assert_eq!(excluded.as_str(), "token");
    }

    /// Two names differing only in their treatment are two values, so a name
    /// cannot be compared without its treatment coming with it.
    #[test]
    fn one_name_under_two_treatments_is_two_values() {
        assert_ne!(FieldName::reduced("item"), FieldName::excluded("item"));
        assert_eq!(FieldName::reduced("item"), FieldName::reduced("item"));
    }

    /// Every shape a field value can take is reduced, not only the one this tree
    /// emits under a reduced name today. 0071 puts the decision on the name, and
    /// a reduction that answered for one shape would put it back on the shape.
    #[test]
    fn every_shape_of_value_reduces() {
        let salt = a_salt();
        let shapes = [
            FieldValue::Count(7),
            FieldValue::Interval(Duration::from_millis(7)),
            FieldValue::Text("seven"),
            FieldValue::Truth(true),
        ];

        let correlators: Vec<String> = shapes
            .iter()
            .map(|shape| Correlator::of(&salt, *shape).as_str().to_owned())
            .collect();

        for correlator in &correlators {
            assert_eq!(correlator.len(), CORRELATOR_WIDTH);
            assert!(
                correlator.chars().all(|c| c.is_ascii_hexdigit()),
                "the correlator was {correlator}"
            );
        }

        let mut distinct = correlators.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            correlators.len(),
            "two shapes reduced to one correlator"
        );
    }

    /// The shape is inside the digest as well as the value, so a count and a
    /// truth that would read the same do not correlate as one thing.
    #[test]
    fn two_shapes_carrying_the_same_number_do_not_correlate() {
        let salt = a_salt();
        assert_ne!(
            Correlator::of(&salt, FieldValue::Count(1)).as_str(),
            Correlator::of(&salt, FieldValue::Truth(true)).as_str()
        );
        assert_ne!(
            Correlator::of(&salt, FieldValue::Count(0)).as_str(),
            Correlator::of(&salt, FieldValue::Truth(false)).as_str()
        );
    }

    /// The same value under two salts is two correlators, which is the property
    /// the salt exists for: a correlator means nothing outside the run that
    /// produced it.
    #[test]
    fn the_salt_is_inside_the_correlator() {
        assert_ne!(
            Correlator::of(&a_salt(), FieldValue::Text("one")).as_str(),
            Correlator::of(&another_salt(), FieldValue::Text("one")).as_str()
        );
        assert_eq!(
            Correlator::of(&a_salt(), FieldValue::Text("one")).as_str(),
            Correlator::of(&a_salt(), FieldValue::Text("one")).as_str()
        );
    }

    /// The one value this type exists to keep off every output cannot reach one
    /// through the trait every type in this crate carries. Written out by hand
    /// rather than derived, and this is what watches the derive coming back.
    #[test]
    fn the_salt_is_not_in_what_a_debug_printing_produces() {
        let written_out = format!("{:?}", a_salt());
        assert!(
            !written_out.contains("5a") && !written_out.contains("90"),
            "the salt was in what was written out: {written_out}"
        );
        assert!(written_out.starts_with("CorrelatorSalt"));
    }

    /// Two salts built from the same bytes are the same salt, which is what lets
    /// a condition anywhere in this suite name one.
    #[test]
    fn a_salt_is_its_bytes() {
        assert_eq!(a_salt(), a_salt());
        assert_ne!(a_salt(), another_salt());
        assert_eq!(CorrelatorSalt::WIDTH, 32);
    }

    /// A treatment is ordered and hashable so that a caller can put one in a set
    /// or a table, which is what the statement the core owes for a bundle will
    /// be built out of.
    #[test]
    fn the_three_treatments_are_ordered_and_distinct() {
        let mut all = [
            Treatment::CarriedWhole,
            Treatment::Excluded,
            Treatment::Reduced,
        ];
        all.sort_unstable();
        assert_eq!(
            all,
            [
                Treatment::Excluded,
                Treatment::Reduced,
                Treatment::CarriedWhole
            ]
        );
    }
}
