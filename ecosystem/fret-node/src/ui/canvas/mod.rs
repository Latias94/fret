//! Node graph canvas widget and editor interaction policy.
//!
//! This module is split into submodules to keep the editor-grade canvas maintainable. The retained
//! widget entry point is `NodeGraphCanvas`.

#[cfg(feature = "compat-retained-canvas")]
mod conversion;
mod geometry;
#[cfg(feature = "compat-retained-canvas")]
mod middleware;
#[cfg(feature = "compat-retained-canvas")]
mod paint;
mod resize_handle;
mod route_math;
#[cfg(feature = "compat-retained-canvas")]
mod searcher;
#[cfg(feature = "compat-retained-canvas")]
mod snaplines;
mod spatial;
#[cfg(feature = "compat-retained-canvas")]
mod state;
#[cfg(feature = "compat-retained-canvas")]
mod widget;
#[cfg(feature = "compat-retained-canvas")]
mod workflow;

#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use middleware::{
    NodeGraphCanvasCommitOutcome, NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareCx,
};
pub use resize_handle::NodeResizeHandle;
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use widget::NodeGraphCanvas;

pub(crate) use geometry::CanvasGeometry;
pub(crate) use geometry::{node_ports, node_size_default_px};
pub(crate) use spatial::CanvasSpatialDerived;
