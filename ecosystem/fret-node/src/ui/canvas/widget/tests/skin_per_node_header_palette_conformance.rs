use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use fret_core::{Color, Corners, DrawOrder, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::NodeKindKey;
use crate::ui::{NodeChromeHint, NodeGraphCanvas, NodeGraphSkin, NodeGraphStyle};

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
struct HeaderPaletteSkin {
    rev: AtomicU64,
}

impl Default for HeaderPaletteSkin {
    fn default() -> Self {
        Self {
            rev: AtomicU64::new(1),
        }
    }
}

impl NodeGraphSkin for HeaderPaletteSkin {
    fn revision(&self) -> u64 {
        self.rev.load(Ordering::Relaxed)
    }

    fn node_chrome_hint(
        &self,
        graph: &crate::core::Graph,
        node: crate::core::NodeId,
        _style: &NodeGraphStyle,
        _selected: bool,
    ) -> NodeChromeHint {
        let kind = graph
            .nodes
            .get(&node)
            .map(|n| n.kind.clone())
            .unwrap_or_else(|| NodeKindKey::new("missing"));
        if kind.0.as_str() == "test.a" {
            NodeChromeHint {
                header_background: Some(Color {
                    r: 0.20,
                    g: 0.55,
                    b: 0.95,
                    a: 1.0,
                }),
                title_text: Some(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                ..NodeChromeHint::default()
            }
        } else {
            NodeChromeHint {
                header_background: Some(Color {
                    r: 0.95,
                    g: 0.75,
                    b: 0.20,
                    a: 1.0,
                }),
                title_text: Some(Color {
                    r: 0.10,
                    g: 0.10,
                    b: 0.10,
                    a: 1.0,
                }),
                ..NodeChromeHint::default()
            }
        }
    }
}

#[test]
fn per_node_header_palette_draws_distinct_header_quads() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes();
    if let Some(node) = graph_value.nodes.get_mut(&a) {
        node.kind = NodeKindKey::new("test.a");
    }
    if let Some(node) = graph_value.nodes.get_mut(&b) {
        node.kind = NodeKindKey::new("test.b");
    }

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style.clone())
        .with_skin(Arc::new(HeaderPaletteSkin::default()));

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let rect_a = geom.nodes.get(&a).expect("node a exists").rect;
    let rect_b = geom.nodes.get(&b).expect("node b exists").rect;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let mut headers = Vec::new();
    for op in scene.ops().iter() {
        let SceneOp::Quad {
            order,
            rect,
            background,
            border,
            corner_radii,
            ..
        } = op
        else {
            continue;
        };
        if *order != DrawOrder(3) {
            continue;
        }
        if *border != fret_core::Edges::all(Px(0.0)) {
            continue;
        }
        let fret_core::Paint::Solid(color) = background.paint else {
            continue;
        };
        if rect.origin != rect_a.origin && rect.origin != rect_b.origin {
            continue;
        }
        if (rect.size.height.0 - style.geometry.node_header_height).abs() > 1.0e-3 {
            continue;
        }
        let expected_corner = Corners {
            top_left: Px(style.paint.node_corner_radius),
            top_right: Px(style.paint.node_corner_radius),
            bottom_right: Px(0.0),
            bottom_left: Px(0.0),
        };
        assert_eq!(*corner_radii, expected_corner);
        headers.push((rect.origin, color));
    }

    assert_eq!(
        headers.len(),
        2,
        "expected one header quad per node with header_background hint"
    );
    assert!(
        headers
            .iter()
            .any(|(o, c)| *o == rect_a.origin && (c.b - 0.95).abs() < 1.0e-3)
    );
    assert!(
        headers
            .iter()
            .any(|(o, c)| *o == rect_b.origin && (c.g - 0.75).abs() < 1.0e-3)
    );
}
