use std::path::Path;
use std::sync::Arc;

use fret_app::App;

use super::workflow_run;
use super::{
    DIAG_REGRESSION_INDEX_FILENAME_V1, DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID, State,
    devtools_workflow_commands_from_state, repo_root_from_script_paths, resolve_repo_or_abs_path,
    workflow_aggregate_index_loaded, workflow_handoff_readiness_lines,
    workflow_summarize_command_from_summary_path,
};

pub(super) struct WorkflowPanelState {
    pub(super) selected_workflow_id: Arc<str>,
    pub(super) commands: Vec<workflow_run::DevtoolsWorkflowRunCommandV1>,
    pub(super) selected_command_label: String,
    pub(super) command_preview: String,
    pub(super) command_state_line: String,
    pub(super) run_enabled: bool,
    pub(super) workflow_run_in_flight: bool,
    pub(super) selected_workflow_run_result_path: Option<String>,
    pub(super) selected_workflow_run_result_json: String,
    pub(super) selected_workflow_suite_summary_path: Option<String>,
    pub(super) selected_workflow_regression_summary_path: Option<String>,
    pub(super) selected_workflow_regression_index_ready: bool,
    pub(super) selected_workflow_run_result_entry:
        Option<workflow_run::WorkflowRunResultHistoryEntry>,
    pub(super) workflow_handoff_readiness: Vec<String>,
    pub(super) workflow_summarize_preview: String,
}

pub(super) fn collect_workflow_panel_state(app: &App, st: &State) -> WorkflowPanelState {
    let selected_workflow_id = app
        .models()
        .read(&st.workflow_run_selected_id, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from(DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID));
    let commands = devtools_workflow_commands_from_state(app, st);
    let selected_command = commands
        .iter()
        .find(|command| command.id == selected_workflow_id.as_ref())
        .or_else(|| commands.first());
    let command_preview = selected_command
        .map(|command| command.command_line.clone())
        .unwrap_or_else(|| "No workflow command available.".to_string());
    let selected_command_label = selected_command
        .map(|command| format!("{} ({})", command.label, command.id))
        .unwrap_or_else(|| selected_workflow_id.to_string());
    let command_state_line = selected_command
        .map(|command| {
            if command.is_runnable() {
                let redacted = workflow_run::redact_workflow_diag_args(&command.diag_args);
                format!("diag args: {}", redacted.join(" "))
            } else if command.missing_inputs.is_empty() {
                "diag args: <not runnable>".to_string()
            } else {
                format!("missing inputs: {}", command.missing_inputs.join(", "))
            }
        })
        .unwrap_or_else(|| "diag args: <unsupported workflow>".to_string());
    let run_enabled = selected_command.is_some_and(|command| command.is_runnable());
    let workflow_run_in_flight = app
        .models()
        .read(&st.workflow_run_in_flight, |v| *v)
        .unwrap_or(false);
    let workflow_run_result_json = app
        .models()
        .read(&st.workflow_run_last_result_json, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_result_history = app
        .models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_selected_result_path = app
        .models()
        .read(&st.workflow_run_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    let selected_workflow_run_result_entry =
        workflow_run::workflow_run_result_history_selected_or_latest_entry(
            &workflow_run_result_history,
            workflow_run_selected_result_path.as_deref(),
        );
    let selected_workflow_run_result_path = selected_workflow_run_result_entry
        .as_ref()
        .map(|entry| entry.result_path.clone());
    let selected_workflow_run_result_json = selected_workflow_run_result_entry
        .as_ref()
        .map(|entry| entry.result_json.clone())
        .unwrap_or_else(|| workflow_run_result_json.clone());
    let selected_workflow_regression_summary_path =
        workflow_run::workflow_run_regression_summary_artifact_path_from_result_json(
            &selected_workflow_run_result_json,
        );
    let selected_workflow_suite_summary_path =
        workflow_run::workflow_run_output_artifact_path_from_result_json(
            &selected_workflow_run_result_json,
            "suite.summary.json",
        );
    let selected_workflow_regression_summary_resolved_path =
        selected_workflow_regression_summary_path.as_ref().map(|path| {
            let repo_root = repo_root_from_script_paths(&st.script_paths);
            resolve_repo_or_abs_path(&repo_root, path)
                .to_string_lossy()
                .to_string()
        });
    let selected_workflow_summarize_command = selected_workflow_regression_summary_resolved_path
        .as_deref()
        .and_then(workflow_summarize_command_from_summary_path);
    let selected_workflow_regression_index_resolved_path =
        workflow_run::workflow_run_regression_index_artifact_path_from_result_json(
            &selected_workflow_run_result_json,
        )
        .map(|path| {
            let repo_root = repo_root_from_script_paths(&st.script_paths);
            resolve_repo_or_abs_path(&repo_root, &path)
                .to_string_lossy()
                .to_string()
        })
        .or_else(|| {
            selected_workflow_regression_summary_resolved_path
                .as_ref()
                .and_then(|path| {
                    Path::new(path).parent().map(|parent| {
                        parent
                            .join(DIAG_REGRESSION_INDEX_FILENAME_V1)
                            .to_string_lossy()
                            .to_string()
                    })
                })
        });
    let selected_workflow_regression_index_ready = selected_workflow_regression_index_resolved_path
        .as_ref()
        .is_some_and(|path| Path::new(path).is_file());
    let loaded_regression_dir = app
        .models()
        .read(&st.regression_loaded_dir, |v| v.clone())
        .ok()
        .flatten()
        .map(|path| path.to_string());
    let regression_index_loaded = app
        .models()
        .read(&st.regression_index_json, |v| !v.trim().is_empty())
        .unwrap_or(false);
    let selected_workflow_aggregate_index_loaded = workflow_aggregate_index_loaded(
        selected_workflow_regression_index_resolved_path.as_deref(),
        loaded_regression_dir.as_deref(),
        regression_index_loaded,
    );
    let loaded_regression_summary_path = app
        .models()
        .read(&st.regression_selected_summary_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|path| path.to_string());
    let workflow_handoff_readiness = workflow_handoff_readiness_lines(
        workflow_run_in_flight,
        selected_workflow_run_result_entry.is_some(),
        selected_workflow_regression_summary_resolved_path.as_deref(),
        loaded_regression_summary_path.as_deref(),
        selected_workflow_regression_index_ready,
        selected_workflow_aggregate_index_loaded,
    );
    let workflow_summarize_preview = selected_workflow_summarize_command
        .as_ref()
        .map(|command| {
            let index_path = selected_workflow_regression_index_resolved_path
                .as_deref()
                .unwrap_or("-");
            format!(
                "command: {}\naggregate_index: {}\nready: {}",
                command.command_line,
                index_path,
                if selected_workflow_regression_index_ready {
                    "true"
                } else {
                    "false"
                }
            )
        })
        .unwrap_or_else(|| {
            "No workflow regression.summary.json artifact selected yet.".to_string()
        });

    WorkflowPanelState {
        selected_workflow_id,
        commands,
        selected_command_label,
        command_preview,
        command_state_line,
        run_enabled,
        workflow_run_in_flight,
        selected_workflow_run_result_path,
        selected_workflow_run_result_json,
        selected_workflow_suite_summary_path,
        selected_workflow_regression_summary_path,
        selected_workflow_regression_index_ready,
        selected_workflow_run_result_entry,
        workflow_handoff_readiness,
        workflow_summarize_preview,
    }
}
