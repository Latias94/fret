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
//! Fret represents this via [`CheckedState`]. Indeterminate is mapped onto the semantics tree
//! using `checked_state: Mixed`, while `checked: Option<bool>` remains as a legacy binary surface.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::PressableA11y;
use fret_ui::{ElementContext, UiHost};

pub use crate::headless::checked_state::CheckedState;

/// Returns a checked-state model that behaves like Radix `useControllableState` (`checked` /
/// `defaultChecked`).
pub fn checkbox_use_checked_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<CheckedState>>,
    default_checked: impl FnOnce() -> CheckedState,
) -> crate::primitives::controllable_state::ControllableModel<CheckedState> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_checked)
}

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
    let checked_state = state.to_semantics_checked_state();
    let checked = state.to_semantics_checked();
    PressableA11y {
        role: Some(fret_core::SemanticsRole::Checkbox),
        label,
        checked,
        checked_state,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn checkbox_use_checked_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(CheckedState::Checked);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = checkbox_use_checked_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                CheckedState::Unchecked
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    #[test]
    fn optional_bool_maps_to_tristate() {
        assert_eq!(
            checked_state_from_optional_bool(None),
            CheckedState::Indeterminate
        );
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
