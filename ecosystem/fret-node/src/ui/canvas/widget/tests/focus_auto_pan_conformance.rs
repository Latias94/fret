use fret_runtime::CommandId;

use fret_core::{Point, Px, Rect, Size};
use fret_ui::retained_bridge::Widget as _;

use crate::core::CanvasPoint;

use crate::ui::commands::CMD_NODE_GRAPH_FOCUS_NEXT;

use super::prelude::NodeGraphCanvas;
use super::{NullServices, TestUiHostImpl, command_cx, insert_view, make_test_graph_two_nodes};

#[test]
fn focus_next_can_pan_viewport_when_auto_pan_on_node_focus_is_enabled() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 5000.0, y: 0.0 };

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);
    canvas.interaction.last_bounds = Some(bounds);

    view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.draw_order = vec![a, b];
        s.interaction.auto_pan.on_node_focus = true;
        s.selected_nodes.clear();
        s.selected_edges.clear();
        s.selected_groups.clear();
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
    }
    let pan_after_first = view.read_ref(&host, |s| s.pan).unwrap_or_default();
    assert!((pan_after_first.x).abs() <= 1.0e-6);

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
    }
    let pan_after_second = view.read_ref(&host, |s| s.pan).unwrap_or_default();
    assert!(
        pan_after_second.x.abs() > 1000.0,
        "expected focus-driven pan to bring far node into view; pan was {pan_after_second:?}"
    );
}
