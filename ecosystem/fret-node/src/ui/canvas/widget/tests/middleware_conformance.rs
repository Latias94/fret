use fret_core::AppWindowId;
use fret_runtime::CommandId;
use fret_ui::UiHost;
use fret_ui::retained_bridge::Widget as _;

use crate::ops::GraphTransaction;
use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};
use crate::ui::commands::CMD_NODE_GRAPH_NUDGE_RIGHT;
use crate::ui::{
    NodeGraphCanvasMiddlewareCx, canvas::NodeGraphCanvasCommitOutcome,
    canvas::NodeGraphCanvasMiddleware,
};

use super::{
    NullServices, TestUiHostImpl, command_cx, insert_graph_view_editor_config,
    make_test_graph_two_nodes, read_node_pos,
};

#[derive(Debug, Default, Clone, Copy)]
struct RejectNudgeCommit;

impl NodeGraphCanvasMiddleware for RejectNudgeCommit {
    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        if tx.label.as_deref() != Some("Nudge") {
            return NodeGraphCanvasCommitOutcome::Continue;
        }

        NodeGraphCanvasCommitOutcome::Reject {
            diagnostics: vec![Diagnostic {
                key: "middleware.reject_nudge".to_string(),
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message: "middleware rejected nudge transaction".to_string(),
                fixes: Vec::new(),
            }],
        }
    }
}

#[test]
fn middleware_can_reject_commits_before_apply() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);

    let mut canvas = new_canvas!(host, graph.clone(), view.clone(), editor_config)
        .with_middleware(RejectNudgeCommit);
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
    })
    .unwrap();

    let before_a = read_node_pos(&mut host, &graph, a);
    let before_b = read_node_pos(&mut host, &graph, b);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
    assert_eq!(canvas.history.undo_len(), 0);
    assert_eq!(read_node_pos(&mut host, &graph, a), before_a);
    assert_eq!(read_node_pos(&mut host, &graph, b), before_b);
}
