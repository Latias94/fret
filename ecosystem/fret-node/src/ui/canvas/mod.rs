//! Node graph canvas widget and editor interaction policy.
//!
//! This module is split into submodules to keep the editor-grade canvas maintainable. The retained
//! widget entry point is `NodeGraphCanvas`.

mod geometry;
mod resize_handle;
mod route_math;
mod spatial;

#[cfg(feature = "compat-retained-canvas")]
mod context_menu;
#[cfg(feature = "compat-retained-canvas")]
mod conversion;
#[cfg(feature = "compat-retained-canvas")]
mod event;
#[cfg(feature = "compat-retained-canvas")]
mod middleware;
#[cfg(feature = "compat-retained-canvas")]
mod paint;
#[cfg(feature = "compat-retained-canvas")]
mod searcher;
#[cfg(feature = "compat-retained-canvas")]
mod snaplines;
#[cfg(feature = "compat-retained-canvas")]
mod state;
#[cfg(feature = "compat-retained-canvas")]
mod widget;
#[cfg(feature = "compat-retained-canvas")]
mod workflow;

pub use resize_handle::NodeResizeHandle;
#[cfg(feature = "compat-retained-canvas")]
pub use widget::NodeGraphCanvas;
#[cfg(feature = "compat-retained-canvas")]
pub use widget::NodeGraphCanvasWith;

pub(crate) use geometry::CanvasGeometry;
pub(crate) use geometry::{node_ports, node_size_default_px};
pub(crate) use spatial::CanvasSpatialDerived;

#[cfg(feature = "compat-retained-canvas")]
pub use middleware::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasEventOutcome,
    NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareChain, NodeGraphCanvasMiddlewareCx,
    NoopNodeGraphCanvasMiddleware, RejectInvalidSizeTx, RejectNonFiniteTx,
};
