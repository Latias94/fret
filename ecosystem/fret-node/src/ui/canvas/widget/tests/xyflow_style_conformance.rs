use fret_core::{Corners, DrawOrder, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::ui::{NodeGraphCanvas, NodeGraphStyle};

use super::{NullServices, TestUiHostImpl, make_test_graph_two_nodes_with_size};

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

    let mut cx = fret_ui::retained_bridge::PaintCx {
        app: host,
        tree,
        node: fret_core::NodeId::default(),
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
    scene
}

#[test]
fn xyflow_default_node_style_sets_expected_tokens() {
    let style = NodeGraphStyle::default().with_xyflow_default_node_style();
    assert_eq!(style.node_width, 150.0);
    assert_eq!(style.node_padding, 10.0);
    assert_eq!(style.node_corner_radius, 3.0);
    assert_eq!(style.pin_radius, 3.0);
    assert_eq!(style.context_menu_text_style.size, Px(12.0));
}

#[test]
fn paint_uses_node_corner_radius_from_style() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let style = NodeGraphStyle::default().with_xyflow_default_node_style();
    let mut canvas = NodeGraphCanvas::new(graph, view).with_style(style.clone());

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let rect_a = geom.nodes.get(&a).expect("node a exists").rect;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let expected = Corners::all(Px(style.node_corner_radius));

    for op in scene.ops().iter() {
        let SceneOp::Quad {
            rect,
            order,
            corner_radii,
            ..
        } = op
        else {
            continue;
        };
        if *order != DrawOrder(3) {
            continue;
        }
        if *rect != rect_a {
            continue;
        }
        assert_eq!(*corner_radii, expected);
        return;
    }

    panic!("expected a node body quad for node a");
}
