use super::*;

pub(super) fn handle_assert_clipboard_text_step(
    service: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::AssertClipboardText {
        text,
        timeout_frames,
    } = step
    else {
        return false;
    };

    if cfg!(target_arch = "wasm32") {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_clipboard_text-not-supported"
        ));
        *stop_script = true;
        *failure_reason = Some("clipboard_text_not_supported_wasm".to_string());
        output.request_redraw = true;
        return true;
    }

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::AssertClipboardText(state)) if state.step_index == step_index => state,
        _ => V2AssertClipboardTextState::new(step_index, window, text, timeout_frames),
    };

    if !state.request_issued {
        let token = service.allocate_clipboard_token();
        state.token = Some(token);
        output
            .effects
            .push(Effect::ClipboardGetText { window, token });
        state.request_issued = true;
        active.wait_until = None;
        active.screenshot_wait = None;
        active.v2_step_state = Some(V2StepState::AssertClipboardText(state));
        output.request_redraw = true;
        return true;
    }

    let Some(token) = state.token else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_clipboard_text-missing-token"
        ));
        *stop_script = true;
        *failure_reason = Some("clipboard_text_internal_missing_token".to_string());
        output.request_redraw = true;
        return true;
    };

    if let Some(response) = service.clipboard_text_response_for_token(token) {
        match response {
            DiagClipboardTextResponseKind::Text(actual) => {
                if actual == &state.expected_text {
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.v2_step_state = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    return true;
                }

                let step_index_u32 = step_index.min(u32::MAX as usize) as u32;
                push_script_event_log(
                    active,
                    &service.cfg,
                    UiScriptEventLogEntryV1 {
                        unix_ms: unix_ms_now(),
                        kind: "clipboard_text_mismatch".to_string(),
                        step_index: Some(step_index_u32),
                        note: Some(format!(
                            "expected_len={} actual_len={}",
                            state.expected_text.len(),
                            actual.len()
                        )),
                        bundle_dir: None,
                        window: Some(window.data().as_ffi()),
                        tick_id: Some(app.tick_id().0),
                        frame_id: Some(app.frame_id().0),
                        window_snapshot_seq: None,
                    },
                );

                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-assert_clipboard_text-mismatch"
                ));
                *stop_script = true;
                *failure_reason = Some("clipboard_text_mismatch".to_string());
                output.request_redraw = true;
                return true;
            }
            DiagClipboardTextResponseKind::Unavailable { message } => {
                let step_index_u32 = step_index.min(u32::MAX as usize) as u32;
                let mut note = message
                    .as_deref()
                    .unwrap_or("clipboard text unavailable")
                    .to_string();
                truncate_string_bytes(&mut note, 512);
                push_script_event_log(
                    active,
                    &service.cfg,
                    UiScriptEventLogEntryV1 {
                        unix_ms: unix_ms_now(),
                        kind: "clipboard_text_unavailable".to_string(),
                        step_index: Some(step_index_u32),
                        note: Some(note),
                        bundle_dir: None,
                        window: Some(window.data().as_ffi()),
                        tick_id: Some(app.tick_id().0),
                        frame_id: Some(app.frame_id().0),
                        window_snapshot_seq: None,
                    },
                );

                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-assert_clipboard_text-unavailable"
                ));
                *stop_script = true;
                *failure_reason = Some("clipboard_text_unavailable".to_string());
                output.request_redraw = true;
                return true;
            }
        }
    }

    state.remaining_frames = state.remaining_frames.saturating_sub(1);
    if state.remaining_frames == 0 {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_clipboard_text-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("clipboard_text_timeout".to_string());
        output.request_redraw = true;
        return true;
    }

    // Keep waiting.
    let state_window = state.window;
    active.v2_step_state = Some(V2StepState::AssertClipboardText(state));
    output.request_redraw = true;

    // Keep the app producing frames while we wait for the runner response.
    output.effects.push(Effect::RequestAnimationFrame(state_window));
    true
}
