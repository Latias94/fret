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
}
