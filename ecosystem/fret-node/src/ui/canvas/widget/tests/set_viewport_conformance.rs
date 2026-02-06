use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;
use crate::ui::{NodeGraphSetViewportOptions, NodeGraphViewQueue};

use super::prelude::NodeGraphCanvas;
use super::{make_host_graph_view, make_test_graph_two_nodes_with_size};

#[test]
fn set_viewport_via_view_queue_updates_pan_and_zoom() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);

    let pan = CanvasPoint { x: 123.0, y: -45.0 };
    let zoom = 2.5;
    let opts = NodeGraphSetViewportOptions {
        duration_ms: Some(0),
        ..NodeGraphSetViewportOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_set_viewport_with_options(pan, zoom, opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);
    assert!((snap.pan.x - pan.x).abs() <= 1.0e-6);
    assert!((snap.pan.y - pan.y).abs() <= 1.0e-6);
    assert!((snap.zoom - zoom).abs() <= 1.0e-6);
}

#[test]
fn set_viewport_via_view_queue_clamps_zoom_to_style_limits() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 0.5;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);
    canvas.style.min_zoom = 0.5;
    canvas.style.max_zoom = 1.0;

    let opts = NodeGraphSetViewportOptions {
        duration_ms: Some(0),
        ..NodeGraphSetViewportOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_set_viewport_with_options(CanvasPoint::default(), 2.0, opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);
    assert!((snap.zoom - 1.0).abs() <= 1.0e-6);
}
