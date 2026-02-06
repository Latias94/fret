use std::any::TypeId;
use std::sync::Arc;

use fret_core::{NodeId as UiNodeId, Px, Rect, Scene, Size, Transform2D};
use fret_runtime::ModelId;
use fret_ui::Invalidation;
use fret_ui::UiTree;
use fret_ui::retained_bridge::Widget as _;

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::ui::internals::NodeGraphInternalsStore;
use crate::ui::measured::MeasuredGeometryStore;

use super::super::NodeGraphCanvas;
use super::{NullServices, TestUiHostImpl, make_test_graph_two_nodes_with_ports};

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    services: &mut NullServices,
    bounds: Rect,
) {
    let mut tree: UiTree<TestUiHostImpl> = UiTree::new();
    let mut scene = Scene::default();
    let mut observe_model = |_id: ModelId, _inv: Invalidation| {};
    let mut observe_global = |_id: TypeId, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx {
        app: host,
        tree: &mut tree,
        node: UiNodeId::default(),
        window: None,
        focus: None,
        children: &[],
        bounds,
        scale_factor: 1.0,
        accumulated_transform: Transform2D::IDENTITY,
        children_render_transform: None,
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
        scene: &mut scene,
    };

    canvas.paint(&mut cx);
}

fn bounds_at(x: f32, y: f32) -> Rect {
    Rect::new(
        fret_core::Point::new(Px(x), Px(y)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn internals_store_is_stable_across_identical_paint() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let measured = Arc::new(MeasuredGeometryStore::new());

    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_internals_store(internals.clone())
        .with_measured_output_store(measured.clone());

    let mut services = NullServices::default();
    let bounds = bounds_at(0.0, 0.0);

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    let rev1 = internals.revision();
    let snap1 = internals.snapshot();
    let measured_rev1 = measured.revision();

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    assert_eq!(internals.revision(), rev1);
    assert_eq!(measured.revision(), measured_rev1);

    let snap2 = internals.snapshot();
    assert_eq!(snap2.transform, snap1.transform);
    assert_eq!(snap2.nodes_window.len(), snap1.nodes_window.len());
    assert_eq!(snap2.ports_window.len(), snap1.ports_window.len());
}

#[test]
fn pan_updates_internals_without_rebuilding_geometry_or_measured_output() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let measured = Arc::new(MeasuredGeometryStore::new());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
        .with_internals_store(internals.clone())
        .with_measured_output_store(measured.clone());

    let mut services = NullServices::default();
    let bounds = bounds_at(0.0, 0.0);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let counters1 = canvas.debug_derived_build_counters();

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    let rev1 = internals.revision();
    let measured_rev1 = measured.revision();
    let x1 = internals
        .snapshot()
        .nodes_window
        .get(&a)
        .expect("node rect must exist")
        .origin
        .x
        .0;

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: 50.0, y: -20.0 };
    });
    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);
    let counters2 = canvas.debug_derived_build_counters();

    assert!(Arc::ptr_eq(&geom1, &geom2));
    assert!(Arc::ptr_eq(&index1, &index2));
    assert_eq!(
        counters2.geom_rebuilds, counters1.geom_rebuilds,
        "pan-only must not rebuild geometry cache"
    );
    assert_eq!(
        counters2.index_rebuilds, counters1.index_rebuilds,
        "pan-only must not rebuild spatial index cache"
    );

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    assert!(internals.revision() > rev1);
    assert_eq!(measured.revision(), measured_rev1);

    let snap2 = internals.snapshot();
    assert_eq!(snap2.transform.pan, CanvasPoint { x: 50.0, y: -20.0 });
    let x2 = snap2
        .nodes_window
        .get(&a)
        .expect("node rect must exist")
        .origin
        .x
        .0;
    assert!((x2 - x1 - 50.0).abs() <= 1.0e-3);
}

#[test]
fn semantic_zoom_keeps_node_sizes_constant_in_window_space() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let measured = Arc::new(MeasuredGeometryStore::new());

    let mut canvas = NodeGraphCanvas::new(graph, view.clone())
        .with_internals_store(internals.clone())
        .with_measured_output_store(measured.clone());

    let mut services = NullServices::default();
    let bounds = bounds_at(0.0, 0.0);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    let rev1 = internals.revision();
    let measured_rev1 = measured.revision();

    let snap1 = internals.snapshot();
    let rect1 = *snap1.nodes_window.get(&b).expect("node rect must exist");

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 2.0;
    });
    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);

    assert!(!Arc::ptr_eq(&geom1, &geom2));
    assert!(!Arc::ptr_eq(&index1, &index2));

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    assert!(internals.revision() > rev1);
    assert_eq!(measured.revision(), measured_rev1);

    let snap2 = internals.snapshot();
    let rect2 = *snap2.nodes_window.get(&b).expect("node rect must exist");

    // Position scales (node lives in canvas space), but size stays constant in window space.
    assert!((rect2.origin.x.0 - rect1.origin.x.0 * 2.0).abs() <= 1.0e-3);
    assert!((rect2.size.width.0 - rect1.size.width.0).abs() <= 1.0e-3);
    assert!((rect2.size.height.0 - rect1.size.height.0).abs() <= 1.0e-3);
}

#[test]
fn bounds_origin_updates_internals_transform() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut canvas = NodeGraphCanvas::new(graph, view).with_internals_store(internals.clone());

    let mut services = NullServices::default();
    let b0 = bounds_at(0.0, 0.0);
    let b1 = bounds_at(120.0, 40.0);

    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    paint_once(&mut canvas, &mut host, &mut services, b0);
    let rev1 = internals.revision();
    let origin1 = internals.snapshot().transform.bounds_origin;

    paint_once(&mut canvas, &mut host, &mut services, b1);
    assert!(internals.revision() > rev1);
    let origin2 = internals.snapshot().transform.bounds_origin;

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    assert!(
        Arc::ptr_eq(&geom0, &geom1),
        "expected bounds changes to update internals without rebuilding geometry"
    );
    assert!(
        Arc::ptr_eq(&index0, &index1),
        "expected bounds changes to update internals without rebuilding spatial index"
    );

    assert_eq!(origin1.x.0, 0.0);
    assert_eq!(origin1.y.0, 0.0);
    assert_eq!(origin2.x.0, 120.0);
    assert_eq!(origin2.y.0, 40.0);
}

#[test]
fn graph_edit_rebuilds_geometry_and_updates_internals() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let measured = Arc::new(MeasuredGeometryStore::new());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
        .with_internals_store(internals.clone())
        .with_measured_output_store(measured.clone());

    let mut services = NullServices::default();
    let bounds = bounds_at(0.0, 0.0);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let counters1 = canvas.debug_derived_build_counters();

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    let rev1 = internals.revision();
    let measured_rev1 = measured.revision();
    let x1 = internals
        .snapshot()
        .nodes_window
        .get(&a)
        .expect("node rect must exist")
        .origin
        .x
        .0;

    let dx = 120.0;
    let _ = graph.update(&mut host, |g, _cx| {
        let n = g.nodes.get_mut(&a).expect("node must exist");
        n.pos.x += dx;
    });

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);
    let counters2 = canvas.debug_derived_build_counters();

    assert!(!Arc::ptr_eq(&geom1, &geom2));
    assert!(!Arc::ptr_eq(&index1, &index2));
    assert!(
        counters2.geom_rebuilds > counters1.geom_rebuilds,
        "graph edit should rebuild geometry cache"
    );
    assert!(
        counters2.index_rebuilds > counters1.index_rebuilds,
        "graph edit should rebuild spatial index cache"
    );

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    assert!(internals.revision() > rev1);
    assert_eq!(measured.revision(), measured_rev1);

    let x2 = internals
        .snapshot()
        .nodes_window
        .get(&a)
        .expect("node rect must exist")
        .origin
        .x
        .0;
    assert!((x2 - x1 - dx).abs() <= 1.0e-3);
}

#[test]
fn spatial_index_tuning_rebuilds_index_without_rebuilding_geometry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let counters1 = canvas.debug_derived_build_counters();

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.spatial_index.edge_aabb_pad_screen_px = 200.0;
    });
    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);
    let counters2 = canvas.debug_derived_build_counters();

    assert!(Arc::ptr_eq(&geom1, &geom2));
    assert!(!Arc::ptr_eq(&index1, &index2));
    assert_eq!(
        counters2.geom_rebuilds, counters1.geom_rebuilds,
        "spatial index tuning should not rebuild geometry cache"
    );
    assert!(
        counters2.index_rebuilds > counters1.index_rebuilds,
        "spatial index tuning should rebuild index cache"
    );
}
