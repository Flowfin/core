//! Which source is played, which streams are chosen, and what the handover
//! carries.
//!
//! `docs/decisions/0111-which-source-is-played-and-the-handover.md` is the
//! record and #111 is the issue. The record decides five things: that the
//! description travels with the play call and the answer is read against that
//! one; that a source sits on the highest of four rungs the description admits,
//! judged after its streams are chosen, with a tie going to the order the server
//! listed; how the audio and subtitle streams are chosen; what the handover
//! carries and nothing else; and that nothing playable is `request-refused`
//! from 0004 rather than an empty answer.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is everything of that a comparison over names and counts
//! settles: which of a source's streams will be played, whether the description
//! covers each of them and the container, which rung that puts the source on,
//! which of several sources is taken, and the value the call hands back. None
//! of it reads a clock, a socket or a store.
//!
//! WHAT IS NOT HERE IS THE CALL. The play call is a request carrying the
//! description in its body, 0010 names the path, and the transport is #27, so
//! nothing here sends or receives a byte. [`WhatTheServerOffered`] is the
//! server's answer as values rather than as bytes, and the reading that turns an
//! answer body into one is a reading 0037 places at its mapping point, for a
//! body nothing in this tree parses. #111's four test conditions are each a run
//! against the fake server, and every one of them is untouched by this module.
//!
//! WHAT IS ALSO NOT HERE IS THE ADDRESS OF A SOURCE PLAYED AS IT STANDS. 0111
//! says the handover carries an address the platform's own player opens. For
//! every rung but the top one the server supplies that address in its answer,
//! and [`WhatThePlayerOpens::TheServersConversion`] carries it as it arrived.
//! For a source played as it stands the server supplies none, the address is a
//! path the core would have to build, and 0010's table carries no row for it.
//! 0010 is the authority for which paths the core may name and says of itself
//! that a path outside its table arrives by a superseding record, so
//! [`WhatThePlayerOpens::TheSourceAsItStands`] names the source and carries no
//! address, and the row is owed to that record rather than invented here.
//!
//! # Why the ladder is here rather than at the call
//!
//! 0111's own sentence: the shortest correct-looking code takes the first
//! source the server listed, it compiles, it works on every machine where every
//! file is already playable, and it ships a stream the first narrower device
//! cannot decode, with the failure arriving from inside the platform's decoder,
//! where 0004 has no kind and the core no visibility. A ladder that was never
//! written does not appear in a diff as a missing ladder. So it is written
//! once, here, and the call asks it.
//!
//! # What the server's flags are, and what they are not
//!
//! Both supported lines compute whether a source may be played as it stands,
//! with its container changed, or converted, against the description the call
//! carried, and set the three flags [`RoutesOffered`] holds from that. The
//! ladder reads the same description and judges the streams that will be
//! played, and the two readings are meant to agree. Where they do not, which
//! 0111 names as a server that ignored the description, the source clears no
//! rung it has an address for and the answer is nothing playable, from the
//! core. That is the reading the record wants: the mismatch arrives here rather
//! than inside a decoder.

use super::{Ticks, resume::Resume};
use crate::failure::Failure;
use crate::session::device::{Capabilities, LargestPicture};

/// Which kind of stream a stream is, as the server reports it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Both supported lines report six kinds, and three of them are ones a player
/// plays. The other three are one variant here because the ladder never chooses
/// among them: an embedded image, a data track and a lyric track ride inside
/// the container and are decoded by nothing 0111 judges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    /// The picture.
    Video,
    /// The sound.
    Audio,
    /// Text or pictures shown over the picture.
    Subtitle,
    /// Anything else the container carries.
    Other,
}

/// The size of a video stream's picture, as the server states it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picture {
    across: u32,
    down: u32,
}

impl Picture {
    /// The two counts of pixels the server stated.
    #[must_use]
    pub const fn stated(across: u32, down: u32) -> Self {
        Self { across, down }
    }

    /// Whether this picture is no larger than the largest the client can
    /// decode, in both directions.
    const fn fits_within(self, largest: LargestPicture) -> bool {
        self.across <= largest.across() && self.down <= largest.down()
    }
}

/// One stream of a source, as the server described it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// What is carried is what the ladder and the stream rule read and nothing
/// else: the index the server names the stream by, its kind, its codec, its
/// language, whether the file marks it as its default, and for a picture its
/// size. The server states a great deal more about a stream and none of it
/// reaches a decision here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferedStream {
    index: i32,
    kind: StreamKind,
    codec: String,
    language: Option<String>,
    marked_default: bool,
    picture: Option<Picture>,
}

impl OfferedStream {
    /// Takes a stream as the server described it, unchanged.
    #[must_use]
    pub fn described(
        index: i32,
        kind: StreamKind,
        codec: &str,
        language: Option<&str>,
        marked_default: bool,
        picture: Option<Picture>,
    ) -> Self {
        Self {
            index,
            kind,
            codec: codec.to_owned(),
            language: language.map(str::to_owned),
            marked_default,
            picture,
        }
    }

    /// The index the server names this stream by.
    #[must_use]
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Which kind of stream it is.
    #[must_use]
    pub const fn kind(&self) -> StreamKind {
        self.kind
    }

    /// The codec, as the server names it.
    #[must_use]
    pub fn codec(&self) -> &str {
        &self.codec
    }

    /// The language, where the server states one.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// The picture's size, where the server states one.
    #[must_use]
    pub const fn picture(&self) -> Option<Picture> {
        self.picture
    }

    /// Whether the description covers this stream as it stands.
    ///
    /// The codec has to be one the client said it decodes, and a picture has to
    /// be no larger than the largest it said it decodes. A PICTURE WHOSE SIZE
    /// THE SERVER DID NOT STATE IS NOT COVERED. The description's second part is
    /// "at what sizes", a size nobody stated cannot be inside it, and 0036 names
    /// the direction this fails in as the cheap one: a conversion the device did
    /// not need, rather than a stream that fails inside a decoder.
    fn is_covered_by(&self, description: &Capabilities) -> bool {
        if !admits_name(description.codecs(), &self.codec) {
            return false;
        }
        match self.kind {
            StreamKind::Video => self
                .picture
                .is_some_and(|picture| picture.fits_within(description.largest_picture())),
            StreamKind::Audio | StreamKind::Subtitle | StreamKind::Other => true,
        }
    }

    fn speaks(&self, language: &str) -> bool {
        self.language
            .as_deref()
            .is_some_and(|spoken| spoken.eq_ignore_ascii_case(language))
    }
}

/// Which of the three routes the server said it will serve a source by.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Both supported lines set the three from the description the call carried,
/// which is why the ladder can read them at all: a flag computed from a stored
/// description would be a flag about a different device. The three are the
/// server's statement about what it will do; whether the description covers
/// what would be played is this module's, and a source is taken only where the
/// two agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutesOffered {
    as_it_stands: bool,
    container_changed: bool,
    converted: bool,
}

impl RoutesOffered {
    /// The three flags as the server stated them.
    #[must_use]
    pub const fn stated(as_it_stands: bool, container_changed: bool, converted: bool) -> Self {
        Self {
            as_it_stands,
            container_changed,
            converted,
        }
    }
}

/// The address the server supplied for a source it will change or convert, as
/// it arrived.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Both supported lines write the session's token into this address, read at
/// the commits 0010 names, so it is a value 0071 excludes from every event and
/// 0005 keeps out of every log. The hand-written formatting below is what keeps
/// it out of anything that formats a handover, and no accessor here goes
/// through 0071's treatment, so a caller that puts it into a report has moved
/// the leak rather than found one.
#[derive(Clone, PartialEq, Eq)]
pub struct ConversionAddress {
    as_sent: String,
}

/// Written out by hand so that the token the server put into the address is
/// never in a formatted line.
impl core::fmt::Debug for ConversionAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConversionAddress").finish_non_exhaustive()
    }
}

impl ConversionAddress {
    /// Takes the address as the server sent it.
    #[must_use]
    pub const fn as_sent(text: String) -> Self {
        Self { as_sent: text }
    }

    /// The address, for the player that opens it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.as_sent
    }
}

/// One source the server offered for an item.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// The server's order among several is the order they are handed to
/// [`WhatTheServerOffered::answered`] in, and 0111 makes that order the
/// tie-break, so nothing here sorts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferedSource {
    id: String,
    container: String,
    streams: Vec<OfferedStream>,
    routes: RoutesOffered,
    conversion_address: Option<ConversionAddress>,
    bitrate: Option<u64>,
    default_audio: Option<i32>,
    default_subtitle: Option<i32>,
}

impl OfferedSource {
    /// A source as the server named it, with the routes it offers and nothing
    /// else yet.
    #[must_use]
    pub fn described(id: &str, container: &str, routes: RoutesOffered) -> Self {
        Self {
            id: id.to_owned(),
            container: container.to_owned(),
            streams: Vec::new(),
            routes,
            conversion_address: None,
            bitrate: None,
            default_audio: None,
            default_subtitle: None,
        }
    }

    /// The streams the server listed, in the order it listed them.
    #[must_use]
    pub fn with_streams(mut self, streams: Vec<OfferedStream>) -> Self {
        self.streams = streams;
        self
    }

    /// The address the server supplied for changing or converting this source.
    #[must_use]
    pub fn with_conversion_address(mut self, address: ConversionAddress) -> Self {
        self.conversion_address = Some(address);
        self
    }

    /// How much the source carries, in bits per second, where the server states
    /// it.
    #[must_use]
    pub const fn with_bitrate(mut self, bits_per_second: u64) -> Self {
        self.bitrate = Some(bits_per_second);
        self
    }

    /// The audio and subtitle streams the server marks as this source's own
    /// defaults, by index.
    #[must_use]
    pub const fn with_defaults(mut self, audio: Option<i32>, subtitle: Option<i32>) -> Self {
        self.default_audio = audio;
        self.default_subtitle = subtitle;
        self
    }

    /// The identifier the server names this source by.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether the source carries more than a ceiling allows.
    ///
    /// A source whose rate the server did not state is not above any ceiling,
    /// because the core cannot say that it is, and 0111 has the core apply no
    /// ceiling of its own.
    fn exceeds(&self, ceiling: Option<Ceiling>) -> bool {
        match (self.bitrate, ceiling) {
            (Some(rate), Some(ceiling)) => rate > ceiling.bits_per_second,
            (None, _) | (_, None) => false,
        }
    }

    /// The streams that will actually be played, chosen before the source is
    /// judged.
    fn streams_to_play(&self, languages: &[&str]) -> StreamsToPlay<'_> {
        StreamsToPlay {
            video: self.streams.iter().find(|s| s.kind == StreamKind::Video),
            audio: self.chosen_of(StreamKind::Audio, languages, self.default_audio),
            subtitle: self.chosen_of(StreamKind::Subtitle, languages, self.default_subtitle),
        }
    }

    /// 0111's stream rule for one kind: the first stream matching the earliest
    /// language the caller named that any stream satisfies, else the one the
    /// source marks as its default, else the first listed.
    ///
    /// Two things can mark a default and both are read, the source's own index
    /// first. The index is the server's statement about this source and the
    /// flag on a stream is the file's about itself, and where the two disagree
    /// the server's is the one that reflects what it will serve.
    fn chosen_of(
        &self,
        kind: StreamKind,
        languages: &[&str],
        marked_by_index: Option<i32>,
    ) -> Option<&OfferedStream> {
        let of_kind = || self.streams.iter().filter(move |s| s.kind == kind);
        languages
            .iter()
            .find_map(|language| of_kind().find(|s| s.speaks(language)))
            .or_else(|| marked_by_index.and_then(|index| of_kind().find(|s| s.index == index)))
            .or_else(|| of_kind().find(|s| s.marked_default))
            .or_else(|| of_kind().next())
    }

    /// Which rung, if any, this source sits on for this description.
    ///
    /// The container and the streams that will be played are judged, and no
    /// other stream is, which is 0111's ordering: judging a source before its
    /// streams are chosen tests it against a track nobody will hear. A source
    /// that describes no stream at all is judged by nothing and clears nothing,
    /// because the alternative admits a file on the strength of its container
    /// alone.
    fn placement<'a>(
        &'a self,
        description: &Capabilities,
        to_play: &StreamsToPlay<'_>,
    ) -> Option<Placement<'a>> {
        let judged: Vec<&OfferedStream> = [to_play.video, to_play.audio, to_play.subtitle]
            .into_iter()
            .flatten()
            .collect();
        if judged.is_empty() {
            return None;
        }
        let not_covered = judged
            .iter()
            .filter(|stream| !stream.is_covered_by(description))
            .count();
        let container_covered = admits_name(description.containers(), &self.container);

        if self.routes.as_it_stands && container_covered && not_covered == 0 {
            return Some(Placement {
                rung: Rung::PlayedAsItStands,
                converted: 0,
                opens: Opens::AsItStands,
            });
        }
        // Every rung below the top one is a route the server carries out, and
        // it names that route by an address. No address, no rung: the placement
        // is built from the address, so a lower rung without one cannot be
        // written here at all.
        let address = self.conversion_address.as_ref()?;
        let opens = Opens::Conversion(address);
        if self.routes.container_changed && not_covered == 0 {
            return Some(Placement {
                rung: Rung::ContainerChanged,
                converted: 0,
                opens,
            });
        }
        if !self.routes.converted {
            return None;
        }
        if not_covered < judged.len() {
            return Some(Placement {
                rung: Rung::SomeStreamsConverted,
                converted: not_covered,
                opens,
            });
        }
        Some(Placement {
            rung: Rung::EveryStreamConverted,
            converted: not_covered,
            opens,
        })
    }
}

/// The streams of one source that will actually be played.
struct StreamsToPlay<'a> {
    video: Option<&'a OfferedStream>,
    audio: Option<&'a OfferedStream>,
    subtitle: Option<&'a OfferedStream>,
}

/// Where a source landed on the ladder, what that costs, and what the player
/// is handed there.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Placement<'a> {
    rung: Rung,
    /// How many of the streams that will be played are converted, which is
    /// what orders two sources on the conversion rung.
    converted: usize,
    opens: Opens<'a>,
}

/// What a placement hands the player, borrowed from the source until one
/// placement has won.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Opens<'a> {
    AsItStands,
    Conversion(&'a ConversionAddress),
}

impl Opens<'_> {
    fn owned(self) -> WhatThePlayerOpens {
        match self {
            Self::AsItStands => WhatThePlayerOpens::TheSourceAsItStands,
            Self::Conversion(address) => WhatThePlayerOpens::TheServersConversion(address.clone()),
        }
    }
}

impl Placement<'_> {
    /// Whether this placement beats another, strictly.
    ///
    /// Strictly, so that a tie leaves the earlier source in place, which is
    /// 0111's tie-break by the server's order arriving through the order the
    /// sources are walked in rather than through a comparison of its own.
    const fn is_higher_than(self, other: Self) -> bool {
        if self.rung.rank() != other.rung.rank() {
            return self.rung.rank() < other.rung.rank();
        }
        self.converted < other.converted
    }
}

/// The rung a source was taken from.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Highest first, and each rung down costs somebody something the rung above
/// does not: nothing at all, a repackage on the server, picture or sound
/// quality that cannot be recovered, and both of those on every stream. The
/// handover carries this so that a client wanting to tell a person the file is
/// being converted has a way to know that is not inferring it from the shape of
/// the address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rung {
    /// The bytes the server holds, sent unchanged.
    PlayedAsItStands,
    /// The same streams, repackaged.
    ContainerChanged,
    /// The streams the description does not admit are converted, and no
    /// others.
    SomeStreamsConverted,
    /// Every stream is converted.
    EveryStreamConverted,
}

impl Rung {
    /// Every rung, highest first, so a caller reads the ladder out of the crate
    /// rather than keeping a copy of it.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::PlayedAsItStands,
            Self::ContainerChanged,
            Self::SomeStreamsConverted,
            Self::EveryStreamConverted,
        ]
    }

    /// The rung as it is reported: data a client reads rather than the text a
    /// debug printing would produce, which 0100 requires of a field.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlayedAsItStands => "played-as-it-stands",
            Self::ContainerChanged => "container-changed",
            Self::SomeStreamsConverted => "some-streams-converted",
            Self::EveryStreamConverted => "every-stream-converted",
        }
    }

    /// Lower is higher on the ladder.
    const fn rank(self) -> u8 {
        match self {
            Self::PlayedAsItStands => 0,
            Self::ContainerChanged => 1,
            Self::SomeStreamsConverted => 2,
            Self::EveryStreamConverted => 3,
        }
    }
}

/// A ceiling the caller supplies on how much a source may carry.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// 0111 has the caller supply it and the core carry it, because a client holds
/// what the core does not: whether the device is on a metered connection, what
/// a person set, and what the platform says about the network. The core invents
/// none and applies none where none is supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ceiling {
    bits_per_second: u64,
}

impl Ceiling {
    /// A ceiling in bits per second, which is the unit the server states a
    /// source's rate in.
    #[must_use]
    pub const fn bits_per_second(bits_per_second: u64) -> Self {
        Self { bits_per_second }
    }

    /// The ceiling, in bits per second.
    #[must_use]
    pub const fn as_bits_per_second(self) -> u64 {
        self.bits_per_second
    }
}

/// What the caller asked for beside the item: the languages it prefers, in the
/// order it prefers them, and a ceiling.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Both belong to the caller for the reason 0111 gives: what a person wants to
/// hear and read is a setting a client holds in a screen the core does not
/// draw, and 0003 keeps the core out of both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences<'a> {
    languages: &'a [&'a str],
    ceiling: Option<Ceiling>,
}

impl<'a> Preferences<'a> {
    /// What the caller stated.
    #[must_use]
    pub const fn stated(languages: &'a [&'a str], ceiling: Option<Ceiling>) -> Self {
        Self { languages, ceiling }
    }

    /// A caller that stated nothing, which is legal and applies no ceiling.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            languages: &[],
            ceiling: None,
        }
    }
}

/// The server's answer to a play call, as values.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatTheServerOffered {
    sources: Vec<OfferedSource>,
    error_code: Option<String>,
}

impl WhatTheServerOffered {
    /// The sources in the order the server listed them, and the error code it
    /// supplied where it supplied one.
    #[must_use]
    pub fn answered(sources: Vec<OfferedSource>, error_code: Option<&str>) -> Self {
        Self {
            sources,
            error_code: error_code.map(str::to_owned),
        }
    }

    /// The sources, in the server's order.
    #[must_use]
    pub fn sources(&self) -> &[OfferedSource] {
        &self.sources
    }
}

/// A stream the rule chose, as the handover names it.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChosenStream {
    index: i32,
    codec: String,
    language: Option<String>,
}

impl ChosenStream {
    fn of(stream: &OfferedStream) -> Self {
        Self {
            index: stream.index,
            codec: stream.codec.clone(),
            language: stream.language.clone(),
        }
    }

    /// The index the server names the stream by.
    #[must_use]
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// The codec, as the server names it.
    #[must_use]
    pub fn codec(&self) -> &str {
        &self.codec
    }

    /// The language, where the server states one.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

/// What the platform's own player is handed.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatThePlayerOpens {
    /// The source itself, unchanged. THE ADDRESS IS NOT HERE: the server
    /// supplies none for this rung, the path is one the core would build, and
    /// 0010's table carries no row for it, so it arrives with a record that
    /// supersedes 0010 rather than with this variant.
    TheSourceAsItStands,
    /// The address the server supplied for changing or converting the source,
    /// as it arrived.
    TheServersConversion(ConversionAddress),
}

/// What a play call hands back, and nothing else.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Not a player, not a decoder, not a frame, not a duration the client should
/// count against, and nothing beyond the point 0112 places outside. The core
/// opens no player and holds no opinion about what the client does with what
/// it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handover {
    source: String,
    rung: Rung,
    opens: WhatThePlayerOpens,
    starts_at: Ticks,
    audio: Option<ChosenStream>,
    subtitle: Option<ChosenStream>,
}

impl Handover {
    /// 0111's rule applied to what the server offered.
    ///
    /// A source above the caller's ceiling is not a candidate. Each remaining
    /// source has its streams chosen, is judged on those and its container, and
    /// lands on a rung or on none. The highest rung wins; on the conversion
    /// rung the fewest converted streams wins; a tie goes to the source the
    /// server listed first. The start is 0058's answer applied and not
    /// re-decided: a position it offers is where playback starts, and an item
    /// it calls finished or keeps no position for starts from the beginning.
    ///
    /// # Errors
    ///
    /// [`NothingPlayable::TheServerRefused`] where the server supplied an error
    /// code or listed no source, carrying the code where there was one.
    /// [`NothingPlayable::TheCoreRefused`] where the server offered sources and
    /// none of them clears a rung it has a route for. Both map onto
    /// `request-refused` through [`NothingPlayable::as_failure`], which is
    /// 0111's decision, and the two are told apart here so that the event under
    /// 0100 can say which it was.
    pub fn chosen(
        offered: &WhatTheServerOffered,
        description: &Capabilities,
        preferences: Preferences<'_>,
        start: Resume,
    ) -> Result<Self, NothingPlayable> {
        if let Some(code) = &offered.error_code {
            return Err(NothingPlayable::TheServerRefused {
                code: Some(code.clone()),
            });
        }
        if offered.sources.is_empty() {
            return Err(NothingPlayable::TheServerRefused { code: None });
        }

        let mut best: Option<(Placement<'_>, &OfferedSource, StreamsToPlay<'_>)> = None;
        for source in &offered.sources {
            if source.exceeds(preferences.ceiling) {
                continue;
            }
            let to_play = source.streams_to_play(preferences.languages);
            let Some(placement) = source.placement(description, &to_play) else {
                continue;
            };
            let beats_the_best = best
                .as_ref()
                .is_none_or(|(held, _, _)| placement.is_higher_than(*held));
            if beats_the_best {
                best = Some((placement, source, to_play));
            }
        }

        let Some((placement, source, to_play)) = best else {
            return Err(NothingPlayable::TheCoreRefused);
        };
        Ok(Self {
            source: source.id.clone(),
            rung: placement.rung,
            opens: placement.opens.owned(),
            starts_at: match start {
                Resume::At(position) => position,
                Resume::ItemIsFinished | Resume::NoPositionIsKept => Ticks::ZERO,
            },
            audio: to_play.audio.map(ChosenStream::of),
            subtitle: to_play.subtitle.map(ChosenStream::of),
        })
    }

    /// The source that was taken, as the server names it.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The rung it was taken from.
    #[must_use]
    pub const fn rung(&self) -> Rung {
        self.rung
    }

    /// What the platform's own player is handed.
    #[must_use]
    pub const fn opens(&self) -> &WhatThePlayerOpens {
        &self.opens
    }

    /// Where playback starts, which is 0058's answer applied.
    #[must_use]
    pub const fn starts_at(&self) -> Ticks {
        self.starts_at
    }

    /// The audio stream that was chosen, where the source has one.
    #[must_use]
    pub const fn audio(&self) -> Option<&ChosenStream> {
        self.audio.as_ref()
    }

    /// The subtitle stream that was chosen, where the source has one. Chosen
    /// and not turned on: whether to show it at the start is a client setting
    /// the core has no view on.
    #[must_use]
    pub const fn subtitle(&self) -> Option<&ChosenStream> {
        self.subtitle.as_ref()
    }
}

/// Why nothing was playable.
///
/// Thread safety, from 0009: a plain value, safe from any thread.
///
/// Two conditions arrive here and 0111 maps both onto one kind, because both
/// are refusals of the same request: one the server made, and one the core made
/// on the description the client supplied. The two are kept apart in this value
/// for the event under 0100, which is where somebody diagnosing this looks, and
/// they are one kind by the time a caller sees them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NothingPlayable {
    /// The server answered that it will serve nothing, with the code it gave
    /// where it gave one.
    TheServerRefused {
        /// The server's own error code, opaque, as 0004 carries it.
        code: Option<String>,
    },
    /// The server offered sources and none of them clears a rung it has a
    /// route for, judged on the description the client supplied.
    TheCoreRefused,
}

impl NothingPlayable {
    /// The server's code, where the server gave one.
    #[must_use]
    pub fn server_code(&self) -> Option<&str> {
        match self {
            Self::TheServerRefused { code } => code.as_deref(),
            Self::TheCoreRefused => None,
        }
    }

    /// The one kind both conditions become, built at 0037's mapping point.
    #[must_use]
    pub fn as_failure(&self) -> Failure {
        Failure::nothing_playable(self.server_code())
    }
}

/// Whether a list of names the client supplied admits a name the server
/// stated.
///
/// Compared without regard to letter case, because both are the same platform
/// fact spelled by two parties, and a client writing a container in capitals
/// would otherwise have every file converted with nothing saying why.
fn admits_name(admitted: &[String], name: &str) -> bool {
    admitted
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::{
        Ceiling, ChosenStream, ConversionAddress, Handover, NothingPlayable, OfferedSource,
        OfferedStream, Picture, Preferences, RoutesOffered, Rung, StreamKind, WhatThePlayerOpens,
        WhatTheServerOffered,
    };
    use crate::failure::{Failure, Kind};
    use crate::playback::Ticks;
    use crate::playback::resume::Resume;
    use crate::session::device::{Capabilities, LargestPicture};

    /// A device that opens two containers, decodes three codecs, and shows up
    /// to a full high-definition picture.
    fn description() -> Capabilities {
        Capabilities::supplied(
            &["mkv", "mp4"],
            &["h264", "aac", "subrip"],
            LargestPicture::supplied(1920, 1080).expect("a size"),
        )
        .expect("a description")
    }

    const EVERY_ROUTE: RoutesOffered = RoutesOffered::stated(true, true, true);
    const ONLY_AS_IT_STANDS: RoutesOffered = RoutesOffered::stated(true, false, false);
    const CHANGED_OR_CONVERTED: RoutesOffered = RoutesOffered::stated(false, true, true);
    const ONLY_CONVERTED: RoutesOffered = RoutesOffered::stated(false, false, true);

    const HD: Picture = Picture::stated(1920, 1080);
    const START: Resume = Resume::At(Ticks::from_seconds(600));

    fn video(codec: &str, picture: Option<Picture>) -> OfferedStream {
        OfferedStream::described(0, StreamKind::Video, codec, None, false, picture)
    }

    fn audio(index: i32, codec: &str, language: &str, marked_default: bool) -> OfferedStream {
        OfferedStream::described(
            index,
            StreamKind::Audio,
            codec,
            Some(language),
            marked_default,
            None,
        )
    }

    fn subtitle(index: i32, codec: &str, language: &str) -> OfferedStream {
        OfferedStream::described(
            index,
            StreamKind::Subtitle,
            codec,
            Some(language),
            false,
            None,
        )
    }

    fn address(id: &str) -> ConversionAddress {
        ConversionAddress::as_sent(format!("/videos/{id}/master.m3u8?api_key=the-token"))
    }

    /// A source with every route and the server's address for the lower ones.
    fn source(id: &str, container: &str, streams: Vec<OfferedStream>) -> OfferedSource {
        OfferedSource::described(id, container, EVERY_ROUTE)
            .with_streams(streams)
            .with_conversion_address(address(id))
    }

    fn a_playable_file(id: &str) -> OfferedSource {
        source(
            id,
            "mkv",
            vec![video("h264", Some(HD)), audio(1, "aac", "eng", true)],
        )
    }

    fn chosen(offered: &WhatTheServerOffered) -> Result<Handover, NothingPlayable> {
        Handover::chosen(offered, &description(), Preferences::none(), START)
    }

    #[test]
    fn a_source_the_description_covers_is_played_as_it_stands() {
        let offered = WhatTheServerOffered::answered(vec![a_playable_file("one")], None);
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.source(), "one");
        assert_eq!(handover.rung(), Rung::PlayedAsItStands);
        assert_eq!(handover.opens(), &WhatThePlayerOpens::TheSourceAsItStands);
        assert_eq!(handover.starts_at(), Ticks::from_seconds(600));
        assert_eq!(handover.audio().map(ChosenStream::index), Some(1));
        assert_eq!(handover.subtitle(), None);
    }

    /// The failure 0111 was written against: a line taking the first element of
    /// a list. The first source needs a conversion and the second does not.
    #[test]
    fn the_first_source_listed_is_not_taken_when_a_later_one_sits_higher() {
        let needs_converting = source(
            "first",
            "mkv",
            vec![video("hevc", Some(HD)), audio(1, "aac", "eng", true)],
        );
        let offered =
            WhatTheServerOffered::answered(vec![needs_converting, a_playable_file("second")], None);
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.source(), "second");
        assert_eq!(handover.rung(), Rung::PlayedAsItStands);
    }

    #[test]
    fn a_tie_on_one_rung_goes_to_the_source_the_server_listed_first() {
        let offered = WhatTheServerOffered::answered(
            vec![a_playable_file("first"), a_playable_file("second")],
            None,
        );
        assert_eq!(chosen(&offered).expect("playable").source(), "first");

        let both_converted = WhatTheServerOffered::answered(
            vec![
                source("first", "mkv", vec![video("hevc", Some(HD))]),
                source("second", "mkv", vec![video("hevc", Some(HD))]),
            ],
            None,
        );
        let handover = chosen(&both_converted).expect("playable");
        assert_eq!(handover.source(), "first");
        assert_eq!(handover.rung(), Rung::EveryStreamConverted);
    }

    #[test]
    fn a_container_the_device_cannot_open_changes_the_container_and_converts_nothing() {
        let offered = WhatTheServerOffered::answered(
            vec![
                OfferedSource::described("one", "avi", CHANGED_OR_CONVERTED)
                    .with_streams(vec![video("h264", Some(HD)), audio(1, "aac", "eng", true)])
                    .with_conversion_address(address("one")),
            ],
            None,
        );
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.rung(), Rung::ContainerChanged);
        assert_eq!(
            handover.opens(),
            &WhatThePlayerOpens::TheServersConversion(address("one"))
        );
    }

    /// 0111's ordering: the streams are chosen first and only those are judged,
    /// so a second audio track nobody will hear does not cost a conversion.
    #[test]
    fn a_stream_nobody_will_play_is_not_judged() {
        let offered = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "mkv",
                vec![
                    video("h264", Some(HD)),
                    audio(1, "aac", "eng", true),
                    audio(2, "dts", "fra", false),
                ],
            )],
            None,
        );
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.rung(), Rung::PlayedAsItStands);
        assert_eq!(handover.audio().map(ChosenStream::codec), Some("aac"));
    }

    #[test]
    fn a_chosen_stream_the_description_does_not_admit_is_converted_and_no_other() {
        let offered = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "mkv",
                vec![video("h264", Some(HD)), audio(1, "dts", "eng", true)],
            )],
            None,
        );
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.rung(), Rung::SomeStreamsConverted);
        assert_eq!(
            handover.opens(),
            &WhatThePlayerOpens::TheServersConversion(address("one"))
        );
    }

    #[test]
    fn fewer_converted_streams_wins_between_two_sources_on_the_conversion_rung() {
        let two_converted = source(
            "first",
            "mkv",
            vec![
                video("hevc", Some(HD)),
                audio(1, "dts", "eng", true),
                subtitle(2, "subrip", "eng"),
            ],
        );
        let one_converted = source(
            "second",
            "mkv",
            vec![
                video("h264", Some(HD)),
                audio(1, "dts", "eng", true),
                subtitle(2, "subrip", "eng"),
            ],
        );
        let offered = WhatTheServerOffered::answered(vec![two_converted, one_converted], None);
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.source(), "second");
        assert_eq!(handover.rung(), Rung::SomeStreamsConverted);
    }

    #[test]
    fn a_source_admitting_no_chosen_stream_converts_every_stream() {
        let offered = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "mkv",
                vec![video("hevc", Some(HD)), audio(1, "dts", "eng", true)],
            )],
            None,
        );
        assert_eq!(
            chosen(&offered).expect("playable").rung(),
            Rung::EveryStreamConverted
        );
    }

    #[test]
    fn a_picture_larger_than_the_largest_or_of_unstated_size_is_not_played_as_it_stands() {
        let too_large = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "mkv",
                vec![video("h264", Some(Picture::stated(3840, 2160)))],
            )],
            None,
        );
        assert_eq!(
            chosen(&too_large).expect("playable").rung(),
            Rung::EveryStreamConverted
        );

        let unstated = WhatTheServerOffered::answered(
            vec![source("one", "mkv", vec![video("h264", None)])],
            None,
        );
        assert_eq!(
            chosen(&unstated).expect("playable").rung(),
            Rung::EveryStreamConverted
        );

        let exactly_the_largest = WhatTheServerOffered::answered(
            vec![source("one", "mkv", vec![video("h264", Some(HD))])],
            None,
        );
        assert_eq!(
            chosen(&exactly_the_largest).expect("playable").rung(),
            Rung::PlayedAsItStands
        );
    }

    #[test]
    fn languages_are_read_in_order_then_the_default_then_the_first_listed() {
        let streams = || {
            vec![
                video("h264", Some(HD)),
                audio(1, "aac", "eng", false),
                audio(2, "aac", "deu", true),
                audio(3, "aac", "fra", false),
            ]
        };
        let offered = WhatTheServerOffered::answered(vec![source("one", "mkv", streams())], None);

        let earliest_satisfiable = Preferences::stated(&["jpn", "FRA", "eng"], None);
        let handover = Handover::chosen(&offered, &description(), earliest_satisfiable, START)
            .expect("playable");
        assert_eq!(handover.audio().map(ChosenStream::index), Some(3));

        let nothing_matches = Preferences::stated(&["jpn"], None);
        let handover =
            Handover::chosen(&offered, &description(), nothing_matches, START).expect("playable");
        assert_eq!(handover.audio().map(ChosenStream::index), Some(2));

        let by_the_sources_index = WhatTheServerOffered::answered(
            vec![source("one", "mkv", streams()).with_defaults(Some(3), None)],
            None,
        );
        let handover = chosen(&by_the_sources_index).expect("playable");
        assert_eq!(handover.audio().map(ChosenStream::index), Some(3));

        let no_default_at_all = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "mkv",
                vec![
                    video("h264", Some(HD)),
                    audio(1, "aac", "eng", false),
                    audio(2, "aac", "deu", false),
                ],
            )],
            None,
        );
        let handover = chosen(&no_default_at_all).expect("playable");
        assert_eq!(handover.audio().map(ChosenStream::index), Some(1));
    }

    #[test]
    fn a_server_that_offers_nothing_is_its_refusal_and_its_code_rides_the_kind() {
        let with_a_code = WhatTheServerOffered::answered(vec![], Some("NoCompatibleStream"));
        let refused = chosen(&with_a_code).expect_err("nothing playable");
        assert_eq!(
            refused,
            NothingPlayable::TheServerRefused {
                code: Some("NoCompatibleStream".to_owned())
            }
        );
        let failure = refused.as_failure();
        assert_eq!(failure.kind(), Kind::RequestRefused);
        let Failure::RequestRefused { server_code, .. } = failure else {
            panic!("nothing playable mapped onto something else");
        };
        assert_eq!(server_code.as_deref(), Some("NoCompatibleStream"));

        let with_no_code = WhatTheServerOffered::answered(vec![], None);
        let refused = chosen(&with_no_code).expect_err("nothing playable");
        assert_eq!(refused, NothingPlayable::TheServerRefused { code: None });
        assert_eq!(refused.as_failure().kind(), Kind::RequestRefused);
    }

    #[test]
    fn a_ladder_nothing_clears_is_the_cores_refusal_on_the_same_kind() {
        let no_route_at_all = WhatTheServerOffered::answered(
            vec![
                OfferedSource::described("one", "mkv", RoutesOffered::stated(false, false, false))
                    .with_streams(vec![video("h264", Some(HD))]),
            ],
            None,
        );
        let refused = chosen(&no_route_at_all).expect_err("nothing playable");
        assert_eq!(refused, NothingPlayable::TheCoreRefused);
        assert_eq!(refused.server_code(), None);
        assert_eq!(refused.as_failure().kind(), Kind::RequestRefused);
    }

    /// The mismatch 0111 names: the server says the source plays as it stands,
    /// the description does not cover it, and the server supplied no address
    /// for any other route. The answer is nothing playable from the core rather
    /// than a stream that fails inside a decoder.
    #[test]
    fn a_source_the_server_marked_playable_that_the_description_does_not_cover_has_no_route() {
        let offered = WhatTheServerOffered::answered(
            vec![
                OfferedSource::described("one", "mkv", ONLY_AS_IT_STANDS)
                    .with_streams(vec![video("hevc", Some(HD))]),
            ],
            None,
        );
        assert_eq!(
            chosen(&offered).expect_err("nothing playable"),
            NothingPlayable::TheCoreRefused
        );
    }

    #[test]
    fn a_route_below_the_top_needs_the_address_the_server_supplies_for_it() {
        let converted_with_no_address = WhatTheServerOffered::answered(
            vec![
                OfferedSource::described("one", "mkv", ONLY_CONVERTED)
                    .with_streams(vec![video("hevc", Some(HD))]),
            ],
            None,
        );
        assert_eq!(
            chosen(&converted_with_no_address).expect_err("nothing playable"),
            NothingPlayable::TheCoreRefused
        );
    }

    #[test]
    fn a_source_above_the_ceiling_is_not_a_candidate_and_one_of_unstated_rate_is() {
        let offered = WhatTheServerOffered::answered(
            vec![
                a_playable_file("heavy").with_bitrate(40_000_000),
                a_playable_file("light").with_bitrate(4_000_000),
                a_playable_file("unstated"),
            ],
            None,
        );
        let ceiling = Preferences::stated(&[], Some(Ceiling::bits_per_second(8_000_000)));
        let handover =
            Handover::chosen(&offered, &description(), ceiling, START).expect("playable");
        assert_eq!(handover.source(), "light");

        let only_heavy = WhatTheServerOffered::answered(
            vec![a_playable_file("heavy").with_bitrate(40_000_000)],
            None,
        );
        assert_eq!(
            Handover::chosen(&only_heavy, &description(), ceiling, START).expect_err("above it"),
            NothingPlayable::TheCoreRefused
        );

        let no_ceiling = chosen(&only_heavy).expect("playable");
        assert_eq!(no_ceiling.source(), "heavy");
        assert_eq!(Ceiling::bits_per_second(1).as_bits_per_second(), 1);
    }

    #[test]
    fn the_start_is_0058s_answer_applied_and_not_re_decided() {
        let offered = WhatTheServerOffered::answered(vec![a_playable_file("one")], None);
        for (start, expected) in [
            (Resume::At(Ticks::from_seconds(90)), Ticks::from_seconds(90)),
            (Resume::ItemIsFinished, Ticks::ZERO),
            (Resume::NoPositionIsKept, Ticks::ZERO),
        ] {
            let handover = Handover::chosen(&offered, &description(), Preferences::none(), start)
                .expect("playable");
            assert_eq!(handover.starts_at(), expected, "for {start:?}");
        }
    }

    #[test]
    fn the_conversion_address_is_written_out_nowhere() {
        let offered = WhatTheServerOffered::answered(
            vec![source("one", "mkv", vec![video("hevc", Some(HD))])],
            None,
        );
        let handover = chosen(&offered).expect("playable");
        let formatted = format!("{handover:?}");
        assert!(!formatted.contains("api_key"), "{formatted}");
        assert!(!formatted.contains("the-token"), "{formatted}");
        let WhatThePlayerOpens::TheServersConversion(address) = handover.opens() else {
            panic!("a converted source opens the server's address");
        };
        assert!(address.as_str().contains("the-token"));
    }

    #[test]
    fn a_source_describing_no_stream_clears_no_rung() {
        let offered = WhatTheServerOffered::answered(vec![source("one", "mkv", vec![])], None);
        assert_eq!(
            chosen(&offered).expect_err("nothing to judge"),
            NothingPlayable::TheCoreRefused
        );
    }

    #[test]
    fn a_subtitle_is_chosen_by_the_same_rule_and_counts_when_judged() {
        let offered = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "mkv",
                vec![
                    video("h264", Some(HD)),
                    audio(1, "aac", "eng", true),
                    subtitle(2, "pgs", "eng"),
                    subtitle(3, "subrip", "deu"),
                ],
            )],
            None,
        );
        let handover = chosen(&offered).expect("playable");
        assert_eq!(handover.subtitle().map(ChosenStream::index), Some(2));
        assert_eq!(handover.rung(), Rung::SomeStreamsConverted);

        let german = Preferences::stated(&["deu"], None);
        let handover = Handover::chosen(&offered, &description(), german, START).expect("playable");
        assert_eq!(handover.subtitle().map(ChosenStream::index), Some(3));
        assert_eq!(
            handover.subtitle().map(ChosenStream::language),
            Some(Some("deu"))
        );
        assert_eq!(handover.rung(), Rung::PlayedAsItStands);
    }

    #[test]
    fn names_are_compared_without_regard_to_case() {
        let offered = WhatTheServerOffered::answered(
            vec![source(
                "one",
                "MKV",
                vec![video("H264", Some(HD)), audio(1, "AAC", "ENG", true)],
            )],
            None,
        );
        assert_eq!(
            chosen(&offered).expect("playable").rung(),
            Rung::PlayedAsItStands
        );
    }

    #[test]
    fn the_ladder_is_read_out_of_the_crate_highest_first() {
        let names: Vec<&str> = Rung::all().iter().map(|rung| rung.as_str()).collect();
        assert_eq!(
            names,
            [
                "played-as-it-stands",
                "container-changed",
                "some-streams-converted",
                "every-stream-converted"
            ]
        );
        let one = WhatTheServerOffered::answered(vec![a_playable_file("one")], None);
        assert_eq!(one.sources().len(), 1);
        assert_eq!(one.sources()[0].id(), "one");
        let stream = video("h264", Some(HD));
        assert_eq!(stream.index(), 0);
        assert_eq!(stream.kind(), StreamKind::Video);
        assert_eq!(stream.codec(), "h264");
        assert_eq!(stream.language(), None);
        assert_eq!(stream.picture(), Some(HD));
    }
}
