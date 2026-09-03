//! Every "safe from any thread" statement 0009 makes, checked from outside the
//! crate.
//!
//! These are compile-time checks in the body of a test rather than assertions a
//! run evaluates, and that is worth stating plainly: a failure here is a build
//! that does not compile, not a red line in the test output. What the run adds is
//! a named result per kind, so that a reader of the output can see which of
//! 0009's statements are covered rather than counting them in a file.
//!
//! Checking from a test crate rather than inside the library is deliberate. It
//! asks the same question a client asks: the type is reachable from outside, and
//! it carries the bound out there too.
//!
//! What this file does not check. 0009 makes statements no bound can carry: that
//! the secret store is called from one lane only and never concurrently for one
//! session, that a sink must not block, and that a sink must not call back into
//! the core. Those are properties of the caller rather than of the type, and the
//! detector in #117 is where they are meant to be observed. Nothing here covers
//! them.

use flowfin_core::Core;
use flowfin_core::artwork::DecodedImage;
use flowfin_core::artwork::address::{
    ArtworkRequest, DrawnSize, Edge, ImageKind, ImageTag, ItemId, NotUsableInARequest,
    SizeNotUsable,
};
use flowfin_core::artwork::announced::{
    AnnouncedWindow, SharedFetches, WhatTheAnnouncementDid, WhatTheHoldDid, WhatTheWithdrawalDid,
};
use flowfin_core::artwork::budget::{
    Budget, BudgetNotUsable, DecodedBytes, DecodedBytesHeld, WhatTheAskDoes,
};
use flowfin_core::artwork::format::{Accepted, Admitted, DeclaredDimensions, Refused};
use flowfin_core::artwork::presence::WhatTheItemHas;
use flowfin_core::artwork::shape::{
    AspectRatio, RatioNotUsable, ReservedRectangle, WhatShapeIsKnown,
};
use flowfin_core::cache::bound::{CacheBounds, Tier, TieredCache};
use flowfin_core::cache::cold_start::{
    CallAtStart, HowTheSecretReadWent, WhatAStartServes, WhetherItCanBeAnsweredYet,
};
use flowfin_core::cache::envelope::{Drops, Entries, WhichCheckFailed};
use flowfin_core::cache::freshness::{
    Age, Answer, EntryKind, Held, Skew, WhyTheAgeIsUnreadable, WrittenAt,
};
use flowfin_core::cache::notification::{
    CachedEntry, ListenerState, Notification, WhatAListenerDoesToAThreshold,
    WhatTheNotificationDoes,
};
use flowfin_core::cache::{ByteStore, EntryKey};
use flowfin_core::clock::Clocks;
use flowfin_core::diagnostics::redaction::{Correlator, CorrelatorSalt, FieldName, Treatment};
use flowfin_core::diagnostics::{Diagnostics, DiagnosticsSink};
use flowfin_core::failure::{
    Answered, Capability, Failure, FaultSite, Kind, ReadingSite, TransportOutcome,
};
use flowfin_core::lifecycle::{
    HowTheStopEnded, Lane, Lifetime, StopBound, Supplied, WhatACallDoes, WhatIsPresent,
};
use flowfin_core::measurement::{Measurement, MeasurementSink};
use flowfin_core::playback::cadence::{
    ReportsWithoutWaiting, TheInterval, WhatItDoesToTheInterval,
};
use flowfin_core::playback::handover::{
    Ceiling, ChosenStream, ConversionAddress, Handover, NothingPlayable, OfferedSource,
    OfferedStream, Picture, Preferences, RoutesOffered, Rung, StreamKind, WhatThePlayerOpens,
    WhatTheServerOffered,
};
use flowfin_core::playback::report::{PositionReport, ReportedOn, Reporting, WhatObservingDid};
use flowfin_core::server::address::{AddressNotUsable, BaseAddress};
use flowfin_core::server::destinations::{
    AdmittedOrigin, Destinations, WhatARedirectDoes, WhatConfiguringDid,
};
use flowfin_core::server::federation::Federation;
use flowfin_core::server::library::{
    LibraryRead, NotAPagedRead, Page, PageRequest, WhatAskingForAPageDid, WhatTheReadAnswers,
};
use flowfin_core::server::retry::{
    Attempts, Jitter, TheWait, WhatAFailureDoes, WhatTheCallDoesNext, WhatTheRequestDoes,
    WhyTheCallStopped,
};
use flowfin_core::server::states::{
    AgingRequest, ConsecutiveAbandonments, State, WhatAnOutcomeSaysAboutTheServer, WhatToReport,
};
use flowfin_core::server::write_queue::{
    Dropped, Entry, Target, WhatIsAsserted, WhatTheEnqueueDid, WriteQueue,
};
use flowfin_core::session::delegated::{
    NoAttemptMatched, OpenAttempts, Relayable, TieValue, ValueAlreadyOpen, ValueNotUsable,
};
use flowfin_core::session::device::{Capabilities, DeviceIdentity, PartNotUsable};
use flowfin_core::session::mid_playback::{WhatARejectedReportDoes, WhatTheOutcomeDoesToPlayback};
use flowfin_core::session::password::{
    AccountName, AnswerRead, FactNotCarried, FactsASessionNeeds, NoSession, Password,
};
use flowfin_core::session::quick_connect::{HowTheCallEnded, IssuedExchange, WhileWaiting};
use flowfin_core::session::renewal::{
    Generation, HowTheRenewalEnded, Rejection, RenewalRoute, RenewalSchedule, Renewals,
    WhatARejectedCallDoes, WhatTheOutcomeDoes,
};
use flowfin_core::session::sign_out::{
    Act, HowItEnds, LocalHalf, Removal, TellingTheServer, WhatIsTakenAway, WhatTheClientIsTold,
    WhySigningOut, WorkInFlight,
};
use flowfin_core::session::{SecretStore, Session};

/// Compiles only for a type that is safe to use from any thread.
///
/// `?Sized` is what lets the same function ask the question of a trait object,
/// which is how the bound on a client-supplied trait is checked rather than the
/// bound on one particular implementor of it.
const fn any_thread<T: Send + Sync + ?Sized>() {}

#[test]
fn the_core_handle_is_safe_from_any_thread() {
    const _: () = any_thread::<Core>();
}

#[test]
fn a_session_handle_is_safe_from_any_thread() {
    const _: () = any_thread::<Session>();
}

#[test]
fn the_two_acts_that_end_a_session_are_safe_from_any_thread() {
    const _: () = any_thread::<Act>();
    const _: () = any_thread::<WhatIsTakenAway>();
    const _: () = any_thread::<WorkInFlight>();
    const _: () = any_thread::<HowItEnds>();
}

#[test]
fn the_halves_of_a_sign_out_are_safe_from_any_thread() {
    const _: () = any_thread::<WhySigningOut>();
    const _: () = any_thread::<LocalHalf>();
    const _: () = any_thread::<TellingTheServer>();
    const _: () = any_thread::<WhatTheClientIsTold>();
    const _: () = any_thread::<Removal>();
}

#[test]
fn a_device_identity_is_safe_from_any_thread() {
    const _: () = any_thread::<DeviceIdentity>();
}

#[test]
fn a_capability_description_is_safe_from_any_thread() {
    const _: () = any_thread::<Capabilities>();
}

#[test]
fn a_refused_part_of_an_identity_is_safe_from_any_thread() {
    const _: () = any_thread::<PartNotUsable>();
}

#[test]
fn a_password_is_safe_from_any_thread() {
    const _: () = any_thread::<Password>();
}

#[test]
fn an_account_name_is_safe_from_any_thread() {
    const _: () = any_thread::<AccountName>();
}

#[test]
fn what_was_read_out_of_a_sign_in_answer_is_safe_from_any_thread() {
    const _: () = any_thread::<AnswerRead<'static>>();
}

#[test]
fn the_facts_a_session_needs_are_safe_from_any_thread() {
    const _: () = any_thread::<FactsASessionNeeds<'static>>();
}

#[test]
fn an_answer_that_yielded_no_session_is_safe_from_any_thread() {
    const _: () = any_thread::<NoSession>();
    const _: () = any_thread::<FactNotCarried>();
}

#[test]
fn a_page_from_a_library_read_is_safe_from_any_thread() {
    const _: () = any_thread::<Page<()>>();
}

#[test]
fn a_page_request_is_safe_from_any_thread() {
    const _: () = any_thread::<PageRequest>();
}

#[test]
fn what_a_library_read_answers_with_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheReadAnswers>();
}

#[test]
fn a_library_read_is_safe_from_any_thread() {
    const _: () = any_thread::<LibraryRead>();
}

#[test]
fn why_a_read_took_no_page_request_is_safe_from_any_thread() {
    const _: () = any_thread::<NotAPagedRead>();
}

#[test]
fn what_asking_a_read_for_a_page_did_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatAskingForAPageDid>();
}

#[test]
fn a_base_address_is_safe_from_any_thread() {
    const _: () = any_thread::<BaseAddress>();
}

#[test]
fn an_unusable_address_is_safe_from_any_thread() {
    const _: () = any_thread::<AddressNotUsable>();
}

#[test]
fn the_four_states_a_request_ages_through_are_safe_from_any_thread() {
    const _: () = any_thread::<State>();
    const _: () = any_thread::<WhatToReport>();
    const _: () = any_thread::<AgingRequest>();
    const _: () = any_thread::<WhatAnOutcomeSaysAboutTheServer>();
    const _: () = any_thread::<ConsecutiveAbandonments>();
}

#[test]
fn the_core_lifetime_and_what_creation_takes_are_safe_from_any_thread() {
    const _: () = any_thread::<Lane>();
    const _: () = any_thread::<HowTheStopEnded>();
    const _: () = any_thread::<StopBound>();
    const _: () = any_thread::<WhatIsPresent>();
    const _: () = any_thread::<Supplied<'static>>();
    const _: () = any_thread::<WhatACallDoes>();
    const _: () = any_thread::<Lifetime>();
}

#[test]
fn the_federation_register_is_safe_from_any_thread() {
    const _: () = any_thread::<Federation<'static>>();
}

#[test]
fn the_set_of_destinations_and_what_it_answers_are_safe_from_any_thread() {
    const _: () = any_thread::<Destinations>();
    const _: () = any_thread::<AdmittedOrigin>();
    const _: () = any_thread::<WhatConfiguringDid>();
    const _: () = any_thread::<WhatARedirectDoes>();
}

#[test]
fn the_diagnostics_facility_is_safe_from_any_thread() {
    const _: () = any_thread::<Diagnostics<'static>>();
}

#[test]
fn a_decoded_image_is_safe_from_any_thread() {
    const _: () = any_thread::<DecodedImage>();
}

#[test]
fn the_byte_store_a_client_supplies_is_safe_from_any_thread() {
    const _: () = any_thread::<dyn ByteStore>();
}

#[test]
fn what_a_start_serves_before_a_session_is_restored_is_safe_from_any_thread() {
    const _: () = any_thread::<HowTheSecretReadWent>();
    const _: () = any_thread::<WhatAStartServes>();
    const _: () = any_thread::<CallAtStart>();
    const _: () = any_thread::<WhetherItCanBeAnsweredYet>();
}

#[test]
fn a_change_the_server_reported_is_safe_from_any_thread() {
    const _: () = any_thread::<Notification<'static>>();
    const _: () = any_thread::<CachedEntry<'static>>();
    const _: () = any_thread::<WhatTheNotificationDoes>();
}

#[test]
fn a_listener_state_and_what_it_does_to_a_threshold_are_safe_from_any_thread() {
    const _: () = any_thread::<ListenerState>();
    const _: () = any_thread::<WhatAListenerDoesToAThreshold>();
}

#[test]
fn a_cache_entry_key_is_safe_from_any_thread() {
    const _: () = any_thread::<EntryKey>();
}

#[test]
fn the_secret_store_a_client_supplies_is_safe_from_any_thread() {
    const _: () = any_thread::<dyn SecretStore>();
}

#[test]
fn the_diagnostics_sink_a_client_supplies_is_safe_from_any_thread() {
    const _: () = any_thread::<dyn DiagnosticsSink>();
}

#[test]
fn the_clock_source_a_client_supplies_is_safe_from_any_thread() {
    const _: () = any_thread::<dyn Clocks>();
}

#[test]
fn the_measurement_sink_a_client_supplies_is_safe_from_any_thread() {
    const _: () = any_thread::<dyn MeasurementSink>();
}

#[test]
fn the_measurement_facility_is_safe_from_any_thread() {
    const _: () = any_thread::<Measurement<'static>>();
}

#[test]
fn the_cache_bookkeeping_is_safe_from_any_thread() {
    const _: () = any_thread::<TieredCache<'static>>();
}

#[test]
fn the_cache_bounds_are_safe_from_any_thread() {
    const _: () = any_thread::<CacheBounds>();
}

#[test]
fn a_cache_tier_is_safe_from_any_thread() {
    const _: () = any_thread::<Tier>();
}

#[test]
fn an_accepted_image_format_is_safe_from_any_thread() {
    const _: () = any_thread::<Accepted>();
}

#[test]
fn the_dimensions_a_header_declared_are_safe_from_any_thread() {
    const _: () = any_thread::<DeclaredDimensions>();
}

#[test]
fn a_refused_image_is_safe_from_any_thread() {
    const _: () = any_thread::<Refused>();
}

#[test]
fn an_admitted_image_is_safe_from_any_thread() {
    const _: () = any_thread::<Admitted>();
}

#[test]
fn an_image_kind_is_safe_from_any_thread() {
    const _: () = any_thread::<ImageKind>();
}

#[test]
fn which_edge_of_a_size_was_refused_is_safe_from_any_thread() {
    const _: () = any_thread::<Edge>();
}

#[test]
fn why_a_size_is_not_usable_is_safe_from_any_thread() {
    const _: () = any_thread::<SizeNotUsable>();
}

#[test]
fn the_size_the_core_asks_for_is_safe_from_any_thread() {
    const _: () = any_thread::<DrawnSize>();
}

#[test]
fn why_a_server_value_may_not_be_written_is_safe_from_any_thread() {
    const _: () = any_thread::<NotUsableInARequest>();
}

#[test]
fn an_item_identifier_is_safe_from_any_thread() {
    const _: () = any_thread::<ItemId>();
}

#[test]
fn a_content_tag_is_safe_from_any_thread() {
    const _: () = any_thread::<ImageTag>();
}

#[test]
fn an_artwork_request_is_safe_from_any_thread() {
    const _: () = any_thread::<ArtworkRequest>();
}

#[test]
fn what_an_item_has_for_a_kind_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheItemHas>();
}

#[test]
fn why_a_stated_ratio_is_not_usable_is_safe_from_any_thread() {
    const _: () = any_thread::<RatioNotUsable>();
}

#[test]
fn an_aspect_ratio_is_safe_from_any_thread() {
    const _: () = any_thread::<AspectRatio>();
}

#[test]
fn what_is_known_of_an_images_shape_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatShapeIsKnown>();
}

#[test]
fn the_rectangle_reserved_for_an_image_is_safe_from_any_thread() {
    const _: () = any_thread::<ReservedRectangle>();
}

#[test]
fn a_kind_of_cache_entry_is_safe_from_any_thread() {
    const _: () = any_thread::<EntryKind>();
}

#[test]
fn the_skew_an_entry_is_anchored_with_is_safe_from_any_thread() {
    const _: () = any_thread::<Skew>();
}

#[test]
fn the_two_moments_an_entry_is_written_at_are_safe_from_any_thread() {
    const _: () = any_thread::<WrittenAt>();
}

#[test]
fn the_age_of_an_entry_is_safe_from_any_thread() {
    const _: () = any_thread::<Age>();
}

#[test]
fn why_an_age_is_unreadable_is_safe_from_any_thread() {
    const _: () = any_thread::<WhyTheAgeIsUnreadable>();
}

#[test]
fn a_held_cache_entry_is_safe_from_any_thread() {
    const _: () = any_thread::<Held>();
}

#[test]
fn what_a_cache_read_answers_with_is_safe_from_any_thread() {
    const _: () = any_thread::<Answer>();
}

#[test]
fn a_field_name_and_its_treatment_are_safe_from_any_thread() {
    const _: () = any_thread::<FieldName>();
    const _: () = any_thread::<Treatment>();
}

#[test]
fn the_salt_a_correlator_is_taken_under_is_safe_from_any_thread() {
    const _: () = any_thread::<CorrelatorSalt>();
}

#[test]
fn a_correlator_is_safe_from_any_thread() {
    const _: () = any_thread::<Correlator>();
}

#[test]
fn the_entries_view_of_the_cache_is_safe_from_any_thread() {
    const _: () = any_thread::<Entries<'static>>();
}

#[test]
fn the_standing_drop_counts_are_safe_from_any_thread() {
    const _: () = any_thread::<Drops>();
}

#[test]
fn which_reading_a_dropped_entry_failed_is_safe_from_any_thread() {
    const _: () = any_thread::<WhichCheckFailed>();
}

#[test]
fn a_failure_of_the_vocabulary_is_safe_from_any_thread() {
    const _: () = any_thread::<Failure>();
}

#[test]
fn which_of_the_fifteen_a_failure_is_is_safe_from_any_thread() {
    const _: () = any_thread::<Kind>();
}

#[test]
fn a_capability_of_the_server_surface_is_safe_from_any_thread() {
    const _: () = any_thread::<Capability>();
}

#[test]
fn where_the_core_was_reading_is_safe_from_any_thread() {
    const _: () = any_thread::<ReadingSite>();
}

#[test]
fn which_defect_produced_an_internal_fault_is_safe_from_any_thread() {
    const _: () = any_thread::<FaultSite>();
}

#[test]
fn what_the_transport_found_is_safe_from_any_thread() {
    const _: () = any_thread::<TransportOutcome<'static>>();
}

#[test]
fn what_a_server_answered_beside_its_status_is_safe_from_any_thread() {
    const _: () = any_thread::<Answered<'static>>();
}

#[test]
fn the_value_tying_a_delegated_attempt_to_its_answer_is_safe_from_any_thread() {
    const _: () = any_thread::<TieValue>();
}

#[test]
fn what_a_matched_answer_may_be_relayed_as_is_safe_from_any_thread() {
    const _: () = any_thread::<Relayable>();
}

#[test]
fn the_delegated_attempts_a_process_has_started_are_safe_from_any_thread() {
    const _: () = any_thread::<OpenAttempts>();
}

#[test]
fn why_a_value_offered_for_an_attempt_was_refused_is_safe_from_any_thread() {
    const _: () = any_thread::<ValueNotUsable>();
}

#[test]
fn a_value_a_second_attempt_reused_is_safe_from_any_thread() {
    const _: () = any_thread::<ValueAlreadyOpen>();
}

#[test]
fn an_answer_naming_no_started_attempt_is_safe_from_any_thread() {
    const _: () = any_thread::<NoAttemptMatched>();
}

#[test]
fn the_generation_a_token_went_out_under_is_safe_from_any_thread() {
    const _: () = any_thread::<Generation>();
}

#[test]
fn whether_a_server_offers_a_renewal_route_is_safe_from_any_thread() {
    const _: () = any_thread::<RenewalRoute>();
}

#[test]
fn a_rejection_a_renewal_is_answered_against_is_safe_from_any_thread() {
    const _: () = any_thread::<Rejection>();
}

#[test]
fn what_a_rejected_call_does_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatARejectedCallDoes>();
}

#[test]
fn how_a_renewal_ended_is_safe_from_any_thread() {
    const _: () = any_thread::<HowTheRenewalEnded>();
}

#[test]
fn what_a_token_that_dies_mid_playback_does_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatARejectedReportDoes>();
    const _: () = any_thread::<WhatTheOutcomeDoesToPlayback>();
}

#[test]
fn what_a_renewal_outcome_does_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheOutcomeDoes>();
}

#[test]
fn where_a_quick_connect_exchange_stands_is_safe_from_any_thread() {
    const _: () = any_thread::<WhileWaiting>();
}

#[test]
fn how_a_quick_connect_call_ended_is_safe_from_any_thread() {
    const _: () = any_thread::<HowTheCallEnded>();
}

#[test]
fn the_two_values_a_quick_connect_exchange_was_issued_with_are_safe_from_any_thread() {
    const _: () = any_thread::<IssuedExchange>();
}

#[test]
fn what_one_write_queue_entry_asserts_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatIsAsserted>();
}

#[test]
fn the_item_a_queued_write_is_about_is_safe_from_any_thread() {
    const _: () = any_thread::<Target>();
}

#[test]
fn one_queued_write_is_safe_from_any_thread() {
    const _: () = any_thread::<Entry<()>>();
}

#[test]
fn what_a_queue_dropped_at_its_bound_is_safe_from_any_thread() {
    const _: () = any_thread::<Dropped>();
}

#[test]
fn what_an_enqueue_did_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheEnqueueDid>();
}

#[test]
fn one_sessions_write_queue_is_safe_from_any_thread() {
    const _: () = any_thread::<WriteQueue<()>>();
}

#[test]
fn an_event_that_reports_without_waiting_is_safe_from_any_thread() {
    const _: () = any_thread::<ReportsWithoutWaiting>();
}

#[test]
fn what_such_an_event_does_to_the_interval_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatItDoesToTheInterval>();
}

#[test]
fn the_progress_reporting_interval_is_safe_from_any_thread() {
    const _: () = any_thread::<TheInterval>();
}

#[test]
fn what_occasioned_a_position_report_is_safe_from_any_thread() {
    const _: () = any_thread::<ReportedOn>();
}

#[test]
fn a_position_report_is_safe_from_any_thread() {
    const _: () = any_thread::<PositionReport>();
}

#[test]
fn what_observing_a_position_did_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatObservingDid>();
}

#[test]
fn one_items_reporting_is_safe_from_any_thread() {
    const _: () = any_thread::<Reporting>();
}

#[test]
fn what_announcing_a_window_did_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheAnnouncementDid>();
}

#[test]
fn an_announced_window_is_safe_from_any_thread() {
    const _: () = any_thread::<AnnouncedWindow>();
}

#[test]
fn what_holding_an_entrys_fetch_did_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheHoldDid>();
}

#[test]
fn what_withdrawing_from_an_entrys_fetch_did_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheWithdrawalDid>();
}

#[test]
fn the_fetches_announcements_share_are_safe_from_any_thread() {
    const _: () = any_thread::<SharedFetches>();
}

#[test]
fn one_sessions_renewals_are_safe_from_any_thread() {
    const _: () = any_thread::<Renewals>();
}

#[test]
fn when_a_renewal_is_due_is_safe_from_any_thread() {
    const _: () = any_thread::<RenewalSchedule>();
}

#[test]
fn the_jitter_seam_a_client_supplies_is_safe_from_any_thread() {
    const _: () = any_thread::<dyn Jitter>();
}

#[test]
fn the_attempts_one_call_has_spent_are_safe_from_any_thread() {
    const _: () = any_thread::<Attempts>();
}

#[test]
fn what_a_request_does_to_the_server_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheRequestDoes>();
}

#[test]
fn what_a_failure_does_under_the_retry_policy_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatAFailureDoes>();
}

#[test]
fn what_a_call_does_after_a_failed_attempt_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheCallDoesNext>();
}

#[test]
fn the_wait_before_the_next_attempt_is_safe_from_any_thread() {
    const _: () = any_thread::<TheWait>();
}

#[test]
fn why_a_call_stopped_rather_than_attempting_again_is_safe_from_any_thread() {
    const _: () = any_thread::<WhyTheCallStopped>();
}

#[test]
fn the_decoded_bytes_budget_is_safe_from_any_thread() {
    const _: () = any_thread::<Budget>();
}

#[test]
fn a_budget_a_client_may_not_set_is_safe_from_any_thread() {
    const _: () = any_thread::<BudgetNotUsable>();
}

#[test]
fn the_buffer_one_decode_occupies_is_safe_from_any_thread() {
    const _: () = any_thread::<DecodedBytes>();
}

#[test]
fn what_asking_for_a_decode_does_is_safe_from_any_thread() {
    const _: () = any_thread::<WhatTheAskDoes>();
}

#[test]
fn the_decoded_bytes_bookkeeping_is_safe_from_any_thread() {
    const _: () = any_thread::<DecodedBytesHeld>();
}

#[test]
fn what_a_play_call_hands_back_is_safe_from_any_thread() {
    const _: () = any_thread::<StreamKind>();
    const _: () = any_thread::<Picture>();
    const _: () = any_thread::<OfferedStream>();
    const _: () = any_thread::<RoutesOffered>();
    const _: () = any_thread::<ConversionAddress>();
    const _: () = any_thread::<OfferedSource>();
    const _: () = any_thread::<WhatTheServerOffered>();
    const _: () = any_thread::<Rung>();
    const _: () = any_thread::<Ceiling>();
    const _: () = any_thread::<Preferences<'static>>();
    const _: () = any_thread::<ChosenStream>();
    const _: () = any_thread::<WhatThePlayerOpens>();
    const _: () = any_thread::<Handover>();
    const _: () = any_thread::<NothingPlayable>();
}
