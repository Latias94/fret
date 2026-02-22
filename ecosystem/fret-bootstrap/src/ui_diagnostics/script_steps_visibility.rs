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

    let docking_diag = app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_latest_for_window(window));
    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window));
    let dock_drag_runtime = dock_drag_runtime_state(app, svc.known_windows.as_slice());
    if eval_predicate(
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
