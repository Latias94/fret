use std::sync::Arc;

use fret_core::{Color, Corners, DrawOrder, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::ui::{NodeChromeHint, NodeGraphCanvas, NodeGraphSkin, NodeGraphStyle, NodeRingHint};

use super::{NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes};

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
struct FocusRingSkin {
    node: crate::core::NodeId,
}

impl NodeGraphSkin for FocusRingSkin {
    fn node_chrome_hint_with_state(
        &self,
        _graph: &crate::core::Graph,
        node: crate::core::NodeId,
        _style: &NodeGraphStyle,
        _selected: bool,
        focused: bool,
    ) -> NodeChromeHint {
        if node == self.node && focused {
            NodeChromeHint {
                ring_focused: Some(NodeRingHint {
                    color: Color {
                        r: 0.20,
                        g: 0.55,
                        b: 0.95,
                        a: 1.0,
                    },
                    width: 3.0,
                    pad: 6.0,
                }),
                ..NodeChromeHint::default()
            }
        } else {
            NodeChromeHint::default()
        }
    }
}

#[test]
fn skin_node_ring_hints_draws_focused_ring_outside_node_rect() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.runtime_tuning.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style.clone())
        .with_skin(Arc::new(FocusRingSkin { node: a }));

    // Focus is a UI-only interaction field, so we set it on the runtime interaction state.
    canvas.interaction.focused_node = Some(a);

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let rect_a = geom.nodes.get(&a).expect("node exists").rect;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let ring_color = Color {
        r: 0.20,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    };
    let expected_rect = Rect::new(
        Point::new(Px(rect_a.origin.x.0 - 6.0), Px(rect_a.origin.y.0 - 6.0)),
        Size::new(
            Px(rect_a.size.width.0 + 12.0),
            Px(rect_a.size.height.0 + 12.0),
        ),
    );
    let expected_corner = Corners::all(Px(style.paint.node_corner_radius + 6.0));

    let approx = |a: f32, b: f32| (a - b).abs() <= 1.0e-3;

    let mut hits = 0usize;
    for op in scene.ops().iter() {
        let SceneOp::Quad {
            order,
            rect,
            background,
            border,
            border_paint,
            corner_radii,
            ..
        } = op
        else {
            continue;
        };
        if *order != DrawOrder(3) {
            continue;
        }
        if background.paint != fret_core::Paint::TRANSPARENT {
            continue;
        }
        if *border != fret_core::Edges::all(Px(3.0)) {
            continue;
        }
        let fret_core::Paint::Solid(c) = border_paint.paint else {
            continue;
        };
        if !(approx(c.r, ring_color.r)
            && approx(c.g, ring_color.g)
            && approx(c.b, ring_color.b)
            && approx(c.a, ring_color.a))
        {
            continue;
        }
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
        assert_eq!(*corner_radii, expected_corner);
        hits += 1;
    }

    assert_eq!(hits, 1, "expected exactly one focused ring quad");
}
