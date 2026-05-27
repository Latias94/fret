//! Node graph canvas geometry and editor interaction support.
//!
//! This module keeps pure geometry, route math, spatial lookup, and small UI-facing helper types for
//! the declarative node graph surface.

mod geometry;
mod resize_handle;
mod route_math;
mod spatial;

pub use resize_handle::{NodeResizeHandle, NodeResizeHandleSet};

pub(crate) use geometry::CanvasGeometry;
pub(crate) use geometry::{node_ports, node_size_default_px};
pub(crate) use spatial::CanvasSpatialDerived;
