use super::*;

pub(super) fn handle_assert_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
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
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::Assert {
        window: target_window,
        predicate,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let mut predicate_window = window;
    if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            if UiDiagnosticsService::predicate_can_eval_off_window(&predicate) {
                predicate_window = target_window;
                output.effects.push(Effect::Redraw(target_window));
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            } else {
                *handoff_to = Some(target_window);
                output.effects.push(Effect::Redraw(target_window));
                output
                    .effects
                    .push(Effect::RequestAnimationFrame(target_window));
                output.request_redraw = true;
            }
        }
    } else if target_window.is_some() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-assert-window-not-found"
        ));
        *stop_script = true;
        *failure_reason = Some("window_target_unresolved".to_string());
        output.request_redraw = true;
    }

    if *stop_script {
        // Fall through to common termination logic.
    } else if handoff_to.is_some() {
        // This step is window-targeted; the runtime will migrate the script.
    } else {
        let ok = match &predicate {
            UiPredicateV1::EventKindSeen { event_kind } => svc
                .per_window
                .get(&predicate_window)
                .is_some_and(|ring| ring.events.iter().any(|e| e.kind == *event_kind)),
            UiPredicateV1::RunnerAccessibilityActivated => app
                .global::<fret_runtime::RunnerAccessibilityDiagnosticsStore>()
                .and_then(|store| store.snapshot(predicate_window))
                .is_some_and(|snapshot| snapshot.activation_requests > 0),
            UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
                text_font_stack_key_stable_frames >= *stable_frames
            }
            UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
            UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
            _ => {
                let docking_diag = app
                    .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                    .and_then(|store| store.docking_latest_for_window(predicate_window));
                let workspace_diag = app
                    .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                    .and_then(|store| store.workspace_latest_for_window(predicate_window));
                let input_ctx = app
                    .global::<fret_runtime::WindowInputContextService>()
                    .and_then(|svc| svc.snapshot(predicate_window));
                let text_input_snapshot = app
                    .global::<fret_runtime::WindowTextInputSnapshotService>()
                    .and_then(|svc| svc.snapshot(predicate_window));
                let dock_drag_runtime = dock_drag_runtime_state(app, svc.known_windows.as_slice());
                let platform_caps = app.global::<fret_runtime::PlatformCapabilities>();

                if let Some(snapshot) = semantics_snapshot {
                    record_overlay_placement_trace(
                        &mut active.overlay_placement_trace,
                        element_runtime,
                        Some(snapshot),
                        window,
                        step_index as u32,
                        "assert",
                    );
                    eval_predicate(
                        snapshot,
                        window_bounds,
                        predicate_window,
                        active.scope_root_for_window(predicate_window),
                        input_ctx,
                        element_runtime,
                        text_input_snapshot,
                        app.global::<fret_core::RendererTextPerfSnapshot>().copied(),
                        app.global::<fret_core::RendererTextFontTraceSnapshot>(),
                        svc.known_windows.as_slice(),
                        platform_caps,
                        docking_diag,
                        workspace_diag,
                        dock_drag_runtime.as_ref(),
                        text_font_stack_key_stable_frames,
                        font_catalog_populated,
                        system_font_rescan_idle,
                        &predicate,
                    )
                } else {
                    eval_predicate_without_semantics(
                        predicate_window,
                        svc.known_windows.as_slice(),
                        platform_caps,
                        docking_diag,
                        workspace_diag,
                        dock_drag_runtime.as_ref(),
                        &predicate,
                    )
                    .unwrap_or_else(|| {
                        *force_dump_label =
                            Some(format!("script-step-{step_index:04}-assert-no-semantics"));
                        *stop_script = true;
                        *failure_reason = Some("no_semantics_snapshot".to_string());
                        output.request_redraw = true;
                        false
                    })
                }
            }
        };

        if ok {
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
        } else {
            *force_dump_label = Some(format!("script-step-{step_index:04}-assert-failed"));
            *stop_script = true;
            *failure_reason = Some("assert_failed".to_string());
            output.request_redraw = true;
        }
    }

    true
}
