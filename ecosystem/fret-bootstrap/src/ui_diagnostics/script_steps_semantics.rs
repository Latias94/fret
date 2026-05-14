use super::*;

pub(super) fn handle_focus_step(
    svc: &mut UiDiagnosticsService,
    app: &mut App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Focus { window: _, target } = step else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!("script-step-{step_index:04}-focus-no-semantics"));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
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
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-focus-no-semantics-match"
        ));
        *stop_script = true;
        *failure_reason = Some("focus_no_semantics_match".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    if !node.actions.focus {
        *force_dump_label = Some(format!("script-step-{step_index:04}-focus-unavailable"));
        *stop_script = true;
        *failure_reason = Some("focus_unavailable".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    }

    let Some(ui) = ui else {
        *force_dump_label = Some(format!("script-step-{step_index:04}-focus-no-ui-tree"));
        *stop_script = true;
        *failure_reason = Some("focus_requires_ui_tree".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    ui.set_focus(Some(node.id));
    ui.publish_window_runtime_snapshots(app);
    let injected_step_index = step_index.min(u32::MAX as usize) as u32;
    record_focus_trace(
        &mut active.focus_trace,
        app,
        window,
        element_runtime,
        Some(snapshot),
        Some(&*ui),
        injected_step_index,
        Some(node.id.data().as_ffi()),
        node.test_id.as_deref(),
        "focus.set_focus",
    );
    if let Some(note) = focus_trace_summary_note(&active.focus_trace, injected_step_index) {
        let kind = if active
            .focus_trace
            .iter()
            .rev()
            .find(|entry| entry.step_index == injected_step_index)
            .and_then(|entry| entry.matches_expected)
            == Some(true)
        {
            "focus.trace"
        } else {
            "focus.trace_mismatch"
        };
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: kind.to_string(),
                step_index: Some(injected_step_index),
                note: Some(note),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
    }
    active.last_injected_step = Some(injected_step_index);
    active.next_step = active.next_step.saturating_add(1);
    active.v2_step_state = None;
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-focus"));
    }

    true
}

pub(super) fn handle_activate_step(
    svc: &mut UiDiagnosticsService,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Activate { window: _, target } = step else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!("script-step-{step_index:04}-activate-no-semantics"));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
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
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-activate-no-semantics-match"
        ));
        *stop_script = true;
        *failure_reason = Some("activate_no_semantics_match".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    if !node.actions.invoke {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-activate-invoke-unavailable"
        ));
        *stop_script = true;
        *failure_reason = Some("activate_invoke_unavailable".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    }

    let Some(ui) = ui else {
        *force_dump_label = Some(format!("script-step-{step_index:04}-activate-no-ui-tree"));
        *stop_script = true;
        *failure_reason = Some("activate_requires_ui_tree".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    fret_ui_app::accessibility_actions::invoke_with_role(
        ui,
        app,
        services,
        node.id,
        Some(node.role),
    );
    let injected_step_index = step_index.min(u32::MAX as usize) as u32;
    active.last_injected_step = Some(injected_step_index);
    active.next_step = active.next_step.saturating_add(1);
    active.v2_step_state = None;
    output.request_redraw = true;
    if svc.cfg.script_auto_dump {
        *force_dump_label = Some(format!("script-step-{step_index:04}-activate"));
    }

    true
}
