use super::*;

impl UiDiagnosticsService {
    pub(super) fn maybe_start_pending_script(&mut self, app: &mut App, window: AppWindowId) {
        if !self.active_scripts.is_empty() {
            return;
        }

        let Some(script) = self.pending_script.take() else {
            return;
        };

        let run_id = self.pending_script_run_id.take().unwrap_or(0);
        // Prefer a deterministic anchor window when starting a script. The trigger touch can
        // be observed by any window, and multi-window apps may produce frames in a different
        // order across runs. Using the smallest observed window key (best-effort "first
        // created") keeps `first_seen` window targets stable for scripts.
        let anchor_window = self
            .known_windows
            .iter()
            .copied()
            .min_by_key(|w| w.data().as_ffi())
            .unwrap_or(window);

        let mut active_script = ActiveScript {
            steps: script.steps,
            run_id,
            anchor_window,
            next_step: 0,
            event_log: Vec::new(),
            event_log_dropped: 0,
            event_log_active_step: None,
            last_injected_step: None,
            wait_frames_remaining: 0,
            wait_until: None,
            wait_shortcut_routing_trace: None,
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
            overlay_placement_trace: Vec::new(),
            web_ime_trace: Vec::new(),
            ime_event_trace: Vec::new(),
        };

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
        app: &App,
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
        let allow_migrate_for_dock_drag = dock_drag_source_window == Some(window);

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
            let step_window_target = match Self::active_step_window_target(other_active) {
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
        }
    }
}
