use std::sync::Arc;

use fret_core::scene::{Paint, PaintBindingV1, PaintEvalSpaceV1};
use fret_core::{Color, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::edge_types::NodeGraphEdgeTypes;
use crate::ui::presenter::EdgeMarker;
use crate::ui::{EdgePaintOverrideV1, NodeGraphCanvas, NodeGraphPaintOverridesMap};

use super::{NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

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
fn paint_overrides_can_drive_edge_marker_paint_binding() {
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

    let overrides = Arc::new(NodeGraphPaintOverridesMap::default());
    let mut canvas = NodeGraphCanvas::new(graph, view).with_paint_overrides(overrides.clone());

    // Ensure the edge has both markers so the scene should contain:
    // - 1 wire path (paint override)
    // - 2 marker paths (marker paint binding)
    canvas = canvas.with_edge_types(NodeGraphEdgeTypes::new().with_fallback(
        |_g, _e, _style, mut h| {
            h.start_marker = Some(EdgeMarker::arrow(12.0));
            h.end_marker = Some(EdgeMarker::arrow(12.0));
            h
        },
    ));

    let paint = PaintBindingV1::with_eval_space(
        Paint::Solid(Color::from_srgb_hex_rgb(0x11_22_33)),
        PaintEvalSpaceV1::ViewportPx,
    );
    overrides.set_edge_override(
        edge_id,
        Some(EdgePaintOverrideV1 {
            stroke_paint: Some(paint),
            ..Default::default()
        }),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let mut matches: u32 = 0;
    for op in scene.ops() {
        let SceneOp::Path { paint: p, .. } = op else {
            continue;
        };
        if p.eval_space == PaintEvalSpaceV1::ViewportPx && p.paint == paint.paint {
            matches = matches.saturating_add(1);
        }
    }

    assert!(
        matches >= 3,
        "expected wire + start marker + end marker paths to use the override PaintBindingV1; matches={matches}"
    );
}
