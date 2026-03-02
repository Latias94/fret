use super::*;

pub(super) fn handle_drag_pointer_step(
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
    let UiActionStepV2::DragPointer {
        window: target_window,
        pointer_kind,
        target,
        button,
        delta_x,
        delta_y,
        steps,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    let step_has_state = active.v2_step_state.as_ref().is_some_and(|s| match s {
        V2StepState::DragPointer(state) => state.step_index == step_index,
        _ => false,
    });

    if !step_has_state {
        if let Some(target_window) =
            svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
        {
            if target_window != window {
                *handoff_to = Some(target_window);
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
                active.v2_step_state = None;
            }
        } else if target_window.is_some() {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-drag_pointer-window-not-found"
            ));
            *stop_script = true;
            *failure_reason = Some("window_target_unresolved".to_string());
            output.request_redraw = true;
        }
    }

    if *stop_script {
        active.v2_step_state = None;
    } else if handoff_to.is_some() {
        // Window-targeted: migrate to the target window before resolving semantics.
        active.v2_step_state = None;
    } else {
        let mut state = match active.v2_step_state.take() {
            Some(V2StepState::DragPointer(state)) if state.step_index == step_index => state,
            _ => {
                let Some(snapshot) = semantics_snapshot else {
                    output.request_redraw = true;
                    let label = format!("script-step-{step_index:04}-drag_pointer-no-semantics");
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
                    let label =
                        format!("script-step-{step_index:04}-drag_pointer-no-semantics-match");
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
                            note: Some("drag_pointer_no_semantics_match".to_string()),
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
                        reason: Some("drag_pointer_no_semantics_match".to_string()),
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
                        Some("drag_pointer.start"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                let end = Point::new(
                    fret_core::Px(start.x.0 + delta_x),
                    fret_core::Px(start.y.0 + delta_y),
                );
                V2DragPointerState {
                    step_index,
                    window,
                    steps: steps.max(1),
                    button,
                    start,
                    end,
                    frame: 0,
                }
            }
        };

        // Run the entire drag gesture in a single diagnostics frame to avoid leaving
        // the app in a "mouse down across frames" state, which can stall scripted
        // playback on some runners/platforms.
        let mut done = false;
        let mut burst_frames: u32 = 0;
        let burst_limit = state.steps.saturating_add(2).min(512);
        while !done && burst_frames < burst_limit {
            done = push_drag_playback_frame(&mut state, &mut output.events, pointer_type);
            burst_frames = burst_frames.saturating_add(1);
        }
        let _ = write_cursor_override_window_client_logical(
            &svc.cfg.out_dir,
            state.window,
            drag_playback_last_position(&state).x.0,
            drag_playback_last_position(&state).y.0,
        );
        if done {
            active.pending_cancel_cross_window_drag =
                Some(PendingCancelCrossWindowDrag::new(PointerId(0)));
            if let Some(ui) = ui.as_deref_mut() {
                record_hit_test_trace_for_selector(
                    &mut active.hit_test_trace,
                    ui,
                    element_runtime,
                    window,
                    semantics_snapshot,
                    &target,
                    step_index as u32,
                    state.end,
                    None,
                    Some("drag_pointer.end"),
                    svc.cfg.max_debug_string_bytes,
                );
            }
            active.v2_step_state = None;
            active.next_step = active.next_step.saturating_add(1);
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-drag_pointer"));
            }
        } else {
            active.v2_step_state = Some(V2StepState::DragPointer(state));
        }
    }

    false
}

pub(super) fn handle_drag_pointer_until_step(
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
    let UiActionStepV2::DragPointerUntil {
        window: target_window,
        pointer_kind,
        target,
        button,
        release_on_success,
        delta_x,
        delta_y,
        steps,
        predicate,
        timeout_frames,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    if *stop_script {
        active.v2_step_state = None;
    } else {
        let docking_diag = app
            .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
            .and_then(|store| store.docking_latest_for_window(window));
        let workspace_diag = app
            .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
            .and_then(|store| store.workspace_latest_for_window(window));

        let mut state = match active.v2_step_state.take() {
            Some(V2StepState::DragPointerUntil(state)) if state.step_index == step_index => state,
            _ => V2DragPointerUntilState {
                step_index,
                remaining_frames: timeout_frames,
                playback: V2DragPointerState {
                    step_index,
                    window,
                    steps: steps.max(1),
                    button,
                    start: Point::default(),
                    end: Point::default(),
                    frame: 0,
                },
                predicate: predicate.clone(),
                release_on_success,
                down_issued: false,
                mouse_buttons_override_issued: false,
                release_armed: false,
            },
        };

        let desired_window = svc.resolve_window_target_for_active_step(
            window,
            anchor_window,
            target_window.as_ref(),
        );
        if desired_window.is_none() && target_window.is_some() {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-drag_pointer_until-window-not-found"
            ));
            *stop_script = true;
            *failure_reason = Some("window_target_unresolved".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
            return true;
        }
        if let Some(desired_window) = desired_window
            && !state.down_issued
            && state.playback.frame == 0
        {
            state.playback.window = desired_window;
        }

        // Diagnostics drag playback behaves like a captured pointer: all events for a
        // `drag_pointer_until` step must be injected into the same window. If this
        // script frame is running on a different window (e.g. because the new tear-off
        // window temporarily starved redraw callbacks), hand off back to the playback
        // window before emitting any more input.
        if state.playback.window != window {
            state.remaining_frames = state.remaining_frames.saturating_sub(1);
            if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-drag_pointer_until-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some("drag_pointer_until_timeout".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                *handoff_to = Some(state.playback.window);
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(state.playback.window));
                output.request_redraw = true;
                active.v2_step_state = Some(V2StepState::DragPointerUntil(state));
                // Window-targeted: migrate before evaluating predicates or injecting input.
            }
        } else {
            // If the predicate is already satisfied (e.g. after runner-owned hover routing on a
            // previous frame), release immediately.
            let dock_drag_runtime = dock_drag_runtime_state(app, svc.known_windows.as_slice());
            let open_window_count = UiDiagnosticsService::open_window_count_for_predicates(app);
            let move_steps = state.playback.steps.max(1);
            let reached_end = state.playback.frame > move_steps;
            let predicate_ok_without_semantics = match &state.predicate {
                UiPredicateV1::EventKindSeen { event_kind } => svc
                    .per_window
                    .get(&window)
                    .is_some_and(|ring| ring.events.iter().any(|e| e.kind == *event_kind)),
                UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
                    text_font_stack_key_stable_frames >= *stable_frames
                }
                UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
                UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
                UiPredicateV1::KnownWindowCountGe { n } => open_window_count >= *n,
                UiPredicateV1::KnownWindowCountIs { n } => open_window_count == *n,
                UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => app
                    .global::<fret_runtime::PlatformCapabilities>()
                    .is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str()),
                UiPredicateV1::DockDragCurrentWindowIs {
                    window: target_window,
                } => resolve_window_target_from_known_windows(
                    window,
                    svc.known_windows.as_slice(),
                    *target_window,
                )
                .is_some_and(|target_window| {
                    dock_drag_runtime
                        .as_ref()
                        .is_some_and(|drag| drag.dragging && drag.current_window == target_window)
                }),
                UiPredicateV1::DockDragActiveIs { active } => {
                    let dragging = dock_drag_runtime.as_ref().is_some_and(|drag| drag.dragging);
                    dragging == *active
                }
                UiPredicateV1::DockDropResolveSourceIs { source } => {
                    if let Some(resolve) = docking_diag.and_then(|d| d.dock_drop_resolve.as_ref()) {
                        let have = match resolve.source {
                            fret_runtime::DockDropResolveSource::InvertDocking => "invert_docking",
                            fret_runtime::DockDropResolveSource::OutsideWindow => "outside_window",
                            fret_runtime::DockDropResolveSource::FloatZone => "float_zone",
                            fret_runtime::DockDropResolveSource::EmptyDockSpace => {
                                "empty_dock_space"
                            }
                            fret_runtime::DockDropResolveSource::LayoutBoundsMiss => {
                                "layout_bounds_miss"
                            }
                            fret_runtime::DockDropResolveSource::LatchedPreviousHover => {
                                "latched_previous_hover"
                            }
                            fret_runtime::DockDropResolveSource::TabBar => "tab_bar",
                            fret_runtime::DockDropResolveSource::FloatingTitleBar => {
                                "floating_title_bar"
                            }
                            fret_runtime::DockDropResolveSource::OuterHintRect => "outer_hint_rect",
                            fret_runtime::DockDropResolveSource::InnerHintRect => "inner_hint_rect",
                            fret_runtime::DockDropResolveSource::None => "none",
                        };
                        have == source.as_str()
                    } else {
                        false
                    }
                }
                UiPredicateV1::DockDropResolvedIsSome { some } => docking_diag
                    .and_then(|d| d.dock_drop_resolve.as_ref())
                    .is_some_and(|d| d.resolved.is_some() == *some),
                UiPredicateV1::DockDropResolvedZoneIs { zone } => {
                    if let Some(resolved) = docking_diag
                        .and_then(|d| d.dock_drop_resolve.as_ref())
                        .and_then(|d| d.resolved.as_ref())
                    {
                        let have = match resolved.zone {
                            fret_core::dock::DropZone::Center => "center",
                            fret_core::dock::DropZone::Left => "left",
                            fret_core::dock::DropZone::Right => "right",
                            fret_core::dock::DropZone::Top => "top",
                            fret_core::dock::DropZone::Bottom => "bottom",
                        };
                        have == zone.as_str()
                    } else {
                        false
                    }
                }
                UiPredicateV1::DockDropResolvedInsertIndexIs { index } => docking_diag
                    .and_then(|d| d.dock_drop_resolve.as_ref())
                    .and_then(|d| d.resolved.as_ref())
                    .is_some_and(|d| d.insert_index == Some(*index as usize)),
                UiPredicateV1::DockGraphCanonicalIs { canonical } => docking_diag
                    .and_then(|d| d.dock_graph_stats)
                    .is_some_and(|s| s.canonical_ok == *canonical),
                UiPredicateV1::DockGraphSignatureIs { signature } => docking_diag
                    .and_then(|d| d.dock_graph_signature.as_ref())
                    .is_some_and(|s| s.signature == *signature),
                UiPredicateV1::DockGraphSignatureContains { needle } => docking_diag
                    .and_then(|d| d.dock_graph_signature.as_ref())
                    .is_some_and(|s| s.signature.contains(needle)),
                UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => docking_diag
                    .and_then(|d| d.dock_graph_signature.as_ref())
                    .is_some_and(|s| s.fingerprint64 == *fingerprint64),
                UiPredicateV1::WorkspaceTabStripActiveOverflowIs { overflow, pane_id } => {
                    workspace_diag
                        .and_then(|w| {
                            w.tab_strip_active_visibility.iter().rev().find(|s| {
                                s.status
                                    == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                                    && pane_id.as_ref().is_none_or(|id| {
                                        s.pane_id
                                            .as_ref()
                                            .is_some_and(|p| p.as_ref() == id.as_str())
                                    })
                            })
                        })
                        .is_some_and(|s| s.overflow == *overflow)
                }
                UiPredicateV1::WorkspaceTabStripActiveVisibleIs { visible, pane_id } => {
                    workspace_diag
                        .and_then(|w| {
                            w.tab_strip_active_visibility.iter().rev().find(|s| {
                                s.status
                                    == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                                    && pane_id.as_ref().is_none_or(|id| {
                                        s.pane_id
                                            .as_ref()
                                            .is_some_and(|p| p.as_ref() == id.as_str())
                                    })
                            })
                        })
                        .is_some_and(|s| s.active_visible == *visible)
                }
                _ => false,
            };
            let input_ctx = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window));
            let predicate_ok = if let Some(snapshot) = semantics_snapshot {
                eval_predicate(
                    snapshot,
                    window_bounds,
                    window,
                    active.scope_root_for_window(window),
                    input_ctx,
                    element_runtime,
                    app.global::<fret_runtime::WindowTextInputSnapshotService>()
                        .and_then(|svc| svc.snapshot(window)),
                    app.global::<fret_core::RendererTextPerfSnapshot>().copied(),
                    app.global::<fret_core::RendererTextFontTraceSnapshot>(),
                    svc.known_windows.as_slice(),
                    open_window_count,
                    app.global::<fret_runtime::PlatformCapabilities>(),
                    docking_diag,
                    workspace_diag,
                    dock_drag_runtime.as_ref(),
                    text_font_stack_key_stable_frames,
                    font_catalog_populated,
                    system_font_rescan_idle,
                    &state.predicate,
                )
            } else {
                predicate_ok_without_semantics
            };
            let is_current_window_predicate = matches!(
                &state.predicate,
                UiPredicateV1::DockDragCurrentWindowIs { .. }
            );
            let preserve_explicit_cursor_override = is_current_window_predicate
                && active
                    .last_explicit_cursor_override
                    .is_some_and(|t| match t {
                        CursorOverrideTarget::WindowClientPhysical(w)
                        | CursorOverrideTarget::WindowClientLogical(w) => {
                            w != state.playback.window
                        }
                        CursorOverrideTarget::ScreenPhysical => false,
                    });
            let predicate_ok = if is_current_window_predicate && !reached_end {
                false
            } else {
                predicate_ok
            };
            let predicate_ok = predicate_ok || state.release_armed;

            if predicate_ok {
                if state.down_issued {
                    let release_pos = drag_playback_last_position(&state.playback);
                    // For `DockDragCurrentWindowIs` predicates, some scripts rely on preserving a
                    // prior, explicit cursor placement in a different window so runner hover
                    // routing can observe the correct "current window" at the moment we
                    // resolve/release. When no explicit cross-window placement has been issued,
                    // keep the cursor override synced to the playback position so scripts can
                    // still simulate crossing window boundaries.
                    if !preserve_explicit_cursor_override {
                        let _ = write_cursor_override_window_client_logical(
                            &svc.cfg.out_dir,
                            state.playback.window,
                            release_pos.x.0,
                            release_pos.y.0,
                        );
                    }

                    if !state.release_on_success {
                        active.pointer_session = Some(V2PointerSessionState {
                            window: state.playback.window,
                            button: state.playback.button,
                            pointer_type,
                            modifiers: Modifiers::default(),
                            position: release_pos,
                        });
                        active.v2_step_state = None;
                        active.next_step = active.next_step.saturating_add(1);
                        output.request_redraw = true;
                        if svc.cfg.script_auto_dump {
                            *force_dump_label =
                                Some(format!("script-step-{step_index:04}-drag_pointer_until"));
                        }
                        return true;
                    }

                    if !state.release_armed {
                        // The runner polls cursor overrides at the top of the event loop. If we
                        // emit `Up/Drop` in the same frame as the override write, the release can
                        // be observed with a stale cursor screen position during cross-window
                        // drags. Stage the release on the next frame.
                        state.release_armed = true;
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state = Some(V2StepState::DragPointerUntil(state));
                        output.request_redraw = true;
                    } else {
                        let drag_pointer_id =
                            dock_drag_pointer_id_best_effort(app, svc.known_windows.as_slice())
                                .unwrap_or(PointerId(0));
                        let cross_window_hover_active = dock_drag_runtime
                            .as_ref()
                            .is_some_and(|d| d.cross_window_hover);
                        if !cross_window_hover_active {
                            output.events.extend(pointer_up_with_internal_drop_events(
                                state.playback.button,
                                release_pos,
                                pointer_type,
                            ));
                        }
                        let _ = write_mouse_buttons_override_all_windows_v1(
                            &svc.cfg.out_dir,
                            match state.playback.button {
                                UiMouseButtonV1::Left => Some(false),
                                _ => None,
                            },
                            match state.playback.button {
                                UiMouseButtonV1::Right => Some(false),
                                _ => None,
                            },
                            match state.playback.button {
                                UiMouseButtonV1::Middle => Some(false),
                                _ => None,
                            },
                        );
                        active.pending_cancel_cross_window_drag =
                            Some(PendingCancelCrossWindowDrag::new(drag_pointer_id));
                        active.v2_step_state = None;
                        active.next_step = active.next_step.saturating_add(1);
                        if svc.cfg.script_auto_dump {
                            *force_dump_label =
                                Some(format!("script-step-{step_index:04}-drag_pointer_until"));
                        }
                    }
                } else {
                    active.v2_step_state = None;
                    active.next_step = active.next_step.saturating_add(1);
                    if svc.cfg.script_auto_dump {
                        *force_dump_label =
                            Some(format!("script-step-{step_index:04}-drag_pointer_until"));
                    }
                }
            } else if state.remaining_frames == 0 {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-drag_pointer_until-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some("drag_pointer_until_timeout".to_string());
                active.v2_step_state = None;
            } else {
                // Initialize start/end positions on the first frame.
                if state.playback.frame == 0 && state.playback.start == Point::default() {
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
                            let start = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                            let end = Point::new(
                                fret_core::Px(start.x.0 + delta_x),
                                fret_core::Px(start.y.0 + delta_y),
                            );
                            state.playback.start = start;
                            state.playback.end = end;
                        } else {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-drag_pointer_until-no-semantics-match"
                            ));
                            *stop_script = true;
                            *failure_reason = Some("drag_pointer_until_no_match".to_string());
                            active.v2_step_state = None;
                            output.request_redraw = true;
                        }
                    }
                }

                // Wait for semantics to become available before selecting coordinates.
                if !*stop_script
                    && state.playback.frame == 0
                    && state.playback.start == Point::default()
                {
                    state.remaining_frames = state.remaining_frames.saturating_sub(1);
                    active.v2_step_state = Some(V2StepState::DragPointerUntil(state));
                    output.request_redraw = true;
                } else if !*stop_script {
                    let move_steps = state.playback.steps.max(1);
                    let reached_end = state.playback.frame > move_steps;
                    let cross_window_hover_active = dock_drag_runtime
                        .as_ref()
                        .is_some_and(|d| d.cross_window_hover);
                    let suppress_pointer_events_for_cross_window_hover = cross_window_hover_active
                        && matches!(pointer_type, PointerType::Mouse)
                        && dock_drag_runtime
                            .as_ref()
                            .is_some_and(|d| d.current_window != state.playback.window)
                        && state.playback.frame >= 1;

                    // Drive pointer-down + move segments until we reach `end`. Do not emit a
                    // pointer-up until the predicate is satisfied; `drag_pointer_until` is
                    // allowed to "hold" the drag at the end position across frames.
                    if !reached_end {
                        if suppress_pointer_events_for_cross_window_hover {
                            state.playback.frame = state.playback.frame.saturating_add(1);
                        } else {
                            let _ = push_drag_playback_frame(
                                &mut state.playback,
                                &mut output.events,
                                pointer_type,
                            );
                        }
                    } else {
                        if !suppress_pointer_events_for_cross_window_hover {
                            output.events.extend(pointer_move_with_internal_over_events(
                                state.playback.button,
                                state.playback.end,
                                pointer_type,
                            ));
                        }
                    }

                    // For cross-window hover, the runner's "true cursor" must remain over the
                    // hovered window for hover routing to stay active. Avoid overwriting the
                    // cursor override with a position expressed in the playback window's client
                    // coordinates; keep the most recent explicit cursor placement instead (e.g.
                    // a prior `set_cursor_in_window` step).
                    if !preserve_explicit_cursor_override
                        && !suppress_pointer_events_for_cross_window_hover
                    {
                        let cursor_pos = drag_playback_last_position(&state.playback);
                        let _ = write_cursor_override_window_client_logical(
                            &svc.cfg.out_dir,
                            state.playback.window,
                            cursor_pos.x.0,
                            cursor_pos.y.0,
                        );
                    }
                    if state.playback.frame >= 1 {
                        state.down_issued = true;
                    }
                    if state.down_issued
                        && !state.mouse_buttons_override_issued
                        && matches!(pointer_type, PointerType::Mouse)
                    {
                        let _ = write_mouse_buttons_override_all_windows_v1(
                            &svc.cfg.out_dir,
                            match state.playback.button {
                                UiMouseButtonV1::Left => Some(true),
                                _ => None,
                            },
                            match state.playback.button {
                                UiMouseButtonV1::Right => Some(true),
                                _ => None,
                            },
                            match state.playback.button {
                                UiMouseButtonV1::Middle => Some(true),
                                _ => None,
                            },
                        );
                        state.mouse_buttons_override_issued = true;
                    }

                    state.remaining_frames = state.remaining_frames.saturating_sub(1);

                    active.v2_step_state = Some(V2StepState::DragPointerUntil(state));
                    output.request_redraw = true;
                }
            }
        }
    }

    true
}

pub(super) struct DragToStepReturn {
    pub(super) should_return_output: bool,
    pub(super) requeue_active_for_window: bool,
}

pub(super) fn handle_drag_to_step(
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
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> Option<DragToStepReturn> {
    let UiActionStepV2::DragTo {
        window: target_window,
        pointer_kind,
        from,
        to,
        button,
        steps,
        timeout_frames,
    } = step
    else {
        return None;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
        if target_window != window {
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
            active.v2_step_state = None;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-drag_to-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    if *stop_script {
        active.v2_step_state = None;
    } else if handoff_to.is_some() {
        active.v2_step_state = None;
    } else if let Some(snapshot) = semantics_snapshot {
        let mut state = match active.v2_step_state.take() {
            Some(V2StepState::DragTo(mut state)) if state.step_index == step_index => {
                state.remaining_frames = state.remaining_frames.min(timeout_frames);
                state
            }
            _ => V2DragToState {
                step_index,
                remaining_frames: timeout_frames,
                playback: None,
                down_issued: false,
                mouse_buttons_override_issued: false,
            },
        };

        let mut playback = if let Some(playback) = state.playback.take() {
            playback
        } else {
            let from_node = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &from,
                active.scope_root_for_window(window),
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            );
            let to_node = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &to,
                active.scope_root_for_window(window),
                step_index as u32,
                svc.cfg.redact_text,
                &mut active.selector_resolution_trace,
            );
            if let (Some(from_node), Some(to_node)) = (from_node, to_node) {
                let start = ui
                    .as_deref()
                    .map(|ui| {
                        wheel_position_prefer_intended_hit(
                            snapshot,
                            ui,
                            from_node,
                            from_node.bounds,
                            window_bounds,
                        )
                    })
                    .unwrap_or_else(|| {
                        center_of_rect_clamped_to_rect(from_node.bounds, window_bounds)
                    });
                let end = ui
                    .as_deref()
                    .map(|ui| {
                        wheel_position_prefer_intended_hit(
                            snapshot,
                            ui,
                            to_node,
                            to_node.bounds,
                            window_bounds,
                        )
                    })
                    .unwrap_or_else(|| {
                        center_of_rect_clamped_to_rect(to_node.bounds, window_bounds)
                    });
                if let Some(ui) = ui.as_deref_mut() {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &from,
                        step_index as u32,
                        start,
                        Some(from_node),
                        Some("drag_to.start"),
                        svc.cfg.max_debug_string_bytes,
                    );
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &to,
                        step_index as u32,
                        end,
                        Some(to_node),
                        Some("drag_to.end"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                V2DragPointerState {
                    step_index,
                    window,
                    steps: steps.max(1),
                    button,
                    start,
                    end,
                    frame: 0,
                }
            } else if state.remaining_frames == 0 {
                output.request_redraw = true;
                let label = format!("script-step-{step_index:04}-drag_to-timeout");
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
                        note: Some("drag_to_timeout".to_string()),
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
                    reason_code: reason_code_for_script_failure("drag_to_timeout")
                        .map(|s| s.to_string()),
                    reason: Some("drag_to_timeout".to_string()),
                    evidence: script_evidence_for_active(active),
                    last_bundle_dir: svc
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&svc.cfg.out_dir, p)),
                    last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
                });
                return Some(DragToStepReturn {
                    should_return_output: true,
                    requeue_active_for_window: false,
                });
            } else {
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::DragTo(state));
                output.request_redraw = true;
                output.effects.push(Effect::RequestAnimationFrame(window));
                return Some(DragToStepReturn {
                    should_return_output: true,
                    requeue_active_for_window: true,
                });
            }
        };

        let done = push_drag_playback_frame(&mut playback, &mut output.events, pointer_type);
        let _ = write_cursor_override_window_client_logical(
            &svc.cfg.out_dir,
            playback.window,
            drag_playback_last_position(&playback).x.0,
            drag_playback_last_position(&playback).y.0,
        );

        if playback.frame >= 1 {
            state.down_issued = true;
        }
        if state.down_issued
            && !state.mouse_buttons_override_issued
            && matches!(pointer_type, PointerType::Mouse)
        {
            let _ = write_mouse_buttons_override_all_windows_v1(
                &svc.cfg.out_dir,
                match playback.button {
                    UiMouseButtonV1::Left => Some(true),
                    _ => None,
                },
                match playback.button {
                    UiMouseButtonV1::Right => Some(true),
                    _ => None,
                },
                match playback.button {
                    UiMouseButtonV1::Middle => Some(true),
                    _ => None,
                },
            );
            state.mouse_buttons_override_issued = true;
        }

        output.request_redraw = true;
        if done {
            if state.mouse_buttons_override_issued && matches!(pointer_type, PointerType::Mouse) {
                let _ = write_mouse_buttons_override_all_windows_v1(
                    &svc.cfg.out_dir,
                    match playback.button {
                        UiMouseButtonV1::Left => Some(false),
                        _ => None,
                    },
                    match playback.button {
                        UiMouseButtonV1::Right => Some(false),
                        _ => None,
                    },
                    match playback.button {
                        UiMouseButtonV1::Middle => Some(false),
                        _ => None,
                    },
                );
            }
            active.pending_cancel_cross_window_drag =
                Some(PendingCancelCrossWindowDrag::new(PointerId(0)));
            active.v2_step_state = None;
            active.next_step = active.next_step.saturating_add(1);
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-drag_to"));
            }
        } else {
            state.playback = Some(playback);
            active.v2_step_state = Some(V2StepState::DragTo(state));
        }
    } else {
        *force_dump_label = Some(format!("script-step-{step_index:04}-drag_to-no-semantics"));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    Some(DragToStepReturn {
        should_return_output: false,
        requeue_active_for_window: false,
    })
}
