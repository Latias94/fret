//! UI-side edit transaction queue.
//!
//! This is a small "message passing" surface for editor subtrees that need to request graph edits
//! (e.g. node parameter widgets) without taking a direct dependency on a particular canvas/editor
//! widget instance.
//!
//! The intended flow is:
//! - UI controls push `GraphTransaction`s into a `Model<NodeGraphEditQueue>`.
//! - The node graph canvas/editor drains the queue and commits transactions into its own history
//!   (undo/redo), applying profile validation/concretization as needed.

use crate::ops::GraphTransaction;

#[derive(Debug, Default, Clone)]
pub struct NodeGraphEditQueue {
    pub pending: Vec<GraphTransaction>,
}

impl NodeGraphEditQueue {
    pub fn push(&mut self, tx: GraphTransaction) {
        self.pending.push(tx);
    }

    pub fn drain(&mut self) -> Vec<GraphTransaction> {
        std::mem::take(&mut self.pending)
    }
}
