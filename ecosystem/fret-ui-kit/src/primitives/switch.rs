//! Switch primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/switch/src/switch.tsx`
//!
//! In Radix, `Switch` is a button-like control with `role="switch"` and a boolean checked state.
//! In Fret, this maps onto [`fret_core::SemanticsRole::Switch`] and `checked: Some(bool)`.

use std::sync::Arc;

use fret_ui::element::PressableA11y;

/// A11y metadata for a Radix-style switch pressable.
pub fn switch_a11y(label: Option<Arc<str>>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(fret_core::SemanticsRole::Switch),
        label,
        checked: Some(checked),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_a11y_sets_role_and_checked() {
        let a11y = switch_a11y(Some(Arc::from("Airplane mode")), true);
        assert_eq!(a11y.role, Some(fret_core::SemanticsRole::Switch));
        assert_eq!(a11y.checked, Some(true));
        assert_eq!(a11y.label.as_deref(), Some("Airplane mode"));
    }
}

