use std::sync::Arc;

use fret_core::scene::DashPatternV1;
use fret_core::{Point, Px, Rect, Scene, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::{EdgePaintOverrideV1, NodeGraphCanvas, NodeGraphPaintOverridesMap, NodeGraphStyle};

use super::{
    insert_editor_config_with, insert_view, make_test_graph_two_nodes_with_ports, NullServices,
    TestUiHostImpl,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut NullServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    let mut observe_model = |_id, _inv: Invalidation| {};
    let mut observe_global = |_id, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx::new(
        host,
        tree,
        fret_core::NodeId::default(),
        None,
        None,
        &[],
        bounds,
        1.0,
        Transform2D::IDENTITY,
        None,
        services,
        &mut observe_model,
        &mut observe_global,
        &mut scene,
    );

    canvas.paint(&mut cx);
    scene
}

#[test]
fn paint_overrides_revision_bump_does_not_rebuild_geometry_or_spatial_index() {
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = true;
        state.interaction.frame_view_duration_ms = 0;
    });

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let overrides = Arc::new(NodeGraphPaintOverridesMap::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
        .with_paint_overrides(overrides.clone())
        .with_editor_config_model(editor_config);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let counters1 = canvas.debug_derived_build_counters();

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let edges_clear_calls_0 = canvas.edges_scene_cache.stats().clear_calls;

    overrides.set_edge_override(
        edge_id,
        Some(
            EdgePaintOverrideV1 {
                dash: Some(DashPatternV1::new(Px(8.0), Px(4.0), Px(0.0))),
                ..Default::default()
            }
            .normalized(),
        ),
    );

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);
    let counters2 = canvas.debug_derived_build_counters();

    assert!(Arc::ptr_eq(&geom1, &geom2));
    assert!(Arc::ptr_eq(&index1, &index2));
    assert_eq!(
        counters2.geom_rebuilds, counters1.geom_rebuilds,
        "paint-only overrides must not rebuild derived geometry"
    );
    assert_eq!(
        counters2.index_rebuilds, counters1.index_rebuilds,
        "paint-only overrides must not rebuild the spatial index"
    );

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let edges_clear_calls_1 = canvas.edges_scene_cache.stats().clear_calls;
    assert!(
        edges_clear_calls_1 > edges_clear_calls_0,
        "paint override revision bump must invalidate static scene caches"
    );

    // Sanity: node graph style surface remains present (paint-only overrides must not replace it).
    let _style: NodeGraphStyle = canvas.style.clone();
}
