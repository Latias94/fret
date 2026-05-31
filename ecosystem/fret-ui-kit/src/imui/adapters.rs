//! Adapter seam contracts for immediate-mode ecosystem integrations.
//!
//! These types provide a minimal, explicit contract for delegating canonical component behavior
//! while reporting interaction signals back to immediate-mode adapters.
//!
//! This module intentionally exposes only the seam contract. Example wrapper functions should live
//! in tests or external crates so they do not become an accidental second public helper family.

use fret_ui::GlobalElementId;

use super::ResponseExt;

mod signal;

pub use signal::{AdapterSignalMetadata, AdapterSignalRecord, AdapterSignalReporter};

/// Shared seam options accepted by immediate adapter helpers.
#[derive(Default)]
pub struct AdapterSeamOptions<'a> {
    pub reporter: Option<&'a mut AdapterSignalReporter<'a>>,
    pub focus_restore_target: Option<GlobalElementId>,
}

/// Emit one adapter signal record through the optional reporter.
pub fn report_adapter_signal(
    response: ResponseExt,
    options: &mut AdapterSeamOptions<'_>,
) -> ResponseExt {
    if let Some(reporter) = &mut options.reporter {
        (**reporter)(AdapterSignalRecord::new(
            response.id(),
            response,
            AdapterSignalMetadata::new(response.rect(), options.focus_restore_target),
        ));
    }
    response
}
