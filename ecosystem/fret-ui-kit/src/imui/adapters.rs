//! Adapter seam contracts for immediate-mode ecosystem integrations.
//!
//! These types provide a minimal, explicit contract for delegating canonical component behavior
//! while reporting interaction signals back to immediate-mode adapters.
//!
//! This module intentionally exposes only the seam contract. Example wrapper functions should live
//! in tests or external crates so they do not become an accidental second public helper family.

use fret_core::Rect;
use fret_ui::GlobalElementId;

use super::ResponseExt;

/// Optional metadata reported by adapter seams for focus/geometry choreography.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterSignalMetadata {
    pub rect: Option<Rect>,
    pub focus_restore_target: Option<GlobalElementId>,
}

/// A single adapter signal report emitted after rendering a canonical wrapper.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterSignalRecord {
    pub identity: Option<GlobalElementId>,
    pub response: ResponseExt,
    pub metadata: AdapterSignalMetadata,
}

/// Signal reporter callback used by adapter seams.
pub type AdapterSignalReporter<'a> = dyn FnMut(AdapterSignalRecord) + 'a;

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
        (**reporter)(AdapterSignalRecord {
            identity: response.id,
            response,
            metadata: AdapterSignalMetadata {
                rect: response.core.rect,
                focus_restore_target: options.focus_restore_target,
            },
        });
    }
    response
}
