use fret_core::AppWindowId;
use fret_runtime::CommandId;
use fret_ui::UiHost;
use fret_ui::retained_bridge::Widget as _;

use crate::ops::GraphTransaction;
use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};
use crate::ui::commands::{CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_SELECT_ALL};
use crate::ui::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasMiddleware,
    NodeGraphCanvasMiddlewareCx,
};

use super::prelude::NodeGraphCanvas;
use super::{
    NullServices, TestUiHostImpl, command_cx, insert_graph_view, make_test_graph_two_nodes,
    read_node_pos,
};

#[derive(Debug, Clone, Copy)]
struct SelectAllOverride {
    target: crate::core::NodeId,
}

impl NodeGraphCanvasMiddleware for SelectAllOverride {
    fn handle_command<H: UiHost>(
        &mut self,
        cx: &mut fret_ui::retained_bridge::CommandCx<'_, H>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        command: &CommandId,
    ) -> NodeGraphCanvasCommandOutcome {
        if command.as_str() != CMD_NODE_GRAPH_SELECT_ALL {
            return NodeGraphCanvasCommandOutcome::NotHandled;
        }

        let target = self.target;
        let _ = ctx.view_state.update(cx.app, |s, _cx| {
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.push(target);
        });
        NodeGraphCanvasCommandOutcome::Handled
    }
}

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
fn middleware_can_override_select_all_command() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes.clear();
        s.selected_edges.clear();
        s.selected_groups.clear();
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
    })
    .unwrap();

    let mut canvas =
        NodeGraphCanvas::new(graph, view.clone()).with_middleware(SelectAllOverride { target: a });
    canvas.sync_view_state(&mut host);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_SELECT_ALL)));
    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);
}

#[test]
fn middleware_can_reject_commits_before_apply() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas =
        NodeGraphCanvas::new(graph.clone(), view.clone()).with_middleware(RejectNudgeCommit);
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
