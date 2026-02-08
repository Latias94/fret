use super::*;

fn availability_cx<'a>(
    host: &'a mut TestUiHostImpl,
    tree: &'a fret_ui::UiTree<TestUiHostImpl>,
) -> fret_ui::retained_bridge::CommandAvailabilityCx<'a, TestUiHostImpl> {
    let mut input_ctx = fret_runtime::InputContext::default();
    input_ctx.caps.clipboard.text = true;

    fret_ui::retained_bridge::CommandAvailabilityCx {
        app: host,
        tree,
        node: fret_core::NodeId::default(),
        window: Some(AppWindowId::default()),
        input_ctx,
        focus: Some(fret_core::NodeId::default()),
    }
}

#[test]
fn node_graph_blocks_edit_copy_without_selection() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let canvas = NodeGraphCanvas::new(graph, view);
    let tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

    let mut cx = availability_cx(&mut host, &tree);
    let availability = canvas.command_availability(&mut cx, &CommandId::from("edit.copy"));
    assert_eq!(
        availability,
        fret_ui::retained_bridge::CommandAvailability::Blocked
    );
}

#[test]
fn node_graph_enables_edit_copy_with_selected_nodes() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = host.models.update(&view, |state| {
        state.selected_nodes = vec![a];
    });

    let canvas = NodeGraphCanvas::new(graph, view);
    let tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

    let mut cx = availability_cx(&mut host, &tree);
    let availability = canvas.command_availability(&mut cx, &CommandId::from("edit.copy"));
    assert_eq!(
        availability,
        fret_ui::retained_bridge::CommandAvailability::Available
    );
}
