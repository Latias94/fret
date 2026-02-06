use fret_core::{AppWindowId, Modifiers, Point, Px, Rect, Size};
use fret_ui::UiHost;
use uuid::Uuid;

use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Graph, GraphId, Group, GroupId, Node, NodeId,
};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction};
use crate::ui::{
    NodeGraphCanvasCommitOutcome, NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareCx,
};

use super::super::{NodeGraphCanvas, node_drag, pointer_up};
use super::{NullServices, TestUiHostImpl, event_cx};
use crate::ui::canvas::state::NodeDrag;

#[derive(Debug, Default)]
struct CaptureGroupRectCommitOrder {
    commits: Vec<Vec<GroupId>>,
}

impl NodeGraphCanvasMiddleware for CaptureGroupRectCommitOrder {
    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        let ids: Vec<GroupId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::SetGroupRect { id, .. } => Some(*id),
                _ => None,
            })
            .collect();

        if !ids.is_empty() {
            self.commits.push(ids);
        }

        NodeGraphCanvasCommitOutcome::Continue
    }
}

#[test]
fn node_drag_end_batches_group_rect_ops_in_sorted_group_id_order() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::from_u128(1));

    // Use explicit ids so the expected order is stable and easy to reason about.
    let group_ids: Vec<GroupId> = [
        120_u128, 3_u128, 99_u128, 42_u128, 77_u128, 1_u128, 88_u128, 7_u128,
    ]
    .into_iter()
    .map(|id| GroupId(Uuid::from_u128(id)))
    .collect();

    let kind = crate::core::NodeKindKey::new("test.node");
    let mut node_ids: Vec<NodeId> = Vec::new();
    let mut start_nodes: Vec<(NodeId, CanvasPoint)> = Vec::new();

    for (i, group_id) in group_ids.iter().copied().enumerate() {
        let y0 = i as f32 * 200.0;
        graph_value.groups.insert(
            group_id,
            Group {
                title: format!("G{i}"),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: y0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 100.0,
                    },
                },
                color: None,
            },
        );

        let node_id = NodeId(Uuid::from_u128(10_000 + i as u128));
        node_ids.push(node_id);

        let start = CanvasPoint {
            x: 10.0,
            y: y0 + 10.0,
        };
        start_nodes.push((node_id, start));

        graph_value.nodes.insert(
            node_id,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: start,
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: Some(group_id),
                extent: None,
                expand_parent: Some(true),
                size: Some(CanvasSize {
                    width: 80.0,
                    height: 40.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );
    }

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas =
        NodeGraphCanvas::new(graph, view).with_middleware(CaptureGroupRectCommitOrder::default());
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: node_ids[0],
        node_ids,
        nodes: start_nodes.clone(),
        current_nodes: start_nodes,
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(10.0), Px(10.0)),
    });

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

    // Move right enough that each group must expand to include its child node.
    assert!(node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(10.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(10.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert_eq!(canvas.middleware.commits.len(), 1);
    let got = &canvas.middleware.commits[0];

    let mut expected = group_ids.clone();
    expected.sort();
    assert_eq!(got, &expected);
}
