//! Material3 foundation utilities.
//!
//! This module exists to reduce per-component divergence by centralizing:
//! - token resolution conventions (Material-only fallbacks),
//! - interaction-driven ink (state layer + ripple),
//! - tree-local overrides (content defaults, ripple configuration),
//! - shared geometry helpers (ripple origin / max radius),
//! - shared focus-ring style defaults.

pub mod content;
pub mod context;
pub mod elevation;
pub mod floating_label;
pub mod focus_ring;
pub mod geometry;
pub mod indication;
pub mod interaction;
pub mod interactive_size;
pub mod layout_probe;
pub mod motion_scheme;
pub mod overlay_motion;
pub mod surface;
pub mod token_resolver;
pub mod tokens;
