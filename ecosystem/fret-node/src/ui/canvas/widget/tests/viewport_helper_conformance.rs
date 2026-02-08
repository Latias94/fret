use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;
use crate::ui::{
    NodeGraphSetViewportOptions, NodeGraphViewQueue, NodeGraphViewRequest, NodeGraphViewportHelper,
};

use super::{TestUiHostImpl, insert_view};

#[test]
fn viewport_helper_set_center_uses_current_zoom_when_omitted() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let view = insert_view(&mut host);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 2.0;
    });

    let helper = NodeGraphViewportHelper::new(view, queue.clone());
    helper.set_center_in_bounds_with_options(
        &mut host,
        bounds,
        CanvasPoint { x: 10.0, y: 20.0 },
        None,
        NodeGraphSetViewportOptions {
            duration_ms: Some(0),
            ..NodeGraphSetViewportOptions::default()
        },
    );

    let pending = queue
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1);

    let NodeGraphViewRequest::SetViewport { pan, zoom, .. } = pending[0].clone() else {
        panic!("expected SetViewport request");
    };
    assert!((zoom - 2.0).abs() <= 1.0e-6);
    assert!((pan.x - (800.0 / 4.0 - 10.0)).abs() <= 1.0e-6);
    assert!((pan.y - (600.0 / 4.0 - 20.0)).abs() <= 1.0e-6);
}

#[test]
fn viewport_helper_set_center_honors_explicit_zoom_override() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let view = insert_view(&mut host);
    let queue = host.models.insert(NodeGraphViewQueue::default());

    let helper = NodeGraphViewportHelper::new(view, queue.clone());
    helper.set_center_in_bounds_with_options(
        &mut host,
        bounds,
        CanvasPoint { x: 10.0, y: 20.0 },
        Some(4.0),
        NodeGraphSetViewportOptions {
            duration_ms: Some(0),
            ..NodeGraphSetViewportOptions::default()
        },
    );

    let pending = queue
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1);

    let NodeGraphViewRequest::SetViewport { pan, zoom, .. } = pending[0].clone() else {
        panic!("expected SetViewport request");
    };
    assert!((zoom - 4.0).abs() <= 1.0e-6);
    assert!((pan.x - (800.0 / 8.0 - 10.0)).abs() <= 1.0e-6);
    assert!((pan.y - (600.0 / 8.0 - 20.0)).abs() <= 1.0e-6);
}
