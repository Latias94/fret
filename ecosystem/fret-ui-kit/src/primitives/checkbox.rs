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
use fret_ui_headless::checked_state::CheckedState;

/// Returns a checked-state model that behaves like Radix `useControllableState` (`checked` /
/// `defaultChecked`).
pub fn checkbox_use_checked_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<CheckedState>>,
    default_checked: impl FnOnce() -> CheckedState,
) -> crate::primitives::controllable_state::ControllableModel<CheckedState> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_checked)
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
}
