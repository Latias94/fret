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
