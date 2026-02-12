//! UI integration helpers for `fret-canvas`.
//!
//! This module is feature-gated (`fret-canvas/ui`) so the default crate remains portable and
//! UI-framework-agnostic. When enabled, it provides declarative wiring and recipe-like helpers
//! for common canvas interaction patterns (pan/zoom, tool routing), built on top of:
//! - `crates/fret-ui` mechanisms (Canvas, PointerRegion, action hooks),
//! - `ecosystem/fret-canvas` substrate (PanZoom2D, scale helpers, geometry).

mod canvas_surface;
mod controllable_state;
mod input_exempt;
mod pan_zoom;
mod tool_router;

pub use canvas_surface::*;
pub use controllable_state::*;
pub use input_exempt::*;
pub use pan_zoom::*;
pub use tool_router::*;
