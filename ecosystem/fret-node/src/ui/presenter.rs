use std::sync::Arc;

use fret_core::Color;

use crate::core::{EdgeId, Graph, NodeId, PortId, PortKind};
use crate::ops::GraphOp;
use crate::rules::{ConnectPlan, EdgeEndpoint, plan_connect, plan_reconnect_edge};

use super::style::NodeGraphStyle;

/// Context menu actions surfaced by the canvas widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphContextMenuAction {
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
