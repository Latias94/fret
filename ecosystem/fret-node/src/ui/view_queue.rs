//! UI-side view command queue.
//!
//! This is a small "message passing" surface for editor subtrees or embedding apps that need to
//! request viewport actions (e.g. fit-view over a specific node set) without taking a direct
//! dependency on a particular canvas/editor widget instance.

use crate::core::NodeId;

#[derive(Debug, Clone)]
pub enum NodeGraphViewRequest {
    /// Frame a subset of nodes in view (XyFlow `fitViewOptions.nodes` mental model).
    FrameNodes { nodes: Vec<NodeId> },
}

#[derive(Debug, Default, Clone)]
pub struct NodeGraphViewQueue {
    pub pending: Vec<NodeGraphViewRequest>,
}

impl NodeGraphViewQueue {
    pub fn push(&mut self, req: NodeGraphViewRequest) {
        self.pending.push(req);
    }

    pub fn push_frame_nodes(&mut self, nodes: Vec<NodeId>) {
        self.push(NodeGraphViewRequest::FrameNodes { nodes });
    }

    pub fn drain(&mut self) -> Vec<NodeGraphViewRequest> {
        std::mem::take(&mut self.pending)
    }
}
