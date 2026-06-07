use std::sync::Arc;

use fret_app::{App, Effect};
use fret_core::Px;
use fret_diag::devtools_gate_profiles_v1;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::command_catalog::{
    CMD_COPY_GATE_RESULT_COMMAND, CMD_COPY_GATE_RESULT_JSON, CMD_COPY_GATE_RESULT_PATH,
    CMD_GATE_RUN_GENERATED,
    CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH, CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH,
    CMD_COPY_WORKFLOW_RESULT_COMMAND, CMD_COPY_WORKFLOW_RESULT_JSON, CMD_COPY_WORKFLOW_RESULT_PATH,
    CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH, CMD_COPY_WORKFLOW_SUMMARIZE_COMMAND,
    CMD_LOAD_WORKFLOW_REGRESSION_INDEX, CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY,
    CMD_OPEN_GATE_RESULT_JSON,
    CMD_OPEN_WORKFLOW_REGRESSION_INDEX, CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY,
    CMD_OPEN_WORKFLOW_RESULT_JSON, CMD_OPEN_WORKFLOW_SUITE_SUMMARY, CMD_RUN_WORKFLOW_SUMMARIZE,
    CMD_WORKFLOW_RUN_SELECTED,
};
use super::demo_metrics_debug::devtools_demo_metrics_debug_panel;
use super::discovery_lines::{devtools_gate_command_lines, devtools_workflow_run_lines};
use super::gate_profile_state::{collect_gate_profile_panel_state, gate_profile_select_items};
use super::guide_recent_evidence_panel::guide_recent_evidence_panel;
use super::guide_reference_panels::{dogfood_reference_panel, first_open_reference_panel};
use super::run_history_panel::{gate_run_history_list, workflow_run_history_list};
use super::ui_primitives::{diag_section, text_blob_sized};
use super::workflow_panel_state::collect_workflow_panel_state;
use super::{State, gate_run, workflow_run};

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

fn devtools_gate_profile_command_builder(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let panel = collect_gate_profile_panel_state(cx.app, st);
    let profile_items = gate_profile_select_items()
        .into_iter()
        .map(|(id, label)| shadcn::SelectItem::new(id, label))
        .collect::<Vec<_>>();
    let profile_select =
        shadcn::Select::new(st.gate_profile_selected_id.clone(), st.gate_profile_open.clone())
            .value(shadcn::SelectValue::new().placeholder("Gate profile"))
            .items(profile_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(260.0)))
            .into_element(cx);
    let gate_inputs = match panel.selected_profile_id.as_ref() {
        "perf-thresholds" => perf_threshold_gate_inputs(cx, st),
        "resource-footprint-thresholds" => resource_footprint_threshold_gate_inputs(cx, st),
        _ => script_target_gate_inputs(cx, st),
    };
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
    let gate_run_result_history = cx
        .app
        .models()
        .read(&st.gate_run_result_history, |v| v.clone())
        .unwrap_or_default();
    let gate_result_actions = ui::h_row(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if panel.selected_gate_run_result_path.is_some() {
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
        if panel.selected_gate_run_result_entry.is_some() {
            out.push(
                shadcn::Button::new("Copy gate command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_GATE_RESULT_COMMAND)
                    .into_element(cx),
            );
        }
        if !panel.selected_gate_run_result_json.trim().is_empty() {
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
            panel.selected_gate_run_result_entry.as_ref(),
        )
        .join("\n"),
        Px(78.0),
    );
    let gate_result_summary = text_blob_sized(
        cx,
        gate_run::gate_run_result_summary_lines(&panel.selected_gate_run_result_json).join("\n"),
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
        panel.selected_gate_run_result_path.as_deref(),
    );
    let gate_run_status_line = format!(
        "gate_run_in_flight={} last_gate_result={} last_gate_error={}",
        panel.gate_run_in_flight,
        gate_run_result_path.as_deref().unwrap_or("-"),
        gate_run_error.as_deref().unwrap_or("-")
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
    let copy_button = shadcn::Button::new("Copy generated command")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!panel.copy_enabled)
        .on_activate(on_copy)
        .into_element(cx);
    let run_button = shadcn::Button::new("Run generated command")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!panel.run_enabled || panel.gate_run_in_flight)
        .on_click(CMD_GATE_RUN_GENERATED)
        .into_element(cx);
    let controls = ui::h_row(|_cx| [profile_select, copy_button, run_button])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let preview = text_blob_sized(cx, panel.command_preview.clone(), Px(58.0));
    let result_preview = text_blob_sized(
        cx,
        if panel.selected_gate_run_result_json.trim().is_empty() {
            "<no generated gate result yet>".to_string()
        } else {
            panel.selected_gate_run_result_json.clone()
        },
        Px(92.0),
    );
    ui::v_stack(|cx| {
        [
            cx.text(format!(
                "Runnable generated gate: {}",
                panel.selected_profile_label
            )),
            controls,
            gate_inputs,
            cx.text(panel.command_state_line.clone()),
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
