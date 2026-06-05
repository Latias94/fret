pub(super) const CMD_COPY_WS_URL: &str = "fret.devtools.copy_ws_url";
pub(super) const CMD_COPY_TOKEN: &str = "fret.devtools.copy_token";
pub(super) const CMD_INSPECT_ENABLE: &str = "fret.devtools.inspect_enable";
pub(super) const CMD_INSPECT_DISABLE: &str = "fret.devtools.inspect_disable";
pub(super) const CMD_PICK_ARM: &str = "fret.devtools.pick_arm";
pub(super) const CMD_BUNDLE_DUMP: &str = "fret.devtools.bundle_dump";
pub(super) const CMD_SCREENSHOT_REQUEST: &str = "fret.devtools.screenshot_request";
pub(super) const CMD_SCRIPT_PUSH: &str = "fret.devtools.script_push";
pub(super) const CMD_SCRIPT_RUN: &str = "fret.devtools.script_run";
pub(super) const CMD_SCRIPT_RUN_AND_PACK: &str = "fret.devtools.script_run_and_pack";
pub(super) const CMD_SCRIPTS_REFRESH: &str = "fret.devtools.scripts.refresh";
pub(super) const CMD_SCRIPT_FORK: &str = "fret.devtools.script.fork";
pub(super) const CMD_SCRIPT_SAVE: &str = "fret.devtools.script.save";
pub(super) const CMD_SCRIPT_APPLY_PICK: &str = "fret.devtools.script.apply_pick";
pub(super) const CMD_PACK_LAST_BUNDLE: &str = "fret.devtools.pack_last_bundle";
pub(super) const CMD_COPY_PACK_PATH: &str = "fret.devtools.copy_pack_path";
pub(super) const CMD_OPEN_VIEWER_URL: &str = "fret.devtools.open_viewer_url";
pub(super) const CMD_REGRESSION_REFRESH: &str = "fret.devtools.regression.refresh";
pub(super) const CMD_REGRESSION_SUMMARIZE: &str = "fret.devtools.regression.summarize";
pub(super) const CMD_REGRESSION_PACK_SELECTED_BUNDLE: &str =
    "fret.devtools.regression.pack_selected_bundle";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_STATS: &str =
    "fret.devtools.regression.followup.stats";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF: &str =
    "fret.devtools.regression.followup.layout_perf";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_MEMORY: &str =
    "fret.devtools.regression.followup.memory";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE: &str =
    "fret.devtools.regression.followup.triage";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS: &str =
    "fret.devtools.regression.followup.hotspots";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_TRACE: &str =
    "fret.devtools.regression.followup.trace";
pub(super) const CMD_REGRESSION_RUN_FOLLOWUP_COMMAND: &str =
    "fret.devtools.regression.followup.run_command";
pub(super) const CMD_REGRESSION_RUN_VISUAL_COMPARE: &str =
    "fret.devtools.regression.followup.visual_compare";
pub(super) const CMD_REGRESSION_RUN_FOOTPRINT_COMPARE: &str =
    "fret.devtools.regression.followup.footprint_compare";
pub(super) const CMD_COPY_FOLLOWUP_RESULT_PATH: &str =
    "fret.devtools.regression.followup.copy_result_path";
pub(super) const CMD_COPY_FOLLOWUP_RESULT_JSON: &str =
    "fret.devtools.regression.followup.copy_result_json";
pub(super) const CMD_COPY_FOLLOWUP_RESULT_COMMAND: &str =
    "fret.devtools.regression.followup.copy_result_command";
pub(super) const CMD_OPEN_FOLLOWUP_RESULT_JSON: &str =
    "fret.devtools.regression.followup.open_result_json";
pub(super) const CMD_COPY_FOLLOWUP_TRACE_ARTIFACT_PATH: &str =
    "fret.devtools.regression.followup.copy_trace_artifact_path";
pub(super) const CMD_OPEN_FOLLOWUP_TRACE_ARTIFACT: &str =
    "fret.devtools.regression.followup.open_trace_artifact";
pub(super) const CMD_GATE_RUN_GENERATED: &str = "fret.devtools.gate.run_generated";
pub(super) const CMD_COPY_GATE_RESULT_PATH: &str = "fret.devtools.gate.copy_result_path";
pub(super) const CMD_COPY_GATE_RESULT_JSON: &str = "fret.devtools.gate.copy_result_json";
pub(super) const CMD_COPY_GATE_RESULT_COMMAND: &str = "fret.devtools.gate.copy_result_command";
pub(super) const CMD_OPEN_GATE_RESULT_JSON: &str = "fret.devtools.gate.open_result_json";
pub(super) const CMD_WORKFLOW_RUN_SELECTED: &str = "fret.devtools.workflow.run_selected";
pub(super) const CMD_COPY_RECENT_EVIDENCE_REPORT: &str =
    "fret.devtools.recent_evidence.copy_report";
pub(super) const CMD_SELECT_RECENT_FAILED_EVIDENCE: &str =
    "fret.devtools.recent_evidence.select_failed";
pub(super) const CMD_RERUN_RECENT_FAILED_EVIDENCE: &str =
    "fret.devtools.recent_evidence.rerun_failed";
pub(super) const CMD_COPY_RECENT_FAILED_EVIDENCE_PATH: &str =
    "fret.devtools.recent_evidence.copy_failed_path";
pub(super) const CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR: &str =
    "fret.devtools.recent_evidence.copy_failed_bundle_dir";
pub(super) const CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND: &str =
    "fret.devtools.recent_evidence.copy_failed_command";
pub(super) const CMD_COPY_RECENT_FAILED_EVIDENCE_JSON: &str =
    "fret.devtools.recent_evidence.copy_failed_json";
pub(super) const CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON: &str =
    "fret.devtools.recent_evidence.open_failed_json";
pub(super) const CMD_COPY_WORKFLOW_RESULT_PATH: &str =
    "fret.devtools.workflow.copy_result_path";
pub(super) const CMD_COPY_WORKFLOW_RESULT_JSON: &str =
    "fret.devtools.workflow.copy_result_json";
pub(super) const CMD_COPY_WORKFLOW_RESULT_COMMAND: &str =
    "fret.devtools.workflow.copy_result_command";
pub(super) const CMD_OPEN_WORKFLOW_RESULT_JSON: &str =
    "fret.devtools.workflow.open_result_json";
pub(super) const CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH: &str =
    "fret.devtools.workflow.copy_suite_summary_path";
pub(super) const CMD_OPEN_WORKFLOW_SUITE_SUMMARY: &str =
    "fret.devtools.workflow.open_suite_summary";
pub(super) const CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH: &str =
    "fret.devtools.workflow.copy_regression_summary_path";
pub(super) const CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY: &str =
    "fret.devtools.workflow.open_regression_summary";
pub(super) const CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH: &str =
    "fret.devtools.workflow.copy_regression_index_path";
pub(super) const CMD_OPEN_WORKFLOW_REGRESSION_INDEX: &str =
    "fret.devtools.workflow.open_regression_index";
pub(super) const CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY: &str =
    "fret.devtools.workflow.load_regression_summary";
pub(super) const CMD_LOAD_WORKFLOW_REGRESSION_INDEX: &str =
    "fret.devtools.workflow.load_regression_index";
pub(super) const CMD_COPY_WORKFLOW_SUMMARIZE_COMMAND: &str =
    "fret.devtools.workflow.copy_summarize_command";
pub(super) const CMD_RUN_WORKFLOW_SUMMARIZE: &str = "fret.devtools.workflow.run_summarize";

pub(super) const DEVTOOLS_FIRST_OPEN_DOC: &str = "docs/diagnostics-first-open.md";
pub(super) const DEVTOOLS_GUI_BRANCH_DOC: &str =
    "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md";
pub(super) const DEVTOOLS_REPO_PREFLIGHT_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag doctor campaigns";
pub(super) const DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag doctor campaigns --json";
pub(super) const DEVTOOLS_FIRST_OPEN_GATE_COMMAND: &str =
    "python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke";
pub(super) const DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID: &str = "devtools-first-open-smoke";
pub(super) const DEVTOOLS_DOGFOOD_WORKFLOW_ID: &str = "ui-gallery-button-dogfood";
pub(super) const DEVTOOLS_DOGFOOD_TARGET_COMMAND: &str = "cargo run -p fret-ui-gallery --release";
pub(super) const DEVTOOLS_DOGFOOD_BASE_SCRIPT: &str =
    "tools/diag-scripts/ui-gallery-lite-smoke.json";
pub(super) const DEVTOOLS_DOGFOOD_BUTTON_SCRIPT: &str =
    "tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json";
pub(super) const DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag pick-script --pick-script-out target/fret-diag/picked.script.json";
pub(super) const DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag pick-apply tools/diag-scripts/ui-gallery-lite-smoke.json --ptr /steps/12/target --out target/fret-diag/ui-gallery-picked.script.json";
pub(super) const DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag run target/fret-diag/ui-gallery-picked.script.json --pack --include-all --pack-schema2-only --launch -- cargo run -p fret-ui-gallery --release";
pub(super) const DEVTOOLS_DOGFOOD_PACK_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag pack <bundle-dir> --include-all --pack-schema2-only";
pub(super) const DEVTOOLS_DOGFOOD_VIEWER_COMMAND: &str = "pnpm -C tools/fret-bundle-viewer dev";
pub(super) const IMUI_PRODUCT_WORKFLOW_ID: &str = fret_first_open::product_workflow::ID;
pub(super) const IMUI_PRODUCT_WORKFLOW_DOC: &str = fret_first_open::product_workflow::DOC;
pub(super) const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = fret_first_open::product_workflow::COMMAND;
pub(super) const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =
    fret_first_open::product_workflow::FOCUSED_COMMAND;
pub(super) const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =
    fret_first_open::product_workflow::LAUNCHED_COMMAND;
pub(super) const IMUI_PRODUCT_WORKFLOW_SUITE: &str = fret_first_open::product_workflow::SUITE;
pub(super) const DEVTOOLS_WORKFLOW_ROUTE_ID: &str = "workflow-runs";
pub(super) const DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID: &str =
    "campaign-validate-devtools-first-open";
pub(super) const DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID: &str =
    "campaign-validate-imui-p3-multiwindow";
pub(super) const DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID: &str = "perf-docking-suite-ws";
pub(super) const DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST: &str =
    "tools/diag-campaigns/devtools-first-open-smoke.json";
pub(super) const DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST: &str =
    "tools/diag-campaigns/imui-p3-multiwindow-parity.json";
pub(super) const DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE: &str =
    "perf-docking-arbitration-steady";
pub(super) const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] =
    fret_first_open::product_workflow::EXPECTED_ARTIFACTS;
pub(super) const DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID: &str =
    fret_first_open::demo_metrics_debug::ROUTE_ID;
pub(super) const DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_WORKBENCH_COMMAND;
pub(super) const DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_PROOF_COMMAND;
pub(super) const DEVTOOLS_DEMO_EDITOR_NOTES_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_NOTES_COMMAND;
pub(super) const DEVTOOLS_DEMO_DEVICE_SHELL_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_DEVICE_SHELL_COMMAND;
pub(super) const DEVTOOLS_METRICS_STATS_COMMAND: &str =
    fret_first_open::demo_metrics_debug::METRICS_STATS_COMMAND;
pub(super) const DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json";
pub(super) const DEVTOOLS_METRICS_MEMORY_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json";
pub(super) const DEVTOOLS_DEBUG_TRIAGE_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json";
pub(super) const DEVTOOLS_DEBUG_HOTSPOTS_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json";
pub(super) const DEVTOOLS_DEBUG_TRACE_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEBUG_TRACE_COMMAND;
pub(super) const DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC: &str =
    fret_first_open::demo_metrics_debug::OWNER_DOC;
pub(super) const DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC: &str =
    fret_first_open::demo_metrics_debug::ACTION_METADATA_DOC;
pub(super) const DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =
    fret_first_open::demo_metrics_debug::DOCKING_OWNER_DOC;
pub(super) const DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str =
    fret_first_open::demo_metrics_debug::WAYLAND_ACCEPTANCE_DOC;
pub(super) const DEVTOOLS_DOCKING_ARBITRATION_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DOCKING_ARBITRATION_COMMAND;
pub(super) const DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DOCKING_CAMPAIGN_VALIDATE_COMMAND;
pub(super) const DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DOCKING_POLICY_SKIP_COMMAND;
pub(super) const CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS: &str =
    "fret.devtools.demo_metrics_debug.copy_actions";
pub(super) const CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW: &str =
    "fret.devtools.demo_metrics_debug.run_docking_workflow";
pub(super) const CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW: &str =
    "fret.devtools.demo_metrics_debug.run_perf_workflow";
