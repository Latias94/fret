use fret_app::App;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;

use super::{
    State, collect_gate_profile_panel_state, devtools_demo_metrics_debug_panel,
    devtools_gate_command_lines, devtools_gate_profile_action_rows,
    devtools_gate_profile_command_builder, devtools_workflow_run_lines, devtools_workflow_run_panel,
    diag_section, dogfood_reference_panel, first_open_reference_panel, guide_recent_evidence_panel,
};

pub(super) fn devtools_guide_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let recent_evidence_panel = guide_recent_evidence_panel(cx, st);
    let first_open_panel = first_open_reference_panel(cx, st);
    let dogfood_workflow_panel = dogfood_reference_panel(cx, st);
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
    let gate_panel = collect_gate_profile_panel_state(cx.app, st);
    let mut gate_command_rows = Vec::new();
    for line in devtools_gate_command_lines(st.cfg.fs_out_dir.as_ref()) {
        gate_command_rows.push(cx.text(line));
    }
    for line in gate_panel.gate_profile_lines {
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
