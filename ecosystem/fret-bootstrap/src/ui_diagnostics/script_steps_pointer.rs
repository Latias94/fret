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

fn selectable_text_interactive_span_bounds_for_semantics_node(
    element_runtime: Option<&ElementRuntime>,
    ui: Option<&UiTree<App>>,
    window: AppWindowId,
    node: NodeId,
) -> Option<Vec<fret_ui::element::SelectableTextInteractiveSpanBounds>> {
    let runtime = element_runtime?;
    runtime
        .selectable_text_interactive_span_bounds_for_node(window, node)
        .or_else(|| {
            let element = ui.and_then(|ui| ui.debug_node_element(node))?;
            runtime.selectable_text_interactive_span_bounds_for_element(window, element)
        })
}

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
    ui: Option<&mut UiTree<App>>,
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

        let mut pointer_trace_note = String::from("click");
        let pos = if let Some(ui_ref) = ui.as_deref() {
            let resolution = pointer_target_resolution_prefer_intended_hit(
                app,
                snapshot,
                element_runtime,
                ui_ref,
                window,
                node,
                window_bounds,
            );
            pointer_trace_note = pointer_target_resolution_trace_note("click", resolution);
            resolution.position
        } else {
            center_of_rect_clamped_to_rect(
                interaction_bounds_for_semantics_node(element_runtime, None, window, node),
                window_bounds,
            )
        };
        if let Some(ui) = ui {
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
                Some(pointer_trace_note.as_str()),
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
        let injected_step_index = step_index.min(u32::MAX as usize) as u32;
        active.last_injected_step = Some(injected_step_index);
        active.last_injected_pointer_source_step = Some(injected_step_index);
        active.last_injected_pointer_source_test_id = match &target {
            UiSelectorV1::TestId { id, .. } => Some(id.clone()),
            _ => node.test_id.clone(),
        };

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

pub(super) fn handle_save_pointer_point_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::SavePointerPoint {
        window: target_window,
        name,
        target,
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
            "script-step-{step_index:04}-save_pointer_point-window-not-found"
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
            let label = format!("script-step-{step_index:04}-save_pointer_point-no-semantics");
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
                    note: Some("save_pointer_point_no_semantics_snapshot".to_string()),
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
                reason: Some("save_pointer_point_no_semantics_snapshot".to_string()),
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
                format!("script-step-{step_index:04}-save_pointer_point-no-semantics-match");
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
                    note: Some("save_pointer_point_no_semantics_match".to_string()),
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
                reason: Some("save_pointer_point_no_semantics_match".to_string()),
                evidence: script_evidence_for_active(active),
                last_bundle_dir: svc
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&svc.cfg.out_dir, p)),
                last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
            });
            return true;
        };

        let mut pointer_trace_note = String::from("save_pointer_point");
        let position = if let Some(ui_ref) = ui.as_deref() {
            let resolution = pointer_target_resolution_prefer_intended_hit(
                app,
                snapshot,
                element_runtime,
                ui_ref,
                window,
                node,
                window_bounds,
            );
            pointer_trace_note =
                pointer_target_resolution_trace_note("save_pointer_point", resolution);
            resolution.position
        } else {
            center_of_rect_clamped_to_rect(
                interaction_bounds_for_semantics_node(element_runtime, None, window, node),
                window_bounds,
            )
        };
        if let Some(ui) = ui {
            record_hit_test_trace_for_selector(
                &mut active.hit_test_trace,
                ui,
                element_runtime,
                window,
                Some(snapshot),
                &target,
                step_index as u32,
                position,
                Some(node),
                Some(pointer_trace_note.as_str()),
                svc.cfg.max_debug_string_bytes,
            );
        }

        let source_test_id = match &target {
            UiSelectorV1::TestId { id, .. } => Some(id.clone()),
            _ => node.test_id.clone(),
        };
        active.saved_pointer_points.insert(
            name.clone(),
            SavedPointerPoint {
                window,
                position,
                selector: target,
                source_test_id,
            },
        );
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "saved_pointer_point".to_string(),
                step_index: Some(step_index as u32),
                note: Some(format!(
                    "name={name} window={} x={:.1} y={:.1}",
                    window.data().as_ffi(),
                    position.x.0,
                    position.y.0
                )),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );

        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
    }

    false
}

pub(super) fn handle_click_saved_point_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::ClickSavedPoint {
        window: target_window,
        name,
        pointer_kind,
        button,
        click_count,
        modifiers,
    } = step
    else {
        return false;
    };

    let Some(saved) = active.saved_pointer_points.get(&name).cloned() else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-click_saved_point-missing-{name}");
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
                note: Some(format!("click_saved_point_missing name={name}")),
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
            reason_code: Some("saved_pointer_point.missing".to_string()),
            reason: Some(format!("click_saved_point_missing name={name}")),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
        return true;
    };

    let requested_window = if let Some(target_window) = target_window.as_ref() {
        svc.resolve_window_target_for_active_step(window, anchor_window, Some(target_window))
    } else {
        Some(saved.window)
    };
    let Some(target_window) = requested_window else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-click_saved_point-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
        return false;
    };
    if target_window != saved.window {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-click_saved_point-window-mismatch"
        ));
        *stop_script = true;
        *failure_reason = Some("saved_pointer_point_window_mismatch".to_string());
        output.request_redraw = true;
        return false;
    }
    if target_window != window {
        *handoff_to = Some(target_window);
        output
            .effects
            .push(Effect::RequestAnimationFrame(target_window));
        output.request_redraw = true;
        return false;
    }

    if let Some(ui) = ui {
        record_hit_test_trace_for_selector(
            &mut active.hit_test_trace,
            ui,
            element_runtime,
            window,
            semantics_snapshot,
            &saved.selector,
            step_index as u32,
            saved.position,
            None,
            Some("click_saved_point"),
            svc.cfg.max_debug_string_bytes,
        );
    }
    record_overlay_placement_trace(
        &mut active.overlay_placement_trace,
        element_runtime,
        semantics_snapshot,
        window,
        step_index as u32,
        "click_saved_point",
    );

    let modifiers = core_modifiers_from_ui(modifiers);
    let pointer_type = pointer_type_from_kind(pointer_kind);
    output.events.extend(click_events_with_modifiers(
        saved.position,
        button,
        click_count,
        modifiers,
        pointer_type,
    ));
    let injected_step_index = step_index.min(u32::MAX as usize) as u32;
    active.last_injected_step = Some(injected_step_index);
    active.last_injected_pointer_source_step = Some(injected_step_index);
    active.last_injected_pointer_source_test_id = saved.source_test_id.clone();

    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-click_saved_point"));
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
    ui: Option<&mut UiTree<App>>,
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

        let pos = if let Some(ui_ref) = ui.as_deref() {
            pointer_position_prefer_intended_hit(
                app,
                snapshot,
                element_runtime,
                ui_ref,
                window,
                node,
                window_bounds,
            )
        } else {
            center_of_rect_clamped_to_rect(
                interaction_bounds_for_semantics_node(element_runtime, None, window, node),
                window_bounds,
            )
        };
        if let Some(ui) = ui {
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
        let injected_step_index = step_index.min(u32::MAX as usize) as u32;
        active.last_injected_step = Some(injected_step_index);
        active.last_injected_pointer_source_step = Some(injected_step_index);
        active.last_injected_pointer_source_test_id = match &target {
            UiSelectorV1::TestId { id, .. } => Some(id.clone()),
            _ => node.test_id.clone(),
        };

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
    ui: Option<&mut UiTree<App>>,
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

    let pos = if let Some(ui_ref) = ui.as_deref() {
        pointer_position_prefer_intended_hit(
            app,
            snapshot,
            element_runtime,
            ui_ref,
            window,
            node,
            window_bounds,
        )
    } else {
        center_of_rect_clamped_to_rect(
            interaction_bounds_for_semantics_node(element_runtime, None, window, node),
            window_bounds,
        )
    };
    if let Some(ui) = ui {
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
    ui: Option<&mut UiTree<App>>,
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

    let start = if let Some(ui_ref) = ui.as_deref() {
        pointer_position_prefer_intended_hit(
            app,
            snapshot,
            element_runtime,
            ui_ref,
            window,
            node,
            window_bounds,
        )
    } else {
        center_of_rect_clamped_to_rect(
            interaction_bounds_for_semantics_node(element_runtime, None, window, node),
            window_bounds,
        )
    };
    let end = clamp_point_to_rect(
        Point::new(
            fret_core::Px(start.x.0 + delta_x),
            fret_core::Px(start.y.0 + delta_y),
        ),
        window_bounds,
    );

    if let Some(ui) = ui {
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
    ui: Option<&mut UiTree<App>>,
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

        let pos = if let Some(ui_ref) = ui.as_deref() {
            pointer_position_prefer_intended_hit(
                app,
                snapshot,
                element_runtime,
                ui_ref,
                window,
                node,
                window_bounds,
            )
        } else {
            center_of_rect_clamped_to_rect(
                interaction_bounds_for_semantics_node(element_runtime, None, window, node),
                window_bounds,
            )
        };
        if let Some(ui) = ui {
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
        enum ResolvedClickStableTarget<'a> {
            Semantics(&'a fret_core::SemanticsNode),
            CachedTestId { id: String, bounds: Rect },
        }

        let resolved = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &target,
            active.scope_root_for_window(window),
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        )
        .map(ResolvedClickStableTarget::Semantics)
        .or_else(|| {
            let UiSelectorV1::TestId { id, .. } = &target else {
                return None;
            };
            svc.per_window
                .get(&window)
                .and_then(|ring| ring.test_id_bounds.get(id).copied())
                .map(|bounds| ResolvedClickStableTarget::CachedTestId {
                    id: id.clone(),
                    bounds,
                })
        });

        if let Some(resolved) = resolved {
            let stable_required = stable_frames.max(1);
            let max_move_px = max_move_px.max(0.0);

            if timeout_frames != 0 && stable_required > timeout_frames {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-click_stable-impossible-stable-frames-gt-timeout"
                ));
                *stop_script = true;
                *failure_reason =
                    Some("click_stable_impossible_stable_frames_gt_timeout_frames".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
                return true;
            }

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

            let center = match &resolved {
                ResolvedClickStableTarget::Semantics(node) => {
                    if let Some(ui) = ui.as_deref_mut() {
                        pointer_position_prefer_intended_hit_routing(
                            app,
                            snapshot,
                            element_runtime,
                            ui,
                            window,
                            node,
                            window_bounds,
                        )
                    } else {
                        center_of_rect_clamped_to_rect(
                            interaction_bounds_for_semantics_node(
                                element_runtime,
                                None,
                                window,
                                node,
                            ),
                            window_bounds,
                        )
                    }
                }
                ResolvedClickStableTarget::CachedTestId { bounds, .. } => {
                    center_of_rect_clamped_to_rect(*bounds, window_bounds)
                }
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
                        center,
                        match &resolved {
                            ResolvedClickStableTarget::Semantics(node) => Some(*node),
                            ResolvedClickStableTarget::CachedTestId { .. } => None,
                        },
                        Some("click_stable.timeout"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                if let Some(note) = click_stable_timeout_hit_test_note_with_window(
                    &active.hit_test_trace,
                    step_index as u32,
                    Some(window_bounds),
                ) {
                    push_script_event_log(
                        active,
                        &svc.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "click_stable.timeout_hit_test".to_string(),
                            step_index: Some(step_index.min(u32::MAX as usize) as u32),
                            note: Some(note),
                            bundle_dir: None,
                            window: Some(window.data().as_ffi()),
                            tick_id: Some(app.tick_id().0),
                            frame_id: Some(app.frame_id().0),
                            window_snapshot_seq: None,
                        },
                    );
                }
                *force_dump_label =
                    Some(format!("script-step-{step_index:04}-click_stable-timeout"));
                *stop_script = true;
                let timeout_reason = match click_stable_timeout_target_window_relation(
                    &active.hit_test_trace,
                    step_index as u32,
                    window_bounds,
                ) {
                    Some("outside_window") => "click_stable_timeout_target_outside_window",
                    _ => "click_stable_timeout",
                };
                *failure_reason = Some(timeout_reason.to_string());
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
                    if let Some(ui) = ui {
                        let mut hit = build_hit_test_trace_entry_for_selector(
                            ui,
                            element_runtime,
                            window,
                            Some(snapshot),
                            &target,
                            step_index as u32,
                            center,
                            match &resolved {
                                ResolvedClickStableTarget::Semantics(node) => Some(*node),
                                ResolvedClickStableTarget::CachedTestId { .. } => None,
                            },
                            Some("click_stable.probe"),
                            svc.cfg.max_debug_string_bytes,
                        );
                        let ok = match &resolved {
                            ResolvedClickStableTarget::Semantics(_) => {
                                hit.includes_intended == Some(true)
                                    || hit.hit_path_contains_intended == Some(true)
                            }
                            ResolvedClickStableTarget::CachedTestId { id, .. } => {
                                hit.hit_semantics_test_id.as_deref() == Some(id.as_str())
                            }
                        };
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
                            // `click_stable` only stabilizes the target geometry. Once the
                            // current frame already hit-tests to the intended node, dispatch
                            // the same full click sequence as plain `click` in one frame.
                            // Splitting hover and down/up across frames is unsafe for
                            // transient overlay content that can rebuild on hover.
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
                            let injected_step_index = step_index.min(u32::MAX as usize) as u32;
                            active.last_injected_step = Some(injected_step_index);
                            active.last_injected_pointer_source_step = Some(injected_step_index);
                            active.last_injected_pointer_source_test_id = match &target {
                                UiSelectorV1::TestId { id, .. } => Some(id.clone()),
                                _ => match &resolved {
                                    ResolvedClickStableTarget::Semantics(node) => {
                                        node.test_id.clone()
                                    }
                                    ResolvedClickStableTarget::CachedTestId { id, .. } => {
                                        Some(id.clone())
                                    }
                                },
                            };
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
                        let injected_step_index = step_index.min(u32::MAX as usize) as u32;
                        active.last_injected_step = Some(injected_step_index);
                        active.last_injected_pointer_source_step = Some(injected_step_index);
                        active.last_injected_pointer_source_test_id = match &target {
                            UiSelectorV1::TestId { id, .. } => Some(id.clone()),
                            _ => match &resolved {
                                ResolvedClickStableTarget::Semantics(node) => node.test_id.clone(),
                                ResolvedClickStableTarget::CachedTestId { id, .. } => {
                                    Some(id.clone())
                                }
                            },
                        };
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

pub(super) fn click_stable_timeout_hit_test_note(
    trace: &[UiHitTestTraceEntryV1],
    step_index: u32,
) -> Option<String> {
    click_stable_timeout_hit_test_note_with_window(trace, step_index, None)
}

fn click_stable_timeout_hit_test_note_with_window(
    trace: &[UiHitTestTraceEntryV1],
    step_index: u32,
    window_bounds: Option<Rect>,
) -> Option<String> {
    let hit = trace
        .iter()
        .rev()
        .find(|entry| entry.step_index == step_index)?;

    let geometry_note = click_stable_timeout_geometry_note(hit, window_bounds);

    Some(format!(
        "hit_test={} position={} intended_node_id={:?} intended_test_id={:?} intended_bounds={}{} hit_node_id={:?} hit_semantics_node_id={:?} hit_semantics_test_id={:?} includes_intended={:?} hit_path_contains_intended={:?} blocking_reason={:?} blocking_root={:?} blocking_layer_id={:?} routing_explain={:?} barrier_root={:?} focus_barrier_root={:?} pointer_occlusion={:?} pointer_occlusion_layer_id={:?} pointer_occlusion_test_id={:?} pointer_capture_active={:?} pointer_capture_layer_id={:?} pointer_capture_test_id={:?} note={:?}",
        selector_debug_summary(&hit.selector),
        point_summary(hit.position),
        hit.intended_node_id,
        hit.intended_test_id.as_deref(),
        hit.intended_bounds
            .as_ref()
            .map(ui_rect_summary)
            .unwrap_or_else(|| "<none>".to_string()),
        geometry_note,
        hit.hit_node_id,
        hit.hit_semantics_node_id,
        hit.hit_semantics_test_id.as_deref(),
        hit.includes_intended,
        hit.hit_path_contains_intended,
        hit.blocking_reason.as_deref(),
        hit.blocking_root,
        hit.blocking_layer_id,
        hit.routing_explain.as_deref(),
        hit.barrier_root,
        hit.focus_barrier_root,
        hit.pointer_occlusion.as_deref(),
        hit.pointer_occlusion_layer_id,
        hit.pointer_occlusion_test_id.as_deref(),
        hit.pointer_capture_active,
        hit.pointer_capture_layer_id,
        hit.pointer_capture_test_id.as_deref(),
        hit.note.as_deref()
    ))
}

fn click_stable_timeout_target_window_relation(
    trace: &[UiHitTestTraceEntryV1],
    step_index: u32,
    window_bounds: Rect,
) -> Option<&'static str> {
    let hit = trace
        .iter()
        .rev()
        .find(|entry| entry.step_index == step_index)?;
    hit.intended_bounds
        .as_ref()
        .map(|bounds| ui_rect_window_relation(bounds, window_bounds))
}

fn click_stable_timeout_geometry_note(
    hit: &UiHitTestTraceEntryV1,
    window_bounds: Option<Rect>,
) -> String {
    let Some(window_bounds) = window_bounds else {
        return String::new();
    };
    let Some(bounds) = hit.intended_bounds.as_ref() else {
        return format!(" window_bounds={}", rect_summary(window_bounds));
    };
    format!(
        " target_window_relation={} window_bounds={}",
        ui_rect_window_relation(bounds, window_bounds),
        rect_summary(window_bounds)
    )
}

fn ui_rect_window_relation(rect: &UiRectV1, window: Rect) -> &'static str {
    let rx0 = rect.x_px;
    let ry0 = rect.y_px;
    let rx1 = rx0 + rect.w_px.max(0.0);
    let ry1 = ry0 + rect.h_px.max(0.0);
    let wx0 = window.origin.x.0;
    let wy0 = window.origin.y.0;
    let wx1 = wx0 + window.size.width.0.max(0.0);
    let wy1 = wy0 + window.size.height.0.max(0.0);

    if rect.w_px <= 0.0 || rect.h_px <= 0.0 || wx1 <= wx0 || wy1 <= wy0 {
        return "empty_or_degenerate";
    }
    if rx1 <= wx0 || rx0 >= wx1 || ry1 <= wy0 || ry0 >= wy1 {
        return "outside_window";
    }
    if rx0 >= wx0 && ry0 >= wy0 && rx1 <= wx1 && ry1 <= wy1 {
        return "within_window";
    }
    "partially_outside_window"
}

fn ui_rect_summary(rect: &UiRectV1) -> String {
    format!(
        "x={} y={} w={} h={}",
        rect.x_px, rect.y_px, rect.w_px, rect.h_px
    )
}

fn rect_summary(rect: Rect) -> String {
    format!(
        "x={} y={} w={} h={}",
        rect.origin.x.0, rect.origin.y.0, rect.size.width.0, rect.size.height.0
    )
}

pub(super) fn selectable_span_timeout_hit_test_note(
    trace: &[UiHitTestTraceEntryV1],
    step_index: u32,
    tag: &str,
    last_lookup_state: Option<&str>,
    stable_count: u32,
    stable_required: u32,
    remaining_frames: u32,
) -> Option<String> {
    let hit_note = click_stable_timeout_hit_test_note(trace, step_index)?;
    Some(format!(
        "tag={:?} last_lookup_state={:?} stable_count={} stable_required={} remaining_frames={} {}",
        tag, last_lookup_state, stable_count, stable_required, remaining_frames, hit_note
    ))
}

fn selector_debug_summary(selector: &UiSelectorV1) -> String {
    match selector {
        UiSelectorV1::TestId { id, root_z_index } => {
            format!("test_id={id:?} root_z_index={root_z_index:?}")
        }
        UiSelectorV1::RoleAndName { role, name, .. } => {
            format!("role={role:?} name={name:?}")
        }
        UiSelectorV1::GlobalElementId {
            element,
            root_z_index,
        } => {
            format!("global_element_id={element} root_z_index={root_z_index:?}")
        }
        other => format!("{other:?}"),
    }
}

fn point_summary(point: UiPointV1) -> String {
    format!("x={} y={}", point.x_px, point.y_px)
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    fn hit_trace(step_index: u32) -> UiHitTestTraceEntryV1 {
        UiHitTestTraceEntryV1 {
            step_index,
            selector: UiSelectorV1::TestId {
                id: "button.primary".to_string(),
                root_z_index: Some(3),
            },
            position: UiPointV1 {
                x_px: 42.0,
                y_px: 24.0,
            },
            intended_node_id: Some(10),
            intended_test_id: Some("button.primary".to_string()),
            intended_bounds: Some(UiRectV1 {
                x_px: 10.0,
                y_px: 8.0,
                w_px: 64.0,
                h_px: 32.0,
            }),
            hit_node_id: Some(90),
            hit_node_path: vec![1, 5, 90],
            hit_semantics_node_id: Some(90),
            hit_semantics_test_id: Some("modal.backdrop".to_string()),
            includes_intended: Some(false),
            hit_path_contains_intended: Some(false),
            blocking_reason: Some("pointer_occlusion".to_string()),
            blocking_root: Some(88),
            blocking_layer_id: Some(7),
            routing_explain: Some(
                "pointer occlusion layer=7 root=88 blocks intended=10".to_string(),
            ),
            barrier_root: Some(88),
            focus_barrier_root: None,
            pointer_occlusion: Some("blocks_underlay_input".to_string()),
            pointer_occlusion_layer_id: Some(7),
            pointer_occlusion_node_id: Some(90),
            pointer_occlusion_test_id: Some("modal.backdrop".to_string()),
            pointer_occlusion_role: Some("panel".to_string()),
            pointer_occlusion_bounds: None,
            pointer_capture_active: Some(true),
            pointer_capture_layer_id: Some(7),
            pointer_capture_multiple_layers: Some(false),
            pointer_capture_node_id: Some(90),
            pointer_capture_test_id: Some("modal.backdrop".to_string()),
            pointer_capture_role: Some("panel".to_string()),
            pointer_capture_bounds: None,
            pointer_capture_element: Some(123),
            pointer_capture_element_path: Some("Root/Overlay/Backdrop".to_string()),
            scope_roots: vec![UiHitTestScopeRootEvidenceV1 {
                kind: "layer_root".to_string(),
                root: 88,
                layer_id: Some(7),
                pointer_occlusion: Some("blocks_underlay_input".to_string()),
                blocks_underlay_input: Some(true),
                hit_testable: Some(true),
            }],
            note: Some("click_stable.timeout".to_string()),
        }
    }

    #[test]
    fn click_stable_timeout_hit_test_note_names_blocking_evidence() {
        let note = click_stable_timeout_hit_test_note(&[hit_trace(12)], 12).unwrap();

        assert!(note.contains("test_id=\"button.primary\""));
        assert!(note.contains("intended_test_id=Some(\"button.primary\")"));
        assert!(note.contains("hit_semantics_test_id=Some(\"modal.backdrop\")"));
        assert!(note.contains("blocking_reason=Some(\"pointer_occlusion\")"));
        assert!(note.contains("blocking_layer_id=Some(7)"));
        assert!(note.contains("pointer_capture_active=Some(true)"));
        assert!(note.contains("pointer_capture_test_id=Some(\"modal.backdrop\")"));
        assert!(note.contains("routing_explain=Some(\"pointer occlusion layer=7"));
    }

    #[test]
    fn click_stable_timeout_hit_test_note_uses_latest_step_entry() {
        let old = hit_trace(11);
        let mut latest = hit_trace(12);
        latest.hit_semantics_test_id = Some("new.overlay".to_string());
        latest.pointer_capture_test_id = Some("new.overlay".to_string());

        let note = click_stable_timeout_hit_test_note(&[old, latest], 12).unwrap();

        assert!(note.contains("hit_semantics_test_id=Some(\"new.overlay\")"));
        assert!(!note.contains("step_index=11"));
    }

    #[test]
    fn selectable_span_timeout_note_adds_span_lookup_state() {
        let note = selectable_span_timeout_hit_test_note(
            &[hit_trace(12)],
            12,
            "docs-link",
            Some("empty_span_bounds"),
            1,
            2,
            0,
        )
        .unwrap();

        assert!(note.contains("tag=\"docs-link\""));
        assert!(note.contains("last_lookup_state=Some(\"empty_span_bounds\")"));
        assert!(note.contains("stable_count=1"));
        assert!(note.contains("stable_required=2"));
        assert!(note.contains("remaining_frames=0"));
        assert!(note.contains("blocking_reason=Some(\"pointer_occlusion\")"));
    }

    #[test]
    fn selectable_span_timeout_note_covers_cached_test_id_fallback() {
        let mut hit = hit_trace(21);
        hit.intended_node_id = None;
        hit.intended_test_id = Some("cached.text".to_string());
        hit.note = Some("click_selectable_text_span_stable.timeout.no_semantics_match".to_string());

        let note = selectable_span_timeout_hit_test_note(
            &[hit],
            21,
            "inline-action",
            Some("no_semantics_match"),
            0,
            1,
            0,
        )
        .unwrap();

        assert!(note.contains("tag=\"inline-action\""));
        assert!(note.contains("last_lookup_state=Some(\"no_semantics_match\")"));
        assert!(note.contains("intended_node_id=None"));
        assert!(note.contains("intended_test_id=Some(\"cached.text\")"));
        assert!(note.contains(
            "note=Some(\"click_selectable_text_span_stable.timeout.no_semantics_match\")"
        ));
    }

    #[test]
    fn click_stable_timeout_note_identifies_offscreen_targets() {
        let mut hit = hit_trace(12);
        hit.intended_bounds = Some(UiRectV1 {
            x_px: 294.0,
            y_px: 2068.0,
            w_px: 106.0,
            h_px: 36.0,
        });
        let window_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(1080.0), Px(720.0)),
        );

        let note = click_stable_timeout_hit_test_note_with_window(&[hit], 12, Some(window_bounds))
            .unwrap();

        assert!(note.contains("target_window_relation=outside_window"));
        assert!(note.contains("intended_bounds=x=294 y=2068 w=106 h=36"));
        assert_eq!(
            click_stable_timeout_target_window_relation(
                &[hit_trace(12)],
                12,
                Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    fret_core::Size::new(Px(100.0), Px(100.0)),
                ),
            ),
            Some("within_window")
        );
    }
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

            if timeout_frames != 0 && stable_required > timeout_frames {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-click_selectable_span-impossible-stable-frames-gt-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some(
                    "click_selectable_text_span_stable_impossible_stable_frames_gt_timeout_frames"
                        .to_string(),
                );
                active.v2_step_state = None;
                output.request_redraw = true;
                return true;
            }

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
                    last_lookup_state: None,
                },
            };
            if state.last_lookup_state == Some("no_semantics_match") {
                state.last_lookup_state = None;
            }

            let cached_test_id_bounds = match &target {
                UiSelectorV1::TestId { id, .. } => svc
                    .per_window
                    .get(&window)
                    .and_then(|ring| ring.test_id_bounds.get(id).copied()),
                _ => None,
            };

            if state.remaining_frames == 0 {
                if let Some(ui) = ui.as_deref_mut() {
                    let timeout_note = match state.last_lookup_state {
                        Some("no_semantics_match") => {
                            "click_selectable_text_span_stable.timeout.no_semantics_match"
                        }
                        Some("no_runtime_state") => {
                            "click_selectable_text_span_stable.timeout.no_runtime_state"
                        }
                        Some("empty_span_bounds") => {
                            "click_selectable_text_span_stable.timeout.empty_span_bounds"
                        }
                        _ => "click_selectable_text_span_stable.timeout",
                    };
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        center_of_rect_clamped_to_rect(
                            cached_test_id_bounds.unwrap_or_else(|| {
                                interaction_bounds_for_semantics_node(
                                    element_runtime,
                                    Some(&*ui),
                                    window,
                                    node,
                                )
                            }),
                            window_bounds,
                        ),
                        Some(node),
                        Some(timeout_note),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                if let Some(note) = selectable_span_timeout_hit_test_note(
                    &active.hit_test_trace,
                    step_index as u32,
                    tag.as_str(),
                    state.last_lookup_state,
                    state.stable_count,
                    stable_required,
                    state.remaining_frames,
                ) {
                    push_script_event_log(
                        active,
                        &svc.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "click_selectable_text_span_stable.timeout_hit_test".to_string(),
                            step_index: Some(step_index.min(u32::MAX as usize) as u32),
                            note: Some(note),
                            bundle_dir: None,
                            window: Some(window.data().as_ffi()),
                            tick_id: None,
                            frame_id: None,
                            window_snapshot_seq: None,
                        },
                    );
                }
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-click_selectable_span-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some(match state.last_lookup_state {
                    Some("no_semantics_match") => {
                        "click_selectable_text_span_stable_no_semantics_match_timeout".to_string()
                    }
                    Some("no_runtime_state") => {
                        "click_selectable_text_span_stable_no_runtime_state_timeout".to_string()
                    }
                    Some("empty_span_bounds") => {
                        "click_selectable_text_span_stable_empty_span_bounds_timeout".to_string()
                    }
                    _ => "click_selectable_text_span_stable_timeout".to_string(),
                });
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                let bounds_local: Option<Rect> = match selectable_text_interactive_span_bounds_for_semantics_node(
                    element_runtime,
                    ui.as_deref(),
                    window,
                    node.id,
                ) {
                    None => {
                        state.last_lookup_state = Some("no_runtime_state");
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state =
                            Some(V2StepState::ClickSelectableTextSpanStable(state.clone()));
                        output.request_redraw = true;
                        None
                    }
                    Some(spans) if spans.is_empty() => {
                        // Best-effort: span bounds are computed during `paint_all()`. If
                        // we don't see them yet, wait a few frames before failing.
                        state.last_lookup_state = Some("empty_span_bounds");
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state =
                            Some(V2StepState::ClickSelectableTextSpanStable(state.clone()));
                        output.request_redraw = true;
                        None
                    }
                    Some(spans) => spans
                        .iter()
                        .find(|span| span.tag.as_ref() == tag.as_str())
                        .map(|span| {
                            state.last_lookup_state = None;
                            span.bounds_local
                        })
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
                    last_lookup_state: None,
                },
            };

            let cached_test_id_bounds = match &target {
                UiSelectorV1::TestId { id, .. } => svc
                    .per_window
                    .get(&window)
                    .and_then(|ring| ring.test_id_bounds.get(id).copied()),
                _ => None,
            };

            if cached_test_id_bounds.is_some() && state.remaining_frames > 0 {
                state.last_lookup_state = Some("no_semantics_match");
                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                active.v2_step_state = Some(V2StepState::ClickSelectableTextSpanStable(state));
                output.request_redraw = true;
            } else if cached_test_id_bounds.is_some() && state.remaining_frames == 0 {
                if let Some(ui) = ui {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        element_runtime,
                        window,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        center_of_rect_clamped_to_rect(
                            cached_test_id_bounds.expect("checked is_some"),
                            window_bounds,
                        ),
                        None,
                        Some("click_selectable_text_span_stable.timeout.no_semantics_match"),
                        svc.cfg.max_debug_string_bytes,
                    );
                }
                if let Some(note) = selectable_span_timeout_hit_test_note(
                    &active.hit_test_trace,
                    step_index as u32,
                    tag.as_str(),
                    Some("no_semantics_match"),
                    state.stable_count,
                    stable_frames.max(1),
                    state.remaining_frames,
                ) {
                    push_script_event_log(
                        active,
                        &svc.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "click_selectable_text_span_stable.timeout_hit_test".to_string(),
                            step_index: Some(step_index.min(u32::MAX as usize) as u32),
                            note: Some(note),
                            bundle_dir: None,
                            window: Some(window.data().as_ffi()),
                            tick_id: None,
                            frame_id: None,
                            window_snapshot_seq: None,
                        },
                    );
                }
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-click_selectable_span-timeout"
                ));
                *stop_script = true;
                *failure_reason = Some(
                    "click_selectable_text_span_stable_no_semantics_match_timeout".to_string(),
                );
                active.v2_step_state = None;
                output.request_redraw = true;
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
    ui: Option<&mut UiTree<App>>,
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

    let pos = if let Some(ui_ref) = ui.as_deref() {
        pointer_position_prefer_intended_hit(
            app,
            snapshot,
            element_runtime,
            ui_ref,
            window,
            node,
            window_bounds,
        )
    } else {
        center_of_rect_clamped_to_rect(
            interaction_bounds_for_semantics_node(element_runtime, None, window, node),
            window_bounds,
        )
    };
    if let Some(ui) = ui {
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

pub(super) fn handle_wheel_burst_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
) -> bool {
    let UiActionStepV2::WheelBurst {
        window: _,
        pointer_kind,
        target,
        delta_x,
        delta_y,
        count,
    } = step
    else {
        return false;
    };

    let Some(snapshot) = semantics_snapshot else {
        output.request_redraw = true;
        let label = format!("script-step-{step_index:04}-wheel_burst-no-semantics");
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
        let label = format!("script-step-{step_index:04}-wheel_burst-no-semantics-match");
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
                note: Some("wheel_burst_no_semantics_match".to_string()),
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
            reason: Some("wheel_burst_no_semantics_match".to_string()),
            evidence: script_evidence_for_active(active),
            last_bundle_dir: svc
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&svc.cfg.out_dir, p)),
            last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
        });
        return true;
    };

    let pos = if let Some(ui_ref) = ui.as_deref() {
        pointer_position_prefer_intended_hit(
            app,
            snapshot,
            element_runtime,
            ui_ref,
            window,
            node,
            window_bounds,
        )
    } else {
        center_of_rect_clamped_to_rect(
            interaction_bounds_for_semantics_node(element_runtime, None, window, node),
            window_bounds,
        )
    };
    if let Some(ui) = ui {
        let note = format!("wheel_burst dx={delta_x} dy={delta_y} count={count}");
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

    let kind = pointer_kind.unwrap_or(UiPointerKindV1::Mouse);
    let kind_label = match kind {
        UiPointerKindV1::Mouse => "mouse",
        UiPointerKindV1::Touch => "touch",
        UiPointerKindV1::Pen => "pen",
    };
    let payload = format!(
        "schema_version=1\nwindow={}\nx_px={}\ny_px={}\ndelta_x={}\ndelta_y={}\ncount={}\npointer_kind={}\n",
        window.data().as_ffi(),
        pos.x.0,
        pos.y.0,
        delta_x,
        delta_y,
        count,
        kind_label
    );
    let text_path = svc.cfg.out_dir.join("wheel_burst.request.txt");
    let trigger_path = svc.cfg.out_dir.join("wheel_burst.touch");
    let _ = std::fs::create_dir_all(&svc.cfg.out_dir);

    if std::fs::write(text_path, payload).is_ok() && touch_file(&trigger_path).is_ok() {
        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-wheel_burst"));
        }
        return false;
    }

    output.request_redraw = true;
    let label = format!("script-step-{step_index:04}-wheel_burst-write-failed");
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
            note: Some("wheel_burst_write_failed".to_string()),
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
        reason_code: Some("wheel_burst.write_failed".to_string()),
        reason: Some("wheel_burst_write_failed".to_string()),
        evidence: script_evidence_for_active(active),
        last_bundle_dir: svc
            .last_dump_dir
            .as_ref()
            .map(|p| display_path(&svc.cfg.out_dir, p)),
        last_bundle_artifact: svc.last_dump_artifact_stats.clone(),
    });
    true
}

pub(super) fn handle_move_pointer_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::MovePointer {
        window: target_window,
        pointer_kind,
        target,
    } = step
    else {
        return false;
    };

    let resolved_target_window =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref());
    append_diag_script_migration_trace(
        &svc.cfg.out_dir,
        &format!(
            "unix_ms={} kind=move_pointer_resolve step_index={} window={:?} anchor_window={:?} target_window_spec={:?} resolved_target_window={:?}",
            unix_ms_now(),
            step_index,
            window,
            anchor_window,
            target_window,
            resolved_target_window,
        ),
    );
    if let Some(target_window) = resolved_target_window {
        if target_window != window {
            let pointer_session_active = active.has_pointer_session();
            let dock_drag_active = app.drag(fret_core::PointerId(0)).is_some_and(|d| {
                (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                    || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                    && d.dragging
            });

            // During cross-window dock drags, the target window can be fully occluded and may not
            // produce frames (and therefore not provide a live semantics snapshot). Treat
            // `move_pointer` as a cursor-positioning operation: resolve the selector against the
            // most recent captured semantics snapshot for the target window and write a cursor
            // override, without forcing a script handoff.
            //
            // This is intentionally narrow: only `test_id` selectors are supported in this
            // fallback path.
            if (pointer_session_active || dock_drag_active)
                && let UiSelectorV1::TestId { id, .. } = &target
            {
                let cached = svc.per_window.get(&target_window).and_then(|ring| {
                    let bounds = ring.test_id_bounds.get(id)?;
                    let window_bounds = ring.snapshots.back().map(|s| s.window_bounds);
                    Some((window_bounds, *bounds))
                });

                if cached.is_none() {
                    append_diag_script_migration_trace(
                        &svc.cfg.out_dir,
                        &format!(
                            "unix_ms={} kind=move_pointer_remote_miss step_index={} window={:?} target_window={:?} anchor_window={:?} test_id={id:?} pointer_session_active={} dock_drag_active={}",
                            unix_ms_now(),
                            step_index,
                            window,
                            target_window,
                            anchor_window,
                            pointer_session_active,
                            dock_drag_active,
                        ),
                    );
                }

                if let Some((window_bounds_v1, node_bounds)) = cached {
                    append_diag_script_migration_trace(
                        &svc.cfg.out_dir,
                        &format!(
                            "unix_ms={} kind=move_pointer_remote_hit step_index={} window={:?} target_window={:?} anchor_window={:?} test_id={id:?} pointer_session_active={} dock_drag_active={} window_bounds_present={}",
                            unix_ms_now(),
                            step_index,
                            window,
                            target_window,
                            anchor_window,
                            pointer_session_active,
                            dock_drag_active,
                            window_bounds_v1.is_some(),
                        ),
                    );
                    let mut x = node_bounds.origin.x.0 + (node_bounds.size.width.0 * 0.5);
                    let mut y = node_bounds.origin.y.0 + (node_bounds.size.height.0 * 0.5);
                    if let Some(window_bounds_v1) = window_bounds_v1 {
                        let clamp_x_min = window_bounds_v1.x;
                        let clamp_y_min = window_bounds_v1.y;
                        let clamp_x_max = window_bounds_v1.x + window_bounds_v1.w;
                        let clamp_y_max = window_bounds_v1.y + window_bounds_v1.h;
                        if x.is_finite() {
                            x = x.clamp(clamp_x_min, clamp_x_max);
                        }
                        if y.is_finite() {
                            y = y.clamp(clamp_y_min, clamp_y_max);
                        }
                    }

                    let _ = write_cursor_override_window_client_logical(
                        &svc.cfg.out_dir,
                        target_window,
                        x,
                        y,
                    );
                    active.last_explicit_cursor_override =
                        Some(CursorOverrideTarget::WindowClientLogical(target_window));
                    active.last_explicit_cursor_override_pos = Some(ExplicitCursorOverridePos {
                        target: CursorOverrideTarget::WindowClientLogical(target_window),
                        x_px: x,
                        y_px: y,
                    });
                    push_script_event_log(
                        active,
                        &svc.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "move_pointer.remote_semantics".to_string(),
                            step_index: Some(step_index as u32),
                            note: Some(format!(
                                "from_window={} target_window={} test_id={id:?} pointer_session_active={} dock_drag_active={}",
                                window.data().as_ffi(),
                                target_window.data().as_ffi(),
                                pointer_session_active,
                                dock_drag_active,
                            )),
                            bundle_dir: None,
                            window: Some(window.data().as_ffi()),
                            tick_id: Some(app.tick_id().0),
                            frame_id: Some(app.frame_id().0),
                            window_snapshot_seq: None,
                        },
                    );

                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    if svc.cfg.script_auto_dump {
                        *force_dump_label =
                            Some(format!("script-step-{step_index:04}-move_pointer-remote"));
                    }
                    return false;
                }
            }

            append_diag_script_migration_trace(
                &svc.cfg.out_dir,
                &format!(
                    "unix_ms={} kind=move_pointer_handoff step_index={} window={:?} target_window={:?} anchor_window={:?} pointer_session_active={} dock_drag_active={}",
                    unix_ms_now(),
                    step_index,
                    window,
                    target_window,
                    anchor_window,
                    pointer_session_active,
                    dock_drag_active,
                ),
            );
            *handoff_to = Some(target_window);
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-move_pointer-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }

    if *stop_script {
        active.wait_until = None;
        active.screenshot_wait = None;
        return false;
    }
    if handoff_to.is_some() {
        active.wait_until = None;
        active.screenshot_wait = None;
        // This step is window-targeted; the runtime will migrate the script.
        return false;
    }

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

    let pos = if let Some(ui_ref) = ui.as_deref() {
        pointer_position_prefer_intended_hit(
            app,
            snapshot,
            element_runtime,
            ui_ref,
            window,
            node,
            window_bounds,
        )
    } else {
        center_of_rect_clamped_to_rect(
            interaction_bounds_for_semantics_node(element_runtime, None, window, node),
            window_bounds,
        )
    };
    if let Some(ui) = ui {
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
    let cross_window_dock_drag_active = app.drag(fret_core::PointerId(0)).is_some_and(|d| {
        (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && d.cross_window_hover
            && d.dragging
    });
    if !cross_window_dock_drag_active {
        output.events.push(move_pointer_event(pos, pointer_type));
    } else {
        // Cross-window dock drags rely on `InternalDrag::Over` to keep the docking hover target
        // refreshed even when pointer move events are suppressed.
        //
        // This keeps `move_pointer` useful for scripted docking flows without requiring scripts to
        // switch to a delta-driven `pointer_move` step purely to update docking arbitration.
        let modifiers = active
            .pointer_sessions
            .values()
            .find(|session| session.window == window)
            .map(|session| session.modifiers)
            .unwrap_or_default();
        output
            .events
            .push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                kind: fret_core::InternalDragKind::Over,
                modifiers,
            }));
    }
    let _ = write_cursor_override_window_client_logical(&svc.cfg.out_dir, window, pos.x.0, pos.y.0);
    active.last_explicit_cursor_override = Some(CursorOverrideTarget::WindowClientLogical(window));
    active.last_explicit_cursor_override_pos = Some(ExplicitCursorOverridePos {
        target: CursorOverrideTarget::WindowClientLogical(window),
        x_px: pos.x.0,
        y_px: pos.y.0,
    });

    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-move_pointer"));
    }

    false
}
