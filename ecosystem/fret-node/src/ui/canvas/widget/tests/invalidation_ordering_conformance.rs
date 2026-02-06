use std::any::TypeId;
use std::sync::Arc;

use fret_core::{NodeId as UiNodeId, Px, Rect, Scene, Size, Transform2D};
use fret_runtime::ModelId;
use fret_ui::Invalidation;
use fret_ui::UiTree;
use fret_ui::retained_bridge::Widget as _;

use crate::ui::internals::NodeGraphInternalsStore;
use crate::ui::measured::{MeasuredGeometryApplyOptions, MeasuredGeometryExclusiveBatch};
use crate::ui::{DefaultNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter};

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
    Rect::new(
        fret_core::Point::new(Px(x), Px(y)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn measured_geometry_updates_are_observed_in_paint_without_layout() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let measured = Arc::new(MeasuredGeometryStore::new());

    let presenter =
        MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone());
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_presenter(presenter)
        .with_internals_store(internals.clone());

    let mut services = NullServices::default();
    let bounds = bounds_at(0.0, 0.0);

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    let rev1 = internals.revision();
    let rect1 = *internals
        .snapshot()
        .nodes_window
        .get(&a)
        .expect("node rect must exist");

    let next_w = rect1.size.width.0 + 200.0;
    let next_h = rect1.size.height.0 + 120.0;
    let _ = measured.apply_exclusive_batch_if_changed(
        MeasuredGeometryExclusiveBatch {
            node_sizes_px: vec![(a, (next_w, next_h))],
            port_anchors_px: Vec::new(),
        },
        MeasuredGeometryApplyOptions::default(),
    );

    paint_once(&mut canvas, &mut host, &mut services, bounds);
    assert!(internals.revision() > rev1);

    let rect2 = *internals
        .snapshot()
        .nodes_window
        .get(&a)
        .expect("node rect must exist");

    assert!(rect2.size.width.0 >= next_w - 1.0e-3);
    assert!(rect2.size.height.0 >= next_h - 1.0e-3);
}
