use fret_canvas::view::{CanvasViewport2D, PanZoom2D};
use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, CanvasRect, CanvasSize};
use crate::ui::{NodeGraphSetViewportOptions, NodeGraphViewQueue};

use super::prelude::NodeGraphCanvas;
use super::{make_host_graph_view, make_test_graph_two_nodes_with_size};

fn rect_contains(outer: Rect, inner: Rect, eps: f32) -> bool {
    let outer_x0 = outer.origin.x.0;
    let outer_y0 = outer.origin.y.0;
    let outer_x1 = outer_x0 + outer.size.width.0;
    let outer_y1 = outer_y0 + outer.size.height.0;

    let inner_x0 = inner.origin.x.0;
    let inner_y0 = inner.origin.y.0;
    let inner_x1 = inner_x0 + inner.size.width.0;
    let inner_y1 = inner_y0 + inner.size.height.0;

    inner_x0 + eps >= outer_x0
        && inner_y0 + eps >= outer_y0
        && inner_x1 - eps <= outer_x1
        && inner_y1 - eps <= outer_y1
}

fn extent_rect(extent: CanvasRect) -> Rect {
    Rect::new(
        Point::new(Px(extent.origin.x), Px(extent.origin.y)),
        Size::new(Px(extent.size.width), Px(extent.size.height)),
    )
}

#[test]
fn set_viewport_clamps_pan_to_translate_extent() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let extent = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 1000.0,
            height: 1000.0,
        },
    };
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.translate_extent = Some(extent);
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);

    // At zoom=1, the viewport is 800x600 canvas units. With a 1000x1000 extent starting at 0,0:
    // - allowed pan_x is [-200, 0]
    // - allowed pan_y is [-400, 0]
    let pan = CanvasPoint {
        x: 500.0,
        y: -900.0,
    };
    let zoom = 1.0;
    let opts = NodeGraphSetViewportOptions {
        duration_ms: Some(0),
        ..NodeGraphSetViewportOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_set_viewport_with_options(pan, zoom, opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);
    assert!((snap.zoom - zoom).abs() <= 1.0e-6);
    assert!((snap.pan.x - 0.0).abs() <= 1.0e-6);
    assert!((snap.pan.y - (-400.0)).abs() <= 1.0e-6);

    let viewport = CanvasViewport2D::new(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(snap.pan.x), Px(snap.pan.y)),
            zoom: snap.zoom,
        },
    );
    assert!(
        rect_contains(extent_rect(extent), viewport.visible_canvas_rect(), 1.0e-3),
        "visible canvas rect must be clamped within the translate_extent"
    );
}

#[test]
fn set_viewport_clamps_pan_to_translate_extent_at_zoom() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let extent = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 1000.0,
            height: 1000.0,
        },
    };
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.translate_extent = Some(extent);
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);

    // At zoom=2, the viewport is 400x300 canvas units:
    // - allowed pan_x is [-600, 0]
    // - allowed pan_y is [-700, 0]
    let pan = CanvasPoint { x: -900.0, y: 50.0 };
    let zoom = 2.0;
    let opts = NodeGraphSetViewportOptions {
        duration_ms: Some(0),
        ..NodeGraphSetViewportOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_set_viewport_with_options(pan, zoom, opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);
    assert!((snap.zoom - zoom).abs() <= 1.0e-6);
    assert!((snap.pan.x - (-600.0)).abs() <= 1.0e-6);
    assert!((snap.pan.y - 0.0).abs() <= 1.0e-6);

    let viewport = CanvasViewport2D::new(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(snap.pan.x), Px(snap.pan.y)),
            zoom: snap.zoom,
        },
    );
    assert!(
        rect_contains(extent_rect(extent), viewport.visible_canvas_rect(), 1.0e-3),
        "visible canvas rect must be clamped within the translate_extent"
    );
}

#[test]
fn translate_extent_centers_when_viewport_is_larger_than_extent() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let extent = CanvasRect {
        origin: CanvasPoint { x: 10.0, y: 20.0 },
        size: CanvasSize {
            width: 100.0,
            height: 200.0,
        },
    };
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.translate_extent = Some(extent);
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_view_queue(queue.clone());
    canvas.interaction.last_bounds = Some(bounds);

    // Force the request to be non-noop relative to the default viewport so the view queue drains.
    let pan = CanvasPoint {
        x: 999.0,
        y: -999.0,
    };
    let zoom = 1.0;
    let opts = NodeGraphSetViewportOptions {
        duration_ms: Some(0),
        ..NodeGraphSetViewportOptions::default()
    };
    let _ = queue.update(&mut host, |q, _cx| {
        q.push_set_viewport_with_options(pan, zoom, opts)
    });

    assert!(canvas.drain_view_queue(&mut host, None));
    let snap = canvas.sync_view_state(&mut host);

    // When the viewport is larger than the extent, the extent is centered.
    let extent_center_x = extent.origin.x + 0.5 * extent.size.width;
    let extent_center_y = extent.origin.y + 0.5 * extent.size.height;
    let expected_pan_x = 0.5 * bounds.size.width.0 - extent_center_x;
    let expected_pan_y = 0.5 * bounds.size.height.0 - extent_center_y;

    assert!((snap.pan.x - expected_pan_x).abs() <= 1.0e-6);
    assert!((snap.pan.y - expected_pan_y).abs() <= 1.0e-6);

    let viewport = CanvasViewport2D::new(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(snap.pan.x), Px(snap.pan.y)),
            zoom: snap.zoom,
        },
    );
    let vis = viewport.visible_canvas_rect();
    let vis_center_x = vis.origin.x.0 + 0.5 * vis.size.width.0;
    let vis_center_y = vis.origin.y.0 + 0.5 * vis.size.height.0;
    assert!((vis_center_x - extent_center_x).abs() <= 1.0e-3);
    assert!((vis_center_y - extent_center_y).abs() <= 1.0e-3);
}
