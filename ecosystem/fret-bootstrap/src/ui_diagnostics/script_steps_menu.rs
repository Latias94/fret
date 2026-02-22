use super::*;

pub(super) fn handle_menu_select_step(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::MenuSelect {
        menu,
        item,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-menu_select-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::MenuSelect(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2MenuSelectState {
            step_index,
            remaining_frames: timeout_frames,
            phase: 0,
        },
    };

    match state.phase {
        0 => {
            if select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &menu,
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            )
            .is_some()
            {
                state.phase = 1;
                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label =
                    Some(format!("script-step-{step_index:04}-menu_select-timeout"));
                *stop_script = true;
                *failure_reason = Some("menu_select_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                output.request_redraw = true;
            }
        }
        1 => {
            if let Some(node) = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &menu,
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            ) {
                let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &menu,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some("menu_select.menu_click"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                output
                    .events
                    .extend(click_events(pos, UiMouseButtonV1::Left, 1));
                state.phase = 2;
                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-menu_select-menu-no-match"
                ));
                *stop_script = true;
                *failure_reason = Some("menu_select_menu_no_match".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            }
        }
        2 => {
            if select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &item,
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            )
            .is_some()
            {
                state.phase = 3;
                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label =
                    Some(format!("script-step-{step_index:04}-menu_select-timeout"));
                *stop_script = true;
                *failure_reason = Some("menu_select_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                output.request_redraw = true;
            }
        }
        _ => {
            if let Some(node) = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &item,
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            ) {
                let pos = center_of_rect(node.bounds);
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &item,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some("menu_select.item_click"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                output
                    .events
                    .extend(click_events(pos, UiMouseButtonV1::Left, 1));
                active.v2_step_state = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-menu_select"));
                }
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-menu_select-item-no-match"
                ));
                *stop_script = true;
                *failure_reason = Some("menu_select_item_no_match".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            }
        }
    }

    true
}
