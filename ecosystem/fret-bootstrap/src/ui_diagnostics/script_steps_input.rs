use super::*;

pub(super) fn handle_keyboard_text_steps(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    match step {
        UiActionStepV2::PressKey {
            key,
            modifiers,
            repeat,
        } => {
            if let Some(key) = parse_key_code(&key) {
                let note = format!("press_key key={key:?} mods={modifiers:?} repeat={repeat}");
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    semantics_snapshot,
                    ui,
                    step_index as u32,
                    None,
                    None,
                    note.as_str(),
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    note.as_str(),
                );
                record_overlay_placement_trace(
                    &mut active.overlay_placement_trace,
                    element_runtime,
                    semantics_snapshot,
                    window,
                    step_index as u32,
                    note.as_str(),
                );
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output
                    .events
                    .extend(press_key_events(key, modifiers, repeat));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-press_key"));
                }
            } else {
                *force_dump_label =
                    Some(format!("script-step-{step_index:04}-press_key-unknown-key"));
                *stop_script = true;
                *failure_reason = Some(format!("unknown_key: {key}"));
                output.request_redraw = true;
            }
            true
        }
        UiActionStepV2::PressKeys {
            keys,
            modifiers,
            repeat,
        } => {
            let mut events = Vec::with_capacity(keys.len().saturating_mul(2));
            let mut parsed_keys = Vec::with_capacity(keys.len());
            for key in &keys {
                let Some(parsed) = parse_key_code(key) else {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-press_keys-unknown-key"
                    ));
                    *stop_script = true;
                    *failure_reason = Some(format!("unknown_key: {key}"));
                    output.request_redraw = true;
                    return true;
                };
                parsed_keys.push(parsed);
                events.extend(press_key_events(parsed, modifiers, repeat));
            }

            let note =
                format!("press_keys keys={parsed_keys:?} mods={modifiers:?} repeat={repeat}");
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                semantics_snapshot,
                ui,
                step_index as u32,
                None,
                None,
                note.as_str(),
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                note.as_str(),
            );
            record_overlay_placement_trace(
                &mut active.overlay_placement_trace,
                element_runtime,
                semantics_snapshot,
                window,
                step_index as u32,
                note.as_str(),
            );
            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            output.events.extend(events);
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-press_keys"));
            }
            true
        }
        UiActionStepV2::PressShortcut { shortcut, repeat } => {
            active.wait_until = None;
            active.screenshot_wait = None;

            if let Some((key, modifiers)) = parse_shortcut(&shortcut) {
                let note = format!("press_shortcut key={key:?} mods={modifiers:?} repeat={repeat}");
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    semantics_snapshot,
                    ui,
                    step_index as u32,
                    None,
                    None,
                    note.as_str(),
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    note.as_str(),
                );
                record_overlay_placement_trace(
                    &mut active.overlay_placement_trace,
                    element_runtime,
                    semantics_snapshot,
                    window,
                    step_index as u32,
                    note.as_str(),
                );
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output
                    .events
                    .extend(press_key_events(key, modifiers, repeat));
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-press_shortcut"));
                }
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-press_shortcut-parse-failed"
                ));
                *stop_script = true;
                *failure_reason = Some(format!("invalid_shortcut: {shortcut}"));
                output.request_redraw = true;
            }
            true
        }
        UiActionStepV2::TypeText { text } => {
            output.events.push(Event::TextInput(text));
            active.wait_until = None;
            active.screenshot_wait = None;
            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-type_text"));
            }
            true
        }
        UiActionStepV2::Ime { event } => {
            active.wait_until = None;
            active.screenshot_wait = None;

            let note = format!("ime_event kind={}", ime_event_kind_name(&event));
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                semantics_snapshot,
                ui,
                step_index as u32,
                None,
                None,
                note.as_str(),
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                note.as_str(),
            );
            record_overlay_placement_trace(
                &mut active.overlay_placement_trace,
                element_runtime,
                semantics_snapshot,
                window,
                step_index as u32,
                note.as_str(),
            );

            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            output.events.push(Event::Ime(ime_event_from_v1(&event)));
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-ime"));
            }
            true
        }
        _ => false,
    }
}

pub(super) fn handle_type_text_into_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
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
    let UiActionStepV2::TypeTextInto {
        window: _,
        pointer_kind,
        target,
        text,
        clear_before_type,
        timeout_frames,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-type_text_into-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::TypeTextInto(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2TypeTextIntoState {
            step_index,
            remaining_frames: timeout_frames,
            phase: 0,
            expected_node_id: None,
            expected_test_id: None,
        },
    };

    match state.phase {
        0 => {
            if select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &target,
                active.scope_root_for_window(window),
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            )
            .is_some()
            {
                state.phase = 1;
                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-type_text_into-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some("type_text_into_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                output.request_redraw = true;
            }
        }
        1 => {
            if let Some(node) = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &target,
                active.scope_root_for_window(window),
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            ) {
                state.expected_node_id = Some(node.id.data().as_ffi());
                state.expected_test_id = node.test_id.clone();

                let pos = if let Some(ui_ref) = ui.as_deref() {
                    pointer_position_prefer_intended_hit(
                        app,
                        snapshot,
                        element_runtime,
                        ui_ref,
                        window,
                        node,
                        window_bounds,
                    )
                } else {
                    center_of_rect_clamped_to_rect(
                        interaction_bounds_for_semantics_node(element_runtime, None, window, node),
                        window_bounds,
                    )
                };
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some("type_text_into.click"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    Some(snapshot),
                    ui.as_deref(),
                    step_index as u32,
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    "type_text_into.click_injected",
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    "type_text_into.click_injected",
                );
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output
                    .events
                    .extend(click_events(pos, UiMouseButtonV1::Left, 1, pointer_type));
                state.phase = 2;
                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-type_text_into-no-semantics-match"
                ));
                *stop_script = true;
                *failure_reason = Some("type_text_into_no_semantics_match".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            }
        }
        _ => {
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                Some(snapshot),
                ui.as_deref(),
                step_index as u32,
                state.expected_node_id,
                state.expected_test_id.as_deref(),
                "type_text_into.wait_focus",
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                "type_text_into.wait_focus",
            );

            let focused_node_id = element_runtime
                .and_then(|rt| rt.diagnostics_snapshot(window))
                .and_then(|s| s.focused_element_node)
                .map(key_to_u64);
            let focus_matches = match (
                state.expected_node_id,
                focused_node_id,
                element_runtime
                    .and_then(|rt| rt.diagnostics_snapshot(window))
                    .is_some(),
            ) {
                (Some(expected), Some(focused), _) => expected == focused,
                (Some(_), None, true) => false,
                _ => true,
            };

            if focus_matches {
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    Some(snapshot),
                    ui.as_deref(),
                    step_index as u32,
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    "type_text_into.text_input",
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    "type_text_into.text_input",
                );

                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                if clear_before_type {
                    output.events.push(Event::SetTextSelection {
                        anchor: 0,
                        focus: u32::MAX,
                    });
                }
                output.events.push(Event::TextInput(text));
                active.v2_step_state = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-type_text_into"));
                }
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-type_text_into-focus-timeout"
                ));
                push_text_input_timeout_event(
                    svc,
                    app,
                    window,
                    active,
                    "type_text_into.focus_timeout",
                    step_index,
                    Some(state.remaining_frames),
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    None,
                );
                *stop_script = true;
                *failure_reason = Some("type_text_into_focus_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                output.request_redraw = true;
            }
        }
    }

    true
}

pub(super) fn handle_set_text_value_step(
    svc: &mut UiDiagnosticsService,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::SetTextValue {
        window: _,
        target,
        text,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::SetTextValue(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2SetTextValueState {
            step_index,
            remaining_frames: timeout_frames,
        },
    };

    let Some(snapshot) = semantics_snapshot else {
        if state.remaining_frames == 0 {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-set_text_value-timeout"
            ));
            push_set_text_value_failure_event(
                svc,
                app,
                window,
                active,
                "set_text_value.no_semantics_timeout",
                step_index,
                Some(state.remaining_frames),
                None,
            );
            *stop_script = true;
            *failure_reason = Some("set_text_value_timeout".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        } else {
            state.remaining_frames = state.remaining_frames.saturating_sub(1);
            active.v2_step_state = Some(V2StepState::SetTextValue(state));
            output.request_redraw = true;
        }
        return true;
    };

    let Some(node) = select_semantics_node_with_trace(
        snapshot,
        window,
        element_runtime,
        &target,
        active.scope_root_for_window(window),
        step_index as u32,
        svc.cfg.redact_text,
        &mut active.selector_resolution_trace,
    ) else {
        if state.remaining_frames == 0 {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-set_text_value-timeout"
            ));
            push_set_text_value_failure_event(
                svc,
                app,
                window,
                active,
                "set_text_value.selector_timeout",
                step_index,
                Some(state.remaining_frames),
                None,
            );
            *stop_script = true;
            *failure_reason = Some("set_text_value_timeout".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        } else {
            state.remaining_frames = state.remaining_frames.saturating_sub(1);
            active.v2_step_state = Some(V2StepState::SetTextValue(state));
            output.request_redraw = true;
        }
        return true;
    };

    if node.flags.disabled {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-set_text_value-disabled"
        ));
        push_set_text_value_failure_event(
            svc,
            app,
            window,
            active,
            "set_text_value.disabled",
            step_index,
            Some(state.remaining_frames),
            Some(node),
        );
        *stop_script = true;
        *failure_reason = Some("set_text_value_disabled".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    }

    if !node.actions.set_value {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-set_text_value-unsupported"
        ));
        push_set_text_value_failure_event(
            svc,
            app,
            window,
            active,
            "set_text_value.unsupported",
            step_index,
            Some(state.remaining_frames),
            Some(node),
        );
        *stop_script = true;
        *failure_reason = Some("set_text_value_unsupported".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    }

    let Some(ui) = ui else {
        *force_dump_label = Some(format!("script-step-{step_index:04}-set_text_value-no-ui"));
        push_set_text_value_failure_event(
            svc,
            app,
            window,
            active,
            "set_text_value.no_ui_tree",
            step_index,
            Some(state.remaining_frames),
            Some(node),
        );
        *stop_script = true;
        *failure_reason = Some("set_text_value_no_ui".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
    fret_ui_app::accessibility_actions::set_value_text(ui, app, services, node.id, text.as_str());
    record_focus_trace(
        &mut active.focus_trace,
        app,
        window,
        element_runtime,
        Some(snapshot),
        Some(ui),
        step_index as u32,
        Some(node.id.data().as_ffi()),
        node.test_id.as_deref(),
        "set_text_value.accessibility_set_value",
    );
    record_web_ime_trace(
        &mut active.web_ime_trace,
        app,
        step_index as u32,
        "set_text_value.accessibility_set_value",
    );

    active.v2_step_state = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-set_text_value"));
    }

    true
}

pub(super) fn handle_paste_text_into_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
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
    let UiActionStepV2::PasteTextInto {
        window: _,
        pointer_kind,
        target,
        text,
        clear_before_paste,
        timeout_frames,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-paste_text_into-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::PasteTextInto(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2PasteTextIntoState {
            step_index,
            remaining_frames: timeout_frames,
            phase: 0,
            expected_node_id: None,
            expected_test_id: None,
            clipboard_token: None,
        },
    };

    match state.phase {
        0 => {
            if select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &target,
                active.scope_root_for_window(window),
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            )
            .is_some()
            {
                state.phase = 1;
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-paste_text_into-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some("paste_text_into_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            }
        }
        1 => {
            if let Some(node) = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &target,
                active.scope_root_for_window(window),
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            ) {
                state.expected_node_id = Some(node.id.data().as_ffi());
                state.expected_test_id = node.test_id.clone();

                let pos = if let Some(ui_ref) = ui.as_deref() {
                    pointer_position_prefer_intended_hit(
                        app,
                        snapshot,
                        element_runtime,
                        ui_ref,
                        window,
                        node,
                        window_bounds,
                    )
                } else {
                    center_of_rect_clamped_to_rect(
                        interaction_bounds_for_semantics_node(element_runtime, None, window, node),
                        window_bounds,
                    )
                };
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some("paste_text_into.click"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    Some(snapshot),
                    ui.as_deref(),
                    step_index as u32,
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    "paste_text_into.click_injected",
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    "paste_text_into.click_injected",
                );
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output
                    .events
                    .extend(click_events(pos, UiMouseButtonV1::Left, 1, pointer_type));
                state.phase = 2;
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-paste_text_into-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some("paste_text_into_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            }
        }
        2 => {
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                Some(snapshot),
                ui.as_deref(),
                step_index as u32,
                state.expected_node_id,
                state.expected_test_id.as_deref(),
                "paste_text_into.wait_focus",
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                "paste_text_into.wait_focus",
            );

            let focused_node_id = element_runtime
                .and_then(|rt| rt.diagnostics_snapshot(window))
                .and_then(|s| s.focused_element_node)
                .map(key_to_u64);
            let focus_matches = match (
                state.expected_node_id,
                focused_node_id,
                element_runtime
                    .and_then(|rt| rt.diagnostics_snapshot(window))
                    .is_some(),
            ) {
                (Some(expected), Some(focused), _) => expected == focused,
                (Some(_), None, true) => false,
                _ => true,
            };

            if focus_matches {
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    Some(snapshot),
                    ui.as_deref(),
                    step_index as u32,
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    "paste_text_into.clipboard_write_requested",
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    "paste_text_into.clipboard_write_requested",
                );

                if clear_before_paste {
                    output.events.push(Event::SetTextSelection {
                        anchor: 0,
                        focus: u32::MAX,
                    });
                }

                let token = svc.allocate_clipboard_token();
                output.effects.push(Effect::ClipboardWriteText {
                    window,
                    token,
                    text,
                });
                state.clipboard_token = Some(token);
                state.phase = 3;
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-paste_text_into-focus-timeout"
                ));
                push_text_input_timeout_event(
                    svc,
                    app,
                    window,
                    active,
                    "paste_text_into.focus_timeout",
                    step_index,
                    Some(state.remaining_frames),
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    state.clipboard_token,
                );
                *stop_script = true;
                *failure_reason = Some("paste_text_into_focus_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            }
        }
        3 => {
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                Some(snapshot),
                ui.as_deref(),
                step_index as u32,
                state.expected_node_id,
                state.expected_test_id.as_deref(),
                "paste_text_into.wait_clipboard_write",
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                "paste_text_into.wait_clipboard_write",
            );

            let Some(token) = state.clipboard_token else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-paste_text_into-missing-clipboard-token"
                ));
                *stop_script = true;
                *failure_reason =
                    Some("paste_text_into_internal_missing_clipboard_token".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
                return true;
            };

            if let Some(outcome) = svc.clipboard_write_completion_for_token(token) {
                match outcome {
                    fret_core::ClipboardWriteOutcome::Succeeded => {
                        record_focus_trace(
                            &mut active.focus_trace,
                            app,
                            window,
                            element_runtime,
                            Some(snapshot),
                            ui.as_deref(),
                            step_index as u32,
                            state.expected_node_id,
                            state.expected_test_id.as_deref(),
                            "paste_text_into.clipboard_write_succeeded",
                        );
                        record_web_ime_trace(
                            &mut active.web_ime_trace,
                            app,
                            step_index as u32,
                            "paste_text_into.clipboard_write_succeeded",
                        );
                        state.phase = 4;
                        active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                        output.request_redraw = true;
                    }
                    fret_core::ClipboardWriteOutcome::Failed { error } => {
                        let mut note = error
                            .message
                            .clone()
                            .unwrap_or_else(|| format!("clipboard_write_failed:{:?}", error.kind));
                        truncate_string_bytes(&mut note, 512);
                        push_script_event_log(
                            active,
                            &svc.cfg,
                            UiScriptEventLogEntryV1 {
                                unix_ms: unix_ms_now(),
                                kind: "clipboard_write_failed".to_string(),
                                step_index: Some(step_index.min(u32::MAX as usize) as u32),
                                note: Some(note),
                                bundle_dir: None,
                                window: Some(window.data().as_ffi()),
                                tick_id: Some(app.tick_id().0),
                                frame_id: Some(app.frame_id().0),
                                window_snapshot_seq: None,
                            },
                        );
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-paste_text_into-clipboard-write-failed"
                        ));
                        *stop_script = true;
                        *failure_reason =
                            Some("paste_text_into_clipboard_write_failed".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    }
                }
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-paste_text_into-clipboard-write-timeout"
                ));
                push_text_input_timeout_event(
                    svc,
                    app,
                    window,
                    active,
                    "paste_text_into.clipboard_write_timeout",
                    step_index,
                    Some(state.remaining_frames),
                    state.expected_node_id,
                    state.expected_test_id.as_deref(),
                    state.clipboard_token,
                );
                *stop_script = true;
                *failure_reason = Some("paste_text_into_clipboard_write_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
                output.effects.push(Effect::RequestAnimationFrame(window));
            }
        }
        4 => {
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                Some(snapshot),
                ui.as_deref(),
                step_index as u32,
                state.expected_node_id,
                state.expected_test_id.as_deref(),
                "paste_text_into.paste_shortcut",
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                "paste_text_into.paste_shortcut",
            );
            record_overlay_placement_trace(
                &mut active.overlay_placement_trace,
                element_runtime,
                Some(snapshot),
                window,
                step_index as u32,
                "paste_text_into.paste_shortcut",
            );

            let mut mods = UiKeyModifiersV1::default();
            if cfg!(target_os = "macos") {
                mods.meta = true;
            } else {
                mods.ctrl = true;
            }

            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            output
                .events
                .extend(press_key_events(KeyCode::KeyV, mods, false));
            active.v2_step_state = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-paste_text_into"));
            }
        }
        _ => {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-paste_text_into-internal-invalid-phase"
            ));
            *stop_script = true;
            *failure_reason = Some("paste_text_into_internal_invalid_phase".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        }
    }

    true
}

fn push_set_text_value_failure_event(
    svc: &UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    active: &mut ActiveScript,
    kind: &'static str,
    step_index: usize,
    remaining_frames: Option<u32>,
    node: Option<&fret_core::SemanticsNode>,
) {
    let step_index = step_index.min(u32::MAX as usize) as u32;
    let note = set_text_value_failure_note(
        kind,
        step_index,
        remaining_frames,
        node,
        &active.selector_resolution_trace,
    );

    push_script_event_log(
        active,
        &svc.cfg,
        UiScriptEventLogEntryV1 {
            unix_ms: unix_ms_now(),
            kind: kind.to_string(),
            step_index: Some(step_index),
            note: Some(note),
            bundle_dir: None,
            window: Some(window.data().as_ffi()),
            tick_id: Some(app.tick_id().0),
            frame_id: Some(app.frame_id().0),
            window_snapshot_seq: None,
        },
    );
}

fn set_text_value_failure_note(
    kind: &'static str,
    step_index: u32,
    remaining_frames: Option<u32>,
    node: Option<&fret_core::SemanticsNode>,
    selector_trace: &[UiSelectorResolutionTraceEntryV1],
) -> String {
    let selector = selector_resolution_summary_note(selector_trace, step_index)
        .unwrap_or_else(|| "selector_resolution_trace=none".to_string());
    let node = node
        .map(set_text_value_node_summary)
        .unwrap_or_else(|| "node=none".to_string());

    format!(
        "kind={kind} step_index={step_index} remaining_frames={remaining_frames:?} {node} selector=[{selector}]"
    )
}

fn set_text_value_node_summary(node: &fret_core::SemanticsNode) -> String {
    format!(
        "node_id={} role={} test_id={:?} disabled={} read_only={} focused={} value_len={} text_selection={:?} actions={{focus:{} invoke:{} set_value:{} set_text_selection:{}}}",
        node.id.data().as_ffi(),
        semantics_role_label(node.role),
        node.test_id.as_deref(),
        node.flags.disabled,
        node.flags.read_only,
        node.flags.focused,
        node.value.as_ref().map(|value| value.len()).unwrap_or(0),
        node.text_selection,
        node.actions.focus,
        node.actions.invoke,
        node.actions.set_value,
        node.actions.set_text_selection,
    )
}

fn selector_resolution_summary_note(
    trace: &[UiSelectorResolutionTraceEntryV1],
    step_index: u32,
) -> Option<String> {
    let entry = trace
        .iter()
        .rev()
        .find(|entry| entry.step_index == step_index)?;
    let candidates = entry
        .candidates
        .iter()
        .map(|candidate| {
            format!(
                "{{node_id:{} role:{} name:{:?} test_id:{:?}}}",
                candidate.node_id,
                candidate.role,
                candidate.name.as_deref(),
                candidate.test_id.as_deref()
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    Some(format!(
        "selector={:?} match_count={} chosen_node_id={:?} note={:?} candidates=[{}]",
        entry.selector,
        entry.match_count,
        entry.chosen_node_id,
        entry.note.as_deref(),
        candidates
    ))
}

fn push_text_input_timeout_event(
    svc: &UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    active: &mut ActiveScript,
    kind: &'static str,
    step_index: usize,
    remaining_frames: Option<u32>,
    expected_node_id: Option<u64>,
    expected_test_id: Option<&str>,
    clipboard_token: Option<fret_core::ClipboardToken>,
) {
    let step_index = step_index.min(u32::MAX as usize) as u32;
    let note = text_input_timeout_note(
        kind,
        step_index,
        remaining_frames,
        expected_node_id,
        expected_test_id,
        clipboard_token.is_some(),
        &active.focus_trace,
        &active.web_ime_trace,
    );

    push_script_event_log(
        active,
        &svc.cfg,
        UiScriptEventLogEntryV1 {
            unix_ms: unix_ms_now(),
            kind: kind.to_string(),
            step_index: Some(step_index),
            note: Some(note),
            bundle_dir: None,
            window: Some(window.data().as_ffi()),
            tick_id: Some(app.tick_id().0),
            frame_id: Some(app.frame_id().0),
            window_snapshot_seq: None,
        },
    );
}

fn text_input_timeout_note(
    kind: &'static str,
    step_index: u32,
    remaining_frames: Option<u32>,
    expected_node_id: Option<u64>,
    expected_test_id: Option<&str>,
    clipboard_token_present: bool,
    focus_trace: &[UiFocusTraceEntryV1],
    web_ime_trace: &[UiWebImeTraceEntryV1],
) -> String {
    let focus = focus_trace_summary_note(focus_trace, step_index)
        .unwrap_or_else(|| "focus_trace=none".to_string());
    let web_ime = web_ime_trace_summary_note(web_ime_trace, step_index)
        .unwrap_or_else(|| "web_ime_trace=none".to_string());

    format!(
        "kind={kind} step_index={step_index} remaining_frames={remaining_frames:?} expected_node_id={expected_node_id:?} expected_test_id={expected_test_id:?} clipboard_token_present={clipboard_token_present} focus=[{focus}] web_ime=[{web_ime}]"
    )
}

#[cfg(test)]
mod set_text_value_failure_tests {
    use super::*;
    use fret_core::{NodeId, Px, Rect, Size};
    use slotmap::KeyData;

    fn selector_trace(step_index: u32) -> UiSelectorResolutionTraceEntryV1 {
        UiSelectorResolutionTraceEntryV1 {
            step_index,
            selector: UiSelectorV1::TestId {
                id: "field.search".to_string(),
                root_z_index: None,
            },
            match_count: 2,
            chosen_node_id: Some(42),
            candidates: vec![
                UiSelectorResolutionCandidateV1 {
                    node_id: 42,
                    role: "text_input".to_string(),
                    name: Some("Search".to_string()),
                    test_id: Some("field.search".to_string()),
                },
                UiSelectorResolutionCandidateV1 {
                    node_id: 41,
                    role: "button".to_string(),
                    name: Some("Search".to_string()),
                    test_id: Some("button.search".to_string()),
                },
            ],
            note: Some("fallback_chrome_suffix".to_string()),
        }
    }

    fn semantics_node() -> fret_core::SemanticsNode {
        fret_core::SemanticsNode {
            id: NodeId::from(KeyData::from_ffi(42)),
            parent: None,
            role: SemanticsRole::TextField,
            bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(24.0))),
            flags: fret_core::SemanticsFlags {
                disabled: true,
                read_only: true,
                focused: false,
                ..Default::default()
            },
            test_id: Some("field.search".to_string()),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: Some("Search".to_string()),
            value: Some("abc".to_string()),
            extra: fret_core::SemanticsNodeExtra::default(),
            text_selection: Some((1, 2)),
            text_composition: None,
            actions: fret_core::SemanticsActions {
                focus: true,
                invoke: false,
                set_value: false,
                set_text_selection: true,
                ..Default::default()
            },
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        }
    }

    #[test]
    fn set_text_value_failure_note_summarizes_selector_candidates() {
        let note = set_text_value_failure_note(
            "set_text_value.selector_timeout",
            5,
            Some(0),
            None,
            &[selector_trace(5)],
        );

        assert!(note.contains("kind=set_text_value.selector_timeout"));
        assert!(note.contains("node=none"));
        assert!(note.contains("match_count=2"));
        assert!(note.contains("chosen_node_id=Some(42)"));
        assert!(note.contains("note=Some(\"fallback_chrome_suffix\")"));
        assert!(note.contains("test_id:Some(\"field.search\")"));
    }

    #[test]
    fn set_text_value_failure_note_summarizes_action_capability() {
        let node = semantics_node();
        let note = set_text_value_failure_note(
            "set_text_value.unsupported",
            5,
            Some(0),
            Some(&node),
            &[selector_trace(5)],
        );

        assert!(note.contains("node_id=42"));
        assert!(note.contains("role=text_field"));
        assert!(note.contains("test_id=Some(\"field.search\")"));
        assert!(note.contains("disabled=true"));
        assert!(note.contains("read_only=true"));
        assert!(note.contains("value_len=3"));
        assert!(note.contains("actions={focus:true invoke:false set_value:false"));
    }

    #[test]
    fn set_text_value_failure_note_reports_missing_selector_trace() {
        let note = set_text_value_failure_note(
            "set_text_value.no_semantics_timeout",
            7,
            Some(0),
            None,
            &[],
        );

        assert!(note.contains("selector=[selector_resolution_trace=none]"));
    }
}

#[cfg(test)]
mod text_input_timeout_tests {
    use super::*;

    fn focus_entry(step_index: u32) -> UiFocusTraceEntryV1 {
        UiFocusTraceEntryV1 {
            step_index,
            note: Some("type_text_into.wait_focus".to_string()),
            reason_code: Some("focus.mismatch".to_string()),
            text_input_snapshot: Some(UiTextInputSnapshotV1 {
                focus_is_text_input: false,
                is_composing: false,
                text_len_utf16: 0,
                selection_utf16: None,
                marked_utf16: None,
                ime_cursor_area: None,
                visual: None,
                ime_surrounding_text_len_bytes: None,
                ime_surrounding_cursor_bytes: None,
                ime_surrounding_anchor_bytes: None,
            }),
            expected_node_id: Some(42),
            expected_test_id: Some("search.input".to_string()),
            modal_barrier_root: None,
            focus_barrier_root: Some(7),
            pointer_occlusion: Some("blocks_underlay_input".to_string()),
            pointer_occlusion_layer_id: Some(8),
            pointer_capture_active: Some(false),
            pointer_capture_layer_id: None,
            pointer_capture_multiple_layers: Some(false),
            focused_element: Some(9),
            focused_element_path: Some("Root/Other".to_string()),
            focused_node_id: Some(43),
            focused_test_id: Some("other.input".to_string()),
            focused_role: Some("text_input".to_string()),
            matches_expected: Some(false),
        }
    }

    fn web_ime_entry(step_index: u32) -> UiWebImeTraceEntryV1 {
        UiWebImeTraceEntryV1 {
            step_index,
            note: Some("paste_text_into.wait_clipboard_write".to_string()),
            enabled: true,
            composing: false,
            suppress_next_input: false,
            textarea_has_focus: Some(true),
            active_element_tag: Some("TEXTAREA".to_string()),
            position_mode: Some("cursor".to_string()),
            mount_kind: Some("hidden_textarea".to_string()),
            device_pixel_ratio: Some(1.5),
            textarea_selection_start_utf16: Some(0),
            textarea_selection_end_utf16: Some(0),
            last_cursor_area: None,
            last_cursor_anchor_px: None,
            last_input_type: Some("insertFromPaste".to_string()),
            last_preedit_len: None,
            last_preedit_cursor_utf16: None,
            last_commit_len: Some(6),
            beforeinput_seen: 1,
            input_seen: 1,
            suppressed_input_seen: 0,
            composition_start_seen: 0,
            composition_update_seen: 0,
            composition_end_seen: 0,
            cursor_area_set_seen: 1,
        }
    }

    #[test]
    fn text_input_timeout_note_summarizes_focus_and_web_ime_evidence() {
        let note = text_input_timeout_note(
            "paste_text_into.clipboard_write_timeout",
            12,
            Some(0),
            Some(42),
            Some("search.input"),
            true,
            &[focus_entry(12)],
            &[web_ime_entry(12)],
        );

        assert!(note.contains("kind=paste_text_into.clipboard_write_timeout"));
        assert!(note.contains("expected_test_id=Some(\"search.input\")"));
        assert!(note.contains("reason_code=Some(\"focus.mismatch\")"));
        assert!(note.contains("focused_test_id=Some(\"other.input\")"));
        assert!(note.contains("focus_barrier_root=Some(7)"));
        assert!(note.contains("textarea_has_focus=Some(true)"));
        assert!(note.contains("last_input_type=Some(\"insertFromPaste\")"));
        assert!(note.contains("clipboard_token_present=true"));
    }

    #[test]
    fn text_input_timeout_note_reports_missing_traces() {
        let note = text_input_timeout_note(
            "type_text_into.focus_timeout",
            99,
            Some(0),
            Some(42),
            Some("search.input"),
            false,
            &[],
            &[],
        );

        assert!(note.contains("focus=[focus_trace=none]"));
        assert!(note.contains("web_ime=[web_ime_trace=none]"));
    }
}
