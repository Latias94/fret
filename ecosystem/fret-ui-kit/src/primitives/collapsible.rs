//! Collapsible primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing collapsible behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/collapsible/src/collapsible.tsx`

use std::sync::Arc;

use fret_core::{Px, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, PressableA11y, SemanticsProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::primitives::trigger_a11y;

/// Returns an open-state model that behaves like Radix `useControllableState` (`open` /
/// `defaultOpen`).
pub fn collapsible_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// A Radix-shaped `Collapsible` root configuration surface.
///
/// Upstream supports a controlled/uncontrolled `open` state (`open` + `defaultOpen`). In Fret this
/// maps to either:
/// - a caller-provided `Model<bool>` (controlled), or
/// - an internal `Model<bool>` stored in element state (uncontrolled).
#[derive(Debug, Clone, Default)]
pub struct CollapsibleRoot {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl CollapsibleRoot {
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
        collapsible_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    /// Reads the current open value from the derived open model.
    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.use_open_model(cx).model();
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }
}

/// Semantics wrapper props for a collapsible root container.
pub fn collapsible_root_semantics(disabled: bool, open: bool) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::Generic,
        disabled,
        expanded: Some(open),
        ..Default::default()
    }
}

/// A11y metadata for a collapsible trigger pressable.
pub fn collapsible_trigger_a11y(label: Option<Arc<str>>, open: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label,
        expanded: Some(open),
        ..Default::default()
    }
}

/// Stamps Radix-like trigger relationships:
/// - `controls_element` mirrors `aria-controls` (by element id).
///
/// In Radix Collapsible, the trigger points at the content by id. In Fret we model this via a
/// portable element-id relationship that resolves into `SemanticsNode.controls` when the content
/// is mounted.
pub fn apply_collapsible_trigger_controls(
    trigger: AnyElement,
    content_element: GlobalElementId,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls(trigger, Some(content_element))
}

/// Stamps Radix-like trigger relationships:
/// - `expanded` mirrors `aria-expanded`.
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_collapsible_trigger_controls_expanded(
    trigger: AnyElement,
    content_element: GlobalElementId,
    open: bool,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls_expanded(trigger, Some(open), Some(content_element))
}

/// Read the last cached open height for a collapsible content subtree.
///
/// This is a Radix-aligned outcome for Collapsible/Accordion height animations: Radix measures the
/// content and exposes it to styling via CSS variables. Fret caches the value in per-element state.
pub fn last_measured_height_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
) -> Px {
    crate::declarative::collapsible_motion::last_measured_height_for(cx, state_id)
}

/// Read the last cached open size for a collapsible content subtree.
pub fn last_measured_size_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
) -> Size {
    crate::declarative::collapsible_motion::last_measured_size_for(cx, state_id)
}

/// Update the cached open height from the previously-laid-out bounds of `wrapper_element_id`.
pub fn update_measured_height_if_open_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
    wrapper_element_id: GlobalElementId,
    open: bool,
    animating: bool,
) -> Px {
    crate::declarative::collapsible_motion::update_measured_height_if_open_for(
        cx,
        state_id,
        wrapper_element_id,
        open,
        animating,
    )
}

/// Update the cached open size from a "measurement element" that is laid out off-flow.
pub fn update_measured_size_from_element_if_open_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
    measure_element_id: GlobalElementId,
    open: bool,
) -> Size {
    crate::declarative::collapsible_motion::update_measured_size_from_element_if_open_for(
        cx,
        state_id,
        measure_element_id,
        open,
    )
}

/// Layout refinement for an off-flow measurement wrapper.
pub fn collapsible_measurement_wrapper_refinement() -> crate::LayoutRefinement {
    crate::declarative::collapsible_motion::collapsible_measurement_wrapper_refinement()
}

/// Compute wrapper mounting and layout patches for a collapsible content subtree.
#[allow(clippy::too_many_arguments)]
pub fn collapsible_height_wrapper_refinement(
    open: bool,
    force_mount: bool,
    require_measurement_for_close: bool,
    transition: crate::headless::transition::TransitionOutput,
    measured_height: Px,
) -> (bool, crate::LayoutRefinement) {
    crate::declarative::collapsible_motion::collapsible_height_wrapper_refinement(
        open,
        force_mount,
        require_measurement_for_close,
        transition,
        measured_height,
    )
}

pub use crate::declarative::collapsible_motion::MeasuredHeightMotionOutput;

/// Computes a measured-height motion plan for the current element root.
///
/// This is a Radix-aligned outcome for components that animate open/close using measured content
/// height (Collapsible, Accordion items, etc.).
#[allow(clippy::too_many_arguments)]
pub fn measured_height_motion_for_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    force_mount: bool,
    require_measurement_for_close: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
) -> MeasuredHeightMotionOutput {
    crate::declarative::collapsible_motion::measured_height_motion_for_root(
        cx,
        open,
        force_mount,
        require_measurement_for_close,
        open_ticks,
        close_ticks,
        ease,
    )
}

/// Updates cached measured size/height for a motion plan based on the wrapper element id.
pub fn update_measured_for_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    motion: MeasuredHeightMotionOutput,
    wrapper_element_id: GlobalElementId,
) -> Size {
    crate::declarative::collapsible_motion::update_measured_for_motion(
        cx,
        motion,
        wrapper_element_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, LayoutStyle, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn collapsible_use_open_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = collapsible_use_open_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                false
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    #[test]
    fn apply_collapsible_trigger_controls_sets_controls_on_pressable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let content = GlobalElementId(0xbeef);
            let trigger = apply_collapsible_trigger_controls(trigger, content);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.controls_element, Some(content.0));
        });
    }

    #[test]
    fn apply_collapsible_trigger_controls_expanded_sets_expanded_and_controls() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let content = GlobalElementId(0xbeef);
            let trigger = apply_collapsible_trigger_controls_expanded(trigger, content, true);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(content.0));
        });
    }
}
