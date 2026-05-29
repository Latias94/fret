//! Switch primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/switch/src/switch.tsx`
//!
//! In Radix, `Switch` is a button-like control with `role="switch"` and a boolean checked state.
//! In Fret, this maps onto [`fret_core::SemanticsRole::Switch`], `checked: Some(bool)`, and an
//! explicit binary `checked_state`.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::PressableA11y;
use fret_ui::{ElementContext, UiHost};

/// A11y metadata for a Radix-style switch pressable.
pub fn switch_a11y(label: Option<Arc<str>>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(fret_core::SemanticsRole::Switch),
        label,
        checked: Some(checked),
        checked_state: Some(if checked {
            fret_core::SemanticsCheckedState::True
        } else {
            fret_core::SemanticsCheckedState::False
        }),
        ..Default::default()
    }
}

/// Returns a checked-state model that behaves like Radix `useControllableState` (`checked` /
/// `defaultChecked`).
pub fn switch_use_checked_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<bool>>,
    default_checked: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_checked)
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
    fn switch_use_checked_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = switch_use_checked_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                false
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    #[test]
    fn switch_a11y_sets_role_and_checked() {
        let a11y = switch_a11y(Some(Arc::from("Airplane mode")), true);
        assert_eq!(a11y.role, Some(fret_core::SemanticsRole::Switch));
        assert_eq!(a11y.checked, Some(true));
        assert_eq!(
            a11y.checked_state,
            Some(fret_core::SemanticsCheckedState::True)
        );
        assert_eq!(a11y.label.as_deref(), Some("Airplane mode"));

        let a11y = switch_a11y(None, false);
        assert_eq!(a11y.checked, Some(false));
        assert_eq!(
            a11y.checked_state,
            Some(fret_core::SemanticsCheckedState::False)
        );
    }
}
