use std::collections::BTreeMap;

use fret_core::{Point, Rect};

use crate::core::{EdgeId, NodeId, PortId};

use super::NodeGraphCanvasTransform;

#[derive(Debug, Clone, Default)]
pub struct NodeGraphInternalsSnapshot {
    pub transform: NodeGraphCanvasTransform,
    pub nodes_window: BTreeMap<NodeId, Rect>,
    pub ports_window: BTreeMap<PortId, Rect>,
    pub port_centers_window: BTreeMap<PortId, Point>,
    pub edge_centers_window: BTreeMap<EdgeId, Point>,
    /// Optional human-readable label for the currently active descendant (a11y support).
    ///
    /// This is an editor-derived surface and must not be serialized into graph assets.
    pub a11y_active_descendant_label: Option<String>,
    pub a11y_focused_node_label: Option<String>,
    pub a11y_focused_port_label: Option<String>,
    pub a11y_focused_edge_label: Option<String>,
    pub focused_node: Option<NodeId>,
    pub focused_port: Option<PortId>,
    pub focused_edge: Option<EdgeId>,
    pub connecting: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NodeGraphA11ySnapshot {
    pub active_descendant_label: Option<String>,
    pub focused_node_label: Option<String>,
    pub focused_port_label: Option<String>,
    pub focused_edge_label: Option<String>,
    pub focused_node: Option<NodeId>,
    pub focused_port: Option<PortId>,
    pub focused_edge: Option<EdgeId>,
    pub connecting: bool,
}
