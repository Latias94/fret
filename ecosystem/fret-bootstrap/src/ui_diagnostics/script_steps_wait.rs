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

pub(super) fn handle_wait_shortcut_routing_trace_step(
    app: &App,
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
    app: &App,
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

pub(super) fn handle_wait_overlay_placement_trace_step(
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
    let found = active.overlay_placement_trace.iter().any(|entry| {
        overlay_placement_trace_entry_matches_query(entry, step_index_u32, &query)
            || overlay_placement_trace_entry_matches_query_any_step(entry, &query)
    });

    if found {
        active.wait_overlay_placement_trace = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    } else if state
        .deadline_unix_ms
        .is_some_and(|deadline| unix_ms_now() >= deadline)
        || state.remaining_frames == 0
    {
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

    let cache_eval = svc.eval_predicate_from_cached_test_id_bounds(predicate_window, &predicate);
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

    if ok {
        active.wait_until = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    } else if state
        .deadline_unix_ms
        .is_some_and(|deadline| unix_ms_now() >= deadline)
        || state.remaining_frames == 0
    {
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
