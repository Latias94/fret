//! Toggle primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing toggle behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/toggle/src/toggle.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::PressableA11y;
use fret_ui::{ElementContext, UiHost};

/// A11y metadata for a toggle-like pressable.
///
/// Note: Radix uses `aria-pressed` to represent the "on" state. Fret currently maps this to the
/// `selected` outcome on a button-like semantics role.
pub fn toggle_a11y(label: Option<Arc<str>>, pressed: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label,
        selected: pressed,
        ..Default::default()
    }
}

/// Returns a pressed-state model that behaves like Radix `useControllableState` (`pressed` /
/// `defaultPressed`).
pub fn toggle_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<bool>>,
    default_pressed: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_pressed)
}
