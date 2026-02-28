use fret_core::scene::DropShadowV1;
use fret_core::{
    EffectMode, EffectQuality, EffectStep, Point, Px, Rect, Scene, SceneOp, Size, Transform2D,
};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{CanvasPoint, Edge, EdgeId, EdgeKind};
use crate::ui::{NodeGraphCanvas, NodeGraphPresetFamily, NodeGraphPresetSkinV1};

use super::{
    NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports_spaced_x,
};

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
fn skin_wire_glow_selected_emits_push_effect_drop_shadow() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
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
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.selected_edges = vec![edge_id];
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let skin = NodeGraphPresetSkinV1::new_builtin(NodeGraphPresetFamily::WorkflowClean);
    let mut canvas = NodeGraphCanvas::new(graph, view).with_skin(skin);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let mut push_hits = 0usize;
    let mut pop_hits = 0usize;
    let mut blur_hits = 0usize;
    for op in scene.ops().iter() {
        match op {
            SceneOp::PushEffect {
                mode,
                chain,
                quality,
                ..
            } => {
                if *mode != EffectMode::FilterContent || *quality != EffectQuality::Auto {
                    continue;
                }
                if chain
                    .iter()
                    .any(|s| matches!(s, EffectStep::DropShadowV1(DropShadowV1 { .. })))
                {
                    push_hits += 1;
                }
                if chain.iter().any(|s| match s {
                    EffectStep::DropShadowV1(shadow) => {
                        (shadow.blur_radius_px.0 - 6.0).abs() <= 1.0e-3
                    }
                    _ => false,
                }) {
                    blur_hits += 1;
                }
            }
            SceneOp::PopEffect => pop_hits += 1,
            _ => {}
        }
    }

    assert_eq!(
        push_hits, 1,
        "expected exactly one PushEffect for selected wire glow"
    );
    assert_eq!(
        pop_hits, 1,
        "expected exactly one PopEffect for selected wire glow"
    );
    assert_eq!(
        blur_hits, 1,
        "expected DropShadow blur radius to match the preset glow"
    );
}
