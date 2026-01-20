//! UI-side view command queue.
//!
//! This is a small "message passing" surface for editor subtrees or embedding apps that need to
//! request viewport actions (e.g. fit-view over a specific node set) without taking a direct
//! dependency on a particular canvas/editor widget instance.

use crate::core::NodeId;
use crate::io::{NodeGraphViewportEase, NodeGraphViewportInterpolate};

#[derive(Debug, Clone)]
pub enum NodeGraphViewRequest {
    /// Frame a subset of nodes in view (XyFlow `fitViewOptions.nodes` mental model).
    FrameNodes {
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    },
}

#[derive(Debug, Default, Clone)]
pub struct NodeGraphFitViewOptions {
    /// Include hidden nodes (XyFlow `fitViewOptions.includeHiddenNodes`).
    pub include_hidden_nodes: bool,
    /// Optional per-call zoom clamp override (XyFlow `fitViewOptions.minZoom`).
    pub min_zoom: Option<f32>,
    /// Optional per-call zoom clamp override (XyFlow `fitViewOptions.maxZoom`).
    pub max_zoom: Option<f32>,
    /// Optional per-call animation duration override (XyFlow `fitViewOptions.duration`).
    pub duration_ms: Option<u32>,
    /// Optional per-call interpolate override (XyFlow `fitViewOptions.interpolate`).
    pub interpolate: Option<NodeGraphViewportInterpolate>,
    /// Optional per-call ease override (XyFlow `fitViewOptions.ease`).
    pub ease: Option<NodeGraphViewportEase>,
    /// Optional per-call padding override (XyFlow `fitViewOptions.padding`).
    pub padding: Option<f32>,
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
        self.push(NodeGraphViewRequest::FrameNodes {
            nodes,
            options: NodeGraphFitViewOptions::default(),
        });
    }

    pub fn push_frame_nodes_with_options(
        &mut self,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) {
        self.push(NodeGraphViewRequest::FrameNodes { nodes, options });
    }

    pub fn drain(&mut self) -> Vec<NodeGraphViewRequest> {
        std::mem::take(&mut self.pending)
    }
}
