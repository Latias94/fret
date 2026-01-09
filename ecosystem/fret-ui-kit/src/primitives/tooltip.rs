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

use std::sync::Arc;

use fret_core::{Point, PointerType, Px, Rect};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, UiHost};

pub use crate::tooltip_provider::{
    TooltipProviderConfig, current_config, last_opened_tooltip, note_closed, note_opened_tooltip,
    open_delay_ticks, open_delay_ticks_with_base, with_tooltip_provider,
};

pub use crate::primitives::popper::{Align, ArrowOptions, LayoutDirection, Side};

use crate::declarative::ModelWatchExt;
use crate::headless::hover_intent::{HoverIntentConfig, HoverIntentState, HoverIntentUpdate};
use crate::headless::safe_hover;
use crate::primitives::trigger_a11y;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerMoveCx, UiActionHost};

/// Stamps Radix-like trigger relationships:
/// - `described_by_element` mirrors `aria-describedby` (by element id).
///
/// In Radix Tooltip, the trigger advertises the tooltip content by id. In Fret we model this via
/// a portable element-id relationship that resolves into `SemanticsNode.described_by` when the
/// tooltip content is mounted.
pub fn apply_tooltip_trigger_a11y(
    trigger: AnyElement,
    open: bool,
    tooltip_element: GlobalElementId,
) -> AnyElement {
    trigger_a11y::apply_trigger_described_by(trigger, open.then_some(tooltip_element))
}

/// Stable per-overlay root naming convention for tooltip overlays.
pub fn tooltip_root_name(id: GlobalElementId) -> String {
    OverlayController::tooltip_root_name(id)
}

#[derive(Debug, Clone, Copy)]
pub struct TooltipInteractionConfig {
    pub disable_hoverable_content: bool,
    /// Overrides the provider-derived open delay (ticks).
    pub open_delay_ticks_override: Option<u64>,
    /// Overrides the hover-close delay (ticks).
    pub close_delay_ticks_override: Option<u64>,
    /// Pointer safe-hover corridor buffer.
    pub safe_hover_buffer: Px,
}

impl Default for TooltipInteractionConfig {
    fn default() -> Self {
        Self {
            disable_hoverable_content: false,
            open_delay_ticks_override: None,
            close_delay_ticks_override: None,
            safe_hover_buffer: Px(5.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipInteractionUpdate {
    pub open: bool,
    pub wants_continuous_ticks: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipFocusEdgeState {
    was_focused: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipOpenBroadcastState {
    last_seen_open_token: u64,
}

/// Returns a per-tooltip pointer tracking model stored in element state.
///
/// Tooltip pointer tracking is used to approximate Radix Tooltip's hoverable-content grace area:
/// while open, the tooltip remains open if the pointer lies within a "safe corridor" between the
/// trigger anchor and the floating content bounds.
pub fn tooltip_last_pointer_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Point>> {
    #[derive(Default)]
    struct State {
        model: Option<Model<Option<Point>>>,
    }

    let existing = cx.with_state(State::default, |st| st.model.clone());
    if let Some(model) = existing {
        return model;
    }

    let model = cx.app.models_mut().insert(None);
    cx.with_state(State::default, |st| st.model = Some(model.clone()));
    model
}

/// Installs a pointer-move observer that updates the tooltip's pointer tracking model.
///
/// Notes:
/// - This is intended for hoverable-content tooltips. When `disable_hoverable_content=true`, the
///   recipe should skip installing this observer.
/// - Touch pointers are ignored to match Radix Tooltip's "no hover on touch" behavior.
pub fn tooltip_install_pointer_move_tracker(
    request: &mut OverlayRequest,
    last_pointer: Model<Option<Point>>,
) {
    let last_pointer = last_pointer.clone();
    request.dismissible_on_pointer_move = Some(Arc::new(
        move |host: &mut dyn UiActionHost, _acx: ActionCx, mv: PointerMoveCx| {
            if mv.pointer_type == PointerType::Touch {
                return false;
            }
            let _ = host
                .models_mut()
                .update(&last_pointer, |v| *v = Some(mv.position));
            false
        },
    ));
}

/// Updates an internal hover-intent state machine using Radix-aligned tooltip timing rules.
///
/// This is a reusable policy helper for recipes:
/// - `open_delay_ticks` comes from the provider delay group, unless overridden.
/// - Blur closes immediately.
/// - Hoverable-content tooltips remain open while the pointer stays inside a safe-hover corridor
///   between `anchor_bounds` and `floating_bounds`.
pub fn tooltip_update_interaction<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_hovered: bool,
    trigger_focused: bool,
    force_close: bool,
    last_pointer: Model<Option<Point>>,
    anchor_bounds: Option<Rect>,
    floating_bounds: Option<Rect>,
    cfg: TooltipInteractionConfig,
) -> TooltipInteractionUpdate {
    let tooltip_id = cx.root_id();
    let (last_id, token) = last_opened_tooltip(cx).unwrap_or((tooltip_id, 0));
    let should_close_because_other_opened =
        cx.with_state(TooltipOpenBroadcastState::default, |st| {
            let should_close = token > st.last_seen_open_token && last_id != tooltip_id;
            st.last_seen_open_token = token;
            should_close
        });
    if should_close_because_other_opened {
        cx.with_state(HoverIntentState::default, |st| st.set_open(false));
    }

    let now = cx.app.frame_id().0;

    let was_open = cx.with_state(HoverIntentState::default, |st| st.is_open());

    let (close_delay_ticks, blurred) = cx.with_state(TooltipFocusEdgeState::default, |st| {
        let was = st.was_focused;
        st.was_focused = trigger_focused;
        let blurred = was && !trigger_focused;

        let close_delay_ticks = if blurred {
            0
        } else if trigger_focused {
            0
        } else {
            cfg.close_delay_ticks_override.unwrap_or(0)
        };

        (close_delay_ticks, blurred)
    });

    let open_delay_ticks = if trigger_focused {
        0
    } else if let Some(base_delay) = cfg.open_delay_ticks_override {
        open_delay_ticks_with_base(cx, now, base_delay)
    } else {
        open_delay_ticks(cx, now)
    };

    let intent_cfg = HoverIntentConfig::new(open_delay_ticks, close_delay_ticks);

    if force_close {
        cx.with_state(HoverIntentState::default, |st| st.set_open(false));
        if was_open {
            note_closed(cx, now);
        }
        return TooltipInteractionUpdate {
            open: false,
            wants_continuous_ticks: false,
        };
    }

    if was_open && !cfg.disable_hoverable_content && !blurred {
        cx.observe_model(&last_pointer, Invalidation::Paint);
    }

    let pointer_safe = if was_open && !cfg.disable_hoverable_content && !blurred {
        let pointer = cx.app.models().read(&last_pointer, |v| *v).ok().flatten();
        match (pointer, anchor_bounds, floating_bounds) {
            (Some(pointer), Some(anchor), Some(floating)) => {
                safe_hover::safe_hover_contains(pointer, anchor, floating, cfg.safe_hover_buffer)
            }
            _ => false,
        }
    } else {
        false
    };

    let HoverIntentUpdate {
        open,
        wants_continuous_ticks,
    } = cx.with_state(HoverIntentState::default, |st| {
        st.update(
            trigger_hovered || trigger_focused || pointer_safe,
            now,
            intent_cfg,
        )
    });

    if !was_open && open {
        let token = note_opened_tooltip(cx, tooltip_id);
        cx.with_state(TooltipOpenBroadcastState::default, |st| {
            st.last_seen_open_token = token;
        });
    }

    if was_open && !open {
        note_closed(cx, now);
    }

    TooltipInteractionUpdate {
        open,
        wants_continuous_ticks,
    }
}

/// A Radix-shaped `Tooltip` root configuration surface (open state only).
///
/// Radix Tooltip supports a controlled/uncontrolled `open` state (`open` + `defaultOpen`). In
/// Fret, this root helper standardizes how recipes derive the open model before applying hover
/// intent or provider delay-group policy.
#[derive(Debug, Clone, Default)]
pub struct TooltipRoot {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl TooltipRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        tooltip_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    pub fn open_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_open_model(cx).model()
    }

    /// Reads the current open value from the derived open model.
    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.open_model(cx);
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }
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
    fn tooltip_root_open_model_uses_controlled_model() {
        let window = Default::default();
        let mut app = App::new();

        let controlled = app.models_mut().insert(true);
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let root = TooltipRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

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
            let trigger = apply_tooltip_trigger_a11y(trigger, true, tooltip);
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
            let trigger = apply_tooltip_trigger_a11y(trigger, true, tooltip);
            let ElementKind::Semantics(props) = &trigger.kind else {
                panic!("expected semantics");
            };
            assert_eq!(props.described_by_element, Some(tooltip.0));
        });
    }

    #[test]
    fn apply_tooltip_trigger_a11y_clears_described_by_when_closed() {
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
            let trigger = apply_tooltip_trigger_a11y(trigger, false, tooltip);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.described_by_element, None);
        });
    }
}
