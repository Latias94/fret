//! Advanced retained transport seams for node-graph UI integrations.
//!
//! These exports exist for retained-only or compatibility layers that intentionally own raw queue
//! transport. App-facing integrations should generally prefer `NodeGraphController` from
//! `crate::ui` and only drop to this module when they explicitly need queue-level bindings.

use fret_runtime::Model;

use super::controller::NodeGraphController;

pub use super::edit_queue::NodeGraphEditQueue;

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
