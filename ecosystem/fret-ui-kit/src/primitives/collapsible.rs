//! Collapsible primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing collapsible behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/collapsible/src/collapsible.tsx`

use std::sync::Arc;

use fret_core::{Px, SemanticsRole, Size};
use fret_ui::element::{PressableA11y, SemanticsProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

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
