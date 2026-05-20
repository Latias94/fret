use fret_core::{Point, Px};

use crate::core::{CanvasPoint, CanvasSize};
use crate::ui::canvas::NodeResizeHandle;
use crate::ui::canvas::state::PendingNodeResize;

use super::{TestUiHostImpl, insert_graph_view_editor_config, make_test_graph_two_nodes};

fn pending_resize_at_start(node: crate::core::NodeId) -> PendingNodeResize {
    PendingNodeResize {
        node,
        handle: NodeResizeHandle::Right,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_node_pos: CanvasPoint::default(),
        start_size: CanvasSize {
            width: 120.0,
            height: 80.0,
        },
        start_size_opt: Some(CanvasSize {
            width: 120.0,
            height: 80.0,
        }),
    }
}

#[test]
fn pending_node_resize_move_below_threshold_keeps_pending_resize() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, node, _other) = make_test_graph_two_nodes();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let mut snapshot = canvas.sync_view_state(&mut host);
    snapshot.interaction.node_drag_threshold = 10.0;
    canvas.interaction.pending_node_resize = Some(pending_resize_at_start(node));

    let handled = super::super::pending_resize::handle_pending_node_resize_move(
        &mut canvas,
        &snapshot,
        Point::new(Px(4.0), Px(0.0)),
        snapshot.zoom,
    );

    assert!(handled);
    assert!(canvas.interaction.pending_node_resize.is_some());
    assert!(canvas.interaction.node_resize.is_none());
}

#[test]
fn pending_node_resize_move_past_threshold_activates_resize() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, node, _other) = make_test_graph_two_nodes();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let mut snapshot = canvas.sync_view_state(&mut host);
    snapshot.interaction.node_drag_threshold = 10.0;
    canvas.interaction.pending_node_resize = Some(pending_resize_at_start(node));

    let handled = super::super::pending_resize::handle_pending_node_resize_move(
        &mut canvas,
        &snapshot,
        Point::new(Px(20.0), Px(0.0)),
        snapshot.zoom,
    );

    assert!(!handled);
    assert!(canvas.interaction.pending_node_resize.is_none());
    let active = canvas
        .interaction
        .node_resize
        .as_ref()
        .expect("node resize active");
    assert_eq!(active.node, node);
    assert_eq!(
        active.current_size_opt,
        Some(CanvasSize {
            width: 120.0,
            height: 80.0
        })
    );
}
