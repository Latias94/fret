#[cfg(feature = "compat-retained-canvas")]
use fret_runtime::Model;
#[cfg(feature = "compat-retained-canvas")]
use fret_ui::UiHost;

#[cfg(feature = "compat-retained-canvas")]
use crate::ui::compat_transport::NodeGraphEditQueue;
#[cfg(feature = "compat-retained-canvas")]
use crate::ui::controller::NodeGraphController;

#[cfg(feature = "compat-retained-canvas")]
use super::group_rename::NodeGraphOverlayState;
#[cfg(feature = "compat-retained-canvas")]
use super::rename_command::RenameHostKeyDecision;
#[cfg(feature = "compat-retained-canvas")]
use super::rename_command::{
    RenameCommandOutcome, apply_rename_host_key_decision as apply_rename_host_key_decision_in_state,
};

#[cfg(feature = "compat-retained-canvas")]
pub(super) fn apply_rename_host_key_decision<H: UiHost>(
    host: &mut H,
    decision: RenameHostKeyDecision,
    graph: &Model<crate::Graph>,
    rename_text: &Model<String>,
    overlays: &Model<NodeGraphOverlayState>,
    controller: Option<&NodeGraphController>,
    edits: Option<&Model<NodeGraphEditQueue>>,
) -> bool {
    let graph_snapshot = graph
        .read_ref(host, |graph| graph.clone())
        .ok()
        .unwrap_or_default();
    let rename_text = rename_text
        .read_ref(host, |text| text.clone())
        .ok()
        .unwrap_or_default();

    let outcome = overlays
        .update(host, |state, _cx| {
            apply_rename_host_key_decision_in_state(&graph_snapshot, state, &rename_text, decision)
        })
        .ok()
        .unwrap_or(RenameCommandOutcome::NotHandled);

    match outcome {
        RenameCommandOutcome::NotHandled => false,
        RenameCommandOutcome::Handled => true,
        RenameCommandOutcome::Commit(tx) => {
            crate::ui::retained_submit::submit_graph_transaction(
                host, controller, edits, graph, &tx,
            );
            true
        }
    }
}

#[cfg(feature = "compat-retained-canvas")]
pub(super) fn close_rename_host_sessions<H: UiHost>(
    host: &mut H,
    overlays: &Model<NodeGraphOverlayState>,
) {
    let _ = overlays.update(host, |state, _cx| {
        crate::ui::overlays::rename_policy::clear_rename_sessions(state);
    });
}

#[cfg(test)]
mod tests {
    use super::super::rename_command::{RenameHostKeyDecision, decide_rename_host_key};
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
