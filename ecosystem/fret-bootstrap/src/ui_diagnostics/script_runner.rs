use super::*;

fn append_diag_script_migration_trace(out_dir: &std::path::Path, line: &str) {
    if std::env::var_os("FRET_DIAG_SCRIPT_MIGRATION_TRACE").is_none() {
        return;
    }

    let path = out_dir.join("ui_diag_script_migration_trace.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        use std::io::Write as _;
        let _ = writeln!(file, "{}", line);
    }
}

impl UiDiagnosticsService {
    pub(super) fn maybe_start_pending_script(&mut self, app: &mut App, window: AppWindowId) {
        if !self.active_scripts.is_empty() {
            return;
        }

        let Some(script) = self.pending_script.take() else {
            return;
        };

        let run_id = self.pending_script_run_id.take().unwrap_or(0);
        // Anchor scripts to a stable "first seen" window. Multi-window diagnostics scripts often
        // treat `UiWindowTargetV1::FirstSeen` as "the main window" and rely on it remaining stable
        // even as additional OS windows are created (tear-off, floating tool windows, etc).
        //
        // Use the smallest window id among windows that have produced at least one snapshot so far
        // (best-effort liveness check). Fall back to the current window if we have no candidates
        // yet (e.g. during very early startup).
        let anchor_window = self
            .known_windows
            .iter()
            .copied()
            .filter(|w| {
                self.per_window
                    .get(w)
                    .is_some_and(|ring| ring.snapshots.back().is_some())
            })
            .min_by_key(|w| w.data().as_ffi())
            .unwrap_or(window);

        let mut active_script = ActiveScript {
            steps: script.steps,
            run_id,
            anchor_window,
            next_step: 0,
            base_ref: None,
            event_log: Vec::new(),
            event_log_dropped: 0,
            event_log_active_step: None,
            last_injected_step: None,
            wait_frames_remaining: 0,
            wait_until: None,
            wait_shortcut_routing_trace: None,
            wait_command_dispatch_trace: None,
            wait_overlay_placement_trace: None,
            screenshot_wait: None,
            v2_step_state: None,
            pointer_session: None,
            pending_cancel_cross_window_drag: None,
            last_reported_step: Some(0),
            last_reported_unix_ms: 0,
            selector_resolution_trace: Vec::new(),
            hit_test_trace: Vec::new(),
            click_stable_trace: Vec::new(),
            bounds_stable_trace: Vec::new(),
            focus_trace: Vec::new(),
            shortcut_routing_trace: Vec::new(),
            last_shortcut_routing_seq: 0,
            command_dispatch_trace: Vec::new(),
            last_command_dispatch_seq: 0,
            overlay_placement_trace: Vec::new(),
            web_ime_trace: Vec::new(),
            ime_event_trace: Vec::new(),
            last_explicit_cursor_override: None,
            last_explicit_cursor_override_pos: None,
        };

        // Avoid leaking clipboard responses across runs. Script steps that assert clipboard state
        // rely on a per-run token -> response map.
        self.reset_clipboard_text_responses();

        push_script_event_log(
            &mut active_script,
            &self.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_start".to_string(),
                step_index: Some(0),
                note: None,
                bundle_dir: None,
                window: Some(anchor_window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );

        if script.legacy_schema_v1 {
            push_script_event_log(
                &mut active_script,
                &self.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "compat.script_schema_v1".to_string(),
                    step_index: Some(0),
                    note: Some(
                        "script schema_version=1 was accepted and upgraded to v2 for execution"
                            .to_string(),
                    ),
                    bundle_dir: None,
                    window: Some(anchor_window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
        }

        self.active_scripts.insert(anchor_window, active_script);
        self.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(anchor_window.data().as_ffi()),
            stage: UiScriptStageV1::Running,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: self
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&self.cfg.out_dir, p)),
            last_bundle_artifact: self.last_dump_artifact_stats.clone(),
        });

        app.request_redraw(anchor_window);
        app.push_effect(Effect::RequestAnimationFrame(anchor_window));
    }

    pub(super) fn maybe_migrate_single_active_script_to_window(
        &mut self,
        app: &mut App,
        window: AppWindowId,
    ) {
        // Multi-window scripts can create additional OS windows (tear-off). Depending on platform
        // scheduling, the newly created window may become the only one receiving redraw callbacks
        // while a drag is active. Keep scripted playback progressing by migrating the single
        // active script to whichever window is currently being driven.
        if self.active_scripts.contains_key(&window) || self.active_scripts.len() != 1 {
            return;
        }

        let Some((&other_window, other_active)) = self.active_scripts.iter().next() else {
            return;
        };

        let allow_migrate_for_current_target = Self::can_migrate_for_current_target(other_active);
        let preferred: Option<AppWindowId> = if allow_migrate_for_current_target {
            None
        } else {
            Self::preferred_window_for_active_script(other_active)
        };

        let allow_migrate_for_unobserved_preferred =
            preferred.is_some_and(|w| !self.per_window.contains_key(&w));
        let dock_drag = dock_drag_runtime_state(app, self.known_windows.as_slice());
        let dock_drag_source_window = dock_drag.as_ref().map(|drag| drag.source_window);
        let step = other_active.steps.get(other_active.next_step);
        let step_window_target = Self::active_step_window_target(other_active);
        let other_step_index = other_active.next_step;
        let step_allows_off_window_eval = step.is_some_and(|step| match step {
            UiActionStepV2::WaitUntil { predicate, .. }
            | UiActionStepV2::Assert { predicate, .. } => {
                UiDiagnosticsService::predicate_can_eval_from_cached_test_id_bounds(predicate)
                    || UiDiagnosticsService::predicate_can_eval_off_window(predicate)
            }
            _ => false,
        });

        // Only drag-playback steps (which can explicitly remap captured drags) should "follow"
        // the runner-owned dock drag source window. Pointer sessions are window-local by default
        // and have their own migration rules; including them here can cause nudge ping-pong
        // between the drag source and a window-targeted step.
        let step_allows_dock_drag_migration = matches!(
            step,
            Some(
                UiActionStepV2::DragPointer { .. }
                    | UiActionStepV2::DragPointerUntil { .. }
                    | UiActionStepV2::DragTo { .. }
            )
        );
        let allow_migrate_for_dock_drag =
            dock_drag_source_window == Some(window) && step_allows_dock_drag_migration;

        // Pointer-session starts (`pointer_down`) are especially sensitive to window migration:
        // they must run in the window that owns the targeted semantics snapshot. If the runner is
        // currently driving a different window (e.g. occlusion/z-order), migrating the script can
        // create a ping-pong loop where `pointer_down` keeps handing off back to the intended
        // window, but migration immediately steals it away again before any snapshots arrive.
        //
        // For relative targets (last_seen/other), keep the script pinned to its current active
        // window and aggressively wake it until it produces frames.
        let pointer_down_relative = matches!(step, Some(UiActionStepV2::PointerDown { .. }))
            && matches!(
                step_window_target,
                Some(
                    UiWindowTargetV1::LastSeen
                        | UiWindowTargetV1::LastSeenOther
                        | UiWindowTargetV1::FirstSeenOther
                )
            );
        if pointer_down_relative && !allow_migrate_for_dock_drag {
            app.request_redraw(other_window);
            app.push_effect(Effect::Redraw(other_window));
            app.push_effect(Effect::RequestAnimationFrame(other_window));
            append_diag_script_migration_trace(
                &self.cfg.out_dir,
                &format!(
                    "unix_ms={} kind=nudge_active_pointer_down_window active_window={:?} requested_from={:?} step_index={} step_window_target={:?}",
                    unix_ms_now(),
                    other_window,
                    window,
                    other_step_index,
                    step_window_target,
                ),
            );
            return;
        }

        // Avoid migration loops for window-targeted steps.
        //
        // If a step resolves to a specific window (e.g. `last_seen`), do not migrate the active
        // script to an "unrelated" window that happens to be producing callbacks. Instead, nudge
        // the resolved window to produce frames and keep the script attached there.
        //
        // Exception: during an active dock drag, allow migration to follow the runner-owned drag
        // source window (ImGui-style tear-off), since the captured drag itself can be remapped.
        let test_id_hint = step.and_then(Self::step_test_id_hint);
        if !step_allows_off_window_eval
            && !allow_migrate_for_dock_drag
            && let Some(step_window_target) = step_window_target.as_ref()
            && let Some(resolved) = self.resolve_window_target_for_active_step_with_test_id_hint(
                other_window,
                other_active.anchor_window,
                Some(step_window_target),
                test_id_hint,
            )
            && resolved != window
        {
            app.request_redraw(resolved);
            app.push_effect(Effect::Redraw(resolved));
            app.push_effect(Effect::RequestAnimationFrame(resolved));
            append_diag_script_migration_trace(
                &self.cfg.out_dir,
                &format!(
                    "unix_ms={} kind=nudge_step_target resolved={:?} requested_from={:?} active_window={:?} step_index={} step_window_target={:?}",
                    unix_ms_now(),
                    resolved,
                    window,
                    other_window,
                    other_step_index,
                    step_window_target,
                ),
            );
            return;
        }

        // Note: do not use `dock_drag.current_window` for migration decisions. Hover can
        // legitimately change during a captured-pointer gesture; `dock_drag.source_window`
        // is the stable ownership signal.

        if preferred.is_none()
            || preferred == Some(window)
            || allow_migrate_for_dock_drag
            || allow_migrate_for_current_target
            || allow_migrate_for_unobserved_preferred
        {
            if let Some(mut active) = self.active_scripts.remove(&other_window) {
                append_diag_script_migration_trace(
                    &self.cfg.out_dir,
                    &format!(
                        "unix_ms={} from={:?} to={:?} preferred={:?} allow_current_target={} allow_unobserved_preferred={} allow_dock_drag={} step_index={} step_window_target={:?}",
                        unix_ms_now(),
                        other_window,
                        window,
                        preferred,
                        allow_migrate_for_current_target,
                        allow_migrate_for_unobserved_preferred,
                        allow_migrate_for_dock_drag,
                        other_step_index,
                        step_window_target,
                    ),
                );
                if allow_migrate_for_current_target
                    || allow_migrate_for_unobserved_preferred
                    || allow_migrate_for_dock_drag
                {
                    let allow_remap_captured_drag = allow_migrate_for_dock_drag;
                    Self::remap_script_per_window_state_for_migration(
                        &mut active,
                        window,
                        allow_remap_captured_drag,
                    );
                }
                self.active_scripts.insert(window, active);
            } else {
                tracing::debug!(
                    target: "ui_diag_script",
                    "script migrate requested but no active script found"
                );
            }
        } else {
            let step_window_target = match step_window_target {
                Some(UiWindowTargetV1::Current) => Some("current"),
                Some(UiWindowTargetV1::FirstSeen) => Some("first_seen"),
                Some(UiWindowTargetV1::FirstSeenOther) => Some("first_seen_other"),
                Some(UiWindowTargetV1::LastSeen) => Some("last_seen"),
                Some(UiWindowTargetV1::LastSeenOther) => Some("last_seen_other"),
                Some(UiWindowTargetV1::WindowFfi { .. }) => Some("window_ffi"),
                None => None,
            };
            tracing::debug!(
                target: "ui_diag_script",
                from_window_ffi = other_window.data().as_ffi(),
                to_window_ffi = window.data().as_ffi(),
                preferred_window_ffi = preferred.map(|w| w.data().as_ffi()),
                dock_drag_source_window_ffi = dock_drag_source_window.map(|w| w.data().as_ffi()),
                allow_migrate_for_unobserved_preferred,
                allow_migrate_for_dock_drag,
                step_index = other_active.next_step as u32,
                step_window_target,
                "script migration skipped"
            );

            if let Some(preferred) = preferred
                && preferred != window
            {
                // Wake the preferred window even if it is fully occluded; window-targeted steps
                // can legitimately need semantics snapshots from a "behind" window during
                // cross-window drags.
                app.request_redraw(preferred);
                app.push_effect(Effect::Redraw(preferred));
                app.push_effect(Effect::RequestAnimationFrame(preferred));
                append_diag_script_migration_trace(
                    &self.cfg.out_dir,
                    &format!(
                        "unix_ms={} kind=nudge_preferred preferred={:?} requested_from={:?} step_index={} step_window_target={:?}",
                        unix_ms_now(),
                        preferred,
                        window,
                        other_step_index,
                        step_window_target,
                    ),
                );
            }
        }
    }

    pub(super) fn script_output_for_non_active_window(
        &mut self,
        app: &mut App,
        devtools_request_redraw: bool,
    ) -> UiScriptFrameOutput {
        let mut output = UiScriptFrameOutput::default();
        output.request_redraw =
            self.cfg.script_keepalive || devtools_request_redraw || !self.active_scripts.is_empty();

        let heartbeat = if self.active_scripts.len() == 1 {
            self.active_scripts
                .iter()
                .next()
                .and_then(|(&active_window, active)| {
                    let now_unix_ms = unix_ms_now();
                    let should_write = active.last_reported_unix_ms == 0
                        || now_unix_ms.saturating_sub(active.last_reported_unix_ms) >= 1_000;
                    should_write.then_some((
                        active_window,
                        active.run_id,
                        active.next_step.min(u32::MAX as usize) as u32,
                        now_unix_ms,
                    ))
                })
        } else {
            None
        };

        if let Some((active_window, run_id, step_index, now_unix_ms)) = heartbeat {
            // Keep the active window producing frames even if it is occluded behind another window.
            // Multi-window drags can starve the "under" window of redraw callbacks while scripted
            // steps are still window-targeted (e.g. selector resolution in the main window).
            app.request_redraw(active_window);
            app.push_effect(Effect::Redraw(active_window));
            app.push_effect(Effect::RequestAnimationFrame(active_window));

            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id,
                updated_unix_ms: now_unix_ms,
                window: Some(active_window.data().as_ffi()),
                stage: UiScriptStageV1::Running,
                step_index: Some(step_index),
                reason_code: None,
                reason: None,
                evidence: None,
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
                last_bundle_artifact: self.last_dump_artifact_stats.clone(),
            });
            if let Some(active) = self.active_scripts.get_mut(&active_window) {
                active.last_reported_unix_ms = now_unix_ms;
            }
        }

        if !self.active_scripts.is_empty() {
            let windows: Vec<AppWindowId> = self.active_scripts.keys().copied().collect();
            for other in windows {
                // Prefer a direct redraw request here. Some platforms/backends can treat
                // "script keepalive" effects as best-effort, which risks starving the
                // window that owns the active script (and therefore preventing wait timeouts
                // from ever elapsing).
                app.request_redraw(other);
                output.effects.push(Effect::Redraw(other));
                output.effects.push(Effect::RequestAnimationFrame(other));
            }
        }

        output
    }

    pub(super) fn maybe_write_running_heartbeat_for_active_window(
        &mut self,
        window: AppWindowId,
        active: &mut ActiveScript,
    ) {
        let now_unix_ms = unix_ms_now();
        if active.last_reported_unix_ms == 0
            || now_unix_ms.saturating_sub(active.last_reported_unix_ms) >= 1_000
        {
            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: now_unix_ms,
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Running,
                step_index: Some(active.next_step.min(u32::MAX as usize) as u32),
                reason_code: None,
                reason: None,
                evidence: None,
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
                last_bundle_artifact: self.last_dump_artifact_stats.clone(),
            });
            active.last_reported_unix_ms = now_unix_ms;
        }
    }

    pub(super) fn maybe_write_running_progress_for_active_window(
        &mut self,
        window: AppWindowId,
        active: &mut ActiveScript,
    ) {
        let now_unix_ms = unix_ms_now();
        let should_report_progress = active.last_reported_step != Some(active.next_step)
            || active.last_reported_unix_ms == 0
            || now_unix_ms.saturating_sub(active.last_reported_unix_ms) >= 1_000;
        if should_report_progress {
            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: now_unix_ms,
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Running,
                step_index: Some(active.next_step.min(u32::MAX as usize) as u32),
                reason_code: None,
                reason: None,
                evidence: None,
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
                last_bundle_artifact: self.last_dump_artifact_stats.clone(),
            });
            active.last_reported_step = Some(active.next_step);
            active.last_reported_unix_ms = now_unix_ms;
        }
    }

    pub(super) fn maybe_cancel_pending_cross_window_drag(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        active: &mut ActiveScript,
    ) {
        let Some(mut pending) = active.pending_cancel_cross_window_drag.take() else {
            return;
        };

        pending.remaining_frames = pending.remaining_frames.saturating_sub(1);
        if pending.remaining_frames > 0 {
            active.pending_cancel_cross_window_drag = Some(pending);
            return;
        }

        let pointer_id = pending.pointer_id;
        let step_index = active.next_step.min(u32::MAX as usize) as u32;
        let mut canceled_any = false;

        if let Some(drag) = app.drag(pointer_id) {
            if drag.cross_window_hover
                || drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS
            {
                push_script_event_log(
                    active,
                    &self.cfg,
                    UiScriptEventLogEntryV1 {
                        unix_ms: unix_ms_now(),
                        kind: "diag.cancel_drag".to_string(),
                        step_index: Some(step_index),
                        note: Some(format!(
                            "pointer_id={} kind={:?} cross_window_hover={}",
                            pointer_id.0, drag.kind, drag.cross_window_hover
                        )),
                        bundle_dir: None,
                        window: Some(window.data().as_ffi()),
                        tick_id: Some(app.tick_id().0),
                        frame_id: Some(app.frame_id().0),
                        window_snapshot_seq: None,
                    },
                );
                app.cancel_drag(pointer_id);
                canceled_any = true;
            }
        }

        if !canceled_any {
            // Fallback: cancel any active dock drags. Some scripted sequences migrate across
            // windows while a captured-pointer gesture is active; if the release is delivered
            // to a different window, the original drag session can remain stuck. Prefer
            // deterministically clearing dock drag state over hanging the suite.
            let mut canceled: Vec<PointerId> = Vec::new();
            while let Some(id) = app.find_drag_pointer_id(|d| {
                d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                    || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS
            }) {
                app.cancel_drag(id);
                canceled.push(id);
            }

            push_script_event_log(
                active,
                &self.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: if canceled.is_empty() {
                        "diag.cancel_drag.skip".to_string()
                    } else {
                        "diag.cancel_drag.fallback".to_string()
                    },
                    step_index: Some(step_index),
                    note: Some(format!(
                        "pointer_id={} drag_present={} canceled={:?}",
                        pointer_id.0,
                        app.drag(pointer_id).is_some(),
                        canceled.iter().map(|id| id.0).collect::<Vec<_>>(),
                    )),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );

            canceled_any = !canceled.is_empty();
        }

        // Retry cancellation for a bounded number of frames. Some runners update drag session
        // state after input dispatch; a one-shot cancel can miss the window where the drag
        // becomes visible.
        if canceled_any {
            // Keep cleared.
        } else {
            pending.remaining_frames = pending.remaining_frames.saturating_sub(1);
            if pending.remaining_frames > 0 {
                active.pending_cancel_cross_window_drag = Some(pending);
            }
        }
    }

    pub(super) fn note_step_start_and_scope_evidence(
        &self,
        app: &App,
        window: AppWindowId,
        step_index: usize,
        step: &UiActionStepV2,
        active: &mut ActiveScript,
    ) -> (u32, String) {
        // Keep evidence scoped to the active step so failures remain focused.
        let step_index_u32 = step_index.min(u32::MAX as usize) as u32;
        let step_kind = script_step_kind_name(step).to_string();
        if active.event_log_active_step != Some(step_index_u32) {
            push_script_event_log(
                active,
                &self.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "step_start".to_string(),
                    step_index: Some(step_index_u32),
                    note: Some(step_kind.clone()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            active.event_log_active_step = Some(step_index_u32);
        }

        active
            .selector_resolution_trace
            .retain(|e| e.step_index == step_index_u32);
        active
            .hit_test_trace
            .retain(|e| e.step_index == step_index_u32);
        active
            .click_stable_trace
            .retain(|e| e.step_index == step_index_u32);
        active
            .bounds_stable_trace
            .retain(|e| e.step_index == step_index_u32);
        active
            .focus_trace
            .retain(|e| e.step_index == step_index_u32);
        active
            .web_ime_trace
            .retain(|e| e.step_index == step_index_u32);
        active
            .ime_event_trace
            .retain(|e| e.step_index == step_index_u32);

        (step_index_u32, step_kind)
    }

    pub(super) fn reset_active_script_state_for_step(
        active: &mut ActiveScript,
        step: &UiActionStepV2,
    ) {
        let is_v2_intent_step = matches!(
            step,
            UiActionStepV2::ClickStable { .. }
                | UiActionStepV2::ClickSelectableTextSpanStable { .. }
                | UiActionStepV2::WaitBoundsStable { .. }
                | UiActionStepV2::EnsureVisible { .. }
                | UiActionStepV2::ScrollIntoView { .. }
                | UiActionStepV2::TypeTextInto { .. }
                | UiActionStepV2::PasteTextInto { .. }
                | UiActionStepV2::MenuSelect { .. }
                | UiActionStepV2::MenuSelectPath { .. }
                | UiActionStepV2::DragPointer { .. }
                | UiActionStepV2::DragPointerUntil { .. }
                | UiActionStepV2::DragTo { .. }
                | UiActionStepV2::SetSliderValue { .. }
                | UiActionStepV2::PointerMove { .. }
                | UiActionStepV2::MovePointerSweep { .. }
                | UiActionStepV2::AssertClipboardText { .. }
        );
        if !is_v2_intent_step {
            active.v2_step_state = None;
        }
        if !matches!(step, UiActionStepV2::WaitShortcutRoutingTrace { .. }) {
            active.wait_shortcut_routing_trace = None;
        }
        if !matches!(step, UiActionStepV2::WaitOverlayPlacementTrace { .. }) {
            active.wait_overlay_placement_trace = None;
        }
    }
}
