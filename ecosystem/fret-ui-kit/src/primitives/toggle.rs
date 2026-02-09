//! Toggle primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing toggle behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! This file is part of the stable primitives surface (do not move without an ADR update).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/toggle/src/toggle.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, PressableA11y, PressableProps, PressableState};
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::declarative::action_hooks::ActionHooksExt as _;

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

/// A Radix-shaped `Toggle` root configuration surface.
///
/// Upstream supports a controlled/uncontrolled pressed state (`pressed` + `defaultPressed`). In
/// Fret this maps to either:
/// - a caller-provided `Model<bool>` (controlled), or
/// - an internal `Model<bool>` stored in element state (uncontrolled).
#[derive(Debug, Clone, Default)]
pub struct ToggleRoot {
    pressed: Option<Model<bool>>,
    default_pressed: bool,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
}

impl ToggleRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `pressed` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn pressed(mut self, pressed: Option<Model<bool>>) -> Self {
        self.pressed = pressed;
        self
    }

    /// Sets the uncontrolled initial pressed value (Radix `defaultPressed`).
    pub fn default_pressed(mut self, default_pressed: bool) -> Self {
        self.default_pressed = default_pressed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Creates a toggle root with a controlled/uncontrolled pressed model (Radix `pressed` /
    /// `defaultPressed`).
    ///
    /// Notes:
    /// - The internal model (uncontrolled mode) is stored in element state at the call site.
    /// - Call this from a stable subtree (key the node if you need state to survive reordering).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        pressed: Option<Model<bool>>,
        default_pressed: impl FnOnce() -> bool,
    ) -> Self {
        let model = toggle_use_model(cx, pressed, default_pressed).model();
        Self::new().pressed(Some(model))
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `pressed`.
    pub fn use_pressed_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        toggle_use_model(cx, self.pressed.clone(), || self.default_pressed)
    }

    pub fn pressed_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_pressed_model(cx).model()
    }

    /// Reads the current pressed value from the derived pressed model.
    pub fn is_pressed<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let model = self.pressed_model(cx);
        cx.watch_model(&model).copied_or_default()
    }

    /// Renders a toggle-like pressable, wiring Radix-like pressed state and a11y.
    ///
    /// Notes:
    /// - Activation toggles the boolean model (disabled guards apply).
    /// - This does not apply any visual skin. Pass the desired `PressableProps`.
    #[track_caller]
    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        mut props: PressableProps,
        f: impl FnOnce(&mut ElementContext<'_, H>, PressableState, bool) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let model = self.pressed_model(cx);
        let disabled = self.disabled;
        let label = self.a11y_label.clone();

        cx.pressable_with_id_props(move |cx, st, _id| {
            if !disabled {
                cx.pressable_toggle_bool(&model);
            }

            let pressed = cx.watch_model(&model).copied_or_default();
            props.enabled = props.enabled && !disabled;
            props.a11y = toggle_a11y(label.clone(), pressed);

            (props, f(cx, st, pressed))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{AppWindowId, Px, Rect};

    fn bounds() -> Rect {
        Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn toggle_root_prefers_controlled_model_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = ToggleRoot::new_controllable(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                false
            });
            assert_eq!(root.pressed_model(cx), controlled);
        });

        assert_eq!(called.get(), 0);
    }
}
