//! RadioGroup primitives (Radix-aligned outcomes).
//!
//! Radix `RadioGroup` composes:
//! - a group-level semantics container, and
//! - radio button items that expose checked/disabled state.
//!
//! In Fret, roving focus + selection policy is composed by wrappers (recipe layer) using
//! `RovingFlex` + action hooks; this module provides stable, Radix-named building blocks for
//! semantics/a11y.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::{PressableA11y, SemanticsProps};
use fret_ui::{ElementContext, UiHost};

/// Semantics wrapper props for a radio group container.
pub fn radio_group_semantics(label: Option<Arc<str>>, disabled: bool) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::RadioGroup,
        label,
        disabled,
        ..Default::default()
    }
}

/// A11y metadata for a radio button-like pressable.
pub fn radio_button_a11y(label: Option<Arc<str>>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::RadioButton),
        label,
        checked: Some(checked),
        ..Default::default()
    }
}

/// Returns a selection model for a radio group that behaves like Radix `useControllableState`
/// (`value` / `defaultValue`).
pub fn radio_group_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}
