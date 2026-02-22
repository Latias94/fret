use super::*;

pub(super) fn handle_scroll_into_view_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::ScrollIntoView {
        container,
        target,
        delta_x,
        delta_y,
        require_fully_within_container,
        require_fully_within_window,
        padding_px,
        padding_insets_px,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-scroll_into_view-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::ScrollIntoView(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2ScrollIntoViewState {
            step_index,
            remaining_frames: timeout_frames,
        },
    };

    let target_predicate = if require_fully_within_window {
        UiPredicateV1::BoundsWithinWindow {
            target: target.clone(),
            padding_px,
            padding_insets_px,
            eps_px: 0.0,
        }
    } else {
        UiPredicateV1::VisibleInWindow {
            target: target.clone(),
        }
    };
    let docking_diag = app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_latest_for_window(window));
    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window));
    let dock_drag_runtime = dock_drag_runtime_state(app, svc.known_windows.as_slice());
    let visible_ok = eval_predicate(
        snapshot,
        window_bounds,
        window,
        input_ctx,
        element_runtime,
        app.global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window)),
        app.global::<fret_core::RendererTextPerfSnapshot>().copied(),
        app.global::<fret_core::RendererTextFontTraceSnapshot>(),
        svc.known_windows.as_slice(),
        app.global::<fret_runtime::PlatformCapabilities>(),
        docking_diag,
        dock_drag_runtime.as_ref(),
        text_font_stack_key_stable_frames,
        font_catalog_populated,
        system_font_rescan_idle,
        &target_predicate,
    );
    let container_ok = if require_fully_within_container {
        let container_node = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &container,
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        );
        let target_node = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &target,
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        );
        if let (Some(container_node), Some(target_node)) = (container_node, target_node) {
            let insets =
                padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(padding_px));
            let inner = rect_inset(container_node.bounds, insets);
            rect_fully_contains(inner, target_node.bounds)
        } else {
            false
        }
    } else {
        true
    };

    if visible_ok && container_ok {
        active.v2_step_state = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-scroll_into_view"));
        }
    } else if state.remaining_frames == 0 {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-scroll_into_view-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("scroll_into_view_timeout".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    } else {
        let container_node = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &container,
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        );
        if let Some(container_node) = container_node {
            let pos = ui
                .as_deref()
                .map(|ui| {
                    wheel_position_prefer_intended_hit(
                        snapshot,
                        ui,
                        container_node,
                        container_node.bounds,
                        window_bounds,
                    )
                })
                .unwrap_or_else(|| {
                    center_of_rect_clamped_to_rect(container_node.bounds, window_bounds)
                });
            if let Some(ui) = ui.as_deref_mut() {
                let note = format!("scroll_into_view.wheel dx={delta_x} dy={delta_y}");
                record_hit_test_trace_for_selector(
                    &mut active.hit_test_trace,
                    ui,
                    element_runtime,
                    window,
                    semantics_snapshot,
                    &container,
                    step_index as u32,
                    pos,
                    Some(container_node),
                    Some(note.as_str()),
                    svc.cfg.max_debug_string_bytes,
                );
            }
            output.events.push(wheel_event(pos, delta_x, delta_y));
        }

        state.remaining_frames = state.remaining_frames.saturating_sub(1);
        active.v2_step_state = Some(V2StepState::ScrollIntoView(state));
        output.request_redraw = true;
    }

    true
}
