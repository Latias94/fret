use super::*;

pub(super) fn handle_ensure_visible_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::EnsureVisible {
        window: _,
        target,
        within_window,
        padding_px,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-ensure_visible-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::EnsureVisible(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2EnsureVisibleState {
            step_index,
            remaining_frames: timeout_frames,
        },
    };

    let predicate = if within_window {
        UiPredicateV1::BoundsWithinWindow {
            target,
            padding_px,
            padding_insets_px: None,
            eps_px: 0.0,
        }
    } else {
        UiPredicateV1::VisibleInWindow { target }
    };

    if within_window {
        if let Some(node) = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            match &predicate {
                UiPredicateV1::BoundsWithinWindow { target, .. } => target,
                _ => unreachable!(),
            },
            active.scope_root_for_window(window),
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        ) {
            let insets = UiPaddingInsetsV1::uniform(padding_px);
            let inner_window = rect_inset(window_bounds, insets);
            let target_w = node.bounds.size.width.0.max(0.0);
            let target_h = node.bounds.size.height.0.max(0.0);
            let inner_w = inner_window.size.width.0.max(0.0);
            let inner_h = inner_window.size.height.0.max(0.0);
            if inner_w > 1.0
                && inner_h > 1.0
                && (target_w > inner_w + 0.5 || target_h > inner_h + 0.5)
            {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-ensure_visible-impossible-oversized"
                ));
                *stop_script = true;
                *failure_reason = Some("ensure_visible_impossible_oversized_target".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
                return true;
            }
        }
    }

    let docking_diag = app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_latest_for_window(window));
    let workspace_diag = app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.workspace_latest_for_window(window));
    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window));
    let dock_drag_runtime = dock_drag_runtime_state(app, svc.known_windows.as_slice());
    let open_window_count = UiDiagnosticsService::open_window_count_for_predicates(app);
    if eval_predicate(
        snapshot,
        window_bounds,
        window,
        active.scope_root_for_window(window),
        input_ctx,
        element_runtime,
        app.global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window)),
        app.global::<fret_core::RendererTextPerfSnapshot>().copied(),
        app.global::<fret_core::RendererTextFontTraceSnapshot>(),
        svc.known_windows.as_slice(),
        open_window_count,
        app.global::<fret_runtime::PlatformCapabilities>(),
        docking_diag,
        workspace_diag,
        dock_drag_runtime.as_ref(),
        text_font_stack_key_stable_frames,
        font_catalog_populated,
        system_font_rescan_idle,
        &predicate,
    ) {
        active.v2_step_state = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-ensure_visible"));
        }
    } else if state.remaining_frames == 0 {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-ensure_visible-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("ensure_visible_timeout".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    } else {
        state.remaining_frames = state.remaining_frames.saturating_sub(1);
        active.v2_step_state = Some(V2StepState::EnsureVisible(state));
        output.request_redraw = true;
    }

    true
}
