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
//! Stage 2: optionally provide a **custom edge path** builder (and still keep the serialized
//! graph model UI-free). The canvas widget uses the custom path for painting and hit-testing,
//! and derives conservative AABBs for culling + spatial indexing.

use std::collections::BTreeMap;

use crate::core::{EdgeId, EdgeKind, Graph};
use crate::ui::presenter::EdgeRenderHint;
use crate::ui::style::NodeGraphStyle;
use fret_core::{PathCommand, Point};

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

#[derive(Debug, Clone, Copy)]
pub struct EdgePathInput {
    pub from: Point,
    pub to: Point,
    pub zoom: f32,
}

/// A custom edge path (Stage 2 `edgeTypes`).
///
/// Consumers are responsible for providing a stable `cache_key` so the canvas path cache can
/// reuse tessellation results across frames.
#[derive(Debug, Clone)]
pub struct EdgeCustomPath {
    pub cache_key: u64,
    pub commands: Vec<PathCommand>,
}

pub type EdgeTypePathBuilder = dyn Fn(&Graph, EdgeId, &NodeGraphStyle, &EdgeRenderHint, EdgePathInput) -> Option<EdgeCustomPath>
    + 'static;

#[derive(Default)]
struct EdgeTypeEntry {
    styler: Option<Box<EdgeTypeStyler>>,
    path: Option<Box<EdgeTypePathBuilder>>,
}

pub struct NodeGraphEdgeTypes {
    rev: u64,
    resolver: Box<EdgeTypeResolver>,
    edge_types: BTreeMap<EdgeTypeKey, EdgeTypeEntry>,
    fallback: Option<Box<EdgeTypeStyler>>,
    fallback_path: Option<Box<EdgeTypePathBuilder>>,
}

impl Default for NodeGraphEdgeTypes {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeGraphEdgeTypes {
    pub fn new() -> Self {
        Self {
            rev: 0,
            resolver: Box::new(default_edge_type_resolver),
            edge_types: BTreeMap::new(),
            fallback: None,
            fallback_path: None,
        }
    }

    pub fn revision(&self) -> u64 {
        self.rev
    }

    pub fn has_custom_paths(&self) -> bool {
        self.fallback_path.is_some() || self.edge_types.values().any(|e| e.path.is_some())
    }

    pub fn with_resolver(
        mut self,
        resolver: impl Fn(&Graph, EdgeId) -> EdgeTypeKey + 'static,
    ) -> Self {
        self.rev = self.rev.wrapping_add(1);
        self.resolver = Box::new(resolver);
        self
    }

    pub fn with_fallback(
        mut self,
        styler: impl Fn(&Graph, EdgeId, &NodeGraphStyle, EdgeRenderHint) -> EdgeRenderHint + 'static,
    ) -> Self {
        self.rev = self.rev.wrapping_add(1);
        self.fallback = Some(Box::new(styler));
        self
    }

    pub fn with_fallback_path(
        mut self,
        builder: impl Fn(
            &Graph,
            EdgeId,
            &NodeGraphStyle,
            &EdgeRenderHint,
            EdgePathInput,
        ) -> Option<EdgeCustomPath>
        + 'static,
    ) -> Self {
        self.rev = self.rev.wrapping_add(1);
        self.fallback_path = Some(Box::new(builder));
        self
    }

    pub fn register(
        mut self,
        key: EdgeTypeKey,
        styler: impl Fn(&Graph, EdgeId, &NodeGraphStyle, EdgeRenderHint) -> EdgeRenderHint + 'static,
    ) -> Self {
        self.rev = self.rev.wrapping_add(1);
        self.edge_types.entry(key).or_default().styler = Some(Box::new(styler));
        self
    }

    pub fn register_path(
        mut self,
        key: EdgeTypeKey,
        builder: impl Fn(
            &Graph,
            EdgeId,
            &NodeGraphStyle,
            &EdgeRenderHint,
            EdgePathInput,
        ) -> Option<EdgeCustomPath>
        + 'static,
    ) -> Self {
        self.rev = self.rev.wrapping_add(1);
        self.edge_types.entry(key).or_default().path = Some(Box::new(builder));
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
        if let Some(styler) = self.edge_types.get(&key).and_then(|e| e.styler.as_ref()) {
            return styler(graph, edge, style, base);
        }
        if let Some(fallback) = self.fallback.as_ref() {
            return fallback(graph, edge, style, base);
        }
        base
    }

    pub fn custom_path(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        hint: &EdgeRenderHint,
        input: EdgePathInput,
    ) -> Option<EdgeCustomPath> {
        let key = (self.resolver)(graph, edge);
        if let Some(builder) = self.edge_types.get(&key).and_then(|e| e.path.as_ref()) {
            return builder(graph, edge, style, hint, input);
        }
        if let Some(builder) = self.fallback_path.as_ref() {
            return builder(graph, edge, style, hint, input);
        }
        None
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
    use fret_core::{PathCommand, Point};

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
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
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
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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

    #[test]
    fn edge_types_custom_path_can_be_registered() {
        let (g, eid) = make_exec_graph();
        let style = NodeGraphStyle::default();
        let hint = EdgeRenderHint::default();

        let edge_types = NodeGraphEdgeTypes::new().register_path(
            EdgeTypeKey::new("exec"),
            |_g, _e, _s, _h, input| {
                Some(EdgeCustomPath {
                    cache_key: 42,
                    commands: vec![
                        PathCommand::MoveTo(input.from),
                        PathCommand::LineTo(input.to),
                    ],
                })
            },
        );

        let path = edge_types.custom_path(
            &g,
            eid,
            &style,
            &hint,
            EdgePathInput {
                from: Point::default(),
                to: Point::default(),
                zoom: 1.0,
            },
        );
        assert!(path.is_some());
    }
}
