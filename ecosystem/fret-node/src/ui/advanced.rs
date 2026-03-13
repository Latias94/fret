//! Advanced retained transport seams for node-graph UI integrations.
//!
//! These exports exist for retained-only or compatibility layers that intentionally own raw queue
//! transport. App-facing integrations should generally prefer `NodeGraphController` from
//! `crate::ui` and only drop to this module when they explicitly need queue-level bindings.

use fret_runtime::Model;

use super::controller::NodeGraphController;

pub use super::edit_queue::NodeGraphEditQueue;
pub use super::view_queue::{
    NodeGraphFitViewOptions, NodeGraphSetViewportOptions, NodeGraphViewQueue, NodeGraphViewRequest,
};
pub use super::viewport_helper::NodeGraphViewportHelper;

/// Advanced transport binding helpers for `NodeGraphController`.
///
/// Import this trait explicitly when a retained-only integration wants controller ergonomics while
/// still routing commits or viewport changes through raw transport queues.
pub trait NodeGraphControllerTransportExt: Sized {
    fn bind_edit_queue_transport(self, queue: Model<NodeGraphEditQueue>) -> Self;
    fn bind_view_queue_transport(self, queue: Model<NodeGraphViewQueue>) -> Self;
}

impl NodeGraphControllerTransportExt for NodeGraphController {
    fn bind_edit_queue_transport(self, queue: Model<NodeGraphEditQueue>) -> Self {
        NodeGraphController::bind_edit_queue_transport(self, queue)
    }

    fn bind_view_queue_transport(self, queue: Model<NodeGraphViewQueue>) -> Self {
        NodeGraphController::bind_view_queue_transport(self, queue)
    }
}
