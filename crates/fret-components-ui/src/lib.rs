//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).

mod style;

pub mod button;
pub mod checkbox;
pub mod dropdown_menu;
pub mod frame;
pub mod icon_button;
pub mod progress;
pub mod select;
pub mod separator;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod toolbar;
pub mod tooltip;

pub use style::{ColorRef, MetricRef, StyleRefinement};
