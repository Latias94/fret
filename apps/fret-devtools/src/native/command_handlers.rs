use std::path::{Path, PathBuf};
use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_core::{AppWindowId, UiServices};
use fret_diag::regression_summary::regression_bundle_followup_commands;
use fret_diag::transport::DiagTransportKind;
use fret_diag_protocol::UiScriptStageV1;
use fret_runtime::Model;

use super::command_catalog::*;
use super::demo_metrics_debug::{
    demo_metrics_debug_action_command_for_copy_command, demo_metrics_debug_action_command_text,
};
use super::followup_panel::materialize_baseline_compare_followup_command;
use super::recent_evidence::recent_failed_evidence_bundle_dir;
use super::{
    RecentEvidenceRerunCommand, State, devtools_recent_evidence_lines_from_state,
    devtools_recent_failed_evidence_target_from_state, devtools_workflow_commands_from_state,
    file_url_from_path, followup, gate_run, generated_gate_command_from_state, is_abs_path,
    load_regression_summary_selection, pack, push_log, recent_failed_evidence_rerun_command_from_state,
    recent_failed_evidence_rerun_unavailable_reason_from_state, refresh_regression_artifacts,
    repo_root_from_script_paths, script_studio, script_studio_panel, select_recent_evidence_target,
    selected_followup_result_command_from_state, selected_followup_result_json_from_state,
    selected_followup_result_path_from_state, selected_followup_trace_artifact_path_from_state,
    selected_gate_run_result_command_from_state, selected_gate_run_result_json_from_state,
    selected_gate_run_result_path_from_state, selected_workflow_run_command_from_state,
    selected_workflow_run_regression_index_path_from_state,
    selected_workflow_run_regression_summary_path_from_state,
    selected_workflow_run_result_command_from_state, selected_workflow_run_result_json_from_state,
    selected_workflow_run_result_path_from_state, selected_workflow_run_suite_summary_path_from_state,
    selected_workflow_summarize_command_from_state, set_regression_summary_selection_error,
    summarize, workflow_regression_index_parent_dir, workflow_run, workflow_run_command_by_id_from_state,
    ws,
};

fn run_selected_regression_followup(app: &mut App, st: &mut State, command_id: &str) {
    let selected_bundle_dirs = app
        .models()
        .read(&st.regression_selected_bundle_dirs, |v| v.clone())
        .unwrap_or_default();
    let Some(mut command) =
        regression_bundle_followup_commands(selected_bundle_dirs.iter().map(|v| v.as_ref()))
            .into_iter()
            .find(|command| command.id == command_id)
    else {
        push_log(
            app,
            &st.log_lines,
            &format!("follow-up refused (no selected command {command_id})"),
        );
        return;
    };
    if let Some(bundle_arg) = command.diag_args.get_mut(1)
        && !is_abs_path(bundle_arg)
    {
        let repo_root = repo_root_from_script_paths(&st.script_paths);
        *bundle_arg = repo_root.join(&bundle_arg).to_string_lossy().to_string();
    }

    if let Err(err) = followup::start_regression_followup_command(app, st, command) {
        push_log(app, &st.log_lines, &format!("follow-up refused: {err}"));
    }
}

fn run_selected_regression_baseline_compare(
    app: &mut App,
    st: &mut State,
    command_id: &str,
    baseline_model: &Model<String>,
) {
    let selected_bundle_dirs = app
        .models()
        .read(&st.regression_selected_bundle_dirs, |v| v.clone())
        .unwrap_or_default();
    let Some(command) =
        regression_bundle_followup_commands(selected_bundle_dirs.iter().map(|v| v.as_ref()))
            .into_iter()
            .find(|command| command.id == command_id)
    else {
        push_log(
            app,
            &st.log_lines,
            &format!("follow-up compare refused (no selected command {command_id})"),
        );
        return;
    };
    let baseline = app
        .models()
        .read(baseline_model, |v| v.clone())
        .unwrap_or_default();
    let mut command = match materialize_baseline_compare_followup_command(&command, &baseline) {
        Ok(command) => command,
        Err(err) => {
            push_log(
                app,
                &st.log_lines,
                &format!("follow-up compare refused: {err}"),
            );
            return;
        }
    };
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    for arg in command.diag_args.iter_mut().skip(1).take(2) {
        if !is_abs_path(arg) {
            *arg = repo_root.join(&arg).to_string_lossy().to_string();
        }
    }

    if let Err(err) = followup::start_regression_followup_command(app, st, command) {
        push_log(
            app,
            &st.log_lines,
            &format!("follow-up compare refused: {err}"),
        );
    }
}

pub(super) fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut State,
    cmd: &CommandId,
) {
    ws::sync_selected_session_to_client(app, st);

    if let Some(text) = demo_metrics_debug_action_command_for_copy_command(cmd.as_str()) {
        let token = app.next_clipboard_token();
        app.push_effect(Effect::ClipboardWriteText {
            window,
            token,
            text,
        });
        return;
    }

    match cmd.as_str() {
        CMD_COPY_WS_URL => {
            let text = format!(
                "{}?fret_devtools_token={}",
                st.cfg.ws_url.as_ref(),
                st.cfg.token.as_ref()
            );
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text,
            });
        }
        CMD_COPY_TOKEN => {
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: st.cfg.token.to_string(),
            });
        }
        CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS => {
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: demo_metrics_debug_action_command_text(),
            });
        }
        CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW => {
            let Some(command) =
                workflow_run_command_by_id_from_state(app, st, DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID)
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "demo metrics debug workflow refused (missing IMUI P3 docking workflow)",
                );
                app.request_redraw(window);
                return;
            };
            if let Err(err) = workflow_run::start_workflow_run(app, st, command) {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("demo metrics debug workflow refused: {err}"),
                );
            }
            app.request_redraw(window);
        }
        CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW => {
            let Some(command) =
                workflow_run_command_by_id_from_state(app, st, DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID)
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "demo metrics debug workflow refused (missing perf docking workflow)",
                );
                app.request_redraw(window);
                return;
            };
            if let Err(err) = workflow_run::start_workflow_run(app, st, command) {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("demo metrics debug perf workflow refused: {err}"),
                );
            }
            app.request_redraw(window);
        }
        CMD_INSPECT_ENABLE | CMD_INSPECT_DISABLE => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            let enabled = cmd.as_str() == CMD_INSPECT_ENABLE;
            let consume_clicks = app
                .models()
                .read(&st.inspect_consume_clicks, |v| *v)
                .unwrap_or(false);
            st.devtools.inspect_set(None, enabled, consume_clicks);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_PICK_ARM => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.devtools.pick_arm(None);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_BUNDLE_DUMP => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            st.devtools.bundle_dump(None, Some("devtools"));
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCREENSHOT_REQUEST => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            if st.devtools.client().kind() != DiagTransportKind::WebSocket {
                push_log(
                    app,
                    &st.log_lines,
                    "screenshot.request requires WebSocket transport (filesystem mode cannot request runner-owned screenshots)",
                );
                app.request_redraw(window);
                return;
            }
            let _ = st
                .devtools
                .screenshot_request(None, Some("devtools"), 300, None);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        CMD_SCRIPTS_REFRESH => {
            refresh_script_library(app, st);
            app.request_redraw(window);
        }
        CMD_REGRESSION_REFRESH => {
            refresh_regression_artifacts(app, st);
            app.request_redraw(window);
        }
        CMD_REGRESSION_SUMMARIZE => {
            if let Err(err) = summarize::start_regression_summarize(app, st) {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("regression summarize refused: {err}"),
                );
            }
            app.request_redraw(window);
        }
        CMD_REGRESSION_PACK_SELECTED_BUNDLE => {
            let Some(bundle_dir) = app
                .models()
                .read(&st.regression_selected_bundle_dirs, |v| v.first().cloned())
                .ok()
                .flatten()
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "regression pack refused (no selected bundle dir)",
                );
                app.request_redraw(window);
                return;
            };
            if let Err(err) = pack::start_pack_bundle_dir(app, st, bundle_dir.as_ref()) {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("regression pack refused: {err}"),
                );
            }
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_STATS => {
            run_selected_regression_followup(app, st, "stats");
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF => {
            run_selected_regression_followup(app, st, "layout-perf-summary");
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_MEMORY => {
            run_selected_regression_followup(app, st, "memory-summary");
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE => {
            run_selected_regression_followup(app, st, "triage");
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS => {
            run_selected_regression_followup(app, st, "hotspots");
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_TRACE => {
            run_selected_regression_followup(app, st, "trace");
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_VISUAL_COMPARE => {
            let baseline_model = st.followup_baseline_bundle_or_dir.clone();
            run_selected_regression_baseline_compare(app, st, "visual-compare", &baseline_model);
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOOTPRINT_COMPARE => {
            let baseline_model = st.followup_baseline_session.clone();
            run_selected_regression_baseline_compare(
                app,
                st,
                "footprint-compare",
                &baseline_model,
            );
            app.request_redraw(window);
        }
        CMD_REGRESSION_RUN_FOLLOWUP_COMMAND => {
            let command_id = app
                .models()
                .read(&st.followup_pending_command_id, |v| v.clone())
                .ok()
                .flatten();
            let _ = app
                .models_mut()
                .update(&st.followup_pending_command_id, |v| *v = None);
            let Some(command_id) = command_id else {
                push_log(
                    app,
                    &st.log_lines,
                    "follow-up refused (missing command payload)",
                );
                app.request_redraw(window);
                return;
            };
            run_selected_regression_followup(app, st, command_id.as_ref());
            app.request_redraw(window);
        }
        CMD_COPY_FOLLOWUP_RESULT_PATH => {
            let Some(path) = selected_followup_result_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected follow-up result refused (no selected-bundle result artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_COPY_FOLLOWUP_RESULT_COMMAND => {
            let Some(command_line) = selected_followup_result_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected follow-up command refused (no selected-bundle result command yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: command_line,
            });
        }
        CMD_OPEN_FOLLOWUP_RESULT_JSON => {
            let Some(path) = selected_followup_result_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open selected follow-up JSON refused (no selected-bundle result artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_COPY_FOLLOWUP_RESULT_JSON => {
            let Some(result_json) = selected_followup_result_json_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected follow-up JSON refused (no selected-bundle result JSON yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: result_json,
            });
        }
        CMD_COPY_FOLLOWUP_TRACE_ARTIFACT_PATH => {
            let Some(path) = selected_followup_trace_artifact_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected trace artifact refused (no selected-bundle trace artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_OPEN_FOLLOWUP_TRACE_ARTIFACT => {
            let Some(path) = selected_followup_trace_artifact_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open selected trace artifact refused (no selected-bundle trace artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_GATE_RUN_GENERATED => {
            let Some(command) = generated_gate_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "gate run refused (unsupported generated gate profile)",
                );
                app.request_redraw(window);
                return;
            };
            if let Err(err) = gate_run::start_gate_run(app, st, command) {
                push_log(app, &st.log_lines, &format!("gate run refused: {err}"));
            }
            app.request_redraw(window);
        }
        CMD_COPY_RECENT_EVIDENCE_REPORT => {
            let report = devtools_recent_evidence_lines_from_state(app, st).join("\n");
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: report,
            });
        }
        CMD_SELECT_RECENT_FAILED_EVIDENCE => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "select recent failed evidence refused (no failed recent evidence)",
                );
                return;
            };
            select_recent_evidence_target(app, st, &target);
            push_log(
                app,
                &st.log_lines,
                &format!(
                    "selected recent failed evidence: {} {} {}",
                    target.kind, target.id, target.result_path
                ),
            );
            app.request_redraw(window);
        }
        CMD_RERUN_RECENT_FAILED_EVIDENCE => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "rerun recent failed evidence refused (no failed recent evidence)",
                );
                app.request_redraw(window);
                return;
            };
            let workflow_commands = devtools_workflow_commands_from_state(app, st);
            let Some(command) =
                recent_failed_evidence_rerun_command_from_state(&target, &workflow_commands)
            else {
                let reason = recent_failed_evidence_rerun_unavailable_reason_from_state(
                    &target,
                    &workflow_commands,
                )
                .unwrap_or_else(|| "unknown".to_string());
                push_log(
                    app,
                    &st.log_lines,
                    &format!("rerun recent failed evidence refused ({reason})"),
                );
                app.request_redraw(window);
                return;
            };
            let kind = command.kind();
            let result = match command {
                RecentEvidenceRerunCommand::Gate(command) => {
                    gate_run::start_gate_run(app, st, command)
                }
                RecentEvidenceRerunCommand::Workflow(command) => {
                    workflow_run::start_workflow_run(app, st, command)
                }
                RecentEvidenceRerunCommand::Followup(command) => {
                    followup::start_regression_followup_command(app, st, command)
                }
            };
            if let Err(err) = result {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("rerun recent failed evidence refused: {err}"),
                );
            } else {
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "rerun recent failed evidence started: {} {}",
                        kind, target.id
                    ),
                );
            }
            app.request_redraw(window);
        }
        CMD_COPY_RECENT_FAILED_EVIDENCE_PATH => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy recent failed evidence path refused (no failed recent evidence)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: target.result_path,
            });
        }
        CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy recent failed evidence bundle dir refused (no failed recent evidence)",
                );
                return;
            };
            let Some(bundle_dir) = recent_failed_evidence_bundle_dir(&target) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy recent failed evidence bundle dir refused (failed evidence has no bundle dir)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: bundle_dir.to_string(),
            });
        }
        CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy recent failed evidence command refused (no failed recent evidence)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: target.command_line,
            });
        }
        CMD_COPY_RECENT_FAILED_EVIDENCE_JSON => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy recent failed evidence JSON refused (no failed recent evidence)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: target.result_json,
            });
        }
        CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON => {
            let Some(target) = devtools_recent_failed_evidence_target_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open recent failed evidence JSON refused (no failed recent evidence)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&target.result_path),
                target: None,
                rel: None,
            });
        }
        CMD_COPY_GATE_RESULT_PATH => {
            let Some(path) = selected_gate_run_result_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected gate result refused (no gate run result artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_COPY_GATE_RESULT_COMMAND => {
            let Some(command_line) = selected_gate_run_result_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected gate command refused (no gate run result command yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: command_line,
            });
        }
        CMD_OPEN_GATE_RESULT_JSON => {
            let Some(path) = selected_gate_run_result_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open selected gate JSON refused (no gate run result artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_COPY_GATE_RESULT_JSON => {
            let Some(result_json) = selected_gate_run_result_json_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected gate JSON refused (no gate run result JSON yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: result_json,
            });
        }
        CMD_WORKFLOW_RUN_SELECTED => {
            let Some(command) = selected_workflow_run_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "workflow run refused (unsupported selected workflow)",
                );
                app.request_redraw(window);
                return;
            };
            if let Err(err) = workflow_run::start_workflow_run(app, st, command) {
                push_log(app, &st.log_lines, &format!("workflow run refused: {err}"));
            }
            app.request_redraw(window);
        }
        CMD_COPY_WORKFLOW_RESULT_PATH => {
            let Some(path) = selected_workflow_run_result_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected workflow result refused (no workflow run result artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_COPY_WORKFLOW_RESULT_COMMAND => {
            let Some(command_line) = selected_workflow_run_result_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected workflow command refused (no workflow run result command yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: command_line,
            });
        }
        CMD_OPEN_WORKFLOW_RESULT_JSON => {
            let Some(path) = selected_workflow_run_result_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open selected workflow JSON refused (no workflow run result artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_COPY_WORKFLOW_RESULT_JSON => {
            let Some(result_json) = selected_workflow_run_result_json_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy selected workflow JSON refused (no workflow run result JSON yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: result_json,
            });
        }
        CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH => {
            let Some(path) = selected_workflow_run_suite_summary_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy workflow suite summary refused (no selected workflow suite summary artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_OPEN_WORKFLOW_SUITE_SUMMARY => {
            let Some(path) = selected_workflow_run_suite_summary_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open workflow suite summary refused (no selected workflow suite summary artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH => {
            let Some(path) = selected_workflow_run_regression_summary_path_from_state(app, st)
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy workflow regression summary refused (no selected workflow regression summary artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY => {
            let Some(path) = selected_workflow_run_regression_summary_path_from_state(app, st)
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "open workflow regression summary refused (no selected workflow regression summary artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH => {
            let Some(path) = selected_workflow_run_regression_index_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy workflow regression index refused (no selected workflow regression index artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path,
            });
        }
        CMD_OPEN_WORKFLOW_REGRESSION_INDEX => {
            let Some(path) = selected_workflow_run_regression_index_path_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "open workflow regression index refused (no selected workflow regression index artifact yet)",
                );
                return;
            };
            app.push_effect(Effect::OpenUrl {
                url: file_url_from_path(&path),
                target: None,
                rel: None,
            });
        }
        CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY => {
            let Some(path) = selected_workflow_run_regression_summary_path_from_state(app, st)
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "load workflow regression summary refused (no selected workflow regression summary artifact yet)",
                );
                return;
            };
            match load_regression_summary_selection(app, st, Path::new(&path)) {
                Ok(()) => {
                    push_log(
                        app,
                        &st.log_lines,
                        &format!("loaded workflow regression summary into Regression Workspace: {path}"),
                    );
                }
                Err(err) => {
                    set_regression_summary_selection_error(app, st, &path, &err);
                    push_log(
                        app,
                        &st.log_lines,
                        &format!("load workflow regression summary failed: {path}: {err}"),
                    );
                }
            }
            app.request_redraw(window);
        }
        CMD_LOAD_WORKFLOW_REGRESSION_INDEX => {
            let Some(index_path) = selected_workflow_run_regression_index_path_from_state(app, st)
            else {
                push_log(
                    app,
                    &st.log_lines,
                    "load workflow regression index refused (no selected workflow regression index artifact yet)",
                );
                return;
            };
            let Some(root) = workflow_regression_index_parent_dir(&index_path) else {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("load workflow regression index refused (cannot derive artifact root): {index_path}"),
                );
                return;
            };
            let _ = app.models_mut().update(&st.target_out_dir, |v| {
                *v = Some(Arc::<str>::from(root.clone()))
            });
            refresh_regression_artifacts(app, st);
            push_log(
                app,
                &st.log_lines,
                &format!("loaded workflow regression index into Regression Workspace: {index_path}"),
            );
            app.request_redraw(window);
        }
        CMD_COPY_WORKFLOW_SUMMARIZE_COMMAND => {
            let Some(command) = selected_workflow_summarize_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "copy workflow summarize refused (no selected workflow regression summary artifact yet)",
                );
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: command.command_line,
            });
        }
        CMD_RUN_WORKFLOW_SUMMARIZE => {
            let Some(command) = selected_workflow_summarize_command_from_state(app, st) else {
                push_log(
                    app,
                    &st.log_lines,
                    "workflow summarize refused (no selected workflow regression summary artifact yet)",
                );
                app.request_redraw(window);
                return;
            };
            if let Err(err) = workflow_run::start_workflow_run(app, st, command) {
                push_log(
                    app,
                    &st.log_lines,
                    &format!("workflow summarize refused: {err}"),
                );
            }
            app.request_redraw(window);
        }
        CMD_SCRIPT_FORK => {
            fork_loaded_script(app, window, st);
            app.request_redraw(window);
        }
        CMD_SCRIPT_SAVE => {
            save_loaded_script(app, window, st);
            app.request_redraw(window);
        }
        CMD_SCRIPT_APPLY_PICK => {
            apply_pick_to_loaded_script(app, window, st);
            app.request_redraw(window);
        }
        CMD_OPEN_VIEWER_URL => {
            let url = app
                .models()
                .read(&st.viewer_url, |v| v.clone())
                .unwrap_or_default();
            if url.trim().is_empty() {
                push_log(app, &st.log_lines, "open viewer refused (empty url)");
                return;
            }
            app.push_effect(Effect::OpenUrl {
                url,
                target: None,
                rel: None,
            });
        }
        CMD_COPY_PACK_PATH => {
            let Some(path) = app
                .models()
                .read(&st.last_pack_path, |v| v.clone())
                .ok()
                .flatten()
            else {
                push_log(app, &st.log_lines, "copy pack path refused (no pack yet)");
                return;
            };
            let token = app.next_clipboard_token();
            app.push_effect(Effect::ClipboardWriteText {
                window,
                token,
                text: path.to_string(),
            });
        }
        CMD_PACK_LAST_BUNDLE => {
            if let Err(err) = pack::start_pack_last_bundle(app, st) {
                push_log(app, &st.log_lines, &format!("pack refused: {err}"));
            }
            app.request_redraw(window);
        }
        CMD_SCRIPT_PUSH | CMD_SCRIPT_RUN | CMD_SCRIPT_RUN_AND_PACK => {
            if !ws::require_session_selected(app, st) {
                app.request_redraw(window);
                return;
            }
            let script_text = app
                .models()
                .read(&st.script_text, |v| v.clone())
                .unwrap_or_default();
            let Ok(script_value) = serde_json::from_str::<serde_json::Value>(&script_text) else {
                push_log(app, &st.log_lines, "script json parse failed");
                app.request_redraw(window);
                return;
            };
            if let Err(err) = script_studio_panel::validate_script_json_value(&script_value) {
                push_log(app, &st.log_lines, &format!("script invalid: {err}"));
                app.request_redraw(window);
                return;
            }

            let ty = match cmd.as_str() {
                CMD_SCRIPT_RUN | CMD_SCRIPT_RUN_AND_PACK => "script.run",
                _ => "script.push",
            };

            if cmd.as_str() == CMD_SCRIPT_RUN_AND_PACK {
                let _ = app
                    .models_mut()
                    .update(&st.script_pack_after_run, |v| *v = true);
            } else {
                let _ = app
                    .models_mut()
                    .update(&st.script_pack_after_run, |v| *v = false);
            }
            let _ = app.models_mut().update(&st.script_last_stage, |v| {
                *v = Some(UiScriptStageV1::Queued)
            });
            let _ = app
                .models_mut()
                .update(&st.script_last_step_index, |v| *v = None);
            let _ = app
                .models_mut()
                .update(&st.script_last_reason, |v| *v = None);
            let _ = app
                .models_mut()
                .update(&st.script_last_bundle_dir, |v| *v = None);
            match ty {
                "script.run" => st.devtools.script_run_value(None, script_value),
                _ => st.devtools.script_push_value(None, script_value),
            }
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        _ => {}
    }
}

pub(super) fn refresh_script_library(app: &mut App, st: &mut State) {
    let scripts = script_studio::scan_script_library(&st.script_paths);
    let _ = app
        .models_mut()
        .update(&st.script_library, |v| *v = scripts.clone());

    let loaded_path = app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| PathBuf::from(s.as_ref()));

    let loaded_origin = loaded_path
        .as_ref()
        .and_then(|p| scripts.iter().find(|i| &i.path == p).map(|i| i.origin));
    let _ = app
        .models_mut()
        .update(&st.loaded_script_origin, |v| *v = loaded_origin);
}

fn fork_loaded_script(app: &mut App, window: AppWindowId, st: &mut State) {
    let origin = app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    let path = app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| PathBuf::from(s.as_ref()));

    if origin != Some(script_studio::ScriptOrigin::WorkspaceTools) {
        push_log(
            app,
            &st.log_lines,
            "fork refused (load a tools/* script first)",
        );
        return;
    }
    let Some(path) = path else {
        push_log(app, &st.log_lines, "fork refused (no script loaded)");
        return;
    };
    let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
        push_log(app, &st.log_lines, "fork refused (invalid file name)");
        return;
    };

    let item = script_studio::ScriptItem {
        origin: script_studio::ScriptOrigin::WorkspaceTools,
        file_name: Arc::from(file_name),
        path,
    };

    let forked = match script_studio::fork_script_to_user(&st.script_paths, &item) {
        Ok(item) => item,
        Err(err) => {
            push_log(app, &st.log_lines, &format!("fork failed: {err}"));
            return;
        }
    };

    refresh_script_library(app, st);
    let _ = app.models_mut().update(&st.script_text, |v| {
        *v = script_studio::load_script_text(&forked.path).unwrap_or_default()
    });
    let _ = app
        .models_mut()
        .update(&st.loaded_script_origin, |v| *v = Some(forked.origin));
    let _ = app.models_mut().update(&st.loaded_script_path, |v| {
        *v = Some(Arc::<str>::from(forked.path.to_string_lossy().to_string()))
    });

    app.push_effect(Effect::RequestAnimationFrame(window));
}

fn save_loaded_script(app: &mut App, window: AppWindowId, st: &mut State) {
    let origin = app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    if origin != Some(script_studio::ScriptOrigin::UserLocal) {
        push_log(
            app,
            &st.log_lines,
            "save refused (fork into .fret/diag/scripts first)",
        );
        return;
    }

    let Some(path) = app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| PathBuf::from(s.as_ref()))
    else {
        push_log(app, &st.log_lines, "save refused (no script loaded)");
        return;
    };

    let text = app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    if let Err(err) = script_studio::save_user_script(&st.script_paths, &path, &text) {
        push_log(app, &st.log_lines, &format!("save failed: {err}"));
        return;
    }

    refresh_script_library(app, st);
    app.push_effect(Effect::RequestAnimationFrame(window));
}

fn apply_pick_to_loaded_script(app: &mut App, window: AppWindowId, st: &mut State) {
    let pointer = app
        .models()
        .read(&st.script_apply_pointer, |v| v.clone())
        .unwrap_or_default();
    let script = app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    let pick = app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    if pick.trim().is_empty() {
        push_log(
            app,
            &st.log_lines,
            "apply pick refused (no pick.result yet)",
        );
        return;
    }

    match script_studio::apply_pick_to_json_pointer(&script, &pointer, &pick) {
        Ok(updated) => {
            let _ = app.models_mut().update(&st.script_text, |v| *v = updated);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }
        Err(err) => push_log(app, &st.log_lines, &format!("apply pick failed: {err}")),
    }
}
