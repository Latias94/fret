//! Adapter seam contracts for immediate-mode ecosystem integrations.
//!
//! These types provide a minimal, explicit contract for delegating canonical component behavior
//! while reporting interaction signals back to immediate-mode adapters.

use std::hash::Hash;
use std::sync::Arc;

use fret_core::Rect;
use fret_runtime::Model;
use fret_ui::{GlobalElementId, UiHost};

use super::{ResponseExt, UiWriterImUiFacadeExt};

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
pub struct AdapterSeamOptions<'a> {
    pub reporter: Option<&'a mut AdapterSignalReporter<'a>>,
    pub focus_restore_target: Option<GlobalElementId>,
}

impl Default for AdapterSeamOptions<'_> {
    fn default() -> Self {
        Self {
            reporter: None,
            focus_restore_target: None,
        }
    }
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

/// Non-shadcn example adapter: wraps the canonical `button` helper and emits seam signals.
pub fn button_adapter<H: UiHost, K: Hash>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    identity_key: K,
    label: impl Into<Arc<str>>,
    mut options: AdapterSeamOptions<'_>,
) -> ResponseExt {
    let label = label.into();
    let response = ui.push_id(identity_key, |ui| ui.button(label.clone()));
    report_adapter_signal(response, &mut options)
}

/// Non-shadcn example adapter: wraps the canonical `checkbox_model` helper and emits seam signals.
pub fn checkbox_model_adapter<H: UiHost, K: Hash>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    identity_key: K,
    label: impl Into<Arc<str>>,
    model: &Model<bool>,
    mut options: AdapterSeamOptions<'_>,
) -> ResponseExt {
    let label = label.into();
    let response = ui.push_id(identity_key, |ui| ui.checkbox_model(label.clone(), model));
    report_adapter_signal(response, &mut options)
}
