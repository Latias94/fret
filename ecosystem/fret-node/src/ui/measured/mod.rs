//! Measured geometry storage and presenter wrapper.
//!
//! This module provides a small, UI-only mechanism to feed measured node sizes and port handle
//! bounds back into the node graph canvas without coupling the canvas to a specific layout engine.
//!
//! Conceptually this is similar to ReactFlow/XyFlow "node internals.handleBounds": the graph model
//! remains pure data, while measured sizes and handle bounds live as derived editor internals.

mod presenter;
mod store;

pub use presenter::{FallbackMeasuredNodeGraphPresenter, MeasuredNodeGraphPresenter};
pub use store::{
    MEASURED_GEOMETRY_EPSILON_PX, MeasuredGeometryApplyOptions, MeasuredGeometryBatch,
    MeasuredGeometryExclusiveBatch, MeasuredGeometryStore,
};
