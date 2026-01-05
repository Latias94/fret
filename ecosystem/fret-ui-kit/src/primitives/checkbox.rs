//! Checkbox primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/checkbox/src/checkbox.tsx`
//!
//! Radix models checkbox state as a tri-state:
//! - `false` (unchecked)
//! - `true` (checked)
//! - `"indeterminate"`
//!
//! Fret represents this via [`CheckedState`], and maps it onto the semantics tree using
//! `checked: Option<bool>` where `None` represents indeterminate.

use std::sync::Arc;

use fret_ui::element::PressableA11y;

pub use crate::headless::checked_state::CheckedState;

/// A11y metadata for a Radix-style checkbox pressable.
pub fn checkbox_a11y(label: Option<Arc<str>>, state: CheckedState) -> PressableA11y {
    PressableA11y {
        role: Some(fret_core::SemanticsRole::Checkbox),
        label,
        checked: state.to_semantics_checked(),
        ..Default::default()
    }
}
