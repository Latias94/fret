use std::sync::Arc;

use fret_core::{
    AppWindowId, Event, InternalDragEvent, InternalDragKind, Modifiers, Point, Px, Rect, Size,
};
use fret_runtime::{DragSession, DragSessionId};
use fret_ui::retained_bridge::Widget as _;
use serde_json::Value;

use crate::core::{NodeKindKey, PortCapacity, PortDirection, PortKey, PortKind};
use crate::rules::{InsertNodeTemplate, PortTemplate};
use crate::ui::presenter::{InsertNodeCandidate, NodeGraphPresenter};
use crate::{core::Graph, ops::GraphOp};

use super::prelude::NodeGraphCanvas;
use super::{NullServices, TestUiHostImpl, event_cx, insert_graph_view, make_test_graph_two_nodes};

#[derive(Default)]
struct BackgroundInsertPresenter;

impl NodeGraphPresenter for BackgroundInsertPresenter {
    fn node_title(&self, graph: &Graph, node: crate::core::NodeId) -> Arc<str> {
        graph
            .nodes
            .get(&node)
            .map(|n| Arc::<str>::from(n.kind.0.clone()))
            .unwrap_or_else(|| Arc::<str>::from("<missing node>"))
    }

    fn port_label(&self, graph: &Graph, port: crate::core::PortId) -> Arc<str> {
        graph
            .ports
            .get(&port)
            .map(|p| Arc::<str>::from(p.key.0.clone()))
            .unwrap_or_else(|| Arc::<str>::from("<missing port>"))
    }

    fn plan_create_node(
        &mut self,
        _graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: crate::core::CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        let Some(template) = candidate.template.as_ref() else {
            return Err(Arc::<str>::from("missing insert template"));
        };
        let spec = template
            .instantiate(at)
            .map_err(|err| Arc::<str>::from(err))?;

        let mut ops: Vec<GraphOp> = Vec::new();
        let mut port_ids: Vec<crate::core::PortId> = Vec::new();

        ops.push(GraphOp::AddNode {
            id: spec.node_id,
            node: spec.node,
        });
        for (port_id, port) in spec.ports {
            port_ids.push(port_id);
            ops.push(GraphOp::AddPort { id: port_id, port });
        }
        ops.push(GraphOp::SetNodePorts {
            id: spec.node_id,
            from: Vec::new(),
            to: port_ids,
        });

        Ok(ops)
    }
}

#[test]
fn internal_drag_drop_candidate_off_edge_creates_node() {
    let mut host = TestUiHostImpl::default();

    let template_kind = NodeKindKey::new("test.insert.off_edge");
    let template = InsertNodeTemplate {
        kind: template_kind.clone(),
        kind_version: 1,
        collapsed: false,
        data: Value::Null,
        ports: vec![
            PortTemplate {
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: Value::Null,
            },
            PortTemplate {
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: Value::Null,
            },
        ],
        input: PortKey::new("in"),
        output: PortKey::new("out"),
    };
    let candidate = InsertNodeCandidate {
        kind: template_kind.clone(),
        label: Arc::<str>::from("Insert"),
        enabled: true,
        template: Some(template),
        payload: Value::Null,
    };

    let pointer_id = fret_core::PointerId(0);
    host.drag = Some(DragSession::new_cross_window(
        DragSessionId(1),
        pointer_id,
        AppWindowId::default(),
        crate::ui::canvas::widget::insert_node_drag::DRAG_KIND_INSERT_NODE,
        Point::new(Px(0.0), Px(0.0)),
        crate::ui::canvas::widget::insert_node_drag::InsertNodeDragPayload { candidate },
    ));

    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    canvas.presenter = Box::<BackgroundInsertPresenter>::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let drop_pos = Point::new(Px(123.0), Px(45.0));
    canvas.event(
        &mut cx,
        &Event::InternalDrag(InternalDragEvent {
            pointer_id,
            position: drop_pos,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(nodes_len, 3);
    assert_eq!(edges_len, 0);
    assert!(
        graph
            .read_ref(cx.app, |g| g
                .nodes
                .values()
                .any(|n| n.kind == template_kind))
            .unwrap_or(false)
    );

    let after = canvas.sync_view_state(cx.app);
    assert_eq!(after.selected_nodes.len(), 1);
    assert_eq!(after.selected_edges.len(), 0);
}

#[test]
fn internal_drag_drop_reroute_candidate_off_edge_creates_node() {
    let mut host = TestUiHostImpl::default();

    let candidate = InsertNodeCandidate {
        kind: NodeKindKey::new(crate::REROUTE_KIND),
        label: Arc::<str>::from("Reroute"),
        enabled: true,
        template: None,
        payload: Value::Null,
    };

    let pointer_id = fret_core::PointerId(0);
    host.drag = Some(DragSession::new_cross_window(
        DragSessionId(1),
        pointer_id,
        AppWindowId::default(),
        crate::ui::canvas::widget::insert_node_drag::DRAG_KIND_INSERT_NODE,
        Point::new(Px(0.0), Px(0.0)),
        crate::ui::canvas::widget::insert_node_drag::InsertNodeDragPayload { candidate },
    ));

    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    canvas.event(
        &mut cx,
        &Event::InternalDrag(InternalDragEvent {
            pointer_id,
            position: Point::new(Px(321.0), Px(654.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    assert_eq!(nodes_len, 3);
    assert!(
        graph
            .read_ref(cx.app, |g| g
                .nodes
                .values()
                .any(|n| n.kind.0 == crate::REROUTE_KIND))
            .unwrap_or(false)
    );
}
