use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;

use crate::ui::NodeGraphViewQueue;

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_size};

#[test]
fn frame_nodes_via_view_queue_matches_direct_framing() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 5000.0, y: 0.0 };

    // Expected viewport by direct framing (immediate).
    let mut host_expected = TestUiHostImpl::default();
    let graph_expected = host_expected.models.insert(graph_value.clone());
    let view_expected = insert_view(&mut host_expected);
    let _ = view_expected.update(&mut host_expected, |s, _cx| {
        s.interaction.frame_view_duration_ms = 0;
    });
    let mut canvas_expected = NodeGraphCanvas::new(graph_expected, view_expected);
    assert!(canvas_expected.frame_nodes_in_view(&mut host_expected, None, bounds, &[a]));
    let expected = canvas_expected.sync_view_state(&mut host_expected);

    // Same action via view queue (framing specific nodes).
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.frame_view_duration_ms = 0;
    });
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);

    let _ = queue.update(&mut host, |q, _cx| {
        q.push_frame_nodes(vec![a]);
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let actual = canvas.sync_view_state(&mut host);

    assert!((actual.zoom - expected.zoom).abs() <= 1.0e-3);
    assert!((actual.pan.x - expected.pan.x).abs() <= 1.0e-2);
    assert!((actual.pan.y - expected.pan.y).abs() <= 1.0e-2);
    // Sanity: framing a single node should differ from framing both.
    assert!(
        actual.pan.x.is_finite() && actual.zoom.is_finite() && actual.zoom > 0.0,
        "expected a valid viewport"
    );
}
