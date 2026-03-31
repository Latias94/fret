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

/// Advanced helper that binds a controller to a raw edit-queue transport.
///
/// This stays under `ui::advanced` on purpose: normal app-facing composition should dispatch
/// directly through the controller/store path instead of teaching queue-owned transport.
pub fn bind_controller_edit_queue_transport(
    controller: NodeGraphController,
    queue: Model<NodeGraphEditQueue>,
) -> NodeGraphController {
    controller.bind_edit_queue_transport(queue)
}

/// Advanced helper that binds a controller to a raw view-queue transport.
///
/// This keeps queue-owned viewport transport explicit without requiring a public extension trait on
/// the controller surface.
pub fn bind_controller_view_queue_transport(
    controller: NodeGraphController,
    queue: Model<NodeGraphViewQueue>,
) -> NodeGraphController {
    controller.bind_view_queue_transport(queue)
}
