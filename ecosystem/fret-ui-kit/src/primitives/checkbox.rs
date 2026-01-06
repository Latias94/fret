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

/// Converts an optional boolean into a tri-state checkbox value.
///
/// This maps `None` to the indeterminate/mixed outcome, matching Radix
/// `checked="indeterminate"`.
pub fn checked_state_from_optional_bool(value: Option<bool>) -> CheckedState {
    match value {
        Some(true) => CheckedState::Checked,
        Some(false) => CheckedState::Unchecked,
        None => CheckedState::Indeterminate,
    }
}

/// Toggle behavior for an optional boolean that represents Radix tri-state outcomes.
///
/// This is the policy used by the shadcn checkbox recipe when binding to `Model<Option<bool>>`:
/// - `None` (indeterminate) -> `Some(true)`
/// - otherwise invert the boolean
pub fn toggle_optional_bool(value: Option<bool>) -> Option<bool> {
    match value {
        None => Some(true),
        Some(true) => Some(false),
        Some(false) => Some(true),
    }
}

/// A11y metadata for a Radix-style checkbox pressable.
pub fn checkbox_a11y(label: Option<Arc<str>>, state: CheckedState) -> PressableA11y {
    PressableA11y {
        role: Some(fret_core::SemanticsRole::Checkbox),
        label,
        checked: state.to_semantics_checked(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_bool_maps_to_tristate() {
        assert_eq!(checked_state_from_optional_bool(None), CheckedState::Indeterminate);
        assert_eq!(
            checked_state_from_optional_bool(Some(true)),
            CheckedState::Checked
        );
        assert_eq!(
            checked_state_from_optional_bool(Some(false)),
            CheckedState::Unchecked
        );
    }

    #[test]
    fn toggle_optional_bool_matches_radix_outcomes() {
        assert_eq!(toggle_optional_bool(None), Some(true));
        assert_eq!(toggle_optional_bool(Some(true)), Some(false));
        assert_eq!(toggle_optional_bool(Some(false)), Some(true));
    }
}
