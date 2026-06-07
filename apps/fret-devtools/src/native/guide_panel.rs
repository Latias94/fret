use std::sync::Arc;

use fret_app::{App, Effect};
use fret_core::Px;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::command_catalog::{
    CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH, CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH,
    CMD_COPY_WORKFLOW_RESULT_COMMAND, CMD_COPY_WORKFLOW_RESULT_JSON, CMD_COPY_WORKFLOW_RESULT_PATH,
    CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH, CMD_COPY_WORKFLOW_SUMMARIZE_COMMAND,
    CMD_LOAD_WORKFLOW_REGRESSION_INDEX, CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY,
    CMD_OPEN_WORKFLOW_REGRESSION_INDEX, CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY,
    CMD_OPEN_WORKFLOW_RESULT_JSON, CMD_OPEN_WORKFLOW_SUITE_SUMMARY, CMD_RUN_WORKFLOW_SUMMARIZE,
    CMD_WORKFLOW_RUN_SELECTED,
};
use super::workflow_panel_state::collect_workflow_panel_state;
use super::{
    State, collect_gate_profile_panel_state, devtools_demo_metrics_debug_panel,
    devtools_gate_command_lines, devtools_gate_profile_action_rows,
    devtools_gate_profile_command_builder, devtools_workflow_run_lines, diag_section,
    dogfood_reference_panel, first_open_reference_panel, guide_recent_evidence_panel,
    text_blob_sized, workflow_run, workflow_run_history_list,
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

fn devtools_workflow_run_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let panel = collect_workflow_panel_state(cx.app, st);
    let workflow_items = panel
        .commands
        .iter()
        .map(|command| {
            shadcn::SelectItem::new(command.id.clone(), format!("{} ({})", command.label, command.id))
        })
        .collect::<Vec<_>>();
    let workflow_select = shadcn::Select::new(
        st.workflow_run_selected_id.clone(),
        st.workflow_run_selected_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("Workflow"))
    .items(workflow_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(340.0)))
    .into_element(cx);

    let workflow_run_error = cx
        .app
        .models()
        .read(&st.workflow_run_last_error, |v| v.clone())
        .ok()
        .flatten()
        .map(|v| v.to_string());
    let workflow_run_result_history = cx
        .app
        .models()
        .read(&st.workflow_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let workflow_result_actions = ui::h_row(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if panel.selected_workflow_run_result_path.is_some() {
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
        if panel.selected_workflow_run_result_entry.is_some() {
            out.push(
                shadcn::Button::new("Copy workflow command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_RESULT_COMMAND)
                    .into_element(cx),
            );
        }
        if !panel.selected_workflow_run_result_json.trim().is_empty() {
            out.push(
                shadcn::Button::new("Copy workflow JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_WORKFLOW_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if panel.selected_workflow_suite_summary_path.is_some() {
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
        if panel.selected_workflow_regression_summary_path.is_some() {
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
                    .disabled(panel.workflow_run_in_flight)
                    .on_click(CMD_RUN_WORKFLOW_SUMMARIZE)
                    .into_element(cx),
            );
        }
        if panel.selected_workflow_regression_index_ready {
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
            panel.selected_workflow_run_result_entry.as_ref(),
        )
        .join("\n"),
        Px(78.0),
    );
    let workflow_result_summary = text_blob_sized(
        cx,
        workflow_run::workflow_run_result_summary_lines(&panel.selected_workflow_run_result_json)
            .join("\n"),
        Px(92.0),
    );
    let workflow_handoff_readiness_blob = text_blob_sized(
        cx,
        panel.workflow_handoff_readiness.join("\n"),
        Px(76.0),
    );
    let workflow_summarize_handoff_blob =
        text_blob_sized(cx, panel.workflow_summarize_preview.clone(), Px(76.0));
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
        panel.selected_workflow_run_result_path.as_deref(),
    );
    let workflow_run_status_line = format!(
        "workflow_run_in_flight={} last_workflow_result={} last_workflow_error={}",
        panel.workflow_run_in_flight,
        panel.selected_workflow_run_result_path.as_deref().unwrap_or("-"),
        workflow_run_error.as_deref().unwrap_or("-")
    );
    let command_line_for_copy = panel.command_preview.clone();
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
        .disabled(panel.commands.is_empty())
        .on_activate(on_copy)
        .into_element(cx);
    let run_button = shadcn::Button::new("Run workflow")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!panel.run_enabled || panel.workflow_run_in_flight)
        .on_click(CMD_WORKFLOW_RUN_SELECTED)
        .into_element(cx);
    let controls = ui::h_row(|_cx| [workflow_select, copy_button, run_button])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let preview = text_blob_sized(cx, panel.command_preview.clone(), Px(58.0));
    let result_preview = text_blob_sized(
        cx,
        if panel.selected_workflow_run_result_json.trim().is_empty() {
            "<no workflow run result yet>".to_string()
        } else {
            panel.selected_workflow_run_result_json.clone()
        },
        Px(92.0),
    );
    ui::v_stack(|cx| {
        [
            cx.text(format!("Runnable workflow: {}", panel.selected_command_label)),
            controls,
            cx.text(panel.command_state_line.clone()),
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
