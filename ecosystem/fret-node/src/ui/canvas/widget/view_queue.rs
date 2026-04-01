//! Retained-canvas-local view command queue.
//!
//! This is a crate-internal compatibility transport for the retained canvas stack. Public
//! controller/binding helpers use the smaller store-first option types from `viewport_options.rs`.

use crate::core::CanvasPoint;
use crate::core::NodeId;
use crate::io::{NodeGraphViewportEase, NodeGraphViewportInterpolate};
use crate::ui::viewport_options::{
    NodeGraphFitViewOptions as PublicNodeGraphFitViewOptions,
    NodeGraphSetViewportOptions as PublicNodeGraphSetViewportOptions,
};

#[derive(Debug, Clone)]
pub(crate) enum NodeGraphViewRequest {
    /// Frame a subset of nodes in view (XyFlow `fitViewOptions.nodes` mental model).
    FrameNodes {
        nodes: Vec<NodeId>,
        options: NodeGraphViewQueueFitViewOptions,
    },
    /// Sets the viewport pan/zoom (XyFlow `setViewport` mental model).
    SetViewport {
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphViewQueueSetViewportOptions,
    },
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct NodeGraphViewQueueFitViewOptions {
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

impl From<PublicNodeGraphFitViewOptions> for NodeGraphViewQueueFitViewOptions {
    fn from(options: PublicNodeGraphFitViewOptions) -> Self {
        Self {
            include_hidden_nodes: options.include_hidden_nodes,
            min_zoom: options.min_zoom,
            max_zoom: options.max_zoom,
            duration_ms: None,
            interpolate: None,
            ease: None,
            padding: options.padding,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct NodeGraphViewQueueSetViewportOptions {
    /// Optional per-call zoom clamp override (XyFlow `setViewport({ zoom }, { duration })`).
    pub min_zoom: Option<f32>,
    /// Optional per-call zoom clamp override (XyFlow `setViewport({ zoom }, { duration })`).
    pub max_zoom: Option<f32>,
    /// Optional per-call animation duration override (XyFlow `setViewport(..., { duration })`).
    pub duration_ms: Option<u32>,
    /// Optional per-call interpolate override.
    pub interpolate: Option<NodeGraphViewportInterpolate>,
    /// Optional per-call ease override.
    pub ease: Option<NodeGraphViewportEase>,
}

impl From<PublicNodeGraphSetViewportOptions> for NodeGraphViewQueueSetViewportOptions {
    fn from(options: PublicNodeGraphSetViewportOptions) -> Self {
        Self {
            min_zoom: options.min_zoom,
            max_zoom: options.max_zoom,
            duration_ms: None,
            interpolate: None,
            ease: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeGraphViewQueue {
    pub pending: Vec<NodeGraphViewRequest>,
}

impl NodeGraphViewQueue {
    pub(crate) fn push(&mut self, req: NodeGraphViewRequest) {
        self.pending.push(req);
    }

    pub(crate) fn push_frame_nodes(&mut self, nodes: Vec<NodeId>) {
        self.push(NodeGraphViewRequest::FrameNodes {
            nodes,
            options: NodeGraphViewQueueFitViewOptions::default(),
        });
    }

    pub(crate) fn push_frame_nodes_with_options(
        &mut self,
        nodes: Vec<NodeId>,
        options: NodeGraphViewQueueFitViewOptions,
    ) {
        self.push(NodeGraphViewRequest::FrameNodes { nodes, options });
    }

    pub(crate) fn push_set_viewport(&mut self, pan: CanvasPoint, zoom: f32) {
        self.push(NodeGraphViewRequest::SetViewport {
            pan,
            zoom,
            options: NodeGraphViewQueueSetViewportOptions::default(),
        });
    }

    pub(crate) fn push_set_viewport_with_options(
        &mut self,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphViewQueueSetViewportOptions,
    ) {
        self.push(NodeGraphViewRequest::SetViewport { pan, zoom, options });
    }

    pub(crate) fn drain(&mut self) -> Vec<NodeGraphViewRequest> {
        std::mem::take(&mut self.pending)
    }
}
