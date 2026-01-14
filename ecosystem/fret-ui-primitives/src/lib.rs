//! UI-level primitives and wiring helpers.
//!
//! This crate hosts Radix-aligned primitives that depend on `fret-ui` runtime types
//! (`ElementContext`, `UiTree`, `ScrollHandle`, focus traversal, etc).
//!
//! Pure logic/state machines belong in `fret-ui-headless`.

pub mod focus_scope;
pub mod scroll_area;
pub mod scroll_area_visibility;
