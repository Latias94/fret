use fret::app::LocalState;
use fret::app::prelude::*;
use fret_app::CommandId;

const TEST_ID_ACTION_WORKBENCH: &str = "imui-editor-workbench.action.open-workbench";
const TEST_ID_ACTION_PROOF: &str = "imui-editor-workbench.action.supporting-proof";
const TEST_ID_ACTION_METRICS: &str = "imui-editor-workbench.action.metrics";
const TEST_ID_ACTION_DEBUG: &str = "imui-editor-workbench.action.debug";
const TEST_ID_ACTION_WAYLAND: &str = "imui-editor-workbench.action.wayland";

const WORKBENCH_COMMAND: &str = fret_first_open::demo_metrics_debug::DEMO_EDITOR_WORKBENCH_COMMAND;
const SUPPORTING_PROOF_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_PROOF_COMMAND;
const METRICS_COMMAND: &str = fret_first_open::demo_metrics_debug::METRICS_STATS_COMMAND;
const DEBUG_COMMAND: &str = fret_first_open::demo_metrics_debug::DEBUG_TRACE_COMMAND;
const WAYLAND_ACCEPTANCE_COMMAND: &str = "FRET_DOCK_TEAROFF_LOG=1 cargo run -p fretboard-dev -- diag campaign run imui-p3-wayland-real-host --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release";

mod act {
    fret::actions!([
        SelectWorkbench = "imui_editor_workbench_demo.action.open_workbench.v1",
        SelectProof = "imui_editor_workbench_demo.action.supporting_proof.v1",
        SelectMetrics = "imui_editor_workbench_demo.action.metrics.v1",
        SelectDebug = "imui_editor_workbench_demo.action.debug.v1",
        SelectWayland = "imui_editor_workbench_demo.action.wayland.v1"
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum WorkbenchQuickAction {
    #[default]
    Workbench,
    Proof,
    Metrics,
    Debug,
    Wayland,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WorkbenchQuickActionSpec {
    pub(super) action: WorkbenchQuickAction,
    pub(super) label: &'static str,
    pub(super) category: &'static str,
    pub(super) command: &'static str,
    pub(super) purpose: &'static str,
    pub(super) test_id: &'static str,
    pub(super) primary: bool,
}

pub(super) const WORKBENCH_QUICK_ACTIONS: &[WorkbenchQuickActionSpec] = &[
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Workbench,
        label: "Workbench",
        category: "demo",
        command: WORKBENCH_COMMAND,
        purpose: "Primary product-facing editor route.",
        test_id: TEST_ID_ACTION_WORKBENCH,
        primary: true,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Proof,
        label: "Proof",
        category: "supporting demo",
        command: SUPPORTING_PROOF_COMMAND,
        purpose: "Dense editor-control and docking proof surface.",
        test_id: TEST_ID_ACTION_PROOF,
        primary: false,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Metrics,
        label: "Metrics",
        category: "metrics",
        command: METRICS_COMMAND,
        purpose: "Read frame, layout, memory, and artifact stats from a diagnostics bundle.",
        test_id: TEST_ID_ACTION_METRICS,
        primary: false,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Debug,
        label: "Debug",
        category: "debug",
        command: DEBUG_COMMAND,
        purpose: "Open a diagnostics trace drill-down for the selected bundle.",
        test_id: TEST_ID_ACTION_DEBUG,
        primary: false,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Wayland,
        label: "Wayland",
        category: "handoff",
        command: WAYLAND_ACCEPTANCE_COMMAND,
        purpose: "Real-host compositor acceptance remains a runner/backend handoff.",
        test_id: TEST_ID_ACTION_WAYLAND,
        primary: false,
    },
];

pub(super) fn install_workbench_quick_action_commands(
    cx: &mut AppUi<'_, '_>,
    active_action: &LocalState<WorkbenchQuickAction>,
) {
    cx.actions()
        .local(active_action)
        .set::<act::SelectWorkbench>(WorkbenchQuickAction::Workbench);
    cx.actions()
        .local(active_action)
        .set::<act::SelectProof>(WorkbenchQuickAction::Proof);
    cx.actions()
        .local(active_action)
        .set::<act::SelectMetrics>(WorkbenchQuickAction::Metrics);
    cx.actions()
        .local(active_action)
        .set::<act::SelectDebug>(WorkbenchQuickAction::Debug);
    cx.actions()
        .local(active_action)
        .set::<act::SelectWayland>(WorkbenchQuickAction::Wayland);
}

pub(super) fn workbench_quick_action_spec(
    action: WorkbenchQuickAction,
) -> &'static WorkbenchQuickActionSpec {
    WORKBENCH_QUICK_ACTIONS
        .iter()
        .find(|spec| spec.action == action)
        .unwrap_or(&WORKBENCH_QUICK_ACTIONS[0])
}

pub(super) fn workbench_quick_action_command(action: WorkbenchQuickAction) -> CommandId {
    match action {
        WorkbenchQuickAction::Workbench => act::SelectWorkbench.into(),
        WorkbenchQuickAction::Proof => act::SelectProof.into(),
        WorkbenchQuickAction::Metrics => act::SelectMetrics.into(),
        WorkbenchQuickAction::Debug => act::SelectDebug.into(),
        WorkbenchQuickAction::Wayland => act::SelectWayland.into(),
    }
}

pub(super) fn workbench_quick_action_command_bundle_text() -> String {
    WORKBENCH_QUICK_ACTIONS
        .iter()
        .map(|spec| format!("{}: {}", spec.label, spec.command))
        .collect::<Vec<_>>()
        .join("\n")
}
