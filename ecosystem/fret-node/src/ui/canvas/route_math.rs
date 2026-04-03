//! Shared edge routing math for the node graph canvas.
//!
//! This module intentionally contains only pure geometry helpers (no UI state).

use fret_canvas::wires as canvas_wires;
use fret_core::{Point, Px};

use crate::ui::presenter::EdgeRouteKind;

mod route_math_curve;
mod route_math_tangent;

#[cfg(feature = "compat-retained-canvas")]
pub(crate) use route_math_curve::{cubic_bezier, normal_from_tangent};
pub(crate) use route_math_curve::{cubic_bezier_derivative, wire_ctrl_points};
#[cfg(feature = "compat-retained-canvas")]
pub(crate) use route_math_tangent::{edge_route_end_tangent, edge_route_start_tangent};
