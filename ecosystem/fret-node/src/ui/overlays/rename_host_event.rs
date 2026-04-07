use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::ops::GraphTransaction;
use crate::ui::compat_transport::NodeGraphEditQueue;
use crate::ui::controller::NodeGraphController;

use super::group_rename::NodeGraphOverlayState;
use super::rename_policy::{
    RenameOverlaySession, build_rename_commit_transaction, clear_rename_sessions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenameHostKeyDecision {
    Close,
    CommitAndClose,
    Ignore,
}

pub(super) fn decide_rename_host_key(key: KeyCode) -> RenameHostKeyDecision {
    match key {
        KeyCode::Escape => RenameHostKeyDecision::Close,
        KeyCode::Enter | KeyCode::NumpadEnter => RenameHostKeyDecision::CommitAndClose,
        _ => RenameHostKeyDecision::Ignore,
    }
}

pub(super) fn apply_rename_host_key_decision<H: UiHost>(
    host: &mut H,
    decision: RenameHostKeyDecision,
    graph: &Model<crate::Graph>,
    rename_text: &Model<String>,
    overlays: &Model<NodeGraphOverlayState>,
    session: &RenameOverlaySession,
    controller: Option<&NodeGraphController>,
    edits: Option<&Model<NodeGraphEditQueue>>,
) -> bool {
    match decision {
        RenameHostKeyDecision::Ignore => false,
        RenameHostKeyDecision::Close => {
            close_rename_host_sessions(host, overlays);
            true
        }
        RenameHostKeyDecision::CommitAndClose => {
            if let Some(tx) = rename_commit_transaction(host, graph, rename_text, session) {
                submit_rename_transaction(host, graph, controller, edits, &tx);
            }
            close_rename_host_sessions(host, overlays);
            true
        }
    }
}

fn rename_commit_transaction<H: UiHost>(
    host: &H,
    graph: &Model<crate::Graph>,
    rename_text: &Model<String>,
    session: &RenameOverlaySession,
) -> Option<GraphTransaction> {
    let to = rename_text
        .read_ref(host, |text| text.clone())
        .ok()
        .unwrap_or_default();
    graph
        .read_ref(host, |g| build_rename_commit_transaction(g, session, &to))
        .ok()
        .flatten()
}

fn submit_rename_transaction<H: UiHost>(
    host: &mut H,
    graph: &Model<crate::Graph>,
    controller: Option<&NodeGraphController>,
    edits: Option<&Model<NodeGraphEditQueue>>,
    tx: &GraphTransaction,
) {
    crate::ui::retained_submit::submit_graph_transaction(host, controller, edits, graph, tx);
}

pub(super) fn close_rename_host_sessions<H: UiHost>(
    host: &mut H,
    overlays: &Model<NodeGraphOverlayState>,
) {
    let _ = overlays.update(host, |state, _cx| {
        clear_rename_sessions(state);
    });
}

#[cfg(test)]
mod tests {
    use super::{RenameHostKeyDecision, decide_rename_host_key};
    use fret_core::KeyCode;

    #[test]
    fn rename_host_key_decision_routes_escape_enter_and_other_keys() {
        assert_eq!(
            decide_rename_host_key(KeyCode::Escape),
            RenameHostKeyDecision::Close
        );
        assert_eq!(
            decide_rename_host_key(KeyCode::Enter),
            RenameHostKeyDecision::CommitAndClose
        );
        assert_eq!(
            decide_rename_host_key(KeyCode::NumpadEnter),
            RenameHostKeyDecision::CommitAndClose
        );
        assert_eq!(
            decide_rename_host_key(KeyCode::Tab),
            RenameHostKeyDecision::Ignore
        );
    }
}
