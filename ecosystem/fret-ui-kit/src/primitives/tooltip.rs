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
use fret_ui::element::ElementKind;

pub use crate::tooltip_provider::{
    TooltipProviderConfig, current_config, note_closed, open_delay_ticks,
    open_delay_ticks_with_base, with_tooltip_provider,
};

pub use crate::primitives::popper::{Align, ArrowOptions, LayoutDirection, Side};

use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Stamps Radix-like trigger relationships:
/// - `described_by_element` mirrors `aria-describedby` (by element id).
///
/// In Radix Tooltip, the trigger advertises the tooltip content by id. In Fret we model this via
/// a portable element-id relationship that resolves into `SemanticsNode.described_by` when the
/// tooltip content is mounted.
pub fn apply_tooltip_trigger_a11y(
    mut trigger: AnyElement,
    tooltip_element: GlobalElementId,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(props) => {
            props.a11y.described_by_element = Some(tooltip_element.0);
        }
        ElementKind::Semantics(props) => {
            props.described_by_element = Some(tooltip_element.0);
        }
        _ => {}
    }
    trigger
}

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
    use fret_ui::element::{ElementKind, LayoutStyle, PressableProps, SemanticsProps};

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

    #[test]
    fn apply_tooltip_trigger_a11y_sets_described_by_on_pressable() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let tooltip = GlobalElementId(0xbeef);
            let trigger = apply_tooltip_trigger_a11y(trigger, tooltip);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.described_by_element, Some(tooltip.0));
        });
    }

    #[test]
    fn apply_tooltip_trigger_a11y_sets_described_by_on_semantics() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.semantics(SemanticsProps::default(), |_cx| Vec::new());
            let tooltip = GlobalElementId(0xbeef);
            let trigger = apply_tooltip_trigger_a11y(trigger, tooltip);
            let ElementKind::Semantics(props) = &trigger.kind else {
                panic!("expected semantics");
            };
            assert_eq!(props.described_by_element, Some(tooltip.0));
        });
    }
}
