//! Collapsible primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing collapsible behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/collapsible/src/collapsible.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{PressableA11y, SemanticsProps};

/// Semantics wrapper props for a collapsible root container.
pub fn collapsible_root_semantics(disabled: bool, open: bool) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::Generic,
        disabled,
        expanded: Some(open),
        ..Default::default()
    }
}

/// A11y metadata for a collapsible trigger pressable.
pub fn collapsible_trigger_a11y(label: Option<Arc<str>>, open: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label,
        expanded: Some(open),
        ..Default::default()
    }
}
