use std::sync::Arc;

use fret_core::{AppWindowId, Px, Rect};
use fret_runtime::ModelsHost as _;
use fret_ui::UiTree;
use fret_ui::element::{LayoutStyle, Length, SemanticsProps, SizeStyle};
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey};

use crate::ui::canvas::geometry::node_size_default_px;
use crate::ui::measured::MeasuredGeometryStore;
use crate::ui::portal::NodeGraphPortalHost;
use crate::ui::style::NodeGraphStyle;

use super::{NullServices, TestUiHostImpl, insert_view};

fn bounds() -> Rect {
    Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(800.0), Px(600.0)),
    )
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
fn portal_publishes_measured_node_sizes_as_growth_only_hints() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();

    let window = AppWindowId::default();
    ui.set_window(window);

    let mut graph_value = Graph::new(GraphId::new());
    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.portal.measured"),
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
    set_portal_content_size(&mut graph_value, node_id, 300.0, 200.0);

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let measured = Arc::new(MeasuredGeometryStore::new());
    let style = NodeGraphStyle::default();

    let portal = NodeGraphPortalHost::new(
        graph.clone(),
        view.clone(),
        measured.clone(),
        style.clone(),
        "test.portal.measured_geometry",
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
    let (w_px, h_px) = measured
        .node_size_px(node_id)
        .expect("expected portal measurement");
    assert!(
        w_px >= 300.0 && w_px >= min_w,
        "expected measured width to clamp to at least portal width and default min"
    );
    assert!(
        h_px >= 200.0 && h_px >= min_h,
        "expected measured height to clamp to at least portal height and default min"
    );

    // Shrink the portal subtree: the store must not shrink (growth-only hint).
    let _ = graph.update(&mut host, |g, _cx| {
        set_portal_content_size(g, node_id, 120.0, 80.0)
    });
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    let (w2, h2) = measured
        .node_size_px(node_id)
        .expect("expected portal measurement to remain present");
    assert_eq!(
        (w2, h2),
        (w_px, h_px),
        "expected portal measurements to be growth-only hints"
    );

    // Explicit node size should disable portal measurement publishing.
    let node2 = NodeId::new();
    let _ = graph.update(&mut host, |g, _cx| {
        g.nodes.insert(
            node2,
            Node {
                kind: NodeKindKey::new("test.portal.measured2"),
                kind_version: 1,
                pos: CanvasPoint { x: 50.0, y: 60.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 120.0,
                    height: 60.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::json!({
                    "portal": { "w_px": 500.0, "h_px": 400.0 }
                }),
            },
        );
    });
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    assert_eq!(
        measured.node_size_px(node2),
        None,
        "expected explicit node.size to disable portal measurement publishing"
    );

    // Removing a previously-published node should remove its measured entry.
    let _ = graph.update(&mut host, |g, _cx| {
        g.nodes.remove(&node_id);
    });
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    assert_eq!(
        measured.node_size_px(node_id),
        None,
        "expected removed node to be removed from MeasuredGeometryStore"
    );
}
