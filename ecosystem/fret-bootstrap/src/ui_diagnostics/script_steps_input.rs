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

                let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
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

                let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
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
                    "paste_text_into.clipboard_set",
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    "paste_text_into.clipboard_set",
                );

                if clear_before_paste {
                    output.events.push(Event::SetTextSelection {
                        anchor: 0,
                        focus: u32::MAX,
                    });
                }

                output.effects.push(Effect::ClipboardSetText { text });
                state.phase = 3;
                active.v2_step_state = Some(V2StepState::PasteTextInto(state));
                output.request_redraw = true;
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-paste_text_into-focus-timeout"
                ));
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
