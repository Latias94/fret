//! AccessKit adapter for Fret semantics snapshots.
//!
//! This crate converts `fret-core`'s portable semantics tree (`SemanticsSnapshot`) into
//! AccessKit's `TreeUpdate` representation.
//!
//! Backend-specific wiring (e.g. hooking this into a windowing system) lives in runner crates such
//! as `fret-runner-winit`.

mod actions;
mod ids;
mod mapping;
mod roles;

pub use actions::{
    ScrollByData, SetTextSelectionData, SetValueData, StepperAction, focus_target_from_action,
    invoke_target_from_action, replace_selected_text_from_action, scroll_by_from_action,
    set_text_selection_from_action, set_value_from_action, stepper_target_from_action,
};
pub use mapping::tree_update_from_snapshot;

#[cfg(test)]
mod tests;
