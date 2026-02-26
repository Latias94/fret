//! Script runner internal types extracted from `ui_diagnostics.rs`.
//!
//! These types are intentionally `pub(super)` so the parent `ui_diagnostics` module can keep the
//! existing call sites stable while we gradually split the implementation into smaller modules.

use super::*;

#[derive(Debug, Clone)]
pub(super) struct ActiveScript {
    pub(super) steps: Vec<UiActionStepV2>,
    pub(super) run_id: u64,
    pub(super) anchor_window: AppWindowId,
    pub(super) next_step: usize,
    pub(super) event_log: Vec<UiScriptEventLogEntryV1>,
    pub(super) event_log_dropped: u64,
    pub(super) event_log_active_step: Option<u32>,
    pub(super) last_injected_step: Option<u32>,
    pub(super) wait_frames_remaining: u32,
    pub(super) wait_until: Option<WaitUntilState>,
    pub(super) wait_shortcut_routing_trace: Option<WaitShortcutRoutingTraceState>,
    pub(super) wait_overlay_placement_trace: Option<WaitOverlayPlacementTraceState>,
    pub(super) screenshot_wait: Option<ScreenshotWaitState>,
    pub(super) v2_step_state: Option<V2StepState>,
    pub(super) pointer_session: Option<V2PointerSessionState>,
    pub(super) pending_cancel_cross_window_drag: Option<PendingCancelCrossWindowDrag>,
    pub(super) last_reported_step: Option<usize>,
    pub(super) last_reported_unix_ms: u64,
    pub(super) selector_resolution_trace: Vec<UiSelectorResolutionTraceEntryV1>,
    pub(super) hit_test_trace: Vec<UiHitTestTraceEntryV1>,
    pub(super) click_stable_trace: Vec<UiClickStableTraceEntryV1>,
    pub(super) bounds_stable_trace: Vec<UiBoundsStableTraceEntryV1>,
    pub(super) focus_trace: Vec<UiFocusTraceEntryV1>,
    pub(super) shortcut_routing_trace: Vec<UiShortcutRoutingTraceEntryV1>,
    pub(super) last_shortcut_routing_seq: u64,
    pub(super) overlay_placement_trace: Vec<UiOverlayPlacementTraceEntryV1>,
    pub(super) web_ime_trace: Vec<UiWebImeTraceEntryV1>,
    pub(super) ime_event_trace: Vec<UiImeEventTraceEntryV1>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingCancelCrossWindowDrag {
    pub(super) pointer_id: PointerId,
    pub(super) remaining_frames: u32,
}

impl PendingCancelCrossWindowDrag {
    // This exists as a diagnostics-only escape hatch: when scripted playback migrates across
    // windows during a captured-pointer drag, the synthetic `PointerUp` can land in a different
    // window than the corresponding `PointerDown`. That can leave a dock drag session stuck.
    //
    // Keep the retry window bounded so we don't accidentally cancel *future* drags started by
    // later script steps.
    const RETRY_FRAMES: u32 = 12;

    pub(super) fn new(pointer_id: PointerId) -> Self {
        Self {
            pointer_id,
            remaining_frames: Self::RETRY_FRAMES,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct PendingScript {
    pub(super) steps: Vec<UiActionStepV2>,
    pub(super) legacy_schema_v1: bool,
}

impl PendingScript {
    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn from_json_value(value: serde_json::Value) -> Option<Self> {
        let schema_version: u32 = value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .min(u32::MAX as u64) as u32;

        match schema_version {
            1 => serde_json::from_value::<UiActionScriptV1>(value)
                .ok()
                .and_then(Self::from_v1),
            2 => serde_json::from_value::<UiActionScriptV2>(value)
                .ok()
                .and_then(Self::from_v2),
            _ => None,
        }
    }

    pub(super) fn from_v1(script: UiActionScriptV1) -> Option<Self> {
        if script.schema_version != 1 {
            return None;
        }
        Some(Self {
            steps: script.steps.into_iter().map(UiActionStepV2::from).collect(),
            legacy_schema_v1: true,
        })
    }

    pub(super) fn from_v2(script: UiActionScriptV2) -> Option<Self> {
        if script.schema_version != 2 {
            return None;
        }
        Some(Self {
            steps: script.steps,
            legacy_schema_v1: false,
        })
    }
}

#[derive(Debug, Clone)]
pub(super) enum V2StepState {
    ClickStable(V2ClickStableState),
    ClickSelectableTextSpanStable(V2ClickSelectableTextSpanStableState),
    WaitBoundsStable(V2WaitBoundsStableState),
    EnsureVisible(V2EnsureVisibleState),
    ScrollIntoView(V2ScrollIntoViewState),
    TypeTextInto(V2TypeTextIntoState),
    MenuSelect(V2MenuSelectState),
    MenuSelectPath(V2MenuSelectPathState),
    DragPointer(V2DragPointerState),
    DragPointerUntil(V2DragPointerUntilState),
    DragTo(V2DragToState),
    SetSliderValue(V2SetSliderValueState),
    PointerMove(V2PointerMoveState),
    MovePointerSweep(V2MovePointerSweepState),
}

#[derive(Debug, Clone)]
pub(super) struct V2PointerSessionState {
    pub(super) window: AppWindowId,
    pub(super) button: UiMouseButtonV1,
    pub(super) modifiers: Modifiers,
    pub(super) position: Point,
}

#[derive(Debug, Clone)]
pub(super) struct V2PointerMoveState {
    pub(super) step_index: usize,
    pub(super) steps: u32,
    pub(super) start: Point,
    pub(super) end: Point,
    pub(super) frame: u32,
}

#[derive(Debug, Clone)]
pub(super) struct V2ClickStableState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) stable_count: u32,
    pub(super) last_center: Option<Point>,
}

#[derive(Debug, Clone)]
pub(super) struct V2ClickSelectableTextSpanStableState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) stable_count: u32,
    pub(super) last_pos: Option<Point>,
}

#[derive(Debug, Clone)]
pub(super) struct V2WaitBoundsStableState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) stable_count: u32,
    pub(super) last_bounds: Option<fret_core::Rect>,
}

#[derive(Debug, Clone)]
pub(super) struct V2EnsureVisibleState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
}

#[derive(Debug, Clone)]
pub(super) struct V2ScrollIntoViewState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
}

#[derive(Debug, Clone)]
pub(super) struct V2TypeTextIntoState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) phase: u32,
    pub(super) expected_node_id: Option<u64>,
    pub(super) expected_test_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct V2MenuSelectState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) phase: u32,
}

#[derive(Debug, Clone)]
pub(super) struct V2MenuSelectPathState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) phase: u32,
    pub(super) next_index: usize,
}

#[derive(Debug, Clone)]
pub(super) struct V2DragPointerState {
    pub(super) step_index: usize,
    /// The window that owns this synthetic pointer session.
    ///
    /// Diagnostics drag playback intentionally behaves like a captured pointer: we keep emitting
    /// `Down/Move/Up` into the same window for the duration of the step, even if the runtime's
    /// notion of a "current window" changes mid-drag (e.g. multi-window docking tear-off).
    pub(super) window: AppWindowId,
    /// Total move segments (not counting the initial `move+down` frame and the final `up` frame).
    pub(super) steps: u32,
    pub(super) button: UiMouseButtonV1,
    pub(super) start: Point,
    pub(super) end: Point,
    /// Playback cursor:
    /// - `0`: emit `move+down` at `start`
    /// - `1..=steps`: emit a pressed `move` (and `InternalDrag::Over`) at interpolated positions
    /// - `steps + 1`: emit `up` at `end` (and `InternalDrag::Drop`)
    pub(super) frame: u32,
}

#[derive(Debug, Clone)]
pub(super) struct V2DragPointerUntilState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) playback: V2DragPointerState,
    pub(super) predicate: UiPredicateV1,
    /// If true, the step has issued a pointer down and should release on completion.
    pub(super) down_issued: bool,
    /// If true, the step has staged a "route cursor to source window" override and will emit the
    /// `Up/Drop` events on the next frame to avoid runner override polling latency.
    pub(super) release_armed: bool,
}

#[derive(Debug, Clone)]
pub(super) struct V2DragToState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) playback: Option<V2DragPointerState>,
}

#[derive(Debug, Clone)]
pub(super) struct V2SetSliderValueState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) phase: u32,
    pub(super) last_drag_x: Option<f32>,
}

#[derive(Debug, Clone)]
pub(super) struct V2MovePointerSweepState {
    pub(super) step_index: usize,
    pub(super) start: Point,
    pub(super) end: Point,
    pub(super) steps: u32,
    pub(super) next_step: u32,
    pub(super) frames_per_step: u32,
    pub(super) wait_frames_remaining: u32,
}

#[derive(Debug, Clone)]
pub(super) struct WaitUntilState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
}

#[derive(Debug, Clone)]
pub(super) struct WaitShortcutRoutingTraceState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) start_frame_id: u64,
}

#[derive(Debug, Clone)]
pub(super) struct WaitOverlayPlacementTraceState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
}

#[derive(Debug, Clone)]
pub(super) struct ScreenshotWaitState {
    pub(super) step_index: usize,
    pub(super) remaining_frames: u32,
    pub(super) request_id: String,
    pub(super) window_ffi: u64,
}
