use std::sync::Arc;

use fret_core::scene::{Paint, PaintBindingV1, PaintEvalSpaceV1};
use fret_core::{Color, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::ui::{NodeGraphCanvas, NodeGraphPaintOverridesMap, NodePaintOverrideV1};

use super::{insert_view, make_test_graph_two_nodes_with_ports, NullServices, TestUiHostImpl};

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
fn paint_overrides_can_override_node_body_border_and_header_paint_bindings() {
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let overrides = Arc::new(NodeGraphPaintOverridesMap::default());
    let mut canvas = NodeGraphCanvas::new(graph, view).with_paint_overrides(overrides.clone());

    let body_paint = PaintBindingV1::with_eval_space(
        Paint::Solid(Color::from_srgb_hex_rgb(0x12_34_56)),
        PaintEvalSpaceV1::ViewportPx,
    );
    let border_paint = PaintBindingV1::with_eval_space(
        Paint::Solid(Color::from_srgb_hex_rgb(0x65_43_21)),
        PaintEvalSpaceV1::ViewportPx,
    );
    let header_paint = PaintBindingV1::with_eval_space(
        Paint::Solid(Color::from_srgb_hex_rgb(0xab_cd_ef)),
        PaintEvalSpaceV1::ViewportPx,
    );

    overrides.set_node_override(
        a,
        Some(NodePaintOverrideV1 {
            body_background: Some(body_paint),
            border_paint: Some(border_paint),
            header_background: Some(header_paint),
        }),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let mut found_body = false;
    let mut found_border = false;
    let mut found_header = false;
    for op in scene.ops() {
        let SceneOp::Quad {
            background,
            border_paint: stroke,
            ..
        } = op
        else {
            continue;
        };

        if background.eval_space == PaintEvalSpaceV1::ViewportPx
            && background.paint == body_paint.paint
        {
            found_body = true;
        }
        if stroke.eval_space == PaintEvalSpaceV1::ViewportPx && stroke.paint == border_paint.paint {
            found_border = true;
        }
        if background.eval_space == PaintEvalSpaceV1::ViewportPx
            && background.paint == header_paint.paint
        {
            found_header = true;
        }
    }

    assert!(
        found_body,
        "expected a node body quad to use the override PaintBindingV1"
    );
    assert!(
        found_border,
        "expected a node border quad to use the override PaintBindingV1"
    );
    assert!(
        found_header,
        "expected a node header quad to use the override PaintBindingV1"
    );
}
