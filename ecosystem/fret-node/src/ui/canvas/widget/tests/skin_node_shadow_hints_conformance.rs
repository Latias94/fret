use std::sync::Arc;

use fret_core::scene::DropShadowV1;
use fret_core::{
    Color, EffectMode, EffectQuality, EffectStep, Point, Px, Rect, Scene, SceneOp, Size,
    Transform2D,
};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::ui::{NodeChromeHint, NodeGraphCanvas, NodeGraphSkin, NodeGraphStyle, NodeShadowHint};

use super::{
    NullServices, TestUiHostImpl, insert_graph_view_editor_config_with, make_test_graph_two_nodes,
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

#[derive(Debug)]
struct ShadowSkin {
    node: crate::core::NodeId,
    shadow: NodeShadowHint,
}

impl NodeGraphSkin for ShadowSkin {
    fn node_chrome_hint(
        &self,
        _graph: &crate::core::Graph,
        node: crate::core::NodeId,
        _style: &NodeGraphStyle,
        _selected: bool,
    ) -> NodeChromeHint {
        if node == self.node {
            NodeChromeHint {
                shadow: Some(self.shadow),
                ..NodeChromeHint::default()
            }
        } else {
            NodeChromeHint::default()
        }
    }
}

#[test]
fn skin_node_shadow_hint_emits_push_effect_drop_shadow() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config_with(&mut host, graph_value, |state| {
            state.runtime_tuning.only_render_visible_elements = false;
            state.interaction.frame_view_duration_ms = 0;
        });
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let shadow = NodeShadowHint {
        offset_x_px: 0.0,
        offset_y_px: 2.0,
        blur_radius_px: 6.0,
        downsample: 2,
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.20,
        },
    };

    let style = NodeGraphStyle::default();
    let mut canvas = new_canvas!(host, graph, view, editor_config)
        .with_style(style)
        .with_skin(Arc::new(ShadowSkin { node: a, shadow }));

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let rect_a = geom.nodes.get(&a).expect("node exists").rect;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let approx = |a: f32, b: f32| (a - b).abs() <= 1.0e-3;
    let expected_bounds = Rect::new(
        Point::new(
            Px(rect_a.origin.x.0 - (shadow.blur_radius_px + shadow.offset_x_px.abs())),
            Px(rect_a.origin.y.0 - (shadow.blur_radius_px + shadow.offset_y_px.abs())),
        ),
        Size::new(
            Px(rect_a.size.width.0 + 2.0 * (shadow.blur_radius_px + shadow.offset_x_px.abs())),
            Px(rect_a.size.height.0 + 2.0 * (shadow.blur_radius_px + shadow.offset_y_px.abs())),
        ),
    );

    let mut push_hits = 0usize;
    let mut pop_hits = 0usize;
    for op in scene.ops().iter() {
        match op {
            SceneOp::PushEffect {
                bounds,
                mode,
                chain,
                quality,
            } => {
                if *mode != EffectMode::FilterContent || *quality != EffectQuality::Auto {
                    continue;
                }
                if !approx(bounds.origin.x.0, expected_bounds.origin.x.0)
                    || !approx(bounds.origin.y.0, expected_bounds.origin.y.0)
                {
                    continue;
                }
                if !approx(bounds.size.width.0, expected_bounds.size.width.0)
                    || !approx(bounds.size.height.0, expected_bounds.size.height.0)
                {
                    continue;
                }
                assert!(
                    chain
                        .iter()
                        .any(|s| matches!(s, EffectStep::DropShadowV1(DropShadowV1 { .. }))),
                    "expected a DropShadowV1 step in the effect chain"
                );
                push_hits += 1;
            }
            SceneOp::PopEffect => {
                pop_hits += 1;
            }
            SceneOp::Quad { order, .. } => {
                let _ = order;
            }
            _ => {}
        }
    }

    assert_eq!(
        push_hits, 1,
        "expected exactly one PushEffect for node shadow"
    );
    assert_eq!(
        pop_hits, 1,
        "expected exactly one PopEffect for node shadow"
    );
}
