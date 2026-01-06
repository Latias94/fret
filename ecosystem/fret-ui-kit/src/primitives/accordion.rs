//! Accordion primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing accordion behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/accordion/src/accordion.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;

/// Matches Radix Accordion `type` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionKind {
    Single,
    Multiple,
}

/// A11y metadata for an accordion trigger.
pub fn accordion_trigger_a11y(label: Arc<str>, open: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label: Some(label),
        expanded: Some(open),
        ..Default::default()
    }
}

/// Derive the "tab stop" index for a single-select accordion:
/// the open enabled item, or the first enabled item.
pub fn tab_stop_index_single(
    values: &[Arc<str>],
    open: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    if let Some(open) = open {
        if let Some(active) =
            crate::headless::roving_focus::active_index_from_str_keys(values, Some(open), disabled)
        {
            return Some(active);
        }
    }
    crate::headless::roving_focus::first_enabled(disabled)
}

/// Derive the "tab stop" index for a multi-select accordion:
/// the first open+enabled item, or the first enabled item.
pub fn tab_stop_index_multiple(
    values: &[Arc<str>],
    open: &[Arc<str>],
    disabled: &[bool],
) -> Option<usize> {
    let first_open_enabled = values.iter().enumerate().find_map(|(idx, v)| {
        let enabled = !disabled.get(idx).copied().unwrap_or(true);
        let is_open = open.iter().any(|s| s.as_ref() == v.as_ref());
        (enabled && is_open).then_some(idx)
    });
    first_open_enabled.or_else(|| crate::headless::roving_focus::first_enabled(disabled))
}
