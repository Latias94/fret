use fret_core::{Corners, DrawOrder, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::CanvasPoint;

use crate::ui::{NodeGraphCanvas, NodeGraphStyle};

use super::{
    NullServices, TestUiHostImpl, insert_editor_config_with, insert_view,
    make_test_graph_two_nodes_with_size,
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
fn compact_default_node_style_sets_expected_tokens() {
    let style = NodeGraphStyle::default().with_compact_node_style();
    assert_eq!(style.geometry.node_width, 150.0);
    assert_eq!(style.geometry.node_padding, 10.0);
    assert_eq!(style.paint.node_corner_radius, 3.0);
    assert_eq!(style.geometry.pin_radius, 3.0);
    assert_eq!(style.geometry.context_menu_text_style.size, Px(12.0));
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
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = false;
        state.interaction.frame_view_duration_ms = 0;
    });

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
    });

    let style = NodeGraphStyle::default().with_compact_node_style();
    let mut canvas = new_canvas!(host, graph, view, editor_config).with_style(style.clone());

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let rect_a = geom.nodes.get(&a).expect("node a exists").rect;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let expected = Corners::all(Px(style.paint.node_corner_radius));

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
