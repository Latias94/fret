use std::any::TypeId;
use std::sync::Arc;

use fret_core::{NodeId as UiNodeId, Point, Px, Rect, Scene, Size, Transform2D};
use fret_runtime::ModelId;
use fret_ui::Invalidation;
use fret_ui::UiTree;
use fret_ui::retained_bridge::Widget as _;

use crate::ui::internals::NodeGraphInternalsStore;
use crate::ui::measured::{MEASURED_GEOMETRY_EPSILON_PX, MeasuredGeometryStore};

use super::super::NodeGraphCanvas;
use super::{NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

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
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(800.0), Px(600.0)))
}

fn quant(v: f32) -> f32 {
    (v / MEASURED_GEOMETRY_EPSILON_PX).round() * MEASURED_GEOMETRY_EPSILON_PX
}

#[test]
fn measured_output_store_matches_internals_query_surfaces() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let measured = Arc::new(MeasuredGeometryStore::new());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
        .with_internals_store(internals.clone())
        .with_measured_output_store(measured.clone());

    let mut services = NullServices::default();
    let bounds = bounds_at(0.0, 0.0);

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    let rev1 = measured.revision();

    let snap = internals.snapshot();
    assert!(
        snap.nodes_window.contains_key(&a) && snap.nodes_window.contains_key(&b),
        "expected internals to publish node window rects"
    );
    assert!(
        snap.ports_window.contains_key(&a_in)
            && snap.ports_window.contains_key(&a_out)
            && snap.ports_window.contains_key(&b_in),
        "expected internals to publish port window rects"
    );
    assert!(
        snap.port_centers_window.contains_key(&a_in)
            && snap.port_centers_window.contains_key(&a_out)
            && snap.port_centers_window.contains_key(&b_in),
        "expected internals to publish port window centers"
    );

    for (&node, rect_window) in &snap.nodes_window {
        let Some((w, h)) = measured.node_size_px(node) else {
            panic!("expected measured_output.node_size_px to contain {node:?}");
        };
        assert!(
            (w - quant(rect_window.size.width.0)).abs() <= 1.0e-6
                && (h - quant(rect_window.size.height.0)).abs() <= 1.0e-6,
            "expected measured node size to match internals window rect size (quantized)"
        );
    }

    for (&port, rect_window) in &snap.ports_window {
        let Some(center_window) = snap.port_centers_window.get(&port).copied() else {
            panic!("expected internals.port_centers_window for {port:?}");
        };
        let Some(node) = canvas
            .graph
            .read_ref(&host, |g| g.ports.get(&port).map(|p| p.node))
            .ok()
            .flatten()
        else {
            panic!("expected graph to map port to node");
        };
        let Some(node_rect) = snap.nodes_window.get(&node).copied() else {
            panic!("expected internals.nodes_window for {node:?}");
        };

        let Some(anchor) = measured.port_anchor_px(port) else {
            panic!("expected measured_output.port_anchor_px to contain {port:?}");
        };

        let expected_center = Point::new(
            Px(quant(center_window.x.0 - node_rect.origin.x.0)),
            Px(quant(center_window.y.0 - node_rect.origin.y.0)),
        );
        let expected_bounds = Rect::new(
            Point::new(
                Px(quant(rect_window.origin.x.0 - node_rect.origin.x.0)),
                Px(quant(rect_window.origin.y.0 - node_rect.origin.y.0)),
            ),
            Size::new(
                Px(quant(rect_window.size.width.0)),
                Px(quant(rect_window.size.height.0)),
            ),
        );

        assert!(
            (anchor.center.x.0 - expected_center.x.0).abs() <= 1.0e-6
                && (anchor.center.y.0 - expected_center.y.0).abs() <= 1.0e-6,
            "expected measured port center to match internals (node-local, quantized)"
        );
        assert!(
            (anchor.bounds.origin.x.0 - expected_bounds.origin.x.0).abs() <= 1.0e-6
                && (anchor.bounds.origin.y.0 - expected_bounds.origin.y.0).abs() <= 1.0e-6
                && (anchor.bounds.size.width.0 - expected_bounds.size.width.0).abs() <= 1.0e-6
                && (anchor.bounds.size.height.0 - expected_bounds.size.height.0).abs() <= 1.0e-6,
            "expected measured port bounds to match internals (node-local, quantized)"
        );
    }

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    assert_eq!(
        measured.revision(),
        rev1,
        "expected identical paint not to rewrite measured output store"
    );
}
