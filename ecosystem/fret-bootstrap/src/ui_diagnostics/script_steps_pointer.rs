use super::*;

pub(super) fn handle_click_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Click {
        window: target_window,
        pointer_kind,
        target,
        button,
        click_count,
        modifiers,
    } = step
    else {
        return false;
    };

    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-click-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }
    if *stop_script {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else if handoff_to.is_some() {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else {
        let Some(snapshot) = semantics_snapshot else {
            output.request_redraw = true;
            let label = format!("script-step-{step_index:04}-click-no-semantics");
            if svc.cfg.script_auto_dump {
                svc.dump_bundle(Some(&label));
            }
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some("no_semantics_snapshot".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            svc.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code: Some("semantics.missing".to_string()),
                reason: Some("no_semantics_snapshot".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
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
            output.request_redraw = true;
            let label = format!("script-step-{step_index:04}-click-no-semantics-match");
            if svc.cfg.script_auto_dump {
                svc.dump_bundle(Some(&label));
            }
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some("click_no_semantics_match".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            svc.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code: Some("selector.not_found".to_string()),
                reason: Some("click_no_semantics_match".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
            return true;
        };

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
                Some("click"),
                svc.cfg.max_debug_string_bytes,
            );
        }
        record_overlay_placement_trace(
            &mut active.overlay_placement_trace,
            element_runtime,
            Some(snapshot),
            window,
            step_index as u32,
            "click",
        );
        let modifiers = core_modifiers_from_ui(modifiers);
        let pointer_type = pointer_type_from_kind(pointer_kind);
        output.events.extend(click_events_with_modifiers(
            pos,
            button,
            click_count,
            modifiers,
            pointer_type,
        ));

        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-click"));
        }
    }

    false
}

pub(super) fn handle_tap_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Tap {
        window: target_window,
        pointer_kind,
        target,
        modifiers,
    } = step
    else {
        return false;
    };

    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!("script-step-{step_index:04}-tap-window-not-found"));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }
    if *stop_script {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else if handoff_to.is_some() {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else {
        let Some(snapshot) = semantics_snapshot else {
            output.request_redraw = true;
            let label = format!("script-step-{step_index:04}-tap-no-semantics");
            if svc.cfg.script_auto_dump {
                svc.dump_bundle(Some(&label));
            }
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some("no_semantics_snapshot".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            svc.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code: Some("semantics.missing".to_string()),
                reason: Some("no_semantics_snapshot".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
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
            output.request_redraw = true;
            let label = format!("script-step-{step_index:04}-tap-no-semantics-match");
            if svc.cfg.script_auto_dump {
                svc.dump_bundle(Some(&label));
            }
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some("tap_no_semantics_match".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            svc.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code: Some("selector.not_found".to_string()),
                reason: Some("tap_no_semantics_match".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
            return true;
        };

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
                Some("tap"),
                svc.cfg.max_debug_string_bytes,
            );
        }
        record_overlay_placement_trace(
            &mut active.overlay_placement_trace,
            element_runtime,
            Some(snapshot),
            window,
            step_index as u32,
            "tap",
        );
        let modifiers = core_modifiers_from_ui(modifiers);
        let pointer_type = pointer_type_from_kind(pointer_kind.or(Some(UiPointerKindV1::Touch)));
        output.events.extend(click_events_with_modifiers(
            pos,
            UiMouseButtonV1::Left,
            1,
            modifiers,
            pointer_type,
        ));

        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-tap"));
        }
    }

    false
}

pub(super) fn handle_long_press_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::LongPress {
        window: target_window,
        pointer_kind,
        target,
        duration_ms,
        modifiers,
    } = step
    else {
        return false;
    };

    // Multi-frame step state: if already started, keep injecting into the original window.
    if let Some(V2StepState::LongPress(mut state)) = active.v2_step_state.take() {
        if state.step_index == step_index {
            if state.window != window {
                *handoff_to = Some(state.window);
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(state.window));
                output.request_redraw = true;
                active.v2_step_state = Some(V2StepState::LongPress(state));
                return false;
            }

            let now_ms = now_monotonic_ms_for_window(app, window).unwrap_or_else(unix_ms_now);
            if !state.down_issued {
                output.events.push(move_pointer_event_with_modifiers(
                    state.position,
                    state.modifiers,
                    state.pointer_type,
                ));
                output.events.push(pointer_down_event_with_modifiers(
                    state.position,
                    UiMouseButtonV1::Left,
                    1,
                    state.modifiers,
                    state.pointer_type,
                ));
                state.down_issued = true;
                state.started_monotonic_ms = Some(now_ms);
            } else if let Some(start) = state.started_monotonic_ms
                && now_ms.saturating_sub(start) >= state.duration_ms
            {
                output.events.push(pointer_up_event_with_modifiers(
                    state.position,
                    UiMouseButtonV1::Left,
                    1,
                    state.modifiers,
                    state.pointer_type,
                    false,
                ));

                active.v2_step_state = None;
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-long-press"));
                }
                return false;
            }

            output.request_redraw = true;
            active.wait_until = None;
            active.screenshot_wait = None;
            active.v2_step_state = Some(V2StepState::LongPress(state));
            return false;
        }

        // Different step: keep the state (should not happen, but avoid losing it).
        active.v2_step_state = Some(V2StepState::LongPress(state));
    }

    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-long-press-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }

    if *stop_script {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
        return false;
    }
    if handoff_to.is_some() {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
        return false;
    }

    let Some(snapshot) = semantics_snapshot else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-long-press-no-semantics");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("no_semantics_snapshot".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("semantics.missing".to_string()),
            reason: Some("no_semantics_snapshot".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
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
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-long-press-no-semantics-match");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("long_press_no_semantics_match".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("selector.not_found".to_string()),
            reason: Some("long_press_no_semantics_match".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
        return true;
    };

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
            Some("long_press"),
            svc.cfg.max_debug_string_bytes,
        );
    }
    record_overlay_placement_trace(
        &mut active.overlay_placement_trace,
        element_runtime,
        Some(snapshot),
        window,
        step_index as u32,
        "long_press",
    );

    let modifiers = core_modifiers_from_ui(modifiers);
    let pointer_type = pointer_type_from_kind(pointer_kind.or(Some(UiPointerKindV1::Touch)));

    active.v2_step_state = Some(V2StepState::LongPress(V2LongPressState {
        step_index,
        window,
        position: pos,
        pointer_type,
        modifiers,
        duration_ms,
        started_monotonic_ms: None,
        down_issued: false,
    }));
    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    false
}

pub(super) fn handle_swipe_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Swipe {
        window: target_window,
        pointer_kind,
        target,
        delta_x,
        delta_y,
        steps,
        modifiers,
    } = step
    else {
        return false;
    };

    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-swipe-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }
    if *stop_script {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
        return false;
    }
    if handoff_to.is_some() {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
        return false;
    }

    let Some(snapshot) = semantics_snapshot else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-swipe-no-semantics");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("no_semantics_snapshot".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("semantics.missing".to_string()),
            reason: Some("no_semantics_snapshot".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
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
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-swipe-no-semantics-match");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("swipe_no_semantics_match".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("selector.not_found".to_string()),
            reason: Some("swipe_no_semantics_match".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
        return true;
    };

    let start = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
    let end = clamp_point_to_rect(
        Point::new(
            fret_core::Px(start.x.0 + delta_x),
            fret_core::Px(start.y.0 + delta_y),
        ),
        window_bounds,
    );

    if let Some(ui) = ui.as_deref_mut() {
        record_hit_test_trace_for_selector(
            &mut active.hit_test_trace,
            ui,
            element_runtime,
            window,
            Some(snapshot),
            &target,
            step_index as u32,
            start,
            Some(node),
            Some("swipe"),
            svc.cfg.max_debug_string_bytes,
        );
    }
    record_overlay_placement_trace(
        &mut active.overlay_placement_trace,
        element_runtime,
        Some(snapshot),
        window,
        step_index as u32,
        "swipe",
    );

    let modifiers = core_modifiers_from_ui(modifiers);
    let pointer_type = pointer_type_from_kind(pointer_kind.or(Some(UiPointerKindV1::Touch)));
    let steps = steps.max(1);

    output.events.push(move_pointer_event_with_modifiers(
        start,
        modifiers,
        pointer_type,
    ));
    output.events.push(pointer_down_event_with_modifiers(
        start,
        UiMouseButtonV1::Left,
        1,
        modifiers,
        pointer_type,
    ));

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let x = start.x.0 + (end.x.0 - start.x.0) * t;
        let y = start.y.0 + (end.y.0 - start.y.0) * t;
        let pos = Point::new(fret_core::Px(x), fret_core::Px(y));
        output
            .events
            .push(pointer_move_event_with_buttons_modifiers(
                UiMouseButtonV1::Left,
                pos,
                modifiers,
                pointer_type,
            ));
    }

    output.events.push(pointer_up_event_with_modifiers(
        end,
        UiMouseButtonV1::Left,
        1,
        modifiers,
        pointer_type,
        false,
    ));

    active.v2_step_state = None;
    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-swipe"));
    }

    false
}

pub(super) fn handle_pinch_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Pinch {
        window: target_window,
        pointer_kind,
        target,
        delta,
        steps,
        modifiers,
    } = step
    else {
        return false;
    };

    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-pinch-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }
    if *stop_script {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else if handoff_to.is_some() {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else {
        let Some(snapshot) = semantics_snapshot else {
            output.request_redraw = true;
            let label = format!("script-step-{step_index:04}-pinch-no-semantics");
            if svc.cfg.script_auto_dump {
                svc.dump_bundle(Some(&label));
            }
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some("no_semantics_snapshot".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            svc.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code: Some("semantics.missing".to_string()),
                reason: Some("no_semantics_snapshot".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
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
            output.request_redraw = true;
            let label = format!("script-step-{step_index:04}-pinch-no-semantics-match");
            if svc.cfg.script_auto_dump {
                svc.dump_bundle(Some(&label));
            }
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some("pinch_no_semantics_match".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            svc.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code: Some("selector.not_found".to_string()),
                reason: Some("pinch_no_semantics_match".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
            return true;
        };

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
                Some("pinch"),
                svc.cfg.max_debug_string_bytes,
            );
        }
        record_overlay_placement_trace(
            &mut active.overlay_placement_trace,
            element_runtime,
            Some(snapshot),
            window,
            step_index as u32,
            "pinch",
        );

        let modifiers = core_modifiers_from_ui(modifiers);
        let pointer_type = pointer_type_from_kind(pointer_kind.or(Some(UiPointerKindV1::Touch)));
        let steps = steps.max(1);
        let delta_per_step = delta / steps as f32;
        for _ in 0..steps {
            output
                .events
                .push(pinch_event(pos, delta_per_step, modifiers, pointer_type));
        }

        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-pinch"));
        }
    }

    false
}

fn now_monotonic_ms_for_window(app: &App, window: AppWindowId) -> Option<u64> {
    let svc = app.global::<fret_core::WindowFrameClockService>()?;
    let snapshot = svc.snapshot(window)?;
    let ms = snapshot.now_monotonic.as_millis();
    Some(ms.min(u64::MAX as u128) as u64)
}

fn clamp_point_to_rect(point: Point, rect: Rect) -> Point {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0.max(0.0);
    let y1 = y0 + rect.size.height.0.max(0.0);
    Point::new(
        fret_core::Px(point.x.0.clamp(x0, x1)),
        fret_core::Px(point.y.0.clamp(y0, y1)),
    )
}

pub(super) fn handle_click_stable_step(
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
    let UiActionStepV2::ClickStable {
        window: _,
        pointer_kind,
        target,
        button,
        click_count,
        modifiers,
        stable_frames,
        max_move_px,
        timeout_frames,
    } = step
    else {
        return false;
    };

    let click_modifiers = core_modifiers_from_ui(modifiers);
    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    // Stable-click: wait for semantics bounds to stop moving before clicking.
    //
    // This is intended for virtualization-heavy views where a target's measured bounds
    // can jump across frames (e.g. estimate -> measured), causing clicks to land at
    // stale coordinates when using a single-frame snapshot.
    if let Some(snapshot) = semantics_snapshot {
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
            let stable_required = stable_frames.max(1);
            let max_move_px = max_move_px.max(0.0);

            let mut state = match active.v2_step_state.take() {
                Some(V2StepState::ClickStable(mut state)) if state.step_index == step_index => {
                    state.remaining_frames = state.remaining_frames.min(timeout_frames);
                    state
                }
                _ => V2ClickStableState {
                    step_index,
                    remaining_frames: timeout_frames,
                    stable_count: 0,
                    last_center: None,
                },
            };

            let center = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
            if state.remaining_frames == 0 {
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        center,
                        Some(node),
                        Some("click_stable.timeout"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                *force_dump_label =
                    Some(format!("script-step-{step_index:04}-click_stable-timeout"));
                *stop_script = true;
                *failure_reason = Some("click_stable_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                let moved = match state.last_center {
                    Some(last) => {
                        let dx = (center.x.0 - last.x.0).abs();
                        let dy = (center.y.0 - last.y.0).abs();
                        dx.max(dy)
                    }
                    None => 0.0,
                };

                if moved <= max_move_px {
                    state.stable_count = state.stable_count.saturating_add(1);
                } else {
                    state.stable_count = 1;
                }
                state.last_center = Some(center);

                if state.stable_count >= stable_required {
                    if let Some(ui) = ui.as_deref_mut() {
                        let mut hit = build_hit_test_trace_entry_for_selector(
                            ui,
                            element_runtime,
                            window,
                            Some(snapshot),
                            &target,
                            step_index as u32,
                            center,
                            Some(node),
                            Some("click_stable.probe"),
                            svc.cfg.max_debug_string_bytes,
                        );
                        let ok = hit.includes_intended == Some(true)
                            || hit.hit_path_contains_intended == Some(true);
                        if !ok {
                            hit.note = Some("click_stable.mismatch".to_string());
                            push_hit_test_trace(&mut active.hit_test_trace, hit.clone());
                            push_click_stable_trace(
                                &mut active.click_stable_trace,
                                UiClickStableTraceEntryV1 {
                                    step_index: step_index as u32,
                                    stable_required,
                                    stable_count: state.stable_count,
                                    moved_px: moved,
                                    max_move_px,
                                    remaining_frames: state.remaining_frames,
                                    hit_test: hit,
                                },
                            );

                            // Scroll and other transform-only updates can land after the
                            // semantics snapshot used to choose `center`. If the target
                            // is not actually hit-testable at this point, keep waiting
                            // instead of clicking a stale coordinate.
                            state.stable_count = 0;
                            state.last_center = None;
                            state.remaining_frames = state.remaining_frames.saturating_sub(1);
                            active.v2_step_state = Some(V2StepState::ClickStable(state));
                            output.request_redraw = true;
                        } else {
                            hit.note = Some("click_stable.click".to_string());
                            push_hit_test_trace(&mut active.hit_test_trace, hit);
                            record_overlay_placement_trace(
                                &mut active.overlay_placement_trace,
                                element_runtime,
                                Some(snapshot),
                                window,
                                step_index as u32,
                                "click_stable.click",
                            );
                            output.events.extend(click_events_with_modifiers(
                                center,
                                button,
                                click_count,
                                click_modifiers,
                                pointer_type,
                            ));
                            active.wait_until = None;
                            active.screenshot_wait = None;
                            active.next_step = active.next_step.saturating_add(1);
                            active.v2_step_state = None;
                            output.request_redraw = true;
                            if svc.cfg.script_auto_dump {
                                *force_dump_label =
                                    Some(format!("script-step-{step_index:04}-click_stable-click"));
                            }
                        }
                    } else {
                        output.events.extend(click_events_with_modifiers(
                            center,
                            button,
                            click_count,
                            click_modifiers,
                            pointer_type,
                        ));
                        active.wait_until = None;
                        active.screenshot_wait = None;
                        active.next_step = active.next_step.saturating_add(1);
                        active.v2_step_state = None;
                        output.request_redraw = true;
                        if svc.cfg.script_auto_dump {
                            *force_dump_label =
                                Some(format!("script-step-{step_index:04}-click_stable-click"));
                        }
                    }
                } else {
                    state.remaining_frames = state.remaining_frames.saturating_sub(1);
                    active.v2_step_state = Some(V2StepState::ClickStable(state));
                    output.request_redraw = true;
                }
            }
        } else {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-click_stable-no-semantics-match"
            ));
            *stop_script = true;
            *failure_reason = Some("click_stable_no_semantics_match".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-click_stable-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    true
}

pub(super) fn handle_click_selectable_text_span_stable_step(
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
    let UiActionStepV2::ClickSelectableTextSpanStable {
        window: _,
        pointer_kind,
        target,
        tag,
        button,
        click_count,
        modifiers,
        stable_frames,
        max_move_px,
        timeout_frames,
    } = step
    else {
        return false;
    };

    let click_modifiers = core_modifiers_from_ui(modifiers);
    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    let note = format!(
        "click_selectable_text_span_stable(tag={})",
        truncate_debug_value(tag.as_str(), 64)
    );

    if let Some(snapshot) = semantics_snapshot {
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
            let stable_required = stable_frames.max(1);
            let max_move_px = max_move_px.max(0.0);

            let mut state = match active.v2_step_state.take() {
                Some(V2StepState::ClickSelectableTextSpanStable(mut state))
                    if state.step_index == step_index =>
                {
                    state.remaining_frames = state.remaining_frames.min(timeout_frames);
                    state
                }
                _ => V2ClickSelectableTextSpanStableState {
                    step_index,
                    remaining_frames: timeout_frames,
                    stable_count: 0,
                    last_pos: None,
                },
            };

            if state.remaining_frames == 0 {
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        center_of_rect_clamped_to_rect(node.bounds, window_bounds),
                        Some(node),
                        Some("click_selectable_text_span_stable.timeout"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-click_selectable_span-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some("click_selectable_text_span_stable_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                let bounds_local: Option<Rect> = match element_runtime.and_then(|rt| {
                    rt.selectable_text_interactive_span_bounds_for_node(window, node.id)
                }) {
                    None => {
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state =
                            Some(V2StepState::ClickSelectableTextSpanStable(state.clone()));
                        output.request_redraw = true;
                        None
                    }
                    Some(spans) if spans.is_empty() => {
                        // Best-effort: span bounds are computed during `paint_all()`. If
                        // we don't see them yet, wait a few frames before failing.
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state =
                            Some(V2StepState::ClickSelectableTextSpanStable(state.clone()));
                        output.request_redraw = true;
                        None
                    }
                    Some(spans) => spans
                        .iter()
                        .find(|span| span.tag.as_ref() == tag.as_str())
                        .map(|span| span.bounds_local)
                        .or_else(|| {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-click_selectable_span-tag-not-found"
                            ));
                            *stop_script = true;
                            *failure_reason =
                                Some("click_selectable_text_span_stable_tag_not_found".to_string());
                            active.v2_step_state = None;
                            output.request_redraw = true;
                            None
                        }),
                };

                if let Some(bounds_local) = bounds_local {
                    let span_bounds = Rect::new(
                        Point::new(
                            Px(node.bounds.origin.x.0 + bounds_local.origin.x.0),
                            Px(node.bounds.origin.y.0 + bounds_local.origin.y.0),
                        ),
                        bounds_local.size,
                    );

                    let skinny = Rect::new(
                        span_bounds.origin,
                        fret_core::Size::new(
                            Px(span_bounds.size.width.0.max(1.0).min(2.0)),
                            span_bounds.size.height,
                        ),
                    );
                    let pos = center_of_rect_clamped_to_rect(skinny, window_bounds);

                    let moved = match state.last_pos {
                        Some(last) => {
                            let dx = (pos.x.0 - last.x.0).abs();
                            let dy = (pos.y.0 - last.y.0).abs();
                            dx.max(dy)
                        }
                        None => 0.0,
                    };

                    if moved <= max_move_px {
                        state.stable_count = state.stable_count.saturating_add(1);
                    } else {
                        state.stable_count = 1;
                    }
                    state.last_pos = Some(pos);

                    if state.stable_count >= stable_required {
                        if let Some(ui) = ui.as_deref_mut() {
                            let mut hit = build_hit_test_trace_entry_for_selector(
                                ui,
                                element_runtime,
                                window,
                                Some(snapshot),
                                &target,
                                step_index as u32,
                                pos,
                                Some(node),
                                Some(&note),
                                svc.cfg.max_debug_string_bytes,
                            );
                            let ok = hit.includes_intended == Some(true)
                                || hit.hit_path_contains_intended == Some(true);
                            if !ok {
                                hit.note = Some("click_selectable_span.mismatch".to_string());
                                push_hit_test_trace(&mut active.hit_test_trace, hit.clone());
                                push_click_stable_trace(
                                    &mut active.click_stable_trace,
                                    UiClickStableTraceEntryV1 {
                                        step_index: step_index as u32,
                                        stable_required,
                                        stable_count: state.stable_count,
                                        moved_px: moved,
                                        max_move_px,
                                        remaining_frames: state.remaining_frames,
                                        hit_test: hit,
                                    },
                                );
                                state.stable_count = 0;
                                state.last_pos = None;
                                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                                active.v2_step_state =
                                    Some(V2StepState::ClickSelectableTextSpanStable(state));
                                output.request_redraw = true;
                            } else {
                                hit.note = Some("click_selectable_span.click".to_string());
                                push_hit_test_trace(&mut active.hit_test_trace, hit);
                                record_overlay_placement_trace(
                                    &mut active.overlay_placement_trace,
                                    element_runtime,
                                    Some(snapshot),
                                    window,
                                    step_index as u32,
                                    "click_selectable_span.click",
                                );
                                output.events.extend(click_events_with_modifiers(
                                    pos,
                                    button,
                                    click_count,
                                    click_modifiers,
                                    pointer_type,
                                ));
                                active.next_step = active.next_step.saturating_add(1);
                                active.v2_step_state = None;
                                output.request_redraw = true;
                                if svc.cfg.script_auto_dump {
                                    *force_dump_label = Some(format!(
                                        "script-step-{step_index:04}-click_selectable_span-click"
                                    ));
                                }
                            }
                        } else {
                            output.events.extend(click_events_with_modifiers(
                                pos,
                                button,
                                click_count,
                                click_modifiers,
                                pointer_type,
                            ));
                            active.next_step = active.next_step.saturating_add(1);
                            active.v2_step_state = None;
                            output.request_redraw = true;
                            if svc.cfg.script_auto_dump {
                                *force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-click_selectable_span-click"
                                ));
                            }
                        }
                    } else {
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state =
                            Some(V2StepState::ClickSelectableTextSpanStable(state));
                        output.request_redraw = true;
                    }
                }
            }
        } else {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-click_selectable_span-no-semantics-match"
            ));
            *stop_script = true;
            *failure_reason =
                Some("click_selectable_text_span_stable_no_semantics_match".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-click_selectable_span-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    true
}

pub(super) fn handle_wheel_step(
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
) -> bool {
    let UiActionStepV2::Wheel {
        window: _,
        pointer_kind,
        target,
        delta_x,
        delta_y,
    } = step
    else {
        return false;
    };

    let Some(snapshot) = semantics_snapshot else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-wheel-no-semantics");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("no_semantics_snapshot".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("semantics.missing".to_string()),
            reason: Some("no_semantics_snapshot".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
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
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-wheel-no-semantics-match");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("wheel_no_semantics_match".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("selector.not_found".to_string()),
            reason: Some("wheel_no_semantics_match".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
        return true;
    };

    let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
    if let Some(ui) = ui.as_deref_mut() {
        let note = format!("wheel dx={delta_x} dy={delta_y}");
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
            Some(note.as_str()),
            svc.cfg.max_debug_string_bytes,
        );
    }
    let pointer_type = pointer_type_from_kind(pointer_kind);
    output
        .events
        .push(wheel_event(pos, delta_x, delta_y, pointer_type));

    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-wheel"));
    }

    false
}

pub(super) fn handle_move_pointer_step(
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
) -> bool {
    let UiActionStepV2::MovePointer {
        window: _,
        pointer_kind,
        target,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    let Some(snapshot) = semantics_snapshot else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-move_pointer-no-semantics");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("no_semantics_snapshot".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("semantics.missing".to_string()),
            reason: Some("no_semantics_snapshot".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
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
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-move_pointer-no-semantics-match");
        if svc.cfg.script_auto_dump {
            svc.dump_bundle(Some(&label));
        }
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: Some("move_pointer_no_semantics_match".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        svc.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code: Some("selector.not_found".to_string()),
            reason: Some("move_pointer_no_semantics_match".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
        return true;
    };

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
            Some("move_pointer"),
            svc.cfg.max_debug_string_bytes,
        );
    }
    output.events.push(move_pointer_event(pos, pointer_type));

    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-move_pointer"));
    }

    false
}
