//! Menu item helpers (Radix-aligned outcomes).
//!
//! This module provides small building blocks that help wrappers stamp consistent menu item
//! semantics without duplicating boilerplate.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;

/// Build default a11y metadata for a menu item-like pressable.
pub fn menu_item_a11y(label: Option<Arc<str>>, expanded: Option<bool>) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::MenuItem),
        label,
        expanded,
        ..Default::default()
    }
}
