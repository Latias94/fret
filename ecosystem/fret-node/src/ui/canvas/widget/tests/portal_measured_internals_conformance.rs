use std::any::TypeId;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId as UiNodeId, Px, Rect, Scene, Size, Transform2D};
use fret_runtime::ModelId;
use fret_ui::Invalidation;
use fret_ui::UiTree;
use fret_ui::element::{LayoutStyle, Length, SemanticsProps, SizeStyle};
use fret_ui::retained_bridge::{UiTreeRetainedExt as _, Widget as _};

use crate::core::{CanvasPoint, Graph, GraphId, Node, NodeId, NodeKindKey};

use crate::ui::canvas::geometry::node_size_default_px;
use crate::ui::internals::NodeGraphInternalsStore;
use crate::ui::measured::{MeasuredGeometryStore, MeasuredNodeGraphPresenter};
use crate::ui::portal::NodeGraphPortalHost;
use crate::ui::presenter::DefaultNodeGraphPresenter;
use crate::ui::style::NodeGraphStyle;

use super::super::NodeGraphCanvas;
use super::{NullServices, TestUiHostImpl, insert_view};

fn bounds() -> Rect {
    Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    services: &mut NullServices,
    bounds: Rect,
) {
    let mut tree: UiTree<TestUiHostImpl> = UiTree::new();
    let mut scene = Scene::default();
    let mut observe_model = |_id: ModelId, _inv: Invalidation| {};
    let mut observe_global = |_id: TypeId, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx {
        app: host,
        tree: &mut tree,
        node: UiNodeId::default(),
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
}

fn set_portal_content_size(graph: &mut Graph, node: NodeId, w: f32, h: f32) {
    let node = graph.nodes.get_mut(&node).expect("node exists");
    node.data = serde_json::json!({
        "portal": { "w_px": w, "h_px": h }
    });
}

fn portal_content_size_px(graph: &Graph, node: NodeId) -> (f32, f32) {
    let node = graph.nodes.get(&node).expect("node exists");
    let portal = node
        .data
        .get("portal")
        .and_then(|v| v.as_object())
        .expect("expected node.data.portal object");
    let w = portal
        .get("w_px")
        .and_then(|v| v.as_f64())
        .expect("expected node.data.portal.w_px number") as f32;
    let h = portal
        .get("h_px")
        .and_then(|v| v.as_f64())
        .expect("expected node.data.portal.h_px number") as f32;
    (w, h)
}

#[test]
fn portal_measured_node_sizes_are_observed_by_canvas_internals_on_next_paint() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();

    let mut graph_value = Graph::new(GraphId::new());
    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.portal.measured_internals"),
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 20.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );
    set_portal_content_size(&mut graph_value, node_id, 320.0, 180.0);

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let measured = Arc::new(MeasuredGeometryStore::new());
    let internals = Arc::new(NodeGraphInternalsStore::new());
    let style = NodeGraphStyle::default();

    // Canvas uses the measured store via a presenter wrapper. This is the intended integration
    // surface: the canvas stays policy-light, while hosts can push measured geometry in.
    let presenter =
        MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
        .with_presenter(presenter)
        .with_internals_store(internals.clone());

    paint_once(&mut canvas, &mut host, &mut services, bounds());
    let before = *internals
        .snapshot()
        .nodes_window
        .get(&node_id)
        .expect("node rect must exist");

    // Layout the portal host, which publishes node size hints into `MeasuredGeometryStore`.
    let window = AppWindowId::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(window);

    let portal = NodeGraphPortalHost::new(
        graph.clone(),
        view,
        measured.clone(),
        style.clone(),
        "test.portal.measured_internals",
        move |ecx: &mut fret_ui::ElementContext<'_, TestUiHostImpl>,
              graph: &Graph,
              layout: crate::ui::portal::NodeGraphPortalNodeLayout| {
            let (w_px, h_px) = portal_content_size_px(graph, layout.node);
            let mut props = SemanticsProps::default();
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(w_px)),
                    height: Length::Px(Px(h_px)),
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![ecx.semantics(props, |_ecx| Vec::new())]
        },
    )
    .with_cull_margin_px(0.0);

    let root = ui.create_node_retained(portal);
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let (min_w, min_h) = node_size_default_px(0, 0, &style);
    let (measured_w, measured_h) = measured
        .node_size_px(node_id)
        .expect("expected measurement");
    assert!(measured_w >= 320.0 && measured_w >= min_w);
    assert!(measured_h >= 180.0 && measured_h >= min_h);

    // The canvas must observe the portal-published measurement without requiring a canvas layout
    // pass: a subsequent paint is sufficient to rebuild derived geometry and internals.
    paint_once(&mut canvas, &mut host, &mut services, bounds());
    let after = *internals
        .snapshot()
        .nodes_window
        .get(&node_id)
        .expect("node rect must exist");

    assert!(
        (after.size.width.0 - measured_w).abs() <= 1.0e-3,
        "expected internals node rect width to match measured store"
    );
    assert!(
        (after.size.height.0 - measured_h).abs() <= 1.0e-3,
        "expected internals node rect height to match measured store"
    );
    assert!(
        (before.size.width.0 - after.size.width.0).abs() > 1.0e-3
            || (before.size.height.0 - after.size.height.0).abs() > 1.0e-3,
        "expected the measured node size to affect canvas internals"
    );
}
