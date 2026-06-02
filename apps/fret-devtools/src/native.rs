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
    devtools_gate_profile_lines, devtools_gate_profiles_v1,
    devtools_gate_resource_footprint_threshold_command, devtools_gate_script_target_command,
    devtools_gate_script_target_profile_ids_v1,
};
use fret_diag::regression_summary::{
    DIAG_REGRESSION_INDEX_FILENAME_V1, DIAG_REGRESSION_SUMMARY_FILENAME_V1,
    RegressionBundleFollowupCommandV1, RegressionSummaryV1, regression_bundle_followup_command_lines,
    regression_bundle_followup_commands, regression_summary_drilldown,
};
use fret_diag::transport::{
    ClientKindV1, DevtoolsWsClientConfig, DiagTransportKind, FsDiagTransportConfig,
    ToolingDiagClient, WsDiagTransportConfig,
};
use fret_diag::{
    dashboard_failing_summary_entries, dashboard_human_lines_from_projection,
    project_dashboard_summary,
};
use fret_diag_protocol::{
    DevtoolsSessionDescriptorV1, UiActionScriptV1, UiActionScriptV2, UiInspectFocusV1,
    UiInspectHoverV1, UiOverlaySummaryV1, UiRectV1, UiScriptStageV1,
};
use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, VirtualListOptions};
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::ScrollStrategy;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

mod demo_metrics_debug;
mod followup;
mod gate_run;
mod pack;
#[path = "native/recent_evidence.rs"]
mod recent_evidence;
mod script_studio;
mod semantics;
mod summarize;
mod workflow_run;
mod ws;

use demo_metrics_debug::{
    demo_metrics_debug_action_command_for_copy_command, demo_metrics_debug_action_command_text,
    devtools_demo_metrics_debug_panel,
};
use recent_evidence::{
    RecentEvidenceRerunCommand, RecentEvidenceTarget,
    devtools_recent_evidence_lines_with_workflow_commands,
    devtools_recent_evidence_selection_effect, devtools_recent_failed_evidence_target,
    recent_evidence_failing_count, recent_evidence_next_action,
    recent_failed_evidence_bundle_dir,
    recent_failed_evidence_rerun_command_from_state,
    recent_failed_evidence_rerun_unavailable_reason_from_state,
};
#[cfg(test)]
use recent_evidence::{
    devtools_recent_evidence_lines, recent_evidence_status_failed,
    recent_failed_evidence_rerun_line,
};
#[cfg(test)]
use demo_metrics_debug::{
    demo_metrics_debug_action_copy_command_lines, demo_metrics_debug_action_metadata_lines,
    demo_metrics_debug_action_readiness_lines, demo_metrics_debug_workflow_artifact_action_lines,
    demo_metrics_debug_workflow_readiness_lines, demo_metrics_debug_workflow_result_action_lines,
    demo_metrics_debug_workflow_status_lines, devtools_demo_metrics_debug_lines,
    devtools_demo_metrics_debug_lines_with_state,
};

const CMD_COPY_WS_URL: &str = "fret.devtools.copy_ws_url";
const CMD_COPY_TOKEN: &str = "fret.devtools.copy_token";
const CMD_INSPECT_ENABLE: &str = "fret.devtools.inspect_enable";
const CMD_INSPECT_DISABLE: &str = "fret.devtools.inspect_disable";
const CMD_PICK_ARM: &str = "fret.devtools.pick_arm";
const CMD_BUNDLE_DUMP: &str = "fret.devtools.bundle_dump";
const CMD_SCREENSHOT_REQUEST: &str = "fret.devtools.screenshot_request";
const CMD_SCRIPT_PUSH: &str = "fret.devtools.script_push";
const CMD_SCRIPT_RUN: &str = "fret.devtools.script_run";
const CMD_SCRIPT_RUN_AND_PACK: &str = "fret.devtools.script_run_and_pack";
const CMD_SCRIPTS_REFRESH: &str = "fret.devtools.scripts.refresh";
const CMD_SCRIPT_FORK: &str = "fret.devtools.script.fork";
const CMD_SCRIPT_SAVE: &str = "fret.devtools.script.save";
const CMD_SCRIPT_APPLY_PICK: &str = "fret.devtools.script.apply_pick";
const CMD_PACK_LAST_BUNDLE: &str = "fret.devtools.pack_last_bundle";
const CMD_COPY_PACK_PATH: &str = "fret.devtools.copy_pack_path";
const CMD_OPEN_VIEWER_URL: &str = "fret.devtools.open_viewer_url";
const CMD_REGRESSION_REFRESH: &str = "fret.devtools.regression.refresh";
const CMD_REGRESSION_SUMMARIZE: &str = "fret.devtools.regression.summarize";
const CMD_REGRESSION_PACK_SELECTED_BUNDLE: &str = "fret.devtools.regression.pack_selected_bundle";
const CMD_REGRESSION_RUN_FOLLOWUP_STATS: &str = "fret.devtools.regression.followup.stats";
const CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF: &str =
    "fret.devtools.regression.followup.layout_perf";
const CMD_REGRESSION_RUN_FOLLOWUP_MEMORY: &str = "fret.devtools.regression.followup.memory";
const CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE: &str = "fret.devtools.regression.followup.triage";
const CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS: &str = "fret.devtools.regression.followup.hotspots";
const CMD_REGRESSION_RUN_FOLLOWUP_TRACE: &str = "fret.devtools.regression.followup.trace";
const CMD_REGRESSION_RUN_FOLLOWUP_COMMAND: &str =
    "fret.devtools.regression.followup.run_command";
const CMD_REGRESSION_RUN_VISUAL_COMPARE: &str = "fret.devtools.regression.followup.visual_compare";
const CMD_REGRESSION_RUN_FOOTPRINT_COMPARE: &str =
    "fret.devtools.regression.followup.footprint_compare";
const CMD_COPY_FOLLOWUP_RESULT_PATH: &str = "fret.devtools.regression.followup.copy_result_path";
const CMD_COPY_FOLLOWUP_RESULT_JSON: &str = "fret.devtools.regression.followup.copy_result_json";
const CMD_COPY_FOLLOWUP_RESULT_COMMAND: &str =
    "fret.devtools.regression.followup.copy_result_command";
const CMD_OPEN_FOLLOWUP_RESULT_JSON: &str = "fret.devtools.regression.followup.open_result_json";
const CMD_COPY_FOLLOWUP_TRACE_ARTIFACT_PATH: &str =
    "fret.devtools.regression.followup.copy_trace_artifact_path";
const CMD_OPEN_FOLLOWUP_TRACE_ARTIFACT: &str =
    "fret.devtools.regression.followup.open_trace_artifact";
const CMD_GATE_RUN_GENERATED: &str = "fret.devtools.gate.run_generated";
const CMD_COPY_GATE_RESULT_PATH: &str = "fret.devtools.gate.copy_result_path";
const CMD_COPY_GATE_RESULT_JSON: &str = "fret.devtools.gate.copy_result_json";
const CMD_COPY_GATE_RESULT_COMMAND: &str = "fret.devtools.gate.copy_result_command";
const CMD_OPEN_GATE_RESULT_JSON: &str = "fret.devtools.gate.open_result_json";
const CMD_WORKFLOW_RUN_SELECTED: &str = "fret.devtools.workflow.run_selected";
const CMD_COPY_RECENT_EVIDENCE_REPORT: &str = "fret.devtools.recent_evidence.copy_report";
const CMD_SELECT_RECENT_FAILED_EVIDENCE: &str =
    "fret.devtools.recent_evidence.select_failed";
const CMD_RERUN_RECENT_FAILED_EVIDENCE: &str =
    "fret.devtools.recent_evidence.rerun_failed";
const CMD_COPY_RECENT_FAILED_EVIDENCE_PATH: &str =
    "fret.devtools.recent_evidence.copy_failed_path";
const CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR: &str =
    "fret.devtools.recent_evidence.copy_failed_bundle_dir";
const CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND: &str =
    "fret.devtools.recent_evidence.copy_failed_command";
const CMD_COPY_RECENT_FAILED_EVIDENCE_JSON: &str =
    "fret.devtools.recent_evidence.copy_failed_json";
const CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON: &str =
    "fret.devtools.recent_evidence.open_failed_json";
const CMD_COPY_WORKFLOW_RESULT_PATH: &str = "fret.devtools.workflow.copy_result_path";
const CMD_COPY_WORKFLOW_RESULT_JSON: &str = "fret.devtools.workflow.copy_result_json";
const CMD_COPY_WORKFLOW_RESULT_COMMAND: &str = "fret.devtools.workflow.copy_result_command";
const CMD_OPEN_WORKFLOW_RESULT_JSON: &str = "fret.devtools.workflow.open_result_json";
const CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH: &str =
    "fret.devtools.workflow.copy_suite_summary_path";
const CMD_OPEN_WORKFLOW_SUITE_SUMMARY: &str = "fret.devtools.workflow.open_suite_summary";
const CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH: &str =
    "fret.devtools.workflow.copy_regression_summary_path";
const CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY: &str =
    "fret.devtools.workflow.open_regression_summary";
const CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH: &str =
    "fret.devtools.workflow.copy_regression_index_path";
const CMD_OPEN_WORKFLOW_REGRESSION_INDEX: &str = "fret.devtools.workflow.open_regression_index";
const CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY: &str =
    "fret.devtools.workflow.load_regression_summary";
const CMD_LOAD_WORKFLOW_REGRESSION_INDEX: &str =
    "fret.devtools.workflow.load_regression_index";
const CMD_COPY_WORKFLOW_SUMMARIZE_COMMAND: &str =
    "fret.devtools.workflow.copy_summarize_command";
const CMD_RUN_WORKFLOW_SUMMARIZE: &str = "fret.devtools.workflow.run_summarize";

const DEVTOOLS_FIRST_OPEN_DOC: &str = "docs/diagnostics-first-open.md";
const DEVTOOLS_GUI_BRANCH_DOC: &str =
    "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md";
const DEVTOOLS_REPO_PREFLIGHT_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag doctor campaigns";
const DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag doctor campaigns --json";
const DEVTOOLS_FIRST_OPEN_GATE_COMMAND: &str =
    "python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke";
const DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID: &str = "devtools-first-open-smoke";
const DEVTOOLS_DOGFOOD_WORKFLOW_ID: &str = "ui-gallery-button-dogfood";
const DEVTOOLS_DOGFOOD_TARGET_COMMAND: &str = "cargo run -p fret-ui-gallery --release";
const DEVTOOLS_DOGFOOD_BASE_SCRIPT: &str = "tools/diag-scripts/ui-gallery-lite-smoke.json";
const DEVTOOLS_DOGFOOD_BUTTON_SCRIPT: &str =
    "tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json";
const DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag pick-script --pick-script-out target/fret-diag/picked.script.json";
const DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag pick-apply tools/diag-scripts/ui-gallery-lite-smoke.json --ptr /steps/12/target --out target/fret-diag/ui-gallery-picked.script.json";
const DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag run target/fret-diag/ui-gallery-picked.script.json --pack --include-all --pack-schema2-only --launch -- cargo run -p fret-ui-gallery --release";
const DEVTOOLS_DOGFOOD_PACK_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag pack <bundle-dir> --include-all --pack-schema2-only";
const DEVTOOLS_DOGFOOD_VIEWER_COMMAND: &str = "pnpm -C tools/fret-bundle-viewer dev";
const IMUI_PRODUCT_WORKFLOW_ID: &str = "imui-product-chain";
const IMUI_PRODUCT_WORKFLOW_DOC: &str =
    "docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md";
const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py";
const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =
    "python tools/diag_gate_imui_product_chain.py --only discovery";
const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =
    "python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release";
const IMUI_PRODUCT_WORKFLOW_SUITE: &str =
    "tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json";
const DEVTOOLS_WORKFLOW_ROUTE_ID: &str = "workflow-runs";
const DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID: &str = "campaign-validate-devtools-first-open";
const DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID: &str = "campaign-validate-imui-p3-multiwindow";
const DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID: &str = "perf-docking-suite-ws";
const DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST: &str =
    "tools/diag-campaigns/devtools-first-open-smoke.json";
const DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST: &str =
    "tools/diag-campaigns/imui-p3-multiwindow-parity.json";
const DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE: &str = "perf-docking-arbitration-steady";
const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] = &[
    "perf-docking/regression.summary.json",
    "perf-docking/check.perf_thresholds.json",
    "perf-docking/*/trace.chrome.json",
];
const DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID: &str = "demo-metrics-debug";
const DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND: &str =
    "cargo run -p fret-demo --bin imui_editor_workbench_demo";
const DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND: &str =
    "cargo run -p fret-demo --bin imui_editor_proof_demo";
const DEVTOOLS_DEMO_EDITOR_NOTES_COMMAND: &str =
    "cargo run -p fret-demo --bin editor_notes_demo";
const DEVTOOLS_DEMO_DEVICE_SHELL_COMMAND: &str =
    "cargo run -p fret-demo --bin editor_notes_device_shell_demo";
const DEVTOOLS_METRICS_STATS_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json";
const DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json";
const DEVTOOLS_METRICS_MEMORY_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json";
const DEVTOOLS_DEBUG_TRIAGE_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json";
const DEVTOOLS_DEBUG_HOTSPOTS_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json";
const DEVTOOLS_DEBUG_TRACE_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json";
const DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC: &str =
    "docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json";
const DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC: &str =
    "docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/WORKSTREAM.json";
const DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =
    "docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json";
const DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str =
    "docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md";
const DEVTOOLS_DOCKING_ARBITRATION_COMMAND: &str =
    "cargo run -p fret-demo --bin docking_arbitration_demo";
const DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str = "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json";
const DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND: &str =
    "python tools/diag_gate_docking_wayland_policy_skip.py";
const CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS: &str =
    "fret.devtools.demo_metrics_debug.copy_actions";
const CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW: &str =
    "fret.devtools.demo_metrics_debug.run_docking_workflow";
const CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW: &str =
    "fret.devtools.demo_metrics_debug.run_perf_workflow";

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
    let scripts_count = cx
        .app
        .models()
        .read(&st.script_library, |scripts| scripts.len())
        .unwrap_or(0);
    let regression_loaded = cx
        .app
        .models()
        .read(&st.regression_loaded_dir, |dir| dir.is_some())
        .unwrap_or(false);
    let regression_selected_summary_loaded = cx
        .app
        .models()
        .read(&st.regression_selected_summary_json, |value| !value.trim().is_empty())
        .unwrap_or(false);
    let selected_followup_result_loaded = selected_followup_result_loaded_from_state(cx.app, st);
    let regression_failing_count = cx
        .app
        .models()
        .read(&st.regression_index_json, |index_json| {
            regression_failing_summary_rows(index_json, 10).len()
        })
        .unwrap_or(0);
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
    let gate_run_result_history = cx
        .app
        .models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_result_history = cx
        .app
        .models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let followup_result_history = cx
        .app
        .models()
        .read(&st.followup_result_history, |v| v.clone())
        .unwrap_or_default();
    let recent_failed_evidence_target = devtools_recent_failed_evidence_target(
        &gate_run_result_history,
        &workflow_run_result_history,
        &followup_result_history,
    );
    let recent_workflow_commands = devtools_workflow_commands_from_state(cx.app, st);
    let recent_failed_evidence_rerunnable_kind = recent_failed_evidence_target
        .as_ref()
        .and_then(|target| {
            recent_failed_evidence_rerun_command_from_state(target, &recent_workflow_commands)
        })
        .map(|command| command.kind());
    let recent_failed_evidence_rerun_reason =
        recent_failed_evidence_target.as_ref().and_then(|target| {
            recent_failed_evidence_rerun_unavailable_reason_from_state(
                target,
                &recent_workflow_commands,
            )
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
    let first_open_recent_evidence_actions = first_open_recent_evidence_action_specs(
        recent_failed_evidence_target.is_some(),
        recent_failed_evidence_rerunnable_kind.is_some(),
    );

    let mut next_action_rows = Vec::new();
    for line in devtools_first_open_next_action_lines(
        has_session,
        session_count,
        selected_session.as_deref(),
        scripts_count,
        regression_loaded,
        regression_selected_summary_loaded,
        selected_followup_result_loaded,
        regression_failing_count,
        st.cfg.fs_out_dir.as_ref(),
        recent_failed_evidence_target.as_ref(),
        recent_failed_evidence_rerunnable_kind,
        recent_failed_evidence_rerun_reason.as_deref(),
        &recent_evidence_next,
    ) {
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

fn diag_card(
    cx: &mut ElementContext<'_, App>,
    title: impl Into<String>,
    description: impl Into<String>,
    content: Vec<AnyElement>,
) -> AnyElement {
    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new(title.into()).into_element(cx),
            shadcn::CardDescription::new(description.into()).into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(content).into_element(cx),
    ])
    .into_element(cx)
}

fn diag_section(
    cx: &mut ElementContext<'_, App>,
    title: impl Into<String>,
    description: impl Into<String>,
    content: Vec<AnyElement>,
) -> AnyElement {
    let theme = cx.theme_snapshot();
    let block = ui::v_stack(|cx| {
        [
            cx.text(title.into()),
            cx.text(description.into()),
            ui::v_stack(|_cx| content)
                .gap(fret_ui_kit::Space::N2)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    cx.container(
        fret_ui_kit::declarative::style::container_props(
            &theme,
            fret_ui_kit::ChromeRefinement::default()
                .bg(fret_ui_kit::ColorRef::Color(theme.color_token("muted")))
                .border_1()
                .border_color(fret_ui_kit::ColorRef::Color(theme.color_token("border")))
                .px(fret_ui_kit::Space::N3)
                .py(fret_ui_kit::Space::N3),
            fret_ui_kit::LayoutRefinement::default().w_full(),
        ),
        |_cx| [block],
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FirstOpenRecentEvidenceActionSpec {
    label: &'static str,
    command: &'static str,
    disabled: bool,
}

fn first_open_recent_evidence_action_specs(
    has_failed_evidence: bool,
    failed_evidence_rerunnable: bool,
) -> Vec<FirstOpenRecentEvidenceActionSpec> {
    vec![
        FirstOpenRecentEvidenceActionSpec {
            label: "Copy recent evidence report",
            command: CMD_COPY_RECENT_EVIDENCE_REPORT,
            disabled: false,
        },
        FirstOpenRecentEvidenceActionSpec {
            label: "Select failed evidence",
            command: CMD_SELECT_RECENT_FAILED_EVIDENCE,
            disabled: !has_failed_evidence,
        },
        FirstOpenRecentEvidenceActionSpec {
            label: "Rerun failed evidence",
            command: CMD_RERUN_RECENT_FAILED_EVIDENCE,
            disabled: !failed_evidence_rerunnable,
        },
    ]
}

fn first_open_recent_evidence_action_row(
    cx: &mut ElementContext<'_, App>,
    specs: &[FirstOpenRecentEvidenceActionSpec],
) -> AnyElement {
    let actions = specs
        .iter()
        .map(|spec| {
            shadcn::Button::new(spec.label)
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(spec.disabled)
                .on_click(spec.command)
                .into_element(cx)
        })
        .collect::<Vec<_>>();
    ui::h_row(|_cx| actions)
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
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

#[derive(Debug, Clone, Copy)]
enum InspectTreeMode {
    Semantics,
    Layout,
    Elements,
}

impl InspectTreeMode {
    fn search_label(self) -> &'static str {
        match self {
            Self::Semantics => "Semantics search",
            Self::Layout => "Layout tree search",
            Self::Elements => "Element tree search",
        }
    }

    fn search_placeholder(self) -> &'static str {
        match self {
            Self::Semantics => "Search role/test_id/label/value...",
            Self::Layout => "Search role/test_id/bounds/parent...",
            Self::Elements => "Search role/test_id/id/relationships...",
        }
    }

    fn empty_text(self) -> &'static str {
        match self {
            Self::Semantics => "No semantics yet. Use 'Dump Bundle' or run a script that dumps a bundle.",
            Self::Layout => {
                "No layout-bounds tree yet. Use 'Dump Bundle' or run a script that dumps semantics bounds."
            }
            Self::Elements => {
                "No element-identity tree yet. Use 'Dump Bundle' or run a script that dumps semantics identity."
            }
        }
    }

    fn error_prefix(self) -> &'static str {
        match self {
            Self::Semantics => "semantics error",
            Self::Layout => "layout tree error",
            Self::Elements => "element tree error",
        }
    }

    fn stats_prefix(self) -> &'static str {
        match self {
            Self::Semantics => "semantics",
            Self::Layout => "layout-derived",
            Self::Elements => "element-derived",
        }
    }

    fn cache_discriminant(self) -> u8 {
        match self {
            Self::Semantics => 0,
            Self::Layout => 1,
            Self::Elements => 2,
        }
    }
}

fn semantics_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    diagnostics_tree_panel(cx, st, InspectTreeMode::Semantics)
}

fn layout_tree_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    diagnostics_tree_panel(cx, st, InspectTreeMode::Layout)
}

fn element_tree_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    diagnostics_tree_panel(cx, st, InspectTreeMode::Elements)
}

fn diagnostics_tree_panel(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    mode: InspectTreeMode,
) -> AnyElement {
    let index = cx
        .app
        .models()
        .read(&st.semantics_cache, |v| v.clone())
        .ok()
        .flatten();
    let error = cx
        .app
        .models()
        .read(&st.semantics_error, |v| v.clone())
        .ok()
        .flatten();
    let search = cx
        .app
        .models()
        .read(&st.semantics_search, |v| v.clone())
        .unwrap_or_default();
    let expanded = cx
        .app
        .models()
        .read(&st.semantics_expanded, |v| v.clone())
        .unwrap_or_default();
    let selected_id = cx
        .app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let source_hash = cx
        .app
        .models()
        .read(&st.semantics_source_hash, |v| *v)
        .ok()
        .flatten()
        .unwrap_or(0);

    let search_input = shadcn::Input::new(st.semantics_search.clone())
        .a11y_label(mode.search_label())
        .placeholder(mode.search_placeholder())
        .into_element(cx);

    let header = ui::h_row(|_cx| [search_input])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .into_element(cx);

    let content: AnyElement = match (index, error) {
        (_index, Some(err)) => cx.text(format!("{}: {err}", mode.error_prefix())),
        (None, None) => cx.text(mode.empty_text()),
        (Some(index), None) => {
            #[derive(Debug, Default)]
            struct RowsCache {
                key: u64,
                rows: Arc<Vec<semantics::SemanticsRow>>,
            }

            #[derive(Debug, Default)]
            struct SelectionScrollSync {
                last: Option<(u64, u64)>,
            }

            let rows_key = {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                mode.cache_discriminant().hash(&mut hasher);
                source_hash.hash(&mut hasher);
                search.trim().to_lowercase().hash(&mut hasher);
                let mut expanded_sorted: Vec<u64> = expanded.iter().copied().collect();
                expanded_sorted.sort_unstable();
                expanded_sorted.hash(&mut hasher);
                hasher.finish()
            };

            let rows = cx.slot_state(RowsCache::default, |cache| {
                if cache.key != rows_key {
                    let next = semantics::compute_rows(&index, &expanded, &search);
                    cache.key = rows_key;
                    cache.rows = Arc::new(next);
                }
                Arc::clone(&cache.rows)
            });

            let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());

            if let Some(sel) = selected_id {
                let rows_for_scroll = Arc::clone(&rows);
                let handle_for_scroll = scroll_handle.clone();
                cx.slot_state(SelectionScrollSync::default, |sync| {
                    let next = (rows_key, sel);
                    if sync.last == Some(next) {
                        return;
                    }
                    sync.last = Some(next);

                    if let Some(idx) = rows_for_scroll.iter().position(|r| r.id == sel) {
                        handle_for_scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
                    }
                });
            } else {
                cx.slot_state(SelectionScrollSync::default, |sync| sync.last = None);
            }

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;
            layout.flex.grow = 1.0;

            let mut options = VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16);
            options.items_revision = rows_key;

            let stats = cx.text(format!(
                "{} window={} roots={} nodes={} rows={}",
                mode.stats_prefix(),
                index.window,
                index.roots.len(),
                index.nodes_by_id.len(),
                rows.len()
            ));

            let rows_for_key = Arc::clone(&rows);
            let rows_for_row = Arc::clone(&rows);
            let index_for_list = Arc::clone(&index);
            let selected_id_for_list = selected_id;
            let has_search = !search.trim().is_empty();

            let list = cx.virtual_list_keyed_with_layout(
                layout,
                rows_for_key.len(),
                options,
                &scroll_handle,
                |i| rows_for_key[i].id,
                move |cx, i| {
                    let row = &rows_for_row[i];
                    let id = row.id;

                    let variant = if selected_id_for_list == Some(id) {
                        shadcn::ButtonVariant::Secondary
                    } else {
                        shadcn::ButtonVariant::Ghost
                    };

                    let toggle: AnyElement = if row.has_children {
                        let glyph = if row.is_expanded { "v" } else { ">" };
                        if has_search {
                            cx.text(glyph.to_string())
                        } else {
                            let expanded_model = st.semantics_expanded.clone();
                            let on_toggle: fret_ui::action::OnActivate =
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host.models_mut().update(&expanded_model, |set| {
                                        if set.contains(&id) {
                                            set.remove(&id);
                                        } else {
                                            set.insert(id);
                                        }
                                    });
                                    host.request_redraw(action_cx.window);
                                });
                            shadcn::Button::new(glyph)
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(on_toggle)
                                .into_element(cx)
                        }
                    } else {
                        cx.text(" ")
                    };

                    let label = index_for_list
                        .node(id)
                        .map(|node| match mode {
                            InspectTreeMode::Semantics => semantics::node_label(node),
                            InspectTreeMode::Layout => semantics::layout_node_label(node),
                            InspectTreeMode::Elements => semantics::element_node_label(node),
                        })
                        .unwrap_or_else(|| format!("<missing semantics node id={id}>"));

                    let selected_id_model = st.semantics_selected_id.clone();
                    let selected_json_model = st.semantics_selected_node_json.clone();
                    let selected_live_json_model = st.semantics_selected_node_live_json.clone();
                    let selected_live_status_model = st.semantics_selected_node_live_status.clone();
                    let selected_live_updated_model =
                        st.semantics_selected_node_live_updated_unix_ms.clone();
                    let selected_live_children_model =
                        st.semantics_selected_node_live_children.clone();
                    let selected_hit_test_explain_json_model =
                        st.semantics_selected_hit_test_explain_json.clone();
                    let selected_hit_test_explain_summary_model =
                        st.semantics_selected_hit_test_explain_summary.clone();
                    let selected_hit_test_explain_status_model =
                        st.semantics_selected_hit_test_explain_status.clone();
                    let selected_hit_test_explain_updated_model = st
                        .semantics_selected_hit_test_explain_updated_unix_ms
                        .clone();
                    let index_for_select = Arc::clone(&index_for_list);
                    let on_select: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&selected_id_model, |v| *v = Some(id));
                            let text =
                                semantics::selected_node_json(index_for_select.as_ref(), Some(id));
                            let _ = host
                                .models_mut()
                                .update(&selected_json_model, |v| *v = text);
                            let _ = host
                                .models_mut()
                                .update(&selected_live_json_model, |v| v.clear());
                            let _ = host.models_mut().update(&selected_live_status_model, |v| {
                                *v = None;
                            });
                            let _ = host
                                .models_mut()
                                .update(&selected_live_updated_model, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&selected_live_children_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_hit_test_explain_json_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_hit_test_explain_summary_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_hit_test_explain_status_model, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&selected_hit_test_explain_updated_model, |v| *v = None);
                            host.request_redraw(action_cx.window);
                        });

                    let row_button = shadcn::Button::new(label)
                        .variant(variant)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_select)
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .flex_1()
                                .min_w_0()
                                .ml_px(Px(12.0 * row.depth as f32)),
                        )
                        .into_element(cx);

                    ui::h_row(|_cx| [toggle, row_button])
                        .gap(fret_ui_kit::Space::N1)
                        .items_center()
                        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                        .into_element(cx)
                },
            );

            ui::v_stack(|_cx| [stats, list])
                .gap(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
                .into_element(cx)
        }
    };

    ui::v_stack(|_cx| [header, content])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
}

fn center_panel(
    cx: &mut ElementContext<'_, App>,
    theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let script_text = cx
        .app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    let pick_text = cx
        .app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    let apply_pointer = cx
        .app
        .models()
        .read(&st.script_apply_pointer, |v| v.clone())
        .unwrap_or_default();
    let scripts = cx
        .app
        .models()
        .read(&st.script_library, |v| v.clone())
        .unwrap_or_default();
    let loaded_origin = cx
        .app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    let loaded_path = cx
        .app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten();
    let script_last_stage = cx
        .app
        .models()
        .read(&st.script_last_stage, |v| v.clone())
        .ok()
        .flatten();
    let script_last_step_index = cx
        .app
        .models()
        .read(&st.script_last_step_index, |v| *v)
        .ok()
        .flatten();
    let script_last_reason = cx
        .app
        .models()
        .read(&st.script_last_reason, |v| v.clone())
        .ok()
        .flatten();
    let pack_after_run = cx
        .app
        .models()
        .read(&st.script_pack_after_run, |v| *v)
        .unwrap_or(false);

    let target_out_dir = cx
        .app
        .models()
        .read(&st.target_out_dir, |v| v.clone())
        .ok()
        .flatten();
    let last_bundle_dir_abs = cx
        .app
        .models()
        .read(&st.last_bundle_dir_abs, |v| v.clone())
        .ok()
        .flatten();
    let last_bundle_dump_bundle_json = cx
        .app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.clone())
        .ok()
        .flatten();
    let last_pack_path = cx
        .app
        .models()
        .read(&st.last_pack_path, |v| v.clone())
        .ok()
        .flatten();
    let pack_in_flight = cx
        .app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    let pack_last_error = cx
        .app
        .models()
        .read(&st.pack_last_error, |v| v.clone())
        .ok()
        .flatten();
    let viewer_url = cx
        .app
        .models()
        .read(&st.viewer_url, |v| v.clone())
        .unwrap_or_default();

    let consume_clicks = cx
        .app
        .models()
        .read(&st.inspect_consume_clicks, |v| *v)
        .unwrap_or(false);

    let consume_toggle = shadcn::Checkbox::new(st.inspect_consume_clicks.clone())
        .a11y_label("Consume clicks while inspecting")
        .into_element(cx);

    let has_session = cx
        .app
        .models()
        .read(&st.selected_session_id, |v| v.is_some())
        .unwrap_or(false);

    let can_fork = loaded_origin == Some(script_studio::ScriptOrigin::WorkspaceTools);
    let can_save = loaded_origin == Some(script_studio::ScriptOrigin::UserLocal);
    let can_apply_pick = !pick_text.trim().is_empty() && !apply_pointer.trim().is_empty();
    let can_pack = last_bundle_dir_abs.is_some() || last_bundle_dump_bundle_json.is_some();

    let pointer_input = shadcn::Input::new(st.script_apply_pointer.clone())
        .a11y_label("JSON pointer")
        .placeholder("/steps/0/target")
        .into_element(cx);

    let viewer_url_input = shadcn::Input::new(st.viewer_url.clone())
        .a11y_label("Bundle viewer URL")
        .placeholder("http://localhost:5173")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(st.script_text.clone())
        .a11y_label("Script JSON")
        .min_height(Px(360.0))
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx);

    let (script_summary, script_is_valid) = script_summary_line(&script_text);
    let script_steps = script_steps_len(&script_text).unwrap_or(0);
    let script_schema_version = infer_script_schema_version(&script_text).unwrap_or(1);
    let pack_status_line = {
        let err = pack_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "pack_in_flight={} pack_last_error={err}",
            if pack_in_flight { "true" } else { "false" }
        )
    };

    let primary_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Push Script")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session || !script_is_valid)
                .on_click(CMD_SCRIPT_PUSH)
                .into_element(cx),
            shadcn::Button::new("Run Script")
                .variant(shadcn::ButtonVariant::Default)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session || !script_is_valid)
                .on_click(CMD_SCRIPT_RUN)
                .into_element(cx),
            shadcn::Button::new("Run & Pack")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session || !script_is_valid)
                .on_click(CMD_SCRIPT_RUN_AND_PACK)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let library_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Refresh Scripts")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_SCRIPTS_REFRESH)
                .into_element(cx),
            shadcn::Button::new("Fork")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_fork)
                .on_click(CMD_SCRIPT_FORK)
                .into_element(cx),
            shadcn::Button::new("Save")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_save)
                .on_click(CMD_SCRIPT_SAVE)
                .into_element(cx),
            consume_toggle,
            shadcn::Badge::new(if consume_clicks {
                "Consume clicks on"
            } else {
                "Consume clicks off"
            })
            .variant(if consume_clicks {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let pack_row = ui::h_row(|cx| {
        let copy_enabled = last_pack_path.is_some();
        [
            cx.text("Artifacts:"),
            shadcn::Button::new("Pack last bundle")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_pack || pack_in_flight)
                .on_click(CMD_PACK_LAST_BUNDLE)
                .into_element(cx),
            shadcn::Button::new("Copy pack path")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!copy_enabled)
                .on_click(CMD_COPY_PACK_PATH)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let viewer_row = ui::h_row(|cx| {
        [
            cx.text("Viewer:"),
            viewer_url_input,
            shadcn::Button::new("Open viewer")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(viewer_url.trim().is_empty())
                .on_click(CMD_OPEN_VIEWER_URL)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let apply_row = ui::h_row(|cx| {
        [
            cx.text("Pick-to-fill:"),
            pointer_input,
            shadcn::Button::new("Apply Pick")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_apply_pick)
                .on_click(CMD_SCRIPT_APPLY_PICK)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let out_dir_line = match target_out_dir.as_deref() {
        Some(dir) => format!("Target diag out_dir: {dir}"),
        None => "Target diag out_dir: <unknown>".to_string(),
    };
    let loaded_summary_line = match (loaded_origin, loaded_path.as_deref()) {
        (Some(origin), Some(path)) => format!("Loaded [{}] {}", origin.label(), path),
        _ => "Loaded <none>".to_string(),
    };
    let run_summary_line = {
        let stage = script_last_stage
            .as_ref()
            .map(|s| format!("{s:?}"))
            .unwrap_or_else(|| "None".to_string());
        let step = script_last_step_index
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!("Run status: {stage} | step {step}/{script_steps}")
    };
    let reason_summary_line = script_last_reason
        .as_deref()
        .map(|s| format!("Reason: {s}"))
        .unwrap_or_else(|| "Reason: -".to_string());
    let pack_summary_line = match last_pack_path.as_deref() {
        Some(path) => format!("Pack output: {path}"),
        None => format!("Pack output: <none> | {pack_status_line}"),
    };
    let script_status_badges = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if has_session {
                "Session connected"
            } else {
                "No session"
            })
            .variant(if has_session {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if script_is_valid {
                format!("Schema v{script_schema_version} valid")
            } else {
                format!("Schema v{script_schema_version} invalid")
            })
            .variant(if script_is_valid {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Destructive
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
            shadcn::Badge::new(if pack_after_run {
                "Run&Pack enabled"
            } else {
                "Run-only mode"
            })
            .variant(if pack_after_run {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("Library {}", scripts.len()))
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let mut script_rows: Vec<AnyElement> = Vec::new();
    for item in scripts.iter() {
        let label = format!("[{}] {}", item.origin.label(), item.file_name);
        let is_loaded = loaded_path
            .as_deref()
            .is_some_and(|p| PathBuf::from(p) == item.path);

        let variant = if is_loaded {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        };

        let origin_for_activate = item.origin;
        let path_for_activate = item.path.clone();
        let script_text_for_activate = st.script_text.clone();
        let loaded_origin_for_activate = st.loaded_script_origin.clone();
        let loaded_path_for_activate = st.loaded_script_path.clone();
        let log_lines_for_activate = st.log_lines.clone();

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let text = match std::fs::read_to_string(&path_for_activate) {
                Ok(text) => text,
                Err(err) => {
                    let line = format!("script load failed: {err}");
                    let _ = host.models_mut().update(&log_lines_for_activate, |v| {
                        v.push(Arc::<str>::from(line));
                        if v.len() > 2000 {
                            let drain = v.len().saturating_sub(2000);
                            v.drain(0..drain);
                        }
                    });
                    host.request_redraw(action_cx.window);
                    return;
                }
            };

            let _ = host.models_mut().update(&script_text_for_activate, |v| {
                *v = text;
            });
            let _ = host.models_mut().update(&loaded_origin_for_activate, |v| {
                *v = Some(origin_for_activate)
            });
            let _ = host.models_mut().update(&loaded_path_for_activate, |v| {
                *v = Some(Arc::<str>::from(
                    path_for_activate.to_string_lossy().to_string(),
                ))
            });

            host.request_redraw(action_cx.window);
            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                action_cx.window,
            ));
        });

        script_rows.push(
            shadcn::Button::new(label)
                .variant(variant)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        );
    }

    let scripts_list = shadcn::ScrollArea::new([ui::v_stack(|_cx| script_rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .into_element(cx);

    let pointer_candidates = script_studio::collect_common_json_pointers(&script_text);

    let step_index_input = shadcn::Input::new(st.script_step_insert_index.clone())
        .a11y_label("Step insert index")
        .placeholder("(append)")
        .into_element(cx);

    let mut step_buttons: Vec<AnyElement> = Vec::new();
    for t in step_templates_for_schema(script_schema_version) {
        let script_text_model = st.script_text.clone();
        let insert_index_model = st.script_step_insert_index.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let step_value = t.step.clone();
        let label = t.label;

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let index_text = host
                .models_mut()
                .read(&insert_index_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            let index = index_text.trim().parse::<usize>().ok();

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();

            let len_before = script_steps_len(&current).unwrap_or(0);
            let insert_at = index.unwrap_or(len_before);
            let inserted_index = insert_at.min(len_before);

            let updated = match index {
                Some(i) => script_studio::insert_step_json(&current, i, step_value.clone()),
                None => script_studio::append_step_json(&current, step_value.clone()),
            };

            match updated {
                Ok(text) => {
                    let _ = host.models_mut().update(&script_text_model, |v| *v = text);
                    if let Some(suffix) = primary_pointer_suffix_for_step_json(&step_value) {
                        let ptr = format!("/steps/{inserted_index}{suffix}");
                        let _ = host.models_mut().update(&pointer_model, |v| *v = ptr);
                    }
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!(
                            "insert step failed ({label}): {err}"
                        )));
                        if v.len() > 2000 {
                            let drain = v.len().saturating_sub(2000);
                            v.drain(0..drain);
                        }
                    });
                }
            }

            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });

        step_buttons.push(
            shadcn::Button::new(t.label)
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        );
    }

    let steps_tab = shadcn::ScrollArea::new([ui::v_stack(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        out.push(cx.text(format!("Schema v{script_schema_version} step palette")));
        out.push(step_index_input);
        out.extend(step_buttons);
        if !pointer_candidates.is_empty() {
            out.push(cx.text("Pointer candidates:"));
            for p in pointer_candidates.iter().take(64) {
                let pointer_model = st.script_apply_pointer.clone();
                let p_value = p.clone();
                let p_label = p.clone();
                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host
                            .models_mut()
                            .update(&pointer_model, |v| *v = p_value.clone());
                        host.request_redraw(action_cx.window);
                        host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
                    });
                out.push(
                    shadcn::Button::new(p_label)
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_activate)
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                        .into_element(cx),
                );
            }
        }
        out
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx)])
    .into_element(cx);

    let selector_kind_items = [
        shadcn::SelectItem::new("test_id", "test_id"),
        shadcn::SelectItem::new("role_and_name", "role_and_name"),
        shadcn::SelectItem::new("role_and_path", "role_and_path"),
        shadcn::SelectItem::new("node_id", "node_id"),
        shadcn::SelectItem::new("global_element_id", "global_element_id"),
    ];
    let selector_kind_select = shadcn::Select::new(
        st.script_selector_kind.clone(),
        st.script_selector_kind_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("selector kind"))
    .items(selector_kind_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let selector_kind = cx
        .app
        .models()
        .read(&st.script_selector_kind, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("test_id"));
    let selector_value = selector_value_from_models(cx, st, selector_kind.as_ref());
    let selector_json =
        serde_json::to_string_pretty(&selector_value).unwrap_or_else(|_| "{}".to_string());

    let selector_apply = {
        let script_text_model = st.script_text.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let selector_value = selector_value.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pointer = host
                .models_mut()
                .read(&pointer_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            if pointer.trim().is_empty() {
                let _ = host.models_mut().update(&log_lines, |v| {
                    v.push(Arc::<str>::from(
                        "apply selector refused (empty json pointer)",
                    ));
                });
                host.request_redraw(action_cx.window);
                return;
            }

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            match script_studio::apply_json_value_to_json_pointer(
                &current,
                &pointer,
                selector_value.clone(),
            ) {
                Ok(updated) => {
                    let _ = host
                        .models_mut()
                        .update(&script_text_model, |v| *v = updated);
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!("apply selector failed: {err}")));
                    });
                }
            }
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });
        on_activate
    };

    let selector_copy = {
        let selector_json = selector_json.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let token = host.next_clipboard_token();
            host.push_effect(Effect::ClipboardWriteText {
                window: action_cx.window,
                token,
                text: selector_json.clone(),
            });
            host.request_redraw(action_cx.window);
        });
        on_activate
    };

    let selector_tab = ui::v_stack(|cx| {
        let fields = selector_fields(cx, st, selector_kind.as_ref());
        let preview = text_blob(cx, selector_json.clone());
        [
            selector_kind_select,
            fields,
            ui::h_row(|cx| {
                [
                    shadcn::Button::new("Apply to pointer")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(selector_apply)
                        .into_element(cx),
                    shadcn::Button::new("Copy JSON")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(selector_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx),
            preview,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let predicate_kind_items = [
        shadcn::SelectItem::new("exists", "exists"),
        shadcn::SelectItem::new("not_exists", "not_exists"),
        shadcn::SelectItem::new("focus_is", "focus_is"),
        shadcn::SelectItem::new("role_is", "role_is"),
        shadcn::SelectItem::new("checked_is", "checked_is"),
        shadcn::SelectItem::new("checked_is_none", "checked_is_none"),
        shadcn::SelectItem::new("label_len_is", "label_len_is"),
        shadcn::SelectItem::new("label_len_ge", "label_len_ge"),
        shadcn::SelectItem::new("value_len_is", "value_len_is"),
        shadcn::SelectItem::new("value_len_ge", "value_len_ge"),
        shadcn::SelectItem::new("barrier_roots", "barrier_roots"),
        shadcn::SelectItem::new("visible_in_window", "visible_in_window"),
        shadcn::SelectItem::new("bounds_within_window", "bounds_within_window"),
        shadcn::SelectItem::new("bounds_min_size", "bounds_min_size"),
        shadcn::SelectItem::new("bounds_non_overlapping", "bounds_non_overlapping"),
        shadcn::SelectItem::new("bounds_overlapping", "bounds_overlapping"),
        shadcn::SelectItem::new("bounds_overlapping_x", "bounds_overlapping_x"),
        shadcn::SelectItem::new("bounds_overlapping_y", "bounds_overlapping_y"),
    ];
    let predicate_kind_select = shadcn::Select::new(
        st.script_predicate_kind.clone(),
        st.script_predicate_kind_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("predicate kind"))
    .items(predicate_kind_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let predicate_kind = cx
        .app
        .models()
        .read(&st.script_predicate_kind, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("exists"));
    let predicate_value =
        predicate_value_from_models(cx, st, predicate_kind.as_ref(), selector_value.clone());
    let predicate_json =
        serde_json::to_string_pretty(&predicate_value).unwrap_or_else(|_| "{}".to_string());

    let predicate_apply = {
        let script_text_model = st.script_text.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let predicate_value = predicate_value.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pointer = host
                .models_mut()
                .read(&pointer_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            if pointer.trim().is_empty() {
                let _ = host.models_mut().update(&log_lines, |v| {
                    v.push(Arc::<str>::from(
                        "apply predicate refused (empty json pointer)",
                    ));
                });
                host.request_redraw(action_cx.window);
                return;
            }

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            match script_studio::apply_json_value_to_json_pointer(
                &current,
                &pointer,
                predicate_value.clone(),
            ) {
                Ok(updated) => {
                    let _ = host
                        .models_mut()
                        .update(&script_text_model, |v| *v = updated);
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!("apply predicate failed: {err}")));
                    });
                }
            }
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });
        on_activate
    };

    let predicate_copy = {
        let predicate_json = predicate_json.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let token = host.next_clipboard_token();
            host.push_effect(Effect::ClipboardWriteText {
                window: action_cx.window,
                token,
                text: predicate_json.clone(),
            });
            host.request_redraw(action_cx.window);
        });
        on_activate
    };

    let predicate_tab = ui::v_stack(|cx| {
        let fields = predicate_fields(cx, st, predicate_kind.as_ref());
        let preview = text_blob(cx, predicate_json.clone());
        [
            predicate_kind_select,
            fields,
            ui::h_row(|cx| {
                [
                    shadcn::Button::new("Apply to pointer")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(predicate_apply)
                        .into_element(cx),
                    shadcn::Button::new("Copy JSON")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(predicate_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx),
            preview,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let helpers_tabs = shadcn::Tabs::new(st.script_studio_helper_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("steps", "Steps", [steps_tab]),
            shadcn::TabsItem::new("selector", "Selector", [selector_tab]),
            shadcn::TabsItem::new("predicate", "Predicate", [predicate_tab]),
        ])
        .into_element(cx);

    let validate_summary = cx.text(format!("Validate: {script_summary}"));
    let run_summary = cx.text(run_summary_line);
    let reason_summary = cx.text(reason_summary_line);
    let loaded_summary = cx.text(loaded_summary_line);
    let out_dir_summary = cx.text(out_dir_line);
    let pack_summary = cx.text(pack_summary_line);

    let workflow_controls = diag_card(
        cx,
        "Workflow Controls",
        "Select a script, validate it, and decide whether the next run also produces evidence.",
        vec![
            primary_actions,
            library_actions,
            script_status_badges,
            validate_summary,
            run_summary,
            reason_summary,
        ],
    );

    let workflow_outputs = diag_card(
        cx,
        "Outputs & Bundles",
        "Apply captured picks, package the latest bundle, and hand off to the offline viewer.",
        vec![
            apply_row,
            pack_row,
            viewer_row,
            loaded_summary,
            out_dir_summary,
            pack_summary,
        ],
    );

    let workflow_summary = ui::h_row(|_cx| [workflow_controls, workflow_outputs])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items_start()
        .into_element(cx);

    let scripts_sidebar = diag_card(
        cx,
        "Script Source",
        format!(
            "Workspace tools and local scripts available: {}",
            scripts.len()
        ),
        vec![scripts_list],
    );

    let editor_workspace = diag_card(
        cx,
        "Editor",
        format!("Current script payload size: {} bytes", script_text.len()),
        vec![textarea],
    );

    let helper_workspace = diag_card(
        cx,
        "Helpers",
        "Build reusable steps, selectors, and predicates without leaving the editor flow.",
        vec![helpers_tabs],
    );

    let split = ui::h_row(|cx| {
        [
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(224.0))
                        .h_full(),
                ),
                |_cx| [scripts_sidebar],
            ),
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .flex_1()
                        .min_w_0()
                        .h_full(),
                ),
                |_cx| [editor_workspace],
            ),
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(304.0))
                        .h_full(),
                ),
                |_cx| [helper_workspace],
            ),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
    .items_start()
    .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Script Studio").into_element(cx),
            shadcn::CardDescription::new(
                "A compact workflow for selecting scripts, editing payloads, and packaging evidence.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([workflow_summary, split]).into_element(cx),
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

fn devtools_guide_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let gate_run_result_history = cx
        .app
        .models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_result_history = cx
        .app
        .models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let followup_result_history = cx
        .app
        .models()
        .read(&st.followup_result_history, |v| v.clone())
        .unwrap_or_default();
    let recent_failed_evidence_target = devtools_recent_failed_evidence_target(
        &gate_run_result_history,
        &workflow_run_result_history,
        &followup_result_history,
    );
    let recent_failed_evidence_bundle_dir_available = recent_failed_evidence_target
        .as_ref()
        .and_then(recent_failed_evidence_bundle_dir)
        .is_some();
    let recent_workflow_commands = devtools_workflow_commands_from_state(cx.app, st);
    let recent_failed_evidence_rerunnable = recent_failed_evidence_target
        .as_ref()
        .and_then(|target| {
            recent_failed_evidence_rerun_command_from_state(target, &recent_workflow_commands)
        })
        .is_some();
    let recent_evidence_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Copy recent evidence report")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_COPY_RECENT_EVIDENCE_REPORT)
                .into_element(cx),
            shadcn::Button::new("Select failed evidence")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent_failed_evidence_target.is_none())
                .on_click(CMD_SELECT_RECENT_FAILED_EVIDENCE)
                .into_element(cx),
            shadcn::Button::new("Rerun failed evidence")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!recent_failed_evidence_rerunnable)
                .on_click(CMD_RERUN_RECENT_FAILED_EVIDENCE)
                .into_element(cx),
            shadcn::Button::new("Copy failed evidence path")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent_failed_evidence_target.is_none())
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_PATH)
                .into_element(cx),
            shadcn::Button::new("Copy failed bundle dir")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!recent_failed_evidence_bundle_dir_available)
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR)
                .into_element(cx),
            shadcn::Button::new("Copy failed evidence command")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent_failed_evidence_target.is_none())
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND)
                .into_element(cx),
            shadcn::Button::new("Copy failed evidence JSON")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent_failed_evidence_target.is_none())
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_JSON)
                .into_element(cx),
            shadcn::Button::new("Open failed evidence JSON")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent_failed_evidence_target.is_none())
                .on_click(CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let recent_evidence_blob = text_blob_sized(
        cx,
        devtools_recent_evidence_lines_with_workflow_commands(
            &gate_run_result_history,
            &workflow_run_result_history,
            &followup_result_history,
            &recent_workflow_commands,
        )
        .join("\n"),
        Px(132.0),
    );
    let recent_evidence_panel = diag_section(
        cx,
        "Recent Evidence",
        "Latest GUI-launched gate, workflow, and follow-up artifacts restored from the shared diagnostics histories.",
        vec![recent_evidence_actions, recent_evidence_blob],
    );
    let mut first_open_rows = Vec::new();
    for line in devtools_first_open_lines(st.cfg.fs_out_dir.as_ref()) {
        first_open_rows.push(cx.text(line));
    }
    let first_open_panel = diag_section(
        cx,
        "First-open Evidence Path",
        "Canonical docs, repo preflight, artifact roots, product-chain evidence, and smoke gate stay visible in the GUI shell.",
        first_open_rows,
    );
    let mut dogfood_workflow_rows = Vec::new();
    for line in devtools_dogfood_workflow_lines(st.cfg.fs_out_dir.as_ref()) {
        dogfood_workflow_rows.push(cx.text(line));
    }
    let dogfood_workflow_panel = diag_section(
        cx,
        "Dogfood Workflow",
        "UI gallery selector capture, script patching, run/pack, and offline viewer handoff stay visible from the GUI shell.",
        dogfood_workflow_rows,
    );
    let demo_metrics_debug_panel = devtools_demo_metrics_debug_panel(cx, st);
    let mut workflow_run_rows = Vec::new();
    for line in devtools_workflow_run_lines(st.cfg.fs_out_dir.as_ref()) {
        workflow_run_rows.push(cx.text(line));
    }
    workflow_run_rows.push(devtools_workflow_run_panel(cx, st));
    let workflow_runs_panel = diag_section(
        cx,
        "Workflow Runs",
        "First-class campaign validation and selected-session suite runs reuse the shared diag command path from the GUI shell.",
        workflow_run_rows,
    );
    let mut gate_command_rows = Vec::new();
    for line in devtools_gate_command_lines(st.cfg.fs_out_dir.as_ref()) {
        gate_command_rows.push(cx.text(line));
    }
    gate_command_rows.push(devtools_gate_profile_command_builder(cx, st));
    gate_command_rows.extend(devtools_gate_profile_action_rows(cx));
    let gate_commands_panel = diag_section(
        cx,
        "Gate Commands",
        "First-class stale, pixels, perf-threshold, and resource-footprint gate entrypoints stay visible from the GUI shell.",
        gate_command_rows,
    );

    ui::v_stack(|_cx| {
        [
            recent_evidence_panel,
            first_open_panel,
            dogfood_workflow_panel,
            demo_metrics_debug_panel,
            workflow_runs_panel,
            gate_commands_panel,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn inspect_panel(
    cx: &mut ElementContext<'_, App>,
    hover_json: &str,
    focus_json: &str,
    overlay_json: &str,
) -> AnyElement {
    let hover_bounds = text_blob_sized(
        cx,
        inspect_hover_bounds_lines(hover_json).join("\n"),
        Px(112.0),
    )
    .test_id("devtools.inspect.hover_bounds");
    let overlay_hooks = text_blob_sized(
        cx,
        inspect_overlay_hook_lines(hover_json, focus_json, overlay_json).join("\n"),
        Px(144.0),
    )
    .test_id("devtools.inspect.overlay_hooks");
    let raw_payloads = text_blob_sized(
        cx,
        inspect_raw_payload_text(hover_json, focus_json, overlay_json),
        Px(180.0),
    )
    .test_id("devtools.inspect.raw_payloads");

    let hover_section = diag_section(
        cx,
        "Live Inspect Hover Bounds",
        "Structured hovered-node bounds projected from inspect.hover.",
        vec![hover_bounds],
    );
    let overlay_section = diag_section(
        cx,
        "Live Inspect Overlay Hooks",
        "Viewport overlay hooks and overlay.summary root hints for live inspect overlays.",
        vec![overlay_hooks],
    );
    let raw_section = diag_section(
        cx,
        "Raw Inspect Payloads",
        "Raw inspect.hover, inspect.focus, and overlay.summary payloads remain available for protocol triage.",
        vec![raw_payloads],
    );

    ui::v_stack(|_cx| [hover_section, overlay_section, raw_section])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
}

fn inspect_hover_bounds_lines(hover_json: &str) -> Vec<String> {
    let Some(payload) = parse_inspect_json::<UiInspectHoverV1>(hover_json) else {
        return vec![
            "hover: <none>".to_string(),
            "hover bounds: <none>".to_string(),
        ];
    };

    let mut lines = vec![format!("hover window={}", payload.window)];
    lines.push(inspect_rect_line("viewport", &payload.viewport_bounds));

    let Some(node) = payload.hovered else {
        lines.push("hovered node: <none>".to_string());
        lines.push("hover bounds: <none>".to_string());
        return lines;
    };

    lines.push(inspect_node_line("hovered node", &node));
    lines.push(inspect_rect_line("hover bounds", &node.bounds));
    lines.push(format!("selector_json: {}", node.selector_json));
    lines
}

fn inspect_overlay_hook_lines(
    hover_json: &str,
    focus_json: &str,
    overlay_json: &str,
) -> Vec<String> {
    let hover = parse_inspect_json::<UiInspectHoverV1>(hover_json);
    let focus = parse_inspect_json::<UiInspectFocusV1>(focus_json);
    let overlay = parse_inspect_json::<UiOverlaySummaryV1>(overlay_json);

    let mut lines = Vec::new();
    if let Some(hover) = hover.as_ref() {
        lines.push(inspect_overlay_hook_line("hover overlay hook", &hover.overlay_hook));
    } else {
        lines.push("hover overlay hook: <none>".to_string());
    }
    if let Some(focus) = focus.as_ref() {
        lines.push(inspect_overlay_hook_line("focus overlay hook", &focus.overlay_hook));
        if let Some(summary) = focus.summary.as_deref() {
            lines.push(format!("focus summary: {summary}"));
        }
        if let Some(path) = focus.path.as_deref() {
            lines.push(format!("focus path: {path}"));
        }
    } else {
        lines.push("focus overlay hook: <none>".to_string());
    }

    if let Some(overlay) = overlay {
        lines.push(format!(
            "overlay barrier root: {}",
            inspect_opt_u64(overlay.barrier_root)
        ));
        lines.push(format!(
            "overlay focus barrier root: {}",
            inspect_opt_u64(overlay.focus_barrier_root)
        ));
        lines.push(format!(
            "overlay blocking roots: {}",
            overlay.blocking_roots.len()
        ));
        for root in overlay.blocking_roots.iter().take(4) {
            lines.push(format!(
                "blocking root={} z={} visible={} hit_testable={}",
                root.root, root.z_index, root.visible, root.hit_testable
            ));
        }
        if let Some(root) = overlay.topmost_interactive_root {
            lines.push(format!(
                "topmost interactive root={} z={} blocks_underlay_input={}",
                root.root, root.z_index, root.blocks_underlay_input
            ));
        } else {
            lines.push("topmost interactive root: <none>".to_string());
        }
    } else {
        lines.push("overlay summary: <none>".to_string());
    }

    lines
}

fn inspect_raw_payload_text(hover_json: &str, focus_json: &str, overlay_json: &str) -> String {
    if hover_json.trim().is_empty() && focus_json.trim().is_empty() && overlay_json.trim().is_empty()
    {
        return String::new();
    }
    format!("hover:\n{hover_json}\n\nfocus:\n{focus_json}\n\noverlay.summary:\n{overlay_json}")
}

fn parse_inspect_json<T: serde::de::DeserializeOwned>(text: &str) -> Option<T> {
    if text.trim().is_empty() {
        return None;
    }
    serde_json::from_str(text).ok()
}

fn inspect_node_line(label: &str, node: &fret_diag_protocol::UiInspectNodeSummaryV1) -> String {
    let test_id = node.test_id.as_deref().unwrap_or("<none>");
    let root = node
        .root
        .map(|root| root.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let root_z = node
        .root_z_index
        .map(|z| z.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    format!(
        "{label}: node={} role={} test_id={} root={} root_z_index={}",
        node.node_id, node.role, test_id, root, root_z
    )
}

fn inspect_rect_line(label: &str, rect: &UiRectV1) -> String {
    format!(
        "{label}: x={:.1} y={:.1} w={:.1} h={:.1}",
        rect.x_px, rect.y_px, rect.w_px, rect.h_px
    )
}

fn inspect_overlay_hook_line(
    label: &str,
    hook: &fret_diag_protocol::UiInspectOverlayHookV1,
) -> String {
    let target = hook
        .target_node_id
        .map(|node| node.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let bounds = hook
        .target_bounds
        .as_ref()
        .map(|rect| inspect_rect_line("target bounds", rect))
        .unwrap_or_else(|| "target bounds: <none>".to_string());
    format!(
        "{label}: kind={} space={} target_node={} {} {}",
        hook.kind,
        hook.coordinate_space,
        target,
        bounds,
        inspect_rect_line("viewport", &hook.viewport_bounds)
    )
}

fn inspect_opt_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_string())
}

fn regression_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let theme = cx.theme_snapshot();
    let loaded_dir = cx
        .app
        .models()
        .read(&st.regression_loaded_dir, |v| v.clone())
        .ok()
        .flatten();
    let error = cx
        .app
        .models()
        .read(&st.regression_last_error, |v| v.clone())
        .ok()
        .flatten();
    let dashboard = cx
        .app
        .models()
        .read(&st.regression_dashboard_human, |v| v.clone())
        .unwrap_or_default();
    let index_json = cx
        .app
        .models()
        .read(&st.regression_index_json, |v| v.clone())
        .unwrap_or_default();
    let summary_json = cx
        .app
        .models()
        .read(&st.regression_summary_json, |v| v.clone())
        .unwrap_or_default();
    let selected_summary_path = cx
        .app
        .models()
        .read(&st.regression_selected_summary_path, |v| v.clone())
        .ok()
        .flatten();
    let selected_summary_json = cx
        .app
        .models()
        .read(&st.regression_selected_summary_json, |v| v.clone())
        .unwrap_or_default();
    let selected_bundle_dirs = cx
        .app
        .models()
        .read(&st.regression_selected_bundle_dirs, |v| v.clone())
        .unwrap_or_default();
    let selected_capability_sources = cx
        .app
        .models()
        .read(&st.regression_selected_capability_sources, |v| v.clone())
        .unwrap_or_default();
    let selected_capabilities_checks = cx
        .app
        .models()
        .read(&st.regression_selected_capabilities_checks, |v| v.clone())
        .unwrap_or_default();
    let selected_perf_evidence = cx
        .app
        .models()
        .read(&st.regression_selected_perf_evidence, |v| v.clone())
        .unwrap_or_default();
    let selected_first_open_evidence = cx
        .app
        .models()
        .read(&st.regression_selected_first_open_evidence, |v| v.clone())
        .unwrap_or_default();
    let selected_share_artifacts = cx
        .app
        .models()
        .read(&st.regression_selected_share_artifacts, |v| v.clone())
        .unwrap_or_default();
    let selected_error = cx
        .app
        .models()
        .read(&st.regression_selected_error, |v| v.clone())
        .ok()
        .flatten();
    let can_refresh = cx
        .app
        .models()
        .read(&st.target_out_dir, |v| v.is_some())
        .unwrap_or(false);
    let pack_in_flight = cx
        .app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    let can_pack_selected_bundle = !selected_bundle_dirs.is_empty();
    let summarize_in_flight = cx
        .app
        .models()
        .read(&st.summarize_in_flight, |v| *v)
        .unwrap_or(false);
    let summarize_last_error = cx
        .app
        .models()
        .read(&st.summarize_last_error, |v| v.clone())
        .ok()
        .flatten();
    let followup_in_flight = cx
        .app
        .models()
        .read(&st.followup_in_flight, |v| *v)
        .unwrap_or(false);
    let followup_last_command_line = cx
        .app
        .models()
        .read(&st.followup_last_command_line, |v| v.clone())
        .ok()
        .flatten();
    let followup_last_result_path = cx
        .app
        .models()
        .read(&st.followup_last_result_path, |v| v.clone())
        .ok()
        .flatten();
    let followup_result_history = cx
        .app
        .models()
        .read(&st.followup_result_history, |v| v.clone())
        .unwrap_or_default();
    let followup_selected_result_path = cx
        .app
        .models()
        .read(&st.followup_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    let followup_last_error = cx
        .app
        .models()
        .read(&st.followup_last_error, |v| v.clone())
        .ok()
        .flatten();
    let followup_baseline_bundle_or_dir = cx
        .app
        .models()
        .read(&st.followup_baseline_bundle_or_dir, |v| v.clone())
        .unwrap_or_default();
    let followup_baseline_session = cx
        .app
        .models()
        .read(&st.followup_baseline_session, |v| v.clone())
        .unwrap_or_default();
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let selected_followup_history_filter_dirs =
        selected_followup_history_filter_dirs_from_bundle_dirs(
            &st.script_paths,
            &selected_bundle_dirs,
        );
    let selected_followup_history_entries =
        followup::followup_result_history_entries_for_selected_bundle(
            &followup_result_history,
            selected_followup_history_filter_dirs
                .iter()
                .map(|value| value.as_str()),
        );
    let selected_followup_result_entry = followup::followup_result_history_selected_or_latest_entry(
        &followup_result_history,
        selected_followup_history_filter_dirs
            .iter()
            .map(|value| value.as_str()),
        followup_selected_result_path.as_deref(),
    );
    let selected_followup_result_path = selected_followup_result_entry
        .as_ref()
        .map(|entry| entry.result_path.clone());
    let selected_followup_result_json = selected_followup_result_entry
        .as_ref()
        .map(|entry| entry.result_json.clone())
        .unwrap_or_default();
    let selected_followup_trace_artifact_path =
        followup::followup_trace_artifact_path_from_result_json(&selected_followup_result_json)
            .map(|path| resolve_repo_or_abs_path(&repo_root, &path).to_string_lossy().to_string());
    let failing_rows = regression_failing_summary_rows(&index_json, 10);
    let failing_count = failing_rows.len();
    let selected_bundle_count = selected_bundle_dirs.len();
    let selected_capability_source_count = selected_capability_sources.len();
    let selected_capabilities_check_count = selected_capabilities_checks.len();
    let selected_perf_evidence_count = selected_perf_evidence.len();
    let selected_first_open_evidence_count = selected_first_open_evidence.len();
    let selected_share_artifact_count = selected_share_artifacts.len();
    let summarize_status_line = {
        let err = summarize_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "summarize_in_flight={} summarize_last_error={err}",
            if summarize_in_flight { "true" } else { "false" }
        )
    };
    let followup_status_line = {
        let command = followup_last_command_line
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let result = followup_last_result_path
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let err = followup_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "followup_in_flight={} last_followup_command={command} last_followup_result={result} followup_last_error={err}",
            if followup_in_flight { "true" } else { "false" }
        )
    };
    let loaded_dir_line = loaded_dir
        .as_deref()
        .map(|v| format!("Artifacts root: {v}"))
        .unwrap_or_else(|| "Artifacts root: <not loaded>".to_string());
    let aggregate_preview = if !dashboard.trim().is_empty() {
        dashboard.clone()
    } else if let Some(err) = error.as_deref() {
        format!("Regression load error: {err}")
    } else {
        "No aggregate dashboard loaded yet. Use Refresh or Summarize against the current artifacts root.".to_string()
    };

    let _aggregate_content = {
        let mut parts: Vec<String> = Vec::new();
        if let Some(dir) = loaded_dir.as_deref() {
            parts.push(format!("loaded_dir: {dir}"));
        }
        if let Some(err) = error.as_deref() {
            parts.push(format!("error: {err}"));
        }
        if !dashboard.trim().is_empty() {
            parts.push("dashboard:".to_string());
            parts.push(dashboard);
        }
        if !index_json.trim().is_empty() {
            parts.push("regression.index.json:".to_string());
            parts.push(index_json.clone());
        }
        if !summary_json.trim().is_empty() {
            parts.push("regression.summary.json:".to_string());
            parts.push(summary_json.clone());
        }
        if parts.is_empty() {
            "<empty>".to_string()
        } else {
            parts.join(
                "

",
            )
        }
    };

    let failing_list = if failing_rows.is_empty() {
        shadcn::ScrollArea::new([
            cx.text("No non-passing summaries in the current regression index.")
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .min_h(Px(220.0)),
        )
        .into_element(cx)
    } else {
        let mut rows: Vec<AnyElement> = Vec::new();
        for row in failing_rows {
            let resolved_summary_path = resolve_repo_or_abs_path(&repo_root, &row.path);
            let resolved_summary_path_str = resolved_summary_path.to_string_lossy().to_string();
            let is_selected = selected_summary_path
                .as_deref()
                .is_some_and(|selected| selected == resolved_summary_path_str);
            let title = row.path.clone();
            let lane_label = format!("lane {}", row.lane);
            let failures_label = format!("failures {}", row.failures);
            let items_label = format!("items {}", row.items_total);
            let resolved_path_label = format!("Resolved path: {}", resolved_summary_path_str);
            let selected_summary_path_model = st.regression_selected_summary_path.clone();
            let selected_summary_json_model = st.regression_selected_summary_json.clone();
            let selected_bundle_dirs_model = st.regression_selected_bundle_dirs.clone();
            let selected_capability_sources_model =
                st.regression_selected_capability_sources.clone();
            let selected_capabilities_checks_model =
                st.regression_selected_capabilities_checks.clone();
            let selected_perf_evidence_model = st.regression_selected_perf_evidence.clone();
            let selected_first_open_evidence_model =
                st.regression_selected_first_open_evidence.clone();
            let selected_share_artifacts_model = st.regression_selected_share_artifacts.clone();
            let selected_error_model = st.regression_selected_error.clone();
            let log_lines_model = st.log_lines.clone();
            let copy_path = resolved_summary_path_str.clone();
            let select_path = resolved_summary_path_str.clone();
            let on_select: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let path = PathBuf::from(&select_path);
                    match load_regression_summary_drilldown(&path) {
                        Ok(data) => {
                            let _ = host.models_mut().update(&selected_summary_path_model, |v| {
                                *v = Some(Arc::<str>::from(select_path.clone()));
                            });
                            let _ = host.models_mut().update(&selected_summary_json_model, |v| {
                                *v = data.summary_json;
                            });
                            let _ = host.models_mut().update(&selected_bundle_dirs_model, |v| {
                                *v = data.bundle_dirs.into_iter().map(Arc::<str>::from).collect();
                            });
                            let _ =
                                host.models_mut()
                                    .update(&selected_capability_sources_model, |v| {
                                        *v = data
                                            .capability_sources
                                            .into_iter()
                                            .map(Arc::<str>::from)
                                            .collect();
                                    });
                            let _ = host.models_mut().update(
                                &selected_capabilities_checks_model,
                                |v| {
                                    *v = data
                                        .capabilities_check_paths
                                        .into_iter()
                                        .map(Arc::<str>::from)
                                        .collect();
                                },
                            );
                            let _ = host.models_mut().update(&selected_perf_evidence_model, |v| {
                                *v = data
                                    .perf_evidence_lines
                                    .into_iter()
                                    .map(Arc::<str>::from)
                                    .collect();
                            });
                            let _ = host.models_mut().update(
                                &selected_first_open_evidence_model,
                                |v| {
                                    *v = data
                                        .first_open_evidence_lines
                                        .into_iter()
                                        .map(Arc::<str>::from)
                                        .collect();
                                },
                            );
                            let _ =
                                host.models_mut()
                                    .update(&selected_share_artifacts_model, |v| {
                                        *v = data
                                            .share_artifacts
                                            .into_iter()
                                            .map(Arc::<str>::from)
                                            .collect();
                                    });
                            let _ = host
                                .models_mut()
                                .update(&selected_error_model, |v| *v = None);
                        }
                        Err(err) => {
                            let _ = host.models_mut().update(&selected_summary_path_model, |v| {
                                *v = Some(Arc::<str>::from(select_path.clone()));
                            });
                            let _ = host
                                .models_mut()
                                .update(&selected_summary_json_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_bundle_dirs_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_capability_sources_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_capabilities_checks_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_perf_evidence_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_first_open_evidence_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_share_artifacts_model, |v| v.clear());
                            let _ = host.models_mut().update(&selected_error_model, |v| {
                                *v = Some(Arc::<str>::from(format!(
                                    "failed to load selected regression summary {}: {err}",
                                    path.display()
                                )))
                            });
                            let _ = host.models_mut().update(&log_lines_model, |v| {
                                v.push(Arc::<str>::from(format!(
                                    "regression summary drill-down load failed: {}",
                                    path.display()
                                )));
                                if v.len() > 2000 {
                                    let drain = v.len().saturating_sub(2000);
                                    v.drain(0..drain);
                                }
                            });
                        }
                    }
                    host.request_redraw(action_cx.window);
                });
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: copy_path.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            let title_text = cx.text(title);
            let resolved_path_text = cx.text(resolved_path_label);
            let badges = ui::h_row(|cx| {
                [
                    shadcn::Badge::new(if is_selected {
                        "Selected"
                    } else {
                        "Non-passing"
                    })
                    .variant(if is_selected {
                        shadcn::BadgeVariant::Secondary
                    } else {
                        shadcn::BadgeVariant::Destructive
                    })
                    .into_element(cx),
                    shadcn::Badge::new(lane_label)
                        .variant(shadcn::BadgeVariant::Outline)
                        .into_element(cx),
                    shadcn::Badge::new(failures_label)
                        .variant(shadcn::BadgeVariant::Destructive)
                        .into_element(cx),
                    shadcn::Badge::new(items_label)
                        .variant(shadcn::BadgeVariant::Outline)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx);
            let actions = ui::h_row(|cx| {
                [
                    shadcn::Button::new(if is_selected {
                        "Opened"
                    } else {
                        "Open details"
                    })
                    .variant(if is_selected {
                        shadcn::ButtonVariant::Secondary
                    } else {
                        shadcn::ButtonVariant::Ghost
                    })
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_select)
                    .into_element(cx),
                    shadcn::Button::new("Copy path")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx);
            rows.push(
                shadcn::Card::new([shadcn::CardContent::new([
                    badges,
                    title_text,
                    resolved_path_text,
                    actions,
                ])
                .into_element(cx)])
                .into_element(cx),
            );
        }
        shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
            .gap(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx)])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .min_h(Px(260.0)),
        )
        .into_element(cx)
    };

    let selected_bundle_dirs_text = selected_bundle_dirs
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_capability_sources_text = selected_capability_sources
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_capabilities_checks_text = selected_capabilities_checks
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_perf_evidence_text = selected_perf_evidence
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_first_open_evidence_text = selected_first_open_evidence
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_share_artifacts_text = selected_share_artifacts
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_followup_commands =
        regression_bundle_followup_commands(selected_bundle_dirs.iter().map(|v| v.as_ref()));
    let selected_runnable_followup_command_lines = selected_followup_commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .map(|command| command.display_line())
        .collect::<Vec<_>>();
    let selected_manual_followup_command_lines = selected_followup_commands
        .iter()
        .filter(|command| command.requires_baseline)
        .map(|command| command.display_line())
        .collect::<Vec<_>>();
    let selected_runnable_followup_count = selected_runnable_followup_command_lines.len();
    let selected_manual_followup_count = selected_manual_followup_command_lines.len();
    let selected_followup_readiness_lines = selected_followup_readiness_lines(
        selected_bundle_count,
        &selected_followup_commands,
        &followup_baseline_bundle_or_dir,
        &followup_baseline_session,
    );
    let selected_followup_command_lines =
        regression_bundle_followup_command_lines(selected_bundle_dirs.iter().map(|v| v.as_ref()));
    let selected_followup_commands_text = selected_followup_command_lines.join("\r\n");
    let selected_followup_commands_display = if selected_followup_command_lines.is_empty() {
        "Select a non-passing summary with bundle_dir evidence to generate concrete follow-up commands.".to_string()
    } else {
        selected_followup_commands_text.clone()
    };
    let selected_runnable_followup_commands_text =
        selected_runnable_followup_command_lines.join("\r\n");
    let selected_runnable_followup_commands_display =
        if selected_runnable_followup_command_lines.is_empty() {
            "No bundle-local follow-up command is runnable from this selection yet.".to_string()
        } else {
            selected_runnable_followup_commands_text.clone()
        };
    let selected_manual_followup_commands_text = selected_manual_followup_command_lines.join("\r\n");
    let selected_manual_followup_commands_display =
        if selected_manual_followup_command_lines.is_empty() {
            "No baseline-required compare follow-up command for this selection.".to_string()
        } else {
            selected_manual_followup_commands_text.clone()
        };
    let selected_summary_overview = {
        let mut parts: Vec<String> = Vec::new();
        match selected_summary_path.as_deref() {
            Some(path) => parts.push(format!("Selected summary: {path}")),
            None => parts.push("Selected summary: <none>".to_string()),
        }
        parts.push(format!("Selected bundle dirs: {selected_bundle_count}"));
        if let Some(first) = selected_bundle_dirs.first() {
            parts.push(format!("First bundle dir: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected capability sources: {selected_capability_source_count}"
        ));
        if let Some(first) = selected_capability_sources.first() {
            parts.push(format!("First capability source: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected capability checks: {selected_capabilities_check_count}"
        ));
        if let Some(first) = selected_capabilities_checks.first() {
            parts.push(format!("First capability check: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected perf evidence lines: {selected_perf_evidence_count}"
        ));
        if let Some(first) = selected_perf_evidence.first() {
            parts.push(format!("First perf evidence: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected first-open evidence lines: {selected_first_open_evidence_count}"
        ));
        if let Some(first) = selected_first_open_evidence.first() {
            parts.push(format!("First first-open evidence: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected share artifacts: {selected_share_artifact_count}"
        ));
        if let Some(first) = selected_share_artifacts.first() {
            parts.push(format!("First share artifact: {}", first.as_ref()));
        }
        parts.push(format!(
            "Runnable follow-up commands: {selected_runnable_followup_count}"
        ));
        parts.push(format!(
            "Manual compare follow-up commands: {selected_manual_followup_count}"
        ));
        if let Some(err) = selected_error.as_deref() {
            parts.push(format!("Selected error: {err}"));
        }
        parts.join("\r\n")
    };
    let selected_detail_content = {
        let mut parts: Vec<String> = Vec::new();
        if let Some(path) = selected_summary_path.as_deref() {
            parts.push(format!("selected_summary_path: {path}"));
        }
        if !selected_bundle_dirs_text.trim().is_empty() {
            parts.push("bundle_dirs:".to_string());
            parts.push(selected_bundle_dirs_text.clone());
        }
        if !selected_capability_sources_text.trim().is_empty() {
            parts.push("capability_sources:".to_string());
            parts.push(selected_capability_sources_text.clone());
        }
        if !selected_capabilities_checks_text.trim().is_empty() {
            parts.push("capabilities_check_paths:".to_string());
            parts.push(selected_capabilities_checks_text.clone());
        }
        if !selected_perf_evidence_text.trim().is_empty() {
            parts.push("perf_evidence:".to_string());
            parts.push(selected_perf_evidence_text.clone());
        }
        if !selected_first_open_evidence_text.trim().is_empty() {
            parts.push("first_open_evidence:".to_string());
            parts.push(selected_first_open_evidence_text.clone());
        }
        if !selected_share_artifacts_text.trim().is_empty() {
            parts.push("share_artifacts:".to_string());
            parts.push(selected_share_artifacts_text.clone());
        }
        if let Some(err) = selected_error.as_deref() {
            parts.push(format!("error: {err}"));
        }
        if !selected_summary_json.trim().is_empty() {
            parts.push("selected regression.summary.json:".to_string());
            parts.push(selected_summary_json);
        }
        if parts.is_empty() {
            "<empty>".to_string()
        } else {
            parts.join(
                "

",
            )
        }
    };
    let selected_actions = ui::h_row(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if let Some(path) = selected_summary_path.as_ref().map(|v| v.to_string()) {
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: path.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy selected path")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                .into_element(cx),
            );
        }
        if !selected_followup_commands_text.trim().is_empty() {
            let followup_commands = selected_followup_commands_text.clone();
            let on_copy: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: followup_commands.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy follow-up commands")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "stats")
        {
            out.push(
                shadcn::Button::new("Run stats")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_STATS)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "layout-perf-summary")
        {
            out.push(
                shadcn::Button::new("Run layout perf")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "memory-summary")
        {
            out.push(
                shadcn::Button::new("Run memory")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_MEMORY)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "triage")
        {
            out.push(
                shadcn::Button::new("Run triage")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "hotspots")
        {
            out.push(
                shadcn::Button::new("Run hotspots")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "trace")
        {
            out.push(
                shadcn::Button::new("Run trace")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_TRACE)
                    .into_element(cx),
            );
        }
        if selected_followup_result_path.is_some() {
            out.push(
                shadcn::Button::new("Copy selected follow-up result")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_RESULT_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open selected follow-up JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_FOLLOWUP_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if selected_followup_result_entry.is_some() {
            out.push(
                shadcn::Button::new("Copy selected follow-up command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_RESULT_COMMAND)
                    .into_element(cx),
            );
        }
        if !selected_followup_result_json.trim().is_empty() {
            out.push(
                shadcn::Button::new("Copy selected follow-up JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if selected_followup_trace_artifact_path.is_some() {
            out.push(
                shadcn::Button::new("Copy selected trace artifact")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_TRACE_ARTIFACT_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open selected trace artifact")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_FOLLOWUP_TRACE_ARTIFACT)
                    .into_element(cx),
            );
        }
        if let Some(first_bundle_dir) = selected_bundle_dirs.first().map(|v| v.to_string()) {
            let on_copy_first: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: first_bundle_dir.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy first bundle dir")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy_first)
                    .into_element(cx),
            );
        }
        if let Some(first_capability_check) =
            selected_capabilities_checks.first().map(|v| v.to_string())
        {
            let on_copy_first: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: first_capability_check.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy first capability check")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy_first)
                    .into_element(cx),
            );
        }
        if let Some(first_capability_source) =
            selected_capability_sources.first().map(|v| v.to_string())
        {
            let on_copy_first: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: first_capability_source.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy first capability source")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy_first)
                    .into_element(cx),
            );
        }
        out.push(
            shadcn::Button::new("Pack selected evidence")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_pack_selected_bundle || pack_in_flight)
                .on_click(CMD_REGRESSION_PACK_SELECTED_BUNDLE)
                .into_element(cx),
        );
        if !selected_bundle_dirs_text.trim().is_empty() {
            let bundle_dirs = selected_bundle_dirs_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: bundle_dirs.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy bundle dirs")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if !selected_capabilities_checks_text.trim().is_empty() {
            let capability_checks = selected_capabilities_checks_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: capability_checks.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy capability checks")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if !selected_capability_sources_text.trim().is_empty() {
            let capability_sources = selected_capability_sources_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: capability_sources.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy capability sources")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                .into_element(cx),
            );
        }
        if !selected_perf_evidence_text.trim().is_empty() {
            let perf_evidence = selected_perf_evidence_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: perf_evidence.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy perf evidence")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                .into_element(cx),
            );
        }
        if !selected_first_open_evidence_text.trim().is_empty() {
            let first_open_evidence = selected_first_open_evidence_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: first_open_evidence.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy first-open evidence")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if !selected_share_artifacts_text.trim().is_empty() {
            let share_artifacts = selected_share_artifacts_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: share_artifacts.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy share artifacts")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        out
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let selected_summary_badges = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if selected_summary_path.is_some() {
                "Summary selected"
            } else {
                "No selection"
            })
            .variant(if selected_summary_path.is_some() {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("bundle dirs {selected_bundle_count}"))
                .variant(if selected_bundle_count > 0 {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(format!(
                "capability sources {selected_capability_source_count}"
            ))
            .variant(if selected_capability_source_count > 0 {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!(
                "capability checks {selected_capabilities_check_count}"
            ))
            .variant(if selected_capabilities_check_count > 0 {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("perf evidence {selected_perf_evidence_count}"))
                .variant(if selected_perf_evidence_count > 0 {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(format!(
                "first-open evidence {selected_first_open_evidence_count}"
            ))
            .variant(if selected_first_open_evidence_count > 0 {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("share artifacts {selected_share_artifact_count}"))
                .variant(if selected_share_artifact_count > 0 {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(if selected_error.is_some() {
                "Selection error"
            } else {
                "Selection ok"
            })
            .variant(if selected_error.is_some() {
                shadcn::BadgeVariant::Destructive
            } else {
                shadcn::BadgeVariant::Secondary
            })
            .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let status_row = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if can_refresh {
                "Artifacts root ready"
            } else {
                "No artifacts root"
            })
            .variant(if can_refresh {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("non-passing {failing_count}"))
                .variant(if failing_count > 0 {
                    shadcn::BadgeVariant::Destructive
                } else {
                    shadcn::BadgeVariant::Secondary
                })
                .into_element(cx),
            shadcn::Badge::new(format!("selected bundles {selected_bundle_count}"))
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
            shadcn::Badge::new(format!(
                "selected capability sources {selected_capability_source_count}"
            ))
            .variant(shadcn::BadgeVariant::Outline)
            .into_element(cx),
            shadcn::Badge::new(format!(
                "selected capability checks {selected_capabilities_check_count}"
            ))
            .variant(shadcn::BadgeVariant::Outline)
            .into_element(cx),
            shadcn::Badge::new(if summarize_in_flight {
                "Summarizing"
            } else {
                "Summarize idle"
            })
            .variant(if summarize_in_flight {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let top_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Refresh")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_refresh)
                .on_click(CMD_REGRESSION_REFRESH)
                .into_element(cx),
            shadcn::Button::new("Summarize")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_refresh || summarize_in_flight)
                .on_click(CMD_REGRESSION_SUMMARIZE)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let loaded_dir_text = cx.text(loaded_dir_line);
    let summarize_status_text = cx.text(summarize_status_line);
    let aggregate_preview_blob = text_blob_sized(cx, aggregate_preview.clone(), Px(96.0));
    let overview_status_section = diag_section(
        cx,
        "Aggregate Status",
        "Keep artifacts-root readiness and current failure counters visible at the top of the workspace.",
        vec![status_row, loaded_dir_text, summarize_status_text],
    );
    let overview_actions_section = diag_section(
        cx,
        "Primary Actions",
        "Refresh aggregate artifacts or run summarize without losing sight of the current counters.",
        vec![top_actions],
    );
    let overview_preview_section = diag_section(
        cx,
        "Dashboard Preview",
        "A compact aggregate preview stays available here, while full debug payloads live lower in the tab.",
        vec![aggregate_preview_blob],
    );

    let overview_card = diag_card(
        cx,
        "Regression Workspace",
        "Summary-first view over aggregate artifacts, non-passing summaries, and evidence actions.",
        vec![
            overview_status_section,
            overview_actions_section,
            overview_preview_section,
        ],
    );

    let left_card = diag_card(
        cx,
        "Non-passing Summaries",
        "Select one non-passing summary to open its evidence-focused drill-down.",
        vec![failing_list],
    );

    let selected_summary_overview_text = cx.text(selected_summary_overview);
    let selected_followup_status_text = cx.text(followup_status_line);
    let selected_followup_result_detail_blob = text_blob_sized(
        cx,
        followup::followup_result_history_entry_detail_lines(
            selected_followup_result_entry.as_ref(),
        )
        .join("\r\n"),
        Px(120.0),
    );
    let selected_followup_result_summary_blob = text_blob_sized(
        cx,
        followup::followup_result_summary_lines(&selected_followup_result_json).join("\r\n"),
        Px(96.0),
    );
    let selected_followup_result_history_blob = text_blob_sized(
        cx,
        followup::followup_result_history_summary_lines(
            &followup_result_history,
            selected_followup_history_filter_dirs
                .iter()
                .map(|value| value.as_str()),
        )
        .join("\r\n"),
        Px(120.0),
    );
    let selected_followup_result_history_list = followup_history_list(
        cx,
        &st.followup_selected_result_path,
        &selected_followup_history_entries,
        selected_followup_result_path.as_deref(),
    );
    let selected_runnable_followup_actions =
        runnable_followup_command_actions(
            cx,
            &st.followup_pending_command_id,
            &selected_followup_commands,
            followup_in_flight,
        );
    let baseline_bundle_input = shadcn::Input::new(st.followup_baseline_bundle_or_dir.clone())
        .a11y_label("Baseline bundle or directory")
        .placeholder("baseline bundle or dir")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);
    let baseline_session_input = shadcn::Input::new(st.followup_baseline_session.clone())
        .a11y_label("Baseline footprint session")
        .placeholder("baseline session")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(260.0)))
        .into_element(cx);
    let has_visual_compare = selected_followup_commands
        .iter()
        .any(|command| command.id == "visual-compare");
    let has_footprint_compare = selected_followup_commands
        .iter()
        .any(|command| command.id == "footprint-compare");
    let visual_compare_ready =
        has_visual_compare && !followup_baseline_bundle_or_dir.trim().is_empty();
    let footprint_compare_ready =
        has_footprint_compare && !followup_baseline_session.trim().is_empty();
    let visual_compare_button = shadcn::Button::new("Run visual compare")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!visual_compare_ready || followup_in_flight)
        .on_click(CMD_REGRESSION_RUN_VISUAL_COMPARE)
        .into_element(cx);
    let footprint_compare_button = shadcn::Button::new("Run footprint compare")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!footprint_compare_ready || followup_in_flight)
        .on_click(CMD_REGRESSION_RUN_FOOTPRINT_COMPARE)
        .into_element(cx);
    let visual_compare_row = ui::h_row(|_cx| [baseline_bundle_input, visual_compare_button])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .into_element(cx);
    let footprint_compare_row =
        ui::h_row(|_cx| [baseline_session_input, footprint_compare_button])
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx);
    let baseline_compare_controls = ui::v_stack(|_cx| [visual_compare_row, footprint_compare_row])
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let selected_followup_result_json_blob = text_blob_sized(
        cx,
        if selected_followup_result_json.trim().is_empty() {
            "<no selected-bundle follow-up result yet>".to_string()
        } else {
            selected_followup_result_json
        },
        Px(140.0),
    );
    let selected_followup_readiness_blob = text_blob_sized(
        cx,
        selected_followup_readiness_lines.join("\r\n"),
        Px(84.0),
    );
    let selected_bundle_dirs_blob =
        text_blob_sized(cx, selected_bundle_dirs_text.clone(), Px(96.0));
    let selected_capability_sources_blob =
        text_blob_sized(cx, selected_capability_sources_text.clone(), Px(96.0));
    let selected_capabilities_blob =
        text_blob_sized(cx, selected_capabilities_checks_text.clone(), Px(96.0));
    let selected_perf_evidence_blob =
        text_blob_sized(cx, selected_perf_evidence_text.clone(), Px(120.0));
    let selected_first_open_evidence_blob =
        text_blob_sized(cx, selected_first_open_evidence_text.clone(), Px(120.0));
    let selected_share_artifacts_blob =
        text_blob_sized(cx, selected_share_artifacts_text.clone(), Px(96.0));
    let selected_followup_commands_blob =
        text_blob_sized(cx, selected_followup_commands_display, Px(116.0));
    let selected_runnable_followup_commands_blob =
        text_blob_sized(cx, selected_runnable_followup_commands_display, Px(96.0));
    let selected_manual_followup_commands_blob =
        text_blob_sized(cx, selected_manual_followup_commands_display, Px(96.0));
    let selected_raw_summary_blob = text_blob_sized(cx, selected_detail_content, Px(220.0));
    let selected_overview_section = diag_section(
        cx,
        "Selection Overview",
        "Keep the current non-passing state visible before diving into raw JSON.",
        vec![selected_summary_badges, selected_summary_overview_text],
    );
    let selected_actions_section = diag_section(
        cx,
        "Evidence Actions",
        "Copy paths or pack the currently selected evidence without leaving this inspector.",
        vec![selected_actions],
    );
    let selected_followup_run_status_section = diag_section(
        cx,
        "Follow-up Run Status",
        "Runnable follow-up commands execute through the shared diagnostics engine and report status here.",
        vec![selected_followup_status_text],
    );
    let selected_followup_readiness_section = diag_section(
        cx,
        "Follow-up Readiness",
        "A compact readiness summary links selected summary evidence to the next runnable command.",
        vec![selected_followup_readiness_blob],
    );
    let selected_followup_result_detail_section = diag_section(
        cx,
        "Follow-up Result Details",
        "Selected result status, path, command, bundle, and error preview for reproduction.",
        vec![selected_followup_result_detail_blob],
    );
    let selected_followup_result_summary_section = diag_section(
        cx,
        "Follow-up Result Summary",
        "Status, command, duration, and error preview from the latest selected-bundle follow-up result.",
        vec![selected_followup_result_summary_blob],
    );
    let selected_followup_result_history_section = diag_section(
        cx,
        "Follow-up Result History",
        "Select a GUI-launched follow-up result for the selected bundle, newest first.",
        vec![
            selected_followup_result_history_blob,
            selected_followup_result_history_list,
        ],
    );
    let selected_runnable_followup_actions_section = diag_section(
        cx,
        "Runnable Follow-up Actions",
        "Run any bundle-local follow-up command generated for the selected summary.",
        vec![selected_runnable_followup_actions],
    );
    let selected_baseline_compare_actions_section = diag_section(
        cx,
        "Baseline Compare Actions",
        "Provide a baseline to turn manual compare templates into runnable diagnostics follow-ups.",
        vec![baseline_compare_controls],
    );
    let selected_followup_result_json_section = diag_section(
        cx,
        "Follow-up Result JSON",
        "The latest selected-bundle follow-up result artifact is mirrored here for quick triage.",
        vec![selected_followup_result_json_blob],
    );
    let selected_followup_commands_section = diag_section(
        cx,
        "Follow-up Commands",
        "Concrete stats, triage, hotspot, trace, visual-compare, and footprint commands are generated from the selected bundle directory.",
        vec![selected_followup_commands_blob],
    );
    let selected_runnable_followup_commands_section = diag_section(
        cx,
        "Runnable Follow-ups",
        "Bundle-local commands have concrete diag args and do not require a baseline selection.",
        vec![selected_runnable_followup_commands_blob],
    );
    let selected_manual_followup_commands_section = diag_section(
        cx,
        "Manual Compare Follow-ups",
        "Compare commands stay visible but are separated because they still require a baseline input.",
        vec![selected_manual_followup_commands_blob],
    );
    let selected_bundle_dirs_section = diag_section(
        cx,
        "Bundle Directories",
        "These are the concrete artifact roots attached to the selected non-passing summary.",
        vec![selected_bundle_dirs_blob],
    );
    let selected_capability_sources_section = diag_section(
        cx,
        "Capability Sources",
        "Capability provenance is shown separately from campaign-local check artifacts and prefers the additive source object when present.",
        vec![selected_capability_sources_blob],
    );
    let selected_capabilities_section = diag_section(
        cx,
        "Capability Checks",
        "Policy-skipped summaries can point at campaign capability check artifacts even when no bundle dir exists.",
        vec![selected_capabilities_blob],
    );
    let selected_perf_evidence_section = diag_section(
        cx,
        "Perf Evidence",
        "Perf summary paths, threshold artifacts, curated metrics, and threshold failures stay above the raw JSON.",
        vec![selected_perf_evidence_blob],
    );
    let selected_first_open_evidence_section = diag_section(
        cx,
        "First-open Evidence",
        "Canonical summary-level paths for triage, script results, screenshots, and share packs use the shared diagnostics drill-down projection.",
        vec![selected_first_open_evidence_blob],
    );
    let selected_share_artifacts_section = diag_section(
        cx,
        "Share Artifacts",
        "Compact handoff packages stay optional, but visible beside canonical evidence when the selected summary exposes them.",
        vec![selected_share_artifacts_blob],
    );
    let selected_raw_summary_section = diag_section(
        cx,
        "Raw Selected Summary",
        "Raw summary payload remains available for debugging, but below overview and actions.",
        vec![selected_raw_summary_blob],
    );

    let right_card = diag_card(
        cx,
        "Selected Summary",
        "Evidence actions and raw summary payload stay close to the current non-passing selection.",
        vec![
            selected_overview_section,
            selected_actions_section,
            selected_followup_readiness_section,
            selected_followup_run_status_section,
            selected_followup_result_detail_section,
            selected_followup_result_summary_section,
            selected_followup_result_history_section,
            selected_runnable_followup_actions_section,
            selected_baseline_compare_actions_section,
            selected_followup_result_json_section,
            selected_followup_commands_section,
            selected_runnable_followup_commands_section,
            selected_manual_followup_commands_section,
            selected_bundle_dirs_section,
            selected_capability_sources_section,
            selected_capabilities_section,
            selected_perf_evidence_section,
            selected_first_open_evidence_section,
            selected_share_artifacts_section,
            selected_raw_summary_section,
        ],
    );

    let split = ui::h_row(|cx| {
        [
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(372.0))
                        .h_full(),
                ),
                |_cx| [left_card],
            ),
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .flex_1()
                        .min_w_0()
                        .h_full(),
                ),
                |_cx| [right_card],
            ),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .items_start()
    .into_element(cx);

    let dashboard_debug_blob = text_blob_sized(cx, aggregate_preview.clone(), Px(96.0));
    let index_debug_blob = text_blob_sized(cx, index_json.clone(), Px(140.0));
    let summary_debug_blob = text_blob_sized(cx, summary_json.clone(), Px(140.0));
    let dashboard_debug_section = diag_section(
        cx,
        "Dashboard Preview",
        "Human-readable aggregate output for quick debugging and copy/paste.",
        vec![dashboard_debug_blob],
    );
    let index_debug_section = diag_section(
        cx,
        "regression.index.json",
        "Campaign index payload backing the aggregate workspace.",
        vec![index_debug_blob],
    );
    let summary_debug_section = diag_section(
        cx,
        "regression.summary.json",
        "Latest aggregate summary payload emitted by summarize/dashboard flows.",
        vec![summary_debug_blob],
    );
    let raw_payloads = diag_card(
        cx,
        "Aggregate Debug Payloads",
        "Keep dashboard and raw aggregate payloads available for debugging, but clearly below the main regression workflow.",
        vec![
            dashboard_debug_section,
            index_debug_section,
            summary_debug_section,
        ],
    );

    ui::v_stack(|_cx| [overview_card, split, raw_payloads])
        .gap(fret_ui_kit::Space::N2)
        .into_element(cx)
}

fn text_blob(cx: &mut ElementContext<'_, App>, text: String) -> AnyElement {
    let text = if text.is_empty() {
        "<empty>".to_string()
    } else {
        text
    };

    let pre = cx.text(text);
    shadcn::ScrollArea::new([pre]).into_element(cx)
}

fn text_blob_sized(cx: &mut ElementContext<'_, App>, text: String, min_h: Px) -> AnyElement {
    let text = if text.is_empty() {
        "<empty>".to_string()
    } else {
        text
    };

    let pre = cx.text(text);
    shadcn::ScrollArea::new([pre])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .min_h(min_h),
        )
        .into_element(cx)
}

fn followup_history_list(
    cx: &mut ElementContext<'_, App>,
    selected_result_path_model: &Model<Option<Arc<str>>>,
    entries: &[followup::FollowupResultHistoryEntry],
    active_result_path: Option<&str>,
) -> AnyElement {
    if entries.is_empty() {
        return text_blob_sized(
            cx,
            "follow-up history entries: <none for selected bundle>".to_string(),
            Px(84.0),
        );
    }

    let mut rows: Vec<AnyElement> = Vec::new();
    for entry in entries.iter().take(8) {
        let is_selected = active_result_path.is_some_and(|path| path == entry.result_path);
        let result_path = entry.result_path.clone();
        let selected_result_path_model = selected_result_path_model.clone();
        let label = format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            short_followup_result_path(&entry.result_path)
        );
        let on_activate: fret_ui::action::OnActivate =
            Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&selected_result_path_model, |value| {
                        *value = Some(Arc::<str>::from(result_path.clone()))
                    });
                host.request_redraw(action_cx.window);
            });
        rows.push(
            shadcn::Button::new(label)
                .variant(if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .into_element(cx),
        );
    }

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .min_h(Px(116.0)),
    )
    .into_element(cx)
}

#[cfg(test)]
fn runnable_followup_command_action_lines(
    commands: &[RegressionBundleFollowupCommandV1],
) -> Vec<String> {
    commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .map(|command| format!("{} ({})", command.label, command.id))
        .collect()
}

fn selected_followup_readiness_lines(
    selected_bundle_count: usize,
    commands: &[RegressionBundleFollowupCommandV1],
    baseline_bundle_or_dir: &str,
    baseline_session: &str,
) -> Vec<String> {
    let runnable = commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .collect::<Vec<_>>();
    let manual = commands
        .iter()
        .filter(|command| command.requires_baseline)
        .count();
    let has_visual_compare = commands.iter().any(|command| command.id == "visual-compare");
    let has_footprint_compare = commands
        .iter()
        .any(|command| command.id == "footprint-compare");
    let mut lines = vec![
        format!("selected_bundle_dirs: {selected_bundle_count}"),
        format!("runnable_followups: {}", runnable.len()),
        format!("manual_compare_followups: {manual}"),
        format!(
            "visual_compare_ready: {}",
            if has_visual_compare && !baseline_bundle_or_dir.trim().is_empty() {
                "true"
            } else {
                "false"
            }
        ),
        format!(
            "footprint_compare_ready: {}",
            if has_footprint_compare && !baseline_session.trim().is_empty() {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if let Some(first) = runnable.first() {
        lines.push(format!("first_runnable: {} ({})", first.label, first.id));
        lines.push(format!("first_command: {}", first.command_line));
    } else if selected_bundle_count == 0 {
        lines.push("state: no selected bundle evidence yet".to_string());
    } else {
        lines.push("state: selected bundle has no bundle-local follow-up command".to_string());
    }
    lines
}

fn materialize_baseline_compare_followup_command(
    command: &RegressionBundleFollowupCommandV1,
    baseline: &str,
) -> Result<RegressionBundleFollowupCommandV1, String> {
    let baseline = baseline.trim();
    if baseline.is_empty() {
        return Err(format!("missing baseline input for {}", command.label));
    }
    let target = command
        .target_bundle_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing target bundle dir for {}", command.label))?;
    if command.id.starts_with("visual-compare") {
        let baseline_arg = shell_quote_for_display(baseline);
        let target_arg = shell_quote_for_display(target);
        let mut out = command.clone();
        out.requires_baseline = false;
        out.command_line = format!(
            "cargo run -p fretboard-dev -- diag compare {baseline_arg} {target_arg} --json"
        );
        out.diag_args = vec![
            "compare".to_string(),
            baseline.to_string(),
            target.to_string(),
            "--json".to_string(),
        ];
        return Ok(out);
    }
    if command.id.starts_with("footprint-compare") {
        let baseline_arg = shell_quote_for_display(baseline);
        let target_arg = shell_quote_for_display(target);
        let mut out = command.clone();
        out.requires_baseline = false;
        out.command_line = format!(
            "cargo run -p fretboard-dev -- diag compare {baseline_arg} {target_arg} --footprint --json"
        );
        out.diag_args = vec![
            "compare".to_string(),
            baseline.to_string(),
            target.to_string(),
            "--footprint".to_string(),
            "--json".to_string(),
        ];
        return Ok(out);
    }
    Err(format!("unsupported baseline compare command {}", command.id))
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

fn runnable_followup_command_actions(
    cx: &mut ElementContext<'_, App>,
    pending_command_id_model: &Model<Option<Arc<str>>>,
    commands: &[RegressionBundleFollowupCommandV1],
    in_flight: bool,
) -> AnyElement {
    let runnable = commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .collect::<Vec<_>>();
    if runnable.is_empty() {
        return cx.text("No runnable follow-up commands for this selection.");
    }

    let rows = runnable
        .into_iter()
        .map(|command| {
            let command_id = command.id.clone();
            let command_label = command.label.clone();
            let command_line = command.command_line.clone();
            let pending_command_id_model = pending_command_id_model.clone();
            let action = CommandId::from(CMD_REGRESSION_RUN_FOLLOWUP_COMMAND);
            let on_run: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, reason| {
                    let _ = host.models_mut().update(&pending_command_id_model, |value| {
                        *value = Some(Arc::<str>::from(command_id.clone()))
                    });
                    host.record_pending_command_dispatch_source(action_cx, &action, reason);
                    host.dispatch_command(Some(action_cx.window), action.clone());
                });
            let label = shadcn::Badge::new(command_label)
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx);
            let run = shadcn::Button::new("Run")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(in_flight)
                .on_activate(on_run)
                .into_element(cx);
            let command = text_blob_sized(cx, command_line, Px(42.0));
            ui::h_row(|_cx| [label, run, command])
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx)
        })
        .collect::<Vec<_>>();

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .max_h(Px(160.0)),
    )
    .into_element(cx)
}

fn gate_run_history_list(
    cx: &mut ElementContext<'_, App>,
    selected_result_path_model: &Model<Option<Arc<str>>>,
    entries: &[gate_run::GateRunResultHistoryEntry],
    active_result_path: Option<&str>,
) -> AnyElement {
    if entries.is_empty() {
        return text_blob_sized(cx, "gate run history: <none>".to_string(), Px(84.0));
    }

    let mut rows: Vec<AnyElement> = Vec::new();
    for entry in entries.iter().take(8) {
        let is_selected = active_result_path.is_some_and(|path| path == entry.result_path);
        let result_path = entry.result_path.clone();
        let selected_result_path_model = selected_result_path_model.clone();
        let label = format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path)
        );
        let on_activate: fret_ui::action::OnActivate =
            Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&selected_result_path_model, |value| {
                        *value = Some(Arc::<str>::from(result_path.clone()))
                    });
                host.request_redraw(action_cx.window);
            });
        rows.push(
            shadcn::Button::new(label)
                .variant(if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .into_element(cx),
        );
    }

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .min_h(Px(116.0)),
    )
    .into_element(cx)
}

fn workflow_run_history_list(
    cx: &mut ElementContext<'_, App>,
    selected_result_path_model: &Model<Option<Arc<str>>>,
    entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    active_result_path: Option<&str>,
) -> AnyElement {
    if entries.is_empty() {
        return text_blob_sized(cx, "workflow run history: <none>".to_string(), Px(84.0));
    }

    let mut rows: Vec<AnyElement> = Vec::new();
    for entry in entries.iter().take(8) {
        let is_selected = active_result_path.is_some_and(|path| path == entry.result_path);
        let result_path = entry.result_path.clone();
        let selected_result_path_model = selected_result_path_model.clone();
        let label = format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path)
        );
        let on_activate: fret_ui::action::OnActivate =
            Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&selected_result_path_model, |value| {
                        *value = Some(Arc::<str>::from(result_path.clone()))
                    });
                host.request_redraw(action_cx.window);
            });
        rows.push(
            shadcn::Button::new(label)
                .variant(if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .into_element(cx),
        );
    }

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .min_h(Px(116.0)),
    )
    .into_element(cx)
}

fn short_followup_result_path(path: &str) -> String {
    short_artifact_result_path(path)
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

fn sem_node_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let fallback = cx
        .app
        .models()
        .read(&st.semantics_selected_node_json, |v| v.clone())
        .unwrap_or_default();
    let live = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_json, |v| v.clone())
        .unwrap_or_default();
    let live_status = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_status, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("unknown"));
    let live_updated = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_updated_unix_ms, |v| *v)
        .ok()
        .flatten();
    let children = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_children, |v| v.clone())
        .unwrap_or_default();
    let hit_test_explain = cx
        .app
        .models()
        .read(&st.semantics_selected_hit_test_explain_json, |v| v.clone())
        .unwrap_or_default();
    let hit_test_explain_summary = cx
        .app
        .models()
        .read(&st.semantics_selected_hit_test_explain_summary, |v| {
            v.clone()
        })
        .unwrap_or_default();
    let hit_test_explain_status = cx
        .app
        .models()
        .read(&st.semantics_selected_hit_test_explain_status, |v| {
            v.clone()
        })
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("unknown"));
    let hit_test_explain_updated = cx
        .app
        .models()
        .read(
            &st.semantics_selected_hit_test_explain_updated_unix_ms,
            |v| *v,
        )
        .ok()
        .flatten();
    let live_enabled = cx
        .app
        .models()
        .read(&st.semantics_live_enabled, |v| *v)
        .unwrap_or(true);
    let selected_id = cx
        .app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let index = cx
        .app
        .models()
        .read(&st.semantics_cache, |v| v.clone())
        .ok()
        .flatten();

    let status_line = {
        let mut line = format!(
            "live_enabled={live_enabled} status={}",
            live_status.as_ref()
        );
        if let Some(ts) = live_updated {
            line.push_str(&format!(" updated_unix_ms={ts}"));
        }
        line
    };

    let hit_test_explain_status_line = {
        let mut line = format!(
            "hit_test.explain status={}",
            hit_test_explain_status.as_ref()
        );
        if let Some(ts) = hit_test_explain_updated {
            line.push_str(&format!(" updated_unix_ms={ts}"));
        }
        line
    };

    let live_toggle_label = if live_enabled {
        "Live: On"
    } else {
        "Live: Off"
    };
    let live_enabled_model = st.semantics_live_enabled.clone();
    let force_nonce_model = st.semantics_live_force_nonce.clone();
    let on_toggle: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&live_enabled_model, |v| *v = !*v);
        let _ = host
            .models_mut()
            .update(&force_nonce_model, |v| *v = v.saturating_add(1));
        host.request_redraw(action_cx.window);
    });

    let on_refresh: fret_ui::action::OnActivate = {
        let force_nonce_model = st.semantics_live_force_nonce.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&force_nonce_model, |v| *v = v.saturating_add(1));
            host.request_redraw(action_cx.window);
        })
    };

    let live_toggle_btn = shadcn::Button::new(live_toggle_label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(on_toggle)
        .into_element(cx);
    let refresh_btn = shadcn::Button::new("Refresh")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(on_refresh)
        .into_element(cx);
    let status_elem = cx.text(status_line);

    let header = ui::h_row(|_cx| [live_toggle_btn, refresh_btn, status_elem])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);

    let mut child_buttons: Vec<AnyElement> = Vec::new();
    child_buttons.reserve(children.len().min(64));
    if let (Some(index), Some(_selected)) = (index, selected_id) {
        for child in children.iter().take(200) {
            let id = *child;
            let label = index
                .node(id)
                .map(semantics::node_label)
                .unwrap_or_else(|| format!("id={id}"));

            let selected_id_model = st.semantics_selected_id.clone();
            let selected_json_model = st.semantics_selected_node_json.clone();
            let selected_live_json_model = st.semantics_selected_node_live_json.clone();
            let selected_live_status_model = st.semantics_selected_node_live_status.clone();
            let selected_live_updated_model =
                st.semantics_selected_node_live_updated_unix_ms.clone();
            let selected_live_children_model = st.semantics_selected_node_live_children.clone();
            let selected_hit_test_explain_json_model =
                st.semantics_selected_hit_test_explain_json.clone();
            let selected_hit_test_explain_summary_model =
                st.semantics_selected_hit_test_explain_summary.clone();
            let selected_hit_test_explain_status_model =
                st.semantics_selected_hit_test_explain_status.clone();
            let selected_hit_test_explain_updated_model = st
                .semantics_selected_hit_test_explain_updated_unix_ms
                .clone();
            let index_for_select = Arc::clone(&index);
            let on_child: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&selected_id_model, |v| *v = Some(id));
                    let text = semantics::selected_node_json(index_for_select.as_ref(), Some(id));
                    let _ = host
                        .models_mut()
                        .update(&selected_json_model, |v| *v = text);
                    let _ = host
                        .models_mut()
                        .update(&selected_live_json_model, |v| v.clear());
                    let _ = host.models_mut().update(&selected_live_status_model, |v| {
                        *v = None;
                    });
                    let _ = host
                        .models_mut()
                        .update(&selected_live_updated_model, |v| *v = None);
                    let _ = host
                        .models_mut()
                        .update(&selected_live_children_model, |v| v.clear());
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_json_model, |v| v.clear());
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_summary_model, |v| v.clear());
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_status_model, |v| *v = None);
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_updated_model, |v| *v = None);
                    host.request_redraw(action_cx.window);
                });

            child_buttons.push(
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_child)
                    .into_element(cx),
            );
        }
    }

    let children_panel = if child_buttons.is_empty() {
        cx.text("children: <none>")
    } else {
        shadcn::ScrollArea::new([ui::v_stack(|_cx| child_buttons)
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx)])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .h_px(Px(160.0)),
        )
        .into_element(cx)
    };

    let json_text = if !live.is_empty() { live } else { fallback };
    let live_body_title = cx.text("Live semantics JSON");
    let live_body_content = text_blob(cx, json_text);
    let live_body = ui::v_stack(|_cx| [live_body_title, live_body_content])
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let hit_test_explain_summary_body = if hit_test_explain_summary.is_empty() {
        cx.text("<no summary yet>")
    } else {
        text_blob(cx, hit_test_explain_summary)
    };
    let hit_test_explain_body = if hit_test_explain.is_empty() {
        cx.text("<no hit_test.explain_ack yet>")
    } else {
        text_blob(cx, hit_test_explain)
    };
    let hit_test_explain_status_text = cx.text(hit_test_explain_status_line);
    let hit_test_explain_summary_title = cx.text("Readable summary");
    let hit_test_explain_panel = ui::v_stack(|_cx| {
        [
            hit_test_explain_status_text,
            hit_test_explain_summary_title,
            hit_test_explain_summary_body,
            hit_test_explain_body,
        ]
    })
    .gap(fret_ui_kit::Space::N1)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    ui::v_stack(|_cx| [header, children_panel, live_body, hit_test_explain_panel])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
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
            if let Err(err) = validate_script_json_value(&script_value) {
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

fn script_steps_len(script_text: &str) -> Option<usize> {
    let v: serde_json::Value = serde_json::from_str(script_text).ok()?;
    v.get("steps").and_then(|v| v.as_array()).map(|a| a.len())
}

fn script_summary_line(script_text: &str) -> (String, bool) {
    let v: serde_json::Value = match serde_json::from_str(script_text) {
        Ok(v) => v,
        Err(err) => return (format!("parse_error: {err}"), false),
    };

    let schema = match validate_script_json_value(&v) {
        Ok(schema) => schema,
        Err(err) => return (format!("invalid: {err}"), false),
    };

    let steps = v.get("steps").and_then(|v| v.as_array()).map(|a| a.len());
    let steps = steps
        .map(|n| n.to_string())
        .unwrap_or_else(|| "<missing>".to_string());
    (format!("ok schema_version={schema} steps={steps}"), true)
}

fn validate_script_json_value(script: &serde_json::Value) -> Result<u32, String> {
    let schema_version = script
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing schema_version".to_string())?;
    let schema_version = schema_version.min(u32::MAX as u64) as u32;

    match schema_version {
        1 => {
            let parsed: UiActionScriptV1 =
                serde_json::from_value(script.clone()).map_err(|e| e.to_string())?;
            if parsed.schema_version != 1 {
                return Err("schema_version mismatch".to_string());
            }
            Ok(1)
        }
        2 => {
            let parsed: UiActionScriptV2 =
                serde_json::from_value(script.clone()).map_err(|e| e.to_string())?;
            if parsed.schema_version != 2 {
                return Err("schema_version mismatch".to_string());
            }
            Ok(2)
        }
        other => Err(format!("unsupported schema_version: {other}")),
    }
}

#[derive(Clone)]
struct StepTemplate {
    label: &'static str,
    step: serde_json::Value,
}

fn infer_script_schema_version(script_text: &str) -> Option<u32> {
    let v: serde_json::Value = serde_json::from_str(script_text).ok()?;
    let schema = v.get("schema_version").and_then(|v| v.as_u64())?;
    Some(schema.min(u32::MAX as u64) as u32)
}

fn placeholder_selector_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "test_id",
        "id": "TODO",
    })
}

fn placeholder_predicate_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "exists",
        "target": placeholder_selector_value(),
    })
}

fn primary_pointer_suffix_for_step_json(step: &serde_json::Value) -> Option<&'static str> {
    let obj = step.as_object()?;
    let has = |k: &str| obj.contains_key(k);
    if has("target") {
        return Some("/target");
    }
    if has("predicate") {
        return Some("/predicate");
    }
    if has("container") {
        return Some("/container");
    }
    if has("from") {
        return Some("/from");
    }
    if has("to") {
        return Some("/to");
    }
    if has("menu") {
        return Some("/menu");
    }
    if has("item") {
        return Some("/item");
    }
    if has("path") {
        return Some("/path/0");
    }
    None
}

fn step_templates_for_schema(schema_version: u32) -> Vec<StepTemplate> {
    let target = placeholder_selector_value();
    let predicate = placeholder_predicate_value();

    let v1 = vec![
        StepTemplate {
            label: "click",
            step: serde_json::json!({
                "type": "click",
                "target": target.clone(),
                "button": "left",
            }),
        },
        StepTemplate {
            label: "drag_pointer",
            step: serde_json::json!({
                "type": "drag_pointer",
                "target": placeholder_selector_value(),
                "button": "left",
                "delta_x": 120.0,
                "delta_y": 0.0,
                "steps": 8,
            }),
        },
        StepTemplate {
            label: "move_pointer",
            step: serde_json::json!({
                "type": "move_pointer",
                "target": placeholder_selector_value(),
            }),
        },
        StepTemplate {
            label: "wheel",
            step: serde_json::json!({
                "type": "wheel",
                "target": placeholder_selector_value(),
                "delta_x": 0.0,
                "delta_y": -120.0,
            }),
        },
        StepTemplate {
            label: "press_key",
            step: serde_json::json!({
                "type": "press_key",
                "key": "Enter",
                "modifiers": { "shift": false, "ctrl": false, "alt": false, "meta": false },
                "repeat": false,
            }),
        },
        StepTemplate {
            label: "type_text",
            step: serde_json::json!({
                "type": "type_text",
                "text": "TODO",
            }),
        },
        StepTemplate {
            label: "wait_frames",
            step: serde_json::json!({
                "type": "wait_frames",
                "n": 30,
            }),
        },
        StepTemplate {
            label: "wait_until",
            step: serde_json::json!({
                "type": "wait_until",
                "predicate": predicate.clone(),
                "timeout_frames": 180,
            }),
        },
        StepTemplate {
            label: "assert",
            step: serde_json::json!({
                "type": "assert",
                "predicate": placeholder_predicate_value(),
            }),
        },
        StepTemplate {
            label: "capture_bundle",
            step: serde_json::json!({
                "type": "capture_bundle",
                "label": "devtools",
            }),
        },
        StepTemplate {
            label: "capture_screenshot",
            step: serde_json::json!({
                "type": "capture_screenshot",
                "label": "devtools",
                "timeout_frames": 300,
            }),
        },
        StepTemplate {
            label: "reset_diagnostics",
            step: serde_json::json!({
                "type": "reset_diagnostics",
            }),
        },
    ];

    if schema_version <= 1 {
        return v1;
    }

    let window = serde_json::json!({ "kind": "current" });

    let mut v2 = Vec::new();
    v2.push(StepTemplate {
        label: "click",
        step: serde_json::json!({
            "type": "click",
            "window": window.clone(),
            "target": target.clone(),
            "button": "left",
        }),
    });
    v2.push(StepTemplate {
        label: "drag_pointer",
        step: serde_json::json!({
            "type": "drag_pointer",
            "window": window.clone(),
            "target": placeholder_selector_value(),
            "button": "left",
            "delta_x": 120.0,
            "delta_y": 0.0,
            "steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "pointer_down",
        step: serde_json::json!({
            "type": "pointer_down",
            "window": window.clone(),
            "target": placeholder_selector_value(),
            "button": "left",
        }),
    });
    v2.push(StepTemplate {
        label: "pointer_move",
        step: serde_json::json!({
            "type": "pointer_move",
            "window": window.clone(),
            "delta_x": 120.0,
            "delta_y": 0.0,
            "steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "pointer_up",
        step: serde_json::json!({
            "type": "pointer_up",
            "window": window.clone(),
        }),
    });
    v2.push(StepTemplate {
        label: "move_pointer",
        step: serde_json::json!({
            "type": "move_pointer",
            "target": placeholder_selector_value(),
        }),
    });
    v2.push(StepTemplate {
        label: "wheel",
        step: serde_json::json!({
            "type": "wheel",
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": -120.0,
        }),
    });
    v2.push(StepTemplate {
        label: "press_key",
        step: serde_json::json!({
            "type": "press_key",
            "key": "Enter",
            "modifiers": { "shift": false, "ctrl": false, "alt": false, "meta": false },
            "repeat": false,
        }),
    });
    v2.push(StepTemplate {
        label: "type_text",
        step: serde_json::json!({
            "type": "type_text",
            "text": "TODO",
        }),
    });
    v2.push(StepTemplate {
        label: "wait_frames",
        step: serde_json::json!({
            "type": "wait_frames",
            "n": 30,
        }),
    });
    v2.push(StepTemplate {
        label: "wait_until",
        step: serde_json::json!({
            "type": "wait_until",
            "window": window.clone(),
            "predicate": predicate.clone(),
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "assert",
        step: serde_json::json!({
            "type": "assert",
            "window": window.clone(),
            "predicate": placeholder_predicate_value(),
        }),
    });
    v2.push(StepTemplate {
        label: "capture_bundle",
        step: serde_json::json!({
            "type": "capture_bundle",
            "label": "devtools",
        }),
    });
    v2.push(StepTemplate {
        label: "capture_screenshot",
        step: serde_json::json!({
            "type": "capture_screenshot",
            "label": "devtools",
            "timeout_frames": 300,
        }),
    });
    v2.push(StepTemplate {
        label: "reset_diagnostics",
        step: serde_json::json!({
            "type": "reset_diagnostics",
        }),
    });

    v2.push(StepTemplate {
        label: "press_shortcut",
        step: serde_json::json!({
            "type": "press_shortcut",
            "shortcut": "Ctrl+P",
            "repeat": false,
        }),
    });
    v2.push(StepTemplate {
        label: "move_pointer_sweep",
        step: serde_json::json!({
            "type": "move_pointer_sweep",
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": 40.0,
            "steps": 8,
            "frames_per_step": 1,
        }),
    });
    v2.push(StepTemplate {
        label: "click_stable",
        step: serde_json::json!({
            "type": "click_stable",
            "target": placeholder_selector_value(),
            "button": "left",
            "stable_frames": 2,
            "max_move_px": 1.0,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "ensure_visible",
        step: serde_json::json!({
            "type": "ensure_visible",
            "target": placeholder_selector_value(),
            "within_window": true,
            "padding_px": 0.0,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "scroll_into_view",
        step: serde_json::json!({
            "type": "scroll_into_view",
            "container": placeholder_selector_value(),
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": -120.0,
            "require_fully_within_container": false,
            "require_fully_within_window": false,
            "padding_px": 0.0,
            "padding_insets_px": null,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "type_text_into",
        step: serde_json::json!({
            "type": "type_text_into",
            "target": placeholder_selector_value(),
            "text": "TODO",
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "menu_select",
        step: serde_json::json!({
            "type": "menu_select",
            "menu": placeholder_selector_value(),
            "item": placeholder_selector_value(),
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "menu_select_path",
        step: serde_json::json!({
            "type": "menu_select_path",
            "path": [placeholder_selector_value()],
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "drag_to",
        step: serde_json::json!({
            "type": "drag_to",
            "window": window.clone(),
            "from": placeholder_selector_value(),
            "to": placeholder_selector_value(),
            "button": "left",
            "steps": 8,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "set_slider_value",
        step: serde_json::json!({
            "type": "set_slider_value",
            "target": placeholder_selector_value(),
            "value": 50.0,
            "min": 0.0,
            "max": 100.0,
            "epsilon": 0.5,
            "timeout_frames": 180,
            "drag_steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "set_window_inner_size",
        step: serde_json::json!({
            "type": "set_window_inner_size",
            "window": window.clone(),
            "width_px": 1280.0,
            "height_px": 720.0,
        }),
    });
    v2.push(StepTemplate {
        label: "set_window_outer_position",
        step: serde_json::json!({
            "type": "set_window_outer_position",
            "window": window.clone(),
            "x_px": 100.0,
            "y_px": 100.0
        }),
    });
    v2.push(StepTemplate {
        label: "set_cursor_at_host_monitor",
        step: serde_json::json!({
            "type": "set_cursor_at_host_monitor",
            "selector": "highest_scale_factor",
            "x_fraction": 0.5,
            "y_fraction": 0.5,
            "offset_x_px": 0.0,
            "offset_y_px": 0.0
        }),
    });
    v2.push(StepTemplate {
        label: "set_cursor_in_window",
        step: serde_json::json!({
            "type": "set_cursor_in_window",
            "window": window.clone(),
            "x_px": 200.0,
            "y_px": 200.0,
        }),
    });
    v2.push(StepTemplate {
        label: "raise_window",
        step: serde_json::json!({
            "type": "raise_window",
            "window": window.clone(),
        }),
    });

    v2
}

fn selector_fields(cx: &mut ElementContext<'_, App>, st: &State, kind: &str) -> AnyElement {
    let test_id = shadcn::Input::new(st.script_selector_test_id.clone())
        .a11y_label("test_id")
        .placeholder("button.ok")
        .into_element(cx);
    let role = shadcn::Input::new(st.script_selector_role.clone())
        .a11y_label("role")
        .placeholder("button")
        .into_element(cx);
    let name = shadcn::Input::new(st.script_selector_name.clone())
        .a11y_label("name")
        .placeholder("OK")
        .into_element(cx);
    let ancestors = shadcn::Textarea::new(st.script_selector_ancestors.clone())
        .a11y_label("ancestors (role:name per line)")
        .min_height(Px(120.0))
        .into_element(cx);
    let node_id = shadcn::Input::new(st.script_selector_node_id.clone())
        .a11y_label("node_id")
        .placeholder("123")
        .into_element(cx);
    let element_id = shadcn::Input::new(st.script_selector_element_id.clone())
        .a11y_label("global_element_id")
        .placeholder("123")
        .into_element(cx);

    match kind {
        "test_id" => ui::v_stack(|_cx| [test_id])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "role_and_name" => ui::v_stack(|_cx| [role, name])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "role_and_path" => ui::v_stack(|_cx| [role, name, ancestors])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "node_id" => ui::v_stack(|_cx| [node_id])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "global_element_id" => ui::v_stack(|_cx| [element_id])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        _ => cx.text("unknown selector kind"),
    }
}

fn selector_value_from_models(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    kind: &str,
) -> serde_json::Value {
    let test_id = cx
        .app
        .models()
        .read(&st.script_selector_test_id, |v| v.clone())
        .unwrap_or_default();
    let role = cx
        .app
        .models()
        .read(&st.script_selector_role, |v| v.clone())
        .unwrap_or_default();
    let name = cx
        .app
        .models()
        .read(&st.script_selector_name, |v| v.clone())
        .unwrap_or_default();
    let ancestors_text = cx
        .app
        .models()
        .read(&st.script_selector_ancestors, |v| v.clone())
        .unwrap_or_default();
    let node_id = cx
        .app
        .models()
        .read(&st.script_selector_node_id, |v| v.clone())
        .unwrap_or_default();
    let element_id = cx
        .app
        .models()
        .read(&st.script_selector_element_id, |v| v.clone())
        .unwrap_or_default();

    match kind {
        "test_id" => serde_json::json!({
            "kind": "test_id",
            "id": test_id.trim(),
        }),
        "role_and_name" => serde_json::json!({
            "kind": "role_and_name",
            "role": role.trim(),
            "name": name.trim(),
        }),
        "role_and_path" => serde_json::json!({
            "kind": "role_and_path",
            "role": role.trim(),
            "name": name.trim(),
            "ancestors": parse_ancestors_lines(&ancestors_text),
        }),
        "node_id" => serde_json::json!({
            "kind": "node_id",
            "node": node_id.trim().parse::<u64>().unwrap_or(0),
        }),
        "global_element_id" => serde_json::json!({
            "kind": "global_element_id",
            "element": element_id.trim().parse::<u64>().unwrap_or(0),
        }),
        _ => placeholder_selector_value(),
    }
}

fn predicate_fields(cx: &mut ElementContext<'_, App>, st: &State, kind: &str) -> AnyElement {
    let role = shadcn::Input::new(st.script_predicate_role.clone())
        .a11y_label("role")
        .placeholder("button")
        .into_element(cx);
    let checked = shadcn::Checkbox::new(st.script_predicate_checked.clone())
        .a11y_label("checked")
        .into_element(cx);
    let len_bytes = shadcn::Input::new(st.script_predicate_len_bytes.clone())
        .a11y_label("len_bytes")
        .placeholder("0")
        .into_element(cx);
    let min_len_bytes = shadcn::Input::new(st.script_predicate_len_bytes.clone())
        .a11y_label("min_len_bytes")
        .placeholder("0")
        .into_element(cx);
    let padding = shadcn::Input::new(st.script_predicate_padding_px.clone())
        .a11y_label("padding_px")
        .placeholder("0")
        .into_element(cx);
    let eps = shadcn::Input::new(st.script_predicate_eps_px.clone())
        .a11y_label("eps_px")
        .placeholder("0")
        .into_element(cx);
    let min_w = shadcn::Input::new(st.script_predicate_min_w_px.clone())
        .a11y_label("min_w_px")
        .placeholder("0")
        .into_element(cx);
    let min_h = shadcn::Input::new(st.script_predicate_min_h_px.clone())
        .a11y_label("min_h_px")
        .placeholder("0")
        .into_element(cx);

    match kind {
        "role_is" => role,
        "checked_is" => checked,
        "label_len_is" | "value_len_is" => len_bytes,
        "label_len_ge" | "value_len_ge" => min_len_bytes,
        "barrier_roots" => {
            let barrier_root_items = [
                shadcn::SelectItem::new("any", "any"),
                shadcn::SelectItem::new("none", "none"),
                shadcn::SelectItem::new("some", "some"),
            ];
            let focus_root_items = [
                shadcn::SelectItem::new("any", "any"),
                shadcn::SelectItem::new("none", "none"),
                shadcn::SelectItem::new("some", "some"),
            ];
            let require_items = [
                shadcn::SelectItem::new("unset", "unset"),
                shadcn::SelectItem::new("true", "true"),
                shadcn::SelectItem::new("false", "false"),
            ];

            let barrier_root = shadcn::Select::new(
                st.script_predicate_barrier_root.clone(),
                st.script_predicate_barrier_root_open.clone(),
            )
            .value(shadcn::SelectValue::new().placeholder("barrier_root"))
            .items(barrier_root_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let focus_root = shadcn::Select::new(
                st.script_predicate_focus_barrier_root.clone(),
                st.script_predicate_focus_barrier_root_open.clone(),
            )
            .value(shadcn::SelectValue::new().placeholder("focus_barrier_root"))
            .items(focus_root_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let require_equal = shadcn::Select::new(
                st.script_predicate_require_equal.clone(),
                st.script_predicate_require_equal_open.clone(),
            )
            .value(shadcn::SelectValue::new().placeholder("require_equal"))
            .items(require_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let other_selector =
                shadcn::Textarea::new(st.script_predicate_other_selector_json.clone())
                    .a11y_label("other selector (optional)")
                    .min_height(Px(96.0))
                    .into_element(cx);

            ui::v_stack(|_cx| [barrier_root, focus_root, require_equal, other_selector])
                .gap(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx)
        }
        "bounds_within_window" => ui::v_stack(|_cx| [padding, eps])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "bounds_min_size" => ui::v_stack(|_cx| [min_w, min_h, eps])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "bounds_non_overlapping"
        | "bounds_overlapping"
        | "bounds_overlapping_x"
        | "bounds_overlapping_y" => {
            let other_selector =
                shadcn::Textarea::new(st.script_predicate_other_selector_json.clone())
                    .a11y_label("selector B (JSON)")
                    .min_height(Px(120.0))
                    .into_element(cx);
            ui::v_stack(|_cx| [eps, other_selector])
                .gap(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx)
        }
        _ => cx.text(""),
    }
}

fn predicate_value_from_models(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    kind: &str,
    selector: serde_json::Value,
) -> serde_json::Value {
    let role = cx
        .app
        .models()
        .read(&st.script_predicate_role, |v| v.clone())
        .unwrap_or_default();
    let other_selector_json = cx
        .app
        .models()
        .read(&st.script_predicate_other_selector_json, |v| v.clone())
        .unwrap_or_default();
    let checked = cx
        .app
        .models()
        .read(&st.script_predicate_checked, |v| *v)
        .unwrap_or(false);
    let len_bytes = parse_u32_model(cx, &st.script_predicate_len_bytes);
    let padding_px = parse_f32_model(cx, &st.script_predicate_padding_px);
    let eps_px = parse_f32_model(cx, &st.script_predicate_eps_px);
    let min_w_px = parse_f32_model(cx, &st.script_predicate_min_w_px);
    let min_h_px = parse_f32_model(cx, &st.script_predicate_min_h_px);

    let other_selector = serde_json::from_str::<serde_json::Value>(&other_selector_json)
        .ok()
        .unwrap_or_else(placeholder_selector_value);

    match kind {
        "exists" => serde_json::json!({
            "kind": "exists",
            "target": selector,
        }),
        "not_exists" => serde_json::json!({
            "kind": "not_exists",
            "target": selector,
        }),
        "focus_is" => serde_json::json!({
            "kind": "focus_is",
            "target": selector,
        }),
        "role_is" => serde_json::json!({
            "kind": "role_is",
            "target": selector,
            "role": role.trim(),
        }),
        "checked_is" => serde_json::json!({
            "kind": "checked_is",
            "target": selector,
            "checked": checked,
        }),
        "checked_is_none" => serde_json::json!({
            "kind": "checked_is_none",
            "target": selector,
        }),
        "label_len_is" => serde_json::json!({
            "kind": "label_len_is",
            "target": selector,
            "len_bytes": len_bytes,
        }),
        "label_len_ge" => serde_json::json!({
            "kind": "label_len_ge",
            "target": selector,
            "min_len_bytes": len_bytes,
        }),
        "value_len_is" => serde_json::json!({
            "kind": "value_len_is",
            "target": selector,
            "len_bytes": len_bytes,
        }),
        "value_len_ge" => serde_json::json!({
            "kind": "value_len_ge",
            "target": selector,
            "min_len_bytes": len_bytes,
        }),
        "barrier_roots" => {
            let barrier_root = cx
                .app
                .models()
                .read(&st.script_predicate_barrier_root, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("any"));
            let focus_barrier_root = cx
                .app
                .models()
                .read(&st.script_predicate_focus_barrier_root, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("any"));
            let require_equal = cx
                .app
                .models()
                .read(&st.script_predicate_require_equal, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("unset"));

            let mut obj = serde_json::Map::new();
            obj.insert(
                "kind".to_string(),
                serde_json::Value::String("barrier_roots".to_string()),
            );
            obj.insert(
                "barrier_root".to_string(),
                serde_json::Value::String(barrier_root.to_string()),
            );
            obj.insert(
                "focus_barrier_root".to_string(),
                serde_json::Value::String(focus_barrier_root.to_string()),
            );
            if require_equal.as_ref() == "true" {
                obj.insert("require_equal".to_string(), serde_json::Value::Bool(true));
            } else if require_equal.as_ref() == "false" {
                obj.insert("require_equal".to_string(), serde_json::Value::Bool(false));
            }
            serde_json::Value::Object(obj)
        }
        "visible_in_window" => serde_json::json!({
            "kind": "visible_in_window",
            "target": selector,
        }),
        "bounds_within_window" => serde_json::json!({
            "kind": "bounds_within_window",
            "target": selector,
            "padding_px": padding_px,
            "eps_px": eps_px,
        }),
        "bounds_min_size" => serde_json::json!({
            "kind": "bounds_min_size",
            "target": selector,
            "min_w_px": min_w_px,
            "min_h_px": min_h_px,
            "eps_px": eps_px,
        }),
        "bounds_non_overlapping" => serde_json::json!({
            "kind": "bounds_non_overlapping",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping" => serde_json::json!({
            "kind": "bounds_overlapping",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping_x" => serde_json::json!({
            "kind": "bounds_overlapping_x",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping_y" => serde_json::json!({
            "kind": "bounds_overlapping_y",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        _ => placeholder_predicate_value(),
    }
}

fn parse_f32_model(cx: &mut ElementContext<'_, App>, m: &Model<String>) -> f32 {
    cx.app
        .models()
        .read(m, |v| v.trim().parse::<f32>().ok())
        .ok()
        .flatten()
        .unwrap_or(0.0)
}

fn parse_u32_model(cx: &mut ElementContext<'_, App>, m: &Model<String>) -> u32 {
    cx.app
        .models()
        .read(m, |v| v.trim().parse::<u32>().ok())
        .ok()
        .flatten()
        .unwrap_or(0)
}

fn parse_ancestors_lines(text: &str) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((role, name)) = line.split_once(':') else {
            continue;
        };
        let role = role.trim();
        let name = name.trim();
        if role.is_empty() || name.is_empty() {
            continue;
        }
        out.push(serde_json::json!({
            "role": role,
            "name": name,
        }));
    }
    out
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

fn devtools_first_open_next_action_lines(
    has_session: bool,
    session_count: usize,
    selected_session_id: Option<&str>,
    scripts_count: usize,
    regression_loaded: bool,
    selected_summary_loaded: bool,
    selected_followup_result_loaded: bool,
    failing_count: usize,
    artifacts_root: &str,
    recent_failed_evidence: Option<&RecentEvidenceTarget>,
    recent_failed_evidence_rerunnable_kind: Option<&'static str>,
    recent_failed_evidence_rerun_unavailable_reason: Option<&str>,
    recent_evidence_next_action: &str,
) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    let target = if has_session {
        "target: session selected; inspect, pick, dump, and screenshot actions are ready".to_string()
    } else if session_count > 0 {
        format!("target: {session_count} session(s) available; select one before inspecting")
    } else {
        "target: no session yet; launch a Fret app with diagnostics enabled".to_string()
    };
    let session_scope = if has_session {
        let selected = selected_session_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("<selected>");
        if session_count > 1 {
            format!(
                "session scope: selected {selected}; {session_count} sessions connected, use the Session selector to retarget inspect, bundle, screenshot, and selected-session suite actions"
            )
        } else {
            format!(
                "session scope: selected {selected}; actions target the current diagnostics session"
            )
        }
    } else if session_count > 0 {
        "session scope: choose one available session before sending inspect, bundle, screenshot, or selected-session suite actions".to_string()
    } else {
        "session scope: waiting for the first diagnostics session".to_string()
    };
    let regression = if regression_loaded {
        format!("regression: aggregate loaded with {failing_count} non-passing summary row(s)")
    } else if selected_summary_loaded && selected_followup_result_loaded {
        "regression: selected follow-up result loaded; inspect Follow-up Result Summary/History"
            .to_string()
    } else if selected_summary_loaded {
        "regression: selected summary loaded; follow-up actions can use selected bundle evidence"
            .to_string()
    } else {
        "regression: no aggregate loaded; run a script, then Refresh or Summarize".to_string()
    };
    let recent_evidence = recent_failed_evidence
        .map(|target| {
            format!(
                "recent evidence: failed {} {} ({})",
                target.kind,
                target.id,
                short_artifact_result_path(&target.result_path)
            )
        })
        .unwrap_or_else(|| "recent evidence: no failed restored GUI-launched evidence".to_string());
    let recent_command = recent_failed_evidence
        .map(|target| format!("recent evidence command: {}", target.command_line))
        .unwrap_or_else(|| {
            "recent evidence command: <none; run a workflow or generated gate>".to_string()
        });
    let recent_rerun = match (recent_failed_evidence, recent_failed_evidence_rerunnable_kind) {
        (Some(_), Some(kind)) => format!("recent evidence rerun: available ({kind})"),
        (Some(_), None) => recent_failed_evidence_rerun_unavailable_reason
            .map(|reason| format!("recent evidence rerun: unavailable ({reason})"))
            .unwrap_or_else(|| "recent evidence rerun: unavailable".to_string()),
        (None, _) => "recent evidence rerun: <none>".to_string(),
    };
    vec![
        target,
        session_scope,
        format!("scripts: {scripts_count} available in Script Studio"),
        regression,
        recent_evidence,
        recent_command,
        recent_rerun,
        format!("recent evidence next: {recent_evidence_next_action}"),
        format!("artifacts root: {artifacts_root}"),
        "guide: open Evidence & Results -> Guide for docs, dogfood, workflow runs, demo/metrics, and gate commands".to_string(),
    ]
}

fn devtools_first_open_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    vec![
        format!("first-open: {DEVTOOLS_FIRST_OPEN_DOC}"),
        format!("gui branch: {DEVTOOLS_GUI_BRANCH_DOC}"),
        format!("repo preflight: {DEVTOOLS_REPO_PREFLIGHT_COMMAND}"),
        format!("repo preflight json: {DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND}"),
        format!("artifacts root: {artifacts_root}"),
        "direct loop: diag run -> diag latest -> diag compare".to_string(),
        format!(
            "campaign loop: diag campaign run {DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID} -> diag summarize -> diag dashboard"
        ),
        format!("gate: {DEVTOOLS_FIRST_OPEN_GATE_COMMAND}"),
        format!("product workflow: {IMUI_PRODUCT_WORKFLOW_ID}"),
        format!("product workflow command: {IMUI_PRODUCT_WORKFLOW_COMMAND}"),
        format!("product workflow focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}"),
        format!("product workflow launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}"),
        format!("product workflow suite: {IMUI_PRODUCT_WORKFLOW_SUITE}"),
        format!("product workflow docs: {IMUI_PRODUCT_WORKFLOW_DOC}"),
        format!(
            "product workflow artifacts: {}",
            IMUI_PRODUCT_WORKFLOW_ARTIFACTS.join(", ")
        ),
    ]
}

fn devtools_dogfood_workflow_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    vec![
        format!("dogfood workflow: {DEVTOOLS_DOGFOOD_WORKFLOW_ID}"),
        format!("dogfood docs: {DEVTOOLS_GUI_BRANCH_DOC}"),
        format!("artifacts root: {artifacts_root}"),
        format!("open ui gallery: {DEVTOOLS_DOGFOOD_TARGET_COMMAND}"),
        "pick target: enable inspect -> Pick -> click a Button page control".to_string(),
        "preferred selector: {\"kind\":\"test_id\",\"id\":\"ui-gallery-nav-button\"}".to_string(),
        format!("base script: {DEVTOOLS_DOGFOOD_BASE_SCRIPT}"),
        format!("button script: {DEVTOOLS_DOGFOOD_BUTTON_SCRIPT}"),
        format!("generate script from pick: {DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND}"),
        format!("apply pick to script: {DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND}"),
        format!("run and pack: {DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND}"),
        format!("pack selected bundle: {DEVTOOLS_DOGFOOD_PACK_COMMAND}"),
        format!("open viewer: {DEVTOOLS_DOGFOOD_VIEWER_COMMAND}"),
        "viewer input: drag bundle.json, bundle.schema2.json, or the packed zip into the offline viewer"
            .to_string(),
    ]
}

fn devtools_workflow_run_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    vec![
        format!("workflow route: {DEVTOOLS_WORKFLOW_ROUTE_ID}"),
        format!("artifacts root: {artifacts_root}"),
        format!("result artifacts: .fret/diag/workflow-runs/*.json"),
        "handoff: load suite regression.summary.json into Regression Workspace".to_string(),
        "handoff: run workflow summarize to create regression.index.json when missing".to_string(),
        format!(
            "campaign validate: cargo run -p fretboard-dev -- diag campaign validate {DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST} --json"
        ),
        format!(
            "imui p3 validate: cargo run -p fretboard-dev -- diag campaign validate {DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST} --json"
        ),
        format!(
            "suite ws: cargo run -p fretboard-dev -- diag suite {DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE} --dir {artifacts_root}/devtools-workflows/perf-docking --devtools-ws-url <devtools-ws-url> --devtools-token <redacted> --devtools-session-id <selected-session> --json"
        ),
    ]
}

fn devtools_workflow_commands(
    artifacts_root: &str,
    ws_url: &str,
    token: &str,
    selected_session_id: Option<&str>,
) -> Vec<workflow_run::DevtoolsWorkflowRunCommandV1> {
    let workflow_out_dir = workflow_run_artifacts_dir(artifacts_root);
    let selected_session = selected_session_id
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let selected_session_or_placeholder = selected_session.unwrap_or("<selected-session>");
    let mut suite_missing_inputs = Vec::new();
    if selected_session.is_none() {
        suite_missing_inputs.push("selected-session".to_string());
    }

    vec![
        workflow_campaign_validate_command(
            DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID,
            "Validate devtools first-open campaign",
            DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST,
        ),
        workflow_campaign_validate_command(
            DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID,
            "Validate IMUI P3 multi-window campaign",
            DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST,
        ),
        workflow_run::DevtoolsWorkflowRunCommandV1 {
            id: DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID.to_string(),
            label: "Run perf docking suite over selected session".to_string(),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag suite {DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE} --dir {workflow_out_dir} --devtools-ws-url {ws_url} --devtools-token <redacted> --devtools-session-id {selected_session_or_placeholder} --json"
            ),
            diag_args: selected_session
                .map(|session_id| {
                    vec![
                        "suite".to_string(),
                        DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE.to_string(),
                        "--dir".to_string(),
                        workflow_out_dir.clone(),
                        "--devtools-ws-url".to_string(),
                        ws_url.to_string(),
                        "--devtools-token".to_string(),
                        token.to_string(),
                        "--devtools-session-id".to_string(),
                        session_id.to_string(),
                        "--json".to_string(),
                    ]
                })
                .unwrap_or_default(),
            missing_inputs: suite_missing_inputs,
        },
    ]
}

fn workflow_campaign_validate_command(
    id: &str,
    label: &str,
    manifest: &str,
) -> workflow_run::DevtoolsWorkflowRunCommandV1 {
    workflow_run::DevtoolsWorkflowRunCommandV1 {
        id: id.to_string(),
        label: label.to_string(),
        command_line: format!(
            "cargo run -p fretboard-dev -- diag campaign validate {manifest} --json"
        ),
        diag_args: vec![
            "campaign".to_string(),
            "validate".to_string(),
            manifest.to_string(),
            "--json".to_string(),
        ],
        missing_inputs: Vec::new(),
    }
}

fn workflow_run_artifacts_dir(artifacts_root: &str) -> String {
    let root = artifacts_root.trim();
    let root = if root.is_empty() {
        "target/fret-diag"
    } else {
        root
    };
    format!(
        "{}/devtools-workflows/perf-docking",
        root.trim_end_matches(['/', '\\'])
    )
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

fn workflow_handoff_readiness_lines(
    workflow_run_in_flight: bool,
    selected_result_loaded: bool,
    regression_summary_path: Option<&str>,
    loaded_regression_summary_path: Option<&str>,
    aggregate_index_ready: bool,
    aggregate_index_loaded: bool,
) -> Vec<String> {
    let selected_state = if workflow_run_in_flight {
        "in_flight"
    } else if selected_result_loaded {
        "loaded"
    } else {
        "none"
    };
    let artifact = regression_summary_path
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let loaded = loaded_regression_summary_path
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let already_loaded = artifact
        .zip(loaded)
        .is_some_and(|(artifact, loaded)| {
            normalize_workflow_artifact_path(artifact) == normalize_workflow_artifact_path(loaded)
        });

    let mut lines = vec![
        format!("selected_workflow_result: {selected_state}"),
        format!(
            "regression_summary_artifact: {}",
            artifact.unwrap_or("<none>")
        ),
        format!(
            "aggregate_index_ready: {}",
            if aggregate_index_ready { "true" } else { "false" }
        ),
        format!(
            "aggregate_index_loaded: {}",
            if aggregate_index_loaded {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if already_loaded {
        lines.push("regression_workspace: selected summary loaded from workflow".to_string());
    } else if artifact.is_some() {
        lines.push("regression_workspace: not loaded from workflow".to_string());
    } else {
        lines.push("regression_workspace: <not applicable>".to_string());
    }
    if aggregate_index_loaded {
        lines.push("aggregate_workspace: workflow index loaded".to_string());
    } else if aggregate_index_ready {
        lines.push("aggregate_workspace: index ready but not loaded".to_string());
    } else {
        lines.push("aggregate_workspace: <not applicable>".to_string());
    }

    let next_action = if workflow_run_in_flight {
        "wait for workflow result artifact"
    } else if !selected_result_loaded {
        "run selected workflow"
    } else if artifact.is_some() && !aggregate_index_ready {
        "Run workflow summarize"
    } else if already_loaded {
        "use Regression Workspace follow-up actions"
    } else if artifact.is_some() {
        "Load workflow regression summary"
    } else {
        "selected workflow result has no regression.summary.json artifact"
    };
    lines.push(format!("next_action: {next_action}"));
    let aggregate_next_action = if workflow_run_in_flight {
        "wait for workflow result artifact"
    } else if !selected_result_loaded {
        "run selected workflow"
    } else if aggregate_index_loaded {
        "aggregate index already loaded"
    } else if aggregate_index_ready {
        "Load workflow regression index"
    } else if artifact.is_some() {
        "Run workflow summarize"
    } else {
        "selected workflow result has no regression.index.json artifact"
    };
    lines.push(format!("aggregate_next_action: {aggregate_next_action}"));
    lines
}

fn devtools_gate_command_lines(artifacts_root: &str) -> Vec<String> {
    devtools_gate_profile_lines(artifacts_root)
}

fn devtools_workflow_run_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let selected_workflow_id = cx
        .app
        .models()
        .read(&st.workflow_run_selected_id, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from(DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID));
    let commands = devtools_workflow_commands_from_state(cx.app, st);
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
    let workflow_items = commands
        .iter()
        .map(|command| shadcn::SelectItem::new(command.id.clone(), format!("{} ({})", command.label, command.id)))
        .collect::<Vec<_>>();
    let workflow_select = shadcn::Select::new(
        st.workflow_run_selected_id.clone(),
        st.workflow_run_selected_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("Workflow"))
    .items(workflow_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(340.0)))
    .into_element(cx);

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
    let workflow_run_in_flight = cx
        .app
        .models()
        .read(&st.workflow_run_in_flight, |v| *v)
        .unwrap_or(false);
    let workflow_run_result_path = cx
        .app
        .models()
        .read(&st.workflow_run_last_result_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|v| v.to_string());
    let workflow_run_error = cx
        .app
        .models()
        .read(&st.workflow_run_last_error, |v| v.clone())
        .ok()
        .flatten()
        .map(|v| v.to_string());
    let workflow_run_result_json = cx
        .app
        .models()
        .read(&st.workflow_run_last_result_json, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_result_history = cx
        .app
        .models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let workflow_run_selected_result_path = cx
        .app
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
    let selected_workflow_regression_summary_path = workflow_run::workflow_run_regression_summary_artifact_path_from_result_json(
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
    let loaded_regression_dir = cx
        .app
        .models()
        .read(&st.regression_loaded_dir, |v| v.clone())
        .ok()
        .flatten()
        .map(|path| path.to_string());
    let regression_index_loaded = cx
        .app
        .models()
        .read(&st.regression_index_json, |v| !v.trim().is_empty())
        .unwrap_or(false);
    let selected_workflow_aggregate_index_loaded = workflow_aggregate_index_loaded(
        selected_workflow_regression_index_resolved_path.as_deref(),
        loaded_regression_dir.as_deref(),
        regression_index_loaded,
    );
    let loaded_regression_summary_path = cx
        .app
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
    let workflow_result_actions = ui::h_row(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if selected_workflow_run_result_path.is_some() {
            out.push(
                shadcn::Button::new("Copy workflow result")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_RESULT_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open workflow JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_WORKFLOW_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if selected_workflow_run_result_entry.is_some() {
            out.push(
                shadcn::Button::new("Copy workflow command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_RESULT_COMMAND)
                    .into_element(cx),
            );
        }
        if !selected_workflow_run_result_json.trim().is_empty() {
            out.push(
                shadcn::Button::new("Copy workflow JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_RESULT_JSON)
                .into_element(cx),
            );
        }
        if selected_workflow_suite_summary_path.is_some() {
            out.push(
                shadcn::Button::new("Copy workflow suite summary")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open workflow suite summary")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_WORKFLOW_SUITE_SUMMARY)
                    .into_element(cx),
            );
        }
        if selected_workflow_regression_summary_path.is_some() {
            out.push(
                shadcn::Button::new("Copy workflow regression summary")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Load workflow regression summary")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open workflow regression summary")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Copy workflow summarize command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_SUMMARIZE_COMMAND)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Run workflow summarize")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(workflow_run_in_flight)
                    .on_click(CMD_RUN_WORKFLOW_SUMMARIZE)
                    .into_element(cx),
            );
        }
        if selected_workflow_regression_index_ready {
            out.push(
                shadcn::Button::new("Copy workflow regression index")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open workflow regression index")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_WORKFLOW_REGRESSION_INDEX)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Load workflow regression index")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_LOAD_WORKFLOW_REGRESSION_INDEX)
                    .into_element(cx),
            );
        }
        out
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let workflow_result_details = text_blob_sized(
        cx,
        workflow_run::workflow_run_result_history_entry_detail_lines(
            selected_workflow_run_result_entry.as_ref(),
        )
        .join("\n"),
        Px(78.0),
    );
    let workflow_result_summary = text_blob_sized(
        cx,
        workflow_run::workflow_run_result_summary_lines(&selected_workflow_run_result_json)
            .join("\n"),
        Px(92.0),
    );
    let workflow_handoff_readiness_blob = text_blob_sized(
        cx,
        workflow_handoff_readiness.join("\n"),
        Px(76.0),
    );
    let workflow_summarize_handoff_blob =
        text_blob_sized(cx, workflow_summarize_preview, Px(76.0));
    let workflow_result_history_summary = text_blob_sized(
        cx,
        workflow_run::workflow_run_result_history_summary_lines(&workflow_run_result_history)
            .join("\n"),
        Px(84.0),
    );
    let workflow_result_history = workflow_run_history_list(
        cx,
        &st.workflow_run_selected_result_path,
        &workflow_run_result_history,
        selected_workflow_run_result_path.as_deref(),
    );
    let workflow_run_status_line = format!(
        "workflow_run_in_flight={} last_workflow_result={} last_workflow_error={}",
        workflow_run_in_flight,
        workflow_run_result_path.as_deref().unwrap_or("-"),
        workflow_run_error.as_deref().unwrap_or("-")
    );
    let command_line_for_copy = command_preview.clone();
    let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let token = host.next_clipboard_token();
        host.push_effect(Effect::ClipboardWriteText {
            window: action_cx.window,
            token,
            text: command_line_for_copy.clone(),
        });
        host.request_redraw(action_cx.window);
    });
    let copy_button = shadcn::Button::new("Copy workflow command")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(selected_command.is_none())
        .on_activate(on_copy)
        .into_element(cx);
    let run_button = shadcn::Button::new("Run workflow")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!run_enabled || workflow_run_in_flight)
        .on_click(CMD_WORKFLOW_RUN_SELECTED)
        .into_element(cx);
    let controls = ui::h_row(|_cx| [workflow_select, copy_button, run_button])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let preview = text_blob_sized(cx, command_preview, Px(58.0));
    let result_preview = text_blob_sized(
        cx,
        if selected_workflow_run_result_json.trim().is_empty() {
            "<no workflow run result yet>".to_string()
        } else {
            selected_workflow_run_result_json
        },
        Px(92.0),
    );
    ui::v_stack(|cx| {
        [
            cx.text(format!("Runnable workflow: {selected_command_label}")),
            controls,
            cx.text(command_state_line),
            cx.text(workflow_run_status_line),
            preview,
            diag_section(
                cx,
                "Workflow Result Details",
                "Selected workflow run result status, path, command, and error preview.",
                vec![workflow_result_actions, workflow_result_details],
            ),
            diag_section(
                cx,
                "Workflow Result Summary",
                "Status, command, duration, and error preview from the selected workflow run result.",
                vec![workflow_result_summary],
            ),
            diag_section(
                cx,
                "Workflow Handoff Readiness",
                "A compact next-action summary links workflow artifacts to Regression Workspace.",
                vec![workflow_handoff_readiness_blob],
            ),
            diag_section(
                cx,
                "Workflow Summarize Handoff",
                "Run shared summarize over the suite regression summary to refresh aggregate index artifacts.",
                vec![workflow_summarize_handoff_blob],
            ),
            diag_section(
                cx,
                "Workflow Result History",
                "Select a GUI-launched workflow result, newest first.",
                vec![workflow_result_history_summary, workflow_result_history],
            ),
            result_preview,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn devtools_gate_profile_command_builder(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let selected_profile_id = cx
        .app
        .models()
        .read(&st.gate_profile_selected_id, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("stale-paint-scene"));
    let generated_command = generated_gate_command_from_state(cx.app, st);
    let command_preview = generated_command
        .as_ref()
        .map(|command| command.command_line.clone())
        .unwrap_or_else(|| "Select a script-target gate profile.".to_string());
    let selected_profile_label = devtools_gate_profiles_v1()
        .iter()
        .find(|profile| profile.id == selected_profile_id.as_ref())
        .map(|profile| format!("{} ({})", profile.label, profile.id))
        .unwrap_or_else(|| selected_profile_id.to_string());

    let profile_items = devtools_gate_profiles_v1()
        .iter()
        .filter(|profile| {
            devtools_gate_script_target_profile_ids_v1().contains(&profile.id)
                || profile.id == "perf-thresholds"
                || profile.id == "resource-footprint-thresholds"
        })
        .map(|profile| shadcn::SelectItem::new(profile.id, format!("{} ({})", profile.label, profile.id)))
        .collect::<Vec<_>>();
    let profile_select =
        shadcn::Select::new(st.gate_profile_selected_id.clone(), st.gate_profile_open.clone())
            .value(shadcn::SelectValue::new().placeholder("Gate profile"))
            .items(profile_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(260.0)))
            .into_element(cx);
    let gate_inputs = match selected_profile_id.as_ref() {
        "perf-thresholds" => perf_threshold_gate_inputs(cx, st),
        "resource-footprint-thresholds" => resource_footprint_threshold_gate_inputs(cx, st),
        _ => script_target_gate_inputs(cx, st),
    };
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
    let gate_run_in_flight = cx
        .app
        .models()
        .read(&st.gate_run_in_flight, |v| *v)
        .unwrap_or(false);
    let gate_run_result_path = cx
        .app
        .models()
        .read(&st.gate_run_last_result_path, |v| v.clone())
        .ok()
        .flatten()
        .map(|v| v.to_string());
    let gate_run_error = cx
        .app
        .models()
        .read(&st.gate_run_last_error, |v| v.clone())
        .ok()
        .flatten()
        .map(|v| v.to_string());
    let gate_run_result_json = cx
        .app
        .models()
        .read(&st.gate_run_last_result_json, |v| v.clone())
        .unwrap_or_default();
    let gate_run_result_history = cx
        .app
        .models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let gate_run_selected_result_path = cx
        .app
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
    let gate_result_actions = ui::h_row(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if selected_gate_run_result_path.is_some() {
            out.push(
                shadcn::Button::new("Copy gate result")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_GATE_RESULT_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open gate JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_GATE_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if selected_gate_run_result_entry.is_some() {
            out.push(
                shadcn::Button::new("Copy gate command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_GATE_RESULT_COMMAND)
                    .into_element(cx),
            );
        }
        if !selected_gate_run_result_json.trim().is_empty() {
            out.push(
                shadcn::Button::new("Copy gate JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_GATE_RESULT_JSON)
                    .into_element(cx),
            );
        }
        out
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let gate_result_details = text_blob_sized(
        cx,
        gate_run::gate_run_result_history_entry_detail_lines(
            selected_gate_run_result_entry.as_ref(),
        )
        .join("\n"),
        Px(78.0),
    );
    let gate_result_summary = text_blob_sized(
        cx,
        gate_run::gate_run_result_summary_lines(&selected_gate_run_result_json).join("\n"),
        Px(92.0),
    );
    let gate_result_history_summary = text_blob_sized(
        cx,
        gate_run::gate_run_result_history_summary_lines(&gate_run_result_history).join("\n"),
        Px(84.0),
    );
    let gate_result_history = gate_run_history_list(
        cx,
        &st.gate_run_selected_result_path,
        &gate_run_result_history,
        selected_gate_run_result_path.as_deref(),
    );
    let gate_run_status_line = format!(
        "gate_run_in_flight={} last_gate_result={} last_gate_error={}",
        gate_run_in_flight,
        gate_run_result_path.as_deref().unwrap_or("-"),
        gate_run_error.as_deref().unwrap_or("-")
    );
    let command_line_for_copy = command_preview.clone();
    let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let token = host.next_clipboard_token();
        host.push_effect(Effect::ClipboardWriteText {
            window: action_cx.window,
            token,
            text: command_line_for_copy.clone(),
        });
        host.request_redraw(action_cx.window);
    });
    let copy_button = shadcn::Button::new("Copy generated command")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!copy_enabled)
        .on_activate(on_copy)
        .into_element(cx);
    let run_button = shadcn::Button::new("Run generated command")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!run_enabled || gate_run_in_flight)
        .on_click(CMD_GATE_RUN_GENERATED)
        .into_element(cx);
    let controls = ui::h_row(|_cx| [profile_select, copy_button, run_button])
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let preview = text_blob_sized(cx, command_preview, Px(58.0));
    let result_preview = text_blob_sized(
        cx,
        if selected_gate_run_result_json.trim().is_empty() {
            "<no generated gate result yet>".to_string()
        } else {
            selected_gate_run_result_json
        },
        Px(92.0),
    );
    ui::v_stack(|cx| {
        [
            cx.text(format!(
                "Runnable generated gate: {selected_profile_label}"
            )),
            controls,
            gate_inputs,
            cx.text(command_state_line),
            cx.text(gate_run_status_line),
            preview,
            diag_section(
                cx,
                "Generated Gate Result Details",
                "Selected script-target gate result status, path, command, and error preview.",
                vec![gate_result_actions, gate_result_details],
            ),
            diag_section(
                cx,
                "Generated Gate Result Summary",
                "Status, command, duration, and error preview from the selected generated gate result.",
                vec![gate_result_summary],
            ),
            diag_section(
                cx,
                "Generated Gate Result History",
                "Select a GUI-launched generated gate result, newest first.",
                vec![gate_result_history_summary, gate_result_history],
            ),
            result_preview,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn script_target_gate_inputs(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let script_input = shadcn::Input::new(st.gate_profile_script_json.clone())
        .placeholder("tools/diag-scripts/<script>.json")
        .a11y_label("Gate script JSON")
        .test_id("devtools.gate.script_json")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);
    let test_id_input = shadcn::Input::new(st.gate_profile_test_id.clone())
        .placeholder("test-id")
        .a11y_label("Gate test id")
        .test_id("devtools.gate.test_id")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
        .into_element(cx);
    ui::h_row(|_cx| [script_input, test_id_input])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
}

fn perf_threshold_gate_inputs(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let target_input = gate_string_input(
        cx,
        st.gate_profile_perf_target.clone(),
        "script-or-suite",
        "Perf gate target",
        "devtools.gate.perf_target",
        300.0,
    );
    let repeat_input = gate_string_input(
        cx,
        st.gate_profile_perf_repeat.clone(),
        "repeat",
        "Perf gate repeat",
        "devtools.gate.perf_repeat",
        92.0,
    );
    let warmup_input = gate_string_input(
        cx,
        st.gate_profile_perf_warmup_frames.clone(),
        "warmup",
        "Perf gate warmup frames",
        "devtools.gate.perf_warmup_frames",
        104.0,
    );
    let agg_input = gate_string_input(
        cx,
        st.gate_profile_perf_threshold_agg.clone(),
        "agg",
        "Perf gate aggregate",
        "devtools.gate.perf_threshold_agg",
        84.0,
    );
    let max_total_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_top_total_us.clone(),
        "max total us",
        "Perf gate max top total microseconds",
        "devtools.gate.perf_max_top_total_us",
        136.0,
    );
    let max_layout_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_top_layout_us.clone(),
        "max layout us",
        "Perf gate max top layout microseconds",
        "devtools.gate.perf_max_top_layout_us",
        138.0,
    );
    let max_solve_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_top_solve_us.clone(),
        "max solve us",
        "Perf gate max top solve microseconds",
        "devtools.gate.perf_max_top_solve_us",
        132.0,
    );
    let max_pointer_dispatch_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_pointer_move_dispatch_us.clone(),
        "max dispatch us",
        "Perf gate max pointer-move dispatch microseconds",
        "devtools.gate.perf_max_pointer_move_dispatch_us",
        152.0,
    );
    let max_pointer_hit_test_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_pointer_move_hit_test_us.clone(),
        "max hit-test us",
        "Perf gate max pointer-move hit-test microseconds",
        "devtools.gate.perf_max_pointer_move_hit_test_us",
        152.0,
    );
    let max_pointer_global_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_pointer_move_global_changes.clone(),
        "max global changes",
        "Perf gate max pointer-move global changes",
        "devtools.gate.perf_max_pointer_move_global_changes",
        160.0,
    );
    let max_renderer_encode_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_encode_scene_us.clone(),
        "max encode us",
        "Perf gate max renderer encode scene microseconds",
        "devtools.gate.perf_max_renderer_encode_scene_us",
        148.0,
    );
    let max_renderer_upload_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_upload_us.clone(),
        "max upload us",
        "Perf gate max renderer upload microseconds",
        "devtools.gate.perf_max_renderer_upload_us",
        136.0,
    );
    let max_renderer_record_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_record_passes_us.clone(),
        "max record us",
        "Perf gate max renderer record passes microseconds",
        "devtools.gate.perf_max_renderer_record_passes_us",
        140.0,
    );
    let max_renderer_finish_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_encoder_finish_us.clone(),
        "max finish us",
        "Perf gate max renderer encoder finish microseconds",
        "devtools.gate.perf_max_renderer_encoder_finish_us",
        140.0,
    );
    let max_renderer_text_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_prepare_text_us.clone(),
        "max text us",
        "Perf gate max renderer prepare text microseconds",
        "devtools.gate.perf_max_renderer_prepare_text_us",
        130.0,
    );
    let max_renderer_svg_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_prepare_svg_us.clone(),
        "max svg us",
        "Perf gate max renderer prepare SVG microseconds",
        "devtools.gate.perf_max_renderer_prepare_svg_us",
        126.0,
    );
    let max_renderer_instance_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_instance_bytes.clone(),
        "max instance bytes",
        "Perf gate max renderer instance bytes",
        "devtools.gate.perf_max_renderer_instance_bytes",
        166.0,
    );
    let max_renderer_text_ops_input = gate_string_input(
        cx,
        st.gate_profile_perf_max_renderer_encode_scene_text_ops.clone(),
        "max text ops",
        "Perf gate max renderer encode scene text ops",
        "devtools.gate.perf_max_renderer_encode_scene_text_ops",
        142.0,
    );
    let run_inputs = ui::h_row(|_cx| [target_input, repeat_input, warmup_input, agg_input])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let top_threshold_inputs =
        ui::h_row(|_cx| [max_total_input, max_layout_input, max_solve_input])
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);
    let pointer_threshold_inputs = ui::h_row(|_cx| {
        [
            max_pointer_dispatch_input,
            max_pointer_hit_test_input,
            max_pointer_global_input,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let renderer_time_inputs = ui::h_row(|_cx| {
        [
            max_renderer_encode_input,
            max_renderer_upload_input,
            max_renderer_record_input,
            max_renderer_finish_input,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let renderer_payload_inputs = ui::h_row(|_cx| {
        [
            max_renderer_text_input,
            max_renderer_svg_input,
            max_renderer_instance_input,
            max_renderer_text_ops_input,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    ui::v_stack(|_cx| {
        [
            run_inputs,
            top_threshold_inputs,
            pointer_threshold_inputs,
            renderer_time_inputs,
            renderer_payload_inputs,
        ]
    })
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
}

fn gate_string_input(
    cx: &mut ElementContext<'_, App>,
    model: Model<String>,
    placeholder: &'static str,
    a11y_label: &'static str,
    test_id: &'static str,
    width_px: f32,
) -> AnyElement {
    shadcn::Input::new(model)
        .placeholder(placeholder)
        .a11y_label(a11y_label)
        .test_id(test_id)
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(width_px)))
        .into_element(cx)
}

fn resource_footprint_threshold_gate_inputs(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let target_input = shadcn::Input::new(st.gate_profile_resource_target.clone())
        .placeholder("script-or-suite")
        .a11y_label("Resource footprint gate target")
        .test_id("devtools.gate.resource_target")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);
    let launch_input = shadcn::Input::new(st.gate_profile_resource_launch_command.clone())
        .placeholder("target/release/app.exe")
        .a11y_label("Resource footprint gate launch command")
        .test_id("devtools.gate.resource_launch_command")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);
    let max_working_input =
        shadcn::Input::new(st.gate_profile_resource_max_working_set_bytes.clone())
            .placeholder("max working bytes")
            .a11y_label("Resource footprint max working set bytes")
            .test_id("devtools.gate.resource_max_working_set_bytes")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx);
    let max_peak_input =
        shadcn::Input::new(st.gate_profile_resource_max_peak_working_set_bytes.clone())
            .placeholder("max peak bytes")
            .a11y_label("Resource footprint max peak working set bytes")
            .test_id("devtools.gate.resource_max_peak_working_set_bytes")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx);
    let max_cpu_input =
        shadcn::Input::new(st.gate_profile_resource_max_cpu_avg_percent_total_cores.clone())
            .placeholder("max cpu %")
            .a11y_label("Resource footprint max CPU average percent total cores")
            .test_id("devtools.gate.resource_max_cpu_avg_percent_total_cores")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(150.0)))
            .into_element(cx);
    let target_inputs = ui::h_row(|_cx| [target_input, launch_input])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let threshold_inputs = ui::h_row(|_cx| [max_working_input, max_peak_input, max_cpu_input])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    ui::v_stack(|_cx| [target_inputs, threshold_inputs])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
}

fn devtools_gate_profile_action_rows(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    devtools_gate_profiles_v1()
        .iter()
        .map(|profile| {
            let command_line = profile.command_line.to_string();
            let on_copy: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: command_line.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            ui::h_row(|cx| {
                [
                    cx.text(format!("{} ({})", profile.label, profile.id)),
                    shadcn::Button::new("Copy command")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx)
        })
        .collect()
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
