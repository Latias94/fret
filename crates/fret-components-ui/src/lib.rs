//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).

mod style;

pub mod button;
pub mod frame;
pub mod text_field;

pub use style::{ColorRef, MetricRef, StyleRefinement};
