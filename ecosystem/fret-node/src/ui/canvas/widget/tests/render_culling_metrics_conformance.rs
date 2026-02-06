use fret_core::{Point, Px, Rect, Size};

use crate::io::NodeGraphViewState;

use super::{NodeGraphCanvas, TestUiHostImpl, make_test_graph_two_nodes_with_size};

#[test]
fn render_metrics_report_culling_reduction_when_enabled() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, _a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos.x = 50_000.0;
    graph_value.nodes.get_mut(&b).expect("node b exists").pos.y = 0.0;

    let metrics_culled = {
        let mut host = TestUiHostImpl::default();
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = crate::core::CanvasPoint::default();
            s.zoom = 1.0;
            s.interaction.only_render_visible_elements = true;
            s.interaction.frame_view_duration_ms = 0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        canvas.debug_render_metrics_for_bounds(&mut host, bounds)
    };

    let metrics_full = {
        let mut host = TestUiHostImpl::default();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = crate::core::CanvasPoint::default();
            s.zoom = 1.0;
            s.interaction.only_render_visible_elements = false;
            s.interaction.frame_view_duration_ms = 0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        canvas.debug_render_metrics_for_bounds(&mut host, bounds)
    };

    assert_eq!(metrics_culled.node_total, metrics_full.node_total);
    assert!(metrics_culled.node_total >= 2);
    assert!(
        metrics_culled.node_candidates < metrics_full.node_candidates,
        "expected culling to reduce node candidates (culled={}, full={})",
        metrics_culled.node_candidates,
        metrics_full.node_candidates
    );
    assert!(
        metrics_culled.node_visible < metrics_full.node_visible,
        "expected culling to reduce visible nodes (culled={}, full={})",
        metrics_culled.node_visible,
        metrics_full.node_visible
    );
}
