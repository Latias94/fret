use std::sync::Arc;

use crate::TypeDesc;
use crate::core::{CanvasPoint, EdgeId, Graph, NodeId, NodeKindKey, PortId};
use crate::ops::GraphOp;
use crate::profile::GraphProfile;
use crate::rules::{ConnectPlan, EdgeEndpoint, InsertNodeTemplate};
use crate::ui::presenter::{
    EdgeRenderHint, InsertNodeCandidate, NodeGraphContextMenuItem, NodeGraphPresenter,
    PortAnchorHint,
};
use crate::ui::style::NodeGraphStyle;

use super::store::MeasuredGeometryStore;

/// Presenter wrapper that consults measured geometry before delegating to an inner presenter.
///
/// This allows hosts to "push" measured handle bounds into the editor without changing the core
/// `NodeGraphCanvas` implementation.
pub struct MeasuredNodeGraphPresenter<P> {
    inner: P,
    measured: Arc<MeasuredGeometryStore>,
}

impl<P> MeasuredNodeGraphPresenter<P> {
    pub fn new(inner: P, measured: Arc<MeasuredGeometryStore>) -> Self {
        Self { inner, measured }
    }

    pub fn measured(&self) -> &Arc<MeasuredGeometryStore> {
        &self.measured
    }

    pub fn inner(&self) -> &P {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut P {
        &mut self.inner
    }
}

impl<P: NodeGraphPresenter> NodeGraphPresenter for MeasuredNodeGraphPresenter<P> {
    fn geometry_revision(&self) -> u64 {
        self.inner.geometry_revision().max(self.measured.revision())
    }

    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        self.inner.node_title(graph, node)
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        self.inner.port_label(graph, port)
    }

    fn node_body_label(&self, graph: &Graph, node: NodeId) -> Option<Arc<str>> {
        self.inner.node_body_label(graph, node)
    }

    fn port_color(&self, graph: &Graph, port: PortId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.port_color(graph, port, style)
    }

    fn edge_color(&self, graph: &Graph, edge: EdgeId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.edge_color(graph, edge, style)
    }

    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
    ) -> EdgeRenderHint {
        self.inner.edge_render_hint(graph, edge, style)
    }

    fn node_size_hint_px(
        &mut self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
    ) -> Option<(f32, f32)> {
        let measured = self.measured.node_size_px(node);
        let hinted = self.inner.node_size_hint_px(graph, node, style);
        match (measured, hinted) {
            (Some(a), Some(b)) => Some((a.0.max(b.0), a.1.max(b.1))),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    fn port_anchor_hint(
        &mut self,
        graph: &Graph,
        node: NodeId,
        port: PortId,
        style: &NodeGraphStyle,
    ) -> Option<PortAnchorHint> {
        self.measured
            .port_anchor_px(port)
            .or_else(|| self.inner.port_anchor_hint(graph, node, port, style))
    }

    fn list_insertable_nodes(&mut self, graph: &Graph) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes(graph)
    }

    fn plan_create_node(
        &mut self,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        self.inner.plan_create_node(graph, candidate, at)
    }

    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes_for_edge(graph, edge)
    }

    fn plan_split_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        node_kind: &NodeKindKey,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner.plan_split_edge(graph, edge, node_kind, at)
    }

    fn plan_split_edge_candidate(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner
            .plan_split_edge_candidate(graph, edge, candidate, at)
    }

    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        self.inner.fill_edge_context_menu(graph, edge, style, out)
    }

    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<GraphOp>> {
        self.inner.on_edge_context_menu_action(graph, edge, action)
    }

    fn plan_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner.plan_connect(graph, a, b, mode)
    }

    fn plan_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner
            .plan_reconnect_edge(graph, edge, endpoint, new_port, mode)
    }

    fn profile_mut(&mut self) -> Option<&mut (dyn GraphProfile + 'static)> {
        self.inner.profile_mut()
    }

    fn type_of_port(&self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        self.inner.type_of_port(graph, port)
    }

    fn can_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner.can_connect(graph, a, b, mode)
    }

    fn can_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner
            .can_reconnect_edge(graph, edge, endpoint, new_port, mode)
    }

    fn list_conversions(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        self.inner.list_conversions(graph, from, to)
    }

    fn conversion_label(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        template: &InsertNodeTemplate,
    ) -> Arc<str> {
        self.inner.conversion_label(graph, from, to, template)
    }

    fn conversion_insert_position(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        default_at: CanvasPoint,
        template: &InsertNodeTemplate,
    ) -> CanvasPoint {
        self.inner
            .conversion_insert_position(graph, from, to, default_at, template)
    }
}

/// Presenter wrapper that uses measured geometry as a fallback.
///
/// This is useful for auto-measured internals maintained by the node-graph widget itself:
/// domain presenters keep full control, while the editor can still provide derived sizing/anchor
/// hints when the presenter returns `None`.
pub struct FallbackMeasuredNodeGraphPresenter<P> {
    inner: P,
    measured: Arc<MeasuredGeometryStore>,
}

impl<P> FallbackMeasuredNodeGraphPresenter<P> {
    pub fn new(inner: P, measured: Arc<MeasuredGeometryStore>) -> Self {
        Self { inner, measured }
    }
}

impl<P: NodeGraphPresenter> NodeGraphPresenter for FallbackMeasuredNodeGraphPresenter<P> {
    fn geometry_revision(&self) -> u64 {
        self.inner.geometry_revision().max(self.measured.revision())
    }

    fn node_title(&self, graph: &Graph, node: NodeId) -> Arc<str> {
        self.inner.node_title(graph, node)
    }

    fn port_label(&self, graph: &Graph, port: PortId) -> Arc<str> {
        self.inner.port_label(graph, port)
    }

    fn node_body_label(&self, graph: &Graph, node: NodeId) -> Option<Arc<str>> {
        self.inner.node_body_label(graph, node)
    }

    fn port_color(&self, graph: &Graph, port: PortId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.port_color(graph, port, style)
    }

    fn edge_color(&self, graph: &Graph, edge: EdgeId, style: &NodeGraphStyle) -> fret_core::Color {
        self.inner.edge_color(graph, edge, style)
    }

    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
    ) -> EdgeRenderHint {
        self.inner.edge_render_hint(graph, edge, style)
    }

    fn node_size_hint_px(
        &mut self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
    ) -> Option<(f32, f32)> {
        self.inner
            .node_size_hint_px(graph, node, style)
            .or_else(|| self.measured.node_size_px(node))
    }

    fn port_anchor_hint(
        &mut self,
        graph: &Graph,
        node: NodeId,
        port: PortId,
        style: &NodeGraphStyle,
    ) -> Option<PortAnchorHint> {
        self.inner
            .port_anchor_hint(graph, node, port, style)
            .or_else(|| self.measured.port_anchor_px(port))
    }

    fn list_insertable_nodes(&mut self, graph: &Graph) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes(graph)
    }

    fn plan_create_node(
        &mut self,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        self.inner.plan_create_node(graph, candidate, at)
    }

    fn list_insertable_nodes_for_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        self.inner.list_insertable_nodes_for_edge(graph, edge)
    }

    fn plan_split_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        node_kind: &NodeKindKey,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner.plan_split_edge(graph, edge, node_kind, at)
    }

    fn plan_split_edge_candidate(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> ConnectPlan {
        self.inner
            .plan_split_edge_candidate(graph, edge, candidate, at)
    }

    fn fill_edge_context_menu(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        out: &mut Vec<NodeGraphContextMenuItem>,
    ) {
        self.inner.fill_edge_context_menu(graph, edge, style, out)
    }

    fn on_edge_context_menu_action(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        action: u64,
    ) -> Option<Vec<GraphOp>> {
        self.inner.on_edge_context_menu_action(graph, edge, action)
    }

    fn plan_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner.plan_connect(graph, a, b, mode)
    }

    fn plan_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner
            .plan_reconnect_edge(graph, edge, endpoint, new_port, mode)
    }

    fn profile_mut(&mut self) -> Option<&mut (dyn GraphProfile + 'static)> {
        self.inner.profile_mut()
    }

    fn type_of_port(&self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        self.inner.type_of_port(graph, port)
    }

    fn can_connect(
        &mut self,
        graph: &Graph,
        a: PortId,
        b: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner.can_connect(graph, a, b, mode)
    }

    fn can_reconnect_edge(
        &mut self,
        graph: &Graph,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        new_port: PortId,
        mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        self.inner
            .can_reconnect_edge(graph, edge, endpoint, new_port, mode)
    }

    fn list_conversions(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        self.inner.list_conversions(graph, from, to)
    }

    fn conversion_label(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        template: &InsertNodeTemplate,
    ) -> Arc<str> {
        self.inner.conversion_label(graph, from, to, template)
    }

    fn conversion_insert_position(
        &mut self,
        graph: &Graph,
        from: PortId,
        to: PortId,
        default_at: CanvasPoint,
        template: &InsertNodeTemplate,
    ) -> CanvasPoint {
        self.inner
            .conversion_insert_position(graph, from, to, default_at, template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX;
    use fret_core::{Point, Px, Rect, Size};

    #[derive(Default)]
    struct StubPresenter {
        hint_size: Option<(f32, f32)>,
        hint_anchor: Option<PortAnchorHint>,
    }

    impl NodeGraphPresenter for StubPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("port")
        }

        fn node_size_hint_px(
            &mut self,
            _graph: &Graph,
            _node: NodeId,
            _style: &NodeGraphStyle,
        ) -> Option<(f32, f32)> {
            self.hint_size
        }

        fn port_anchor_hint(
            &mut self,
            _graph: &Graph,
            _node: NodeId,
            _port: PortId,
            _style: &NodeGraphStyle,
        ) -> Option<PortAnchorHint> {
            self.hint_anchor
        }
    }

    fn hint(x: f32, y: f32) -> PortAnchorHint {
        PortAnchorHint {
            center: Point::new(Px(x), Px(y)),
            bounds: Rect::new(
                Point::new(Px(x - 1.0), Px(y - 1.0)),
                Size::new(Px(2.0), Px(2.0)),
            ),
        }
    }

    fn assert_anchor_eq(a: PortAnchorHint, b: PortAnchorHint) {
        assert!((a.center.x.0 - b.center.x.0).abs() <= 1.0e-6);
        assert!((a.center.y.0 - b.center.y.0).abs() <= 1.0e-6);
        assert!((a.bounds.origin.x.0 - b.bounds.origin.x.0).abs() <= 1.0e-6);
        assert!((a.bounds.origin.y.0 - b.bounds.origin.y.0).abs() <= 1.0e-6);
        assert!((a.bounds.size.width.0 - b.bounds.size.width.0).abs() <= 1.0e-6);
        assert!((a.bounds.size.height.0 - b.bounds.size.height.0).abs() <= 1.0e-6);
    }

    #[test]
    fn measured_presenter_node_size_uses_max_of_measured_and_hint() {
        let measured = Arc::new(MeasuredGeometryStore::new());
        let node = NodeId::new();
        measured.update(|sizes, _| {
            sizes.insert(node, (100.0, 200.0));
        });

        let mut presenter = MeasuredNodeGraphPresenter::new(
            StubPresenter {
                hint_size: Some((100.0 + MEASURED_GEOMETRY_EPSILON_PX, 10.0)),
                ..StubPresenter::default()
            },
            measured,
        );

        let got = presenter.node_size_hint_px(&Graph::default(), node, &NodeGraphStyle::default());
        assert_eq!(got, Some((100.0 + MEASURED_GEOMETRY_EPSILON_PX, 200.0)));
    }

    #[test]
    fn measured_presenter_port_anchor_prefers_measured_over_hint() {
        let measured = Arc::new(MeasuredGeometryStore::new());
        let node = NodeId::new();
        let port = PortId::new();
        let measured_hint = hint(10.0, 20.0);
        measured.update(|_, anchors| {
            anchors.insert(port, measured_hint);
        });

        let mut presenter = MeasuredNodeGraphPresenter::new(
            StubPresenter {
                hint_anchor: Some(hint(1.0, 2.0)),
                ..StubPresenter::default()
            },
            measured,
        );

        let got =
            presenter.port_anchor_hint(&Graph::default(), node, port, &NodeGraphStyle::default());
        assert_anchor_eq(got.expect("expected anchor hint"), measured_hint);
    }

    #[test]
    fn fallback_presenter_prefers_hint_over_measured() {
        let measured = Arc::new(MeasuredGeometryStore::new());
        let node = NodeId::new();
        let port = PortId::new();
        measured.update(|sizes, anchors| {
            sizes.insert(node, (100.0, 200.0));
            anchors.insert(port, hint(10.0, 20.0));
        });

        let mut presenter = FallbackMeasuredNodeGraphPresenter::new(
            StubPresenter {
                hint_size: Some((1.0, 2.0)),
                hint_anchor: Some(hint(3.0, 4.0)),
            },
            measured,
        );

        let size = presenter.node_size_hint_px(&Graph::default(), node, &NodeGraphStyle::default());
        assert_eq!(size, Some((1.0, 2.0)));

        let anchor =
            presenter.port_anchor_hint(&Graph::default(), node, port, &NodeGraphStyle::default());
        assert_anchor_eq(anchor.expect("expected anchor hint"), hint(3.0, 4.0));
    }
}
