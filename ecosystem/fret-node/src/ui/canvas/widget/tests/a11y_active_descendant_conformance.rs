use std::sync::Arc;

use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{NodeId, PortId};
use crate::ui::internals::{NodeGraphInternalsSnapshot, NodeGraphInternalsStore};
use crate::ui::{
    NodeGraphA11yFocusedEdge, NodeGraphA11yFocusedNode, NodeGraphA11yFocusedPort, NodeGraphCanvas,
    NodeGraphEditor,
};

use super::{NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn set_internals_focus(
    internals: &NodeGraphInternalsStore,
    node: Option<NodeId>,
    port: Option<PortId>,
) {
    let mut snap = internals.snapshot();
    snap.focused_node = node;
    snap.focused_port = port;
    snap.focused_edge = None;
    internals.update(snap);
}

fn mount_canvas_with_a11y_children(
    ui: &mut UiTree<TestUiHostImpl>,
    canvas: NodeGraphCanvas,
    internals: Arc<NodeGraphInternalsStore>,
) -> fret_core::NodeId {
    let canvas_node = ui.create_node_retained(canvas);
    let a11y_port = ui.create_node_retained(NodeGraphA11yFocusedPort::new(internals.clone()));
    let a11y_edge = ui.create_node_retained(NodeGraphA11yFocusedEdge::new(internals.clone()));
    let a11y_node = ui.create_node_retained(NodeGraphA11yFocusedNode::new(internals));
    ui.set_children(canvas_node, vec![a11y_port, a11y_edge, a11y_node]);
    canvas_node
}

#[test]
fn canvas_active_descendant_points_to_focused_port_semantics_node() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let (graph_value, a, a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.a11y_focused_port_label = Some("Port A".to_string());
    snap.a11y_focused_node_label = Some("Node A".to_string());
    internals.update(snap);
    set_internals_focus(&internals, Some(a), Some(a_in));

    let mut canvas = NodeGraphCanvas::new(graph, view).with_internals_store(internals.clone());
    canvas.interaction.focused_node = Some(a);
    canvas.interaction.focused_port = Some(a_in);
    canvas.interaction.focused_edge = None;
    let canvas_node = mount_canvas_with_a11y_children(&mut ui, canvas, internals);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let canvas_sem = snapshot
        .nodes
        .iter()
        .find(|n| n.id == canvas_node)
        .expect("canvas semantics node");
    let active_id = canvas_sem
        .active_descendant
        .expect("expected active_descendant for focused port");
    let active = snapshot
        .nodes
        .iter()
        .find(|n| n.id == active_id)
        .expect("active descendant semantics node");
    assert_eq!(active.label.as_deref(), Some("Port A"));
}

#[test]
fn canvas_active_descendant_points_to_focused_node_semantics_node() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.a11y_focused_node_label = Some("Node A".to_string());
    internals.update(snap);
    set_internals_focus(&internals, Some(a), None);

    let mut canvas = NodeGraphCanvas::new(graph, view).with_internals_store(internals.clone());
    canvas.interaction.focused_node = Some(a);
    canvas.interaction.focused_port = None;
    canvas.interaction.focused_edge = None;
    let canvas_node = mount_canvas_with_a11y_children(&mut ui, canvas, internals);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let canvas_sem = snapshot
        .nodes
        .iter()
        .find(|n| n.id == canvas_node)
        .expect("canvas semantics node");
    let active_id = canvas_sem
        .active_descendant
        .expect("expected active_descendant for focused node");
    let active = snapshot
        .nodes
        .iter()
        .find(|n| n.id == active_id)
        .expect("active descendant semantics node");
    assert_eq!(active.label.as_deref(), Some("Node A"));
}
