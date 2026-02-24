//! ToggleGroup primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing toggle group behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/toggle-group/src/toggle-group.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::PressableA11y;
use fret_ui::{ElementContext, UiHost};

/// Matches Radix ToggleGroup `type` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleGroupKind {
    Single,
    Multiple,
}

/// Matches Radix ToggleGroup `orientation` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A11y metadata for a toggle-group item.
///
/// Radix uses `aria-pressed` in multiple mode and `role="radio" + aria-checked` in single mode.
/// Fret models this by switching the item role and using `pressed_state` for multiple mode and
/// the `checked` flag for single mode.
pub fn toggle_group_item_a11y_multiple(label: Arc<str>, pressed: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label: Some(label),
        pressed_state: Some(if pressed {
            fret_core::SemanticsPressedState::True
        } else {
            fret_core::SemanticsPressedState::False
        }),
        ..Default::default()
    }
}

/// A11y metadata for a single-select toggle-group item (Radix `role="radio"`).
pub fn toggle_group_item_a11y_single(label: Arc<str>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::RadioButton),
        label: Some(label),
        checked: Some(checked),
        ..Default::default()
    }
}

/// Back-compat shim: treated as the multiple-select button-like outcome.
pub fn toggle_group_item_a11y(label: Arc<str>, pressed: bool) -> PressableA11y {
    toggle_group_item_a11y_multiple(label, pressed)
}

/// Returns a selection model for a single-select toggle group that behaves like Radix
/// `useControllableState` (`value` / `defaultValue`).
pub fn toggle_group_use_single_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

/// Returns a selection model for a multi-select toggle group that behaves like Radix
/// `useControllableState` (`value` / `defaultValue`).
pub fn toggle_group_use_multiple_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Vec<Arc<str>>>>,
    default_value: impl FnOnce() -> Vec<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Vec<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

/// Derive the "tab stop" index for a single-select toggle group:
/// the selected enabled item, or the first enabled item.
pub fn tab_stop_index_single(
    values: &[Arc<str>],
    selected: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    if let Some(selected) = selected {
        if let Some(active) = crate::headless::roving_focus::active_index_from_str_keys(
            values,
            Some(selected),
            disabled,
        ) {
            return Some(active);
        }
    }
    crate::headless::roving_focus::first_enabled(disabled)
}

/// Derive the "tab stop" index for a multi-select toggle group:
/// the first selected+enabled item, or the first enabled item.
pub fn tab_stop_index_multiple(
    values: &[Arc<str>],
    selected: &[Arc<str>],
    disabled: &[bool],
) -> Option<usize> {
    let first_selected_enabled = values.iter().enumerate().find_map(|(idx, v)| {
        let enabled = !disabled.get(idx).copied().unwrap_or(true);
        let on = selected.iter().any(|s| s.as_ref() == v.as_ref());
        (enabled && on).then_some(idx)
    });
    first_selected_enabled.or_else(|| crate::headless::roving_focus::first_enabled(disabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_group_item_a11y_single_uses_radio_role_and_checked() {
        let a11y = toggle_group_item_a11y_single(Arc::from("A"), true);
        assert_eq!(a11y.role, Some(SemanticsRole::RadioButton));
        assert_eq!(a11y.checked, Some(true));
        assert!(!a11y.selected);
    }

    #[test]
    fn toggle_group_item_a11y_multiple_uses_button_role_and_pressed_state() {
        let a11y = toggle_group_item_a11y_multiple(Arc::from("A"), true);
        assert_eq!(a11y.role, Some(SemanticsRole::Button));
        assert_eq!(
            a11y.pressed_state,
            Some(fret_core::SemanticsPressedState::True)
        );
        assert_eq!(a11y.checked, None);
    }
}
