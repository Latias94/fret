use super::*;

pub(super) fn handle_wait_bounds_stable_step(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::WaitBoundsStable {
        window: _,
        target,
        stable_frames,
        max_move_px,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    if let Some(snapshot) = semantics_snapshot {
        let stable_required = stable_frames.max(1);
        let max_move_px = max_move_px.max(0.0);

        if timeout_frames != 0 && stable_required > timeout_frames {
            push_bounds_stable_trace(
                &mut active.bounds_stable_trace,
                UiBoundsStableTraceEntryV1 {
                    step_index: step_index as u32,
                    selector: target.clone(),
                    stable_required,
                    stable_count: 0,
                    moved_px: 0.0,
                    max_move_px,
                    remaining_frames: timeout_frames,
                    bounds: None,
                    note: Some(
                        "wait_bounds_stable.impossible.stable_frames_gt_timeout_frames".to_string(),
                    ),
                },
            );

            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-wait_bounds_stable-impossible-stable-frames-gt-timeout"
            ));
            *stop_script = true;
            *failure_reason =
                Some("wait_bounds_stable_impossible_stable_frames_gt_timeout_frames".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
            return true;
        }

        let mut state = match active.v2_step_state.take() {
            Some(V2StepState::WaitBoundsStable(mut state)) if state.step_index == step_index => {
                state.remaining_frames = state.remaining_frames.min(timeout_frames);
                state
            }
            _ => V2WaitBoundsStableState {
                step_index,
                remaining_frames: timeout_frames,
                stable_count: 0,
                last_bounds: None,
            },
        };

        let node = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &target,
            active.scope_root_for_window(window),
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        );

        if state.remaining_frames == 0 {
            push_bounds_stable_trace(
                &mut active.bounds_stable_trace,
                UiBoundsStableTraceEntryV1 {
                    step_index: step_index as u32,
                    selector: target.clone(),
                    stable_required,
                    stable_count: state.stable_count,
                    moved_px: 0.0,
                    max_move_px,
                    remaining_frames: state.remaining_frames,
                    bounds: node.map(|n| UiRectV1 {
                        x_px: n.bounds.origin.x.0,
                        y_px: n.bounds.origin.y.0,
                        w_px: n.bounds.size.width.0,
                        h_px: n.bounds.size.height.0,
                    }),
                    note: Some("wait_bounds_stable.timeout".to_string()),
                },
            );

            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-wait_bounds_stable-timeout"
            ));
            *stop_script = true;
            *failure_reason = Some("wait_bounds_stable_timeout".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        } else if let Some(node) = node {
            let bounds = node.bounds;
            let moved = match state.last_bounds {
                Some(last) => {
                    let dx = (bounds.origin.x.0 - last.origin.x.0).abs();
                    let dy = (bounds.origin.y.0 - last.origin.y.0).abs();
                    let dw = (bounds.size.width.0 - last.size.width.0).abs();
                    let dh = (bounds.size.height.0 - last.size.height.0).abs();
                    dx.max(dy).max(dw).max(dh)
                }
                None => 0.0,
            };

            if moved <= max_move_px {
                state.stable_count = state.stable_count.saturating_add(1);
            } else {
                state.stable_count = 1;
            }
            state.last_bounds = Some(bounds);

            push_bounds_stable_trace(
                &mut active.bounds_stable_trace,
                UiBoundsStableTraceEntryV1 {
                    step_index: step_index as u32,
                    selector: target.clone(),
                    stable_required,
                    stable_count: state.stable_count,
                    moved_px: moved,
                    max_move_px,
                    remaining_frames: state.remaining_frames,
                    bounds: Some(UiRectV1 {
                        x_px: bounds.origin.x.0,
                        y_px: bounds.origin.y.0,
                        w_px: bounds.size.width.0,
                        h_px: bounds.size.height.0,
                    }),
                    note: Some("wait_bounds_stable.waiting".to_string()),
                },
            );

            if state.stable_count >= stable_required {
                active.v2_step_state = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label =
                        Some(format!("script-step-{step_index:04}-wait_bounds_stable"));
                }
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::WaitBoundsStable(state));
                output.request_redraw = true;
            }
        } else {
            push_bounds_stable_trace(
                &mut active.bounds_stable_trace,
                UiBoundsStableTraceEntryV1 {
                    step_index: step_index as u32,
                    selector: target.clone(),
                    stable_required,
                    stable_count: 0,
                    moved_px: 0.0,
                    max_move_px,
                    remaining_frames: state.remaining_frames,
                    bounds: None,
                    note: Some("wait_bounds_stable.no_semantics_match".to_string()),
                },
            );

            if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-wait_bounds_stable-no-semantics-match"
                ));
                *stop_script = true;
                *failure_reason = Some("wait_bounds_stable_no_semantics_match".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::WaitBoundsStable(state));
                output.request_redraw = true;
            }
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-wait_bounds_stable-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    true
}

pub(super) fn handle_wait_semantics_scroll_stable_step(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::WaitSemanticsScrollStable {
        window: _,
        target,
        field,
        stable_frames,
        max_delta,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    if let Some(snapshot) = semantics_snapshot {
        let stable_required = stable_frames.max(1);
        let max_delta = max_delta.abs();

        if timeout_frames != 0 && stable_required > timeout_frames {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-wait_semantics_scroll_stable-impossible-stable-frames-gt-timeout"
            ));
            *stop_script = true;
            *failure_reason = Some(
                "wait_semantics_scroll_stable_impossible_stable_frames_gt_timeout_frames"
                    .to_string(),
            );
            active.v2_step_state = None;
            output.request_redraw = true;
            return true;
        }

        let mut state = match active.v2_step_state.take() {
            Some(V2StepState::WaitSemanticsScrollStable(mut state))
                if state.step_index == step_index =>
            {
                state.remaining_frames = state.remaining_frames.min(timeout_frames);
                state
            }
            _ => V2WaitSemanticsScrollStableState {
                step_index,
                remaining_frames: timeout_frames,
                stable_count: 0,
                last_value: None,
            },
        };

        let node = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &target,
            active.scope_root_for_window(window),
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        );

        if state.remaining_frames == 0 {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-wait_semantics_scroll_stable-timeout"
            ));
            *stop_script = true;
            *failure_reason = Some("wait_semantics_scroll_stable_timeout".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        } else if let Some(node) = node {
            let value = semantics_scroll_field_value(node, field).filter(|v| v.is_finite());
            if let Some(value) = value {
                let delta = state
                    .last_value
                    .map(|last| (value - last).abs())
                    .unwrap_or(0.0);
                if delta <= max_delta {
                    state.stable_count = state.stable_count.saturating_add(1);
                } else {
                    state.stable_count = 1;
                }
                state.last_value = Some(value);

                if state.stable_count >= stable_required {
                    active.v2_step_state = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    if svc.cfg.script_auto_dump {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-wait_semantics_scroll_stable"
                        ));
                    }
                } else {
                    state.remaining_frames = state.remaining_frames.saturating_sub(1);
                    active.v2_step_state = Some(V2StepState::WaitSemanticsScrollStable(state));
                    output.request_redraw = true;
                }
            } else {
                state.stable_count = 0;
                state.last_value = None;
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::WaitSemanticsScrollStable(state));
                output.request_redraw = true;
            }
        } else {
            state.stable_count = 0;
            state.last_value = None;
            state.remaining_frames = state.remaining_frames.saturating_sub(1);
            active.v2_step_state = Some(V2StepState::WaitSemanticsScrollStable(state));
            output.request_redraw = true;
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-wait_semantics_scroll_stable-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    true
}

pub(super) fn handle_assert_semantics_scroll_idle_stable_step(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::AssertSemanticsScrollIdleStable {
        window: _,
        target,
        field,
        frames,
        max_delta,
        max_total_delta,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_semantics_scroll_idle_stable-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let required_samples = frames.max(1);
    let max_delta = max_delta.abs();
    let max_total_delta = max_total_delta.abs();

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::AssertSemanticsScrollIdleStable(state))
            if state.step_index == step_index =>
        {
            state
        }
        _ => V2AssertSemanticsScrollIdleStableState {
            step_index,
            sample_count: 0,
            baseline_value: None,
            last_value: None,
        },
    };

    let node = select_semantics_node_with_trace(
        snapshot,
        window,
        element_runtime,
        &target,
        active.scope_root_for_window(window),
        step_index as u32,
        svc.cfg.redact_text,
        &mut active.selector_resolution_trace,
    );

    let Some(node) = node else {
        push_semantics_scroll_idle_stable_trace(
            &mut active.semantics_scroll_idle_stable_trace,
            UiSemanticsScrollIdleStableTraceEntryV1 {
                step_index: step_index as u32,
                selector: target.clone(),
                field,
                sample_count: state.sample_count,
                required_samples,
                baseline_value: state.baseline_value,
                value: None,
                frame_delta: None,
                total_delta: None,
                max_delta,
                max_total_delta,
                bounds: None,
                note: Some("assert_semantics_scroll_idle_stable.no_semantics_match".to_string()),
            },
        );
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_semantics_scroll_idle_stable-no-semantics-match"
        ));
        *stop_script = true;
        *failure_reason =
            Some("assert_semantics_scroll_idle_stable_no_semantics_match".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let value = semantics_scroll_field_value(node, field).filter(|v| v.is_finite());
    let Some(value) = value else {
        push_semantics_scroll_idle_stable_trace(
            &mut active.semantics_scroll_idle_stable_trace,
            UiSemanticsScrollIdleStableTraceEntryV1 {
                step_index: step_index as u32,
                selector: target.clone(),
                field,
                sample_count: state.sample_count,
                required_samples,
                baseline_value: state.baseline_value,
                value: None,
                frame_delta: None,
                total_delta: None,
                max_delta,
                max_total_delta,
                bounds: Some(UiRectV1 {
                    x_px: node.bounds.origin.x.0,
                    y_px: node.bounds.origin.y.0,
                    w_px: node.bounds.size.width.0,
                    h_px: node.bounds.size.height.0,
                }),
                note: Some("assert_semantics_scroll_idle_stable.missing_scroll_field".to_string()),
            },
        );
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_semantics_scroll_idle_stable-missing-scroll-field"
        ));
        *stop_script = true;
        *failure_reason =
            Some("assert_semantics_scroll_idle_stable_missing_scroll_field".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let baseline = state.baseline_value.unwrap_or(value);
    let frame_delta = state
        .last_value
        .map(|last| (value - last).abs())
        .unwrap_or(0.0);
    let total_delta = (value - baseline).abs();
    state.baseline_value = Some(baseline);
    state.last_value = Some(value);
    state.sample_count = state.sample_count.saturating_add(1);

    let bounds = UiRectV1 {
        x_px: node.bounds.origin.x.0,
        y_px: node.bounds.origin.y.0,
        w_px: node.bounds.size.width.0,
        h_px: node.bounds.size.height.0,
    };

    let note = if frame_delta > max_delta {
        "assert_semantics_scroll_idle_stable.frame_delta_exceeded"
    } else if total_delta > max_total_delta {
        "assert_semantics_scroll_idle_stable.total_delta_exceeded"
    } else {
        "assert_semantics_scroll_idle_stable.sample"
    };

    push_semantics_scroll_idle_stable_trace(
        &mut active.semantics_scroll_idle_stable_trace,
        UiSemanticsScrollIdleStableTraceEntryV1 {
            step_index: step_index as u32,
            selector: target,
            field,
            sample_count: state.sample_count,
            required_samples,
            baseline_value: Some(baseline),
            value: Some(value),
            frame_delta: Some(frame_delta),
            total_delta: Some(total_delta),
            max_delta,
            max_total_delta,
            bounds: Some(bounds),
            note: Some(note.to_string()),
        },
    );

    if frame_delta > max_delta {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_semantics_scroll_idle_stable-frame-delta-exceeded"
        ));
        *stop_script = true;
        *failure_reason =
            Some("assert_semantics_scroll_idle_stable_frame_delta_exceeded".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    } else if total_delta > max_total_delta {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert_semantics_scroll_idle_stable-total-delta-exceeded"
        ));
        *stop_script = true;
        *failure_reason =
            Some("assert_semantics_scroll_idle_stable_total_delta_exceeded".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    } else if state.sample_count >= required_samples {
        active.v2_step_state = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-assert_semantics_scroll_idle_stable"
            ));
        }
    } else {
        active.v2_step_state = Some(V2StepState::AssertSemanticsScrollIdleStable(state));
        output.request_redraw = true;
    }

    true
}

pub(super) fn handle_wait_shortcut_routing_trace_step(
    cfg: &UiDiagnosticsConfig,
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
    let UiActionStepV2::WaitShortcutRoutingTrace {
        query,
        timeout_frames,
        timeout_ms,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let state = match active.wait_shortcut_routing_trace.take() {
        Some(mut state) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            if state.deadline_unix_ms.is_none() {
                state.deadline_unix_ms =
                    timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64));
            }
            state
        }
        _ => WaitShortcutRoutingTraceState {
            step_index,
            remaining_frames: timeout_frames,
            deadline_unix_ms: timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64)),
            start_frame_id: app.frame_id().0.saturating_sub(1),
        },
    };

    let found = active.shortcut_routing_trace.iter().any(|entry| {
        entry.frame_id >= state.start_frame_id
            && shortcut_routing_trace_entry_matches_query(entry, &query)
    });

    if found {
        active.wait_shortcut_routing_trace = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    } else if state
        .deadline_unix_ms
        .is_some_and(|deadline| unix_ms_now() >= deadline)
        || state.remaining_frames == 0
    {
        if let Some(note) = shortcut_routing_trace_timeout_note(
            &active.shortcut_routing_trace,
            &query,
            state.start_frame_id,
        ) {
            push_script_event_log(
                active,
                cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "wait_shortcut_routing_trace.candidate_mismatch".to_string(),
                    step_index: Some(step_index.min(u32::MAX as usize) as u32),
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
            "script-step-{step_index:04}-wait_shortcut_routing_trace-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("wait_shortcut_routing_trace_timeout".to_string());
        active.wait_shortcut_routing_trace = None;
        output.request_redraw = true;
    } else {
        active.wait_shortcut_routing_trace = Some(WaitShortcutRoutingTraceState {
            step_index: state.step_index,
            remaining_frames: state.remaining_frames.saturating_sub(1),
            deadline_unix_ms: state.deadline_unix_ms,
            start_frame_id: state.start_frame_id,
        });
        output.request_redraw = true;
    }

    true
}

pub(super) fn handle_wait_command_dispatch_trace_step(
    cfg: &UiDiagnosticsConfig,
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
    let UiActionStepV2::WaitCommandDispatchTrace {
        query,
        timeout_frames,
        timeout_ms,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let state = match active.wait_command_dispatch_trace.take() {
        Some(mut state) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            if state.deadline_unix_ms.is_none() {
                state.deadline_unix_ms =
                    timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64));
            }
            state
        }
        _ => WaitCommandDispatchTraceState {
            step_index,
            remaining_frames: timeout_frames,
            deadline_unix_ms: timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64)),
            start_frame_id: app.frame_id().0.saturating_sub(1),
        },
    };

    let found = active.command_dispatch_trace.iter().any(|entry| {
        entry.frame_id >= state.start_frame_id
            && command_dispatch_trace_entry_matches_query(entry, &query)
    });

    if found {
        active.wait_command_dispatch_trace = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    } else if state
        .deadline_unix_ms
        .is_some_and(|deadline| unix_ms_now() >= deadline)
        || state.remaining_frames == 0
    {
        if let Some(note) = command_dispatch_trace_timeout_note(
            &active.command_dispatch_trace,
            &query,
            state.start_frame_id,
        ) {
            push_script_event_log(
                active,
                cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "wait_command_dispatch_trace.candidate_mismatch".to_string(),
                    step_index: Some(step_index.min(u32::MAX as usize) as u32),
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
            "script-step-{step_index:04}-wait_command_dispatch_trace-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("wait_command_dispatch_trace_timeout".to_string());
        active.wait_command_dispatch_trace = None;
        output.request_redraw = true;
    } else {
        active.wait_command_dispatch_trace = Some(WaitCommandDispatchTraceState {
            step_index: state.step_index,
            remaining_frames: state.remaining_frames.saturating_sub(1),
            deadline_unix_ms: state.deadline_unix_ms,
            start_frame_id: state.start_frame_id,
        });
        output.request_redraw = true;
    }

    true
}

pub(super) fn shortcut_routing_trace_timeout_note(
    trace: &[UiShortcutRoutingTraceEntryV1],
    query: &UiShortcutRoutingTraceQueryV1,
    start_frame_id: u64,
) -> Option<String> {
    let (candidate, mismatches) = trace
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.frame_id >= start_frame_id)
        .filter_map(|(index, entry)| {
            let mismatches = shortcut_routing_trace_query_mismatches(entry, query);
            if mismatches.is_empty() {
                None
            } else {
                let score = shortcut_routing_trace_query_field_count(query) as i32
                    - mismatches.len() as i32;
                Some(((score, index), entry, mismatches))
            }
        })
        .max_by_key(|(rank, _, _)| *rank)
        .map(|(_, entry, mismatches)| (entry, mismatches))?;

    Some(format!(
        "query={} start_frame_id={} trace_count={} best_candidate={} mismatches={}",
        shortcut_routing_query_summary(query),
        start_frame_id,
        trace
            .iter()
            .filter(|entry| entry.frame_id >= start_frame_id)
            .count(),
        shortcut_routing_trace_candidate_summary(candidate),
        mismatches.join("; ")
    ))
}

fn shortcut_routing_trace_query_field_count(query: &UiShortcutRoutingTraceQueryV1) -> usize {
    query.phase.is_some() as usize
        + query.outcome.is_some() as usize
        + query.key.is_some() as usize
        + query.command.is_some() as usize
        + query.ime_composing.is_some() as usize
        + query.focus_is_text_input.is_some() as usize
        + query.key_context.is_some() as usize
}

fn shortcut_routing_trace_query_mismatches(
    entry: &UiShortcutRoutingTraceEntryV1,
    query: &UiShortcutRoutingTraceQueryV1,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    push_string_mismatch(
        &mut mismatches,
        "phase",
        query.phase.as_deref(),
        Some(entry.phase.as_str()),
    );
    push_string_mismatch(
        &mut mismatches,
        "outcome",
        query.outcome.as_deref(),
        Some(entry.outcome.as_str()),
    );
    push_string_mismatch(
        &mut mismatches,
        "key",
        query.key.as_deref(),
        Some(entry.key.as_str()),
    );
    push_string_mismatch(
        &mut mismatches,
        "command",
        query.command.as_deref(),
        entry.command.as_deref(),
    );
    push_bool_mismatch(
        &mut mismatches,
        "ime_composing",
        query.ime_composing,
        entry.ime_composing,
    );
    push_bool_mismatch(
        &mut mismatches,
        "focus_is_text_input",
        query.focus_is_text_input,
        entry.focus_is_text_input,
    );
    if let Some(expected) = query.key_context.as_deref()
        && !entry.key_contexts.iter().any(|ctx| ctx == expected)
    {
        mismatches.push(format!(
            "key_context expected {:?} actual {:?}",
            Some(expected),
            entry.key_contexts
        ));
    }
    mismatches
}

fn shortcut_routing_query_summary(query: &UiShortcutRoutingTraceQueryV1) -> String {
    format!(
        "phase={:?} outcome={:?} key={:?} command={:?} ime_composing={:?} focus_is_text_input={:?} key_context={:?}",
        query.phase.as_deref(),
        query.outcome.as_deref(),
        query.key.as_deref(),
        query.command.as_deref(),
        query.ime_composing,
        query.focus_is_text_input,
        query.key_context.as_deref(),
    )
}

fn shortcut_routing_trace_candidate_summary(entry: &UiShortcutRoutingTraceEntryV1) -> String {
    format!(
        "step_index={} frame_id={} phase={:?} outcome={:?} key={:?} command={:?} ime_composing={} focus_is_text_input={} key_contexts={:?} deferred={} repeat={} command_enabled={:?} pending_sequence_len={:?}",
        entry.step_index,
        entry.frame_id,
        entry.phase,
        entry.outcome,
        entry.key,
        entry.command.as_deref(),
        entry.ime_composing,
        entry.focus_is_text_input,
        entry.key_contexts,
        entry.deferred,
        entry.repeat,
        entry.command_enabled,
        entry.pending_sequence_len
    )
}

pub(super) fn command_dispatch_trace_timeout_note(
    trace: &[UiScriptCommandDispatchTraceEntryV1],
    query: &UiScriptCommandDispatchTraceQueryV1,
    start_frame_id: u64,
) -> Option<String> {
    let (candidate, mismatches) = trace
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.frame_id >= start_frame_id)
        .filter_map(|(index, entry)| {
            let mismatches = command_dispatch_trace_query_mismatches(entry, query);
            if mismatches.is_empty() {
                None
            } else {
                let score = command_dispatch_trace_query_field_count(query) as i32
                    - mismatches.len() as i32;
                Some(((score, index), entry, mismatches))
            }
        })
        .max_by_key(|(rank, _, _)| *rank)
        .map(|(_, entry, mismatches)| (entry, mismatches))?;

    Some(format!(
        "query={} start_frame_id={} trace_count={} best_candidate={} mismatches={}",
        command_dispatch_query_summary(query),
        start_frame_id,
        trace
            .iter()
            .filter(|entry| entry.frame_id >= start_frame_id)
            .count(),
        command_dispatch_trace_candidate_summary(candidate),
        mismatches.join("; ")
    ))
}

fn command_dispatch_trace_query_field_count(query: &UiScriptCommandDispatchTraceQueryV1) -> usize {
    query.command.is_some() as usize
        + query.source_kind.is_some() as usize
        + query.source_test_id.is_some() as usize
        + query.handled.is_some() as usize
        + query.handled_by_scope.is_some() as usize
        + query.handled_by_driver.is_some() as usize
        + query.handled_by_test_id.is_some() as usize
        + query.started_from_focus.is_some() as usize
        + query.used_default_root_fallback.is_some() as usize
}

fn command_dispatch_trace_query_mismatches(
    entry: &UiScriptCommandDispatchTraceEntryV1,
    query: &UiScriptCommandDispatchTraceQueryV1,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    push_string_mismatch(
        &mut mismatches,
        "command",
        query.command.as_deref(),
        Some(entry.command.as_str()),
    );
    push_string_mismatch(
        &mut mismatches,
        "source_kind",
        query.source_kind.as_deref(),
        Some(entry.source_kind.as_str()),
    );
    push_string_mismatch(
        &mut mismatches,
        "source_test_id",
        query.source_test_id.as_deref(),
        entry.source_test_id.as_deref(),
    );
    push_bool_mismatch(&mut mismatches, "handled", query.handled, entry.handled);
    push_string_mismatch(
        &mut mismatches,
        "handled_by_scope",
        query.handled_by_scope.as_deref(),
        entry.handled_by_scope.as_deref(),
    );
    push_bool_mismatch(
        &mut mismatches,
        "handled_by_driver",
        query.handled_by_driver,
        entry.handled_by_driver,
    );
    push_string_mismatch(
        &mut mismatches,
        "handled_by_test_id",
        query.handled_by_test_id.as_deref(),
        entry.handled_by_test_id.as_deref(),
    );
    push_bool_mismatch(
        &mut mismatches,
        "started_from_focus",
        query.started_from_focus,
        entry.started_from_focus,
    );
    push_bool_mismatch(
        &mut mismatches,
        "used_default_root_fallback",
        query.used_default_root_fallback,
        entry.used_default_root_fallback,
    );
    mismatches
}

fn command_dispatch_query_summary(query: &UiScriptCommandDispatchTraceQueryV1) -> String {
    format!(
        "command={:?} source_kind={:?} source_test_id={:?} handled={:?} handled_by_scope={:?} handled_by_driver={:?} handled_by_test_id={:?} started_from_focus={:?} used_default_root_fallback={:?}",
        query.command.as_deref(),
        query.source_kind.as_deref(),
        query.source_test_id.as_deref(),
        query.handled,
        query.handled_by_scope.as_deref(),
        query.handled_by_driver,
        query.handled_by_test_id.as_deref(),
        query.started_from_focus,
        query.used_default_root_fallback,
    )
}

fn command_dispatch_trace_candidate_summary(entry: &UiScriptCommandDispatchTraceEntryV1) -> String {
    format!(
        "step_index={} frame_id={} command={:?} source_kind={:?} source_test_id={:?} handled={} handled_by_scope={:?} handled_by_driver={} handled_by_test_id={:?} stopped={} started_from_focus={} used_default_root_fallback={}",
        entry.step_index,
        entry.frame_id,
        entry.command,
        entry.source_kind,
        entry.source_test_id.as_deref(),
        entry.handled,
        entry.handled_by_scope.as_deref(),
        entry.handled_by_driver,
        entry.handled_by_test_id.as_deref(),
        entry.stopped,
        entry.started_from_focus,
        entry.used_default_root_fallback
    )
}

fn push_string_mismatch(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if let Some(expected) = expected
        && actual != Some(expected)
    {
        mismatches.push(format!(
            "{field} expected {:?} actual {:?}",
            Some(expected),
            actual
        ));
    }
}

fn push_bool_mismatch(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: Option<bool>,
    actual: bool,
) {
    if let Some(expected) = expected
        && actual != expected
    {
        mismatches.push(format!("{field} expected {expected} actual {actual}"));
    }
}

pub(super) fn handle_wait_overlay_placement_trace_step(
    cfg: &UiDiagnosticsConfig,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::WaitOverlayPlacementTrace {
        query,
        timeout_frames,
        timeout_ms,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    if semantics_snapshot.is_none()
        && (query.anchor_test_id.is_some() || query.content_test_id.is_some())
    {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-wait_overlay_placement_trace-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        output.request_redraw = true;
        return true;
    }

    record_overlay_placement_trace(
        &mut active.overlay_placement_trace,
        element_runtime,
        semantics_snapshot,
        window,
        step_index as u32,
        "wait_overlay_placement_trace",
    );

    let state = match active.wait_overlay_placement_trace.take() {
        Some(mut state) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            if state.deadline_unix_ms.is_none() {
                state.deadline_unix_ms =
                    timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64));
            }
            state
        }
        _ => WaitOverlayPlacementTraceState {
            step_index,
            remaining_frames: timeout_frames,
            deadline_unix_ms: timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64)),
        },
    };

    let step_index_u32 = step_index.min(u32::MAX as usize) as u32;
    let found = active
        .overlay_placement_trace
        .iter()
        .any(|entry| overlay_placement_trace_entry_matches_query(entry, step_index_u32, &query));

    if found {
        active.wait_overlay_placement_trace = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    } else if state
        .deadline_unix_ms
        .is_some_and(|deadline| unix_ms_now() >= deadline)
        || state.remaining_frames == 0
    {
        if let Some(note) =
            overlay_placement_trace_timeout_note(&active.overlay_placement_trace, &query)
        {
            push_script_event_log(
                active,
                cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "wait_overlay_placement_trace.candidate_mismatch".to_string(),
                    step_index: Some(step_index_u32),
                    note: Some(note),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: None,
                    frame_id: None,
                    window_snapshot_seq: None,
                },
            );
        }
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-wait_overlay_placement_trace-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("wait_overlay_placement_trace_timeout".to_string());
        active.wait_overlay_placement_trace = None;
        output.request_redraw = true;
    } else {
        active.wait_overlay_placement_trace = Some(WaitOverlayPlacementTraceState {
            step_index: state.step_index,
            remaining_frames: state.remaining_frames.saturating_sub(1),
            deadline_unix_ms: state.deadline_unix_ms,
        });
        output.request_redraw = true;
    }

    true
}

pub(super) fn overlay_placement_trace_timeout_note(
    trace: &[UiOverlayPlacementTraceEntryV1],
    query: &UiOverlayPlacementTraceQueryV1,
) -> Option<String> {
    let (candidate, mismatches) = trace
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let mismatches = overlay_placement_trace_query_mismatches(entry, query);
            if mismatches.is_empty() {
                None
            } else {
                let score = overlay_placement_trace_candidate_score(query, mismatches.len());
                Some(((score, index), entry, mismatches))
            }
        })
        .max_by_key(|(rank, _, _)| *rank)
        .map(|(_, entry, mismatches)| (entry, mismatches))?;

    Some(format!(
        "query={} trace_count={} best_candidate={} mismatches={}",
        overlay_placement_query_summary(query),
        trace.len(),
        overlay_placement_trace_candidate_summary(candidate),
        mismatches.join("; ")
    ))
}

fn overlay_placement_trace_candidate_score(
    query: &UiOverlayPlacementTraceQueryV1,
    mismatch_count: usize,
) -> i32 {
    overlay_placement_trace_query_field_count(query) as i32 - mismatch_count as i32
}

fn overlay_placement_trace_query_field_count(query: &UiOverlayPlacementTraceQueryV1) -> usize {
    query.kind.is_some() as usize
        + query.overlay_root_name.is_some() as usize
        + query.anchor_test_id.is_some() as usize
        + query.content_test_id.is_some() as usize
        + query.preferred_side.is_some() as usize
        + query.chosen_side.is_some() as usize
        + query.side_offset_px.is_some() as usize
        + query.flipped.is_some() as usize
        + query.align.is_some() as usize
        + query.sticky.is_some() as usize
}

fn overlay_placement_trace_query_mismatches(
    entry: &UiOverlayPlacementTraceEntryV1,
    query: &UiOverlayPlacementTraceQueryV1,
) -> Vec<String> {
    match entry {
        UiOverlayPlacementTraceEntryV1::AnchoredPanel {
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            side_offset_px,
            preferred_side,
            chosen_side,
            align,
            sticky,
            ..
        } => {
            let mut mismatches = Vec::new();
            if let Some(kind) = query.kind
                && kind != UiOverlayPlacementTraceKindV1::AnchoredPanel
            {
                mismatches.push(format!(
                    "kind expected {kind:?} actual {:?}",
                    UiOverlayPlacementTraceKindV1::AnchoredPanel
                ));
            }
            push_option_string_mismatch(
                &mut mismatches,
                "overlay_root_name",
                query.overlay_root_name.as_deref(),
                overlay_root_name.as_deref(),
            );
            push_option_string_mismatch(
                &mut mismatches,
                "anchor_test_id",
                query.anchor_test_id.as_deref(),
                anchor_test_id.as_deref(),
            );
            push_option_string_mismatch(
                &mut mismatches,
                "content_test_id",
                query.content_test_id.as_deref(),
                content_test_id.as_deref(),
            );
            if let Some(expected) = query.preferred_side
                && *preferred_side != expected
            {
                mismatches.push(format!(
                    "preferred_side expected {expected:?} actual {preferred_side:?}"
                ));
            }
            if let Some(expected) = query.chosen_side
                && *chosen_side != expected
            {
                mismatches.push(format!(
                    "chosen_side expected {expected:?} actual {chosen_side:?}"
                ));
            }
            if let Some(expected) = query.side_offset_px {
                let eps = query.side_offset_eps_px.unwrap_or(0.001).max(0.0);
                if (*side_offset_px - expected).abs() > eps {
                    mismatches.push(format!(
                        "side_offset_px expected {expected}+/-{eps} actual {side_offset_px}"
                    ));
                }
            }
            if let Some(expected) = query.flipped {
                let actual = *chosen_side != *preferred_side;
                if actual != expected {
                    mismatches.push(format!("flipped expected {expected} actual {actual}"));
                }
            }
            if let Some(expected) = query.align
                && *align != expected
            {
                mismatches.push(format!("align expected {expected:?} actual {align:?}"));
            }
            if let Some(expected) = query.sticky
                && *sticky != expected
            {
                mismatches.push(format!("sticky expected {expected:?} actual {sticky:?}"));
            }
            mismatches
        }
        UiOverlayPlacementTraceEntryV1::PlacedRect {
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            side,
            ..
        } => {
            let mut mismatches = Vec::new();
            if let Some(kind) = query.kind
                && kind != UiOverlayPlacementTraceKindV1::PlacedRect
            {
                mismatches.push(format!(
                    "kind expected {kind:?} actual {:?}",
                    UiOverlayPlacementTraceKindV1::PlacedRect
                ));
            }
            push_option_string_mismatch(
                &mut mismatches,
                "overlay_root_name",
                query.overlay_root_name.as_deref(),
                overlay_root_name.as_deref(),
            );
            push_option_string_mismatch(
                &mut mismatches,
                "anchor_test_id",
                query.anchor_test_id.as_deref(),
                anchor_test_id.as_deref(),
            );
            push_option_string_mismatch(
                &mut mismatches,
                "content_test_id",
                query.content_test_id.as_deref(),
                content_test_id.as_deref(),
            );
            if let Some(expected) = query.chosen_side
                && *side != Some(expected)
            {
                mismatches.push(format!("chosen_side expected {expected:?} actual {side:?}"));
            }
            mismatches
        }
    }
}

fn push_option_string_mismatch(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if let Some(expected) = expected
        && actual != Some(expected)
    {
        mismatches.push(format!(
            "{field} expected {:?} actual {:?}",
            Some(expected),
            actual
        ));
    }
}

fn overlay_placement_query_summary(query: &UiOverlayPlacementTraceQueryV1) -> String {
    format!(
        "kind={:?} overlay_root={:?} anchor={:?} content={:?} preferred_side={:?} chosen_side={:?} side_offset_px={:?} side_offset_eps_px={:?} flipped={:?} align={:?} sticky={:?}",
        query.kind,
        query.overlay_root_name.as_deref(),
        query.anchor_test_id.as_deref(),
        query.content_test_id.as_deref(),
        query.preferred_side,
        query.chosen_side,
        query.side_offset_px,
        query.side_offset_eps_px,
        query.flipped,
        query.align,
        query.sticky
    )
}

fn overlay_placement_trace_candidate_summary(entry: &UiOverlayPlacementTraceEntryV1) -> String {
    match entry {
        UiOverlayPlacementTraceEntryV1::AnchoredPanel {
            step_index,
            frame_id,
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            preferred_side,
            chosen_side,
            side_offset_px,
            align,
            sticky,
            final_rect,
            ..
        } => format!(
            "kind=anchored_panel step_index={step_index} frame_id={frame_id} overlay_root={:?} anchor={:?} content={:?} preferred_side={preferred_side:?} chosen_side={chosen_side:?} side_offset_px={side_offset_px} flipped={} align={align:?} sticky={sticky:?} final_rect={}",
            overlay_root_name.as_deref(),
            anchor_test_id.as_deref(),
            content_test_id.as_deref(),
            preferred_side != chosen_side,
            overlay_rect_summary(final_rect)
        ),
        UiOverlayPlacementTraceEntryV1::PlacedRect {
            step_index,
            frame_id,
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            side,
            placed,
            ..
        } => format!(
            "kind=placed_rect step_index={step_index} frame_id={frame_id} overlay_root={:?} anchor={:?} content={:?} chosen_side={side:?} placed_rect={}",
            overlay_root_name.as_deref(),
            anchor_test_id.as_deref(),
            content_test_id.as_deref(),
            overlay_rect_summary(placed)
        ),
    }
}

fn overlay_rect_summary(rect: &UiRectV1) -> String {
    format!(
        "x={} y={} w={} h={}",
        rect.x_px, rect.y_px, rect.w_px, rect.h_px
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_diagnostics::script_engine::overlay_placement_trace_entry_matches_query_any_step;
    use fret_core::{
        Point, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRoot,
        SemanticsSnapshot, Size,
    };
    use slotmap::KeyData;

    fn key_modifiers() -> UiKeyModifiersV1 {
        UiKeyModifiersV1 {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> UiRectV1 {
        UiRectV1 {
            x_px: x,
            y_px: y,
            w_px: w,
            h_px: h,
        }
    }

    fn node_id(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn semantics_rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn semantics_node(
        id: u64,
        parent: Option<u64>,
        role: SemanticsRole,
        bounds: Rect,
        label: &str,
        test_id: Option<&str>,
    ) -> SemanticsNode {
        SemanticsNode {
            id: node_id(id),
            parent: parent.map(node_id),
            role,
            bounds,
            flags: SemanticsFlags::default(),
            test_id: test_id.map(|s| s.to_string()),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: Some(label.to_string()),
            value: None,
            extra: Default::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        }
    }

    fn anchored_trace(
        anchor_test_id: &str,
        content_test_id: &str,
        preferred_side: UiOverlaySideV1,
        chosen_side: UiOverlaySideV1,
    ) -> UiOverlayPlacementTraceEntryV1 {
        let outer = rect(0.0, 0.0, 240.0, 160.0);
        let anchor = rect(40.0, 60.0, 120.0, 32.0);
        let desired = UiSizeV1 {
            w_px: 180.0,
            h_px: 120.0,
        };
        let placed = rect(32.0, 92.0, 180.0, 120.0);
        UiOverlayPlacementTraceEntryV1::AnchoredPanel {
            step_index: 18,
            note: None,
            frame_id: 44,
            overlay_root_name: Some("overlay-root".to_string()),
            anchor_element: Some(1),
            anchor_test_id: Some(anchor_test_id.to_string()),
            content_element: Some(2),
            content_test_id: Some(content_test_id.to_string()),
            outer_input: outer,
            outer_collision: outer,
            anchor,
            desired,
            side_offset_px: 4.0,
            preferred_side,
            align: UiOverlayAlignV1::Start,
            direction: UiLayoutDirectionV1::Ltr,
            sticky: UiOverlayStickyModeV1::Partial,
            offset: UiOverlayOffsetV1 {
                main_axis_px: 4.0,
                cross_axis_px: 0.0,
                alignment_axis_px: None,
            },
            shift: UiOverlayShiftV1 {
                main_axis: true,
                cross_axis: true,
            },
            collision_padding: UiEdgesV1 {
                top_px: 0.0,
                right_px: 0.0,
                bottom_px: 0.0,
                left_px: 0.0,
            },
            collision_boundary: None,
            gap_px: 0.0,
            preferred_rect: placed,
            flipped_rect: placed,
            preferred_fits_without_main_clamp: true,
            flipped_fits_without_main_clamp: false,
            preferred_available_main_px: 120.0,
            flipped_available_main_px: 20.0,
            chosen_side,
            chosen_rect: placed,
            rect_after_shift: placed,
            shift_delta: UiPointV1 {
                x_px: 0.0,
                y_px: 0.0,
            },
            final_rect: placed,
            arrow: None,
        }
    }

    #[test]
    fn wait_overlay_trace_matching_is_step_scoped() {
        let query = UiOverlayPlacementTraceQueryV1 {
            kind: Some(UiOverlayPlacementTraceKindV1::AnchoredPanel),
            anchor_test_id: Some("trigger".to_string()),
            content_test_id: Some("content".to_string()),
            ..Default::default()
        };
        let trace = anchored_trace(
            "trigger",
            "content",
            UiOverlaySideV1::Bottom,
            UiOverlaySideV1::Bottom,
        );

        assert!(overlay_placement_trace_entry_matches_query_any_step(
            &trace, &query
        ));
        assert!(!overlay_placement_trace_entry_matches_query(
            &trace, 19, &query
        ));
        assert!(overlay_placement_trace_entry_matches_query(
            &trace, 18, &query
        ));
    }

    #[test]
    fn overlay_trace_timeout_note_names_content_selector_mismatch() {
        let trace = vec![anchored_trace(
            "ui-gallery-combobox-demo-trigger",
            "ui-gallery-combobox-demo-content",
            UiOverlaySideV1::Bottom,
            UiOverlaySideV1::Bottom,
        )];
        let query = UiOverlayPlacementTraceQueryV1 {
            kind: Some(UiOverlayPlacementTraceKindV1::AnchoredPanel),
            anchor_test_id: Some("ui-gallery-combobox-demo-trigger".to_string()),
            content_test_id: Some("ui-gallery-combobox-demo-listbox".to_string()),
            chosen_side: Some(UiOverlaySideV1::Bottom),
            ..Default::default()
        };

        let note = overlay_placement_trace_timeout_note(&trace, &query).unwrap();

        assert!(note.contains("trace_count=1"));
        assert!(note.contains("content_test_id"));
        assert!(note.contains("ui-gallery-combobox-demo-listbox"));
        assert!(note.contains("ui-gallery-combobox-demo-content"));
    }

    #[test]
    fn wait_until_selector_trace_reports_modal_barrier_filtering() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(10),
                    visible: true,
                    blocks_underlay_input: true,
                    hit_testable: true,
                    z_index: 1,
                },
            ],
            barrier_root: Some(node_id(10)),
            focus_barrier_root: Some(node_id(10)),
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Window,
                    semantics_rect(0.0, 0.0, 200.0, 200.0),
                    "app-root",
                    None,
                ),
                semantics_node(
                    2,
                    Some(1),
                    SemanticsRole::Group,
                    semantics_rect(0.0, 0.0, 200.0, 200.0),
                    "page",
                    Some("ui-gallery-page-dialog"),
                ),
                semantics_node(
                    10,
                    None,
                    SemanticsRole::Generic,
                    semantics_rect(0.0, 0.0, 200.0, 200.0),
                    "barrier",
                    None,
                ),
            ],
        };

        let predicate = UiPredicateV1::Exists {
            target: UiSelectorV1::TestId {
                id: "ui-gallery-page-dialog".to_string(),
                root_z_index: None,
            },
        };

        let mut trace = Vec::new();
        record_wait_until_predicate_selector_trace(
            &snapshot, window, None, None, 5, false, &predicate, &mut trace,
        );

        assert_eq!(trace.len(), 1);
        let note = trace[0].note.as_deref().unwrap_or_default();
        assert!(note.contains("raw_match_count=1"));
        assert!(note.contains("below_barrier_count=1"));
    }

    #[test]
    fn overlay_trace_timeout_note_names_side_and_flip_mismatches() {
        let trace = vec![anchored_trace(
            "trigger",
            "content",
            UiOverlaySideV1::Bottom,
            UiOverlaySideV1::Top,
        )];
        let query = UiOverlayPlacementTraceQueryV1 {
            kind: Some(UiOverlayPlacementTraceKindV1::AnchoredPanel),
            anchor_test_id: Some("trigger".to_string()),
            content_test_id: Some("content".to_string()),
            chosen_side: Some(UiOverlaySideV1::Bottom),
            flipped: Some(false),
            ..Default::default()
        };

        let note = overlay_placement_trace_timeout_note(&trace, &query).unwrap();

        assert!(note.contains("chosen_side expected Bottom actual Top"));
        assert!(note.contains("flipped expected false actual true"));
        assert!(note.contains("best_candidate=kind=anchored_panel"));
    }

    #[test]
    fn shortcut_routing_timeout_note_names_query_mismatches() {
        let trace = vec![UiShortcutRoutingTraceEntryV1 {
            step_index: 7,
            note: None,
            frame_id: 120,
            phase: "dispatch".to_string(),
            deferred: false,
            focus_is_text_input: true,
            ime_composing: true,
            key_contexts: vec!["editor".to_string()],
            key: "Enter".to_string(),
            modifiers: key_modifiers(),
            repeat: false,
            outcome: "ignored".to_string(),
            command: Some("editor.submit".to_string()),
            command_enabled: Some(false),
            pending_sequence_len: Some(0),
        }];
        let query = UiShortcutRoutingTraceQueryV1 {
            phase: Some("dispatch".to_string()),
            outcome: Some("handled".to_string()),
            key: Some("Enter".to_string()),
            command: Some("editor.accept".to_string()),
            ime_composing: Some(false),
            focus_is_text_input: Some(true),
            key_context: Some("global".to_string()),
        };

        let note = shortcut_routing_trace_timeout_note(&trace, &query, 100).unwrap();

        assert!(note.contains("trace_count=1"));
        assert!(note.contains("outcome expected Some(\"handled\") actual Some(\"ignored\")"));
        assert!(
            note.contains(
                "command expected Some(\"editor.accept\") actual Some(\"editor.submit\")"
            )
        );
        assert!(note.contains("ime_composing expected false actual true"));
        assert!(note.contains("key_context expected Some(\"global\") actual [\"editor\"]"));
    }

    #[test]
    fn command_dispatch_timeout_note_names_query_mismatches() {
        let trace = vec![UiScriptCommandDispatchTraceEntryV1 {
            step_index: 9,
            frame_id: 201,
            command: "menu.open".to_string(),
            handled: false,
            handled_by_scope: Some("window".to_string()),
            handled_by_driver: true,
            stopped: false,
            source_kind: "pointer".to_string(),
            source_element: Some(10),
            source_test_id: Some("toolbar-menu-trigger".to_string()),
            handled_by_element: Some(11),
            handled_by_test_id: Some("window-shell".to_string()),
            started_from_focus: false,
            used_default_root_fallback: true,
        }];
        let query = UiScriptCommandDispatchTraceQueryV1 {
            command: Some("menu.open".to_string()),
            source_kind: Some("keyboard".to_string()),
            source_test_id: Some("palette-trigger".to_string()),
            handled: Some(true),
            handled_by_scope: Some("widget".to_string()),
            handled_by_driver: Some(false),
            handled_by_test_id: Some("menu-button".to_string()),
            started_from_focus: Some(true),
            used_default_root_fallback: Some(false),
        };

        let note = command_dispatch_trace_timeout_note(&trace, &query, 200).unwrap();

        assert!(note.contains("trace_count=1"));
        assert!(note.contains("source_kind expected Some(\"keyboard\") actual Some(\"pointer\")"));
        assert!(note.contains(
            "source_test_id expected Some(\"palette-trigger\") actual Some(\"toolbar-menu-trigger\")"
        ));
        assert!(note.contains("handled expected true actual false"));
        assert!(note.contains("handled_by_driver expected false actual true"));
        assert!(note.contains(
            "handled_by_test_id expected Some(\"menu-button\") actual Some(\"window-shell\")"
        ));
        assert!(note.contains("used_default_root_fallback expected false actual true"));
    }
}

fn record_wait_until_predicate_selector_trace(
    snapshot: &fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    scope_root: Option<u64>,
    step_index: u32,
    redact_text: bool,
    predicate: &UiPredicateV1,
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
) {
    fn record_selector(
        snapshot: &fret_core::SemanticsSnapshot,
        window: AppWindowId,
        element_runtime: Option<&ElementRuntime>,
        scope_root: Option<u64>,
        step_index: u32,
        redact_text: bool,
        selector: &UiSelectorV1,
        trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
    ) {
        select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            selector,
            scope_root,
            step_index,
            redact_text,
            trace,
        );
    }

    match predicate {
        UiPredicateV1::Exists { target }
        | UiPredicateV1::NotExists { target }
        | UiPredicateV1::RawSemanticsHiddenIs { target, .. }
        | UiPredicateV1::FocusIs { target }
        | UiPredicateV1::RoleIs { target, .. }
        | UiPredicateV1::LabelContains { target, .. }
        | UiPredicateV1::LabelLenIs { target, .. }
        | UiPredicateV1::LabelLenGe { target, .. }
        | UiPredicateV1::ValueContains { target, .. }
        | UiPredicateV1::ValueEquals { target, .. }
        | UiPredicateV1::ValueLenIs { target, .. }
        | UiPredicateV1::ValueLenGe { target, .. }
        | UiPredicateV1::PosInSetIs { target, .. }
        | UiPredicateV1::SetSizeIs { target, .. }
        | UiPredicateV1::LevelIs { target, .. }
        | UiPredicateV1::CheckedIs { target, .. }
        | UiPredicateV1::CheckedStateIs { target, .. }
        | UiPredicateV1::PressedStateIs { target, .. }
        | UiPredicateV1::ExpandedIs { target, .. }
        | UiPredicateV1::SemanticsLiveIs { target, .. }
        | UiPredicateV1::SemanticsLiveAtomicIs { target, .. }
        | UiPredicateV1::SelectedIs { target, .. }
        | UiPredicateV1::DisabledIs { target, .. }
        | UiPredicateV1::ReadOnlyIs { target, .. }
        | UiPredicateV1::SemanticsActionIs { target, .. }
        | UiPredicateV1::CapturedIs { target, .. }
        | UiPredicateV1::SemanticsNumericApproxEq { target, .. }
        | UiPredicateV1::SemanticsScrollIsFinite { target, .. }
        | UiPredicateV1::SemanticsScrollApproxEq { target, .. }
        | UiPredicateV1::SemanticsScrollNotApproxEq { target, .. }
        | UiPredicateV1::TextCompositionIs { target, .. }
        | UiPredicateV1::VisibleInWindow { target }
        | UiPredicateV1::ElementEffectiveOpacityApproxEq { target, .. }
        | UiPredicateV1::BoundsWithinWindow { target, .. }
        | UiPredicateV1::TextInputImeCursorAreaWithinBounds { target, .. }
        | UiPredicateV1::BoundsMinSize { target, .. }
        | UiPredicateV1::BoundsMaxSize { target, .. } => {
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                target,
                trace,
            );
        }
        UiPredicateV1::ExistsUnder { scope, target }
        | UiPredicateV1::NotExistsUnder { scope, target }
        | UiPredicateV1::FocusedDescendantIs { scope, target } => {
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                scope,
                trace,
            );
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                target,
                trace,
            );
        }
        UiPredicateV1::SemanticsRelationIncludes { source, target, .. } => {
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                source,
                trace,
            );
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                target,
                trace,
            );
        }
        UiPredicateV1::SemanticsRelationIsEmpty { source, .. }
        | UiPredicateV1::ActiveItemIsNone { container: source } => {
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                source,
                trace,
            );
        }
        UiPredicateV1::ActiveItemIs { container, item } => {
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                container,
                trace,
            );
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                item,
                trace,
            );
        }
        UiPredicateV1::BoundsApproxEqual { a, b, .. }
        | UiPredicateV1::BoundsCenterApproxEqual { a, b, .. }
        | UiPredicateV1::BoundsMetricDelta { a, b, .. }
        | UiPredicateV1::BoundsMetricPairDelta { a, b, .. }
        | UiPredicateV1::BoundsNonOverlapping { a, b, .. }
        | UiPredicateV1::BoundsOverlapping { a, b, .. }
        | UiPredicateV1::BoundsOverlappingX { a, b, .. }
        | UiPredicateV1::BoundsOverlappingY { a, b, .. } => {
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                a,
                trace,
            );
            record_selector(
                snapshot,
                window,
                element_runtime,
                scope_root,
                step_index,
                redact_text,
                b,
                trace,
            );
        }
        _ => {}
    }
}

pub(super) fn handle_wait_until_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::WaitUntil {
        window: target_window,
        predicate,
        timeout_frames,
        timeout_ms,
    } = step
    else {
        return false;
    };

    active.screenshot_wait = None;

    let mut predicate_window = window;
    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            if UiDiagnosticsService::predicate_can_eval_off_window(&predicate)
                || UiDiagnosticsService::predicate_can_eval_from_cached_test_id_bounds(&predicate)
            {
                predicate_window = target_window;
                output.effects.push(Effect::Redraw(target_window));
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            } else {
                *handoff_to = Some(target_window);
                output.effects.push(Effect::Redraw(target_window));
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            }
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-wait_until-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }

    if *stop_script {
        active.wait_until = None;
        active.screenshot_wait = None;
        return true;
    }
    if handoff_to.is_some() {
        active.wait_until = None;
        active.screenshot_wait = None;
        // This step is window-targeted; the runtime will migrate the script.
        return true;
    }

    let mut state = match active.wait_until.take() {
        Some(mut state) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            if state.deadline_unix_ms.is_none() {
                state.deadline_unix_ms =
                    timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64));
            }
            state
        }
        _ => WaitUntilState {
            step_index,
            remaining_frames: timeout_frames,
            deadline_unix_ms: timeout_ms.map(|ms| unix_ms_now().saturating_add(ms as u64)),
            cached_test_id_predicate_last_stale: None,
        },
    };

    let cache_eval = svc.eval_predicate_from_cached_test_id_bounds_if_allowed(
        window,
        predicate_window,
        semantics_snapshot.is_some(),
        &predicate,
    );
    if cache_eval.used_cache {
        let should_log = state
            .cached_test_id_predicate_last_stale
            .map(|prev| prev != cache_eval.stale)
            .unwrap_or(true);
        if should_log {
            let kind = if cache_eval.stale {
                "diag.cached_test_id_predicate.stale"
            } else {
                "diag.cached_test_id_predicate.hit"
            };
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: kind.to_string(),
                    step_index: Some(step_index.min(u32::MAX as usize) as u32),
                    note: Some(format!(
                        "predicate_window={} test_id={:?} ok={:?} age_ms={:?} snapshot_seq={:?} max_age_ms={:?}",
                        predicate_window.data().as_ffi(),
                        cache_eval.test_id.as_deref(),
                        cache_eval.ok,
                        cache_eval.age_ms,
                        cache_eval.window_snapshot_seq,
                        cache_eval.max_age_ms
                    )),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            state.cached_test_id_predicate_last_stale = Some(cache_eval.stale);
        }
    }

    let ok = match cache_eval.ok {
        Some(ok) => ok,
        None if cache_eval.used_cache
            && cache_eval.stale
            && UiDiagnosticsService::predicate_can_eval_from_cached_test_id_bounds(&predicate) =>
        {
            false
        }
        None => match &predicate {
            UiPredicateV1::WindowInnerSizeApproxEqual {
                width_px,
                height_px,
                eps_px,
            } => window_inner_size_approx_equal(window_bounds, *width_px, *height_px, *eps_px),
            UiPredicateV1::EventKindSeen { event_kind } => svc
                .per_window
                .get(&predicate_window)
                .is_some_and(|ring| ring.events.iter().any(|e| e.kind == *event_kind)),
            UiPredicateV1::RunnerAccessibilityActivated => app
                .global::<fret_runtime::RunnerAccessibilityDiagnosticsStore>()
                .and_then(|store| store.snapshot(predicate_window))
                .is_some_and(|snapshot| snapshot.activation_requests > 0),
            UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
                text_font_stack_key_stable_frames >= *stable_frames
            }
            UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
            UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
            _ => {
                if let Some(ok) = eval_debug_snapshot_predicate_from_recent_snapshot(
                    svc,
                    predicate_window,
                    &predicate,
                    250,
                ) {
                    ok
                } else {
                    let docking_diag = app
                        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                        .and_then(|store| store.docking_latest_for_window(predicate_window));
                    let workspace_diag = app
                        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                        .and_then(|store| store.workspace_latest_for_window(predicate_window));
                    let input_ctx = app
                        .global::<fret_runtime::WindowInputContextService>()
                        .and_then(|svc| svc.snapshot(predicate_window));
                    let text_input_snapshot = app
                        .global::<fret_runtime::WindowTextInputSnapshotService>()
                        .and_then(|svc| svc.snapshot(predicate_window));
                    let dock_drag_runtime =
                        dock_drag_runtime_state(app, svc.known_windows.as_slice());
                    let platform_caps = app.global::<fret_runtime::PlatformCapabilities>();
                    let open_window_count =
                        UiDiagnosticsService::open_window_count_for_predicates(app);
                    let app_snapshot = svc.app_snapshot_for_window(app, predicate_window);

                    if predicate_window == window {
                        if let Some(snapshot) = semantics_snapshot {
                            record_overlay_placement_trace(
                                &mut active.overlay_placement_trace,
                                element_runtime,
                                Some(snapshot),
                                window,
                                step_index as u32,
                                "wait_until",
                            );
                            eval_predicate(
                                snapshot,
                                window_bounds,
                                predicate_window,
                                active.scope_root_for_window(predicate_window),
                                input_ctx,
                                element_runtime,
                                text_input_snapshot,
                                app.global::<fret_core::RendererTextPerfSnapshot>().copied(),
                                app.global::<fret_core::RendererTextFontTraceSnapshot>(),
                                app_snapshot.as_ref(),
                                svc.known_windows.as_slice(),
                                open_window_count,
                                platform_caps,
                                app.global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>(),
                                app.global::<fret_runtime::RunnerPlatformWindowReceiverDiagnosticsStore>(),
                                docking_diag,
                                workspace_diag,
                                dock_drag_runtime.as_ref(),
                                text_font_stack_key_stable_frames,
                                font_catalog_populated,
                                system_font_rescan_idle,
                                &predicate,
                            )
                        } else {
                            eval_predicate_without_semantics(
                                predicate_window,
                                svc.known_windows.as_slice(),
                                open_window_count,
                                app_snapshot.as_ref(),
                                platform_caps,
                                app.global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>(),
                                app.global::<fret_runtime::RunnerPlatformWindowReceiverDiagnosticsStore>(),
                                docking_diag,
                                workspace_diag,
                                dock_drag_runtime.as_ref(),
                                &predicate,
                            )
                            .unwrap_or_else(|| {
                                output.request_redraw = true;
                                false
                            })
                        }
                    } else {
                        // Off-window predicates must not reuse the current window's semantics snapshot.
                        eval_predicate_without_semantics(
                            predicate_window,
                            svc.known_windows.as_slice(),
                            open_window_count,
                            app_snapshot.as_ref(),
                            platform_caps,
                            app.global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>(),
                            app.global::<fret_runtime::RunnerPlatformWindowReceiverDiagnosticsStore>(),
                            docking_diag,
                            workspace_diag,
                            dock_drag_runtime.as_ref(),
                            &predicate,
                        )
                        .unwrap_or_else(|| {
                            output.request_redraw = true;
                            false
                        })
                    }
                }
            }
        },
    };
    if !ok
        && predicate_window == window
        && let Some(snapshot) = semantics_snapshot
    {
        record_wait_until_predicate_selector_trace(
            snapshot,
            predicate_window,
            element_runtime,
            active.scope_root_for_window(predicate_window),
            step_index as u32,
            svc.cfg.redact_text,
            &predicate,
            &mut active.selector_resolution_trace,
        );
    }
    if ok {
        active.wait_until = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    } else if state
        .deadline_unix_ms
        .is_some_and(|deadline| unix_ms_now() >= deadline)
        || state.remaining_frames == 0
    {
        let text_font_stack_key = app.global::<fret_runtime::TextFontStackKey>().map(|k| k.0);
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "wait_until_timeout".to_string(),
                step_index: Some(step_index as u32),
                note: Some(format!(
                    "predicate_window={} predicate={predicate:?} remaining_frames={} deadline_unix_ms={:?} text_font_stack_key={text_font_stack_key:?} text_font_stack_key_stable_frames={text_font_stack_key_stable_frames} font_catalog_populated={font_catalog_populated} system_font_rescan_idle={system_font_rescan_idle}",
                    predicate_window.data().as_ffi(),
                    state.remaining_frames,
                    state.deadline_unix_ms,
                )),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        *force_dump_label = Some(format!("script-step-{step_index:04}-wait_until-timeout"));
        *stop_script = true;
        *failure_reason = Some("wait_until_timeout".to_string());
        active.wait_until = None;
        output.request_redraw = true;
    } else {
        active.wait_until = Some(WaitUntilState {
            step_index: state.step_index,
            remaining_frames: state.remaining_frames.saturating_sub(1),
            deadline_unix_ms: state.deadline_unix_ms,
            cached_test_id_predicate_last_stale: state.cached_test_id_predicate_last_stale,
        });
        output.request_redraw = true;
    }

    true
}
