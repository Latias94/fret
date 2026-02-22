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
