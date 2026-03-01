//! Script runner engine extracted from `ui_diagnostics.rs`.
//!
//! This module exists to keep the main `ui_diagnostics.rs` file from growing without bound.
//! During the fearless refactor we move the large per-frame script driver here in small steps.

use super::*;

pub(super) fn active_script_needs_semantics_snapshot(active: &ActiveScript) -> bool {
    if active.wait_until.is_some() {
        return true;
    }

    if let Some(state) = active.v2_step_state.as_ref() {
        // Multi-frame script steps may still need fresh semantics snapshots to make progress.
        // In particular, scroll/visibility + "stable" gates must observe updated bounds as the
        // UI reacts to injected events.
        return matches!(
            state,
            V2StepState::ClickStable(_)
                | V2StepState::ClickSelectableTextSpanStable(_)
                | V2StepState::WaitBoundsStable(_)
                | V2StepState::EnsureVisible(_)
                | V2StepState::ScrollIntoView(_)
                | V2StepState::TypeTextInto(_)
                | V2StepState::MenuSelect(_)
                | V2StepState::MenuSelectPath(_)
                | V2StepState::DragPointerUntil(_)
                | V2StepState::DragTo(_)
                | V2StepState::SetSliderValue(_)
        );
    }

    let Some(step) = active.steps.get(active.next_step) else {
        return false;
    };

    match step {
        UiActionStepV2::Click { .. }
        | UiActionStepV2::Tap { .. }
        | UiActionStepV2::LongPress { .. }
        | UiActionStepV2::Swipe { .. }
        | UiActionStepV2::Pinch { .. }
        | UiActionStepV2::SetBaseRef { .. }
        | UiActionStepV2::ClickStable { .. }
        | UiActionStepV2::ClickSelectableTextSpanStable { .. }
        | UiActionStepV2::WaitBoundsStable { .. }
        | UiActionStepV2::MovePointer { .. }
        | UiActionStepV2::PointerDown { .. }
        | UiActionStepV2::DragPointer { .. }
        | UiActionStepV2::DragPointerUntil { .. }
        | UiActionStepV2::MovePointerSweep { .. }
        | UiActionStepV2::Wheel { .. }
        | UiActionStepV2::WaitUntil { .. }
        | UiActionStepV2::WaitOverlayPlacementTrace { .. }
        | UiActionStepV2::Assert { .. }
        | UiActionStepV2::EnsureVisible { .. }
        | UiActionStepV2::ScrollIntoView { .. }
        | UiActionStepV2::TypeTextInto { .. }
        | UiActionStepV2::MenuSelect { .. }
        | UiActionStepV2::MenuSelectPath { .. }
        | UiActionStepV2::DragTo { .. }
        | UiActionStepV2::SetSliderValue { .. } => true,
        UiActionStepV2::ResetDiagnostics
        | UiActionStepV2::ClearBaseRef
        | UiActionStepV2::PressKey { .. }
        | UiActionStepV2::PressShortcut { .. }
        | UiActionStepV2::TypeText { .. }
        | UiActionStepV2::Ime { .. }
        | UiActionStepV2::WaitFrames { .. }
        | UiActionStepV2::WaitShortcutRoutingTrace { .. }
        | UiActionStepV2::CaptureBundle { .. }
        | UiActionStepV2::CaptureScreenshot { .. }
        | UiActionStepV2::SetWindowInnerSize { .. }
        | UiActionStepV2::SetWindowInsets { .. }
        | UiActionStepV2::SetClipboardForceUnavailable { .. }
        | UiActionStepV2::SetClipboardText { .. }
        | UiActionStepV2::AssertClipboardText { .. }
        | UiActionStepV2::InjectIncomingOpen { .. }
        | UiActionStepV2::SetWindowOuterPosition { .. }
        | UiActionStepV2::SetCursorScreenPos { .. }
        | UiActionStepV2::SetCursorInWindow { .. }
        | UiActionStepV2::SetCursorInWindowLogical { .. }
        | UiActionStepV2::SetMouseButtons { .. }
        | UiActionStepV2::RaiseWindow { .. }
        | UiActionStepV2::PointerMove { .. }
        | UiActionStepV2::PointerUp { .. } => false,
    }
}

pub(super) fn script_step_kind_name(step: &UiActionStepV2) -> &'static str {
    match step {
        UiActionStepV2::Click { .. } => "click",
        UiActionStepV2::Tap { .. } => "tap",
        UiActionStepV2::LongPress { .. } => "long_press",
        UiActionStepV2::Swipe { .. } => "swipe",
        UiActionStepV2::Pinch { .. } => "pinch",
        UiActionStepV2::SetBaseRef { .. } => "set_base_ref",
        UiActionStepV2::ClearBaseRef => "clear_base_ref",
        UiActionStepV2::ClickStable { .. } => "click_stable",
        UiActionStepV2::ClickSelectableTextSpanStable { .. } => "click_selectable_text_span_stable",
        UiActionStepV2::DragPointer { .. } => "drag_pointer",
        UiActionStepV2::DragPointerUntil { .. } => "drag_pointer_until",
        UiActionStepV2::DragTo { .. } => "drag_to",
        UiActionStepV2::Wheel { .. } => "wheel",
        UiActionStepV2::TypeText { .. } => "type_text",
        UiActionStepV2::TypeTextInto { .. } => "type_text_into",
        UiActionStepV2::WaitFrames { .. } => "wait_frames",
        UiActionStepV2::WaitUntil { .. } => "wait_until",
        UiActionStepV2::Assert { .. } => "assert",
        UiActionStepV2::CaptureBundle { .. } => "capture_bundle",
        UiActionStepV2::CaptureScreenshot { .. } => "capture_screenshot",
        UiActionStepV2::ResetDiagnostics => "reset_diagnostics",
        UiActionStepV2::SetClipboardText { .. } => "set_clipboard_text",
        UiActionStepV2::AssertClipboardText { .. } => "assert_clipboard_text",
        _ => "step",
    }
}

pub(super) fn push_script_event_log(
    active: &mut ActiveScript,
    cfg: &UiDiagnosticsConfig,
    entry: UiScriptEventLogEntryV1,
) {
    let max = cfg.max_gating_trace_entries;
    if max == 0 {
        active.event_log_dropped = active.event_log_dropped.saturating_add(1);
        return;
    }

    active.event_log.push(entry);
    if active.event_log.len() > max {
        let extra = active.event_log.len().saturating_sub(max);
        active.event_log.drain(0..extra);
        active.event_log_dropped = active.event_log_dropped.saturating_add(extra as u64);
    }
}

pub(super) fn script_evidence_for_active(active: &ActiveScript) -> Option<UiScriptEvidenceV1> {
    if active.event_log.is_empty()
        && active.event_log_dropped == 0
        && active.selector_resolution_trace.is_empty()
        && active.hit_test_trace.is_empty()
        && active.click_stable_trace.is_empty()
        && active.bounds_stable_trace.is_empty()
        && active.focus_trace.is_empty()
        && active.shortcut_routing_trace.is_empty()
        && active.overlay_placement_trace.is_empty()
        && active.web_ime_trace.is_empty()
        && active.ime_event_trace.is_empty()
    {
        return None;
    }
    Some(UiScriptEvidenceV1 {
        event_log: active.event_log.clone(),
        event_log_dropped: active.event_log_dropped,
        capabilities_check: None,
        selector_resolution_trace: active.selector_resolution_trace.clone(),
        hit_test_trace: active.hit_test_trace.clone(),
        click_stable_trace: active.click_stable_trace.clone(),
        bounds_stable_trace: active.bounds_stable_trace.clone(),
        focus_trace: active.focus_trace.clone(),
        shortcut_routing_trace: active.shortcut_routing_trace.clone(),
        overlay_placement_trace: active.overlay_placement_trace.clone(),
        web_ime_trace: active.web_ime_trace.clone(),
        ime_event_trace: active.ime_event_trace.clone(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DriveScriptStepDispatchOutcome {
    Continue,
    ReturnOutput,
    RequeueActiveAndReturnOutput,
}

pub(super) fn dispatch_drive_script_step(
    service: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    scale_factor: f32,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: &mut Option<&mut UiTree<App>>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    force_dump_max_snapshots: &mut Option<usize>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> DriveScriptStepDispatchOutcome {
    match step {
        UiActionStepV2::SetBaseRef {
            window: target_window,
            target,
        } => {
            if let Some(target_window) = service.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
                if target_window != window {
                    *handoff_to = Some(target_window);
                    output
                        .effects
                        .push(Effect::RequestAnimationFrame(target_window));
                    output.request_redraw = true;
                    active.v2_step_state = None;
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    return DriveScriptStepDispatchOutcome::Continue;
                }
            } else if target_window.is_some() {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_base_ref-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                output.request_redraw = true;
                active.v2_step_state = None;
                active.wait_until = None;
                active.screenshot_wait = None;
                return DriveScriptStepDispatchOutcome::Continue;
            }

            let Some(snapshot) = semantics_snapshot else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_base_ref-no-semantics"
                ));
                *stop_script = true;
                *failure_reason = Some("no_semantics_snapshot".to_string());
                active.v2_step_state = None;
                active.wait_until = None;
                active.screenshot_wait = None;
                output.request_redraw = true;
                return DriveScriptStepDispatchOutcome::Continue;
            };

            let Some(node) = select_semantics_node_with_trace(
                snapshot,
                window,
                element_runtime,
                &target,
                None,
                step_index as u32,
                service.cfg.redact_text,
                &mut active.selector_resolution_trace,
            ) else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_base_ref-no-semantics-match"
                ));
                *stop_script = true;
                *failure_reason = Some("set_base_ref_no_semantics_match".to_string());
                active.v2_step_state = None;
                active.wait_until = None;
                active.screenshot_wait = None;
                output.request_redraw = true;
                return DriveScriptStepDispatchOutcome::Continue;
            };

            active.base_ref = Some(ScriptBaseRefState {
                window,
                scope_root: node.id.data().as_ffi(),
            });
            push_script_event_log(
                active,
                &service.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "base_ref.set".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some(format!(
                        "window={} scope_root={} test_id={:?}",
                        window.data().as_ffi(),
                        node.id.data().as_ffi(),
                        node.test_id
                    )),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );

            active.v2_step_state = None;
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
        }
        UiActionStepV2::ClearBaseRef => {
            active.base_ref = None;
            push_script_event_log(
                active,
                &service.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "base_ref.cleared".to_string(),
                    step_index: Some(step_index as u32),
                    note: None,
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            active.v2_step_state = None;
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
        }
        step @ (UiActionStepV2::SetWindowInnerSize { .. }
        | UiActionStepV2::SetWindowOuterPosition { .. }
        | UiActionStepV2::SetCursorScreenPos { .. }
        | UiActionStepV2::SetCursorInWindow { .. }
        | UiActionStepV2::SetCursorInWindowLogical { .. }
        | UiActionStepV2::SetMouseButtons { .. }
        | UiActionStepV2::RaiseWindow { .. }
        | UiActionStepV2::SetWindowInsets { .. }) => {
            script_steps::handle_window_effect_steps(
                service,
                window,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
        }
        step @ (UiActionStepV2::SetClipboardForceUnavailable { .. }
        | UiActionStepV2::SetClipboardText { .. }
        | UiActionStepV2::InjectIncomingOpen { .. }
        | UiActionStepV2::WaitFrames { .. }
        | UiActionStepV2::ResetDiagnostics) => {
            let handled =
                script_steps::handle_effect_only_steps(service, window, step, active, output);
            debug_assert!(handled);
        }
        step @ UiActionStepV2::AssertClipboardText { .. } => {
            let handled = script_steps_clipboard::handle_assert_clipboard_text_step(
                service,
                app,
                window,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step
        @ (UiActionStepV2::CaptureBundle { .. } | UiActionStepV2::CaptureScreenshot { .. }) => {
            let handled = script_steps::handle_capture_steps(
                service,
                app,
                window,
                step_index,
                step,
                scale_factor,
                active,
                output,
                force_dump_label,
                force_dump_max_snapshots,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ (UiActionStepV2::PressKey { .. }
        | UiActionStepV2::PressShortcut { .. }
        | UiActionStepV2::TypeText { .. }
        | UiActionStepV2::Ime { .. }) => {
            let handled = script_steps_input::handle_keyboard_text_steps(
                service,
                app,
                window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::WaitUntil { .. } => {
            let handled = script_steps_wait::handle_wait_until_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                text_font_stack_key_stable_frames,
                font_catalog_populated,
                system_font_rescan_idle,
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::WaitShortcutRoutingTrace { .. } => {
            let handled = script_steps_wait::handle_wait_shortcut_routing_trace_step(
                app,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::WaitOverlayPlacementTrace { .. } => {
            let handled = script_steps_wait::handle_wait_overlay_placement_trace_step(
                window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::Assert { .. } => {
            let handled = script_steps_assert::handle_assert_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                text_font_stack_key_stable_frames,
                font_catalog_populated,
                system_font_rescan_idle,
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::Click { .. } => {
            let should_return = script_steps_pointer::handle_click_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::Tap { .. } => {
            let should_return = script_steps_pointer::handle_tap_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::LongPress { .. } => {
            let should_return = script_steps_pointer::handle_long_press_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::Swipe { .. } => {
            let should_return = script_steps_pointer::handle_swipe_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::Pinch { .. } => {
            let should_return = script_steps_pointer::handle_pinch_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::ClickStable { .. } => {
            let handled = script_steps_pointer::handle_click_stable_step(
                service,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::ClickSelectableTextSpanStable { .. } => {
            let handled = script_steps_pointer::handle_click_selectable_text_span_stable_step(
                service,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::MovePointer { .. } => {
            let should_return = script_steps_pointer::handle_move_pointer_step(
                service,
                app,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::PointerDown { .. } => {
            let handled = script_steps_pointer_session::handle_pointer_down_step(
                service,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::PointerMove { .. } => {
            let handled = script_steps_pointer_session::handle_pointer_move_step(
                service,
                app,
                window,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::PointerUp { .. } => {
            let handled = script_steps_pointer_session::handle_pointer_up_step(
                service,
                app,
                window,
                step_index,
                step,
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::DragPointer { .. } => {
            let should_return = script_steps_drag::handle_drag_pointer_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::DragPointerUntil { .. } => {
            let handled = script_steps_drag::handle_drag_pointer_until_step(
                service,
                app,
                window,
                window_bounds,
                anchor_window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                text_font_stack_key_stable_frames,
                font_catalog_populated,
                system_font_rescan_idle,
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::MovePointerSweep { .. } => {
            let should_return = script_steps_pointer_sweep::handle_move_pointer_sweep_step(
                service,
                app,
                window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::Wheel { .. } => {
            let should_return = script_steps_pointer::handle_wheel_step(
                service,
                app,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
            );
            if should_return {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::WaitBoundsStable { .. } => {
            let handled = script_steps_wait::handle_wait_bounds_stable_step(
                service,
                window,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::EnsureVisible { .. } => {
            let handled = script_steps_visibility::handle_ensure_visible_step(
                service,
                app,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                text_font_stack_key_stable_frames,
                font_catalog_populated,
                system_font_rescan_idle,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        UiActionStepV2::ScrollIntoView { .. } => {
            let handled = script_steps_scroll::handle_scroll_into_view_step(
                service,
                app,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                text_font_stack_key_stable_frames,
                font_catalog_populated,
                system_font_rescan_idle,
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::TypeTextInto { .. } => {
            let handled = script_steps_input::handle_type_text_into_step(
                service,
                app,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::MenuSelect { .. } => {
            let handled = script_steps_menu::handle_menu_select_step(
                service,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::MenuSelectPath { .. } => {
            let handled = script_steps_menu::handle_menu_select_path_step(
                service,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
        step @ UiActionStepV2::DragTo { .. } => {
            let Some(result) = script_steps_drag::handle_drag_to_step(
                service,
                app,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                handoff_to,
                stop_script,
                failure_reason,
            ) else {
                unreachable!("DragTo step must be handled");
            };
            if result.requeue_active_for_window {
                return DriveScriptStepDispatchOutcome::RequeueActiveAndReturnOutput;
            }
            if result.should_return_output {
                return DriveScriptStepDispatchOutcome::ReturnOutput;
            }
        }
        step @ UiActionStepV2::SetSliderValue { .. } => {
            let handled = script_steps_slider::handle_set_slider_value_step(
                service,
                window,
                window_bounds,
                step_index,
                step,
                element_runtime,
                semantics_snapshot,
                ui.as_deref_mut(),
                active,
                output,
                force_dump_label,
                stop_script,
                failure_reason,
            );
            debug_assert!(handled);
        }
    }

    DriveScriptStepDispatchOutcome::Continue
}

pub(super) fn finalize_drive_script_for_window(
    service: &mut UiDiagnosticsService,
    app: &mut App,
    window: AppWindowId,
    prev_next_step: usize,
    step_index: usize,
    step_index_u32: u32,
    step_kind: String,
    mut active: ActiveScript,
    mut output: UiScriptFrameOutput,
    mut force_dump_label: Option<String>,
    force_dump_max_snapshots: Option<usize>,
    mut stop_script: bool,
    mut failure_reason: Option<String>,
    handoff_to: Option<AppWindowId>,
) -> UiScriptFrameOutput {
    if !stop_script && handoff_to.is_none() && active.next_step > prev_next_step {
        push_script_event_log(
            &mut active,
            &service.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "step_end".to_string(),
                step_index: Some(step_index_u32),
                note: Some(step_kind.clone()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        active.event_log_active_step = None;
    }

    if let Some(target_window) = handoff_to {
        if service.active_scripts.contains_key(&target_window) {
            force_dump_label = Some(format!(
                "script-step-{step_index:04}-handoff-target-window-busy"
            ));
            stop_script = true;
            failure_reason = Some("script_handoff_target_window_busy".to_string());
        } else {
            service.active_scripts.insert(target_window, active);
            return output;
        }
    }

    if !output.events.is_empty() {
        for event in &output.events {
            service.record_script_event(app, window, event);
        }
    }

    if stop_script {
        push_script_event_log(
            &mut active,
            &service.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "script_failed".to_string(),
                step_index: Some(step_index as u32),
                note: failure_reason.clone(),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        if service.cfg.script_auto_dump {
            if let Some(label) = force_dump_label.as_deref() {
                push_script_event_log(
                    &mut active,
                    &service.cfg,
                    UiScriptEventLogEntryV1 {
                        unix_ms: unix_ms_now(),
                        kind: "bundle_dump_requested".to_string(),
                        step_index: Some(step_index as u32),
                        note: Some(label.to_string()),
                        bundle_dir: None,
                        window: Some(window.data().as_ffi()),
                        tick_id: Some(app.tick_id().0),
                        frame_id: Some(app.frame_id().0),
                        window_snapshot_seq: None,
                    },
                );
                let dumped_dir = service.dump_bundle(Some(label));
                if let Some(dir) = dumped_dir.as_ref() {
                    push_script_event_log(
                        &mut active,
                        &service.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "bundle_dumped".to_string(),
                            step_index: Some(step_index as u32),
                            note: Some(format_bundle_dump_note(label, None, None)),
                            bundle_dir: Some(display_path(&service.cfg.out_dir, dir)),
                            window: Some(window.data().as_ffi()),
                            tick_id: Some(app.tick_id().0),
                            frame_id: Some(app.frame_id().0),
                            window_snapshot_seq: None,
                        },
                    );
                }
            }
        } else if let Some(label) = force_dump_label {
            let note = format_bundle_dump_note(&label, force_dump_max_snapshots, None);
            push_script_event_log(
                &mut active,
                &service.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "bundle_dump_requested".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some(note),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            service.request_force_dump(
                label,
                force_dump_max_snapshots,
                Some(active.run_id),
                Some(step_index as u32),
                None,
            );
        }

        let reason_code = failure_reason
            .as_deref()
            .and_then(reason_code_for_script_failure)
            .map(|s| s.to_string());
        let evidence = script_evidence_for_active(&active);
        service.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id: active.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiScriptStageV1::Failed,
            step_index: Some(step_index as u32),
            reason_code,
            reason: failure_reason,
            evidence,
            last_bundle_dir: service
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&service.cfg.out_dir, p)),
            last_bundle_artifact: service.last_dump_artifact_stats.clone(),
        });
    } else {
        if let Some(label) = force_dump_label {
            let note = format_bundle_dump_note(&label, force_dump_max_snapshots, None);
            push_script_event_log(
                &mut active,
                &service.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "bundle_dump_requested".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some(note),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            service.request_force_dump(
                label,
                force_dump_max_snapshots,
                Some(active.run_id),
                Some(step_index as u32),
                None,
            );
        }

        if active.next_step >= active.steps.len() {
            let passed_step_index = active.next_step.saturating_sub(1) as u32;
            push_script_event_log(
                &mut active,
                &service.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_passed".to_string(),
                    step_index: Some(passed_step_index),
                    note: None,
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            service.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Passed,
                step_index: Some(passed_step_index),
                reason_code: None,
                reason: None,
                evidence: script_evidence_for_active(&active),
                last_bundle_dir: service
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&service.cfg.out_dir, p)),
                last_bundle_artifact: service.last_dump_artifact_stats.clone(),
            });
        } else if active.next_step < active.steps.len() {
            // Keep the app ticking while a script is active, even if the last injected events
            // did not invalidate UI state. This ensures `wait_until`/timeouts progress and
            // cross-window gates (tear-off, hover detection) do not stall.
            output.request_redraw = true;
            output.effects.push(Effect::Redraw(window));
            output.effects.push(Effect::RequestAnimationFrame(window));
            service.active_scripts.insert(window, active);
        }
    }

    output
}

pub(super) fn click_stable_trace_entry_eq(
    a: &UiClickStableTraceEntryV1,
    b: &UiClickStableTraceEntryV1,
) -> bool {
    a.step_index == b.step_index
        && a.stable_required == b.stable_required
        && a.stable_count == b.stable_count
        && a.remaining_frames == b.remaining_frames
        && a.hit_test.note == b.hit_test.note
        && a.hit_test.position.x_px.to_bits() == b.hit_test.position.x_px.to_bits()
        && a.hit_test.position.y_px.to_bits() == b.hit_test.position.y_px.to_bits()
        && a.hit_test.blocking_reason == b.hit_test.blocking_reason
        && a.hit_test.blocking_root == b.hit_test.blocking_root
        && a.hit_test.blocking_layer_id == b.hit_test.blocking_layer_id
}

pub(super) fn push_click_stable_trace(
    trace: &mut Vec<UiClickStableTraceEntryV1>,
    entry: UiClickStableTraceEntryV1,
) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| click_stable_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_CLICK_STABLE_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_CLICK_STABLE_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

pub(super) fn focus_trace_entry_eq(a: &UiFocusTraceEntryV1, b: &UiFocusTraceEntryV1) -> bool {
    a.step_index == b.step_index
        && a.note == b.note
        && a.expected_node_id == b.expected_node_id
        && a.expected_test_id == b.expected_test_id
}

pub(super) fn push_focus_trace(trace: &mut Vec<UiFocusTraceEntryV1>, entry: UiFocusTraceEntryV1) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| focus_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_FOCUS_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_FOCUS_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

pub(super) fn push_shortcut_routing_trace(
    trace: &mut Vec<UiShortcutRoutingTraceEntryV1>,
    entry: UiShortcutRoutingTraceEntryV1,
) {
    trace.push(entry);
    if trace.len() > MAX_SHORTCUT_ROUTING_TRACE_ENTRIES {
        let extra = trace
            .len()
            .saturating_sub(MAX_SHORTCUT_ROUTING_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

pub(super) fn shortcut_routing_trace_entry_matches_query(
    entry: &UiShortcutRoutingTraceEntryV1,
    query: &UiShortcutRoutingTraceQueryV1,
) -> bool {
    if let Some(phase) = &query.phase
        && entry.phase != *phase
    {
        return false;
    }
    if let Some(outcome) = &query.outcome
        && entry.outcome != *outcome
    {
        return false;
    }
    if let Some(key) = &query.key
        && entry.key != *key
    {
        return false;
    }
    if let Some(command) = &query.command
        && entry.command.as_deref() != Some(command.as_str())
    {
        return false;
    }
    if let Some(ime_composing) = query.ime_composing
        && entry.ime_composing != ime_composing
    {
        return false;
    }
    if let Some(focus_is_text_input) = query.focus_is_text_input
        && entry.focus_is_text_input != focus_is_text_input
    {
        return false;
    }
    true
}

pub(super) fn overlay_placement_trace_entry_matches_query(
    entry: &UiOverlayPlacementTraceEntryV1,
    expected_step_index: u32,
    query: &UiOverlayPlacementTraceQueryV1,
) -> bool {
    match entry {
        UiOverlayPlacementTraceEntryV1::AnchoredPanel {
            step_index,
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            preferred_side,
            chosen_side,
            align,
            sticky,
            ..
        } => {
            if *step_index != expected_step_index {
                return false;
            }
            if let Some(kind) = query.kind
                && kind != UiOverlayPlacementTraceKindV1::AnchoredPanel
            {
                return false;
            }
            if let Some(q) = &query.overlay_root_name
                && overlay_root_name.as_deref() != Some(q.as_str())
            {
                return false;
            }
            if let Some(q) = &query.anchor_test_id
                && anchor_test_id.as_deref() != Some(q.as_str())
            {
                return false;
            }
            if let Some(q) = &query.content_test_id
                && content_test_id.as_deref() != Some(q.as_str())
            {
                return false;
            }
            if let Some(q) = query.preferred_side
                && *preferred_side != q
            {
                return false;
            }
            if let Some(q) = query.chosen_side
                && *chosen_side != q
            {
                return false;
            }
            if let Some(flipped) = query.flipped {
                let actual = *chosen_side != *preferred_side;
                if actual != flipped {
                    return false;
                }
            }
            if let Some(q) = query.align
                && *align != q
            {
                return false;
            }
            if let Some(q) = query.sticky
                && *sticky != q
            {
                return false;
            }
            true
        }
        UiOverlayPlacementTraceEntryV1::PlacedRect {
            step_index,
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            side,
            ..
        } => {
            if *step_index != expected_step_index {
                return false;
            }
            if let Some(kind) = query.kind
                && kind != UiOverlayPlacementTraceKindV1::PlacedRect
            {
                return false;
            }
            if let Some(q) = &query.overlay_root_name
                && overlay_root_name.as_deref() != Some(q.as_str())
            {
                return false;
            }
            if let Some(q) = &query.anchor_test_id
                && anchor_test_id.as_deref() != Some(q.as_str())
            {
                return false;
            }
            if let Some(q) = &query.content_test_id
                && content_test_id.as_deref() != Some(q.as_str())
            {
                return false;
            }
            if let Some(q) = query.chosen_side
                && side != &Some(q)
            {
                return false;
            }
            true
        }
    }
}

pub(super) fn overlay_placement_trace_entry_matches_query_any_step(
    entry: &UiOverlayPlacementTraceEntryV1,
    query: &UiOverlayPlacementTraceQueryV1,
) -> bool {
    let step_index = match entry {
        UiOverlayPlacementTraceEntryV1::AnchoredPanel { step_index, .. } => *step_index,
        UiOverlayPlacementTraceEntryV1::PlacedRect { step_index, .. } => *step_index,
    };
    overlay_placement_trace_entry_matches_query(entry, step_index, query)
}

pub(super) fn overlay_placement_trace_entry_eq(
    a: &UiOverlayPlacementTraceEntryV1,
    b: &UiOverlayPlacementTraceEntryV1,
) -> bool {
    match (a, b) {
        (
            UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                step_index: a_step,
                overlay_root_name: a_name,
                anchor_element: a_anchor,
                content_element: a_content,
                ..
            },
            UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                step_index: b_step,
                overlay_root_name: b_name,
                anchor_element: b_anchor,
                content_element: b_content,
                ..
            },
        ) => a_step == b_step && a_name == b_name && a_anchor == b_anchor && a_content == b_content,
        (
            UiOverlayPlacementTraceEntryV1::PlacedRect {
                step_index: a_step,
                overlay_root_name: a_name,
                anchor_element: a_anchor,
                content_element: a_content,
                ..
            },
            UiOverlayPlacementTraceEntryV1::PlacedRect {
                step_index: b_step,
                overlay_root_name: b_name,
                anchor_element: b_anchor,
                content_element: b_content,
                ..
            },
        ) => a_step == b_step && a_name == b_name && a_anchor == b_anchor && a_content == b_content,
        _ => false,
    }
}

pub(super) fn push_overlay_placement_trace(
    trace: &mut Vec<UiOverlayPlacementTraceEntryV1>,
    entry: UiOverlayPlacementTraceEntryV1,
) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| overlay_placement_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_OVERLAY_PLACEMENT_TRACE_ENTRIES {
        let extra = trace
            .len()
            .saturating_sub(MAX_OVERLAY_PLACEMENT_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

pub(super) fn web_ime_trace_entry_eq(a: &UiWebImeTraceEntryV1, b: &UiWebImeTraceEntryV1) -> bool {
    a.step_index == b.step_index && a.note == b.note
}

pub(super) fn push_web_ime_trace(
    trace: &mut Vec<UiWebImeTraceEntryV1>,
    entry: UiWebImeTraceEntryV1,
) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| web_ime_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_WEB_IME_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_WEB_IME_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

pub(super) fn push_ime_event_trace(
    trace: &mut Vec<UiImeEventTraceEntryV1>,
    entry: UiImeEventTraceEntryV1,
) {
    trace.push(entry);
    if trace.len() > MAX_IME_EVENT_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_IME_EVENT_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

// --- Extracted from `ui_diagnostics.rs` (fearless refactor) ---

impl UiDiagnosticsService {
    pub fn drive_script_for_window(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        window_bounds: Rect,
        scale_factor: f32,
        mut ui: Option<&mut UiTree<App>>,
        semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ) -> UiScriptFrameOutput {
        if !self.is_enabled() {
            return UiScriptFrameOutput::default();
        }

        self.note_window_seen(window);
        let text_font_stack_key_stable_frames =
            self.update_text_font_stack_key_stability(app, window);
        let font_catalog_populated = app
            .global::<fret_runtime::FontCatalog>()
            .is_some_and(|catalog| !catalog.families.is_empty());
        let system_font_rescan_idle = match app.global::<fret_runtime::SystemFontRescanState>() {
            Some(state) => !state.in_flight && !state.pending,
            None => true,
        };

        self.ensure_ready_file();
        self.poll_script_trigger();

        let devtools_request_redraw =
            self.drive_devtools_requests_for_window(app, window, scale_factor, ui.as_deref());

        self.maybe_start_pending_script(app, window);

        self.maybe_migrate_single_active_script_to_window(app, window);

        let Some(mut active) = self.active_scripts.remove(&window) else {
            return self.script_output_for_non_active_window(app, devtools_request_redraw);
        };

        self.maybe_write_running_heartbeat_for_active_window(window, &mut active);

        self.maybe_cancel_pending_cross_window_drag(app, window, &mut active);

        let element_runtime = app.global::<ElementRuntime>();

        if active.next_step >= active.steps.len() {
            let passed_step_index = active.steps.len().saturating_sub(1) as u32;
            push_script_event_log(
                &mut active,
                &self.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "script_passed".to_string(),
                    step_index: Some(passed_step_index),
                    note: Some("script_already_complete".to_string()),
                    bundle_dir: None,
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Passed,
                step_index: Some(passed_step_index),
                reason_code: None,
                reason: None,
                evidence: script_evidence_for_active(&active),
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
                last_bundle_artifact: self.last_dump_artifact_stats.clone(),
            });
            return UiScriptFrameOutput {
                request_redraw: devtools_request_redraw,
                ..UiScriptFrameOutput::default()
            };
        }

        self.maybe_write_running_progress_for_active_window(window, &mut active);

        if active.wait_frames_remaining > 0 {
            active.wait_frames_remaining = active.wait_frames_remaining.saturating_sub(1);
            self.active_scripts.insert(window, active);
            return UiScriptFrameOutput {
                request_redraw: true,
                ..UiScriptFrameOutput::default()
            };
        }

        let prev_next_step = active.next_step;
        let step_index = active.next_step;
        let step = active.steps.get(step_index).cloned();
        let Some(step) = step else {
            return UiScriptFrameOutput {
                request_redraw: devtools_request_redraw,
                ..UiScriptFrameOutput::default()
            };
        };

        let (step_index_u32, step_kind) =
            self.note_step_start_and_scope_evidence(app, window, step_index, &step, &mut active);

        let mut output = UiScriptFrameOutput::default();
        output.request_redraw |= devtools_request_redraw;
        let mut force_dump_label: Option<String> = None;
        let mut force_dump_max_snapshots: Option<usize> = None;
        let mut stop_script = false;
        let mut failure_reason: Option<String> = None;
        let mut handoff_to: Option<AppWindowId> = None;
        let anchor_window = active.anchor_window;

        Self::reset_active_script_state_for_step(&mut active, &step);

        let outcome = dispatch_drive_script_step(
            self,
            app,
            window,
            window_bounds,
            anchor_window,
            step_index,
            step,
            scale_factor,
            element_runtime,
            semantics_snapshot,
            &mut ui,
            text_font_stack_key_stable_frames,
            font_catalog_populated,
            system_font_rescan_idle,
            &mut active,
            &mut output,
            &mut force_dump_label,
            &mut force_dump_max_snapshots,
            &mut handoff_to,
            &mut stop_script,
            &mut failure_reason,
        );

        match outcome {
            DriveScriptStepDispatchOutcome::Continue => {}
            DriveScriptStepDispatchOutcome::ReturnOutput => {
                return output;
            }
            DriveScriptStepDispatchOutcome::RequeueActiveAndReturnOutput => {
                self.active_scripts.insert(window, active);
                return output;
            }
        }

        return finalize_drive_script_for_window(
            self,
            app,
            window,
            prev_next_step,
            step_index,
            step_index_u32,
            step_kind,
            active,
            output,
            force_dump_label,
            force_dump_max_snapshots,
            stop_script,
            failure_reason,
            handoff_to,
        );
    }

    fn record_script_event(&mut self, app: &App, window: AppWindowId, event: &Event) {
        let ring = self.per_window.entry(window).or_default();
        ring.update_pointer_position(event);

        let mut recorded = RecordedUiEventV1::from_event(app, window, event, self.cfg.redact_text);
        truncate_string_bytes(&mut recorded.debug, self.cfg.max_debug_string_bytes);
        ring.push_event(&self.cfg, recorded);
    }
}
