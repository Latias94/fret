use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_bootstrap::BootstrapBuilder;
use fret_bootstrap::ui_app_driver::{UiAppDriver, ViewElements};
use fret_core::{AppWindowId, Px, UiServices};
use fret_diag::devtools::DevtoolsOps;
use fret_diag::{
    DevtoolsGatePerfThresholdCommandInputV1, DevtoolsGateScriptTargetCommandInputV1,
    DevtoolsGateResourceFootprintThresholdCommandInputV1, devtools_gate_perf_threshold_command,
    devtools_gate_resource_footprint_threshold_command, devtools_gate_script_target_command,
};
use fret_diag::regression_summary::{
    DIAG_REGRESSION_INDEX_FILENAME_V1, DIAG_REGRESSION_SUMMARY_FILENAME_V1,
    RegressionSummaryV1, regression_bundle_followup_commands, regression_summary_drilldown,
};
use fret_diag::transport::{
    ClientKindV1, DevtoolsWsClientConfig, DiagTransportKind, FsDiagTransportConfig,
    ToolingDiagClient, WsDiagTransportConfig,
};
use fret_diag::{
    dashboard_failing_summary_entries, dashboard_human_lines_from_projection,
    project_dashboard_summary,
};
use fret_diag_protocol::{DevtoolsSessionDescriptorV1, UiScriptStageV1};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::ContinuousFrames;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

mod demo_metrics_debug;
#[path = "native/diagnostics_tree_panel.rs"]
mod diagnostics_tree_panel;
mod followup;
#[path = "native/followup_panel.rs"]
mod followup_panel;
mod gate_run;
#[path = "native/run_history_panel.rs"]
mod run_history_panel;
#[path = "native/regression_panel.rs"]
mod regression_panel;
#[path = "native/script_studio_panel.rs"]
mod script_studio_panel;
#[path = "native/semantics_detail_panel.rs"]
mod semantics_detail_panel;
#[path = "native/command_catalog.rs"]
mod command_catalog;
#[path = "native/ui_primitives.rs"]
mod ui_primitives;
#[path = "native/gate_profile_state.rs"]
mod gate_profile_state;
#[path = "native/guide_reference_panels.rs"]
mod guide_reference_panels;
#[path = "native/guide_panel.rs"]
mod guide_panel;
#[path = "native/guide_recent_evidence_state.rs"]
mod guide_recent_evidence_state;
#[path = "native/guide_recent_evidence_panel.rs"]
mod guide_recent_evidence_panel;
#[path = "native/header_state.rs"]
mod header_state;
#[path = "native/inspect_panel.rs"]
mod inspect_panel;
mod pack;
#[path = "native/workflow_panel_state.rs"]
mod workflow_panel_state;
#[path = "native/discovery_lines.rs"]
mod discovery_lines;
#[path = "native/recent_evidence.rs"]
mod recent_evidence;
mod script_studio;
mod semantics;
mod summarize;
mod workflow_run;
mod ws;

use command_catalog::*;
use demo_metrics_debug::{
    demo_metrics_debug_action_command_for_copy_command, demo_metrics_debug_action_command_text,
};
use diagnostics_tree_panel::{element_tree_panel, layout_tree_panel, semantics_panel};
use followup_panel::materialize_baseline_compare_followup_command;
use guide_panel::devtools_guide_panel;
use guide_recent_evidence_panel::{
    first_open_recent_evidence_action_row, first_open_recent_evidence_action_specs,
};
use header_state::{collect_header_diagnostics_state, header_next_action_lines};
use inspect_panel::inspect_panel;
#[cfg(test)]
use inspect_panel::{inspect_hover_bounds_lines, inspect_overlay_hook_lines};
use discovery_lines::{
    devtools_first_open_next_action_lines, devtools_workflow_commands,
    workflow_handoff_readiness_lines,
};
use recent_evidence::{
    RecentEvidenceRerunCommand, RecentEvidenceTarget,
    devtools_recent_evidence_lines_with_workflow_commands,
    devtools_recent_evidence_selection_effect, devtools_recent_failed_evidence_target,
    recent_failed_evidence_bundle_dir,
    recent_failed_evidence_rerun_command_from_state,
    recent_failed_evidence_rerun_unavailable_reason_from_state,
};
use regression_panel::regression_panel;
use script_studio_panel::center_panel;
use semantics_detail_panel::sem_node_panel;
use ui_primitives::{diag_card, diag_section, text_blob};
#[cfg(test)]
use discovery_lines::{
    devtools_dogfood_workflow_lines, devtools_first_open_lines, devtools_gate_command_lines,
    devtools_workflow_run_lines,
};
#[cfg(test)]
use recent_evidence::{
    devtools_recent_evidence_lines, recent_evidence_status_failed,
    recent_failed_evidence_rerun_command, recent_failed_evidence_rerun_line,
};
#[cfg(test)]
use demo_metrics_debug::{
    demo_metrics_debug_action_copy_command_lines, demo_metrics_debug_action_metadata_lines,
    demo_metrics_debug_action_readiness_lines, demo_metrics_debug_workflow_artifact_action_lines,
    demo_metrics_debug_workflow_readiness_lines, demo_metrics_debug_workflow_result_action_lines,
    demo_metrics_debug_workflow_status_lines, devtools_demo_metrics_debug_lines,
    devtools_demo_metrics_debug_lines_with_state,
};

#[derive(Clone)]
struct DevtoolsConfig {
    transport: DiagTransportKind,
    fs_out_dir: Arc<str>,
    ws_port: u16,
    ws_url: Arc<str>,
    token: Arc<str>,
}

struct State {
    cfg: DevtoolsConfig,

    panel_fractions: Model<Vec<f32>>,
    left_tab: Model<Option<Arc<str>>>,
    details_tab: Model<Option<Arc<str>>>,
    sessions: Model<Vec<DevtoolsSessionDescriptorV1>>,
    selected_session_id: Model<Option<Arc<str>>>,
    selected_session_open: Model<bool>,
    inspect_consume_clicks: Model<bool>,
    gate_profile_selected_id: Model<Option<Arc<str>>>,
    gate_profile_open: Model<bool>,
    gate_profile_script_json: Model<String>,
    gate_profile_test_id: Model<String>,
    gate_profile_perf_target: Model<String>,
    gate_profile_perf_repeat: Model<String>,
    gate_profile_perf_warmup_frames: Model<String>,
    gate_profile_perf_threshold_agg: Model<String>,
    gate_profile_perf_max_top_total_us: Model<String>,
    gate_profile_perf_max_top_layout_us: Model<String>,
    gate_profile_perf_max_top_solve_us: Model<String>,
    gate_profile_perf_max_pointer_move_dispatch_us: Model<String>,
    gate_profile_perf_max_pointer_move_hit_test_us: Model<String>,
    gate_profile_perf_max_pointer_move_global_changes: Model<String>,
    gate_profile_perf_max_renderer_encode_scene_us: Model<String>,
    gate_profile_perf_max_renderer_upload_us: Model<String>,
    gate_profile_perf_max_renderer_record_passes_us: Model<String>,
    gate_profile_perf_max_renderer_encoder_finish_us: Model<String>,
    gate_profile_perf_max_renderer_prepare_text_us: Model<String>,
    gate_profile_perf_max_renderer_prepare_svg_us: Model<String>,
    gate_profile_perf_max_renderer_instance_bytes: Model<String>,
    gate_profile_perf_max_renderer_encode_scene_text_ops: Model<String>,
    gate_profile_resource_target: Model<String>,
    gate_profile_resource_max_working_set_bytes: Model<String>,
    gate_profile_resource_max_peak_working_set_bytes: Model<String>,
    gate_profile_resource_max_cpu_avg_percent_total_cores: Model<String>,
    gate_profile_resource_launch_command: Model<String>,
    gate_run_in_flight: Model<bool>,
    gate_run_last_command_line: Model<Option<Arc<str>>>,
    gate_run_last_result_path: Model<Option<Arc<str>>>,
    gate_run_last_result_json: Model<String>,
    gate_run_result_history: Model<Vec<gate_run::GateRunResultHistoryEntry>>,
    gate_run_selected_result_path: Model<Option<Arc<str>>>,
    gate_run_last_error: Model<Option<Arc<str>>>,
    workflow_run_selected_id: Model<Option<Arc<str>>>,
    workflow_run_selected_open: Model<bool>,
    workflow_run_in_flight: Model<bool>,
    workflow_run_last_command_line: Model<Option<Arc<str>>>,
    workflow_run_last_result_path: Model<Option<Arc<str>>>,
    workflow_run_last_result_json: Model<String>,
    workflow_run_result_history: Model<Vec<workflow_run::WorkflowRunResultHistoryEntry>>,
    workflow_run_selected_result_path: Model<Option<Arc<str>>>,
    workflow_run_last_error: Model<Option<Arc<str>>>,

    script_paths: script_studio::ScriptPaths,
    script_library: Model<Vec<script_studio::ScriptItem>>,
    loaded_script_origin: Model<Option<script_studio::ScriptOrigin>>,
    loaded_script_path: Model<Option<Arc<str>>>,
    script_apply_pointer: Model<String>,
    script_text: Model<String>,
    script_studio_helper_tab: Model<Option<Arc<str>>>,
    script_step_insert_index: Model<String>,
    script_selector_kind: Model<Option<Arc<str>>>,
    script_selector_kind_open: Model<bool>,
    script_selector_test_id: Model<String>,
    script_selector_role: Model<String>,
    script_selector_name: Model<String>,
    script_selector_ancestors: Model<String>,
    script_selector_node_id: Model<String>,
    script_selector_element_id: Model<String>,
    script_predicate_kind: Model<Option<Arc<str>>>,
    script_predicate_kind_open: Model<bool>,
    script_predicate_other_selector_json: Model<String>,
    script_predicate_role: Model<String>,
    script_predicate_checked: Model<bool>,
    script_predicate_len_bytes: Model<String>,
    script_predicate_padding_px: Model<String>,
    script_predicate_eps_px: Model<String>,
    script_predicate_min_w_px: Model<String>,
    script_predicate_min_h_px: Model<String>,
    script_predicate_barrier_root: Model<Option<Arc<str>>>,
    script_predicate_barrier_root_open: Model<bool>,
    script_predicate_focus_barrier_root: Model<Option<Arc<str>>>,
    script_predicate_focus_barrier_root_open: Model<bool>,
    script_predicate_require_equal: Model<Option<Arc<str>>>,
    script_predicate_require_equal_open: Model<bool>,

    script_last_stage: Model<Option<UiScriptStageV1>>,
    script_last_step_index: Model<Option<u32>>,
    script_last_reason: Model<Option<Arc<str>>>,
    script_last_bundle_dir: Model<Option<Arc<str>>>,
    script_pack_after_run: Model<bool>,

    target_out_dir: Model<Option<Arc<str>>>,
    last_bundle_dir_abs: Model<Option<Arc<str>>>,
    last_bundle_dump_exported_unix_ms: Model<Option<u64>>,
    last_bundle_dump_bundle_json: Model<Option<Arc<str>>>,
    last_pack_path: Model<Option<Arc<str>>>,
    pack_in_flight: Model<bool>,
    pack_last_error: Model<Option<Arc<str>>>,
    summarize_in_flight: Model<bool>,
    summarize_last_error: Model<Option<Arc<str>>>,
    followup_in_flight: Model<bool>,
    followup_last_command_line: Model<Option<Arc<str>>>,
    followup_last_result_path: Model<Option<Arc<str>>>,
    followup_last_result_json: Model<String>,
    followup_result_history: Model<Vec<followup::FollowupResultHistoryEntry>>,
    followup_selected_result_path: Model<Option<Arc<str>>>,
    followup_last_error: Model<Option<Arc<str>>>,
    followup_pending_command_id: Model<Option<Arc<str>>>,
    followup_baseline_bundle_or_dir: Model<String>,
    followup_baseline_session: Model<String>,
    viewer_url: Model<String>,

    last_pick_json: Model<String>,
    last_inspect_hover_json: Model<String>,
    last_inspect_focus_json: Model<String>,
    last_overlay_summary_json: Model<String>,
    last_script_result_json: Model<String>,
    last_bundle_json: Model<String>,
    last_screenshot_json: Model<String>,
    regression_summary_json: Model<String>,
    regression_index_json: Model<String>,
    regression_dashboard_human: Model<String>,
    regression_loaded_dir: Model<Option<Arc<str>>>,
    regression_last_error: Model<Option<Arc<str>>>,
    regression_selected_summary_path: Model<Option<Arc<str>>>,
    regression_selected_summary_json: Model<String>,
    regression_selected_bundle_dirs: Model<Vec<Arc<str>>>,
    regression_selected_capability_sources: Model<Vec<Arc<str>>>,
    regression_selected_capabilities_checks: Model<Vec<Arc<str>>>,
    regression_selected_perf_evidence: Model<Vec<Arc<str>>>,
    regression_selected_first_open_evidence: Model<Vec<Arc<str>>>,
    regression_selected_share_artifacts: Model<Vec<Arc<str>>>,
    regression_selected_error: Model<Option<Arc<str>>>,
    log_lines: Model<Vec<Arc<str>>>,

    semantics_cache: Model<Option<Arc<semantics::SemanticsIndex>>>,
    semantics_source_hash: Model<Option<u64>>,
    semantics_error: Model<Option<Arc<str>>>,
    semantics_search: Model<String>,
    semantics_expanded: Model<HashSet<u64>>,
    semantics_selected_id: Model<Option<u64>>,
    semantics_selected_node_json: Model<String>,
    semantics_selected_node_live_json: Model<String>,
    semantics_selected_node_live_status: Model<Option<Arc<str>>>,
    semantics_selected_node_live_updated_unix_ms: Model<Option<u64>>,
    semantics_selected_node_live_children: Model<Vec<u64>>,
    semantics_selected_hit_test_explain_json: Model<String>,
    semantics_selected_hit_test_explain_summary: Model<String>,
    semantics_selected_hit_test_explain_status: Model<Option<Arc<str>>>,
    semantics_selected_hit_test_explain_updated_unix_ms: Model<Option<u64>>,
    semantics_live_enabled: Model<bool>,
    semantics_live_force_nonce: Model<u64>,

    devtools: DevtoolsOps,
    applied_session_id: Option<Arc<str>>,

    live_semantics_last_target: Option<(u64, u64)>,
    live_semantics_last_sent_unix_ms: Option<u64>,
    live_semantics_last_force_nonce: u64,

    pack_tx: std::sync::mpsc::Sender<pack::PackJobResult>,
    pack_rx: std::sync::mpsc::Receiver<pack::PackJobResult>,
    summarize_tx: std::sync::mpsc::Sender<summarize::SummarizeJobResult>,
    summarize_rx: std::sync::mpsc::Receiver<summarize::SummarizeJobResult>,
    followup_tx: std::sync::mpsc::Sender<followup::FollowupJobResult>,
    followup_rx: std::sync::mpsc::Receiver<followup::FollowupJobResult>,
    gate_run_tx: std::sync::mpsc::Sender<gate_run::GateRunJobResult>,
    gate_run_rx: std::sync::mpsc::Receiver<gate_run::GateRunJobResult>,
    workflow_run_tx: std::sync::mpsc::Sender<workflow_run::WorkflowRunJobResult>,
    workflow_run_rx: std::sync::mpsc::Receiver<workflow_run::WorkflowRunJobResult>,
}

fn main() -> anyhow::Result<()> {
    let transport =
        env_transport_kind("FRET_DEVTOOLS_TRANSPORT").unwrap_or(DiagTransportKind::WebSocket);
    let fs_out_dir =
        std::env::var("FRET_DIAG_DIR").unwrap_or_else(|_| "target/fret-diag".to_string());

    let port = env_u16("FRET_DEVTOOLS_WS_PORT").unwrap_or(7331);
    let token =
        std::env::var("FRET_DEVTOOLS_TOKEN").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    eprintln!("fret-devtools: bind={bind} token={token}");
    eprintln!("fret-devtools: url=ws://127.0.0.1:{port}/?fret_devtools_token={token}");
    eprintln!("fret-devtools: transport={transport:?} fs_out_dir={fs_out_dir}");

    std::thread::spawn({
        let token = token.clone();
        move || {
            let server = DevtoolsWsServer::new(DevtoolsWsServerConfig { bind, token });
            let _ = server.run();
        }
    });

    let ws_url = Arc::<str>::from(format!("ws://127.0.0.1:{port}/"));
    let token = Arc::<str>::from(token);

    let mut app = App::new();
    app.set_global(DevtoolsConfig {
        transport,
        fs_out_dir: Arc::<str>::from(fs_out_dir),
        ws_port: port,
        ws_url: ws_url.clone(),
        token: token.clone(),
    });

    let driver = UiAppDriver::new("fret-devtools", init_window, view)
        .on_command(on_command)
        .into_fn_driver();

    BootstrapBuilder::new(app, driver)
        .with_default_config_files()?
        .with_lucide_icons()
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(app: &mut App, _window: AppWindowId) -> State {
    let cfg = app
        .global::<DevtoolsConfig>()
        .cloned()
        .expect("DevtoolsConfig must be set before starting the app");

    let panel_fractions = app.models_mut().insert(vec![0.22f32, 0.50f32, 0.28f32]);
    let left_tab = app.models_mut().insert(Some(Arc::<str>::from("semantics")));
    let details_tab = app.models_mut().insert(Some(Arc::<str>::from("guide")));
    let sessions = app
        .models_mut()
        .insert(Vec::<DevtoolsSessionDescriptorV1>::new());
    let selected_session_id = app.models_mut().insert(None::<Arc<str>>);
    let selected_session_open = app.models_mut().insert(false);
    let inspect_consume_clicks = app.models_mut().insert(false);
    let gate_profile_selected_id = app
        .models_mut()
        .insert(Some(Arc::<str>::from("stale-paint-scene")));
    let gate_profile_open = app.models_mut().insert(false);
    let gate_profile_script_json = app.models_mut().insert(String::new());
    let gate_profile_test_id = app.models_mut().insert(String::new());
    let perf_defaults = DevtoolsGatePerfThresholdCommandInputV1::product_chain_docking_defaults();
    let gate_profile_perf_target = app
        .models_mut()
        .insert(perf_defaults.target.to_string());
    let gate_profile_perf_repeat = app.models_mut().insert(perf_defaults.repeat.to_string());
    let gate_profile_perf_warmup_frames = app
        .models_mut()
        .insert(perf_defaults.warmup_frames.to_string());
    let gate_profile_perf_threshold_agg = app
        .models_mut()
        .insert(perf_defaults.perf_threshold_agg.to_string());
    let gate_profile_perf_max_top_total_us = app
        .models_mut()
        .insert(perf_defaults.max_top_total_us.to_string());
    let gate_profile_perf_max_top_layout_us = app
        .models_mut()
        .insert(perf_defaults.max_top_layout_us.to_string());
    let gate_profile_perf_max_top_solve_us = app
        .models_mut()
        .insert(perf_defaults.max_top_solve_us.to_string());
    let gate_profile_perf_max_pointer_move_dispatch_us = app
        .models_mut()
        .insert(perf_defaults.max_pointer_move_dispatch_us.to_string());
    let gate_profile_perf_max_pointer_move_hit_test_us = app
        .models_mut()
        .insert(perf_defaults.max_pointer_move_hit_test_us.to_string());
    let gate_profile_perf_max_pointer_move_global_changes = app
        .models_mut()
        .insert(perf_defaults.max_pointer_move_global_changes.to_string());
    let gate_profile_perf_max_renderer_encode_scene_us = app
        .models_mut()
        .insert(perf_defaults.max_renderer_encode_scene_us.to_string());
    let gate_profile_perf_max_renderer_upload_us = app
        .models_mut()
        .insert(perf_defaults.max_renderer_upload_us.to_string());
    let gate_profile_perf_max_renderer_record_passes_us = app
        .models_mut()
        .insert(perf_defaults.max_renderer_record_passes_us.to_string());
    let gate_profile_perf_max_renderer_encoder_finish_us = app
        .models_mut()
        .insert(perf_defaults.max_renderer_encoder_finish_us.to_string());
    let gate_profile_perf_max_renderer_prepare_text_us = app
        .models_mut()
        .insert(perf_defaults.max_renderer_prepare_text_us.to_string());
    let gate_profile_perf_max_renderer_prepare_svg_us = app
        .models_mut()
        .insert(perf_defaults.max_renderer_prepare_svg_us.to_string());
    let gate_profile_perf_max_renderer_instance_bytes = app
        .models_mut()
        .insert(perf_defaults.max_renderer_instance_bytes.to_string());
    let gate_profile_perf_max_renderer_encode_scene_text_ops = app
        .models_mut()
        .insert(perf_defaults.max_renderer_encode_scene_text_ops.to_string());
    let gate_profile_resource_target = app
        .models_mut()
        .insert("tools/diag-scripts/ui-gallery-intro-idle.json".to_string());
    let gate_profile_resource_max_working_set_bytes = app.models_mut().insert(String::new());
    let gate_profile_resource_max_peak_working_set_bytes = app.models_mut().insert(String::new());
    let gate_profile_resource_max_cpu_avg_percent_total_cores =
        app.models_mut().insert(String::new());
    let gate_profile_resource_launch_command = app.models_mut().insert(String::new());
    let gate_run_in_flight = app.models_mut().insert(false);
    let workflow_run_selected_id = app.models_mut().insert(Some(Arc::<str>::from(
        DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID,
    )));
    let workflow_run_selected_open = app.models_mut().insert(false);
    let workflow_run_in_flight = app.models_mut().insert(false);

    let repo_root = script_studio::repo_root_from_manifest_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let gate_run_initial_history = gate_run::load_recent_gate_run_result_history(&repo_root, 32);
    let workflow_run_initial_history =
        workflow_run::load_recent_workflow_run_result_history(&repo_root, 32);
    let followup_initial_history = followup::load_recent_followup_result_history(&repo_root, 32);
    let script_paths = script_studio::ScriptPaths::from_repo_root(repo_root);
    let gate_run_initial_latest = gate_run_initial_history.first().cloned();
    let gate_run_last_command_line = app.models_mut().insert(
        gate_run_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.command_line.clone())),
    );
    let gate_run_last_result_path = app.models_mut().insert(
        gate_run_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.result_path.clone())),
    );
    let gate_run_last_result_json = app.models_mut().insert(
        gate_run_initial_latest
            .as_ref()
            .map(|entry| entry.result_json.clone())
            .unwrap_or_default(),
    );
    let gate_run_result_history = app.models_mut().insert(gate_run_initial_history);
    let gate_run_selected_result_path = app.models_mut().insert(
        gate_run_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.result_path.clone())),
    );
    let gate_run_last_error = app.models_mut().insert(
        gate_run_initial_latest
            .as_ref()
            .and_then(|entry| entry.error.as_ref())
            .map(|error| Arc::<str>::from(error.clone())),
    );
    let workflow_run_initial_latest = workflow_run_initial_history.first().cloned();
    let workflow_run_last_command_line = app.models_mut().insert(
        workflow_run_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.command_line.clone())),
    );
    let workflow_run_last_result_path = app.models_mut().insert(
        workflow_run_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.result_path.clone())),
    );
    let workflow_run_last_result_json = app.models_mut().insert(
        workflow_run_initial_latest
            .as_ref()
            .map(|entry| entry.result_json.clone())
            .unwrap_or_default(),
    );
    let workflow_run_result_history = app.models_mut().insert(workflow_run_initial_history);
    let workflow_run_selected_result_path = app.models_mut().insert(
        workflow_run_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.result_path.clone())),
    );
    let workflow_run_last_error = app.models_mut().insert(
        workflow_run_initial_latest
            .as_ref()
            .and_then(|entry| entry.error.as_ref())
            .map(|error| Arc::<str>::from(error.clone())),
    );
    let script_library = app
        .models_mut()
        .insert(Vec::<script_studio::ScriptItem>::new());
    let loaded_script_origin = app.models_mut().insert(None::<script_studio::ScriptOrigin>);
    let loaded_script_path = app.models_mut().insert(None::<Arc<str>>);
    let script_apply_pointer = app.models_mut().insert("/steps/0/target".to_string());
    let script_studio_helper_tab = app.models_mut().insert(Some(Arc::<str>::from("steps")));
    let script_step_insert_index = app.models_mut().insert(String::new());
    let script_selector_kind = app.models_mut().insert(Some(Arc::<str>::from("test_id")));
    let script_selector_kind_open = app.models_mut().insert(false);
    let script_selector_test_id = app.models_mut().insert("TODO".to_string());
    let script_selector_role = app.models_mut().insert("button".to_string());
    let script_selector_name = app.models_mut().insert("TODO".to_string());
    let script_selector_ancestors = app.models_mut().insert(String::new());
    let script_selector_node_id = app.models_mut().insert("0".to_string());
    let script_selector_element_id = app.models_mut().insert("0".to_string());
    let script_predicate_kind = app.models_mut().insert(Some(Arc::<str>::from("exists")));
    let script_predicate_kind_open = app.models_mut().insert(false);
    let script_predicate_other_selector_json = app.models_mut().insert(String::new());
    let script_predicate_role = app.models_mut().insert("button".to_string());
    let script_predicate_checked = app.models_mut().insert(false);
    let script_predicate_len_bytes = app.models_mut().insert("0".to_string());
    let script_predicate_padding_px = app.models_mut().insert("0".to_string());
    let script_predicate_eps_px = app.models_mut().insert("0".to_string());
    let script_predicate_min_w_px = app.models_mut().insert("0".to_string());
    let script_predicate_min_h_px = app.models_mut().insert("0".to_string());
    let script_predicate_barrier_root = app.models_mut().insert(Some(Arc::<str>::from("any")));
    let script_predicate_barrier_root_open = app.models_mut().insert(false);
    let script_predicate_focus_barrier_root =
        app.models_mut().insert(Some(Arc::<str>::from("any")));
    let script_predicate_focus_barrier_root_open = app.models_mut().insert(false);
    let script_predicate_require_equal = app.models_mut().insert(Some(Arc::<str>::from("unset")));
    let script_predicate_require_equal_open = app.models_mut().insert(false);

    let script_text = app.models_mut().insert(String::new());
    let script_last_stage = app.models_mut().insert(None::<UiScriptStageV1>);
    let script_last_step_index = app.models_mut().insert(None::<u32>);
    let script_last_reason = app.models_mut().insert(None::<Arc<str>>);
    let script_last_bundle_dir = app.models_mut().insert(None::<Arc<str>>);
    let script_pack_after_run = app.models_mut().insert(false);

    let target_out_dir = match cfg.transport {
        DiagTransportKind::FileSystem => app.models_mut().insert(Some(cfg.fs_out_dir.clone())),
        DiagTransportKind::WebSocket => app.models_mut().insert(None::<Arc<str>>),
    };
    let last_bundle_dir_abs = app.models_mut().insert(None::<Arc<str>>);
    let last_bundle_dump_exported_unix_ms = app.models_mut().insert(None::<u64>);
    let last_bundle_dump_bundle_json = app.models_mut().insert(None::<Arc<str>>);
    let last_pack_path = app.models_mut().insert(None::<Arc<str>>);
    let pack_in_flight = app.models_mut().insert(false);
    let pack_last_error = app.models_mut().insert(None::<Arc<str>>);
    let summarize_in_flight = app.models_mut().insert(false);
    let summarize_last_error = app.models_mut().insert(None::<Arc<str>>);
    let followup_in_flight = app.models_mut().insert(false);
    let followup_initial_latest = followup_initial_history.first().cloned();
    let followup_last_command_line = app.models_mut().insert(
        followup_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.command_line.clone())),
    );
    let followup_last_result_path = app.models_mut().insert(
        followup_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.result_path.clone())),
    );
    let followup_last_result_json = app.models_mut().insert(
        followup_initial_latest
            .as_ref()
            .map(|entry| entry.result_json.clone())
            .unwrap_or_default(),
    );
    let followup_result_history = app.models_mut().insert(followup_initial_history);
    let followup_selected_result_path = app.models_mut().insert(
        followup_initial_latest
            .as_ref()
            .map(|entry| Arc::<str>::from(entry.result_path.clone())),
    );
    let followup_last_error = app.models_mut().insert(
        followup_initial_latest
            .as_ref()
            .and_then(|entry| entry.error.as_ref())
            .map(|error| Arc::<str>::from(error.clone())),
    );
    let followup_pending_command_id = app.models_mut().insert(None::<Arc<str>>);
    let followup_baseline_bundle_or_dir = app.models_mut().insert(String::new());
    let followup_baseline_session = app.models_mut().insert(String::new());
    let viewer_url = app.models_mut().insert("http://localhost:5173".to_string());
    let last_pick_json = app.models_mut().insert(String::new());
    let last_inspect_hover_json = app.models_mut().insert(String::new());
    let last_inspect_focus_json = app.models_mut().insert(String::new());
    let last_overlay_summary_json = app.models_mut().insert(String::new());
    let last_script_result_json = app.models_mut().insert(String::new());
    let last_bundle_json = app.models_mut().insert(String::new());
    let last_screenshot_json = app.models_mut().insert(String::new());
    let regression_summary_json = app.models_mut().insert(String::new());
    let regression_index_json = app.models_mut().insert(String::new());
    let regression_dashboard_human = app.models_mut().insert(String::new());
    let regression_loaded_dir = app.models_mut().insert(None::<Arc<str>>);
    let regression_last_error = app.models_mut().insert(None::<Arc<str>>);
    let regression_selected_summary_path = app.models_mut().insert(None::<Arc<str>>);
    let regression_selected_summary_json = app.models_mut().insert(String::new());
    let regression_selected_bundle_dirs = app.models_mut().insert(Vec::<Arc<str>>::new());
    let regression_selected_capability_sources = app.models_mut().insert(Vec::<Arc<str>>::new());
    let regression_selected_capabilities_checks = app.models_mut().insert(Vec::<Arc<str>>::new());
    let regression_selected_perf_evidence = app.models_mut().insert(Vec::<Arc<str>>::new());
    let regression_selected_first_open_evidence = app.models_mut().insert(Vec::<Arc<str>>::new());
    let regression_selected_share_artifacts = app.models_mut().insert(Vec::<Arc<str>>::new());
    let regression_selected_error = app.models_mut().insert(None::<Arc<str>>);
    let log_lines = match cfg.transport {
        DiagTransportKind::FileSystem => app.models_mut().insert(vec![Arc::<str>::from(format!(
            "filesystem transport: polling FRET_DIAG_DIR={}",
            cfg.fs_out_dir
        ))]),
        DiagTransportKind::WebSocket => app.models_mut().insert(Vec::<Arc<str>>::new()),
    };

    let semantics_cache = app
        .models_mut()
        .insert(None::<Arc<semantics::SemanticsIndex>>);
    let semantics_source_hash = app.models_mut().insert(None::<u64>);
    let semantics_error = app.models_mut().insert(None::<Arc<str>>);
    let semantics_search = app.models_mut().insert(String::new());
    let semantics_expanded = app.models_mut().insert(HashSet::<u64>::new());
    let semantics_selected_id = app.models_mut().insert(None::<u64>);
    let semantics_selected_node_json = app.models_mut().insert(String::new());
    let semantics_selected_node_live_json = app.models_mut().insert(String::new());
    let semantics_selected_node_live_status = app.models_mut().insert(None::<Arc<str>>);
    let semantics_selected_node_live_updated_unix_ms = app.models_mut().insert(None::<u64>);
    let semantics_selected_node_live_children = app.models_mut().insert(Vec::<u64>::new());
    let semantics_selected_hit_test_explain_json = app.models_mut().insert(String::new());
    let semantics_selected_hit_test_explain_summary = app.models_mut().insert(String::new());
    let semantics_selected_hit_test_explain_status = app.models_mut().insert(None::<Arc<str>>);
    let semantics_selected_hit_test_explain_updated_unix_ms = app.models_mut().insert(None::<u64>);
    let semantics_live_enabled = app.models_mut().insert(true);
    let semantics_live_force_nonce = app.models_mut().insert(0u64);

    let client = match cfg.transport {
        DiagTransportKind::WebSocket => {
            let mut client_cfg = DevtoolsWsClientConfig::with_defaults(
                cfg.ws_url.to_string(),
                cfg.token.to_string(),
            );
            client_cfg.client_kind = ClientKindV1::Tooling;
            client_cfg.capabilities = vec![
                "inspect".to_string(),
                "pick".to_string(),
                "scripts".to_string(),
                "bundles".to_string(),
            ];
            ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(client_cfg))
                .expect("devtools ws client connect must succeed")
        }
        DiagTransportKind::FileSystem => {
            let fs_cfg =
                FsDiagTransportConfig::from_out_dir(PathBuf::from(cfg.fs_out_dir.as_ref()));
            ToolingDiagClient::connect_fs(fs_cfg).expect("devtools fs client connect must succeed")
        }
    };
    let devtools = DevtoolsOps::new(client);

    let (pack_tx, pack_rx) = pack::new_pack_channel();
    let (summarize_tx, summarize_rx) = summarize::new_summarize_channel();
    let (followup_tx, followup_rx) = followup::new_followup_channel();
    let (gate_run_tx, gate_run_rx) = gate_run::new_gate_run_channel();
    let (workflow_run_tx, workflow_run_rx) = workflow_run::new_workflow_run_channel();

    let mut st = State {
        cfg,
        panel_fractions,
        left_tab,
        details_tab,
        sessions,
        selected_session_id,
        selected_session_open,
        inspect_consume_clicks,
        gate_profile_selected_id,
        gate_profile_open,
        gate_profile_script_json,
        gate_profile_test_id,
        gate_profile_perf_target,
        gate_profile_perf_repeat,
        gate_profile_perf_warmup_frames,
        gate_profile_perf_threshold_agg,
        gate_profile_perf_max_top_total_us,
        gate_profile_perf_max_top_layout_us,
        gate_profile_perf_max_top_solve_us,
        gate_profile_perf_max_pointer_move_dispatch_us,
        gate_profile_perf_max_pointer_move_hit_test_us,
        gate_profile_perf_max_pointer_move_global_changes,
        gate_profile_perf_max_renderer_encode_scene_us,
        gate_profile_perf_max_renderer_upload_us,
        gate_profile_perf_max_renderer_record_passes_us,
        gate_profile_perf_max_renderer_encoder_finish_us,
        gate_profile_perf_max_renderer_prepare_text_us,
        gate_profile_perf_max_renderer_prepare_svg_us,
        gate_profile_perf_max_renderer_instance_bytes,
        gate_profile_perf_max_renderer_encode_scene_text_ops,
        gate_profile_resource_target,
        gate_profile_resource_max_working_set_bytes,
        gate_profile_resource_max_peak_working_set_bytes,
        gate_profile_resource_max_cpu_avg_percent_total_cores,
        gate_profile_resource_launch_command,
        gate_run_in_flight,
        gate_run_last_command_line,
        gate_run_last_result_path,
        gate_run_last_result_json,
        gate_run_result_history,
        gate_run_selected_result_path,
        gate_run_last_error,
        workflow_run_selected_id,
        workflow_run_selected_open,
        workflow_run_in_flight,
        workflow_run_last_command_line,
        workflow_run_last_result_path,
        workflow_run_last_result_json,
        workflow_run_result_history,
        workflow_run_selected_result_path,
        workflow_run_last_error,
        script_paths,
        script_library,
        loaded_script_origin,
        loaded_script_path,
        script_apply_pointer,
        script_text,
        script_studio_helper_tab,
        script_step_insert_index,
        script_selector_kind,
        script_selector_kind_open,
        script_selector_test_id,
        script_selector_role,
        script_selector_name,
        script_selector_ancestors,
        script_selector_node_id,
        script_selector_element_id,
        script_predicate_kind,
        script_predicate_kind_open,
        script_predicate_other_selector_json,
        script_predicate_role,
        script_predicate_checked,
        script_predicate_len_bytes,
        script_predicate_padding_px,
        script_predicate_eps_px,
        script_predicate_min_w_px,
        script_predicate_min_h_px,
        script_predicate_barrier_root,
        script_predicate_barrier_root_open,
        script_predicate_focus_barrier_root,
        script_predicate_focus_barrier_root_open,
        script_predicate_require_equal,
        script_predicate_require_equal_open,
        script_last_stage,
        script_last_step_index,
        script_last_reason,
        script_last_bundle_dir,
        script_pack_after_run,
        target_out_dir,
        last_bundle_dir_abs,
        last_bundle_dump_exported_unix_ms,
        last_bundle_dump_bundle_json,
        last_pack_path,
        pack_in_flight,
        pack_last_error,
        summarize_in_flight,
        summarize_last_error,
        followup_in_flight,
        followup_last_command_line,
        followup_last_result_path,
        followup_last_result_json,
        followup_result_history,
        followup_selected_result_path,
        followup_last_error,
        followup_pending_command_id,
        followup_baseline_bundle_or_dir,
        followup_baseline_session,
        viewer_url,
        last_pick_json,
        last_inspect_hover_json,
        last_inspect_focus_json,
        last_overlay_summary_json,
        last_script_result_json,
        last_bundle_json,
        last_screenshot_json,
        regression_summary_json,
        regression_index_json,
        regression_dashboard_human,
        regression_loaded_dir,
        regression_last_error,
        regression_selected_summary_path,
        regression_selected_summary_json,
        regression_selected_bundle_dirs,
        regression_selected_capability_sources,
        regression_selected_capabilities_checks,
        regression_selected_perf_evidence,
        regression_selected_first_open_evidence,
        regression_selected_share_artifacts,
        regression_selected_error,
        log_lines,
        semantics_cache,
        semantics_source_hash,
        semantics_error,
        semantics_search,
        semantics_expanded,
        semantics_selected_id,
        semantics_selected_node_json,
        semantics_selected_node_live_json,
        semantics_selected_node_live_status,
        semantics_selected_node_live_updated_unix_ms,
        semantics_selected_node_live_children,
        semantics_selected_hit_test_explain_json,
        semantics_selected_hit_test_explain_summary,
        semantics_selected_hit_test_explain_status,
        semantics_selected_hit_test_explain_updated_unix_ms,
        semantics_live_enabled,
        semantics_live_force_nonce,
        devtools,
        applied_session_id: None,
        live_semantics_last_target: None,
        live_semantics_last_sent_unix_ms: None,
        live_semantics_last_force_nonce: 0,
        pack_tx,
        pack_rx,
        summarize_tx,
        summarize_rx,
        followup_tx,
        followup_rx,
        gate_run_tx,
        gate_run_rx,
        workflow_run_tx,
        workflow_run_rx,
    };

    refresh_script_library(app, &mut st);
    refresh_regression_artifacts(app, &mut st);
    st
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut State) -> ViewElements {
    pack::poll_pack_jobs(cx.app, st);
    summarize::poll_summarize_jobs(cx.app, st);
    followup::poll_followup_jobs(cx.app, st);
    gate_run::poll_gate_run_jobs(cx.app, st);
    workflow_run::poll_workflow_run_jobs(cx.app, st);
    ws::drain_ws_messages(cx.app, st);
    ws::sync_selected_session_to_client(cx.app, st);
    semantics::refresh_semantics_cache_if_needed(cx.app, st);
    ws::maybe_request_semantics_node_details(cx.app, st);

    let continuous_frames_slot = cx.slot_id();
    let mut needs_frames = false;
    cx.state_for(
        continuous_frames_slot,
        || None::<ContinuousFrames>,
        |lease: &mut Option<ContinuousFrames>| {
            if lease.is_none() {
                needs_frames = true;
            }
        },
    );
    if needs_frames {
        let lease = cx.begin_continuous_frames();
        cx.state_for(
            continuous_frames_slot,
            || None::<ContinuousFrames>,
            |slot: &mut Option<ContinuousFrames>| {
                *slot = Some(lease);
            },
        );
    }

    cx.observe_model(&st.panel_fractions, Invalidation::Layout);
    cx.observe_model(&st.left_tab, Invalidation::Paint);
    cx.observe_model(&st.details_tab, Invalidation::Paint);
    cx.observe_model(&st.sessions, Invalidation::Paint);
    cx.observe_model(&st.selected_session_id, Invalidation::Paint);
    cx.observe_model(&st.selected_session_open, Invalidation::Paint);
    cx.observe_model(&st.inspect_consume_clicks, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_selected_id, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_open, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_script_json, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_test_id, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_target, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_repeat, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_warmup_frames, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_threshold_agg, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_max_top_total_us, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_max_top_layout_us, Invalidation::Paint);
    cx.observe_model(&st.gate_profile_perf_max_top_solve_us, Invalidation::Paint);
    cx.observe_model(
        &st.gate_profile_perf_max_pointer_move_dispatch_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_pointer_move_hit_test_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_pointer_move_global_changes,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_encode_scene_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_upload_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_record_passes_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_encoder_finish_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_prepare_text_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_prepare_svg_us,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_instance_bytes,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_perf_max_renderer_encode_scene_text_ops,
        Invalidation::Paint,
    );
    cx.observe_model(&st.gate_profile_resource_target, Invalidation::Paint);
    cx.observe_model(
        &st.gate_profile_resource_max_working_set_bytes,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_resource_max_peak_working_set_bytes,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_resource_max_cpu_avg_percent_total_cores,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.gate_profile_resource_launch_command,
        Invalidation::Paint,
    );
    cx.observe_model(&st.gate_run_in_flight, Invalidation::Paint);
    cx.observe_model(&st.gate_run_last_command_line, Invalidation::Paint);
    cx.observe_model(&st.gate_run_last_result_path, Invalidation::Paint);
    cx.observe_model(&st.gate_run_last_result_json, Invalidation::Paint);
    cx.observe_model(&st.gate_run_result_history, Invalidation::Paint);
    cx.observe_model(&st.gate_run_selected_result_path, Invalidation::Paint);
    cx.observe_model(&st.gate_run_last_error, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_selected_id, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_selected_open, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_in_flight, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_last_command_line, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_last_result_path, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_last_result_json, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_result_history, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_selected_result_path, Invalidation::Paint);
    cx.observe_model(&st.workflow_run_last_error, Invalidation::Paint);
    cx.observe_model(&st.script_library, Invalidation::Paint);
    cx.observe_model(&st.loaded_script_origin, Invalidation::Paint);
    cx.observe_model(&st.loaded_script_path, Invalidation::Paint);
    cx.observe_model(&st.script_apply_pointer, Invalidation::Paint);
    cx.observe_model(&st.script_text, Invalidation::Paint);
    cx.observe_model(&st.script_studio_helper_tab, Invalidation::Paint);
    cx.observe_model(&st.script_step_insert_index, Invalidation::Paint);
    cx.observe_model(&st.script_selector_kind, Invalidation::Paint);
    cx.observe_model(&st.script_selector_kind_open, Invalidation::Paint);
    cx.observe_model(&st.script_selector_test_id, Invalidation::Paint);
    cx.observe_model(&st.script_selector_role, Invalidation::Paint);
    cx.observe_model(&st.script_selector_name, Invalidation::Paint);
    cx.observe_model(&st.script_selector_ancestors, Invalidation::Paint);
    cx.observe_model(&st.script_selector_node_id, Invalidation::Paint);
    cx.observe_model(&st.script_selector_element_id, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_kind, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_kind_open, Invalidation::Paint);
    cx.observe_model(
        &st.script_predicate_other_selector_json,
        Invalidation::Paint,
    );
    cx.observe_model(&st.script_predicate_role, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_checked, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_len_bytes, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_padding_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_eps_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_min_w_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_min_h_px, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_barrier_root, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_barrier_root_open, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_focus_barrier_root, Invalidation::Paint);
    cx.observe_model(
        &st.script_predicate_focus_barrier_root_open,
        Invalidation::Paint,
    );
    cx.observe_model(&st.script_predicate_require_equal, Invalidation::Paint);
    cx.observe_model(&st.script_predicate_require_equal_open, Invalidation::Paint);
    cx.observe_model(&st.script_last_stage, Invalidation::Paint);
    cx.observe_model(&st.script_last_step_index, Invalidation::Paint);
    cx.observe_model(&st.script_last_reason, Invalidation::Paint);
    cx.observe_model(&st.script_last_bundle_dir, Invalidation::Paint);
    cx.observe_model(&st.script_pack_after_run, Invalidation::Paint);
    cx.observe_model(&st.target_out_dir, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_dir_abs, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_dump_exported_unix_ms, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_dump_bundle_json, Invalidation::Paint);
    cx.observe_model(&st.last_pack_path, Invalidation::Paint);
    cx.observe_model(&st.pack_in_flight, Invalidation::Paint);
    cx.observe_model(&st.pack_last_error, Invalidation::Paint);
    cx.observe_model(&st.summarize_in_flight, Invalidation::Paint);
    cx.observe_model(&st.summarize_last_error, Invalidation::Paint);
    cx.observe_model(&st.followup_in_flight, Invalidation::Paint);
    cx.observe_model(&st.followup_last_command_line, Invalidation::Paint);
    cx.observe_model(&st.followup_last_result_path, Invalidation::Paint);
    cx.observe_model(&st.followup_last_result_json, Invalidation::Paint);
    cx.observe_model(&st.followup_result_history, Invalidation::Paint);
    cx.observe_model(&st.followup_selected_result_path, Invalidation::Paint);
    cx.observe_model(&st.followup_last_error, Invalidation::Paint);
    cx.observe_model(&st.followup_baseline_bundle_or_dir, Invalidation::Paint);
    cx.observe_model(&st.followup_baseline_session, Invalidation::Paint);
    cx.observe_model(&st.viewer_url, Invalidation::Paint);
    cx.observe_model(&st.last_pick_json, Invalidation::Paint);
    cx.observe_model(&st.last_inspect_hover_json, Invalidation::Paint);
    cx.observe_model(&st.last_inspect_focus_json, Invalidation::Paint);
    cx.observe_model(&st.last_overlay_summary_json, Invalidation::Paint);
    cx.observe_model(&st.last_script_result_json, Invalidation::Paint);
    cx.observe_model(&st.last_bundle_json, Invalidation::Paint);
    cx.observe_model(&st.last_screenshot_json, Invalidation::Paint);
    cx.observe_model(&st.regression_summary_json, Invalidation::Paint);
    cx.observe_model(&st.regression_index_json, Invalidation::Paint);
    cx.observe_model(&st.regression_dashboard_human, Invalidation::Paint);
    cx.observe_model(&st.regression_loaded_dir, Invalidation::Paint);
    cx.observe_model(&st.regression_last_error, Invalidation::Paint);
    cx.observe_model(&st.regression_selected_summary_path, Invalidation::Paint);
    cx.observe_model(&st.regression_selected_summary_json, Invalidation::Paint);
    cx.observe_model(&st.regression_selected_bundle_dirs, Invalidation::Paint);
    cx.observe_model(
        &st.regression_selected_capability_sources,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.regression_selected_capabilities_checks,
        Invalidation::Paint,
    );
    cx.observe_model(&st.regression_selected_perf_evidence, Invalidation::Paint);
    cx.observe_model(
        &st.regression_selected_first_open_evidence,
        Invalidation::Paint,
    );
    cx.observe_model(&st.regression_selected_share_artifacts, Invalidation::Paint);
    cx.observe_model(&st.regression_selected_error, Invalidation::Paint);
    cx.observe_model(&st.log_lines, Invalidation::Paint);
    cx.observe_model(&st.semantics_cache, Invalidation::Paint);
    cx.observe_model(&st.semantics_error, Invalidation::Paint);
    cx.observe_model(&st.semantics_search, Invalidation::Paint);
    cx.observe_model(&st.semantics_expanded, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_id, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_node_json, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_node_live_json, Invalidation::Paint);
    cx.observe_model(&st.semantics_selected_node_live_status, Invalidation::Paint);
    cx.observe_model(
        &st.semantics_selected_node_live_updated_unix_ms,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.semantics_selected_node_live_children,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.semantics_selected_hit_test_explain_json,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.semantics_selected_hit_test_explain_summary,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.semantics_selected_hit_test_explain_status,
        Invalidation::Paint,
    );
    cx.observe_model(
        &st.semantics_selected_hit_test_explain_updated_unix_ms,
        Invalidation::Paint,
    );
    cx.observe_model(&st.semantics_live_enabled, Invalidation::Paint);
    cx.observe_model(&st.semantics_live_force_nonce, Invalidation::Paint);

    let theme = cx.theme_snapshot();

    let header = header_bar(cx, theme.clone(), st);
    let body = resizable_body(cx, theme.clone(), st);
    let footer = footer_bar(cx, theme.clone(), st);

    let body_slot = cx.container(
        fret_ui_kit::declarative::style::container_props(
            &theme,
            fret_ui_kit::ChromeRefinement::default(),
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .flex_1()
                .min_h_0(),
        ),
        |_cx| [body],
    );

    let shell = ui::v_stack(|_cx| [header, body_slot, footer])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx);

    let wrap = fret_ui_kit::declarative::style::container_props(
        &theme,
        fret_ui_kit::ChromeRefinement::default()
            .bg(fret_ui_kit::ColorRef::Color(
                theme.color_token("background"),
            ))
            .p(fret_ui_kit::Space::N2),
        fret_ui_kit::LayoutRefinement::default().w_full().h_full(),
    );

    vec![cx.container(wrap, |_cx| [shell])].into()
}

fn header_bar(
    cx: &mut ElementContext<'_, App>,
    _theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let ws_url_with_token = format!(
        "{}?fret_devtools_token={}",
        st.cfg.ws_url.as_ref(),
        st.cfg.token.as_ref()
    );
    let has_session = cx
        .app
        .models()
        .read(&st.selected_session_id, |v| v.is_some())
        .unwrap_or(false);
    let selected_session = cx
        .app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let session_count = cx
        .app
        .models()
        .read(&st.sessions, |sessions| sessions.len())
        .unwrap_or(0);
    let header = collect_header_diagnostics_state(cx.app, st);
    let session_items = cx
        .app
        .models()
        .read(&st.sessions, |sessions| {
            sessions
                .iter()
                .map(|s| {
                    let label = if s.client_version.trim().is_empty() {
                        format!("{} ({})", s.session_id, s.client_kind)
                    } else {
                        format!("{} ({} {})", s.session_id, s.client_kind, s.client_version)
                    };
                    shadcn::SelectItem::new(s.session_id.clone(), label)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let session_select = shadcn::Select::new(
        st.selected_session_id.clone(),
        st.selected_session_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("Session"))
    .items(session_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(240.0)))
    .into_element(cx);

    let transport_label = match st.cfg.transport {
        DiagTransportKind::WebSocket => "WebSocket",
        DiagTransportKind::FileSystem => "Filesystem",
    };
    let session_status_label = selected_session
        .as_deref()
        .map(|value| format!("Session {value}"))
        .unwrap_or_else(|| "No session selected".to_string());
    let shell_badges = ui::h_row(|cx| {
        [
            shadcn::Badge::new(transport_label)
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            shadcn::Badge::new(format!("{} sessions", session_count))
                .variant(if session_count > 0 {
                    shadcn::BadgeVariant::Secondary
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(session_status_label)
                .variant(if has_session {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let endpoint_line = cx.text(format!("Endpoint: {ws_url_with_token}"));
    let workspace_line = cx.text(format!(
        "Artifacts root: {} | token: {} | port: {}",
        st.cfg.fs_out_dir, st.cfg.token, st.cfg.ws_port
    ));
    let first_open_recent_evidence_actions = first_open_recent_evidence_action_specs(
        header.recent_failed_evidence_target.is_some(),
        header.recent_failed_evidence_rerunnable_kind.is_some(),
    );

    let mut next_action_rows = Vec::new();
    for line in header_next_action_lines(st, &header) {
        next_action_rows.push(cx.text(line));
    }
    next_action_rows.push(first_open_recent_evidence_action_row(
        cx,
        &first_open_recent_evidence_actions,
    ));
    let next_actions_panel = diag_section(
        cx,
        "First-open Next Actions",
        "Stateful next-step summary stays in the header; full command references live in the Guide tab.",
        next_action_rows,
    );

    let connection_actions = ui::h_row(|cx| {
        [
            session_select,
            shadcn::Button::new("Copy WS URL")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_COPY_WS_URL)
                .into_element(cx),
            shadcn::Button::new("Copy Token")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_COPY_TOKEN)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let quick_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Inspect On")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session)
                .on_click(CMD_INSPECT_ENABLE)
                .into_element(cx),
            shadcn::Button::new("Inspect Off")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session)
                .on_click(CMD_INSPECT_DISABLE)
                .into_element(cx),
            shadcn::Button::new("Pick")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session)
                .on_click(CMD_PICK_ARM)
                .into_element(cx),
            shadcn::Button::new("Dump Bundle")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session)
                .on_click(CMD_BUNDLE_DUMP)
                .into_element(cx),
            shadcn::Button::new("Screenshot")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session)
                .on_click(CMD_SCREENSHOT_REQUEST)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Diagnostics Workspace").into_element(cx),
            shadcn::CardDescription::new(
                "Top-level shell for sessions, transport controls, and capture actions.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            shell_badges,
            endpoint_line,
            workspace_line,
            next_actions_panel,
            connection_actions,
            quick_actions,
        ])
        .into_element(cx),
    ])
    .into_element(cx)
}

fn footer_bar(
    cx: &mut ElementContext<'_, App>,
    theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let has_session = cx
        .app
        .models()
        .read(&st.selected_session_id, |v| v.is_some())
        .unwrap_or(false);
    let pack_in_flight = cx
        .app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    let summarize_in_flight = cx
        .app
        .models()
        .read(&st.summarize_in_flight, |v| *v)
        .unwrap_or(false);
    let workflow_run_in_flight = cx
        .app
        .models()
        .read(&st.workflow_run_in_flight, |v| *v)
        .unwrap_or(false);
    let regression_loaded = cx
        .app
        .models()
        .read(&st.regression_loaded_dir, |v| v.is_some())
        .unwrap_or(false);
    let scripts_count = cx
        .app
        .models()
        .read(&st.script_library, |v| v.len())
        .unwrap_or(0);

    let status_badges = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if has_session {
                "Session ready"
            } else {
                "Session idle"
            })
            .variant(if has_session {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if pack_in_flight {
                "Pack busy"
            } else {
                "Pack idle"
            })
            .variant(if pack_in_flight {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if summarize_in_flight {
                "Summarize busy"
            } else {
                "Summarize idle"
            })
            .variant(if summarize_in_flight {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if workflow_run_in_flight {
                "Workflow busy"
            } else {
                "Workflow idle"
            })
            .variant(if workflow_run_in_flight {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if regression_loaded {
                "Regression loaded"
            } else {
                "Regression unloaded"
            })
            .variant(if regression_loaded {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("scripts {}", scripts_count))
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    cx.container(
        fret_ui_kit::declarative::style::container_props(
            &theme,
            fret_ui_kit::ChromeRefinement::default()
                .bg(fret_ui_kit::ColorRef::Color(theme.color_token("muted")))
                .px(fret_ui_kit::Space::N3)
                .py(fret_ui_kit::Space::N2)
                .border_1()
                .border_color(fret_ui_kit::ColorRef::Color(theme.color_token("border"))),
            fret_ui_kit::LayoutRefinement::default().w_full(),
        ),
        |_cx| [status_badges],
    )
}

fn resizable_body(
    cx: &mut ElementContext<'_, App>,
    theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let group = shadcn::ResizablePanelGroup::new(st.panel_fractions.clone())
        .axis(fret_core::Axis::Horizontal)
        .entries([
            shadcn::ResizablePanel::new([left_panel(cx, theme.clone(), st)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([center_panel(cx, theme.clone(), st)]).into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([right_panel(cx, theme.clone(), st)]).into(),
        ])
        .into_element(cx);

    cx.container(
        fret_ui_kit::declarative::style::container_props(
            &theme,
            fret_ui_kit::ChromeRefinement::default(),
            fret_ui_kit::LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| [group],
    )
}

fn left_panel(
    cx: &mut ElementContext<'_, App>,
    _theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let active_left_tab = cx
        .app
        .models()
        .read(&st.left_tab, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("semantics"));
    let semantics = if active_left_tab.as_ref() == "semantics" {
        semantics_panel(cx, st)
    } else {
        cx.text("")
    };
    let layout_tree = if active_left_tab.as_ref() == "layout" {
        layout_tree_panel(cx, st)
    } else {
        cx.text("")
    };
    let element_tree = if active_left_tab.as_ref() == "elements" {
        element_tree_panel(cx, st)
    } else {
        cx.text("")
    };
    let lines = cx
        .app
        .models()
        .read(&st.log_lines, |v| v.clone())
        .unwrap_or_default();

    let mut rows: Vec<AnyElement> = Vec::new();
    rows.reserve(lines.len().min(500));
    for (i, line) in lines.iter().rev().take(500).enumerate() {
        rows.push(cx.keyed(i as u64, |cx| cx.text(line.as_ref())));
    }

    let list = shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .into_element(cx);

    let tabs = shadcn::Tabs::new(st.left_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("semantics", "Semantics", [semantics]),
            shadcn::TabsItem::new("layout", "Layout", [layout_tree]),
            shadcn::TabsItem::new("elements", "Elements", [element_tree]),
            shadcn::TabsItem::new("events", "Events", [list]),
        ])
        .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Inspect Workspace").into_element(cx),
            shadcn::CardDescription::new(
                "Semantics navigation, layout bounds, element identity, and recent diagnostics events.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([tabs]).into_element(cx),
    ])
    .into_element(cx)
}

fn right_panel(
    cx: &mut ElementContext<'_, App>,
    _theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let pick = cx
        .app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    let inspect_hover = cx
        .app
        .models()
        .read(&st.last_inspect_hover_json, |v| v.clone())
        .unwrap_or_default();
    let inspect_focus = cx
        .app
        .models()
        .read(&st.last_inspect_focus_json, |v| v.clone())
        .unwrap_or_default();
    let overlay_summary = cx
        .app
        .models()
        .read(&st.last_overlay_summary_json, |v| v.clone())
        .unwrap_or_default();
    let script = cx
        .app
        .models()
        .read(&st.last_script_result_json, |v| v.clone())
        .unwrap_or_default();
    let bundle = cx
        .app
        .models()
        .read(&st.last_bundle_json, |v| v.clone())
        .unwrap_or_default();
    let screenshot = cx
        .app
        .models()
        .read(&st.last_screenshot_json, |v| v.clone())
        .unwrap_or_default();
    let guide = devtools_guide_panel(cx, st);
    let regression = regression_panel(cx, st);
    let semantics_node = sem_node_panel(cx, st);

    let inspect = inspect_panel(cx, &inspect_hover, &inspect_focus, &overlay_summary);

    let tabs = shadcn::Tabs::new(st.details_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("guide", "Guide", [guide]),
            shadcn::TabsItem::new("inspect", "Inspect", [inspect]),
            shadcn::TabsItem::new("pick", "Pick", [text_blob(cx, pick)]),
            shadcn::TabsItem::new("script", "Script", [text_blob(cx, script)]),
            shadcn::TabsItem::new("bundle", "Bundle", [text_blob(cx, bundle)]),
            shadcn::TabsItem::new("screenshot", "Screenshot", [text_blob(cx, screenshot)]),
            shadcn::TabsItem::new("regression", "Regression", [regression]),
            shadcn::TabsItem::new("sem_node", "Sem Node", [semantics_node]),
        ])
        .into_element(cx);

    diag_card(
        cx,
        "Evidence & Results",
        "Latest inspect, script, bundle, screenshot, and regression payloads.",
        vec![tabs],
    )
}

fn shell_quote_for_display(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return "''".to_string();
    }
    let needs_quote = value
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '\'' | '"' | '(' | ')' | '[' | ']'));
    if !needs_quote {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn short_artifact_result_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(path)
        .to_string()
}

fn file_url_from_path(path: &str) -> String {
    let normalized = path.trim().replace('\\', "/");
    let encoded = percent_encode_file_url_path(&normalized);
    if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    }
}

fn percent_encode_file_url_path(path: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'/'
            | b':' => out.push(byte as char),
            _ => {
                let _ = write!(&mut out, "%{byte:02X}");
            }
        }
    }
    out
}

fn selected_followup_history_filter_dirs_from_bundle_dirs(
    script_paths: &script_studio::ScriptPaths,
    selected_bundle_dirs: &[Arc<str>],
) -> Vec<String> {
    let repo_root = repo_root_from_script_paths(script_paths);
    let mut dirs: Vec<String> = Vec::new();
    for dir in selected_bundle_dirs.iter() {
        let dir = dir.trim();
        if dir.is_empty() {
            continue;
        }
        dirs.push(dir.to_string());
        if !is_abs_path(dir) {
            dirs.push(repo_root.join(dir).to_string_lossy().to_string());
        }
    }
    dirs
}

fn selected_followup_history_filter_dirs_from_state(app: &App, st: &State) -> Vec<String> {
    let selected_bundle_dirs = app
        .models()
        .read(&st.regression_selected_bundle_dirs, |v| v.clone())
        .unwrap_or_default();
    selected_followup_history_filter_dirs_from_bundle_dirs(&st.script_paths, &selected_bundle_dirs)
}

fn followup_result_history_from_state(
    app: &App,
    st: &State,
) -> Vec<followup::FollowupResultHistoryEntry> {
    app.models()
        .read(&st.followup_result_history, |v| v.clone())
        .unwrap_or_default()
}

fn selected_followup_result_entry_from_state(
    app: &App,
    st: &State,
) -> Option<followup::FollowupResultHistoryEntry> {
    let selected_followup_history_filter_dirs =
        selected_followup_history_filter_dirs_from_state(app, st);
    let followup_result_history = followup_result_history_from_state(app, st);
    let followup_selected_result_path = app
        .models()
        .read(&st.followup_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    followup::followup_result_history_selected_or_latest_entry(
        &followup_result_history,
        selected_followup_history_filter_dirs
            .iter()
            .map(|value| value.as_str()),
        followup_selected_result_path.as_deref(),
    )
}

fn selected_followup_result_path_from_state(app: &App, st: &State) -> Option<String> {
    selected_followup_result_entry_from_state(app, st).map(|entry| entry.result_path)
}

fn selected_followup_result_command_from_state(app: &App, st: &State) -> Option<String> {
    selected_followup_result_entry_from_state(app, st).map(|entry| entry.command_line)
}

fn selected_followup_result_json_from_state(app: &App, st: &State) -> Option<String> {
    selected_followup_result_entry_from_state(app, st).map(|entry| entry.result_json)
}

fn selected_followup_result_loaded_from_state(app: &App, st: &State) -> bool {
    selected_followup_result_entry_from_state(app, st).is_some()
}

fn selected_followup_trace_artifact_path_from_state(app: &App, st: &State) -> Option<String> {
    let result_json = selected_followup_result_json_from_state(app, st)?;
    let artifact_path = followup::followup_trace_artifact_path_from_result_json(&result_json)?;
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    Some(
        resolve_repo_or_abs_path(&repo_root, &artifact_path)
            .to_string_lossy()
            .to_string(),
    )
}

fn devtools_recent_evidence_lines_from_state(app: &App, st: &State) -> Vec<String> {
    let gate_entries = gate_run_result_history_from_state(app, st);
    let workflow_entries = workflow_run_result_history_from_state(app, st);
    let followup_entries = followup_result_history_from_state(app, st);
    let workflow_commands = devtools_workflow_commands_from_state(app, st);
    devtools_recent_evidence_lines_with_workflow_commands(
        &gate_entries,
        &workflow_entries,
        &followup_entries,
        &workflow_commands,
    )
}

fn devtools_recent_failed_evidence_target_from_state(
    app: &App,
    st: &State,
) -> Option<RecentEvidenceTarget> {
    let gate_entries = gate_run_result_history_from_state(app, st);
    let workflow_entries = workflow_run_result_history_from_state(app, st);
    let followup_entries = followup_result_history_from_state(app, st);
    devtools_recent_failed_evidence_target(&gate_entries, &workflow_entries, &followup_entries)
}

fn select_recent_evidence_target(app: &mut App, st: &State, target: &RecentEvidenceTarget) {
    let effect = devtools_recent_evidence_selection_effect(target);
    let _ = app.models_mut().update(&st.details_tab, |v| {
        *v = Some(Arc::<str>::from(effect.details_tab));
    });
    match target.kind {
        "gate" => {
            let _ = app
                .models_mut()
                .update(&st.gate_run_selected_result_path, |v| {
                    *v = Some(Arc::<str>::from(effect.selected_path));
                });
        }
        "workflow" => {
            let _ = app
                .models_mut()
                .update(&st.workflow_run_selected_result_path, |v| {
                    *v = Some(Arc::<str>::from(effect.selected_path));
                });
        }
        "follow-up" => {
            let _ = app
                .models_mut()
                .update(&st.followup_selected_result_path, |v| {
                    *v = Some(Arc::<str>::from(effect.selected_path));
                });
            if let Some(bundle_dir) = effect
                .selected_bundle_dir
                .filter(|value| !value.trim().is_empty())
            {
                let _ = app
                    .models_mut()
                    .update(&st.regression_selected_bundle_dirs, |v| {
                        *v = vec![Arc::<str>::from(bundle_dir)];
                    });
            }
        }
        _ => {}
    }
}

fn gate_run_result_history_from_state(
    app: &App,
    st: &State,
) -> Vec<gate_run::GateRunResultHistoryEntry> {
    app.models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default()
}

fn selected_gate_run_result_entry_from_state(
    app: &App,
    st: &State,
) -> Option<gate_run::GateRunResultHistoryEntry> {
    let gate_run_result_history = gate_run_result_history_from_state(app, st);
    let gate_run_selected_result_path = app
        .models()
        .read(&st.gate_run_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    gate_run::gate_run_result_history_selected_or_latest_entry(
        &gate_run_result_history,
        gate_run_selected_result_path.as_deref(),
    )
}

fn selected_gate_run_result_path_from_state(app: &App, st: &State) -> Option<String> {
    selected_gate_run_result_entry_from_state(app, st).map(|entry| entry.result_path)
}

fn selected_gate_run_result_command_from_state(app: &App, st: &State) -> Option<String> {
    selected_gate_run_result_entry_from_state(app, st).map(|entry| entry.command_line)
}

fn selected_gate_run_result_json_from_state(app: &App, st: &State) -> Option<String> {
    selected_gate_run_result_entry_from_state(app, st).map(|entry| entry.result_json)
}

fn workflow_run_result_history_from_state(
    app: &App,
    st: &State,
) -> Vec<workflow_run::WorkflowRunResultHistoryEntry> {
    app.models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default()
}

fn selected_workflow_run_result_entry_from_state(
    app: &App,
    st: &State,
) -> Option<workflow_run::WorkflowRunResultHistoryEntry> {
    let workflow_run_result_history = workflow_run_result_history_from_state(app, st);
    let workflow_run_selected_result_path = app
        .models()
        .read(&st.workflow_run_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    workflow_run::workflow_run_result_history_selected_or_latest_entry(
        &workflow_run_result_history,
        workflow_run_selected_result_path.as_deref(),
    )
}

fn selected_workflow_run_result_path_from_state(app: &App, st: &State) -> Option<String> {
    selected_workflow_run_result_entry_from_state(app, st).map(|entry| entry.result_path)
}

fn selected_workflow_run_result_command_from_state(app: &App, st: &State) -> Option<String> {
    selected_workflow_run_result_entry_from_state(app, st).map(|entry| entry.command_line)
}

fn selected_workflow_run_result_json_from_state(app: &App, st: &State) -> Option<String> {
    if let Some(entry) = selected_workflow_run_result_entry_from_state(app, st) {
        return Some(entry.result_json);
    }
    app.models()
        .read(&st.workflow_run_last_result_json, |v| v.clone())
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn selected_workflow_run_regression_summary_path_from_state(
    app: &App,
    st: &State,
) -> Option<String> {
    let result_json = selected_workflow_run_result_json_from_state(app, st)?;
    let artifact_path =
        workflow_run::workflow_run_regression_summary_artifact_path_from_result_json(&result_json)?;
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    Some(
        resolve_repo_or_abs_path(&repo_root, &artifact_path)
            .to_string_lossy()
            .to_string(),
    )
}

fn selected_workflow_run_suite_summary_path_from_state(app: &App, st: &State) -> Option<String> {
    let result_json = selected_workflow_run_result_json_from_state(app, st)?;
    let artifact_path =
        workflow_run::workflow_run_output_artifact_path_from_result_json(&result_json, "suite.summary.json")?;
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    Some(
        resolve_repo_or_abs_path(&repo_root, &artifact_path)
            .to_string_lossy()
            .to_string(),
    )
}

fn selected_workflow_run_regression_index_path_from_state(
    app: &App,
    st: &State,
) -> Option<String> {
    let result_json = selected_workflow_run_result_json_from_state(app, st)?;
    let artifact_path =
        workflow_run::workflow_run_regression_index_artifact_path_from_result_json(&result_json)?;
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    Some(
        resolve_repo_or_abs_path(&repo_root, &artifact_path)
            .to_string_lossy()
            .to_string(),
    )
}

fn workflow_regression_index_parent_dir(index_path: &str) -> Option<String> {
    Path::new(index_path.trim())
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| path.to_string_lossy().to_string())
}

fn normalize_workflow_artifact_path(path: &str) -> String {
    path.trim().replace('\\', "/").trim_end_matches('/').to_string()
}

fn workflow_aggregate_index_loaded(
    aggregate_index_path: Option<&str>,
    loaded_regression_dir: Option<&str>,
    regression_index_loaded: bool,
) -> bool {
    if !regression_index_loaded {
        return false;
    }
    let Some(index_path) = aggregate_index_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let Some(loaded_dir) = loaded_regression_dir
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let Some(index_parent) = workflow_regression_index_parent_dir(index_path) else {
        return false;
    };
    normalize_workflow_artifact_path(&index_parent) == normalize_workflow_artifact_path(loaded_dir)
}

fn workflow_summarize_command_from_summary_path(
    regression_summary_path: &str,
) -> Option<workflow_run::DevtoolsWorkflowRunCommandV1> {
    let summary_path = regression_summary_path.trim();
    if summary_path.is_empty() {
        return None;
    }
    let summary = PathBuf::from(summary_path);
    let out_dir = summary.parent().filter(|path| !path.as_os_str().is_empty())?;
    let out_dir_text = out_dir.to_string_lossy().to_string();
    Some(workflow_run::DevtoolsWorkflowRunCommandV1 {
        id: "summarize-workflow-regression-index".to_string(),
        label: "Generate workflow regression index".to_string(),
        command_line: format!(
            "cargo run -p fretboard-dev -- diag summarize {} --dir {} --json",
            shell_quote_for_display(summary_path),
            shell_quote_for_display(&out_dir_text)
        ),
        diag_args: vec![
            "summarize".to_string(),
            summary_path.to_string(),
            "--dir".to_string(),
            out_dir_text,
            "--json".to_string(),
        ],
        missing_inputs: Vec::new(),
    })
}

fn selected_workflow_summarize_command_from_state(
    app: &App,
    st: &State,
) -> Option<workflow_run::DevtoolsWorkflowRunCommandV1> {
    let summary_path = selected_workflow_run_regression_summary_path_from_state(app, st)?;
    workflow_summarize_command_from_summary_path(&summary_path)
}

fn selected_workflow_run_command_from_state(
    app: &App,
    st: &State,
) -> Option<workflow_run::DevtoolsWorkflowRunCommandV1> {
    let selected_workflow_id = app
        .models()
        .read(&st.workflow_run_selected_id, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from(DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID));
    workflow_run_command_by_id_from_state(app, st, selected_workflow_id.as_ref())
}

fn workflow_run_command_by_id_from_state(
    app: &App,
    st: &State,
    workflow_id: &str,
) -> Option<workflow_run::DevtoolsWorkflowRunCommandV1> {
    devtools_workflow_commands_from_state(app, st)
        .into_iter()
        .find(|command| command.id == workflow_id)
}

fn generated_gate_command_from_state(
    app: &App,
    st: &State,
) -> Option<fret_diag::DevtoolsGateCommandV1> {
    let selected_profile_id = app
        .models()
        .read(&st.gate_profile_selected_id, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("stale-paint-scene"));
    if selected_profile_id.as_ref() == "perf-thresholds" {
        let target = app
            .models()
            .read(&st.gate_profile_perf_target, |v| v.clone())
            .unwrap_or_default();
        let repeat = app
            .models()
            .read(&st.gate_profile_perf_repeat, |v| v.clone())
            .unwrap_or_default();
        let warmup_frames = app
            .models()
            .read(&st.gate_profile_perf_warmup_frames, |v| v.clone())
            .unwrap_or_default();
        let perf_threshold_agg = app
            .models()
            .read(&st.gate_profile_perf_threshold_agg, |v| v.clone())
            .unwrap_or_default();
        let max_top_total_us = app
            .models()
            .read(&st.gate_profile_perf_max_top_total_us, |v| v.clone())
            .unwrap_or_default();
        let max_top_layout_us = app
            .models()
            .read(&st.gate_profile_perf_max_top_layout_us, |v| v.clone())
            .unwrap_or_default();
        let max_top_solve_us = app
            .models()
            .read(&st.gate_profile_perf_max_top_solve_us, |v| v.clone())
            .unwrap_or_default();
        let max_pointer_move_dispatch_us = app
            .models()
            .read(&st.gate_profile_perf_max_pointer_move_dispatch_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_pointer_move_hit_test_us = app
            .models()
            .read(&st.gate_profile_perf_max_pointer_move_hit_test_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_pointer_move_global_changes = app
            .models()
            .read(&st.gate_profile_perf_max_pointer_move_global_changes, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_encode_scene_us = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_encode_scene_us, |v| v.clone())
            .unwrap_or_default();
        let max_renderer_upload_us = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_upload_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_record_passes_us = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_record_passes_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_encoder_finish_us = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_encoder_finish_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_prepare_text_us = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_prepare_text_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_prepare_svg_us = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_prepare_svg_us, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_instance_bytes = app
            .models()
            .read(&st.gate_profile_perf_max_renderer_instance_bytes, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let max_renderer_encode_scene_text_ops = app
            .models()
            .read(
                &st.gate_profile_perf_max_renderer_encode_scene_text_ops,
                |v| v.clone(),
            )
            .unwrap_or_default();
        let input = DevtoolsGatePerfThresholdCommandInputV1::new(
            &target,
            &repeat,
            &warmup_frames,
            &perf_threshold_agg,
            &max_top_total_us,
            &max_top_layout_us,
            &max_top_solve_us,
            &max_pointer_move_dispatch_us,
            &max_pointer_move_hit_test_us,
            &max_pointer_move_global_changes,
            &max_renderer_encode_scene_us,
            &max_renderer_upload_us,
            &max_renderer_record_passes_us,
            &max_renderer_encoder_finish_us,
            &max_renderer_prepare_text_us,
            &max_renderer_prepare_svg_us,
            &max_renderer_instance_bytes,
            &max_renderer_encode_scene_text_ops,
        );
        return Some(devtools_gate_perf_threshold_command(input));
    }
    if selected_profile_id.as_ref() == "resource-footprint-thresholds" {
        let target = app
            .models()
            .read(&st.gate_profile_resource_target, |v| v.clone())
            .unwrap_or_default();
        let max_working_set_bytes = app
            .models()
            .read(&st.gate_profile_resource_max_working_set_bytes, |v| v.clone())
            .unwrap_or_default();
        let max_peak_working_set_bytes = app
            .models()
            .read(&st.gate_profile_resource_max_peak_working_set_bytes, |v| v.clone())
            .unwrap_or_default();
        let max_cpu_avg_percent_total_cores = app
            .models()
            .read(&st.gate_profile_resource_max_cpu_avg_percent_total_cores, |v| {
                v.clone()
            })
            .unwrap_or_default();
        let launch_command = app
            .models()
            .read(&st.gate_profile_resource_launch_command, |v| v.clone())
            .unwrap_or_default();
        let input = DevtoolsGateResourceFootprintThresholdCommandInputV1::new(
            &target,
            &max_working_set_bytes,
            &max_peak_working_set_bytes,
            &max_cpu_avg_percent_total_cores,
            &launch_command,
        );
        return Some(devtools_gate_resource_footprint_threshold_command(input));
    }

    let script_json = app
        .models()
        .read(&st.gate_profile_script_json, |v| v.clone())
        .unwrap_or_default();
    let test_id = app
        .models()
        .read(&st.gate_profile_test_id, |v| v.clone())
        .unwrap_or_default();
    let input = DevtoolsGateScriptTargetCommandInputV1::new(&script_json, &test_id);
    devtools_gate_script_target_command(selected_profile_id.as_ref(), input)
}

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

fn on_command(
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

fn refresh_script_library(app: &mut App, st: &mut State) {
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

fn regression_artifacts_root(app: &mut App, st: &State) -> Option<PathBuf> {
    let out_dir = app
        .models()
        .read(&st.target_out_dir, |v| v.clone())
        .ok()
        .flatten()?;
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    Some(resolve_repo_or_abs_path(&repo_root, out_dir.as_ref()))
}

pub(crate) fn clear_regression_selection(app: &mut App, st: &State) {
    let _ = app
        .models_mut()
        .update(&st.regression_selected_summary_path, |v| *v = None);
    let _ = app
        .models_mut()
        .update(&st.regression_selected_summary_json, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_bundle_dirs, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_capability_sources, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_capabilities_checks, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_perf_evidence, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_first_open_evidence, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_share_artifacts, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_error, |v| *v = None);
}

pub(crate) fn clear_regression_artifacts(app: &mut App, st: &State) {
    let _ = app
        .models_mut()
        .update(&st.regression_summary_json, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_index_json, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_dashboard_human, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_loaded_dir, |v| *v = None);
    let _ = app
        .models_mut()
        .update(&st.regression_last_error, |v| *v = None);
    clear_regression_selection(app, st);
}

pub(crate) fn refresh_regression_artifacts(app: &mut App, st: &mut State) {
    let Some(root) = regression_artifacts_root(app, st) else {
        clear_regression_artifacts(app, st);
        return;
    };

    let summary_path = root.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let index_path = root.join(DIAG_REGRESSION_INDEX_FILENAME_V1);
    let summary_json = std::fs::read_to_string(&summary_path).ok();
    let index_json = std::fs::read_to_string(&index_path).ok();

    let dashboard_human = index_json
        .as_deref()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .map(|payload| build_regression_dashboard_human(&index_path, &payload, 5))
        .unwrap_or_default();

    let error = match (summary_json.is_some(), index_json.is_some()) {
        (false, false) => Some(Arc::<str>::from(format!(
            "no regression artifacts found under {}",
            root.display()
        ))),
        (true, false) => Some(Arc::<str>::from(format!(
            "{} is present but {} is missing under {}",
            DIAG_REGRESSION_SUMMARY_FILENAME_V1,
            DIAG_REGRESSION_INDEX_FILENAME_V1,
            root.display()
        ))),
        _ => None,
    };

    let _ = app.models_mut().update(&st.regression_summary_json, |v| {
        *v = summary_json.unwrap_or_default();
    });
    let _ = app.models_mut().update(&st.regression_index_json, |v| {
        *v = index_json.unwrap_or_default();
    });
    let _ = app
        .models_mut()
        .update(&st.regression_dashboard_human, |v| {
            *v = dashboard_human;
        });
    let _ = app.models_mut().update(&st.regression_loaded_dir, |v| {
        *v = Some(Arc::<str>::from(root.to_string_lossy().to_string()));
    });
    let _ = app.models_mut().update(&st.regression_last_error, |v| {
        *v = error;
    });
    reload_selected_regression_summary(app, st);
}

#[derive(Debug, Clone)]
struct RegressionFailingSummaryRow {
    path: String,
    lane: String,
    failures: u64,
    items_total: u64,
}

#[derive(Debug, Clone)]
struct RegressionSummaryDrilldownData {
    summary_json: String,
    bundle_dirs: Vec<String>,
    capability_sources: Vec<String>,
    capabilities_check_paths: Vec<String>,
    perf_evidence_lines: Vec<String>,
    first_open_evidence_lines: Vec<String>,
    share_artifacts: Vec<String>,
}

fn regression_failing_summary_rows(
    index_json: &str,
    top: usize,
) -> Vec<RegressionFailingSummaryRow> {
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(index_json) else {
        return Vec::new();
    };
    dashboard_failing_summary_entries(&payload, top)
        .into_iter()
        .filter(|row| !row.path.trim().is_empty())
        .map(|row| RegressionFailingSummaryRow {
            path: row.path,
            lane: row.lane,
            failures: row.failures,
            items_total: row.items_total,
        })
        .collect()
}

fn load_regression_summary_drilldown(
    summary_path: &Path,
) -> Result<RegressionSummaryDrilldownData, String> {
    let summary_json = std::fs::read_to_string(summary_path).map_err(|e| e.to_string())?;
    let summary: RegressionSummaryV1 =
        serde_json::from_str(&summary_json).map_err(|e| e.to_string())?;
    let drilldown = regression_summary_drilldown(&summary);
    Ok(RegressionSummaryDrilldownData {
        summary_json,
        bundle_dirs: drilldown.bundle_dirs,
        capability_sources: drilldown.capability_sources,
        capabilities_check_paths: drilldown.capabilities_check_paths,
        perf_evidence_lines: drilldown.perf_evidence_lines,
        first_open_evidence_lines: drilldown.first_open_evidence_lines,
        share_artifacts: drilldown.share_artifacts,
    })
}

pub(crate) fn reload_selected_regression_summary(app: &mut App, st: &State) {
    let Some(path) = app
        .models()
        .read(&st.regression_selected_summary_path, |v| v.clone())
        .ok()
        .flatten()
    else {
        return;
    };
    let path_text = path.to_string();
    if let Err(err) = load_regression_summary_selection(app, st, Path::new(&path_text)) {
        set_regression_summary_selection_error(app, st, &path_text, &err);
    }
}

fn load_regression_summary_selection(app: &mut App, st: &State, path: &Path) -> Result<(), String> {
    let data = load_regression_summary_drilldown(path)?;
    let selected_path = path.to_string_lossy().to_string();
    let _ = app
        .models_mut()
        .update(&st.regression_selected_summary_path, |v| {
            *v = Some(Arc::<str>::from(selected_path))
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_summary_json, |v| {
            *v = data.summary_json
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_bundle_dirs, |v| {
            *v = data.bundle_dirs.into_iter().map(Arc::<str>::from).collect();
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_capability_sources, |v| {
            *v = data
                .capability_sources
                .into_iter()
                .map(Arc::<str>::from)
                .collect();
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_capabilities_checks, |v| {
            *v = data
                .capabilities_check_paths
                .into_iter()
                .map(Arc::<str>::from)
                .collect();
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_perf_evidence, |v| {
            *v = data
                .perf_evidence_lines
                .into_iter()
                .map(Arc::<str>::from)
                .collect();
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_first_open_evidence, |v| {
            *v = data
                .first_open_evidence_lines
                .into_iter()
                .map(Arc::<str>::from)
                .collect();
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_share_artifacts, |v| {
            *v = data
                .share_artifacts
                .into_iter()
                .map(Arc::<str>::from)
                .collect();
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_error, |v| *v = None);
    Ok(())
}

fn set_regression_summary_selection_error(app: &mut App, st: &State, path: &str, err: &str) {
    let _ = app
        .models_mut()
        .update(&st.regression_selected_summary_path, |v| {
            *v = Some(Arc::<str>::from(path.to_string()))
        });
    let _ = app
        .models_mut()
        .update(&st.regression_selected_summary_json, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_bundle_dirs, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_capability_sources, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_capabilities_checks, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_perf_evidence, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_first_open_evidence, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.regression_selected_share_artifacts, |v| v.clear());
    let _ = app.models_mut().update(&st.regression_selected_error, |v| {
        *v = Some(Arc::<str>::from(format!(
            "failed to load selected regression summary {path}: {err}",
        )))
    });
}

fn build_regression_dashboard_human(
    index_path: &Path,
    payload: &serde_json::Value,
    top: usize,
) -> String {
    let projection = project_dashboard_summary(payload, top);
    dashboard_human_lines_from_projection(index_path, &projection).join("\n")
}

fn devtools_workflow_commands_from_state(
    app: &App,
    st: &State,
) -> Vec<workflow_run::DevtoolsWorkflowRunCommandV1> {
    let selected_session_id = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    devtools_workflow_commands(
        st.cfg.fs_out_dir.as_ref(),
        st.cfg.ws_url.as_ref(),
        st.cfg.token.as_ref(),
        selected_session_id.as_deref(),
    )
}

fn resolve_repo_or_abs_path(repo_root: &Path, raw: &str) -> PathBuf {
    if is_abs_path(raw) {
        PathBuf::from(raw)
    } else {
        repo_root.join(raw)
    }
}

fn is_abs_path(s: &str) -> bool {
    if s.starts_with('/') || s.starts_with('\\') {
        return true;
    }
    let bytes = s.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn repo_root_from_script_paths(paths: &script_studio::ScriptPaths) -> PathBuf {
    paths
        .workspace_tools_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn push_log(app: &mut App, model: &Model<Vec<Arc<str>>>, line: &str) {
    let line = Arc::<str>::from(line);
    let _ = app.models_mut().update(model, |v| {
        v.push(line);
        if v.len() > 2000 {
            let drain = v.len().saturating_sub(2000);
            v.drain(0..drain);
        }
    });
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

fn env_transport_kind(key: &str) -> Option<DiagTransportKind> {
    let v = std::env::var(key).ok()?;
    let v = v.trim().to_lowercase();
    match v.as_str() {
        "ws" | "websocket" => Some(DiagTransportKind::WebSocket),
        "fs" | "filesystem" => Some(DiagTransportKind::FileSystem),
        _ => None,
    }
}

#[cfg(test)]
#[path = "native/tests.rs"]
mod tests;
