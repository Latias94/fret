use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardWriteResultStepMode {
    Wait,
    Assert,
}

fn clipboard_access_error_kind_to_diag(
    kind: fret_core::ClipboardAccessErrorKind,
) -> fret_diag_protocol::UiClipboardAccessErrorKindV1 {
    match kind {
        fret_core::ClipboardAccessErrorKind::Unavailable => {
            fret_diag_protocol::UiClipboardAccessErrorKindV1::Unavailable
        }
        fret_core::ClipboardAccessErrorKind::PermissionDenied => {
            fret_diag_protocol::UiClipboardAccessErrorKindV1::PermissionDenied
        }
        fret_core::ClipboardAccessErrorKind::UserActivationRequired => {
            fret_diag_protocol::UiClipboardAccessErrorKindV1::UserActivationRequired
        }
        fret_core::ClipboardAccessErrorKind::Unsupported => {
            fret_diag_protocol::UiClipboardAccessErrorKindV1::Unsupported
        }
        fret_core::ClipboardAccessErrorKind::BackendError => {
            fret_diag_protocol::UiClipboardAccessErrorKindV1::BackendError
        }
        fret_core::ClipboardAccessErrorKind::Unknown => {
            fret_diag_protocol::UiClipboardAccessErrorKindV1::Unknown
        }
    }
}

fn clipboard_write_completion_matches(
    completion: &ObservedClipboardWriteCompletion,
    expected_outcome: fret_diag_protocol::UiClipboardWriteResultV1,
    expected_error_kind: Option<fret_diag_protocol::UiClipboardAccessErrorKindV1>,
    expected_message_contains: Option<&str>,
) -> bool {
    match (&completion.outcome, expected_outcome) {
        (
            fret_core::ClipboardWriteOutcome::Succeeded,
            fret_diag_protocol::UiClipboardWriteResultV1::Success,
        ) => true,
        (
            fret_core::ClipboardWriteOutcome::Failed { error },
            fret_diag_protocol::UiClipboardWriteResultV1::Failure,
        ) => {
            let error_kind_matches = expected_error_kind.is_none_or(|expected_kind| {
                clipboard_access_error_kind_to_diag(error.kind) == expected_kind
            });
            let message_matches = expected_message_contains.is_none_or(|needle| {
                error
                    .message
                    .as_deref()
                    .is_some_and(|message| message.contains(needle))
            });
            error_kind_matches && message_matches
        }
        _ => false,
    }
}

fn describe_clipboard_write_expectation(
    expected_outcome: fret_diag_protocol::UiClipboardWriteResultV1,
    expected_error_kind: Option<fret_diag_protocol::UiClipboardAccessErrorKindV1>,
    expected_message_contains: Option<&str>,
) -> String {
    match expected_outcome {
        fret_diag_protocol::UiClipboardWriteResultV1::Success => "success".to_string(),
        fret_diag_protocol::UiClipboardWriteResultV1::Failure => {
            let mut note = String::from("failure");
            if let Some(kind) = expected_error_kind {
                note.push_str(&format!(" kind={kind:?}"));
            }
            if let Some(message_contains) = expected_message_contains {
                note.push_str(&format!(" message_contains={message_contains:?}"));
            }
            note
        }
    }
}

fn describe_clipboard_write_completion(completion: &ObservedClipboardWriteCompletion) -> String {
    match &completion.outcome {
        fret_core::ClipboardWriteOutcome::Succeeded => {
            format!("success token={}", completion.token.0)
        }
        fret_core::ClipboardWriteOutcome::Failed { error } => format!(
            "failure token={} kind={:?} message={:?}",
            completion.token.0, error.kind, error.message
        ),
    }
}

fn fail_invalid_clipboard_write_expectation(
    step_name: &str,
    step_index: usize,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
    output: &mut UiScriptFrameOutput,
) -> bool {
    *force_dump_label = Some(format!(
        "script-step-{step_index:04}-{step_name}-invalid-expectation"
    ));
    *stop_script = true;
    *failure_reason = Some("clipboard_write_result_invalid_expectation".to_string());
    output.request_redraw = true;
    true
}

fn fail_clipboard_write_result_step(
    service: &UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step_name: &str,
    failure_kind: &str,
    note: Option<String>,
    active: &mut ActiveScript,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
    output: &mut UiScriptFrameOutput,
) -> bool {
    let step_index_u32 = step_index.min(u32::MAX as usize) as u32;
    if let Some(note) = note.clone() {
        push_script_event_log(
            active,
            &service.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: format!("{step_name}_{failure_kind}"),
                step_index: Some(step_index_u32),
                note: Some(note),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
    }
    *force_dump_label = Some(format!(
        "script-step-{step_index:04}-{step_name}-{}",
        failure_kind.replace('_', "-")
    ));
    *stop_script = true;
    *failure_reason = Some(format!("{step_name}_{failure_kind}"));
    output.request_redraw = true;
    true
}

fn observed_completion_from_diag(
    completion: &DiagClipboardWriteCompletion,
) -> ObservedClipboardWriteCompletion {
    ObservedClipboardWriteCompletion {
        seq: completion.seq,
        token: completion.token,
        outcome: completion.outcome.clone(),
    }
}

fn handle_clipboard_write_result_step(
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
    mode: ClipboardWriteResultStepMode,
) -> bool {
    let (step_name, outcome, error_kind, message_contains, timeout_frames) = match (mode, step) {
        (
            ClipboardWriteResultStepMode::Wait,
            UiActionStepV2::WaitClipboardWriteResult {
                outcome,
                error_kind,
                message_contains,
                timeout_frames,
            },
        ) => (
            "wait_clipboard_write_result",
            outcome,
            error_kind,
            message_contains.filter(|value| !value.is_empty()),
            timeout_frames,
        ),
        (
            ClipboardWriteResultStepMode::Assert,
            UiActionStepV2::AssertClipboardWriteResult {
                outcome,
                error_kind,
                message_contains,
                timeout_frames,
            },
        ) => (
            "assert_clipboard_write_result",
            outcome,
            error_kind,
            message_contains.filter(|value| !value.is_empty()),
            timeout_frames,
        ),
        _ => return false,
    };

    if matches!(
        outcome,
        fret_diag_protocol::UiClipboardWriteResultV1::Success
    ) && (error_kind.is_some() || message_contains.is_some())
    {
        return fail_invalid_clipboard_write_expectation(
            step_name,
            step_index,
            force_dump_label,
            stop_script,
            failure_reason,
            output,
        );
    }

    let mut state = match (mode, active.v2_step_state.take()) {
        (
            ClipboardWriteResultStepMode::Wait,
            Some(V2StepState::WaitClipboardWriteResult(state)),
        ) if state.step_index == step_index => state,
        (
            ClipboardWriteResultStepMode::Assert,
            Some(V2StepState::AssertClipboardWriteResult(state)),
        ) if state.step_index == step_index => state,
        _ => V2ClipboardWriteResultState::new(
            step_index,
            outcome,
            error_kind,
            message_contains,
            timeout_frames,
            service.latest_clipboard_write_completion_seq(),
        ),
    };

    if matches!(mode, ClipboardWriteResultStepMode::Assert)
        && state.last_seen_seq == state.start_after_seq
        && let Some(completion) = active.last_clipboard_write_completion.clone()
    {
        if clipboard_write_completion_matches(
            &completion,
            state.expected_outcome,
            state.expected_error_kind,
            state.expected_message_contains.as_deref(),
        ) {
            active.v2_step_state = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            return true;
        }

        let mut note = format!(
            "expected={} actual={}",
            describe_clipboard_write_expectation(
                state.expected_outcome,
                state.expected_error_kind,
                state.expected_message_contains.as_deref(),
            ),
            describe_clipboard_write_completion(&completion)
        );
        truncate_string_bytes(&mut note, 512);
        return fail_clipboard_write_result_step(
            service,
            app,
            window,
            step_index,
            step_name,
            "mismatch",
            Some(note),
            active,
            force_dump_label,
            stop_script,
            failure_reason,
            output,
        );
    }

    for completion in service.clipboard_write_completions_after(state.last_seen_seq) {
        let observed = observed_completion_from_diag(completion);
        state.last_seen_seq = observed.seq;

        if clipboard_write_completion_matches(
            &observed,
            state.expected_outcome,
            state.expected_error_kind,
            state.expected_message_contains.as_deref(),
        ) {
            active.last_clipboard_write_completion = Some(observed);
            active.v2_step_state = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            return true;
        }

        state.last_observed = Some(observed);
    }

    state.remaining_frames = state.remaining_frames.saturating_sub(1);
    if state.remaining_frames == 0 {
        if let Some(observed) = state.last_observed.as_ref() {
            let mut note = format!(
                "expected={} actual={}",
                describe_clipboard_write_expectation(
                    state.expected_outcome,
                    state.expected_error_kind,
                    state.expected_message_contains.as_deref(),
                ),
                describe_clipboard_write_completion(observed)
            );
            truncate_string_bytes(&mut note, 512);
            return fail_clipboard_write_result_step(
                service,
                app,
                window,
                step_index,
                step_name,
                "mismatch",
                Some(note),
                active,
                force_dump_label,
                stop_script,
                failure_reason,
                output,
            );
        }

        return fail_clipboard_write_result_step(
            service,
            app,
            window,
            step_index,
            step_name,
            "timeout",
            None,
            active,
            force_dump_label,
            stop_script,
            failure_reason,
            output,
        );
    }

    active.v2_step_state = Some(match mode {
        ClipboardWriteResultStepMode::Wait => V2StepState::WaitClipboardWriteResult(state),
        ClipboardWriteResultStepMode::Assert => V2StepState::AssertClipboardWriteResult(state),
    });
    output.request_redraw = true;
    output.effects.push(Effect::RequestAnimationFrame(window));
    true
}

pub(super) fn handle_clipboard_steps(
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
    match step {
        step @ UiActionStepV2::WaitClipboardWriteResult { .. } => {
            handle_clipboard_write_result_step(
                service,
                app,
                window,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
                ClipboardWriteResultStepMode::Wait,
            )
        }
        step @ UiActionStepV2::AssertClipboardWriteResult { .. } => {
            handle_clipboard_write_result_step(
                service,
                app,
                window,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
                ClipboardWriteResultStepMode::Assert,
            )
        }
        step @ UiActionStepV2::AssertClipboardText { .. } => handle_assert_clipboard_text_step(
            service,
            app,
            window,
            step_index,
            step,
            active,
            output,
            force_dump_label,
            stop_script,
            failure_reason,
        ),
        _ => false,
    }
}

fn handle_assert_clipboard_text_step(
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
            .push(Effect::ClipboardReadText { window, token });
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
                state.saw_text_response = true;
                if actual == &state.expected_text {
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.v2_step_state = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    return true;
                }

                state.last_text_len = Some(actual.len());
                state.request_issued = false;
                state.token = None;
            }
            DiagClipboardTextResponseKind::Unavailable { message } => {
                state.saw_unavailable_response = true;
                let mut note = message
                    .as_deref()
                    .unwrap_or("clipboard text unavailable")
                    .to_string();
                truncate_string_bytes(&mut note, 512);
                state.last_unavailable_message = Some(note);
                state.request_issued = false;
                state.token = None;
            }
        }
    }

    state.remaining_frames = state.remaining_frames.saturating_sub(1);
    if state.remaining_frames == 0 {
        let step_index_u32 = step_index.min(u32::MAX as usize) as u32;

        if state.saw_text_response {
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
                        state.last_text_len.unwrap_or(0)
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
            *failure_reason = Some("clipboard_text_mismatch".to_string());
        } else if state.saw_unavailable_response {
            let note = state
                .last_unavailable_message
                .clone()
                .unwrap_or_else(|| "clipboard text unavailable".to_string());
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
            *failure_reason = Some("clipboard_text_unavailable".to_string());
        } else {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-assert_clipboard_text-timeout"
            ));
            *failure_reason = Some("clipboard_text_timeout".to_string());
        }
        *stop_script = true;
        output.request_redraw = true;
        return true;
    }

    let state_window = state.window;
    active.v2_step_state = Some(V2StepState::AssertClipboardText(state));
    output.request_redraw = true;
    output
        .effects
        .push(Effect::RequestAnimationFrame(state_window));
    true
}
