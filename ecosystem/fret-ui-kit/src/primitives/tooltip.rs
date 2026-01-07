//! Tooltip helpers (Radix `@radix-ui/react-tooltip` outcomes).
//!
//! Radix Tooltip composes three concerns:
//!
//! - Provider-scoped open delay behavior (`TooltipProvider` / "delay group")
//! - Floating placement (`Popper`)
//! - Dismiss / focus policy (handled by per-window overlay infrastructure in Fret)
//!
//! In `fret-ui-kit`, we keep the reusable delay mechanics split into:
//!
//! - `crate::headless::tooltip_delay_group` (pure, deterministic state machine), and
//! - `crate::tooltip_provider` (provider stack service for declarative trees).
//!
//! This module is the Radix-named facade that re-exports the pieces under a single entry point.

pub use crate::headless::tooltip_delay_group::{TooltipDelayGroupConfig, TooltipDelayGroupState};

use fret_ui::element::AnyElement;

pub use crate::tooltip_provider::{
    TooltipProviderConfig, current_config, note_closed, open_delay_ticks,
    open_delay_ticks_with_base, with_tooltip_provider,
};

pub use crate::primitives::popper::{Align, ArrowOptions, LayoutDirection, Side};

use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Stable per-overlay root naming convention for tooltip overlays.
pub fn tooltip_root_name(id: GlobalElementId) -> String {
    OverlayController::tooltip_root_name(id)
}

/// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
///
/// This is a convenience helper for authoring Radix-shaped tooltip roots:
/// - if `controlled_open` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_open` (Radix `defaultOpen`)
pub fn tooltip_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// Builds an overlay request for a Radix-style tooltip.
pub fn tooltip_request(
    id: GlobalElementId,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::tooltip(id, presence, children);
    request.root_name = Some(tooltip_root_name(id));
    request
}

/// Requests a tooltip overlay for the current window.
pub fn request_tooltip<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;

    #[test]
    fn tooltip_request_sets_default_root_name() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |_cx| {
                let id = GlobalElementId(0x123);
                let req = tooltip_request(id, OverlayPresence::instant(true), Vec::new());
                let expected = tooltip_root_name(id);
                assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
            },
        );
    }
}
