//! Crate-internal compatibility transport for retained node-graph glue.
//!
//! These queues exist so retained-only subtrees can request graph edits without teaching them as
//! part of the app-facing `fret_node::ui::*` surface.

use crate::ops::GraphTransaction;

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeGraphEditQueue {
    pub(crate) pending: Vec<GraphTransaction>,
}

impl NodeGraphEditQueue {
    pub(crate) fn push(&mut self, tx: GraphTransaction) {
        self.pending.push(tx);
    }

    pub(crate) fn drain(&mut self) -> Vec<GraphTransaction> {
        std::mem::take(&mut self.pending)
    }
}
