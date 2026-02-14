//! GenUI core: json-render-inspired, guardrailed UI spec rendering for Fret.
//!
//! This crate is ecosystem-only and must not introduce policy into `crates/fret-ui`.

pub mod json_pointer;
pub mod props;
pub mod render;
pub mod spec;
pub mod validate;
pub mod visibility;
