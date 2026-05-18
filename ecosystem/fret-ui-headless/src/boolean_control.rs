//! Headless state transitions for boolean-family controls.
//!
//! This module owns deterministic, renderer-free behavior shared by checkbox/switch recipes and
//! UI primitive facades. Runtime wiring, semantics stamping, and visual policy stay in UI crates.

use crate::checked_state::CheckedState;

/// Maps an optional boolean binding onto a Radix-style checkbox tri-state.
///
/// `None` represents the indeterminate/mixed outcome.
pub fn checkbox_checked_state_from_optional_bool(value: Option<bool>) -> CheckedState {
    match value {
        Some(true) => CheckedState::Checked,
        Some(false) => CheckedState::Unchecked,
        None => CheckedState::Indeterminate,
    }
}

/// Toggle behavior for optional boolean checkbox bindings.
///
/// This matches Radix checkbox outcomes:
/// - `None` (indeterminate) -> `Some(true)`
/// - otherwise invert the boolean
pub fn checkbox_toggle_optional_bool(value: Option<bool>) -> Option<bool> {
    match value {
        None => Some(true),
        Some(true) => Some(false),
        Some(false) => Some(true),
    }
}

/// Maps an optional boolean binding onto a Radix-style switch checked state.
///
/// Radix `Switch` is boolean-only. Missing values render as off.
pub fn switch_checked_from_optional_bool(value: Option<bool>) -> bool {
    value.unwrap_or(false)
}

/// Toggle behavior for optional boolean switch bindings.
///
/// Missing values are treated as off before toggling.
pub fn switch_toggle_optional_bool(value: Option<bool>) -> Option<bool> {
    Some(!switch_checked_from_optional_bool(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_optional_bool_maps_to_tristate() {
        assert_eq!(
            checkbox_checked_state_from_optional_bool(None),
            CheckedState::Indeterminate
        );
        assert_eq!(
            checkbox_checked_state_from_optional_bool(Some(true)),
            CheckedState::Checked
        );
        assert_eq!(
            checkbox_checked_state_from_optional_bool(Some(false)),
            CheckedState::Unchecked
        );
    }

    #[test]
    fn checkbox_optional_bool_toggle_matches_radix_outcomes() {
        assert_eq!(checkbox_toggle_optional_bool(None), Some(true));
        assert_eq!(checkbox_toggle_optional_bool(Some(true)), Some(false));
        assert_eq!(checkbox_toggle_optional_bool(Some(false)), Some(true));
    }

    #[test]
    fn switch_optional_bool_maps_missing_to_off() {
        assert!(!switch_checked_from_optional_bool(None));
        assert!(!switch_checked_from_optional_bool(Some(false)));
        assert!(switch_checked_from_optional_bool(Some(true)));
    }

    #[test]
    fn switch_optional_bool_toggle_inverts_and_sets_some() {
        assert_eq!(switch_toggle_optional_bool(None), Some(true));
        assert_eq!(switch_toggle_optional_bool(Some(false)), Some(true));
        assert_eq!(switch_toggle_optional_bool(Some(true)), Some(false));
    }
}
