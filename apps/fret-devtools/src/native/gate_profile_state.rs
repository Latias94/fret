use std::sync::Arc;

use fret_app::App;
use fret_diag::{devtools_gate_profile_lines, devtools_gate_profiles_v1, devtools_gate_script_target_profile_ids_v1};

use super::gate_run;
use super::{State, generated_gate_command_from_state};

pub(super) struct GateProfilePanelState {
    pub(super) selected_profile_id: Arc<str>,
    pub(super) selected_profile_label: String,
    pub(super) command_preview: String,
    pub(super) command_state_line: String,
    pub(super) copy_enabled: bool,
    pub(super) run_enabled: bool,
    pub(super) gate_run_in_flight: bool,
    pub(super) selected_gate_run_result_path: Option<String>,
    pub(super) selected_gate_run_result_json: String,
    pub(super) selected_gate_run_result_entry: Option<gate_run::GateRunResultHistoryEntry>,
    pub(super) gate_profile_lines: Vec<String>,
}

pub(super) fn collect_gate_profile_panel_state(
    app: &App,
    st: &State,
) -> GateProfilePanelState {
    let selected_profile_id = app
        .models()
        .read(&st.gate_profile_selected_id, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("stale-paint-scene"));
    let generated_command = generated_gate_command_from_state(app, st);
    let command_preview = generated_command
        .as_ref()
        .map(|command| command.command_line.clone())
        .unwrap_or_else(|| "Select a script-target gate profile.".to_string());
    let selected_profile_label = devtools_gate_profiles_v1()
        .iter()
        .find(|profile| profile.id == selected_profile_id.as_ref())
        .map(|profile| format!("{} ({})", profile.label, profile.id))
        .unwrap_or_else(|| selected_profile_id.to_string());
    let command_state_line = generated_command
        .as_ref()
        .map(|command| {
            if command.is_runnable() {
                format!("diag args: {}", command.diag_args.join(" "))
            } else if command.missing_inputs.is_empty() {
                "diag args: <not runnable>".to_string()
            } else {
                format!("missing inputs: {}", command.missing_inputs.join(", "))
            }
        })
        .unwrap_or_else(|| "diag args: <unsupported profile>".to_string());
    let copy_enabled = generated_command.is_some();
    let run_enabled = generated_command
        .as_ref()
        .is_some_and(|command| command.is_runnable());
    let gate_run_in_flight = app
        .models()
        .read(&st.gate_run_in_flight, |v| *v)
        .unwrap_or(false);
    let gate_run_result_json = app
        .models()
        .read(&st.gate_run_last_result_json, |v| v.clone())
        .unwrap_or_default();
    let gate_run_result_history = app
        .models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let gate_run_selected_result_path = app
        .models()
        .read(&st.gate_run_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    let selected_gate_run_result_entry = gate_run::gate_run_result_history_selected_or_latest_entry(
        &gate_run_result_history,
        gate_run_selected_result_path.as_deref(),
    );
    let selected_gate_run_result_path = selected_gate_run_result_entry
        .as_ref()
        .map(|entry| entry.result_path.clone());
    let selected_gate_run_result_json = selected_gate_run_result_entry
        .as_ref()
        .map(|entry| entry.result_json.clone())
        .unwrap_or_else(|| gate_run_result_json.clone());
    let gate_profile_lines = devtools_gate_profile_lines(st.cfg.fs_out_dir.as_ref());

    GateProfilePanelState {
        selected_profile_id,
        selected_profile_label,
        command_preview,
        command_state_line,
        copy_enabled,
        run_enabled,
        gate_run_in_flight,
        selected_gate_run_result_path,
        selected_gate_run_result_json,
        selected_gate_run_result_entry,
        gate_profile_lines,
    }
}

pub(super) fn gate_profile_select_items() -> Vec<(&'static str, String)> {
    devtools_gate_profiles_v1()
        .iter()
        .filter(|profile| {
            devtools_gate_script_target_profile_ids_v1().contains(&profile.id)
                || profile.id == "perf-thresholds"
                || profile.id == "resource-footprint-thresholds"
        })
        .map(|profile| (profile.id, format!("{} ({})", profile.label, profile.id)))
        .collect()
}
