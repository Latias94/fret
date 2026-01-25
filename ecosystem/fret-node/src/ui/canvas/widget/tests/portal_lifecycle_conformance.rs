use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{AppWindowId, Px, Rect};
use fret_runtime::ModelsHost as _;
use fret_ui::UiTree;
use fret_ui::element::SemanticsProps;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey};
use crate::io::NodeGraphViewState;
use crate::ui::measured::MeasuredGeometryStore;
use crate::ui::portal::NodeGraphPortalHost;
use crate::ui::style::NodeGraphStyle;

use super::{NullServices, TestUiHostImpl};

fn bounds() -> Rect {
    Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn portal_subtree_resets_state_on_node_kind_change() {
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
            kind: NodeKindKey::new("test.kind.a"),
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 20.0 },
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
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let measured = Arc::new(MeasuredGeometryStore::new());
    let next_instance = Arc::new(AtomicUsize::new(0));
    let last_instance = Arc::new(AtomicUsize::new(usize::MAX));

    let next_instance2 = next_instance.clone();
    let last_instance2 = last_instance.clone();
    let portal = NodeGraphPortalHost::new(
        graph.clone(),
        view.clone(),
        measured,
        NodeGraphStyle::default(),
        "test.portal.lifecycle",
        move |ecx: &mut fret_ui::ElementContext<'_, TestUiHostImpl>,
              _graph: &Graph,
              _layout: crate::ui::portal::NodeGraphPortalNodeLayout| {
            let id = ecx.with_state(|| next_instance2.fetch_add(1, Ordering::SeqCst), |s| *s);
            last_instance2.store(id, Ordering::SeqCst);
            vec![ecx.semantics(SemanticsProps::default(), |_ecx| Vec::new())]
        },
    )
    .with_cull_margin_px(0.0);

    let root = ui.create_node_retained(portal);
    ui.set_root(root);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    let first = last_instance.load(Ordering::SeqCst);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    let second = last_instance.load(Ordering::SeqCst);
    assert_eq!(
        first, second,
        "expected subtree state to persist across frames"
    );

    let _ = graph.update(&mut host, |g, _cx| {
        let node = g.nodes.get_mut(&node_id).expect("node exists");
        node.kind = NodeKindKey::new("test.kind.b");
    });
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    let third = last_instance.load(Ordering::SeqCst);
    assert_ne!(
        third, second,
        "expected subtree state to reset when node kind changes"
    );
}
