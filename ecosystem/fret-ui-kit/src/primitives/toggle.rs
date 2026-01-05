//! Toggle primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing toggle behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/toggle/src/toggle.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;

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

