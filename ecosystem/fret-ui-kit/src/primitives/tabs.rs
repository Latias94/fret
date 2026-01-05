//! Tabs primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing tabs behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/tabs/src/tabs.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;

/// Matches Radix Tabs `orientation` outcome: horizontal (default) vs vertical layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Matches Radix Tabs `activationMode` outcome:
/// - `Automatic`: moving focus (arrow keys) activates the tab.
/// - `Manual`: moving focus does not activate; activation happens on click/Enter/Space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsActivationMode {
    #[default]
    Automatic,
    Manual,
}

/// A11y metadata for a tab-like pressable.
pub fn tab_a11y(label: Option<Arc<str>>, selected: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Tab),
        label,
        selected,
        ..Default::default()
    }
}

/// Maps a selected `value` (string key) to the active index, skipping disabled items.
///
/// This is the Radix outcome "value controls which trigger is active", expressed in Fret terms.
pub fn active_index_from_values(
    values: &[Arc<str>],
    selected: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    crate::headless::roving_focus::active_index_from_str_keys(values, selected, disabled)
}
