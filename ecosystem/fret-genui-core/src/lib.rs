//! GenUI core: json-render-inspired, guardrailed UI spec rendering for Fret.
//!
//! This crate is ecosystem-only and must not introduce policy into `crates/fret-ui`.

pub mod actions;
pub mod catalog;
pub mod executor;
pub mod form_validation;
pub mod json_pointer;
pub mod mixed_stream;
pub mod prompt;
pub mod props;
pub mod render;
pub mod spec;
pub mod spec_fixer;
pub mod spec_stream;
pub mod validate;
pub mod validation_checks;
pub mod visibility;
