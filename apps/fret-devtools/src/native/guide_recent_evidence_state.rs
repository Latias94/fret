use fret_app::App;

use super::recent_evidence::{
    RecentEvidenceTarget, devtools_recent_evidence_lines_with_workflow_commands,
    devtools_recent_failed_evidence_target, recent_failed_evidence_bundle_dir,
    recent_failed_evidence_rerun_command_from_state,
};
use super::{State, devtools_workflow_commands_from_state};

pub(super) struct GuideRecentEvidenceState {
    pub(super) target: Option<RecentEvidenceTarget>,
    pub(super) bundle_dir_available: bool,
    pub(super) rerunnable: bool,
    pub(super) report_text: String,
}

pub(super) fn collect_guide_recent_evidence_state(
    app: &App,
    st: &State,
) -> GuideRecentEvidenceState {
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
    let target = devtools_recent_failed_evidence_target(
        &gate_run_result_history,
        &workflow_run_result_history,
        &followup_result_history,
    );
    let workflow_commands = devtools_workflow_commands_from_state(app, st);
    let bundle_dir_available = target
        .as_ref()
        .and_then(recent_failed_evidence_bundle_dir)
        .is_some();
    let rerunnable = target
        .as_ref()
        .and_then(|target| {
            recent_failed_evidence_rerun_command_from_state(target, &workflow_commands)
        })
        .is_some();
    let report_text = devtools_recent_evidence_lines_with_workflow_commands(
        &gate_run_result_history,
        &workflow_run_result_history,
        &followup_result_history,
        &workflow_commands,
    )
    .join("\n");

    GuideRecentEvidenceState {
        target,
        bundle_dir_available,
        rerunnable,
        report_text,
    }
}
