use std::sync::Arc;

use fret_app::App;

use super::recent_evidence::{
    RecentEvidenceTarget, devtools_recent_failed_evidence_target, recent_evidence_failing_count,
    recent_evidence_next_action, recent_failed_evidence_rerun_command_from_state,
    recent_failed_evidence_rerun_unavailable_reason_from_state,
};
use super::{
    State, devtools_first_open_next_action_lines, devtools_workflow_commands_from_state,
    regression_failing_summary_rows, selected_followup_result_loaded_from_state,
};

pub(super) struct HeaderDiagnosticsState {
    pub(super) has_session: bool,
    pub(super) selected_session: Option<Arc<str>>,
    pub(super) session_count: usize,
    pub(super) scripts_count: usize,
    pub(super) regression_loaded: bool,
    pub(super) regression_selected_summary_loaded: bool,
    pub(super) selected_followup_result_loaded: bool,
    pub(super) regression_failing_count: usize,
    pub(super) recent_failed_evidence_target: Option<RecentEvidenceTarget>,
    pub(super) recent_failed_evidence_rerunnable_kind: Option<&'static str>,
    pub(super) recent_failed_evidence_rerun_reason: Option<String>,
    pub(super) recent_evidence_next: String,
}

pub(super) fn collect_header_diagnostics_state(
    app: &App,
    st: &State,
) -> HeaderDiagnosticsState {
    let has_session = app
        .models()
        .read(&st.selected_session_id, |v| v.is_some())
        .unwrap_or(false);
    let selected_session = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let session_count = app
        .models()
        .read(&st.sessions, |sessions| sessions.len())
        .unwrap_or(0);
    let scripts_count = app
        .models()
        .read(&st.script_library, |scripts| scripts.len())
        .unwrap_or(0);
    let regression_loaded = app
        .models()
        .read(&st.regression_loaded_dir, |dir| dir.is_some())
        .unwrap_or(false);
    let regression_selected_summary_loaded = app
        .models()
        .read(&st.regression_selected_summary_json, |value| !value.trim().is_empty())
        .unwrap_or(false);
    let selected_followup_result_loaded = selected_followup_result_loaded_from_state(app, st);
    let regression_failing_count = app
        .models()
        .read(&st.regression_index_json, |index_json| {
            regression_failing_summary_rows(index_json, 10).len()
        })
        .unwrap_or(0);
    let gate_run_result_history = app
        .models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_result_history = app
        .models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let followup_result_history = app
        .models()
        .read(&st.followup_result_history, |v| v.clone())
        .unwrap_or_default();
    let recent_failed_evidence_target = devtools_recent_failed_evidence_target(
        &gate_run_result_history,
        &workflow_run_result_history,
        &followup_result_history,
    );
    let recent_workflow_commands = devtools_workflow_commands_from_state(app, st);
    let recent_failed_evidence_rerunnable_kind = recent_failed_evidence_target
        .as_ref()
        .and_then(|target| {
            recent_failed_evidence_rerun_command_from_state(target, &recent_workflow_commands)
        })
        .map(|command| command.kind());
    let recent_failed_evidence_rerun_reason =
        recent_failed_evidence_target.as_ref().and_then(|target| {
            recent_failed_evidence_rerun_unavailable_reason_from_state(target, &recent_workflow_commands)
        });
    let recent_failing_count = recent_evidence_failing_count(
        &gate_run_result_history,
        &workflow_run_result_history,
        &followup_result_history,
    );
    let recent_evidence_empty = gate_run_result_history.is_empty()
        && workflow_run_result_history.is_empty()
        && followup_result_history.is_empty();
    let recent_evidence_next = recent_evidence_next_action(
        recent_failing_count,
        recent_evidence_empty,
        recent_failed_evidence_target.as_ref(),
        &recent_workflow_commands,
    );

    HeaderDiagnosticsState {
        has_session,
        selected_session,
        session_count,
        scripts_count,
        regression_loaded,
        regression_selected_summary_loaded,
        selected_followup_result_loaded,
        regression_failing_count,
        recent_failed_evidence_target,
        recent_failed_evidence_rerunnable_kind,
        recent_failed_evidence_rerun_reason,
        recent_evidence_next,
    }
}

pub(super) fn header_next_action_lines(
    st: &State,
    header: &HeaderDiagnosticsState,
) -> Vec<String> {
    header_next_action_lines_for_artifacts_root(st.cfg.fs_out_dir.as_ref(), header)
}

pub(super) fn header_next_action_lines_for_artifacts_root(
    artifacts_root: &str,
    header: &HeaderDiagnosticsState,
) -> Vec<String> {
    devtools_first_open_next_action_lines(
        header.has_session,
        header.session_count,
        header.selected_session.as_deref(),
        header.scripts_count,
        header.regression_loaded,
        header.regression_selected_summary_loaded,
        header.selected_followup_result_loaded,
        header.regression_failing_count,
        artifacts_root,
        header.recent_failed_evidence_target.as_ref(),
        header.recent_failed_evidence_rerunnable_kind,
        header.recent_failed_evidence_rerun_reason.as_deref(),
        &header.recent_evidence_next,
    )
}
