use fret_runtime::CommandId;
use fret_ui::retained_bridge::Widget as _;

use crate::core::{CanvasPoint, CanvasSize};
use crate::ui::NodeGraphCanvas;
use crate::ui::commands::{
    CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_NUDGE_RIGHT_FAST, CMD_NODE_GRAPH_NUDGE_UP,
};

use super::{
    NullServices, TestUiHostImpl, command_cx, make_host_graph_view, make_test_graph_two_nodes,
    read_node_pos,
};

#[test]
fn nudge_screen_px_step_is_zoom_invariant() {
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    view.update(&mut host, |s, _cx| {
        s.zoom = 2.0;
        s.selected_nodes = vec![a];
    })
    .unwrap();

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
    assert_eq!(
        read_node_pos(&mut host, &graph, a),
        CanvasPoint { x: 0.5, y: 0.0 }
    );
}

#[test]
fn nudge_grid_step_uses_snap_grid_even_when_snap_to_grid_is_disabled() {
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    view.update(&mut host, |s, _cx| {
        s.zoom = 2.0;
        s.selected_nodes = vec![a];
        s.interaction.snap_to_grid = false;
        s.interaction.snap_grid = CanvasSize {
            width: 16.0,
            height: 12.0,
        };
        s.interaction.nudge_step_mode = crate::io::NodeGraphNudgeStepMode::Grid;
    })
    .unwrap();

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
    }
    assert_eq!(read_node_pos(&mut host, &graph, a).x, 16.0);

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_UP)));
    }
    assert_eq!(read_node_pos(&mut host, &graph, a).y, -12.0);
}

#[test]
fn nudge_grid_fast_step_moves_ten_cells_by_default() {
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    view.update(&mut host, |s, _cx| {
        s.zoom = 0.5;
        s.selected_nodes = vec![a];
        s.interaction.snap_to_grid = false;
        s.interaction.snap_grid = CanvasSize {
            width: 16.0,
            height: 12.0,
        };
        s.interaction.nudge_step_mode = crate::io::NodeGraphNudgeStepMode::Grid;
    })
    .unwrap();

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT_FAST)));
    assert_eq!(read_node_pos(&mut host, &graph, a).x, 160.0);
}
