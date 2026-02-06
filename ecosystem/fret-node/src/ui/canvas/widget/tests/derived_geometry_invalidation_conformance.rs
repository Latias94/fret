use std::sync::Arc;

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::ui::measured::{
    MEASURED_GEOMETRY_EPSILON_PX, MeasuredGeometryApplyOptions, MeasuredGeometryExclusiveBatch,
};
use crate::ui::{DefaultNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter};

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports};

#[test]
fn measured_geometry_revision_rebuilds_canvas_derived_geometry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let measured = Arc::new(MeasuredGeometryStore::new());
    let presenter =
        MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(presenter);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    let changed = measured.apply_exclusive_batch_if_changed(
        MeasuredGeometryExclusiveBatch {
            node_sizes_px: vec![(a, (420.0, 180.0))],
            port_anchors_px: Vec::new(),
        },
        MeasuredGeometryApplyOptions::default(),
    );
    assert!(
        changed.is_some(),
        "expected measured geometry update to bump revision"
    );

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);

    assert!(
        !Arc::ptr_eq(&geom1, &geom2),
        "expected measured geometry revision to invalidate cached CanvasGeometry"
    );
    assert!(
        !Arc::ptr_eq(&index1, &index2),
        "expected measured geometry revision to invalidate cached CanvasSpatialIndex"
    );
}

#[test]
fn measured_geometry_updates_within_epsilon_do_not_rebuild_canvas_derived_geometry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let measured = Arc::new(MeasuredGeometryStore::new());
    let presenter =
        MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(presenter);

    let _ = measured.apply_exclusive_batch_if_changed(
        MeasuredGeometryExclusiveBatch {
            node_sizes_px: vec![(a, (420.0, 180.0))],
            port_anchors_px: Vec::new(),
        },
        MeasuredGeometryApplyOptions::default(),
    );

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    let tiny = 0.5 * MEASURED_GEOMETRY_EPSILON_PX;
    let changed = measured.apply_exclusive_batch_if_changed(
        MeasuredGeometryExclusiveBatch {
            node_sizes_px: vec![(a, (420.0 + tiny, 180.0 + tiny))],
            port_anchors_px: Vec::new(),
        },
        MeasuredGeometryApplyOptions::default(),
    );
    assert!(
        changed.is_none(),
        "expected epsilon filtering to suppress tiny measured geometry updates"
    );

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);

    assert!(
        Arc::ptr_eq(&geom1, &geom2),
        "expected CanvasGeometry to remain cached when measured geometry revision does not change"
    );
    assert!(
        Arc::ptr_eq(&index1, &index2),
        "expected CanvasSpatialIndex to remain cached when measured geometry revision does not change"
    );
}

#[test]
fn pan_updates_do_not_rebuild_canvas_derived_geometry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: 123.0, y: 45.0 };
    });

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);

    assert!(
        Arc::ptr_eq(&geom1, &geom2),
        "expected pan-only changes to preserve cached CanvasGeometry"
    );
    assert!(
        Arc::ptr_eq(&index1, &index2),
        "expected pan-only changes to preserve cached CanvasSpatialIndex"
    );
}
