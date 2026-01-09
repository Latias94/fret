//! Menu item helpers (Radix-aligned outcomes).
//!
//! This module provides small building blocks that help wrappers stamp consistent menu item
//! semantics without duplicating boilerplate.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;
use fret_ui::elements::GlobalElementId;

/// Build default a11y metadata for a menu item-like pressable.
pub fn menu_item_a11y(label: Option<Arc<str>>, expanded: Option<bool>) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::MenuItem),
        label,
        expanded,
        ..Default::default()
    }
}

/// Build default a11y metadata for a menu item-like pressable, including an optional `controls`
/// relationship (Radix `aria-controls` outcome).
pub fn menu_item_a11y_with_controls(
    label: Option<Arc<str>>,
    expanded: Option<bool>,
    controls_element: Option<GlobalElementId>,
) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::MenuItem),
        label,
        expanded,
        controls_element: controls_element.map(|id| id.0),
        ..Default::default()
    }
}

/// Build a11y metadata for a checkbox-style menu item pressable.
pub fn menu_item_checkbox_a11y(label: Option<Arc<str>>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::MenuItemCheckbox),
        label,
        checked: Some(checked),
        ..Default::default()
    }
}

/// Build a11y metadata for a radio-style menu item pressable.
pub fn menu_item_radio_a11y(label: Option<Arc<str>>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::MenuItemRadio),
        label,
        checked: Some(checked),
        ..Default::default()
    }
}
