//! Material3 foundation utilities.
//!
//! This module exists to reduce per-component divergence by centralizing:
//! - token resolution conventions (Material-only fallbacks),
//! - interaction-driven ink (state layer + ripple),
//! - tree-local overrides (content defaults, ripple configuration),
//! - shared geometry helpers (ripple origin / max radius),
//! - shared focus-ring style defaults.

pub mod action;
pub mod active_indicator;
pub mod arc_str;
pub mod content;
pub mod context;
pub mod elevation;
pub mod field;
pub mod field_motion;
pub mod floating_label;
pub mod focus_ring;
pub mod geometry;
pub mod icon;
pub mod indication;
pub mod interaction;
pub mod interactive_size;
pub mod layout_probe;
pub mod modal_motion;
pub mod motion_scheme;
pub mod overlay_motion;
pub mod search_motion;
pub mod strings;
pub mod surface;
pub mod test_id;
pub mod token_resolver;
pub mod tokens;
