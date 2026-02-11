//! Shared trigger a11y stamping helpers (Radix-aligned outcomes).
//!
//! Many Radix primitives stamp the same trigger relationships:
//! - `aria-expanded` (modeled as `expanded`)
//! - `aria-controls` (modeled as `controls_element`)
//!
//! In Fret, trigger nodes are typically `Pressable` or `Semantics` wrappers. These helpers keep
//! the stamping logic consistent across primitives while staying typed and non-DOM-specific
//! (see ADR 0115).

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, ElementKind, PressableProps};
use fret_ui::elements::GlobalElementId;

/// Apply `controls_element` relationship to a trigger node.
pub fn apply_trigger_controls(
    mut trigger: AnyElement,
    controls: Option<GlobalElementId>,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            a11y.controls_element = controls.map(|id| id.0);
        }
        ElementKind::Semantics(props) => {
            props.controls_element = controls.map(|id| id.0);
        }
        _ => {}
    }
    trigger
}

/// Apply `described_by_element` relationship to a trigger node.
pub fn apply_trigger_described_by(
    mut trigger: AnyElement,
    described_by: Option<GlobalElementId>,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            a11y.described_by_element = described_by.map(|id| id.0);
        }
        ElementKind::Semantics(props) => {
            props.described_by_element = described_by.map(|id| id.0);
        }
        _ => {}
    }
    trigger
}

/// Apply `expanded` and `controls_element` relationships to a trigger node.
pub fn apply_trigger_controls_expanded(
    mut trigger: AnyElement,
    expanded: Option<bool>,
    controls: Option<GlobalElementId>,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            a11y.expanded = expanded;
            a11y.controls_element = controls.map(|id| id.0);
        }
        ElementKind::Semantics(props) => {
            props.expanded = expanded;
            props.controls_element = controls.map(|id| id.0);
        }
        _ => {}
    }
    trigger
}

/// Apply common trigger semantics metadata in one place.
///
/// This is intended for primitives like `Select` that stamp a dedicated role/label in addition to
/// the expanded/controls relationships.
pub fn apply_trigger_semantics(
    mut trigger: AnyElement,
    role: Option<SemanticsRole>,
    label: Option<Arc<str>>,
    expanded: Option<bool>,
    controls: Option<GlobalElementId>,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            if let Some(role) = role {
                a11y.role = Some(role);
            }
            a11y.label = label;
            a11y.expanded = expanded;
            a11y.controls_element = controls.map(|id| id.0);
        }
        ElementKind::Semantics(props) => {
            if let Some(role) = role {
                props.role = role;
            }
            props.label = label;
            props.expanded = expanded;
            props.controls_element = controls.map(|id| id.0);
        }
        _ => {}
    }
    trigger
}
