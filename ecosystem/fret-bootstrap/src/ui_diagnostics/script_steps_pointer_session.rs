use super::*;

fn dock_drag_active(app: &App) -> bool {
    app.drag(fret_core::PointerId(0)).is_some_and(|d| {
        (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && d.dragging
    })
}

fn seed_pointer_session_position_from_explicit_cursor_override(
    active: &ActiveScript,
    window: AppWindowId,
) -> Option<Point> {
    let last = active.last_explicit_cursor_override_pos?;
    match last.target {
        CursorOverrideTarget::WindowClientLogical(w) if w == window => {
            Some(Point::new(Px(last.x_px), Px(last.y_px)))
        }
        _ => None,
    }
}

pub(super) fn handle_pointer_down_step(
    svc: &mut UiDiagnosticsService,
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
    let UiActionStepV2::PointerDown {
        window: target_window,
        pointer_kind,
        target,
        button: button_ui,
        modifiers,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    if active.pointer_session.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-pointer_down-pointer-session-already-active"
        ));
        *stop_script = true;
        *failure_reason = Some("pointer_session_already_active".to_string());
        output.request_redraw = true;
    }

    if !*stop_script {
        let test_id_hint = match &target {
            UiSelectorV1::TestId { id } => Some(id.as_str()),
            _ => None,
        };
        if let Some(target_window) = svc.resolve_window_target_for_active_step_with_test_id_hint(
            window,
            anchor_window,
            target_window.as_ref(),
            test_id_hint,
        ) {
            if target_window != window {
                *handoff_to = Some(target_window);
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            }
        } else if target_window.is_some() {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-pointer_down-window-not-found"
            ));
            *stop_script = true;
            *failure_reason = Some("window_target_unresolved".to_string());
            output.request_redraw = true;
        }
    }

    if *stop_script {
        active.v2_step_state = None;
        active.wait_until = None;
        active.screenshot_wait = None;
    } else if handoff_to.is_some() {
        // Window-targeted: migrate to the target window before resolving semantics.
    } else {
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
                        Some("pointer_down"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }

                let modifiers = core_modifiers_from_ui(modifiers);
                let pointer_id = PointerId(0);
                let pointer_type = pointer_type_from_kind(pointer_kind);
                let button = match button_ui {
                    UiMouseButtonV1::Left => MouseButton::Left,
                    UiMouseButtonV1::Right => MouseButton::Right,
                    UiMouseButtonV1::Middle => MouseButton::Middle,
                };

                output.events.push(Event::Pointer(PointerEvent::Move {
                    pointer_id,
                    position: pos,
                    buttons: MouseButtons::default(),
                    modifiers,
                    pointer_type,
                }));
                output.events.push(Event::Pointer(PointerEvent::Down {
                    pointer_id,
                    position: pos,
                    button,
                    modifiers,
                    click_count: 1,
                    pointer_type,
                }));
                let _ = write_cursor_override_window_client_logical(
                    &svc.cfg.out_dir,
                    window,
                    pos.x.0,
                    pos.y.0,
                );
                let _ = write_mouse_buttons_override_window_v1(
                    &svc.cfg.out_dir,
                    window,
                    match button_ui {
                        UiMouseButtonV1::Left => Some(true),
                        _ => None,
                    },
                    match button_ui {
                        UiMouseButtonV1::Right => Some(true),
                        _ => None,
                    },
                    match button_ui {
                        UiMouseButtonV1::Middle => Some(true),
                        _ => None,
                    },
                );

                active.pointer_session = Some(V2PointerSessionState {
                    window,
                    button: button_ui,
                    pointer_type,
                    modifiers,
                    position: pos,
                });
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-pointer_down"));
                }
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-pointer_down-no-semantics-match"
                ));
                *stop_script = true;
                *failure_reason = Some("selector.not_found".to_string());
                output.request_redraw = true;
            }
        } else {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-pointer_down-no-semantics"
            ));
            *stop_script = true;
            *failure_reason = Some("no_semantics_snapshot".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        }
    }

    true
}

pub(super) fn handle_pointer_move_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::PointerMove {
        window: target_window_spec,
        pointer_kind,
        delta_x,
        delta_y,
        steps,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    if let Some(mut session) = active.pointer_session.clone() {
        let allow_cross_window_migration = dock_drag_active(app);
        let resolved_target_window = svc.resolve_window_target_for_active_step(
            window,
            anchor_window,
            target_window_spec.as_ref(),
        );

        match resolved_target_window {
            Some(target_window) => {
                if target_window != window {
                    if allow_cross_window_migration && target_window_spec.is_some() {
                        *handoff_to = Some(target_window);
                        output
                            .effects
                            .push(Effect::RequestAnimationFrame(target_window));
                        output.request_redraw = true;
                    } else if target_window == session.window {
                        *handoff_to = Some(target_window);
                        output
                            .effects
                            .push(Effect::RequestAnimationFrame(target_window));
                        output.request_redraw = true;
                    } else {
                        // Pointer sessions are window-local by default. If the script resolves a
                        // window target that doesn't match the active session, prefer continuing
                        // the session in its owning window rather than failing the entire run.
                        push_script_event_log(
                            active,
                            &svc.cfg,
                            UiScriptEventLogEntryV1 {
                                unix_ms: unix_ms_now(),
                                kind: "diag.pointer_move_window_redirect".to_string(),
                                step_index: Some(step_index as u32),
                                note: Some(format!(
                                    "pointer_move window mismatch: current={} target={} session={}; redirecting to session window",
                                    window.data().as_ffi(),
                                    target_window.data().as_ffi(),
                                    session.window.data().as_ffi(),
                                )),
                                bundle_dir: None,
                                window: Some(window.data().as_ffi()),
                                tick_id: Some(app.tick_id().0),
                                frame_id: Some(app.frame_id().0),
                                window_snapshot_seq: None,
                            },
                        );
                        if session.window != window {
                            *handoff_to = Some(session.window);
                            output
                                .effects
                                .push(Effect::RequestAnimationFrame(session.window));
                            output.request_redraw = true;
                        }
                    }
                } else if session.window != window {
                    // We are already in the script-targeted window, but the pointer session still
                    // belongs to a different window.
                    if allow_cross_window_migration && target_window_spec.is_some() {
                        if let Some(seed) =
                            seed_pointer_session_position_from_explicit_cursor_override(active, window)
                        {
                            session.window = window;
                            session.position = seed;
                            active.pointer_session = Some(session.clone());
                        } else {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-pointer_move-missing-cursor-seed"
                            ));
                            *stop_script = true;
                            *failure_reason =
                                Some("pointer_session_missing_cursor_seed".to_string());
                            output.request_redraw = true;
                        }
                    } else {
                        *handoff_to = Some(session.window);
                        output
                            .effects
                            .push(Effect::RequestAnimationFrame(session.window));
                        output.request_redraw = true;
                    }
                }
            }
            None => {
                if target_window_spec.is_some() {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-pointer_move-window-not-found"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("window_target_unresolved".to_string());
                    output.request_redraw = true;
                } else if session.window != window {
                    // The script migrated away from the window that owns the pointer session.
                    *handoff_to = Some(session.window);
                    output
                        .effects
                        .push(Effect::RequestAnimationFrame(session.window));
                    output.request_redraw = true;
                }
            }
        }

        if *stop_script {
            active.v2_step_state = None;
        } else if handoff_to.is_some() {
            // Window-targeted: migrate to the target window before continuing the session.
            active.v2_step_state = None;
        } else {
            let mut state = match active.v2_step_state.take() {
                Some(V2StepState::PointerMove(state)) if state.step_index == step_index => state,
                _ => {
                    let steps = steps.max(1);
                    let start = session.position;
                    let end = Point::new(
                        fret_core::Px(start.x.0 + delta_x),
                        fret_core::Px(start.y.0 + delta_y),
                    );
                    V2PointerMoveState {
                        step_index,
                        steps,
                        start,
                        end,
                        frame: 1,
                    }
                }
            };

            let pressed_buttons = match session.button {
                UiMouseButtonV1::Left => MouseButtons {
                    left: true,
                    ..Default::default()
                },
                UiMouseButtonV1::Right => MouseButtons {
                    right: true,
                    ..Default::default()
                },
                UiMouseButtonV1::Middle => MouseButtons {
                    middle: true,
                    ..Default::default()
                },
            };

            if let Some(want) = pointer_kind {
                let want_type = pointer_type_from_kind(Some(want));
                if want_type != session.pointer_type {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-pointer_move-pointer-kind-mismatch"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("pointer_session_pointer_kind_mismatch".to_string());
                    output.request_redraw = true;
                    active.v2_step_state = None;
                    return true;
                }
            }

            let pointer_id = PointerId(0);
            let pointer_type = session.pointer_type;

            if state.frame == 0 {
                state.frame = 1;
            }

            if state.frame <= state.steps {
                let t = state.frame as f32 / state.steps as f32;
                let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
                let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
                let position = Point::new(fret_core::Px(x), fret_core::Px(y));

                output.events.push(Event::Pointer(PointerEvent::Move {
                    pointer_id,
                    position,
                    buttons: pressed_buttons,
                    modifiers: session.modifiers,
                    pointer_type,
                }));
                output
                    .events
                    .push(Event::InternalDrag(fret_core::InternalDragEvent {
                        pointer_id,
                        position,
                        kind: fret_core::InternalDragKind::Over,
                        modifiers: session.modifiers,
                    }));

                session.position = position;
                active.pointer_session = Some(session);
                let preserve_explicit_cursor_override = active
                    .last_explicit_cursor_override
                    .is_some_and(|t| match t {
                        CursorOverrideTarget::ScreenPhysical => true,
                        CursorOverrideTarget::WindowClientPhysical(w)
                        | CursorOverrideTarget::WindowClientLogical(w) => w != window,
                    });
                if !preserve_explicit_cursor_override {
                    let _ = write_cursor_override_window_client_logical(
                        &svc.cfg.out_dir,
                        window,
                        position.x.0,
                        position.y.0,
                    );
                }

                state.frame = state.frame.saturating_add(1);
                active.v2_step_state = Some(V2StepState::PointerMove(state));
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output.request_redraw = true;
            } else {
                session.position = state.end;
                active.pointer_session = Some(session);
                let preserve_explicit_cursor_override = active
                    .last_explicit_cursor_override
                    .is_some_and(|t| match t {
                        CursorOverrideTarget::ScreenPhysical => true,
                        CursorOverrideTarget::WindowClientPhysical(w)
                        | CursorOverrideTarget::WindowClientLogical(w) => w != window,
                    });
                if !preserve_explicit_cursor_override {
                    let _ = write_cursor_override_window_client_logical(
                        &svc.cfg.out_dir,
                        window,
                        state.end.x.0,
                        state.end.y.0,
                    );
                }

                active.v2_step_state = None;
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-pointer_move"));
                }
            }
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-pointer_move-no-session"
        ));
        *stop_script = true;
        *failure_reason = Some("pointer_session_missing".to_string());
        output.request_redraw = true;
        active.v2_step_state = None;
    }

    true
}

pub(super) fn handle_pointer_up_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::PointerUp {
        window: target_window_spec,
        pointer_kind,
        button: want_button,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    if let Some(mut session) = active.pointer_session.clone() {
        let pointer_id = PointerId(0);
        let cross_window_dock_drag_active = app.drag(pointer_id).is_some_and(|d| {
            (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                && d.dragging
                && d.cross_window_hover
        });

        let allow_cross_window_migration = dock_drag_active(app);
        let resolved_target_window = svc.resolve_window_target_for_active_step(
            window,
            anchor_window,
            target_window_spec.as_ref(),
        );

        match resolved_target_window {
            Some(target_window) => {
                if target_window != window {
                    if allow_cross_window_migration && target_window_spec.is_some() {
                        *handoff_to = Some(target_window);
                        output
                            .effects
                            .push(Effect::RequestAnimationFrame(target_window));
                        output.request_redraw = true;
                    } else if target_window == session.window {
                        *handoff_to = Some(target_window);
                        output
                            .effects
                            .push(Effect::RequestAnimationFrame(target_window));
                        output.request_redraw = true;
                    } else {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-pointer_up-window-mismatch"
                        ));
                        *stop_script = true;
                        *failure_reason =
                            Some("pointer_session_cross_window_unsupported".to_string());
                        output.request_redraw = true;
                    }
                } else if session.window != window {
                    if allow_cross_window_migration && target_window_spec.is_some() {
                        if let Some(seed) =
                            seed_pointer_session_position_from_explicit_cursor_override(active, window)
                        {
                            session.window = window;
                            session.position = seed;
                            active.pointer_session = Some(session.clone());
                        } else {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-pointer_up-missing-cursor-seed"
                            ));
                            *stop_script = true;
                            *failure_reason =
                                Some("pointer_session_missing_cursor_seed".to_string());
                            output.request_redraw = true;
                            active.v2_step_state = None;
                        }
                    } else if !cross_window_dock_drag_active {
                        *handoff_to = Some(session.window);
                        output
                            .effects
                            .push(Effect::RequestAnimationFrame(session.window));
                        output.request_redraw = true;
                    }
                }
            }
            None => {
                if target_window_spec.is_some() {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-pointer_up-window-not-found"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("window_target_unresolved".to_string());
                    output.request_redraw = true;
                } else if session.window != window && !cross_window_dock_drag_active {
                    // The script migrated away from the window that owns the pointer session.
                    *handoff_to = Some(session.window);
                    output
                        .effects
                        .push(Effect::RequestAnimationFrame(session.window));
                    output.request_redraw = true;
                }
            }
        }

        if *stop_script {
            active.v2_step_state = None;
        } else if handoff_to.is_some() {
            // Window-targeted: migrate to the target window before releasing the session.
        } else {
            if let Some(want) = want_button
                && want != session.button
            {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-pointer_up-button-mismatch"
                ));
                *stop_script = true;
                *failure_reason = Some("pointer_up_button_mismatch".to_string());
                output.request_redraw = true;
                active.v2_step_state = None;
            } else if let Some(want) = pointer_kind
                && pointer_type_from_kind(Some(want)) != session.pointer_type
            {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-pointer_up-pointer-kind-mismatch"
                ));
                *stop_script = true;
                *failure_reason = Some("pointer_session_pointer_kind_mismatch".to_string());
                output.request_redraw = true;
                active.v2_step_state = None;
            } else {
                // When a cross-window dock drag is active, the desktop runner owns drop routing
                // (`InternalDragKind::Drop`) based on the current cursor override. Injecting a
                // window-local `PointerUp`/`InternalDrag::Drop` here can prematurely cancel the
                // drag session before the runner dispatches the cross-window drop.
                if cross_window_dock_drag_active {
                    // Preserve any explicit cursor override set by earlier script steps (e.g.
                    // `move_pointer` targeting a dock hint). Overwriting the cursor here can
                    // "snap back" to a stale pointer-session position and cause the runner-routed
                    // cross-window drop to land in the wrong place.
                    if active.last_explicit_cursor_override.is_none() {
                        let _ = write_cursor_override_window_client_logical(
                            &svc.cfg.out_dir,
                            window,
                            session.position.x.0,
                            session.position.y.0,
                        );
                    }
                    let _ = write_mouse_buttons_override_all_windows_v1(
                        &svc.cfg.out_dir,
                        match session.button {
                            UiMouseButtonV1::Left => Some(false),
                            _ => None,
                        },
                        match session.button {
                            UiMouseButtonV1::Right => Some(false),
                            _ => None,
                        },
                        match session.button {
                            UiMouseButtonV1::Middle => Some(false),
                            _ => None,
                        },
                    );
                    active.pending_cancel_cross_window_drag =
                        Some(PendingCancelCrossWindowDrag::new(pointer_id));
                    push_script_event_log(
                        active,
                        &svc.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "diag.pending_cancel_drag".to_string(),
                            step_index: Some(step_index.min(u32::MAX as usize) as u32),
                            note: Some(format!("pointer_id={}", pointer_id.0)),
                            bundle_dir: None,
                            window: Some(window.data().as_ffi()),
                            tick_id: Some(app.tick_id().0),
                            frame_id: Some(app.frame_id().0),
                            window_snapshot_seq: None,
                        },
                    );

                    active.pointer_session = None;
                    active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    if svc.cfg.script_auto_dump {
                        *force_dump_label =
                            Some(format!("script-step-{step_index:04}-pointer_up-cross-window"));
                    }
                    return true;
                }

                let pointer_type = session.pointer_type;
                let button = match session.button {
                    UiMouseButtonV1::Left => MouseButton::Left,
                    UiMouseButtonV1::Right => MouseButton::Right,
                    UiMouseButtonV1::Middle => MouseButton::Middle,
                };

                output.events.push(Event::Pointer(PointerEvent::Up {
                    pointer_id,
                    position: session.position,
                    button,
                    modifiers: session.modifiers,
                    is_click: false,
                    click_count: 1,
                    pointer_type,
                }));
                output
                    .events
                    .push(Event::InternalDrag(fret_core::InternalDragEvent {
                        pointer_id,
                        position: session.position,
                        kind: fret_core::InternalDragKind::Drop,
                        modifiers: session.modifiers,
                    }));
                let preserve_explicit_cursor_override = active
                    .last_explicit_cursor_override
                    .is_some_and(|t| match t {
                        CursorOverrideTarget::ScreenPhysical => true,
                        CursorOverrideTarget::WindowClientPhysical(w)
                        | CursorOverrideTarget::WindowClientLogical(w) => w != window,
                    });
                if !preserve_explicit_cursor_override {
                    let _ = write_cursor_override_window_client_logical(
                        &svc.cfg.out_dir,
                        window,
                        session.position.x.0,
                        session.position.y.0,
                    );
                }
                let _ = write_mouse_buttons_override_all_windows_v1(
                    &svc.cfg.out_dir,
                    match session.button {
                        UiMouseButtonV1::Left => Some(false),
                        _ => None,
                    },
                    match session.button {
                        UiMouseButtonV1::Right => Some(false),
                        _ => None,
                    },
                    match session.button {
                        UiMouseButtonV1::Middle => Some(false),
                        _ => None,
                    },
                );
                active.pending_cancel_cross_window_drag =
                    Some(PendingCancelCrossWindowDrag::new(pointer_id));
                push_script_event_log(
                    active,
                    &svc.cfg,
                    UiScriptEventLogEntryV1 {
                        unix_ms: unix_ms_now(),
                        kind: "diag.pending_cancel_drag".to_string(),
                        step_index: Some(step_index.min(u32::MAX as usize) as u32),
                        note: Some(format!("pointer_id={}", pointer_id.0)),
                        bundle_dir: None,
                        window: Some(window.data().as_ffi()),
                        tick_id: Some(app.tick_id().0),
                        frame_id: Some(app.frame_id().0),
                        window_snapshot_seq: None,
                    },
                );

                active.pointer_session = None;
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-pointer_up"));
                }
            }
        }
    } else {
        // Cross-window dock drags (runner-routed) can be held by diagnostics via mouse button
        // overrides without a pointer session (e.g. `drag_pointer_until` with
        // `release_on_success=false` while `cross_window_hover` is active).
        //
        // In that state, allow a best-effort pointer release that does not inject an
        // `InternalDrag::Drop` into the current window. The runner owns cross-window drop routing
        // based on the current cursor override.
        let pointer_id = PointerId(0);
        let cross_window_dock_drag_active = app.drag(pointer_id).is_some_and(|d| {
            (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                && d.dragging
                && d.cross_window_hover
        });

        if cross_window_dock_drag_active {
            let _ = write_mouse_buttons_override_all_windows_v1(
                &svc.cfg.out_dir,
                Some(false),
                Some(false),
                Some(false),
            );
            active.pending_cancel_cross_window_drag =
                Some(PendingCancelCrossWindowDrag::new(pointer_id));
            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-pointer_up-cross-window"
                ));
            }
        } else {
            *force_dump_label = Some(format!("script-step-{step_index:04}-pointer_up-no-session"));
            *stop_script = true;
            *failure_reason = Some("pointer_session_missing".to_string());
            output.request_redraw = true;
            active.v2_step_state = None;
        }
    }

    true
}

pub(super) fn handle_pointer_cancel_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::PointerCancel {
        window: target_window,
        pointer_kind,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;
    output.request_redraw = true;

    if let Some(session) = active.pointer_session.clone() {
        if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
            if target_window != window {
                if target_window == session.window {
                    *handoff_to = Some(target_window);
                    output
                        .effects
                        .push(Effect::RequestAnimationFrame(target_window));
                    output.request_redraw = true;
                } else {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-pointer_cancel-window-mismatch"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("pointer_session_cross_window_unsupported".to_string());
                    output.request_redraw = true;
                }
            }
        } else if target_window.is_some() {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-pointer_cancel-window-not-found"
            ));
            *stop_script = true;
            *failure_reason = Some("window_target_unresolved".to_string());
            output.request_redraw = true;
        } else if session.window != window {
            // The script migrated away from the window that owns the pointer session.
            *handoff_to = Some(session.window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(session.window));
            output.request_redraw = true;
        }

        if *stop_script {
            active.v2_step_state = None;
        } else if handoff_to.is_some() {
            // Window-targeted: migrate to the target window before canceling the session.
        } else if let Some(want) = pointer_kind
            && pointer_type_from_kind(Some(want)) != session.pointer_type
        {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-pointer_cancel-pointer-kind-mismatch"
            ));
            *stop_script = true;
            *failure_reason = Some("pointer_session_pointer_kind_mismatch".to_string());
            output.request_redraw = true;
            active.v2_step_state = None;
        } else {
            let pointer_id = PointerId(0);
            let pointer_type = session.pointer_type;
            let buttons = MouseButtons {
                left: matches!(session.button, UiMouseButtonV1::Left),
                right: matches!(session.button, UiMouseButtonV1::Right),
                middle: matches!(session.button, UiMouseButtonV1::Middle),
            };

            output
                .events
                .push(Event::PointerCancel(fret_core::PointerCancelEvent {
                    pointer_id,
                    position: Some(session.position),
                    buttons,
                    modifiers: session.modifiers,
                    pointer_type,
                    reason: fret_core::PointerCancelReason::LeftWindow,
                }));
            output
                .events
                .push(Event::InternalDrag(fret_core::InternalDragEvent {
                    pointer_id,
                    position: session.position,
                    kind: fret_core::InternalDragKind::Cancel,
                    modifiers: session.modifiers,
                }));
            let _ = write_cursor_override_window_client_logical(
                &svc.cfg.out_dir,
                window,
                session.position.x.0,
                session.position.y.0,
            );
            let _ = write_mouse_buttons_override_all_windows_v1(
                &svc.cfg.out_dir,
                match session.button {
                    UiMouseButtonV1::Left => Some(false),
                    _ => None,
                },
                match session.button {
                    UiMouseButtonV1::Right => Some(false),
                    _ => None,
                },
                match session.button {
                    UiMouseButtonV1::Middle => Some(false),
                    _ => None,
                },
            );
            active.pending_cancel_cross_window_drag =
                Some(PendingCancelCrossWindowDrag::new(pointer_id));
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "diag.pending_cancel_drag".to_string(),
                    step_index: Some(step_index.min(u32::MAX as usize) as u32),
                    note: Some(format!("pointer_id={}", pointer_id.0)),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );

            active.pointer_session = None;
            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-pointer_cancel"));
            }
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-pointer_cancel-no-session"
        ));
        *stop_script = true;
        *failure_reason = Some("pointer_session_missing".to_string());
        output.request_redraw = true;
        active.v2_step_state = None;
    }

    true
}
