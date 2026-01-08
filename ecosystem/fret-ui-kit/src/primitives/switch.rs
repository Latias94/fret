//! Switch primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/switch/src/switch.tsx`
//!
//! In Radix, `Switch` is a button-like control with `role="switch"` and a boolean checked state.
//! In Fret, this maps onto [`fret_core::SemanticsRole::Switch`] and `checked: Some(bool)`.

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

/// shadcn-friendly helper for mapping optional boolean values onto a switch checked state.
///
/// Radix `Switch` is a boolean control. Some shadcn authoring patterns treat missing values as
/// "off" (`value || false`), so this helper preserves that ergonomic while keeping the core
/// primitives surface discoverable.
pub fn switch_checked_from_optional_bool(value: Option<bool>) -> bool {
    value.unwrap_or(false)
}

/// shadcn-friendly toggle policy for `Option<bool>` switch bindings.
pub fn toggle_optional_bool(value: Option<bool>) -> Option<bool> {
    Some(!switch_checked_from_optional_bool(value))
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
        assert_eq!(a11y.label.as_deref(), Some("Airplane mode"));
    }

    #[test]
    fn optional_bool_maps_to_checked_state() {
        assert_eq!(switch_checked_from_optional_bool(None), false);
        assert_eq!(switch_checked_from_optional_bool(Some(false)), false);
        assert_eq!(switch_checked_from_optional_bool(Some(true)), true);
    }

    #[test]
    fn toggle_optional_bool_inverts_and_sets_some() {
        assert_eq!(toggle_optional_bool(None), Some(true));
        assert_eq!(toggle_optional_bool(Some(false)), Some(true));
        assert_eq!(toggle_optional_bool(Some(true)), Some(false));
    }
}
