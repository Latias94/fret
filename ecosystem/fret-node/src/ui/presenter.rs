use std::sync::Arc;

use fret_core::Color;
use serde_json::Value;

use crate::REROUTE_KIND;
use crate::core::{
    CanvasPoint, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};
use crate::ops::GraphOp;
use crate::rules::{
    ConnectPlan, EdgeEndpoint, InsertNodeSpec, plan_connect, plan_reconnect_edge,
    plan_split_edge_by_inserting_node,
};

use super::style::NodeGraphStyle;

/// Context menu actions surfaced by the canvas widget.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeGraphContextMenuAction {
    OpenInsertNodePicker,
    InsertNode(NodeKindKey),
    InsertReroute,
    DeleteEdge,
    Custom(u64),
}

/// A context menu item.
#[derive(Debug, Clone)]
pub struct NodeGraphContextMenuItem {
    pub label: Arc<str>,
    pub enabled: bool,
    pub action: NodeGraphContextMenuAction,
}

/// A candidate node kind for insertion.
#[derive(Debug, Clone)]
pub struct InsertNodeCandidate {
    pub kind: NodeKindKey,
    pub label: Arc<str>,
    pub enabled: bool,
}

/// Viewer/presenter surface for the node graph UI.
///
/// This is the primary extensibility point: domain code can define titles, styles, and connection
/// behavior without forking the editor widget.
pub trait NodeGraphPresenter {
    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str>;
    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str>;

    fn port_color(&self, graph: &Graph, port: PortId, style: &NodeGraphStyle) -> Color {
        let Some(p) = graph.ports.get(&port) else {
            return style.node_border;
        };
        match p.kind {
            PortKind::Data => style.pin_color_data,
            PortKind::Exec => style.pin_color_exec,
        }
    }

    fn edge_color(&self, graph: &Graph, edge: EdgeId, style: &NodeGraphStyle) -> Color {
        let Some(e) = graph.edges.get(&edge) else {
            return style.node_border;
        };
        match e.kind {
            crate::core::EdgeKind::Data => style.wire_color_data,
            crate::core::EdgeKind::Exec => style.wire_color_exec,
        }
    }

    /// Lists insertable nodes for split-edge workflows.
    ///
    /// Implementations may inspect the edge type and port types to return compatible candidates.
    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        let _ = (graph, edge);
        Vec::new()
    }

    /// Plans splitting an edge by inserting a node.
    ///
    /// Returning a rejected plan will surface diagnostics to the user.
    fn plan_split_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        node_kind: &NodeKindKey,
        at: CanvasPoint,
    ) -> ConnectPlan {
        if node_kind.0 != REROUTE_KIND {
            let _ = (graph, edge, node_kind, at);
            return ConnectPlan::reject("split-edge insertion is not supported");
        }

        let edge_id = edge;

        let Some(edge) = graph.edges.get(&edge_id) else {
            return ConnectPlan::reject("missing edge");
        };
        let Some(from_port) = graph.ports.get(&edge.from) else {
            return ConnectPlan::reject("missing edge.from port");
        };
        let Some(to_port) = graph.ports.get(&edge.to) else {
            return ConnectPlan::reject("missing edge.to port");
        };

        let port_kind = match edge.kind {
            EdgeKind::Data => PortKind::Data,
            EdgeKind::Exec => PortKind::Exec,
        };
        let ty = from_port.ty.clone().or_else(|| to_port.ty.clone());

        let node_id = NodeId::new();
        let in_port_id = PortId::new();
        let out_port_id = PortId::new();

        let node = Node {
            kind: node_kind.clone(),
            kind_version: 1,
            pos: at,
            collapsed: false,
            ports: Vec::new(),
            data: Value::default(),
        };

        let in_port = Port {
            node: node_id,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: port_kind,
            capacity: PortCapacity::Single,
            ty: ty.clone(),
            data: Value::default(),
        };

        let out_port = Port {
            node: node_id,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: port_kind,
            capacity: PortCapacity::Multi,
            ty,
            data: Value::default(),
        };

        plan_split_edge_by_inserting_node(
            graph,
            edge_id,
            EdgeId::new(),
            InsertNodeSpec {
                node_id,
                node,
                ports: vec![(in_port_id, in_port), (out_port_id, out_port)],
                input: in_port_id,
                output: out_port_id,
            },
        )
    }

    /// Fills the right-click context menu for an edge.
    ///
    /// The canvas will append built-in actions (e.g. `Delete`) after these items.
    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        _style: &NodeGraphStyle,
        _out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        let _ = (graph, edge);
    }

    /// Handles a custom context menu action.
    ///
    /// Returning `Some(ops)` applies them as a single transaction.
    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<GraphOp>> {
        let _ = (graph, edge, action);
        None
    }

    /// Connection decision point.
    ///
    /// Implementations may return a `ConnectPlan` that:
    /// - rejects the connection (diagnostics only),
    /// - accepts it with direct edge changes,
    /// - accepts it with additional ops (e.g. insert conversion nodes).
    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        plan_connect(graph, a, b)
    }

    /// Reconnection decision point (preserve edge identity when possible).
    fn plan_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
    ) -> ConnectPlan {
        plan_reconnect_edge(graph, edge, endpoint, new_port)
    }
}

/// Default presenter used by the canvas widget when no domain presenter is provided.
#[derive(Debug, Default, Clone)]
pub struct DefaultNodeGraphPresenter;

impl NodeGraphPresenter for DefaultNodeGraphPresenter {
    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        graph
            .nodes
            .get(&node)
            .map(|n| Arc::<str>::from(n.kind.0.clone()))
            .unwrap_or_else(|| Arc::<str>::from("<missing node>"))
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        graph
            .ports
            .get(&port)
            .map(|p| Arc::<str>::from(p.key.0.clone()))
            .unwrap_or_else(|| Arc::<str>::from("<missing port>"))
    }
}
