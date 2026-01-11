//! B-layer edge view registry (ReactFlow-style `edgeTypes`).
//!
//! `fret-node` keeps the serialized `Graph` free of UI policy. Edge view customizations live in
//! the UI integration layer and are applied as *render-hint overrides* on top of the presenter's
//! baseline hints.
//!
//! Stage 1 (this module): `edgeTypes` can only override [`crate::ui::presenter::EdgeRenderHint`]
//! (routing, label, markers, widths, colors). This covers most ReactFlow/ShaderGraph use-cases
//! and keeps hit-testing consistent by sharing the same hint source.
//!
//! Future (Stage 2): allow optional custom path builders / painters for more advanced visuals
//! (ECharts-like styling), without changing the serialized graph model.

use std::collections::BTreeMap;

use crate::core::{EdgeId, EdgeKind, Graph};
use crate::ui::presenter::EdgeRenderHint;
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeTypeKey(pub String);

impl EdgeTypeKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

pub type EdgeTypeResolver = dyn Fn(&Graph, EdgeId) -> EdgeTypeKey + 'static;

pub type EdgeTypeStyler =
    dyn Fn(&Graph, EdgeId, &NodeGraphStyle, EdgeRenderHint) -> EdgeRenderHint + 'static;

pub struct NodeGraphEdgeTypes {
    resolver: Box<EdgeTypeResolver>,
    edge_types: BTreeMap<EdgeTypeKey, Box<EdgeTypeStyler>>,
    fallback: Option<Box<EdgeTypeStyler>>,
}

impl Default for NodeGraphEdgeTypes {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeGraphEdgeTypes {
    pub fn new() -> Self {
        Self {
            resolver: Box::new(default_edge_type_resolver),
            edge_types: BTreeMap::new(),
            fallback: None,
        }
    }

    pub fn with_resolver(
        mut self,
        resolver: impl Fn(&Graph, EdgeId) -> EdgeTypeKey + 'static,
    ) -> Self {
        self.resolver = Box::new(resolver);
        self
    }

    pub fn with_fallback(
        mut self,
        styler: impl Fn(&Graph, EdgeId, &NodeGraphStyle, EdgeRenderHint) -> EdgeRenderHint + 'static,
    ) -> Self {
        self.fallback = Some(Box::new(styler));
        self
    }

    pub fn register(
        mut self,
        key: EdgeTypeKey,
        styler: impl Fn(&Graph, EdgeId, &NodeGraphStyle, EdgeRenderHint) -> EdgeRenderHint + 'static,
    ) -> Self {
        self.edge_types.insert(key, Box::new(styler));
        self
    }

    pub fn apply(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        base: EdgeRenderHint,
    ) -> EdgeRenderHint {
        let key = (self.resolver)(graph, edge);
        if let Some(styler) = self.edge_types.get(&key) {
            return styler(graph, edge, style, base);
        }
        if let Some(fallback) = self.fallback.as_ref() {
            return fallback(graph, edge, style, base);
        }
        base
    }
}

fn default_edge_type_resolver(graph: &Graph, edge: EdgeId) -> EdgeTypeKey {
    let kind = graph
        .edges
        .get(&edge)
        .map(|e| e.kind)
        .unwrap_or(EdgeKind::Data);
    match kind {
        EdgeKind::Data => EdgeTypeKey::new("data"),
        EdgeKind::Exec => EdgeTypeKey::new("exec"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        CanvasPoint, Edge, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection,
        PortId, PortKey, PortKind,
    };

    fn make_exec_graph() -> (Graph, EdgeId) {
        let mut g = Graph::new(GraphId::from_u128(1));
        let a = NodeId::new();
        let b = NodeId::new();
        let out_port = PortId::new();
        let in_port = PortId::new();

        g.nodes.insert(
            a,
            Node {
                kind: NodeKindKey::new("demo.a"),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![out_port],
                data: serde_json::Value::Null,
            },
        );
        g.nodes.insert(
            b,
            Node {
                kind: NodeKindKey::new("demo.b"),
                kind_version: 1,
                pos: CanvasPoint { x: 100.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![in_port],
                data: serde_json::Value::Null,
            },
        );
        g.ports.insert(
            out_port,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Exec,
                capacity: PortCapacity::Multi,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        g.ports.insert(
            in_port,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Exec,
                capacity: PortCapacity::Single,
                ty: None,
                data: serde_json::Value::Null,
            },
        );

        let eid = EdgeId::new();
        g.edges.insert(
            eid,
            Edge {
                kind: EdgeKind::Exec,
                from: out_port,
                to: in_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        (g, eid)
    }

    #[test]
    fn edge_types_apply_can_override_hint_for_kind() {
        let (g, eid) = make_exec_graph();
        let style = NodeGraphStyle::default();
        let base = EdgeRenderHint::default();

        let edge_types =
            NodeGraphEdgeTypes::new().register(EdgeTypeKey::new("exec"), |_g, _e, _s, mut h| {
                h.route = crate::ui::presenter::EdgeRouteKind::Step;
                h
            });

        let hint = edge_types.apply(&g, eid, &style, base);
        assert_eq!(hint.route, crate::ui::presenter::EdgeRouteKind::Step);
    }
}
