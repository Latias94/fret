//! Node graph canvas widget and editor interaction policy.
//!
//! This module is split into submodules to keep the editor-grade canvas maintainable. The retained
//! widget entry point is `NodeGraphCanvas`.

mod context_menu;
mod conversion;
mod event;
mod geometry;
mod middleware;
mod paint;
mod route_math;
mod searcher;
mod snaplines;
mod spatial;
mod state;
mod widget;
mod workflow;

pub use state::NodeResizeHandle;
pub use widget::NodeGraphCanvas;
pub use widget::NodeGraphCanvasWith;

pub(crate) use geometry::CanvasGeometry;
pub(crate) use geometry::{node_order, node_ports, node_size_default_px};
pub(crate) use spatial::CanvasSpatialDerived;

pub use middleware::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasEventOutcome,
    NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareChain, NodeGraphCanvasMiddlewareCx,
    NoopNodeGraphCanvasMiddleware, RejectInvalidSizeTx, RejectNonFiniteTx,
};
