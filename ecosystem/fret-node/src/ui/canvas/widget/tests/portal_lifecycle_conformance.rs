use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{AppWindowId, Px, Rect};
use fret_runtime::ModelsHost as _;
use fret_ui::UiTree;
use fret_ui::element::SemanticsProps;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::retained_bridge::Widget as _;

use crate::core::{CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction};
use crate::runtime::store::NodeGraphStore;
use crate::ui::controller::NodeGraphController;
use crate::ui::edit_queue::NodeGraphEditQueue;
use crate::ui::measured::MeasuredGeometryStore;
use crate::ui::portal::{
    NodeGraphPortalCommandHandler, NodeGraphPortalHost, PortalCommandOutcome, PortalTextCommand,
    portal_submit_text_command,
};
use crate::ui::style::NodeGraphStyle;

use super::{
    NullServices, TestUiHostImpl, command_cx, insert_graph_view, insert_view,
    make_test_graph_two_nodes,
};

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
    let view = insert_view(&mut host);

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

#[derive(Debug, Clone, Copy)]
struct CommitMoveHandler {
    node: NodeId,
}

impl NodeGraphPortalCommandHandler<TestUiHostImpl> for CommitMoveHandler {
    fn handle_portal_command(
        &mut self,
        _cx: &mut fret_ui::retained_bridge::CommandCx<'_, TestUiHostImpl>,
        _graph: &Graph,
        command: PortalTextCommand,
    ) -> PortalCommandOutcome {
        match command {
            PortalTextCommand::Submit { node } if node == self.node => {
                PortalCommandOutcome::Commit(GraphTransaction {
                    label: Some("Move Node".to_string()),
                    ops: vec![GraphOp::SetNodePos {
                        id: node,
                        from: CanvasPoint { x: 0.0, y: 0.0 },
                        to: CanvasPoint { x: 64.0, y: 32.0 },
                    }],
                })
            }
            _ => PortalCommandOutcome::NotHandled,
        }
    }
}

#[test]
fn portal_command_prefers_controller_over_raw_edit_queue() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut tree = UiTree::<TestUiHostImpl>::default();

    let (graph_value, node_id, _other_node) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value.clone());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
    ));
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let controller = NodeGraphController::new(store.clone());

    let measured = Arc::new(MeasuredGeometryStore::new());
    let mut portal = NodeGraphPortalHost::new(
        graph.clone(),
        view,
        measured,
        NodeGraphStyle::default(),
        "test.portal.controller",
        move |ecx: &mut fret_ui::ElementContext<'_, TestUiHostImpl>,
              _graph: &Graph,
              _layout: crate::ui::portal::NodeGraphPortalNodeLayout| {
            vec![ecx.semantics(SemanticsProps::default(), |_ecx| Vec::new())]
        },
    )
    .with_edit_queue(edits.clone())
    .with_controller(controller)
    .with_command_handler(CommitMoveHandler { node: node_id });

    let mut cx = command_cx(&mut host, &mut services, &mut tree);
    assert!(portal.command(&mut cx, &portal_submit_text_command(node_id)));

    let queued = edits
        .read_ref(&host, |queue| queue.pending.clone())
        .ok()
        .unwrap_or_default();
    assert!(
        queued.is_empty(),
        "expected controller-first path to avoid pushing into the raw edit queue"
    );

    let graph_pos = graph
        .read_ref(&host, |graph| {
            graph.nodes.get(&node_id).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("graph node pos");
    assert_eq!(graph_pos, CanvasPoint { x: 64.0, y: 32.0 });

    let store_pos = store
        .read_ref(&host, |store| {
            store.graph().nodes.get(&node_id).map(|node| node.pos)
        })
        .ok()
        .flatten()
        .expect("store node pos");
    assert_eq!(store_pos, CanvasPoint { x: 64.0, y: 32.0 });
}
