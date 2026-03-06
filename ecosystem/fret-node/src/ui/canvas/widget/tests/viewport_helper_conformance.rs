use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::NodeGraphController;
use crate::ui::view_queue::{
    NodeGraphSetViewportOptions, NodeGraphViewQueue, NodeGraphViewRequest,
};
use crate::ui::viewport_helper::NodeGraphViewportHelper;

use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes};

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

#[test]
fn viewport_helper_from_controller_falls_back_to_store_without_queue() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
    let mut store_view = NodeGraphViewState::default();
    store_view.zoom = 2.0;
    let store = host
        .models
        .insert(NodeGraphStore::new(graph_value, store_view));
    let helper = NodeGraphViewportHelper::from_controller(NodeGraphController::new(store.clone()));

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

    let (pan, zoom) = store
        .read_ref(&host, |store| {
            let view = store.view_state();
            (view.pan, view.zoom)
        })
        .ok()
        .unwrap_or((CanvasPoint::default(), 1.0));
    assert!((zoom - 2.0).abs() <= 1.0e-6);
    assert!((pan.x - (800.0 / 4.0 - 10.0)).abs() <= 1.0e-6);
    assert!((pan.y - (600.0 / 4.0 - 20.0)).abs() <= 1.0e-6);
}

#[test]
fn viewport_helper_from_controller_preserves_queue_transport_when_present() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _node_a, _node_b) = make_test_graph_two_nodes();
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
    ));
    let queue = host.models.insert(NodeGraphViewQueue::default());
    let helper = NodeGraphViewportHelper::from_controller(
        NodeGraphController::new(store.clone()).with_view_queue(queue.clone()),
    );

    helper.set_viewport_with_options(
        &mut host,
        CanvasPoint { x: 10.0, y: 20.0 },
        1.5,
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
    assert_eq!(pan, CanvasPoint { x: 10.0, y: 20.0 });
    assert!((zoom - 1.5).abs() <= 1.0e-6);
    assert_eq!(
        store
            .read_ref(&host, |store| {
                let view = store.view_state();
                (view.pan, view.zoom)
            })
            .ok()
            .unwrap_or((CanvasPoint::default(), 1.0)),
        (CanvasPoint::default(), 1.0)
    );
}
