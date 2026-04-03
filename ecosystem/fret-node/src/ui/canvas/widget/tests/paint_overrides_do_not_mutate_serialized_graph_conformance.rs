use std::sync::Arc;

use fret_core::{Point, Px, Rect, Scene, Size, Transform2D};
use fret_runtime::ui_host::GlobalsHost;
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, Theme, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::{
    EdgePaintOverrideV1, NodeGraphCanvas, NodeGraphPaintOverridesMap, NodePaintOverrideV1,
};

use super::{
    NullServices, TestUiHostImpl, insert_graph_view_editor_config_with,
    make_test_graph_two_nodes_with_ports,
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
fn paint_overrides_do_not_mutate_serialized_graph() {
    let (mut graph_value, a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
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
    host.set_global(Theme::global(&host).clone());

    let (graph, view, editor_config) =
        insert_graph_view_editor_config_with(&mut host, graph_value, |state| {
            state.runtime_tuning.only_render_visible_elements = false;
            state.interaction.frame_view_duration_ms = 0;
        });

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let overrides = Arc::new(NodeGraphPaintOverridesMap::default());
    let mut canvas = new_canvas!(host, graph.clone(), view, editor_config)
        .with_paint_overrides(overrides.clone());

    let baseline = graph
        .read_ref(&host, |g| serde_json::to_value(g).expect("serialize graph"))
        .expect("read graph baseline");

    overrides.set_node_override(
        a,
        Some(NodePaintOverrideV1 {
            body_background: Some(fret_core::Color::from_srgb_hex_rgb(0x12_34_56).into()),
            border_paint: Some(fret_core::Color::from_srgb_hex_rgb(0x65_43_21).into()),
            header_background: Some(fret_core::Color::from_srgb_hex_rgb(0xab_cd_ef).into()),
        }),
    );
    overrides.set_edge_override(
        EdgeId::new(),
        Some(EdgePaintOverrideV1 {
            stroke_paint: Some(fret_core::Color::from_srgb_hex_rgb(0xff_33_66).into()),
            ..Default::default()
        }),
    );

    overrides.set_edge_override(
        edge_id,
        Some(EdgePaintOverrideV1 {
            stroke_width_mul: Some(1.25),
            dash: Some(fret_core::scene::DashPatternV1::new(
                Px(6.0),
                Px(4.0),
                Px(0.0),
            )),
            ..Default::default()
        }),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let after = graph
        .read_ref(&host, |g| serde_json::to_value(g).expect("serialize graph"))
        .expect("read graph after");

    assert_eq!(
        baseline, after,
        "paint-only overrides must not mutate the serialized Graph"
    );

    // Sanity: changing overrides again must still keep the graph stable.
    overrides.set_edge_override(
        edge_id,
        Some(EdgePaintOverrideV1 {
            stroke_width_mul: Some(2.0),
            ..Default::default()
        }),
    );
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let after2 = graph
        .read_ref(&host, |g| serde_json::to_value(g).expect("serialize graph"))
        .expect("read graph after2");
    assert_eq!(
        baseline, after2,
        "paint-only overrides must not mutate the serialized Graph (after override revision)"
    );
}
