use super::{
    DEVTOOLS_DEBUG_HOTSPOTS_COMMAND, DEVTOOLS_DEBUG_TRACE_COMMAND, DEVTOOLS_DEBUG_TRIAGE_COMMAND,
    DEVTOOLS_DEMO_DEVICE_SHELL_COMMAND, DEVTOOLS_DEMO_EDITOR_NOTES_COMMAND,
    DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND, DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND,
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC, DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC,
    DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC, DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID,
    DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC, DEVTOOLS_DOCKING_ARBITRATION_COMMAND,
    DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND, DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND,
    DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND, DEVTOOLS_METRICS_MEMORY_COMMAND,
    DEVTOOLS_METRICS_STATS_COMMAND, DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID,
    DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID, IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND,
};
use super::{
    CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS, CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW,
    CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW, CMD_COPY_WORKFLOW_RESULT_PATH,
    CMD_LOAD_WORKFLOW_REGRESSION_INDEX, CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY,
    CMD_OPEN_WORKFLOW_RESULT_JSON, State, devtools_workflow_commands_from_state,
    selected_workflow_run_regression_index_path_from_state,
    selected_workflow_run_regression_summary_path_from_state,
};
use super::ui_primitives::diag_section;
use fret_app::{App, CommandId};
use fret_ui::element::AnyElement;
use fret_ui::ElementContext;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

mod actions;
mod workflow;

pub(crate) use workflow::{
    demo_metrics_debug_workflow_artifact_action_lines,
    demo_metrics_debug_workflow_readiness_lines, demo_metrics_debug_workflow_result_action_lines,
    demo_metrics_debug_workflow_status_lines,
};

pub(crate) fn demo_metrics_debug_action_command_text() -> String {
    actions::demo_metrics_debug_action_command_text()
}

pub(crate) fn demo_metrics_debug_action_copy_command_id(action_id: &str) -> CommandId {
    actions::demo_metrics_debug_action_copy_command_id(action_id)
}

pub(crate) fn demo_metrics_debug_action_command_for_copy_command(
    command_id: &str,
) -> Option<String> {
    actions::demo_metrics_debug_action_command_for_copy_command(command_id)
}

pub(crate) fn demo_metrics_debug_action_copy_command_lines() -> Vec<String> {
    actions::demo_metrics_debug_action_copy_command_lines()
}

pub(crate) fn demo_metrics_debug_action_metadata_lines() -> Vec<String> {
    actions::demo_metrics_debug_action_metadata_lines()
}

pub(crate) fn demo_metrics_debug_action_readiness_lines(
    selected_bundle_count: usize,
) -> Vec<String> {
    actions::demo_metrics_debug_action_readiness_lines(selected_bundle_count)
}

#[cfg(test)]
pub(crate) fn devtools_demo_metrics_debug_lines(artifacts_root: &str) -> Vec<String> {
    devtools_demo_metrics_debug_lines_with_state(artifacts_root, 0)
}

#[cfg(test)]
pub(crate) fn devtools_demo_metrics_debug_lines_with_state(
    artifacts_root: &str,
    selected_bundle_count: usize,
) -> Vec<String> {
    devtools_demo_metrics_debug_lines_with_runtime_state(
        artifacts_root,
        selected_bundle_count,
        false,
        false,
        false,
        false,
        false,
        None,
        None,
    )
}

fn devtools_demo_metrics_debug_lines_with_runtime_state(
    artifacts_root: &str,
    selected_bundle_count: usize,
    workflow_run_in_flight: bool,
    perf_workflow_runnable: bool,
    workflow_result_available: bool,
    regression_summary_available: bool,
    regression_index_available: bool,
    last_result_path: Option<&str>,
    last_error: Option<&str>,
) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    let mut lines = vec![
        format!("route: {DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID}"),
        format!("route owner: {DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC}"),
        format!("action metadata owner: {DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}"),
        format!("docking owner: {DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC}"),
        format!("wayland acceptance: {DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC}"),
        format!("artifacts root: {artifacts_root}"),
        "route surface: Always-available editor demos, metrics commands, and debug drill-down entrypoints stay visible in the GUI shell."
            .to_string(),
        "action surface: dedicated DevTools guide panel + copyable action command bundle and per-action copy commands"
            .to_string(),
        format!(
            "workflow handoff: validate docking campaign | workflow_id={DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID} | run_command={CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW}"
        ),
        format!(
            "workflow handoff: run perf docking suite | workflow_id={DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID} | run_command={CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW} | requires=selected-session"
        ),
        "command palette: deferred until DevTools has a shared command palette contract".to_string(),
        format!("action: open workbench -> {DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND}"),
        format!("action: run product discovery -> {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}"),
        format!("action: inspect metrics stats -> {DEVTOOLS_METRICS_STATS_COMMAND}"),
        format!("action: inspect debug trace -> {DEVTOOLS_DEBUG_TRACE_COMMAND}"),
        format!("action: validate docking campaign -> {DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND}"),
    ];
    lines.extend(demo_metrics_debug_action_metadata_lines());
    lines.extend(demo_metrics_debug_action_copy_command_lines());
    lines.extend(demo_metrics_debug_action_readiness_lines(
        selected_bundle_count,
    ));
    lines.extend(demo_metrics_debug_workflow_readiness_lines(
        workflow_run_in_flight,
        perf_workflow_runnable,
    ));
    lines.extend(demo_metrics_debug_workflow_status_lines(
        workflow_run_in_flight,
        last_result_path,
        last_error,
    ));
    lines.extend(demo_metrics_debug_workflow_result_action_lines(
        workflow_result_available,
    ));
    lines.extend(demo_metrics_debug_workflow_artifact_action_lines(
        regression_summary_available,
        regression_index_available,
    ));
    lines.extend([
        format!("demo editor workbench: {DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND}"),
        format!("demo editor proof supporting: {DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND}"),
        format!("demo editor notes: {DEVTOOLS_DEMO_EDITOR_NOTES_COMMAND}"),
        format!("demo device shell: {DEVTOOLS_DEMO_DEVICE_SHELL_COMMAND}"),
        format!("metrics stats: {DEVTOOLS_METRICS_STATS_COMMAND}"),
        format!("metrics layout perf: {DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND}"),
        format!("metrics memory: {DEVTOOLS_METRICS_MEMORY_COMMAND}"),
        format!("debug triage: {DEVTOOLS_DEBUG_TRIAGE_COMMAND}"),
        format!("debug hotspots: {DEVTOOLS_DEBUG_HOTSPOTS_COMMAND}"),
        format!("debug trace: {DEVTOOLS_DEBUG_TRACE_COMMAND}"),
        format!("docking arbitration supporting: {DEVTOOLS_DOCKING_ARBITRATION_COMMAND}"),
        format!("docking campaign validate: {DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND}"),
        format!("docking policy-skip local: {DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND}"),
    ]);
    lines
}

pub(crate) fn devtools_demo_metrics_debug_panel(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let mut demo_metrics_debug_rows = Vec::new();
    let demo_metrics_debug_selected_bundle_count = cx
        .app
        .models()
        .read(&st.regression_selected_bundle_dirs, |v| v.len())
        .unwrap_or(0);
    let workflow_run_in_flight = cx
        .app
        .models()
        .read(&st.workflow_run_in_flight, |v| *v)
        .unwrap_or(false);
    let workflow_run_last_result_path = cx
        .app
        .models()
        .read(&st.workflow_run_last_result_path, |v| v.clone())
        .ok()
        .flatten();
    let workflow_run_last_error = cx
        .app
        .models()
        .read(&st.workflow_run_last_error, |v| v.clone())
        .ok()
        .flatten();
    let workflow_result_available = workflow_run_last_result_path
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let regression_summary_available =
        selected_workflow_run_regression_summary_path_from_state(cx.app, st).is_some();
    let regression_index_available =
        selected_workflow_run_regression_index_path_from_state(cx.app, st).is_some();
    let perf_workflow_runnable = devtools_workflow_commands_from_state(cx.app, st)
        .into_iter()
        .find(|command| command.id == DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID)
        .is_some_and(|command| command.is_runnable());
    for line in devtools_demo_metrics_debug_lines_with_runtime_state(
        st.cfg.fs_out_dir.as_ref(),
        demo_metrics_debug_selected_bundle_count,
        workflow_run_in_flight,
        perf_workflow_runnable,
        workflow_result_available,
        regression_summary_available,
        regression_index_available,
        workflow_run_last_result_path.as_deref(),
        workflow_run_last_error.as_deref(),
    ) {
        demo_metrics_debug_rows.push(cx.text(line));
    }
    demo_metrics_debug_rows.push(devtools_demo_metrics_debug_action_row(
        cx,
        workflow_run_in_flight,
        perf_workflow_runnable,
        workflow_result_available,
        regression_summary_available,
        regression_index_available,
    ));
    diag_section(
        cx,
        "Demo / Metrics / Debug Routes",
        "Always-available editor demos, action commands, metrics commands, and debug drill-down entrypoints stay visible in the GUI shell. Per-action copy commands stay visible alongside the bundle copy control.",
        demo_metrics_debug_rows,
    )
}

fn devtools_demo_metrics_debug_action_row(
    cx: &mut ElementContext<'_, App>,
    workflow_run_in_flight: bool,
    perf_workflow_runnable: bool,
    workflow_result_available: bool,
    regression_summary_available: bool,
    regression_index_available: bool,
) -> AnyElement {
    let mut actions = vec![
        shadcn::Button::new("Copy Demo/Metrics/Debug actions")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_click(CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS)
            .into_element(cx),
        shadcn::Button::new("Run docking workflow")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .disabled(workflow_run_in_flight)
            .on_click(CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW)
            .into_element(cx),
        shadcn::Button::new("Run perf workflow")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .disabled(workflow_run_in_flight || !perf_workflow_runnable)
            .on_click(CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW)
            .into_element(cx),
        shadcn::Button::new("Copy workflow result")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .disabled(!workflow_result_available)
            .on_click(CMD_COPY_WORKFLOW_RESULT_PATH)
            .into_element(cx),
        shadcn::Button::new("Open workflow JSON")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .disabled(!workflow_result_available)
            .on_click(CMD_OPEN_WORKFLOW_RESULT_JSON)
            .into_element(cx),
        shadcn::Button::new("Load workflow regression summary")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .disabled(!regression_summary_available)
            .on_click(CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY)
            .into_element(cx),
        shadcn::Button::new("Load workflow regression index")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .disabled(!regression_index_available)
            .on_click(CMD_LOAD_WORKFLOW_REGRESSION_INDEX)
            .into_element(cx),
    ];
    actions.extend(actions::DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS.iter().map(|action| {
        shadcn::Button::new(format!("Copy {}", action.label))
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Sm)
            .on_click(demo_metrics_debug_action_copy_command_id(action.id))
            .into_element(cx)
    }));
    ui::h_row(|_cx| actions)
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx)
}
