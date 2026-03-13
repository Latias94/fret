use std::sync::Arc;

use fret_core::{Color, DrawOrder, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::ui::{
    NodeGraphCanvas, NodeGraphPresetFamily, NodeGraphPresetSkinV1, NodeGraphSkin, NodeGraphStyle,
    PortChromeHint, PortShapeHint,
};

use super::{NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

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
struct PortHintSkin {
    target: crate::core::PortId,
}

impl NodeGraphSkin for PortHintSkin {
    fn port_chrome_hint(
        &self,
        _graph: &crate::core::Graph,
        port: crate::core::PortId,
        _style: &NodeGraphStyle,
        base_fill: Color,
    ) -> PortChromeHint {
        if port == self.target {
            PortChromeHint {
                fill: Some(Color {
                    r: 0.20,
                    g: 0.55,
                    b: 0.95,
                    a: 1.0,
                }),
                stroke: Some(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                stroke_width: Some(2.0),
                inner_scale: Some(0.6),
                ..PortChromeHint::default()
            }
        } else {
            PortChromeHint {
                // Prove that non-target ports remain based on the presenter default unless
                // overridden.
                fill: Some(base_fill),
                ..PortChromeHint::default()
            }
        }
    }
}

#[derive(Debug)]
struct PortShapeHintSkin {
    target: crate::core::PortId,
    shape: PortShapeHint,
}

impl NodeGraphSkin for PortShapeHintSkin {
    fn port_chrome_hint(
        &self,
        _graph: &crate::core::Graph,
        port: crate::core::PortId,
        _style: &NodeGraphStyle,
        base_fill: Color,
    ) -> PortChromeHint {
        if port == self.target {
            PortChromeHint {
                fill: Some(Color {
                    r: 0.20,
                    g: 0.55,
                    b: 0.95,
                    a: 1.0,
                }),
                stroke: Some(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                stroke_width: Some(2.0),
                inner_scale: Some(0.6),
                shape: Some(self.shape),
            }
        } else {
            PortChromeHint {
                fill: Some(base_fill),
                ..PortChromeHint::default()
            }
        }
    }
}

#[test]
fn skin_port_chrome_hints_apply_fill_stroke_and_inner_scale_paint_only() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.runtime_tuning.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style)
        .with_skin(Arc::new(PortHintSkin { target: a_out }));

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let port_bounds = geom.ports.get(&a_out).expect("port exists").bounds;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let expected_fill = Color {
        r: 0.20,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    };
    let expected_stroke = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    let cx = port_bounds.origin.x.0 + 0.5 * port_bounds.size.width.0;
    let cy = port_bounds.origin.y.0 + 0.5 * port_bounds.size.height.0;
    let expected_w = port_bounds.size.width.0 * 0.6;
    let expected_h = port_bounds.size.height.0 * 0.6;
    let expected_rect = Rect::new(
        Point::new(Px(cx - 0.5 * expected_w), Px(cy - 0.5 * expected_h)),
        Size::new(Px(expected_w), Px(expected_h)),
    );
    let expected_outer_rect = port_bounds;

    let approx = |a: f32, b: f32| (a - b).abs() <= 1.0e-3;

    let mut fill_hits = 0usize;
    let mut stroke_hits = 0usize;
    for op in scene.ops().iter() {
        let SceneOp::Quad {
            order,
            rect,
            background,
            border,
            border_paint,
            ..
        } = op
        else {
            continue;
        };
        if *order != DrawOrder(4) {
            continue;
        }
        let background = background.paint;
        let border_paint = border_paint.paint;
        match (background, border_paint) {
            (fret_core::Paint::Solid(c), _) if *border == fret_core::Edges::all(Px(0.0)) => {
                if !approx(rect.origin.x.0, expected_rect.origin.x.0)
                    || !approx(rect.origin.y.0, expected_rect.origin.y.0)
                {
                    continue;
                }
                if !approx(rect.size.width.0, expected_rect.size.width.0)
                    || !approx(rect.size.height.0, expected_rect.size.height.0)
                {
                    continue;
                }
                if approx(c.r, expected_fill.r)
                    && approx(c.g, expected_fill.g)
                    && approx(c.b, expected_fill.b)
                    && approx(c.a, expected_fill.a)
                {
                    fill_hits += 1;
                }
            }
            (fret_core::Paint::TRANSPARENT, fret_core::Paint::Solid(c))
                if *border == fret_core::Edges::all(Px(2.0)) =>
            {
                if !approx(rect.origin.x.0, expected_outer_rect.origin.x.0)
                    || !approx(rect.origin.y.0, expected_outer_rect.origin.y.0)
                {
                    continue;
                }
                if !approx(rect.size.width.0, expected_outer_rect.size.width.0)
                    || !approx(rect.size.height.0, expected_outer_rect.size.height.0)
                {
                    continue;
                }
                if approx(c.r, expected_stroke.r)
                    && approx(c.g, expected_stroke.g)
                    && approx(c.b, expected_stroke.b)
                    && approx(c.a, expected_stroke.a)
                {
                    stroke_hits += 1;
                }
            }
            _ => {}
        }
    }

    assert_eq!(fill_hits, 1, "expected exactly one inner-scaled fill quad");
    assert_eq!(stroke_hits, 1, "expected exactly one stroke overlay quad");
}

#[test]
fn skin_port_shape_hint_renders_path_ops_for_non_circle_shapes() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.runtime_tuning.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style)
        .with_skin(Arc::new(PortShapeHintSkin {
            target: a_out,
            shape: PortShapeHint::Diamond,
        }));

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let port_bounds = geom.ports.get(&a_out).expect("port exists").bounds;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let expected_fill = Color {
        r: 0.20,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    };
    let expected_stroke = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    let cx = port_bounds.origin.x.0 + 0.5 * port_bounds.size.width.0;
    let cy = port_bounds.origin.y.0 + 0.5 * port_bounds.size.height.0;
    let expected_w = port_bounds.size.width.0 * 0.6;
    let expected_h = port_bounds.size.height.0 * 0.6;
    let expected_rect = Rect::new(
        Point::new(Px(cx - 0.5 * expected_w), Px(cy - 0.5 * expected_h)),
        Size::new(Px(expected_w), Px(expected_h)),
    );
    let expected_outer_rect = port_bounds;

    let approx = |a: f32, b: f32| (a - b).abs() <= 1.0e-3;

    let mut fill_path_hits = 0usize;
    let mut stroke_path_hits = 0usize;
    for op in scene.ops() {
        match op {
            SceneOp::Path {
                order: DrawOrder(4),
                origin,
                paint,
                ..
            } => match paint.paint {
                fret_core::Paint::Solid(c) if c == expected_fill => {
                    if approx(origin.x.0, expected_rect.origin.x.0)
                        && approx(origin.y.0, expected_rect.origin.y.0)
                    {
                        fill_path_hits += 1;
                    }
                }
                fret_core::Paint::Solid(c) if c == expected_stroke => {
                    if approx(origin.x.0, expected_outer_rect.origin.x.0)
                        && approx(origin.y.0, expected_outer_rect.origin.y.0)
                    {
                        stroke_path_hits += 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    assert!(
        fill_path_hits >= 1,
        "expected at least one fill Path op for the shaped pin"
    );
    assert!(
        stroke_path_hits >= 1,
        "expected at least one stroke Path op for the shaped pin"
    );
}

#[test]
fn preset_exec_ports_use_triangle_shape_and_emit_path_ops() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    graph_value
        .ports
        .entry(a_out)
        .and_modify(|p| p.kind = crate::core::PortKind::Exec);
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.runtime_tuning.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let skin = NodeGraphPresetSkinV1::new_builtin(NodeGraphPresetFamily::WorkflowClean);
    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style)
        .with_skin(skin);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let mut path_ops = 0usize;
    for op in scene.ops() {
        if matches!(
            op,
            SceneOp::Path {
                order: DrawOrder(4),
                ..
            }
        ) {
            path_ops += 1;
        }
    }

    assert!(
        path_ops > 0,
        "expected at least one Path op for a non-circle preset port (exec should be Triangle)"
    );
}
