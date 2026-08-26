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
use flowfin_core::cache::{ByteStore, EntryKey};
use flowfin_core::clock::Clocks;
use flowfin_core::diagnostics::{Diagnostics, DiagnosticsSink};
use flowfin_core::measurement::{Measurement, MeasurementSink};
use flowfin_core::server::QueryResult;
use flowfin_core::server::address::{AddressNotUsable, BaseAddress};
use flowfin_core::server::federation::Federation;
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
fn a_query_result_is_safe_from_any_thread() {
    const _: () = any_thread::<QueryResult>();
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
fn the_federation_register_is_safe_from_any_thread() {
    const _: () = any_thread::<Federation<'static>>();
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
