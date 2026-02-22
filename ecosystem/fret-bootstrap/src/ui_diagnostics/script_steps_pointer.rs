use super::*;

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
    output.events.push(wheel_event(pos, delta_x, delta_y));

    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-wheel"));
    }

    false
}
