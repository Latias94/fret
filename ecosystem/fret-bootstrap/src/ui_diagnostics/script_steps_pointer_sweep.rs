use super::*;

pub(super) fn handle_move_pointer_sweep_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
) -> bool {
    let UiActionStepV2::MovePointerSweep {
        window: _,
        pointer_kind,
        target,
        delta_x,
        delta_y,
        steps,
        frames_per_step,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-move_pointer_sweep-no-semantics");
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

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::MovePointerSweep(state)) if state.step_index == step_index => state,
        _ => {
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
                    format!("script-step-{step_index:04}-move_pointer_sweep-no-semantics-match");
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
                        note: Some("move_pointer_sweep_no_semantics_match".to_string()),
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
                    reason: Some("move_pointer_sweep_no_semantics_match".to_string()),
                    evidence: script_evidence_for_active(active),
                    last_bundle_dir: svc
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&svc.cfg.out_dir, p)),
                    last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
                });
                return true;
            };

            let start = center_of_rect(node.bounds);
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
                    Some("move_pointer_sweep.start"),
                    svc.cfg.max_debug_string_bytes,
                );
            }
            let end = Point::new(
                fret_core::Px(start.x.0 + delta_x),
                fret_core::Px(start.y.0 + delta_y),
            );
            V2MovePointerSweepState {
                step_index,
                start,
                end,
                steps: steps.max(1),
                next_step: 0,
                frames_per_step: frames_per_step.max(1),
                wait_frames_remaining: 0,
            }
        }
    };

    if state.wait_frames_remaining > 0 {
        state.wait_frames_remaining = state.wait_frames_remaining.saturating_sub(1);
        active.v2_step_state = Some(V2StepState::MovePointerSweep(state));
        output.request_redraw = true;
    } else if state.next_step > state.steps {
        if let Some(ui) = ui {
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
                Some("move_pointer_sweep.end"),
                svc.cfg.max_debug_string_bytes,
            );
        }
        active.v2_step_state = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-move_pointer_sweep"));
        }
    } else {
        let t = state.next_step as f32 / state.steps as f32;
        let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
        let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
        let position = Point::new(fret_core::Px(x), fret_core::Px(y));
        output
            .events
            .push(move_pointer_event(position, pointer_type));

        state.next_step = state.next_step.saturating_add(1);
        state.wait_frames_remaining = state.frames_per_step.saturating_sub(1);
        active.v2_step_state = Some(V2StepState::MovePointerSweep(state));
        output.request_redraw = true;
    }

    false
}
