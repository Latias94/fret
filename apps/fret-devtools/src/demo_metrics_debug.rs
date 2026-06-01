use super::{
    DEVTOOLS_DEBUG_HOTSPOTS_COMMAND, DEVTOOLS_DEBUG_TRACE_COMMAND, DEVTOOLS_DEBUG_TRIAGE_COMMAND,
    DEVTOOLS_DEMO_DEVICE_SHELL_COMMAND, DEVTOOLS_DEMO_EDITOR_NOTES_COMMAND,
    DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND, DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND,
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC, DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC,
    DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC, DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID,
    DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC, DEVTOOLS_DOCKING_ARBITRATION_COMMAND,
    DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND, DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND,
    DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND, DEVTOOLS_METRICS_MEMORY_COMMAND,
    DEVTOOLS_METRICS_STATS_COMMAND, IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND,
};
use super::{CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS, State, diag_section};
use fret_app::{App, CommandId};
use fret_ui::element::AnyElement;
use fret_ui::ElementContext;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

const DEVTOOLS_DEMO_METRICS_DEBUG_COPY_ACTION_PREFIX: &str =
    "fret.devtools.demo_metrics_debug.copy_action.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DemoMetricsDebugActionSpec {
    id: &'static str,
    label: &'static str,
    command: &'static str,
    category: &'static str,
    requires_bundle: bool,
    primary: bool,
}

const DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS: &[DemoMetricsDebugActionSpec] = &[
    DemoMetricsDebugActionSpec {
        id: "open_workbench",
        label: "open workbench",
        command: DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND,
        category: "demo",
        requires_bundle: false,
        primary: true,
    },
    DemoMetricsDebugActionSpec {
        id: "product_discovery",
        label: "run product discovery",
        command: IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND,
        category: "product-gate",
        requires_bundle: false,
        primary: false,
    },
    DemoMetricsDebugActionSpec {
        id: "inspect_metrics_stats",
        label: "inspect metrics stats",
        command: DEVTOOLS_METRICS_STATS_COMMAND,
        category: "metrics",
        requires_bundle: true,
        primary: false,
    },
    DemoMetricsDebugActionSpec {
        id: "inspect_debug_trace",
        label: "inspect debug trace",
        command: DEVTOOLS_DEBUG_TRACE_COMMAND,
        category: "debug",
        requires_bundle: true,
        primary: false,
    },
    DemoMetricsDebugActionSpec {
        id: "validate_docking_campaign",
        label: "validate docking campaign",
        command: DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND,
        category: "handoff",
        requires_bundle: false,
        primary: false,
    },
];

pub(crate) fn demo_metrics_debug_action_command_text() -> String {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| format!("{}: {}", action.label, action.command))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn demo_metrics_debug_action_copy_command_id(action_id: &str) -> CommandId {
    CommandId::new(format!(
        "{DEVTOOLS_DEMO_METRICS_DEBUG_COPY_ACTION_PREFIX}{action_id}"
    ))
}

pub(crate) fn demo_metrics_debug_action_command_for_copy_command(
    command_id: &str,
) -> Option<String> {
    let action_id = command_id.strip_prefix(DEVTOOLS_DEMO_METRICS_DEBUG_COPY_ACTION_PREFIX)?;
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .find(|action| action.id == action_id)
        .map(|action| action.command.to_string())
}

pub(crate) fn demo_metrics_debug_action_copy_command_lines() -> Vec<String> {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| {
            format!(
                "action copy command: {} | id={} | copy_command={}",
                action.label,
                action.id,
                demo_metrics_debug_action_copy_command_id(action.id).as_str()
            )
        })
        .collect()
}

pub(crate) fn demo_metrics_debug_action_metadata_lines() -> Vec<String> {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| {
            format!(
                "action metadata: {} | id={} | category={} | primary={} | requires_bundle={}",
                action.label, action.id, action.category, action.primary, action.requires_bundle
            )
        })
        .collect()
}

pub(crate) fn demo_metrics_debug_action_readiness_lines(
    selected_bundle_count: usize,
) -> Vec<String> {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| {
            let runnable = !action.requires_bundle || selected_bundle_count > 0;
            let reason = if action.requires_bundle {
                if selected_bundle_count > 0 {
                    "selected bundle evidence available"
                } else {
                    "select a regression bundle"
                }
            } else {
                "no bundle required"
            };
            format!(
                "action readiness: {} | id={} | category={} | runnable={} | reason={}",
                action.label, action.id, action.category, runnable, reason
            )
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn devtools_demo_metrics_debug_lines(artifacts_root: &str) -> Vec<String> {
    devtools_demo_metrics_debug_lines_with_state(artifacts_root, 0)
}

pub(crate) fn devtools_demo_metrics_debug_lines_with_state(
    artifacts_root: &str,
    selected_bundle_count: usize,
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
    for line in devtools_demo_metrics_debug_lines_with_state(
        st.cfg.fs_out_dir.as_ref(),
        demo_metrics_debug_selected_bundle_count,
    ) {
        demo_metrics_debug_rows.push(cx.text(line));
    }
    demo_metrics_debug_rows.push(devtools_demo_metrics_debug_action_row(cx));
    diag_section(
        cx,
        "Demo / Metrics / Debug Routes",
        "Always-available editor demos, action commands, metrics commands, and debug drill-down entrypoints stay visible in the GUI shell. Per-action copy commands stay visible alongside the bundle copy control.",
        demo_metrics_debug_rows,
    )
}

fn devtools_demo_metrics_debug_action_row(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let mut actions = vec![
        shadcn::Button::new("Copy Demo/Metrics/Debug actions")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_click(CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS)
            .into_element(cx),
    ];
    actions.extend(DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS.iter().map(|action| {
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
