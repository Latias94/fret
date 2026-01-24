//! Shared canvas substrate helpers for ecosystem widgets.
//!
//! This crate is intentionally policy-light:
//! - It provides reusable math/state helpers (pan/zoom transforms, drag phases, pixel policies).
//! - It does not prescribe interaction maps, snapping rules, or domain models.
//!
//! ADR reference: `docs/adr/0137-canvas-widgets-and-interactive-surfaces.md`.

#![forbid(unsafe_code)]

pub mod budget;
pub mod cache;
pub mod diagnostics;
pub mod drag;
pub mod interaction;
pub mod scale;
pub mod spatial;
#[cfg(feature = "rstar")]
pub mod spatial_rstar;
pub mod text;
pub mod view;
pub mod wires;

#[cfg(feature = "declarative")]
pub mod declarative;

#[cfg(feature = "ui")]
pub mod ui;
