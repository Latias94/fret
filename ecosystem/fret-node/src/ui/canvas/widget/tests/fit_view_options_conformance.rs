use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;

use crate::ui::{NodeGraphFitViewOptions, NodeGraphViewQueue};

use super::prelude::NodeGraphCanvas;
use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_size};

#[test]
fn fit_view_options_min_zoom_clamps() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 8000.0, y: 0.0 };

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);
    canvas.style.min_zoom = 0.01;
    canvas.style.max_zoom = 10.0;

    let opts = NodeGraphFitViewOptions {
        duration_ms: Some(0),
        min_zoom: Some(0.8),
        ..NodeGraphFitViewOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_frame_nodes_with_options(vec![a, b], opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);
    assert!(
        snap.zoom >= 0.8 - 1.0e-6,
        "expected zoom to be clamped by min_zoom (zoom={})",
        snap.zoom
    );
}

#[test]
fn fit_view_options_max_zoom_clamps() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, a, b) = make_test_graph_two_nodes_with_size();

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);
    canvas.style.min_zoom = 0.01;
    canvas.style.max_zoom = 10.0;

    let opts = NodeGraphFitViewOptions {
        duration_ms: Some(0),
        max_zoom: Some(0.5),
        ..NodeGraphFitViewOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_frame_nodes_with_options(vec![a, b], opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);
    assert!(
        snap.zoom <= 0.5 + 1.0e-6,
        "expected zoom to be clamped by max_zoom (zoom={})",
        snap.zoom
    );
}

#[test]
fn fit_view_options_include_hidden_nodes_controls_framing() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").hidden = true;
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 8000.0, y: 0.0 };

    // Expected viewport when hidden nodes are excluded (effectively frame only A).
    let mut host_expected = TestUiHostImpl::default();
    let graph_expected = host_expected.models.insert(graph_value.clone());
    let view_expected = insert_view(&mut host_expected);
    let mut canvas_expected = NodeGraphCanvas::new(graph_expected, view_expected);
    assert!(canvas_expected.frame_nodes_in_view(&mut host_expected, None, bounds, &[a]));
    let expected_excluding_hidden = canvas_expected.sync_view_state(&mut host_expected);

    // Expected viewport when hidden nodes are included.
    let mut host_expected2 = TestUiHostImpl::default();
    let graph_expected2 = host_expected2.models.insert(graph_value.clone());
    let view_expected2 = insert_view(&mut host_expected2);
    let mut canvas_expected2 = NodeGraphCanvas::new(graph_expected2, view_expected2);
    let opts = NodeGraphFitViewOptions {
        duration_ms: Some(0),
        include_hidden_nodes: true,
        ..NodeGraphFitViewOptions::default()
    };
    assert!(canvas_expected2.frame_nodes_in_view_with_options(
        &mut host_expected2,
        None,
        bounds,
        &[a, b],
        Some(&opts),
    ));
    let expected_including_hidden = canvas_expected2.sync_view_state(&mut host_expected2);

    // Actual via view queue defaults (exclude hidden).
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value.clone());
    let view = insert_view(&mut host);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);

    let _ = queue.update(&mut host, |q, _cx| q.push_frame_nodes(vec![a, b]));
    assert!(canvas.drain_view_queue(&mut host, None));
    let actual_excluding_hidden = canvas.sync_view_state(&mut host);
    assert!((actual_excluding_hidden.zoom - expected_excluding_hidden.zoom).abs() <= 1.0e-3);
    assert!((actual_excluding_hidden.pan.x - expected_excluding_hidden.pan.x).abs() <= 1.0e-2);
    assert!((actual_excluding_hidden.pan.y - expected_excluding_hidden.pan.y).abs() <= 1.0e-2);

    // Actual via view queue with include_hidden_nodes=true.
    let mut host2 = TestUiHostImpl::default();
    let graph2 = host2.models.insert(graph_value);
    let view2 = insert_view(&mut host2);
    let queue2 = host2.models.insert(NodeGraphViewQueue::default());
    let mut canvas2 = NodeGraphCanvas::new(graph2, view2).with_view_queue(queue2.clone());
    canvas2.interaction.last_bounds = Some(bounds);

    let opts2 = NodeGraphFitViewOptions {
        duration_ms: Some(0),
        include_hidden_nodes: true,
        ..NodeGraphFitViewOptions::default()
    };
    let _ = queue2.update(&mut host2, |q, _cx| {
        q.push_frame_nodes_with_options(vec![a, b], opts2)
    });
    assert!(canvas2.drain_view_queue(&mut host2, None));
    let actual_including_hidden = canvas2.sync_view_state(&mut host2);

    assert!((actual_including_hidden.zoom - expected_including_hidden.zoom).abs() <= 1.0e-3);
    assert!((actual_including_hidden.pan.x - expected_including_hidden.pan.x).abs() <= 1.0e-2);
    assert!((actual_including_hidden.pan.y - expected_including_hidden.pan.y).abs() <= 1.0e-2);
}
