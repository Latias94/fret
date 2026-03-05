use super::*;
use fret_runtime::WindowKeyContextStackService;
use std::collections::HashMap;

use super::event_chain::pointer_cancel_event_for_capture_switch;

impl<H: UiHost> UiTree<H> {
    #[stacksafe::stacksafe]
    pub fn dispatch_event(&mut self, app: &mut H, services: &mut dyn UiServices, event: &Event) {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };

        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let window = self.window;
        let frame_id = app.frame_id();
        let kind: &'static str = match event {
            Event::Pointer(_) | Event::PointerCancel(_) => "pointer",
            Event::Timer { .. } => "timer",
            _ => "other",
        };

        let ((), elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || {
                tracing::trace_span!(
                    "fret.ui.dispatch.event",
                    window = ?window,
                    frame_id = frame_id.0,
                    kind,
                )
            },
            || self.dispatch_event_inner(app, services, event, base_root),
        );
        if self.debug_enabled {
            self.debug_stats.dispatch_events = self.debug_stats.dispatch_events.saturating_add(1);
            if let Some(elapsed) = elapsed {
                self.debug_stats.dispatch_time += elapsed;
                match kind {
                    "pointer" => {
                        self.debug_stats.dispatch_pointer_events =
                            self.debug_stats.dispatch_pointer_events.saturating_add(1);
                        self.debug_stats.dispatch_pointer_event_time += elapsed;
                    }
                    "timer" => {
                        self.debug_stats.dispatch_timer_events =
                            self.debug_stats.dispatch_timer_events.saturating_add(1);
                        self.debug_stats.dispatch_timer_event_time += elapsed;
                    }
                    _ => {
                        self.debug_stats.dispatch_other_events =
                            self.debug_stats.dispatch_other_events.saturating_add(1);
                        self.debug_stats.dispatch_other_event_time += elapsed;
                    }
                }
            }
        }
    }

    #[stacksafe::stacksafe]
    fn dispatch_event_inner(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        event: &Event,
        base_root: NodeId,
    ) {
        self.begin_debug_frame_if_needed(app.frame_id());
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(debug_assertions)]
        let debug_focus_scope = std::env::var_os("FRET_TEST_DEBUG_FOCUS_SCOPE").is_some();
        #[cfg(debug_assertions)]
        let mut debug_focus_last = self.focus;
        #[cfg(debug_assertions)]
        let mut debug_focus_note = |label: &str, focus: Option<NodeId>| {
            if !debug_focus_scope {
                return;
            }
            if focus != debug_focus_last {
                eprintln!(
                    "debug: dispatch {}: focus {:?} -> {:?}",
                    label, debug_focus_last, focus
                );
                debug_focus_last = focus;
            }
        };

        if let Some(window) = self.window {
            let frame_id = app.frame_id();
            let now_monotonic = app
                .global::<fret_core::WindowFrameClockService>()
                .and_then(|svc| svc.snapshot(window))
                .map(|s| s.now_monotonic);

            match event {
                Event::ClipboardText { token, .. } => {
                    app.with_global_mut_untracked(
                        fret_runtime::WindowClipboardDiagnosticsStore::default,
                        |svc, _host| {
                            svc.record_read_ok(window, frame_id, *token);
                        },
                    );
                }
                Event::ClipboardTextUnavailable { token, message } => {
                    app.with_global_mut_untracked(
                        fret_runtime::WindowClipboardDiagnosticsStore::default,
                        |svc, _host| {
                            svc.record_read_unavailable(window, frame_id, *token, message.clone());
                        },
                    );
                }
                _ => {}
            }

            let update_pointer = |app: &mut H,
                                  pointer_id: fret_core::PointerId,
                                  position: Point| {
                app.with_global_mut_untracked(
                    crate::pointer_motion::WindowPointerMotionService::default,
                    |svc, _host| {
                        svc.update_position(window, pointer_id, position, frame_id, now_monotonic);
                    },
                );
            };

            match event {
                Event::Pointer(pe) => match pe {
                    PointerEvent::Move {
                        pointer_id,
                        position,
                        ..
                    }
                    | PointerEvent::Down {
                        pointer_id,
                        position,
                        ..
                    }
                    | PointerEvent::Up {
                        pointer_id,
                        position,
                        ..
                    }
                    | PointerEvent::Wheel {
                        pointer_id,
                        position,
                        ..
                    }
                    | PointerEvent::PinchGesture {
                        pointer_id,
                        position,
                        ..
                    } => {
                        update_pointer(app, *pointer_id, *position);
                    }
                },
                Event::PointerCancel(e) => {
                    if let Some(position) = e.position {
                        update_pointer(app, e.pointer_id, position);
                    }
                }
                _ => {}
            }
        }

        // Keep wheel routing and hover detection in sync with out-of-band scroll handle mutations
        // (e.g. forwarded wheel handlers) by applying scroll-handle-driven invalidations before
        // hit-testing.
        if matches!(event, Event::Pointer(_)) {
            let (_, elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_enabled,
                || tracing::trace_span!("fret.ui.dispatch.scroll_handle_invalidation"),
                || {
                    self.invalidate_scroll_handle_bindings_for_changed_handles(
                        app,
                        crate::layout_pass::LayoutPassKind::Final,
                        /* consume_deferred_scroll_to_item */ false,
                        /* commit_scroll_handle_baselines */ false,
                    );
                },
            );
            if let Some(elapsed) = elapsed {
                self.debug_stats.dispatch_scroll_handle_invalidation_time += elapsed;
            }
        }

        let is_wheel = matches!(event, Event::Pointer(PointerEvent::Wheel { .. }));

        let ((active_layers, barrier_root), active_layers_elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.ui.dispatch.active_layers"),
            || {
                let (active_layers, barrier_root) = self.active_input_layers();
                (active_layers, barrier_root)
            },
        );
        #[cfg(debug_assertions)]
        debug_focus_note("after active layers", self.focus);
        if let Some(active_layers_elapsed) = active_layers_elapsed {
            self.debug_stats.dispatch_active_layers_time += active_layers_elapsed;
        }

        let dispatch_cx = self.build_dispatch_cx(app.frame_id(), active_layers, barrier_root);
        let active_layers: &[NodeId] = dispatch_cx.active_input_roots.as_slice();
        let barrier_root = dispatch_cx.input_barrier_root;

        let hit_test_layer_roots: &[NodeId] = active_layers;
        let pointer_chain_snapshot: &UiDispatchSnapshot = &dispatch_cx.input_snapshot;

        let node_in_active_layers = |node: NodeId| dispatch_cx.node_in_active_input_layers(node);

        // Focus barriers (trap scopes / modal focus arbitration) must not rely on retained parent
        // pointers for correctness under retained/view-cache reuse. Enforce focus-barrier scope
        // using a snapshot forest built from child edges.
        if dispatch_cx.focus_barrier_root.is_some() && self.focus.is_some() {
            if self
                .focus
                .is_some_and(|n| !dispatch_cx.node_in_active_focus_layers(n))
            {
                self.set_focus_unchecked(None, "dispatch/window: focus barrier scope");
            }
        }

        let to_remove: Vec<fret_core::PointerId> = self
            .captured
            .iter()
            .filter_map(|(p, n)| (!node_in_active_layers(*n)).then_some(*p))
            .collect();
        for p in to_remove {
            self.captured.remove(&p);
        }
        if self.focus.is_some_and(|n| !self.node_exists(n)) {
            self.set_focus_unchecked(None, "dispatch/window: missing focus node");
        }
        #[cfg(debug_assertions)]
        debug_focus_note("after pre-dispatch cleanup", self.focus);

        let focus_is_text_input = self.focus_is_text_input(app);
        self.update_ime_composing_for_event(focus_is_text_input, event);
        self.set_ime_allowed(app, focus_is_text_input);

        let (input_ctx, input_ctx_elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.ui.dispatch.input_context"),
            || {
                let caps = app
                    .global::<PlatformCapabilities>()
                    .cloned()
                    .unwrap_or_default();
                let mut input_ctx = InputContext {
                    platform: Platform::current(),
                    caps,
                    ui_has_modal: barrier_root.is_some(),
                    window_arbitration: None,
                    focus_is_text_input,
                    text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
                    edit_can_undo: true,
                    edit_can_redo: true,
                    router_can_back: false,
                    router_can_forward: false,
                    dispatch_phase: InputDispatchPhase::Bubble,
                };
                if let Some(window) = self.window {
                    let is_pointer_move =
                        matches!(event, Event::Pointer(fret_core::PointerEvent::Move { .. }));
                    if let Some(mode) = app
                        .global::<fret_runtime::WindowTextBoundaryModeService>()
                        .and_then(|svc| svc.mode(window))
                    {
                        input_ctx.text_boundary_mode = mode;
                    }
                    if let Some(mode) = self.focus_text_boundary_mode_override() {
                        input_ctx.text_boundary_mode = mode;
                    }
                    if let Some(availability) = app
                        .global::<fret_runtime::WindowCommandAvailabilityService>()
                        .and_then(|svc| svc.snapshot(window))
                        .copied()
                    {
                        input_ctx.edit_can_undo = availability.edit_can_undo;
                        input_ctx.edit_can_redo = availability.edit_can_redo;
                        input_ctx.router_can_back = availability.router_can_back;
                        input_ctx.router_can_forward = availability.router_can_forward;
                    }

                    let window_arbitration = self.window_input_arbitration_snapshot();
                    input_ctx.window_arbitration = Some(window_arbitration);

                    if is_pointer_move {
                        // Keep pointer-move dispatch cheap: publish the snapshot without
                        // participating in global-change propagation.
                        app.with_global_mut_untracked(
                            fret_runtime::WindowInputContextService::default,
                            |svc, _app| {
                                svc.set_snapshot(window, input_ctx.clone());
                            },
                        );
                    } else {
                        let needs_update = app
                            .global::<fret_runtime::WindowInputContextService>()
                            .and_then(|svc| svc.snapshot(window))
                            .is_none_or(|prev| prev != &input_ctx);
                        if needs_update {
                            app.with_global_mut(
                                fret_runtime::WindowInputContextService::default,
                                |svc, _app| {
                                    svc.set_snapshot(window, input_ctx.clone());
                                },
                            );
                        }

                        let next_key_contexts = self.shortcut_key_context_stack(app, barrier_root);
                        let needs_key_contexts_update = app
                            .global::<WindowKeyContextStackService>()
                            .and_then(|svc| svc.snapshot(window))
                            .is_none_or(|prev| prev != next_key_contexts.as_slice());
                        if needs_key_contexts_update {
                            app.with_global_mut(
                                WindowKeyContextStackService::default,
                                |svc, _app| {
                                    svc.set_snapshot(window, next_key_contexts);
                                },
                            );
                        }
                    }
                }
                input_ctx
            },
        );
        if let Some(input_ctx_elapsed) = input_ctx_elapsed {
            self.debug_stats.dispatch_input_context_time += input_ctx_elapsed;
        }

        let mut invalidation_visited = HashMap::<NodeId, u8>::new();
        let mut needs_redraw = false;

        // ADR 0012: when a text input is focused, reserve common IME/navigation keys for the
        // text/IME path first, and only fall back to shortcut matching if the widget doesn't
        // consume the event.
        let defer_keydown_shortcuts_until_after_dispatch =
            self.pending_shortcut.keystrokes.is_empty()
                && !self.replaying_pending_shortcut
                && self.focus.is_some()
                && match event {
                    Event::KeyDown { key, modifiers, .. } => {
                        Self::should_defer_keydown_shortcut_matching_to_text_input(
                            *key,
                            *modifiers,
                            focus_is_text_input,
                        )
                    }
                    _ => false,
                };

        if let Some(window) = self.window {
            let pointer_type = match event {
                Event::Pointer(fret_core::PointerEvent::Move { pointer_type, .. }) => {
                    Some(*pointer_type)
                }
                Event::Pointer(fret_core::PointerEvent::Down { pointer_type, .. }) => {
                    Some(*pointer_type)
                }
                Event::Pointer(fret_core::PointerEvent::Up { pointer_type, .. }) => {
                    Some(*pointer_type)
                }
                Event::PointerCancel(fret_core::PointerCancelEvent { pointer_type, .. }) => {
                    Some(*pointer_type)
                }
                _ => None,
            };
            if let Some(pointer_type) = pointer_type {
                app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _app| {
                    rt.set_window_primary_pointer_type(window, pointer_type);
                });
            }

            let changed = crate::focus_visible::update_for_event(app, window, event);
            if changed {
                if let Some(focus) = self.focus {
                    self.mark_invalidation_dedup_with_detail(
                        focus,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::FocusVisiblePolicy,
                    );
                } else {
                    self.mark_invalidation_dedup_with_detail(
                        base_root,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::FocusVisiblePolicy,
                    );
                }
                self.request_redraw_coalesced(app);
            }

            let changed = crate::input_modality::update_for_event(app, window, event);
            if changed {
                if let Some(focus) = self.focus {
                    self.mark_invalidation_dedup_with_detail(
                        focus,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::InputModalityPolicy,
                    );
                } else {
                    self.mark_invalidation_dedup_with_detail(
                        base_root,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::InputModalityPolicy,
                    );
                }
                self.request_redraw_coalesced(app);
            }
        }

        if !self.replaying_pending_shortcut
            && !self.pending_shortcut.keystrokes.is_empty()
            && ((self.pending_shortcut.focus.is_some()
                && self.pending_shortcut.focus != self.focus)
                || self.pending_shortcut.barrier_root != barrier_root)
        {
            self.clear_pending_shortcut(app);
        }

        if let Event::Timer { token } = event
            && !self.replaying_pending_shortcut
            && !self.pending_shortcut.keystrokes.is_empty()
            && self.pending_shortcut.timer == Some(*token)
        {
            let pending = std::mem::take(&mut self.pending_shortcut);
            self.sync_pending_shortcut_overlay_state(app, None);
            if let Some(command) = pending.fallback {
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
            } else {
                self.replay_captured_keystrokes(app, services, &input_ctx, pending.keystrokes);
            }
            return;
        }
        if let Event::Timer { token } = event {
            let window = self.window;
            let frame_id = app.frame_id();
            let token = *token;
            // Timer events should be dispatched to visible layers even when they are not currently
            // hit-testable (e.g. during open/close transitions). The regular dispatch context is
            // built from active *input* layers, which can exclude visible timer listeners and
            // cause snapshot membership assertions during bubble dispatch.
            let timer_dispatch_cx = {
                let timer_layer_roots: Vec<NodeId> = self
                    .visible_layers_in_paint_order()
                    .filter_map(|layer_id| self.layers.get(layer_id).map(|l| l.root))
                    .collect();
                self.build_dispatch_cx(frame_id, timer_layer_roots, None)
            };
            let mut timer_target: Option<NodeId> = None;
            let mut broadcast_rebuild_visible_layers_elapsed: Option<Duration> = None;
            let mut broadcast_loop_elapsed: Option<Duration> = None;
            let mut broadcast_layers_visited: u32 = 0;
            let mut stopped = false;
            let mut broadcast_attempted = false;

            let ((), timer_elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_enabled,
                || {
                    tracing::trace_span!(
                        "fret.ui.dispatch.timer",
                        window = ?window,
                        frame_id = frame_id.0,
                        token = token.0,
                    )
                },
                || {
                    if let Some(window) = window {
                        timer_target = crate::elements::timer_target_node(app, window, token);
                    }
                    if let Some(node) = timer_target {
                        let (targeted_stopped, _) = fret_perf::measure_span(
                            self.debug_enabled,
                            trace_enabled,
                            || {
                                tracing::trace_span!(
                                    "fret.ui.dispatch.timer.targeted",
                                    window = ?window,
                                    frame_id = frame_id.0,
                                    token = token.0,
                                    node = ?node,
                                )
                            },
                            || {
                                self.dispatch_event_to_node_chain(
                                    app,
                                    services,
                                    &timer_dispatch_cx,
                                    &input_ctx,
                                    node,
                                    event,
                                    &mut needs_redraw,
                                    &mut invalidation_visited,
                                )
                            },
                        );
                        stopped = targeted_stopped;
                    }

                    if !stopped {
                        broadcast_attempted = true;
                        let (layers, rebuild_elapsed) = fret_perf::measure_span(
                            self.debug_enabled,
                            trace_enabled,
                            || {
                                tracing::trace_span!(
                                    "fret.ui.dispatch.timer.broadcast.rebuild_visible_layers",
                                    window = ?window,
                                    frame_id = frame_id.0,
                                    token = token.0,
                                )
                            },
                            || {
                                self.visible_layers_in_paint_order()
                                    .collect::<Vec<UiLayerId>>()
                            },
                        );
                        broadcast_rebuild_visible_layers_elapsed = rebuild_elapsed;

                        let (broadcast_stopped, loop_elapsed) = fret_perf::measure_span(
                            self.debug_enabled,
                            trace_enabled,
                            || {
                                tracing::trace_span!(
                                    "fret.ui.dispatch.timer.broadcast.loop",
                                    window = ?window,
                                    frame_id = frame_id.0,
                                    token = token.0,
                                )
                            },
                            || {
                                for layer_id in layers.into_iter().rev() {
                                    broadcast_layers_visited =
                                        broadcast_layers_visited.saturating_add(1);
                                    let Some(layer) = self.layers.get(layer_id) else {
                                        continue;
                                    };
                                    if !layer.wants_timer_events || !layer.visible {
                                        continue;
                                    }
                                    let stopped = self.dispatch_event_to_node_chain(
                                        app,
                                        services,
                                        &timer_dispatch_cx,
                                        &input_ctx,
                                        layer.root,
                                        event,
                                        &mut needs_redraw,
                                        &mut invalidation_visited,
                                    );
                                    if stopped {
                                        return true;
                                    }
                                }
                                false
                            },
                        );
                        broadcast_loop_elapsed = loop_elapsed;
                        stopped = broadcast_stopped;
                    }
                },
            );

            if self.debug_enabled {
                let is_targeted = timer_target.is_some();
                if is_targeted {
                    self.debug_stats.dispatch_timer_targeted_events = self
                        .debug_stats
                        .dispatch_timer_targeted_events
                        .saturating_add(1);
                } else {
                    self.debug_stats.dispatch_timer_broadcast_events = self
                        .debug_stats
                        .dispatch_timer_broadcast_events
                        .saturating_add(1);
                }

                if let Some(timer_elapsed) = timer_elapsed {
                    if is_targeted {
                        self.debug_stats.dispatch_timer_targeted_time += timer_elapsed;
                    } else {
                        self.debug_stats.dispatch_timer_broadcast_time += timer_elapsed;
                    }

                    if timer_elapsed > self.debug_stats.dispatch_timer_slowest_event_time {
                        self.debug_stats.dispatch_timer_slowest_event_time = timer_elapsed;
                        self.debug_stats.dispatch_timer_slowest_token = Some(token);
                        self.debug_stats.dispatch_timer_slowest_was_broadcast = !is_targeted;
                    }
                }

                if broadcast_attempted && timer_target.is_none() {
                    self.debug_stats.dispatch_timer_broadcast_layers_visited = self
                        .debug_stats
                        .dispatch_timer_broadcast_layers_visited
                        .saturating_add(broadcast_layers_visited);

                    if let Some(rebuild_elapsed) = broadcast_rebuild_visible_layers_elapsed {
                        self.debug_stats
                            .dispatch_timer_broadcast_rebuild_visible_layers_time +=
                            rebuild_elapsed;
                    }
                    if let Some(loop_elapsed) = broadcast_loop_elapsed {
                        self.debug_stats.dispatch_timer_broadcast_loop_time += loop_elapsed;
                    }
                }
            }

            if stopped {
                if needs_redraw {
                    self.request_redraw_coalesced(app);
                }
                return;
            }
        }

        if let Event::TextInput(text) = event {
            if !self.replaying_pending_shortcut
                && self.pending_shortcut.capture_next_text_input_key.is_some()
            {
                self.pending_shortcut.capture_next_text_input_key = None;
                if let Some(last) = self.pending_shortcut.keystrokes.last_mut() {
                    last.text = Some(text.clone());
                }
                self.suppress_text_input_until_key_up = None;
                return;
            }

            if self.suppress_text_input_until_key_up.is_some() {
                self.suppress_text_input_until_key_up = None;
                return;
            }
        }

        if let Event::KeyUp { key, .. } = event {
            if self.suppress_text_input_until_key_up == Some(*key) {
                self.suppress_text_input_until_key_up = None;
            }
            if self.pending_shortcut.capture_next_text_input_key == Some(*key) {
                self.pending_shortcut.capture_next_text_input_key = None;
            }
        }

        if let Some(window) = self.window
            && self.handle_alt_menu_bar_activation(app, window, focus_is_text_input, event)
        {
            return;
        }

        let mut cursor_choice: Option<fret_core::CursorIcon> = None;
        let mut cursor_choice_from_query = false;
        let mut stop_propagation_requested = false;
        let mut stop_propagation_requested_by: Option<NodeId> = None;
        let mut pointer_down_outside = PointerDownOutsideOutcome::default();
        let mut suppress_touch_up_outside_dispatch = false;
        let mut suppress_pointer_dispatch = false;
        let is_scroll_like = Self::event_is_scroll_like(event);
        let mut wheel_stop_node: Option<NodeId> = None;
        let mut synth_pointer_move_prev_target: Option<NodeId> = None;
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let event_window_position = event_position(event);
        let event_window_wheel_delta = match event {
            Event::Pointer(PointerEvent::Wheel { delta, .. }) => Some(*delta),
            _ => None,
        };
        let mut focus_requested = false;
        let mut defer_escape_overlay_dismiss = false;

        if let Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            repeat: false,
            ..
        } = event
            && let Some(window) = self.window
            && {
                let dock_drag_affects_window = app.any_drag_session(|d| {
                    d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        && (d.source_window == window || d.current_window == window)
                });
                if dock_drag_affects_window {
                    // ADR 0072: Escape cancels the active dock drag session, and must not be
                    // routed to overlays while the drag is in progress.
                    let canceled = app.cancel_drag_sessions(|d| {
                        d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                            && (d.source_window == window || d.current_window == window)
                    });
                    for pointer_id in canceled {
                        self.captured.remove(&pointer_id);
                    }
                    true
                } else {
                    defer_escape_overlay_dismiss = true;
                    false
                }
            }
        {
            self.request_redraw_coalesced(app);
            return;
        }

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && !defer_keydown_shortcuts_until_after_dispatch
            && self.handle_keydown_shortcuts(
                app,
                services,
                KeydownShortcutParams {
                    input_ctx: &input_ctx,
                    barrier_root,
                    focus_is_text_input,
                    #[cfg(feature = "diagnostics")]
                    phase: fret_runtime::ShortcutRoutingPhase::PreDispatch,
                    #[cfg(feature = "diagnostics")]
                    deferred: false,
                    key: *key,
                    modifiers: *modifiers,
                    repeat: *repeat,
                },
            )
        {
            return;
        }

        let default_root = barrier_root.unwrap_or(base_root);

        // Pointer capture only affects pointer events. Drag-and-drop style events
        // (external/internal) must continue to follow the cursor for correct cross-window UX.
        let event_pointer_id_for_capture: Option<fret_core::PointerId> = match event {
            Event::Pointer(PointerEvent::Move { pointer_id, .. })
            | Event::Pointer(PointerEvent::Down { pointer_id, .. })
            | Event::Pointer(PointerEvent::Up { pointer_id, .. })
            | Event::Pointer(PointerEvent::Wheel { pointer_id, .. })
            | Event::Pointer(PointerEvent::PinchGesture { pointer_id, .. }) => Some(*pointer_id),
            Event::PointerCancel(e) => Some(e.pointer_id),
            _ => None,
        };

        let captured = event_pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
        if let Event::Pointer(PointerEvent::Move {
            pointer_id,
            position,
            pointer_type: fret_core::PointerType::Touch,
            ..
        }) = event
        {
            self.update_touch_pointer_down_outside_move(*pointer_id, *position);
        }
        let (dock_drag_affects_window, dock_drag_capture_anchor) = self
            .window
            .map(|window| {
                let affects = app.any_drag_session(|d| {
                    d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        && (d.source_window == window || d.current_window == window)
                });
                let anchor =
                    crate::internal_drag::route(&*app, window, fret_runtime::DRAG_KIND_DOCK_PANEL);
                (affects, anchor)
            })
            .unwrap_or((false, None));

        // Internal drag overrides may need to route events to a stable "anchor" node, even if
        // hit-testing fails or the cursor is over an unrelated widget (e.g. docking tear-off).
        let internal_drag_target = (|| {
            let Event::InternalDrag(e) = event else {
                return None;
            };
            let window = self.window?;
            let drag = app.drag(e.pointer_id)?;
            if !drag.cross_window_hover {
                return None;
            }
            let target = crate::internal_drag::route(app, window, drag.kind)?;
            // Cross-window internal drags are runner-routed and rely on a stable, per-window
            // anchor node. Do not gate the route target on the current "active layer" set:
            // modal barriers/overlays can temporarily deactivate the base layer while a dock drag
            // is in flight (ADR 0072), but docking still needs `InternalDrag` hover/drop events.
            //
            // Only require that the node still exists in the tree (mechanism-only contract).
            self.nodes.get(target).is_some().then_some(target)
        })();
        if std::env::var_os("FRET_INTERNAL_DRAG_ROUTE_TRACE").is_some_and(|v| !v.is_empty())
            && let Some(window) = self.window
            && let Event::InternalDrag(e) = event
            && matches!(e.kind, fret_core::InternalDragKind::Drop)
        {
            let (drag_kind, cross_window_hover, route, route_in_active_layer) = if let Some(drag) =
                app.drag(e.pointer_id)
            {
                let route = crate::internal_drag::route(app, window, drag.kind);
                let route_in_active_layer = route.is_some_and(|node| node_in_active_layers(node));
                (
                    Some(drag.kind),
                    drag.cross_window_hover,
                    route,
                    route_in_active_layer,
                )
            } else {
                (None, false, None, false)
            };
            tracing::info!(
                window = ?window,
                pointer_id = ?e.pointer_id,
                kind = ?e.kind,
                position = ?e.position,
                modifiers = ?e.modifiers,
                drag_kind = ?drag_kind,
                cross_window_hover = cross_window_hover,
                route = ?route,
                route_in_active_layer = route_in_active_layer,
                internal_drag_target = ?internal_drag_target,
                last_internal_drag_target = ?self.last_internal_drag_target,
                "internal drag route trace"
            );
        }

        if let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
            && let Some(pos) = event_position(event)
        {
            // Hit-testing is performance-sensitive (especially for pointer move), but must remain
            // correct across discrete interactions like clicks where the pointer position can jump
            // substantially between events.
            //
            // For now, only allow cached hit-test reuse for pointer-move events; other pointer
            // events clear the cache and rebuild it from a full hit-test pass.
            let hit = if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            } else {
                self.hit_test_path_cache = None;
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            };

            if let Event::Pointer(PointerEvent::Up {
                pointer_id,
                pointer_type: fret_core::PointerType::Touch,
                ..
            }) = event
                && captured.is_none()
            {
                if dock_drag_affects_window {
                    self.touch_pointer_down_outside_candidates
                        .remove(pointer_id);
                } else if let Some(candidate) = self
                    .touch_pointer_down_outside_candidates
                    .remove(pointer_id)
                    && let Some(layer) = self.layers.get(candidate.layer_id)
                {
                    let foreign_capture_active = self.captured.iter().any(|(pid, node)| {
                        *pid != *pointer_id
                            && self
                                .node_layer(*node)
                                .is_some_and(|layer_id| layer_id != candidate.layer_id)
                    });

                    if !foreign_capture_active && !candidate.moved {
                        let active_pointer_down_outside_layers =
                            self.active_pointer_down_outside_layer_roots(barrier_root);
                        let snapshot = self.build_dispatch_snapshot_for_layer_roots(
                            app.frame_id(),
                            active_pointer_down_outside_layers.as_slice(),
                            barrier_root,
                        );

                        let hit_is_inside_layer = hit.is_some_and(|hit| {
                            if snapshot.pre.get(layer.root).is_some()
                                && snapshot.pre.get(hit).is_some()
                            {
                                snapshot.is_descendant(layer.root, hit)
                            } else {
                                self.is_reachable_from_root_via_children(layer.root, hit)
                            }
                        });
                        let hit_is_inside_branch = hit.is_some_and(|hit| {
                            layer
                                .pointer_down_outside_branches
                                .iter()
                                .copied()
                                .any(|branch| {
                                    if snapshot.pre.get(branch).is_some()
                                        && snapshot.pre.get(hit).is_some()
                                    {
                                        snapshot.is_descendant(branch, hit)
                                    } else {
                                        self.is_reachable_from_root_via_children(branch, hit)
                                    }
                                })
                        });

                        if !hit_is_inside_layer && !hit_is_inside_branch {
                            let (window, root_element, tick_id) = if let Some(window) = self.window
                                && let Some(root_element) =
                                    self.nodes.get(candidate.root).and_then(|n| n.element)
                            {
                                let tick_id = app.tick_id();
                                crate::elements::with_element_state(
                                    app,
                                    window,
                                    root_element,
                                    crate::action::DismissibleLastDismissRequest::default,
                                    |st| {
                                        st.tick_id = tick_id;
                                        st.reason = None;
                                        st.default_prevented = false;
                                    },
                                );
                                (Some(window), Some(root_element), Some(tick_id))
                            } else {
                                (None, None, None)
                            };
                            self.dispatch_event_to_node_chain_observer(
                                app,
                                services,
                                &input_ctx,
                                candidate.root,
                                &candidate.down_event,
                                Some(&snapshot),
                                &mut invalidation_visited,
                            );
                            let mut clear_focus = true;
                            if let (Some(window), Some(root_element), Some(tick_id)) =
                                (window, root_element, tick_id)
                            {
                                let prevented = crate::elements::with_element_state(
                                    app,
                                    window,
                                    root_element,
                                    crate::action::DismissibleLastDismissRequest::default,
                                    |st| {
                                        st.tick_id == tick_id
                                            && matches!(
                                                st.reason,
                                                Some(
                                                    crate::action::DismissReason::OutsidePress { .. }
                                                )
                                            )
                                            && st.default_prevented
                                    },
                                );
                                if prevented {
                                    clear_focus = false;
                                }
                            }
                            if clear_focus {
                                self.set_focus(None);
                            }
                            needs_redraw = true;
                            suppress_touch_up_outside_dispatch = candidate.consume;
                        }
                    }
                }
            }

            // Pointer occlusion is a window-level layer substrate mechanism (policy-owned).
            //
            // When active, the runtime must:
            // - suppress hover state for underlay layers (even when scroll is allowed),
            // - optionally suppress hit-tested pointer dispatch for underlay layers depending on
            //   the occlusion mode.
            let mut hit_for_hover = hit;
            let mut hit_for_hover_region = hit;
            let mut hit_for_raw_below_barrier: Option<NodeId> = None;
            if captured.is_none()
                && let Some((occlusion_layer, occlusion)) =
                    self.topmost_pointer_occlusion_layer(barrier_root)
                && occlusion != PointerOcclusion::None
            {
                let occlusion_z = self
                    .layer_order
                    .iter()
                    .position(|id| *id == occlusion_layer);
                let hit_layer_z = hit
                    .and_then(|hit| self.node_layer(hit))
                    .and_then(|layer| self.layer_order.iter().position(|id| *id == layer));

                let hit_is_below_occlusion = match (occlusion_z, hit_layer_z, hit) {
                    (Some(oz), Some(hz), Some(_)) => hz < oz,
                    (Some(_), None, Some(_)) => true,
                    (Some(_), _, None) => true,
                    _ => false,
                };

                if hit_is_below_occlusion {
                    hit_for_raw_below_barrier = hit;
                    // Match GPUI-style "occluded hover": underlay hover/pressable detection is
                    // disabled while occlusion is active, even when scroll is still allowed.
                    hit_for_hover = None;
                    hit_for_hover_region = None;

                    let blocks_pointer_dispatch = match occlusion {
                        PointerOcclusion::None => false,
                        PointerOcclusion::BlockMouse => true,
                        PointerOcclusion::BlockMouseExceptScroll => !is_scroll_like,
                    };
                    if blocks_pointer_dispatch {
                        suppress_pointer_dispatch = true;
                    }
                }
            }

            if input_ctx.caps.ui.cursor_icons
                && cursor_choice.is_none()
                && matches!(event, Event::Pointer(PointerEvent::Move { .. }))
            {
                let (_, elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.dispatch.cursor_query"),
                    || {
                        if let Some(start) = captured.or(hit_for_hover) {
                            cursor_choice = self.cursor_icon_query_for_pointer_hit(
                                start,
                                &input_ctx,
                                event,
                                Some(pointer_chain_snapshot),
                            );
                            cursor_choice_from_query = cursor_choice.is_some();
                        }
                    },
                );
                if let Some(elapsed) = elapsed {
                    self.debug_stats.dispatch_cursor_query_time += elapsed;
                }
            }

            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) && captured.is_none() {
                if dock_drag_affects_window {
                    // ADR 0072: while a dock drag session is active, outside-press dismissal must
                    // not trigger. The drag owns input arbitration for the window.
                    //
                    // This is intentionally window-global (not pointer-local): a dock drag session
                    // is exclusive for the window, and we do not want secondary pointers to dismiss
                    // overlays or change focus while the drag is in progress.
                    //
                    // Note: overlay policy is expected to close/suspend non-modal overlays when a
                    // dock drag starts; this suppression makes the routing rule durable even if a
                    // layer remains mounted for a close transition.
                    pointer_down_outside = PointerDownOutsideOutcome::default();
                } else {
                    let active_pointer_down_outside_layers =
                        self.active_pointer_down_outside_layer_roots(barrier_root);
                    pointer_down_outside = self.dispatch_pointer_down_outside(
                        app,
                        services,
                        PointerDownOutsideParams {
                            input_ctx: &input_ctx,
                            active_layer_roots: &active_pointer_down_outside_layers,
                            barrier_root,
                            base_root,
                            hit,
                            event,
                        },
                        &mut invalidation_visited,
                    );
                    if pointer_down_outside.dispatched {
                        needs_redraw = true;
                    }
                }
            }

            let hover_capable = match event {
                Event::Pointer(PointerEvent::Move { pointer_type, .. })
                | Event::Pointer(PointerEvent::Down { pointer_type, .. })
                | Event::Pointer(PointerEvent::Up { pointer_type, .. })
                | Event::Pointer(PointerEvent::Wheel { pointer_type, .. })
                | Event::Pointer(PointerEvent::PinchGesture { pointer_type, .. }) => {
                    pointer_type_supports_hover(*pointer_type)
                }
                _ => false,
            };

            if hover_capable {
                let position = event_position(event);
                self.update_hover_state_from_hit(
                    app,
                    window,
                    barrier_root,
                    position,
                    hit_for_hover,
                    hit_for_hover_region,
                    hit_for_raw_below_barrier,
                    Some(pointer_chain_snapshot),
                    &mut invalidation_visited,
                    &mut needs_redraw,
                );
            }
        }

        let mut pointer_hit: Option<NodeId> = None;
        let target = if let Some(captured) = captured {
            Some(captured)
        } else if let Some(target) = internal_drag_target {
            Some(target)
        } else if let Some(pos) = event_position(event) {
            // See the cached hit-test reuse note above.
            let hit = if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            } else {
                self.hit_test_path_cache = None;
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            };

            let hit = if matches!(event, Event::InternalDrag(_)) {
                (|| {
                    let window = self.window?;
                    crate::declarative::with_window_frame(app, window, |window_frame| {
                        let window_frame = window_frame?;
                        let mut node = hit?;
                        loop {
                            if let Some(record) = window_frame.instances.get(node)
                                && matches!(
                                    record.instance,
                                    crate::declarative::ElementInstance::InternalDragRegion(p)
                                        if p.enabled
                                )
                            {
                                return Some(node);
                            }
                            node = pointer_chain_snapshot.parent.get(node).copied().flatten()?;
                        }
                    })
                })()
                .or(hit)
            } else {
                hit
            };
            pointer_hit = hit;

            if let Event::Pointer(PointerEvent::Move {
                buttons,
                pointer_id,
                ..
            }) = event
                && !buttons.left
                && !buttons.right
                && !buttons.middle
            {
                // When a modal barrier becomes active, the previous pointer-move hit may belong to
                // an underlay layer that is now inactive. Do not synthesize hover-move events into
                // the underlay in that case (e.g. Radix `disableOutsidePointerEvents`).
                let mut last_pointer_move_hit = self
                    .last_pointer_move_hit
                    .get(pointer_id)
                    .copied()
                    .flatten();
                if barrier_root.is_some()
                    && last_pointer_move_hit.is_some_and(|n| !node_in_active_layers(n))
                {
                    self.last_pointer_move_hit.remove(pointer_id);
                    last_pointer_move_hit = None;
                }

                if hit != last_pointer_move_hit {
                    synth_pointer_move_prev_target = last_pointer_move_hit;
                    match hit {
                        Some(hit) => {
                            self.last_pointer_move_hit.insert(*pointer_id, Some(hit));
                        }
                        None => {
                            self.last_pointer_move_hit.remove(pointer_id);
                        }
                    }
                }
            }

            if matches!(event, Event::InternalDrag(_)) {
                if let Some(node) = hit {
                    self.last_internal_drag_target = Some(node);
                } else if self
                    .last_internal_drag_target
                    .is_some_and(|n| !node_in_active_layers(n))
                {
                    self.last_internal_drag_target = None;
                }
            }

            hit.or_else(|| {
                matches!(event, Event::InternalDrag(_)).then_some(self.last_internal_drag_target)?
            })
            .or(barrier_root)
            .or(Some(default_root))
        } else {
            match event {
                Event::SetTextSelection { .. } => {
                    let selection_node = self.window.and_then(|window| {
                        crate::elements::with_window_state(app, window, |window_state| {
                            window_state
                                .active_text_selection()
                                .and_then(|selection| window_state.node_entry(selection.element))
                                .map(|entry| entry.node)
                        })
                    });
                    selection_node.or(self.focus).or(Some(default_root))
                }
                _ => self.focus.or(Some(default_root)),
            }
        };

        let Some(mut node_id) = target else {
            return;
        };

        if matches!(event, Event::Pointer(PointerEvent::Down { .. }))
            && pointer_down_outside.suppress_hit_test_dispatch
        {
            if needs_redraw {
                self.request_redraw_coalesced(app);
            }
            return;
        }

        if matches!(event, Event::Pointer(PointerEvent::Up { .. }))
            && suppress_touch_up_outside_dispatch
        {
            if needs_redraw {
                self.request_redraw_coalesced(app);
            }
            return;
        }

        if suppress_pointer_dispatch && matches!(event, Event::Pointer(_)) {
            if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                let (_, elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.dispatch.pointer_move_layer_observers"),
                    || {
                        self.dispatch_pointer_move_layer_observers(
                            app,
                            services,
                            &input_ctx,
                            barrier_root,
                            event,
                            &mut needs_redraw,
                            &mut invalidation_visited,
                        );
                    },
                );
                if let Some(elapsed) = elapsed {
                    self.debug_stats.dispatch_pointer_move_layer_observers_time += elapsed;
                }
            }
            if needs_redraw {
                self.request_redraw_coalesced(app);
            }
            return;
        }

        if cursor_choice.is_none()
            && input_ctx.caps.ui.cursor_icons
            && matches!(event, Event::Pointer(_))
            && let Some(hit) = pointer_hit
        {
            cursor_choice = self.cursor_icon_query_for_pointer_hit(
                hit,
                &input_ctx,
                event,
                Some(pointer_chain_snapshot),
            );
            cursor_choice_from_query = cursor_choice.is_some();
        }

        if !suppress_pointer_dispatch
            && matches!(
                event,
                Event::Pointer(_)
                    | Event::PointerCancel(_)
                    | Event::ExternalDrag(_)
                    | Event::InternalDrag(_)
            )
        {
            let (chain, chain_elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_enabled,
                || tracing::trace_span!("fret.ui.dispatch.event_chain_build"),
                || {
                    if event_position(event).is_some() {
                        self.build_mapped_event_chain(node_id, event, Some(pointer_chain_snapshot))
                    } else {
                        self.build_unmapped_event_chain(
                            node_id,
                            event,
                            Some(&dispatch_cx.focus_snapshot),
                        )
                    }
                },
            );
            if let Some(chain_elapsed) = chain_elapsed {
                self.debug_stats.dispatch_event_chain_build_time += chain_elapsed;
            }
            let pointer_hit_is_text_input =
                if matches!(event, Event::Pointer(PointerEvent::Down { .. }))
                    && let Some(window) = self.window
                {
                    chain.iter().any(|(node_id, _)| {
                        crate::declarative::element_record_for_node(app, window, *node_id)
                            .is_some_and(|record| {
                                matches!(
                                    &record.instance,
                                    crate::declarative::ElementInstance::TextInput(_)
                                        | crate::declarative::ElementInstance::TextArea(_)
                                        | crate::declarative::ElementInstance::TextInputRegion(_)
                                )
                            })
                    })
                } else {
                    false
                };
            let pointer_hit_is_pressable =
                if matches!(event, Event::Pointer(PointerEvent::Down { .. }))
                    && let Some(window) = self.window
                {
                    chain.iter().any(|(node_id, _)| {
                        crate::declarative::element_record_for_node(app, window, *node_id)
                            .is_some_and(|record| {
                                matches!(
                                    &record.instance,
                                    crate::declarative::ElementInstance::Pressable(_)
                                )
                            })
                    })
                } else {
                    false
                };
            let should_run_capture_phase = match event {
                Event::Pointer(PointerEvent::Down { .. })
                | Event::Pointer(PointerEvent::Up { .. })
                | Event::Pointer(PointerEvent::Wheel { .. })
                | Event::Pointer(PointerEvent::PinchGesture { .. })
                | Event::PointerCancel(..) => true,
                Event::Pointer(PointerEvent::Move { buttons, .. }) => {
                    captured.is_some() || buttons.left || buttons.right || buttons.middle
                }
                _ => false,
            };
            let mut stopped_in_capture = false;
            if should_run_capture_phase {
                let mut capture_ctx = input_ctx.clone();
                capture_ctx.dispatch_phase = InputDispatchPhase::Capture;

                let (_, capture_elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.dispatch.widget_capture"),
                    || {
                        for (node_id, event_for_node) in chain.iter().rev() {
                            let node_id = *node_id;
                            let (
                                invalidations,
                                requested_focus,
                                requested_capture,
                                requested_cursor,
                                notify_requested,
                                notify_requested_location,
                                stop_propagation,
                            ) = self.with_widget_mut(node_id, |widget, tree| {
                                let (children, bounds) = tree
                                    .nodes
                                    .get(node_id)
                                    .map(|n| (n.children.as_slice(), n.bounds))
                                    .unwrap_or((&[][..], Rect::default()));
                                let mut cx = EventCx {
                                    app,
                                    services: &mut *services,
                                    node: node_id,
                                    layer_root: tree.node_root(node_id),
                                    window: tree.window,
                                    pointer_id: event_pointer_id_for_capture,
                                    scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                                    event_window_position,
                                    event_window_wheel_delta,
                                    input_ctx: capture_ctx.clone(),
                                    pointer_hit_is_text_input,
                                    pointer_hit_is_pressable,
                                    prevented_default_actions: &mut prevented_default_actions,
                                    children,
                                    focus: tree.focus,
                                    captured: event_pointer_id_for_capture
                                        .and_then(|p| tree.captured.get(&p).copied()),
                                    bounds,
                                    invalidations: Vec::new(),
                                    requested_focus: None,
                                    requested_capture: None,
                                    requested_cursor: None,
                                    notify_requested: false,
                                    notify_requested_location: None,
                                    stop_propagation: false,
                                };
                                widget.event_capture(&mut cx, event_for_node);
                                (
                                    cx.invalidations,
                                    cx.requested_focus,
                                    cx.requested_capture,
                                    cx.requested_cursor,
                                    cx.notify_requested,
                                    cx.notify_requested_location,
                                    cx.stop_propagation,
                                )
                            });

                            if !invalidations.is_empty()
                                || requested_focus.is_some()
                                || requested_capture.is_some()
                                || notify_requested
                            {
                                needs_redraw = true;
                            }

                            for (id, inv) in invalidations {
                                self.mark_invalidation(id, inv);
                            }
                            if notify_requested {
                                self.debug_record_notify_request(
                                    app.frame_id(),
                                    node_id,
                                    notify_requested_location,
                                );
                                self.mark_invalidation_with_source(
                                    node_id,
                                    Invalidation::Paint,
                                    UiDebugInvalidationSource::Notify,
                                );
                            }

                            if let Some(focus) = requested_focus
                                && self.focus_request_is_allowed(
                                    app,
                                    self.window,
                                    dispatch_cx.active_focus_roots.as_slice(),
                                    focus,
                                    Some(&dispatch_cx.focus_snapshot),
                                )
                            {
                                focus_requested = true;
                                if let Some(prev) = self.focus {
                                    self.mark_invalidation(prev, Invalidation::Paint);
                                }
                                self.focus = Some(focus);
                                self.mark_invalidation(focus, Invalidation::Paint);
                                // Avoid scrolling during pointer-driven focus changes:
                                // programmatic scroll-to-focus can move content under a stationary cursor,
                                // causing pointer activation to miss/cancel (especially for nested pressables).
                                //
                                // Keyboard traversal still scrolls focused nodes into view.
                                if !matches!(event, Event::Pointer(_) | Event::PointerCancel(_)) {
                                    self.scroll_node_into_view(app, focus);
                                }
                            } else if requested_focus.is_some() {
                                focus_requested = true;
                            }

                            if let Some(capture) = requested_capture
                                && let Some(pointer_id) = event_pointer_id_for_capture
                            {
                                match capture {
                                    Some(node) => {
                                        let allow = !dock_drag_affects_window
                                            || dock_drag_capture_anchor == Some(node);
                                        if allow {
                                            if !matches!(event, Event::PointerCancel(_))
                                                && let Some(old_capture) =
                                                    self.captured.get(&pointer_id).copied()
                                                && old_capture != node
                                                && node_in_active_layers(old_capture)
                                            {
                                                let mut cancel_ctx = input_ctx.clone();
                                                cancel_ctx.dispatch_phase =
                                                    InputDispatchPhase::Bubble;
                                                let cancel_event =
                                                    pointer_cancel_event_for_capture_switch(
                                                        event, pointer_id,
                                                    );
                                                let _ = self.dispatch_event_to_node_chain(
                                                    app,
                                                    services,
                                                    &dispatch_cx,
                                                    &cancel_ctx,
                                                    old_capture,
                                                    &cancel_event,
                                                    &mut needs_redraw,
                                                    &mut invalidation_visited,
                                                );
                                            }
                                            self.captured.insert(pointer_id, node);
                                        }
                                    }
                                    None => {
                                        self.captured.remove(&pointer_id);
                                    }
                                }
                            }

                            if let Some(requested_cursor) = requested_cursor
                                && (cursor_choice.is_none() || cursor_choice_from_query)
                            {
                                cursor_choice = Some(requested_cursor);
                                cursor_choice_from_query = false;
                            }

                            if stop_propagation {
                                stop_propagation_requested = true;
                                if stop_propagation_requested_by.is_none() {
                                    stop_propagation_requested_by = Some(node_id);
                                }
                                if is_wheel && wheel_stop_node.is_none() {
                                    wheel_stop_node = Some(node_id);
                                }
                                stopped_in_capture = true;
                                break;
                            }
                        }
                    },
                );
                if let Some(capture_elapsed) = capture_elapsed {
                    self.debug_stats.dispatch_widget_capture_time += capture_elapsed;
                }
            }

            if !stopped_in_capture {
                let mut bubble_ctx = input_ctx.clone();
                bubble_ctx.dispatch_phase = InputDispatchPhase::Bubble;

                let (_, bubble_elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.dispatch.widget_bubble"),
                    || {
                        for (node_id, event_for_node) in chain {
                            let (
                                invalidations,
                                requested_focus,
                                requested_capture,
                                requested_cursor,
                                notify_requested,
                                notify_requested_location,
                                stop_propagation,
                            ) = self.with_widget_mut(node_id, |widget, tree| {
                                let (children, bounds) = tree
                                    .nodes
                                    .get(node_id)
                                    .map(|n| (n.children.as_slice(), n.bounds))
                                    .unwrap_or((&[][..], Rect::default()));
                                let mut cx = EventCx {
                                    app,
                                    services: &mut *services,
                                    node: node_id,
                                    layer_root: tree.node_root(node_id),
                                    window: tree.window,
                                    pointer_id: event_pointer_id_for_capture,
                                    scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                                    event_window_position,
                                    event_window_wheel_delta,
                                    input_ctx: bubble_ctx.clone(),
                                    pointer_hit_is_text_input,
                                    pointer_hit_is_pressable,
                                    prevented_default_actions: &mut prevented_default_actions,
                                    children,
                                    focus: tree.focus,
                                    captured: event_pointer_id_for_capture
                                        .and_then(|p| tree.captured.get(&p).copied()),
                                    bounds,
                                    invalidations: Vec::new(),
                                    requested_focus: None,
                                    requested_capture: None,
                                    requested_cursor: None,
                                    notify_requested: false,
                                    notify_requested_location: None,
                                    stop_propagation: false,
                                };
                                widget.event(&mut cx, &event_for_node);
                                if cx.requested_cursor.is_none()
                                    && matches!(event_for_node, Event::Pointer(_))
                                    && cx.input_ctx.caps.ui.cursor_icons
                                    && let Some(position) = event_position(&event_for_node)
                                {
                                    cx.requested_cursor =
                                        widget.cursor_icon_at(bounds, position, &cx.input_ctx);
                                }
                                (
                                    cx.invalidations,
                                    cx.requested_focus,
                                    cx.requested_capture,
                                    cx.requested_cursor,
                                    cx.notify_requested,
                                    cx.notify_requested_location,
                                    cx.stop_propagation,
                                )
                            });

                            if !invalidations.is_empty()
                                || requested_focus.is_some()
                                || requested_capture.is_some()
                                || notify_requested
                            {
                                needs_redraw = true;
                            }

                            for (id, inv) in invalidations {
                                self.mark_invalidation(id, inv);
                            }
                            if notify_requested {
                                self.debug_record_notify_request(
                                    app.frame_id(),
                                    node_id,
                                    notify_requested_location,
                                );
                                self.mark_invalidation_with_source(
                                    node_id,
                                    Invalidation::Paint,
                                    UiDebugInvalidationSource::Notify,
                                );
                            }

                            if let Some(focus) = requested_focus
                                && self.focus_request_is_allowed(
                                    app,
                                    self.window,
                                    dispatch_cx.active_focus_roots.as_slice(),
                                    focus,
                                    Some(&dispatch_cx.focus_snapshot),
                                )
                            {
                                focus_requested = true;
                                if let Some(prev) = self.focus {
                                    self.mark_invalidation(prev, Invalidation::Paint);
                                }
                                self.focus = Some(focus);
                                self.mark_invalidation(focus, Invalidation::Paint);
                                // Avoid scrolling during pointer-driven focus changes:
                                // programmatic scroll-to-focus can move content under a stationary cursor,
                                // causing pointer activation to miss/cancel (especially for nested pressables).
                                //
                                // Keyboard traversal still scrolls focused nodes into view.
                                if !matches!(
                                    event_for_node,
                                    Event::Pointer(_) | Event::PointerCancel(_)
                                ) {
                                    self.scroll_node_into_view(app, focus);
                                }
                            } else if requested_focus.is_some() {
                                focus_requested = true;
                            }

                            if let Some(capture) = requested_capture
                                && let Some(pointer_id) = event_pointer_id_for_capture
                            {
                                match capture {
                                    Some(node) => {
                                        let allow = !dock_drag_affects_window
                                            || dock_drag_capture_anchor == Some(node);
                                        if allow {
                                            if !matches!(event, Event::PointerCancel(_))
                                                && let Some(old_capture) =
                                                    self.captured.get(&pointer_id).copied()
                                                && old_capture != node
                                                && node_in_active_layers(old_capture)
                                            {
                                                let mut cancel_ctx = input_ctx.clone();
                                                cancel_ctx.dispatch_phase =
                                                    InputDispatchPhase::Bubble;
                                                let cancel_event =
                                                    pointer_cancel_event_for_capture_switch(
                                                        event, pointer_id,
                                                    );
                                                let _ = self.dispatch_event_to_node_chain(
                                                    app,
                                                    services,
                                                    &dispatch_cx,
                                                    &cancel_ctx,
                                                    old_capture,
                                                    &cancel_event,
                                                    &mut needs_redraw,
                                                    &mut invalidation_visited,
                                                );
                                            }
                                            self.captured.insert(pointer_id, node);
                                        }
                                    }
                                    None => {
                                        self.captured.remove(&pointer_id);
                                    }
                                }
                            }

                            if let Some(requested_cursor) = requested_cursor
                                && (cursor_choice.is_none() || cursor_choice_from_query)
                            {
                                cursor_choice = Some(requested_cursor);
                                cursor_choice_from_query = false;
                            }

                            if stop_propagation {
                                stop_propagation_requested = true;
                                if stop_propagation_requested_by.is_none() {
                                    stop_propagation_requested_by = Some(node_id);
                                }
                                if is_wheel && wheel_stop_node.is_none() {
                                    wheel_stop_node = Some(node_id);
                                }
                            }

                            let captured_now = event_pointer_id_for_capture
                                .and_then(|p| self.captured.get(&p).copied());
                            if captured_now.is_some() || stop_propagation {
                                break;
                            }
                        }
                    },
                );
                if let Some(bubble_elapsed) = bubble_elapsed {
                    self.debug_stats.dispatch_widget_bubble_time += bubble_elapsed;
                }
            }
        } else if matches!(event, Event::KeyDown { .. } | Event::KeyUp { .. }) {
            // Key events must be scoped to the active focus layers (including focus barriers that
            // do not block pointer input). When no focused node is available, default to the
            // combined barrier root instead of the underlay base root.
            let key_start = self
                .focus
                .filter(|&n| dispatch_cx.node_in_active_focus_layers(n))
                .or(dispatch_cx.barrier_root)
                .unwrap_or(node_id);

            let mut chain: Vec<NodeId> = Vec::new();
            let mut cur = Some(key_start);
            while let Some(id) = cur {
                chain.push(id);
                if dispatch_cx.focus_snapshot.pre.get(id).is_none() {
                    debug_assert!(
                        false,
                        "dispatch/window: key chain node missing from focus snapshot (node={id:?}, frame_id={:?}, window={:?})",
                        dispatch_cx.focus_snapshot.frame_id, dispatch_cx.focus_snapshot.window
                    );
                    break;
                }
                cur = dispatch_cx.focus_snapshot.parent.get(id).copied().flatten();
            }

            let mut stopped_in_capture = false;
            {
                let mut capture_ctx = input_ctx.clone();
                capture_ctx.dispatch_phase = InputDispatchPhase::Capture;

                let (_, capture_elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.dispatch.widget_capture", kind = "key"),
                    || {
                        for &node_id in chain.iter().rev() {
                            let (
                                invalidations,
                                requested_focus,
                                requested_capture,
                                requested_cursor,
                                notify_requested,
                                notify_requested_location,
                                stop_propagation,
                            ) = self.with_widget_mut(node_id, |widget, tree| {
                                let (children, bounds) = tree
                                    .nodes
                                    .get(node_id)
                                    .map(|n| (n.children.as_slice(), n.bounds))
                                    .unwrap_or((&[][..], Rect::default()));
                                let mut cx = EventCx {
                                    app,
                                    services: &mut *services,
                                    node: node_id,
                                    layer_root: tree.node_root(node_id),
                                    window: tree.window,
                                    pointer_id: event_pointer_id_for_capture,
                                    scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                                    event_window_position,
                                    event_window_wheel_delta,
                                    input_ctx: capture_ctx.clone(),
                                    pointer_hit_is_text_input: false,
                                    pointer_hit_is_pressable: false,
                                    prevented_default_actions: &mut prevented_default_actions,
                                    children,
                                    focus: tree.focus,
                                    captured: event_pointer_id_for_capture
                                        .and_then(|p| tree.captured.get(&p).copied()),
                                    bounds,
                                    invalidations: Vec::new(),
                                    requested_focus: None,
                                    requested_capture: None,
                                    requested_cursor: None,
                                    notify_requested: false,
                                    notify_requested_location: None,
                                    stop_propagation: false,
                                };
                                widget.event_capture(&mut cx, event);
                                (
                                    cx.invalidations,
                                    cx.requested_focus,
                                    cx.requested_capture,
                                    cx.requested_cursor,
                                    cx.notify_requested,
                                    cx.notify_requested_location,
                                    cx.stop_propagation,
                                )
                            });

                            if !invalidations.is_empty()
                                || requested_focus.is_some()
                                || requested_capture.is_some()
                                || notify_requested
                            {
                                needs_redraw = true;
                            }

                            for (id, inv) in invalidations {
                                self.mark_invalidation(id, inv);
                            }
                            if notify_requested {
                                self.debug_record_notify_request(
                                    app.frame_id(),
                                    node_id,
                                    notify_requested_location,
                                );
                                self.mark_invalidation_with_source(
                                    node_id,
                                    Invalidation::Paint,
                                    UiDebugInvalidationSource::Notify,
                                );
                            }

                            if let Some(focus) = requested_focus
                                && self.focus_request_is_allowed(
                                    app,
                                    self.window,
                                    dispatch_cx.active_focus_roots.as_slice(),
                                    focus,
                                    Some(&dispatch_cx.focus_snapshot),
                                )
                            {
                                focus_requested = true;
                                if let Some(prev) = self.focus {
                                    self.mark_invalidation(prev, Invalidation::Paint);
                                }
                                self.focus = Some(focus);
                                self.mark_invalidation(focus, Invalidation::Paint);
                                self.scroll_node_into_view(app, focus);
                            } else if requested_focus.is_some() {
                                focus_requested = true;
                            }

                            if let Some(capture) = requested_capture
                                && let Some(pointer_id) = event_pointer_id_for_capture
                            {
                                match capture {
                                    Some(node) => {
                                        let allow = !dock_drag_affects_window
                                            || dock_drag_capture_anchor == Some(node);
                                        if allow {
                                            self.captured.insert(pointer_id, node);
                                        }
                                    }
                                    None => {
                                        self.captured.remove(&pointer_id);
                                    }
                                }
                            }

                            if requested_cursor.is_some() && cursor_choice.is_none() {
                                cursor_choice = requested_cursor;
                            }

                            if stop_propagation {
                                stop_propagation_requested = true;
                                if stop_propagation_requested_by.is_none() {
                                    stop_propagation_requested_by = Some(node_id);
                                }
                                stopped_in_capture = true;
                                break;
                            }
                        }
                    },
                );
                if let Some(capture_elapsed) = capture_elapsed {
                    self.debug_stats.dispatch_widget_capture_time += capture_elapsed;
                }
            }
            if !stopped_in_capture {
                let mut bubble_ctx = input_ctx.clone();
                bubble_ctx.dispatch_phase = InputDispatchPhase::Bubble;

                let (_, bubble_elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.dispatch.widget_bubble", kind = "key"),
                    || {
                        for node_id in chain {
                            let (
                                invalidations,
                                requested_focus,
                                requested_capture,
                                requested_cursor,
                                notify_requested,
                                notify_requested_location,
                                stop_propagation,
                            ) = self.with_widget_mut(node_id, |widget, tree| {
                                let (children, bounds) = tree
                                    .nodes
                                    .get(node_id)
                                    .map(|n| (n.children.as_slice(), n.bounds))
                                    .unwrap_or((&[][..], Rect::default()));
                                let mut cx = EventCx {
                                    app,
                                    services: &mut *services,
                                    node: node_id,
                                    layer_root: tree.node_root(node_id),
                                    window: tree.window,
                                    pointer_id: event_pointer_id_for_capture,
                                    scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                                    event_window_position,
                                    event_window_wheel_delta,
                                    input_ctx: bubble_ctx.clone(),
                                    pointer_hit_is_text_input: false,
                                    pointer_hit_is_pressable: false,
                                    prevented_default_actions: &mut prevented_default_actions,
                                    children,
                                    focus: tree.focus,
                                    captured: event_pointer_id_for_capture
                                        .and_then(|p| tree.captured.get(&p).copied()),
                                    bounds,
                                    invalidations: Vec::new(),
                                    requested_focus: None,
                                    requested_capture: None,
                                    requested_cursor: None,
                                    notify_requested: false,
                                    notify_requested_location: None,
                                    stop_propagation: false,
                                };
                                widget.event(&mut cx, event);
                                (
                                    cx.invalidations,
                                    cx.requested_focus,
                                    cx.requested_capture,
                                    cx.requested_cursor,
                                    cx.notify_requested,
                                    cx.notify_requested_location,
                                    cx.stop_propagation,
                                )
                            });

                            if !invalidations.is_empty()
                                || requested_focus.is_some()
                                || requested_capture.is_some()
                                || notify_requested
                            {
                                needs_redraw = true;
                            }

                            for (id, inv) in invalidations {
                                self.mark_invalidation(id, inv);
                            }
                            if notify_requested {
                                self.debug_record_notify_request(
                                    app.frame_id(),
                                    node_id,
                                    notify_requested_location,
                                );
                                self.mark_invalidation_with_source(
                                    node_id,
                                    Invalidation::Paint,
                                    UiDebugInvalidationSource::Notify,
                                );
                            }

                            if let Some(focus) = requested_focus
                                && self.focus_request_is_allowed(
                                    app,
                                    self.window,
                                    dispatch_cx.active_focus_roots.as_slice(),
                                    focus,
                                    Some(&dispatch_cx.focus_snapshot),
                                )
                            {
                                focus_requested = true;
                                if let Some(prev) = self.focus {
                                    self.mark_invalidation(prev, Invalidation::Paint);
                                }
                                self.focus = Some(focus);
                                self.mark_invalidation(focus, Invalidation::Paint);
                                self.scroll_node_into_view(app, focus);
                            } else if requested_focus.is_some() {
                                focus_requested = true;
                            }

                            if let Some(capture) = requested_capture
                                && let Some(pointer_id) = event_pointer_id_for_capture
                            {
                                match capture {
                                    Some(node) => {
                                        let allow = !dock_drag_affects_window
                                            || dock_drag_capture_anchor == Some(node);
                                        if allow {
                                            self.captured.insert(pointer_id, node);
                                        }
                                    }
                                    None => {
                                        self.captured.remove(&pointer_id);
                                    }
                                }
                            }

                            if requested_cursor.is_some() && cursor_choice.is_none() {
                                cursor_choice = requested_cursor;
                            }

                            if stop_propagation {
                                stop_propagation_requested = true;
                                if stop_propagation_requested_by.is_none() {
                                    stop_propagation_requested_by = Some(node_id);
                                }
                                break;
                            }
                        }
                    },
                );
                if let Some(bubble_elapsed) = bubble_elapsed {
                    self.debug_stats.dispatch_widget_bubble_time += bubble_elapsed;
                }
            }

            let stopped_by_dismissible_root_hook = stop_propagation_requested
                && self.window.is_some_and(|window| {
                    stop_propagation_requested_by
                        .and_then(|node| self.nodes.get(node).and_then(|n| n.element))
                        .and_then(|element| {
                            crate::elements::with_element_state(
                                app,
                                window,
                                element,
                                crate::action::DismissibleActionHooks::default,
                                |hooks| hooks.on_dismiss_request.clone(),
                            )
                        })
                        .is_some()
                });

            if defer_escape_overlay_dismiss
                && !stopped_by_dismissible_root_hook
                && (!stop_propagation_requested || !focus_requested)
                && let Event::KeyDown {
                    key: fret_core::KeyCode::Escape,
                    repeat: false,
                    ..
                } = event
                && let Some(window) = self.window
                && self.dismiss_topmost_overlay_on_escape(app, window, base_root, barrier_root)
            {
                self.request_redraw_coalesced(app);
                return;
            }
        } else {
            loop {
                let (
                    invalidations,
                    requested_focus,
                    requested_capture,
                    requested_cursor,
                    notify_requested,
                    notify_requested_location,
                    stop_propagation,
                ) = self.with_widget_mut(node_id, |widget, tree| {
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: &mut *services,
                        node: node_id,
                        layer_root: tree.node_root(node_id),
                        window: tree.window,
                        pointer_id: event_pointer_id_for_capture,
                        scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                        event_window_position,
                        event_window_wheel_delta,
                        input_ctx: input_ctx.clone(),
                        pointer_hit_is_text_input: false,
                        pointer_hit_is_pressable: false,
                        prevented_default_actions: &mut prevented_default_actions,
                        children,
                        focus: tree.focus,
                        captured: event_pointer_id_for_capture
                            .and_then(|p| tree.captured.get(&p).copied()),
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        notify_requested: false,
                        notify_requested_location: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, event);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.requested_cursor,
                        cx.notify_requested,
                        cx.notify_requested_location,
                        cx.stop_propagation,
                    )
                });
                if !invalidations.is_empty()
                    || requested_focus.is_some()
                    || requested_capture.is_some()
                    || notify_requested
                {
                    needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }
                if notify_requested {
                    self.debug_record_notify_request(
                        app.frame_id(),
                        node_id,
                        notify_requested_location,
                    );
                    self.mark_invalidation_with_source(
                        node_id,
                        Invalidation::Paint,
                        UiDebugInvalidationSource::Notify,
                    );
                }

                if let Some(focus) = requested_focus
                    && self.focus_request_is_allowed(
                        app,
                        self.window,
                        dispatch_cx.active_focus_roots.as_slice(),
                        focus,
                        Some(&dispatch_cx.focus_snapshot),
                    )
                {
                    focus_requested = true;
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                    // Avoid scrolling during pointer-driven focus changes:
                    // programmatic scroll-to-focus can move content under a stationary cursor,
                    // causing pointer activation to miss/cancel (especially for nested pressables).
                    //
                    // Keyboard traversal still scrolls focused nodes into view.
                    if !matches!(event, Event::Pointer(_) | Event::PointerCancel(_)) {
                        self.scroll_node_into_view(app, focus);
                    }
                } else if requested_focus.is_some() {
                    focus_requested = true;
                }

                if let Some(capture) = requested_capture
                    && let Some(pointer_id) = event_pointer_id_for_capture
                {
                    match capture {
                        Some(node) => {
                            let allow =
                                !dock_drag_affects_window || dock_drag_capture_anchor == Some(node);
                            if allow {
                                self.captured.insert(pointer_id, node);
                            }
                        }
                        None => {
                            self.captured.remove(&pointer_id);
                        }
                    }
                };

                if requested_cursor.is_some() && cursor_choice.is_none() {
                    cursor_choice = requested_cursor;
                }

                if stop_propagation {
                    stop_propagation_requested = true;
                    if stop_propagation_requested_by.is_none() {
                        stop_propagation_requested_by = Some(node_id);
                    }
                    if is_wheel && wheel_stop_node.is_none() {
                        wheel_stop_node = Some(node_id);
                    }
                }

                let captured_now =
                    event_pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
                if captured_now.is_some() || stop_propagation {
                    break;
                }

                if dispatch_cx.focus_snapshot.pre.get(node_id).is_none() {
                    tracing::warn!(
                        node = ?node_id,
                        frame_id = ?dispatch_cx.focus_snapshot.frame_id,
                        window = ?dispatch_cx.focus_snapshot.window,
                        "dispatch/window: bubble chain node missing from focus snapshot"
                    );
                    break;
                }
                node_id = match dispatch_cx
                    .focus_snapshot
                    .parent
                    .get(node_id)
                    .copied()
                    .flatten()
                {
                    Some(parent) => parent,
                    None => break,
                };
            }
        }

        if let Event::Pointer(PointerEvent::Down {
            button,
            pointer_type,
            ..
        }) = event
            && *button == fret_core::MouseButton::Left
            && !focus_requested
            && !prevented_default_actions.contains(fret_runtime::DefaultAction::FocusOnPointerDown)
            && captured.is_none()
            && internal_drag_target.is_none()
            && let Some(window) = self.window
            && let Some(hit) = pointer_hit
        {
            let candidate = self.first_focusable_ancestor_including_declarative(app, window, hit);
            if let Some(focus) = candidate
                && self.focus_request_is_allowed(
                    app,
                    self.window,
                    dispatch_cx.active_focus_roots.as_slice(),
                    focus,
                    Some(&dispatch_cx.focus_snapshot),
                )
            {
                if let Some(prev) = self.focus {
                    self.mark_invalidation(prev, Invalidation::Paint);
                }
                self.focus = Some(focus);
                self.mark_invalidation(focus, Invalidation::Paint);

                // Mobile-friendly best-effort: if touch input focused a text-editing widget,
                // request the virtual keyboard within the same input turn so platforms that
                // require user activation can comply (ADR 0261).
                if *pointer_type == fret_core::PointerType::Touch && self.focus_is_text_input(app) {
                    app.push_effect(Effect::ImeRequestVirtualKeyboard {
                        window,
                        visible: true,
                    });
                }

                // Pointer-driven focus should not scroll: the user is already interacting at the
                // pointer location, and scrolling here can move content under the cursor between
                // pointer-down and pointer-up.
                needs_redraw = true;
            }
        }

        if is_wheel
            && let Some(scroll_target) = wheel_stop_node
            && let Some(window) = self.window
        {
            let is_scroll_target = declarative::with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let record = window_frame.instances.get(scroll_target)?;
                Some(matches!(
                    record.instance,
                    declarative::ElementInstance::Scroll(_)
                        | declarative::ElementInstance::VirtualList(_)
                        | declarative::ElementInstance::WheelRegion(_)
                        | declarative::ElementInstance::Scrollbar(_)
                ))
            })
            .unwrap_or(false);

            if is_scroll_target {
                struct ScrollDismissHookHost<'a, H: crate::UiHost> {
                    app: &'a mut H,
                    window: AppWindowId,
                    element: crate::GlobalElementId,
                }

                impl<H: crate::UiHost> crate::action::UiActionHost for ScrollDismissHookHost<'_, H> {
                    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                        self.app.models_mut()
                    }

                    fn push_effect(&mut self, effect: Effect) {
                        match effect {
                            Effect::SetTimer {
                                window: Some(window),
                                token,
                                ..
                            } if window == self.window => {
                                crate::elements::record_timer_target(
                                    &mut *self.app,
                                    window,
                                    token,
                                    self.element,
                                );
                            }
                            Effect::CancelTimer { token } => {
                                crate::elements::clear_timer_target(
                                    &mut *self.app,
                                    self.window,
                                    token,
                                );
                            }
                            _ => {}
                        }
                        self.app.push_effect(effect);
                    }

                    fn request_redraw(&mut self, window: AppWindowId) {
                        self.app.request_redraw(window);
                    }

                    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
                        self.app.next_timer_token()
                    }

                    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
                        self.app.next_clipboard_token()
                    }

                    fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
                        self.app.next_share_sheet_token()
                    }
                }

                let mut dismissed_any = false;
                for layer_id in self.visible_layers_in_paint_order() {
                    let Some(layer) = self.layers.get(layer_id) else {
                        continue;
                    };
                    if layer.scroll_dismiss_elements.is_empty() {
                        continue;
                    }
                    let should_dismiss = layer
                        .scroll_dismiss_elements
                        .iter()
                        .copied()
                        .filter_map(|element| {
                            crate::elements::node_for_element(app, window, element)
                        })
                        .any(|node| self.is_descendant(scroll_target, node));
                    if !should_dismiss {
                        continue;
                    }
                    let Some(root_element) = self.nodes.get(layer.root).and_then(|n| n.element)
                    else {
                        continue;
                    };
                    let hook = crate::elements::with_element_state(
                        app,
                        window,
                        root_element,
                        crate::action::DismissibleActionHooks::default,
                        |hooks| hooks.on_dismiss_request.clone(),
                    );
                    let Some(hook) = hook else {
                        continue;
                    };
                    let mut host = ScrollDismissHookHost {
                        app,
                        window,
                        element: root_element,
                    };
                    let mut req =
                        crate::action::DismissRequestCx::new(crate::action::DismissReason::Scroll);
                    hook(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: root_element,
                        },
                        &mut req,
                    );
                    dismissed_any = true;
                }

                if dismissed_any {
                    needs_redraw = true;
                }
            }
        }

        if matches!(event, Event::PointerCancel(_))
            && let Some(pointer_id) = event_pointer_id_for_capture
        {
            self.captured.remove(&pointer_id);
        }

        if let Event::PointerCancel(e) = event
            && let Some(window) = self.window
            && pointer_type_supports_hover(e.pointer_type)
        {
            let (prev_element, prev_node, _next_element, _next_node) =
                crate::elements::update_hovered_pressable(app, window, None);
            if prev_node.is_some() {
                needs_redraw = true;
                self.debug_record_hover_edge_pressable();
                if let Some(node) = prev_node {
                    self.mark_invalidation_dedup_with_source(
                        node,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Hover,
                    );
                }
            }

            if let Some(element) = prev_element
                && prev_node.is_some()
            {
                Self::run_pressable_hover_hook(app, window, element, false);
            }

            let (_prev_element, prev_node, _next_element, _next_node) =
                crate::elements::update_hovered_hover_region(app, window, None);
            if prev_node.is_some() {
                needs_redraw = true;
                self.debug_record_hover_edge_hover_region();
                if let Some(node) = prev_node {
                    self.mark_invalidation_dedup_with_source(
                        node,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Hover,
                    );
                }
            }
        }

        if let Event::PointerCancel(e) = event {
            self.touch_pointer_down_outside_candidates
                .remove(&e.pointer_id);
        }

        #[cfg(feature = "diagnostics")]
        if defer_keydown_shortcuts_until_after_dispatch
            && stop_propagation_requested
            && let Some(window) = self.window
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
        {
            let focus_is_text_input = self.focus_is_text_input(app);
            let key_contexts = if !self.pending_shortcut.keystrokes.is_empty() {
                self.pending_shortcut.key_contexts.clone()
            } else {
                self.shortcut_key_context_stack(app, barrier_root)
            };
            app.with_global_mut_untracked(
                fret_runtime::WindowShortcutRoutingDiagnosticsStore::default,
                |store, app| {
                    store.record(
                        window,
                        fret_runtime::ShortcutRoutingDecision {
                            seq: 0,
                            frame_id: app.frame_id(),
                            phase: fret_runtime::ShortcutRoutingPhase::PostDispatch,
                            key: *key,
                            modifiers: *modifiers,
                            repeat: *repeat,
                            deferred: true,
                            focus_is_text_input,
                            ime_composing: self.ime_composing,
                            pending_sequence_len: self
                                .pending_shortcut
                                .keystrokes
                                .len()
                                .min(u32::MAX as usize)
                                as u32,
                            outcome: fret_runtime::ShortcutRoutingOutcome::ConsumedByWidget,
                            command: None,
                            command_enabled: None,
                            key_contexts,
                        },
                    );
                },
            );
        }

        if defer_keydown_shortcuts_until_after_dispatch
            && !stop_propagation_requested
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
        {
            let focus_is_text_input = self.focus_is_text_input(app);
            let input_ctx_for_shortcuts = InputContext {
                focus_is_text_input,
                ..input_ctx.clone()
            };

            let ime_reserved = self.ime_composing
                && Self::should_defer_keydown_shortcut_matching_to_text_input(
                    *key,
                    *modifiers,
                    focus_is_text_input,
                );

            #[cfg(feature = "diagnostics")]
            if let Some(window) = self.window
                && ime_reserved
            {
                let key_contexts = if !self.pending_shortcut.keystrokes.is_empty() {
                    self.pending_shortcut.key_contexts.clone()
                } else {
                    self.shortcut_key_context_stack(app, barrier_root)
                };
                app.with_global_mut_untracked(
                    fret_runtime::WindowShortcutRoutingDiagnosticsStore::default,
                    |store, app| {
                        store.record(
                            window,
                            fret_runtime::ShortcutRoutingDecision {
                                seq: 0,
                                frame_id: app.frame_id(),
                                phase: fret_runtime::ShortcutRoutingPhase::PostDispatch,
                                key: *key,
                                modifiers: *modifiers,
                                repeat: *repeat,
                                deferred: true,
                                focus_is_text_input,
                                ime_composing: self.ime_composing,
                                pending_sequence_len: self
                                    .pending_shortcut
                                    .keystrokes
                                    .len()
                                    .min(u32::MAX as usize)
                                    as u32,
                                outcome: fret_runtime::ShortcutRoutingOutcome::ReservedForIme,
                                command: None,
                                command_enabled: None,
                                key_contexts,
                            },
                        );
                    },
                );
            }

            if !ime_reserved
                && self.handle_keydown_shortcuts(
                    app,
                    services,
                    KeydownShortcutParams {
                        input_ctx: &input_ctx_for_shortcuts,
                        barrier_root,
                        focus_is_text_input,
                        #[cfg(feature = "diagnostics")]
                        phase: fret_runtime::ShortcutRoutingPhase::PostDispatch,
                        #[cfg(feature = "diagnostics")]
                        deferred: true,
                        key: *key,
                        modifiers: *modifiers,
                        repeat: *repeat,
                    },
                )
            {
                if needs_redraw {
                    self.request_redraw_coalesced(app);
                }
                return;
            }
        }

        if let Event::Pointer(PointerEvent::Move { .. }) = event
            && let Some(prev) = synth_pointer_move_prev_target
            && captured.is_none()
            && node_in_active_layers(prev)
        {
            // Forward a synthetic hover-move to the previously hovered target so retained
            // widgets can clear hover state when the pointer crosses between siblings.
            //
            // We intentionally use observer dispatch to avoid allowing the previous target to
            // mutate focus/capture/cursor routing on the transition frame.
            let (_, elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_enabled,
                || tracing::trace_span!("fret.ui.dispatch.synth_hover_observer", node = ?prev),
                || {
                    self.dispatch_event_to_node_chain_observer(
                        app,
                        services,
                        &input_ctx,
                        prev,
                        event,
                        Some(&dispatch_cx.input_snapshot),
                        &mut invalidation_visited,
                    );
                    needs_redraw = true;
                },
            );
            if let Some(elapsed) = elapsed {
                self.debug_stats.dispatch_synth_hover_observer_time += elapsed;
            }
        }

        if is_wheel
            && wheel_stop_node.is_some()
            && captured.is_none()
            && let Some(window) = self.window
            && let Event::Pointer(PointerEvent::Wheel {
                position,
                pointer_type,
                ..
            }) = event
            && pointer_type_supports_hover(*pointer_type)
        {
            // Capture scroll-handle-driven invalidations triggered by this wheel event, including
            // out-of-band handle mutations that were not routed through a `Scroll` widget.
            self.invalidate_scroll_handle_bindings_for_changed_handles(
                app,
                crate::layout_pass::LayoutPassKind::Final,
                /* consume_deferred_scroll_to_item */ false,
                /* commit_scroll_handle_baselines */ false,
            );

            self.hit_test_path_cache = None;
            let hit = self.hit_test_layers_cached(hit_test_layer_roots, *position);

            let mut hit_for_hover = hit;
            let mut hit_for_hover_region = hit;
            let mut hit_for_raw_below_barrier: Option<NodeId> = None;
            if let Some((occlusion_layer, occlusion)) =
                self.topmost_pointer_occlusion_layer(barrier_root)
                && occlusion != PointerOcclusion::None
            {
                let occlusion_z = self
                    .layer_order
                    .iter()
                    .position(|id| *id == occlusion_layer);
                let hit_layer_z = hit
                    .and_then(|hit| self.node_layer(hit))
                    .and_then(|layer| self.layer_order.iter().position(|id| *id == layer));
                let hit_is_below_occlusion = match (occlusion_z, hit_layer_z, hit) {
                    (Some(oz), Some(hz), Some(_)) => hz < oz,
                    (Some(_), None, Some(_)) => true,
                    (Some(_), _, None) => true,
                    _ => false,
                };
                if hit_is_below_occlusion {
                    hit_for_raw_below_barrier = hit;
                    hit_for_hover = None;
                    hit_for_hover_region = None;
                }
            }

            let (_, elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_enabled,
                || tracing::trace_span!("fret.ui.dispatch.hover_update"),
                || {
                    self.update_hover_state_from_hit(
                        app,
                        window,
                        barrier_root,
                        Some(*position),
                        hit_for_hover,
                        hit_for_hover_region,
                        hit_for_raw_below_barrier,
                        Some(pointer_chain_snapshot),
                        &mut invalidation_visited,
                        &mut needs_redraw,
                    );
                },
            );
            if let Some(elapsed) = elapsed {
                self.debug_stats.dispatch_hover_update_time += elapsed;
            }
        }

        if input_ctx.caps.ui.cursor_icons
            && let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
        {
            let icon = cursor_choice.unwrap_or(fret_core::CursorIcon::Default);
            let (_, elapsed) = fret_perf::measure_span(
                self.debug_enabled,
                trace_enabled,
                || {
                    tracing::trace_span!(
                        "fret.ui.dispatch.cursor_effect",
                        window = ?window,
                        icon = ?icon
                    )
                },
                || app.push_effect(Effect::CursorSetIcon { window, icon }),
            );
            if let Some(elapsed) = elapsed {
                self.debug_stats.dispatch_cursor_effect_time += elapsed;
            }
        }

        if needs_redraw {
            self.request_redraw_coalesced(app);
        }
        let (_, elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.ui.dispatch.pointer_move_layer_observers"),
            || {
                self.dispatch_pointer_move_layer_observers(
                    app,
                    services,
                    &input_ctx,
                    barrier_root,
                    event,
                    &mut needs_redraw,
                    &mut invalidation_visited,
                );
            },
        );
        if let Some(elapsed) = elapsed {
            self.debug_stats.dispatch_pointer_move_layer_observers_time += elapsed;
        }
        if needs_redraw {
            self.request_redraw_coalesced(app);
        }

        // Keep IME enable/disable tightly coupled to focus changes caused by the event itself.
        let focus_is_text_input = self.focus_is_text_input(app);
        self.set_ime_allowed(app, focus_is_text_input);

        // Publish a post-dispatch snapshot so runner-level integration surfaces (e.g. OS menubars)
        // see the latest focus/modal state without waiting for the next paint pass.
        let (_, elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.ui.dispatch.post_dispatch_snapshot"),
            || {
                if let Some(window) = self.window {
                    let (_active_layers, barrier_root) = self.active_input_layers();
                    let is_pointer_move =
                        matches!(event, Event::Pointer(fret_core::PointerEvent::Move { .. }));
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    let mut input_ctx = InputContext {
                        platform: Platform::current(),
                        caps,
                        ui_has_modal: barrier_root.is_some(),
                        window_arbitration: None,
                        focus_is_text_input,
                        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
                        edit_can_undo: true,
                        edit_can_redo: true,
                        router_can_back: false,
                        router_can_forward: false,
                        dispatch_phase: InputDispatchPhase::Bubble,
                    };
                    if let Some(mode) = app
                        .global::<fret_runtime::WindowTextBoundaryModeService>()
                        .and_then(|svc| svc.mode(window))
                    {
                        input_ctx.text_boundary_mode = mode;
                    }
                    if let Some(mode) = self.focus_text_boundary_mode_override() {
                        input_ctx.text_boundary_mode = mode;
                    }
                    if let Some(availability) = app
                        .global::<fret_runtime::WindowCommandAvailabilityService>()
                        .and_then(|svc| svc.snapshot(window))
                        .copied()
                    {
                        input_ctx.edit_can_undo = availability.edit_can_undo;
                        input_ctx.edit_can_redo = availability.edit_can_redo;
                        input_ctx.router_can_back = availability.router_can_back;
                        input_ctx.router_can_forward = availability.router_can_forward;
                    }

                    let window_arbitration = self.window_input_arbitration_snapshot();
                    input_ctx.window_arbitration = Some(window_arbitration);

                    if is_pointer_move {
                        app.with_global_mut_untracked(
                            fret_runtime::WindowInputContextService::default,
                            |svc, _app| {
                                svc.set_snapshot(window, input_ctx);
                            },
                        );
                    } else {
                        let needs_update = app
                            .global::<fret_runtime::WindowInputContextService>()
                            .and_then(|svc| svc.snapshot(window))
                            .is_none_or(|prev| prev != &input_ctx);
                        if needs_update {
                            app.with_global_mut(
                                fret_runtime::WindowInputContextService::default,
                                |svc, _app| {
                                    svc.set_snapshot(window, input_ctx.clone());
                                },
                            );
                        }

                        self.publish_window_command_action_availability_snapshot(app, &input_ctx);
                    }
                }
            },
        );
        if let Some(elapsed) = elapsed {
            self.debug_stats.dispatch_post_dispatch_snapshot_time += elapsed;
        }
    }
}
