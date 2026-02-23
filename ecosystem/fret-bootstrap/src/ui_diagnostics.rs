use fret_app::{App, Effect, ModelId};
use fret_core::{
    AppWindowId, Event, ImeEvent, KeyCode, Modifiers, MouseButton, MouseButtons, NodeId, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, SemanticsRole,
};
#[cfg(feature = "diagnostics-ws")]
use fret_diag_protocol::{
    DevtoolsAppExitRequestV1, DevtoolsBundleDumpV1, DevtoolsBundleDumpedV1,
    DevtoolsScreenshotRequestV1, DevtoolsScreenshotResultV1, DiagTransportMessageV1,
    UiSemanticsNodeGetAckV1, UiSemanticsNodeGetV1,
};
use fret_diag_protocol::{
    UiActionScriptV1, UiActionScriptV2, UiActionStepV2, UiArtifactStatsV1,
    UiBoundsStableTraceEntryV1, UiClickStableTraceEntryV1, UiEdgesV1, UiFocusTraceEntryV1,
    UiHitTestScopeRootEvidenceV1, UiHitTestTraceEntryV1, UiImeEventTraceEntryV1, UiImeEventV1,
    UiIncomingOpenInjectItemV1, UiInspectConfigV1, UiKeyModifiersV1, UiLayoutDirectionV1,
    UiMouseButtonV1, UiOptionalRootStateV1, UiOverlayAlignV1, UiOverlayArrowLayoutV1,
    UiOverlayOffsetV1, UiOverlayPlacementTraceEntryV1, UiOverlayPlacementTraceKindV1,
    UiOverlayPlacementTraceQueryV1, UiOverlayShiftV1, UiOverlaySideV1, UiOverlayStickyModeV1,
    UiPaddingInsetsV1, UiPointV1, UiPredicateV1, UiRectV1, UiRoleAndNameV1,
    UiScriptEventLogEntryV1, UiScriptEvidenceV1, UiScriptResultV1, UiScriptStageV1,
    UiSelectorResolutionCandidateV1, UiSelectorResolutionTraceEntryV1, UiSelectorV1,
    UiShortcutRoutingTraceEntryV1, UiShortcutRoutingTraceQueryV1, UiSizeV1, UiTextInputSnapshotV1,
    UiWebImeTraceEntryV1, UiWindowTargetV1,
};
use fret_runtime::DragHost as _;
use fret_ui::elements::ElementRuntime;
use fret_ui::{Invalidation, UiDebugFrameStats, UiDebugHitTest, UiDebugLayerInfo, UiTree};
use serde::{Deserialize, Serialize};
use slotmap::{Key as _, KeyData};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature = "diagnostics-ws")]
use crate::ui_diagnostics_ws_bridge::UiDiagnosticsWsBridge;

mod semantics;
pub use semantics::{
    UiSemanticsActionsV1, UiSemanticsFlagsV1, UiSemanticsNodeV1, UiSemanticsRootV1,
    UiSemanticsSnapshotV1,
};

mod bundle;
pub use bundle::{
    UiDiagnosticsBundleConfigV1, UiDiagnosticsBundleTablesV1, UiDiagnosticsBundleV1,
    UiDiagnosticsBundleV2, UiDiagnosticsEnvDiagnosticsV1, UiDiagnosticsEnvFingerprintV1,
    UiDiagnosticsWindowBundleV1,
};

mod bundle_index;

mod bundle_dump;
mod bundle_dump_policy;
mod fs_triggers;
mod inspect;
mod pick;
mod pick_flow;
mod test_id_bloom;
pub(crate) use pick::pick_semantics_node_by_bounds;
use pick::{pick_best_match, pick_semantics_node_at};
mod script_engine;
use script_engine::{
    overlay_placement_trace_entry_matches_query,
    overlay_placement_trace_entry_matches_query_any_step, push_click_stable_trace,
    push_focus_trace, push_ime_event_trace, push_overlay_placement_trace, push_script_event_log,
    push_shortcut_routing_trace, push_web_ime_trace, script_evidence_for_active,
    script_step_kind_name, shortcut_routing_trace_entry_matches_query,
};
mod script_result;
mod script_runner;
mod script_step_index;
mod script_steps;
mod script_steps_assert;
mod script_steps_drag;
mod script_steps_input;
mod script_steps_menu;
mod script_steps_pointer;
mod script_steps_pointer_session;
mod script_steps_pointer_sweep;
mod script_steps_scroll;
mod script_steps_slider;
mod script_steps_visibility;
mod script_steps_wait;
mod script_types;
use script_types::*;

mod selector;
use selector::SemanticsIndex;
pub(crate) use selector::semantics_role_label;
use selector::{
    best_selector_for_node, format_inspect_path, parent_node_id, parse_semantics_role,
    select_semantics_node, suggest_selectors, truncate_debug_value,
};

// Split out the DevTools WS wiring to reduce churn in this file.
#[path = "ui_diagnostics/ui_diagnostics_devtools_ws.rs"]
mod ui_diagnostics_devtools_ws;

mod config;
pub use config::UiDiagnosticsConfig;

#[derive(Default)]
pub struct UiDiagnosticsService {
    cfg: UiDiagnosticsConfig,
    per_window: HashMap<AppWindowId, WindowRing>,
    text_font_stack_key_stability: HashMap<AppWindowId, TextFontStackKeyStability>,
    known_windows: Vec<AppWindowId>,
    last_trigger_stamp: Option<u64>,
    last_script_trigger_stamp: Option<u64>,
    last_pick_trigger_mtime: Option<std::time::SystemTime>,
    last_inspect_trigger_mtime: Option<std::time::SystemTime>,
    exit_armed: bool,
    exit_last_mtime: Option<std::time::SystemTime>,
    ws_exit_deadline_unix_ms: Option<u64>,
    ready_written: bool,
    ready_write_warned: bool,
    capabilities_written: bool,
    capabilities_write_warned: bool,
    inspect_enabled: bool,
    inspect_consume_clicks: bool,
    pending_script: Option<PendingScript>,
    pending_script_run_id: Option<u64>,
    active_scripts: HashMap<AppWindowId, ActiveScript>,
    pending_force_dump: Option<fs_triggers::PendingForceDumpRequest>,
    last_dump_dir: Option<PathBuf>,
    last_dump_artifact_stats: Option<UiArtifactStatsV1>,
    last_script_run_id: u64,
    last_pick_run_id: u64,
    last_picked_node_id: HashMap<AppWindowId, u64>,
    last_picked_selector_json: HashMap<AppWindowId, String>,
    last_hovered_node_id: HashMap<AppWindowId, u64>,
    last_hovered_selector_json: HashMap<AppWindowId, String>,
    inspect_focus_node_id: HashMap<AppWindowId, u64>,
    inspect_focus_selector_json: HashMap<AppWindowId, String>,
    inspect_focus_down_stack: HashMap<AppWindowId, Vec<u64>>,
    inspect_pending_nav: HashMap<AppWindowId, inspect::InspectNavCommand>,
    inspect_focus_summary_line: HashMap<AppWindowId, String>,
    inspect_focus_path_line: HashMap<AppWindowId, String>,
    inspect_locked_windows: HashSet<AppWindowId>,
    inspect_toast: HashMap<AppWindowId, inspect::InspectToast>,
    pick_overlay_grace_frames: HashMap<AppWindowId, u32>,
    pick_armed_run_id: Option<u64>,
    pending_pick: Option<PendingPick>,
    app_snapshot_provider:
        Option<Arc<dyn Fn(&App, AppWindowId) -> Option<serde_json::Value> + 'static>>,
    #[cfg(feature = "diagnostics-ws")]
    pending_devtools_screenshot: Option<PendingDevtoolsScreenshotRequest>,
    #[cfg(feature = "diagnostics-ws")]
    pending_devtools_semantics_node_get: Option<PendingDevtoolsSemanticsNodeGetRequest>,
    #[cfg(feature = "diagnostics-ws")]
    ws_bridge: UiDiagnosticsWsBridge,
}

#[cfg(feature = "diagnostics-ws")]
#[derive(Debug, Clone)]
struct PendingDevtoolsScreenshotRequest {
    request_id: Option<u64>,
    request_id_str: String,
    label: Option<String>,
    timeout_frames: u32,
    window_ffi: u64,
    bundle_dir_name: Option<String>,
    remaining_frames: u32,
    last_result_trigger_stamp: Option<u64>,
    started: bool,
}

#[cfg(feature = "diagnostics-ws")]
#[derive(Debug, Clone)]
struct PendingDevtoolsSemanticsNodeGetRequest {
    request_id: Option<u64>,
    window_ffi: u64,
    node_id: u64,
}

#[derive(Debug, Default, Clone, Copy)]
struct TextFontStackKeyStability {
    last_key: Option<u64>,
    stable_frames: u32,
}

impl UiDiagnosticsService {
    fn is_wasm_ws_only(&self) -> bool {
        cfg!(target_arch = "wasm32") && self.ws_is_configured()
    }

    pub fn known_windows(&self) -> &[AppWindowId] {
        &self.known_windows
    }

    fn poll_ws_inbox_and_is_wasm_ws_only(&mut self) -> bool {
        self.poll_ws_inbox();
        self.is_wasm_ws_only()
    }

    fn note_window_seen(&mut self, window: AppWindowId) {
        if self.known_windows.contains(&window) {
            return;
        }
        self.known_windows.push(window);
    }

    fn resolve_window_target(
        &self,
        current_window: AppWindowId,
        target: Option<&UiWindowTargetV1>,
    ) -> Option<AppWindowId> {
        let first_seen = self
            .known_windows
            .iter()
            .copied()
            .min_by_key(|w| w.data().as_ffi());
        let last_seen = self
            .known_windows
            .iter()
            .copied()
            .max_by_key(|w| w.data().as_ffi());
        match target.copied().unwrap_or(UiWindowTargetV1::Current) {
            UiWindowTargetV1::Current => Some(current_window),
            UiWindowTargetV1::FirstSeen => first_seen,
            UiWindowTargetV1::FirstSeenOther => self
                .known_windows
                .iter()
                .copied()
                .filter(|w| *w != current_window)
                .min_by_key(|w| w.data().as_ffi()),
            UiWindowTargetV1::LastSeen => last_seen,
            UiWindowTargetV1::LastSeenOther => self
                .known_windows
                .iter()
                .copied()
                .filter(|w| *w != current_window)
                .max_by_key(|w| w.data().as_ffi()),
            UiWindowTargetV1::WindowFfi { window } => {
                let want = AppWindowId::from(KeyData::from_ffi(window));
                self.known_windows.contains(&want).then_some(want)
            }
        }
    }

    fn resolve_window_target_for_active_step(
        &self,
        current_window: AppWindowId,
        anchor_window: AppWindowId,
        target: Option<&UiWindowTargetV1>,
    ) -> Option<AppWindowId> {
        let Some(target) = target else {
            return Some(current_window);
        };

        match target {
            UiWindowTargetV1::Current => Some(current_window),
            UiWindowTargetV1::FirstSeen => Some(anchor_window),
            _ => self.resolve_window_target(anchor_window, Some(target)),
        }
    }

    fn predicate_can_eval_off_window(predicate: &UiPredicateV1) -> bool {
        matches!(
            predicate,
            UiPredicateV1::KnownWindowCountGe { .. }
                | UiPredicateV1::KnownWindowCountIs { .. }
                | UiPredicateV1::PlatformUiWindowHoverDetectionIs { .. }
                | UiPredicateV1::DockDragCurrentWindowIs { .. }
                | UiPredicateV1::DockDragMovingWindowIs { .. }
                | UiPredicateV1::DockDragWindowUnderMovingWindowIs { .. }
                | UiPredicateV1::DockDragActiveIs { .. }
                | UiPredicateV1::DockDragTransparentPayloadAppliedIs { .. }
                | UiPredicateV1::DockDragTransparentPayloadMousePassthroughAppliedIs { .. }
                | UiPredicateV1::DockDragWindowUnderCursorSourceIs { .. }
                | UiPredicateV1::DockDragWindowUnderMovingWindowSourceIs { .. }
                | UiPredicateV1::DockFloatingDragActiveIs { .. }
                | UiPredicateV1::DockDropPreviewKindIs { .. }
                | UiPredicateV1::DockDropResolveSourceIs { .. }
                | UiPredicateV1::DockDropResolvedIsSome { .. }
                | UiPredicateV1::DockGraphCanonicalIs { .. }
                | UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { .. }
                | UiPredicateV1::DockGraphNodeCountLe { .. }
                | UiPredicateV1::DockGraphMaxSplitDepthLe { .. }
                | UiPredicateV1::DockGraphSignatureIs { .. }
                | UiPredicateV1::DockGraphSignatureContains { .. }
                | UiPredicateV1::DockGraphSignatureFingerprint64Is { .. }
        )
    }

    fn preferred_window_for_active_script(active: &ActiveScript) -> Option<AppWindowId> {
        if let Some(step) = active.steps.get(active.next_step) {
            match step {
                UiActionStepV2::WaitUntil { predicate, .. }
                | UiActionStepV2::Assert { predicate, .. }
                    if Self::predicate_can_eval_off_window(predicate) =>
                {
                    // Avoid pinning scripts to a specific window during "read-only" docking
                    // assertions / waits. Overlap + occlusion can prevent the target window from
                    // producing frames, so allowing migration keeps timeouts and gates progressing.
                    return None;
                }
                _ => {}
            }

            // Avoid migrating a newly started script before any per-window state is established.
            // The first few steps typically establish window geometry and must run consistently.
            if active.next_step == 0 {
                return Some(active.anchor_window);
            }

            // Before a step caches any per-window state (pointer session / v2 step state), we may
            // still need to "pin" execution to a specific window to avoid migration loops.
            //
            // Example: a window-targeted drag step (`drag_pointer_until`) can be repeatedly stolen
            // by any window that happens to be producing frames. If the step keeps handing off to
            // its intended window without ever initializing playback, timeouts may never decrement
            // and tooling can hang waiting for `script.result.json` to complete.
            //
            // Prefer a stable window when the step targets:
            // - `first_seen` (use the script's `anchor_window`)
            // - `window_ffi` (resolve directly)
            //
            // Leave other relative targets (last_seen/other) migratable until per-window state is
            // established; those depend on `known_windows`, which is maintained at runtime.
            let step_window_target: Option<&UiWindowTargetV1> = match step {
                UiActionStepV2::Click { window, .. }
                | UiActionStepV2::PointerDown { window, .. }
                | UiActionStepV2::PointerMove { window, .. }
                | UiActionStepV2::PointerUp { window, .. }
                | UiActionStepV2::DragPointer { window, .. }
                | UiActionStepV2::DragPointerUntil { window, .. }
                | UiActionStepV2::DragTo { window, .. }
                | UiActionStepV2::SetWindowInnerSize { window, .. }
                | UiActionStepV2::SetWindowOuterPosition { window, .. }
                | UiActionStepV2::SetCursorInWindow { window, .. }
                | UiActionStepV2::SetCursorInWindowLogical { window, .. }
                | UiActionStepV2::SetMouseButtons { window, .. }
                | UiActionStepV2::RaiseWindow { window, .. }
                | UiActionStepV2::WaitUntil { window, .. }
                | UiActionStepV2::Assert { window, .. } => window.as_ref(),
                _ => None,
            };
            match step_window_target.copied() {
                Some(UiWindowTargetV1::FirstSeen) => return Some(active.anchor_window),
                Some(UiWindowTargetV1::WindowFfi { window }) => {
                    return Some(AppWindowId::from(KeyData::from_ffi(window)));
                }
                _ => {}
            }
        }

        if let Some(session) = active.pointer_session.as_ref() {
            return Some(session.window);
        }
        match active.v2_step_state.as_ref()? {
            V2StepState::DragPointer(state) => Some(state.window),
            V2StepState::DragPointerUntil(state) => Some(state.playback.window),
            V2StepState::DragTo(state) => state.playback.as_ref().map(|p| p.window),
            _ => None,
        }
    }

    fn active_step_window_target(active: &ActiveScript) -> Option<UiWindowTargetV1> {
        let step = active.steps.get(active.next_step)?;
        let step_window_target: Option<&UiWindowTargetV1> = match step {
            UiActionStepV2::Click { window, .. }
            | UiActionStepV2::PointerDown { window, .. }
            | UiActionStepV2::PointerMove { window, .. }
            | UiActionStepV2::PointerUp { window, .. }
            | UiActionStepV2::DragPointer { window, .. }
            | UiActionStepV2::DragPointerUntil { window, .. }
            | UiActionStepV2::DragTo { window, .. }
            | UiActionStepV2::SetWindowInnerSize { window, .. }
            | UiActionStepV2::SetWindowOuterPosition { window, .. }
            | UiActionStepV2::SetCursorInWindow { window, .. }
            | UiActionStepV2::SetCursorInWindowLogical { window, .. }
            | UiActionStepV2::SetMouseButtons { window, .. }
            | UiActionStepV2::RaiseWindow { window, .. }
            | UiActionStepV2::WaitUntil { window, .. }
            | UiActionStepV2::Assert { window, .. } => window.as_ref(),
            _ => None,
        };
        step_window_target.copied()
    }

    fn remap_script_per_window_state_for_migration(
        active: &mut ActiveScript,
        new_window: AppWindowId,
        allow_remap_captured_drag: bool,
    ) {
        if let Some(session) = active.pointer_session.as_mut() {
            session.window = new_window;
        }
        if let Some(state) = active.v2_step_state.as_mut() {
            match state {
                V2StepState::DragPointer(state) => state.window = new_window,
                V2StepState::DragPointerUntil(state) => {
                    // Avoid splitting a captured-pointer gesture across windows. `drag_pointer_until`
                    // is allowed to "hold" the drag across frames; once we've emitted a down/move
                    // segment, keep injecting into the original playback window unless the runner
                    // has migrated the captured drag to a different window (ImGui-style tear-off).
                    if (!state.down_issued && state.playback.frame == 0)
                        || allow_remap_captured_drag
                    {
                        state.playback.window = new_window;
                    }
                }
                V2StepState::DragTo(state) => {
                    if let Some(playback) = state.playback.as_mut() {
                        playback.window = new_window;
                    }
                }
                _ => {}
            }
        }
    }

    fn can_migrate_for_current_target(active: &ActiveScript) -> bool {
        if !matches!(
            Self::active_step_window_target(active),
            Some(UiWindowTargetV1::Current)
        ) {
            return false;
        }

        // Avoid splitting a captured-pointer gesture across windows. After a drag step has issued
        // a pointer down, migrating execution to a different window would cause the corresponding
        // pointer up to land in the wrong window and leave the original runtime drag state stuck.
        match active.v2_step_state.as_ref() {
            None => true,
            Some(V2StepState::DragPointerUntil(state))
                if state.step_index == active.next_step
                    && !state.down_issued
                    && state.playback.frame == 0 =>
            {
                true
            }
            Some(V2StepState::DragPointer(state))
                if state.step_index == active.next_step && state.frame == 0 =>
            {
                true
            }
            Some(V2StepState::DragTo(state))
                if state.step_index == active.next_step && state.playback.is_none() =>
            {
                true
            }
            _ => false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.cfg.enabled
    }

    fn update_text_font_stack_key_stability(&mut self, app: &App, window: AppWindowId) -> u32 {
        let key = app.global::<fret_runtime::TextFontStackKey>().map(|k| k.0);
        let state = self
            .text_font_stack_key_stability
            .entry(window)
            .or_default();

        match (key, state.last_key) {
            (Some(key), Some(prev)) if key == prev => {
                state.stable_frames = state.stable_frames.saturating_add(1);
            }
            (Some(key), _) => {
                state.last_key = Some(key);
                state.stable_frames = 0;
            }
            (None, _) => {
                state.last_key = None;
                state.stable_frames = 0;
            }
        }

        state.stable_frames
    }

    /// Returns the index of the next script step to execute for `window`, if a script is active.
    ///
    /// This is intended for diag-only app logic that wants to run after a particular scripted
    /// step has completed (e.g. "after the baseline screenshot").
    pub fn active_script_next_step_index(&self, window: AppWindowId) -> Option<u32> {
        self.active_scripts
            .get(&window)
            .map(|active| active.next_step.min(u32::MAX as usize) as u32)
    }

    pub fn set_app_snapshot_provider(
        &mut self,
        provider: Option<Arc<dyn Fn(&App, AppWindowId) -> Option<serde_json::Value> + 'static>>,
    ) {
        self.app_snapshot_provider = provider;
    }

    /// Returns `true` if the current diagnostics state would benefit from (or requires) a fresh
    /// semantics snapshot for `window` on this frame.
    ///
    /// This is a performance knob: semantics snapshots are expensive, and many scripted steps only
    /// need semantics for *initial target resolution*. Once a step has cached its target geometry
    /// (via `v2_step_state`), we can often skip requesting semantics until a selector-based step is
    /// about to run again.
    pub fn wants_semantics_snapshot(&mut self, window: AppWindowId) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.note_window_seen(window);

        self.poll_pick_trigger();
        self.poll_inspect_trigger();
        self.poll_script_trigger();

        if self.cfg.capture_semantics {
            return true;
        }

        if self.pick_armed_run_id.is_some()
            || self
                .pending_pick
                .as_ref()
                .is_some_and(|p| p.window == window)
            || self.inspect_enabled
            || self.inspect_locked_windows.contains(&window)
            || self.inspect_toast.contains_key(&window)
        {
            return true;
        }

        if self.pending_script.is_some() {
            return true;
        }

        self.active_scripts
            .get(&window)
            .is_some_and(script_engine::active_script_needs_semantics_snapshot)
    }

    pub fn redact_text(&self) -> bool {
        self.cfg.redact_text
    }

    pub fn last_pointer_position(&self, window: AppWindowId) -> Option<Point> {
        self.per_window
            .get(&window)
            .and_then(|ring| ring.last_pointer_position)
    }

    pub fn last_picked_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.last_picked_node_id.get(&window).copied()
    }

    pub fn pick_is_armed(&self) -> bool {
        self.pick_armed_run_id.is_some()
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.per_window.remove(&window);
        self.known_windows.retain(|w| *w != window);
        self.active_scripts.remove(&window);
        self.last_picked_node_id.remove(&window);
        self.last_picked_selector_json.remove(&window);
        self.last_hovered_node_id.remove(&window);
        self.last_hovered_selector_json.remove(&window);
        self.inspect_focus_node_id.remove(&window);
        self.inspect_focus_selector_json.remove(&window);
        self.inspect_focus_down_stack.remove(&window);
        self.inspect_pending_nav.remove(&window);
        self.inspect_focus_summary_line.remove(&window);
        self.inspect_focus_path_line.remove(&window);
        self.inspect_locked_windows.remove(&window);
        self.inspect_toast.remove(&window);
        if self
            .pending_pick
            .as_ref()
            .is_some_and(|p| p.window == window)
        {
            self.pending_pick = None;
        }
    }

    fn reset_diagnostics_ring_for_window(&mut self, window: AppWindowId) {
        self.per_window.entry(window).or_default().clear();
    }

    pub fn record_model_changes(&mut self, window: AppWindowId, changed: &[ModelId]) {
        if !self.is_enabled() {
            return;
        }
        let ring = self.per_window.entry(window).or_default();
        ring.last_changed_models = changed.iter().map(|id| id.data().as_ffi()).collect();
    }

    pub fn record_global_changes(
        &mut self,
        app: &App,
        window: AppWindowId,
        changed: &[std::any::TypeId],
    ) {
        if !self.is_enabled() {
            return;
        }
        let ring = self.per_window.entry(window).or_default();
        ring.last_changed_globals = changed
            .iter()
            .map(|&t| {
                app.global_type_name(t)
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| format!("{t:?}"))
            })
            .collect();
    }

    pub fn record_event(&mut self, app: &App, window: AppWindowId, event: &Event) {
        if !self.is_enabled() {
            return;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let ring = self.per_window.entry(window).or_default();
        ring.update_pointer_position(event);

        let mut recorded = RecordedUiEventV1::from_event(app, window, event, self.cfg.redact_text);
        truncate_string_bytes(&mut recorded.debug, self.cfg.max_debug_string_bytes);
        ring.push_event(&self.cfg, recorded);

        if let Some(active) = self.active_scripts.get_mut(&window)
            && let Event::Ime(ime) = event
        {
            let step_index = active
                .last_injected_step
                .unwrap_or_else(|| active.next_step.min(u32::MAX as usize) as u32);
            record_ime_event_trace(&mut active.ime_event_trace, step_index, "record_event", ime);
        }
    }

    pub fn record_viewport_input(&mut self, event: fret_core::ViewportInputEvent) {
        if !self.is_enabled() {
            return;
        }

        let ring = self.per_window.entry(event.window).or_default();
        if ring.viewport_input_this_frame.len() >= self.cfg.max_events {
            return;
        }
        ring.viewport_input_this_frame
            .push(UiViewportInputEventV1::from_event(event));
    }

    pub fn record_snapshot(
        &mut self,
        app: &App,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
        ui: &mut UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        scene: &Scene,
    ) {
        if !self.is_enabled() {
            return;
        }

        let last_pointer_position = self
            .per_window
            .get(&window)
            .and_then(|ring| ring.last_pointer_position);
        let hit_test = last_pointer_position.map(|pos| UiHitTestSnapshotV1::from_tree(pos, ui));

        let element_diag = element_runtime.and_then(|runtime| {
            runtime.diagnostics_snapshot(window).map(|snapshot| {
                ElementDiagnosticsSnapshotV1::from_runtime(
                    window,
                    runtime,
                    snapshot,
                    self.cfg.max_debug_string_bytes,
                )
            })
        });

        let raw_semantics = ui.semantics_snapshot();
        let semantics_fingerprint = raw_semantics.map(|snapshot| {
            semantics_fingerprint_v1(
                snapshot,
                self.cfg.redact_text,
                self.cfg.max_debug_string_bytes,
            )
        });

        if self.inspect_enabled {
            let hovered = last_pointer_position.and_then(|pos| {
                raw_semantics.and_then(|snap| {
                    pick_semantics_node_by_bounds(snap, pos).map(|n| n.id.data().as_ffi())
                })
            });
            self.update_inspect_hover(window, raw_semantics, hovered, element_runtime);
        }
        self.apply_inspect_navigation(window, raw_semantics, element_runtime);
        self.update_inspect_focus_lines(window, raw_semantics, element_runtime);

        let semantics = self
            .cfg
            .capture_semantics
            .then_some(raw_semantics)
            .flatten()
            .map(|snap| {
                UiSemanticsSnapshotV1::from_snapshot(
                    snap,
                    self.cfg.redact_text,
                    self.cfg.max_debug_string_bytes,
                    self.cfg.max_semantics_nodes,
                    self.cfg.semantics_test_ids_only,
                )
            });

        let ring = self.per_window.entry(window).or_default();
        let viewport_input = std::mem::take(&mut ring.viewport_input_this_frame);

        let changed_models = std::mem::take(&mut ring.last_changed_models);
        let changed_model_sources_top = if cfg!(debug_assertions) && !changed_models.is_empty() {
            let mut counts: HashMap<(String, String, u32, u32), u32> = HashMap::new();
            for &model in &changed_models {
                let id = ModelId::from(KeyData::from_ffi(model));
                let Some(info) = app.models().debug_last_changed_info_for_id(id) else {
                    continue;
                };
                let ty = info.type_name.to_string();
                *counts
                    .entry((ty, info.file.to_string(), info.line, info.column))
                    .or_insert(0) += 1;
            }
            let mut out: Vec<UiChangedModelSourceHotspotV1> = counts
                .into_iter()
                .map(
                    |((type_name, file, line, column), count)| UiChangedModelSourceHotspotV1 {
                        type_name,
                        changed_at: UiSourceLocationV1 { file, line, column },
                        count,
                    },
                )
                .collect();
            out.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.type_name.cmp(&b.type_name))
                    .then_with(|| a.changed_at.file.cmp(&b.changed_at.file))
                    .then_with(|| a.changed_at.line.cmp(&b.changed_at.line))
                    .then_with(|| a.changed_at.column.cmp(&b.changed_at.column))
            });
            out.truncate(8);
            out
        } else {
            Vec::new()
        };

        let resource_caches = {
            let icon_svg_cache = icon_svg_cache_stats(app);
            let canvas = canvas_cache_stats_for_window(app, window.data().as_ffi());
            let render_text = app
                .global::<fret_core::RendererTextPerfSnapshot>()
                .copied()
                .map(UiRendererTextPerfSnapshotV1::from_core);
            let render_text_font_trace = app
                .global::<fret_core::RendererTextFontTraceSnapshot>()
                .cloned()
                .map(|s| {
                    UiRendererTextFontTraceSnapshotV1::from_core(
                        s,
                        self.cfg.redact_text,
                        self.cfg.max_debug_string_bytes,
                    )
                });
            let render_text_fallback_policy = app
                .global::<fret_core::RendererTextFallbackPolicySnapshot>()
                .cloned()
                .map(|s| {
                    UiRendererTextFallbackPolicySnapshotV1::from_core(
                        s,
                        self.cfg.max_debug_string_bytes,
                    )
                });
            (icon_svg_cache.is_some()
                || !canvas.is_empty()
                || render_text.is_some()
                || render_text_font_trace.is_some()
                || render_text_fallback_policy.is_some())
            .then_some(UiResourceCachesV1 {
                icon_svg_cache,
                canvas,
                render_text,
                render_text_font_trace,
                render_text_fallback_policy,
            })
        };

        let renderer_perf = app
            .global::<fret_render::RendererPerfFrameStore>()
            .and_then(|store| store.latest_for_window(window));

        let mut debug = UiTreeDebugSnapshotV1::from_tree(
            app,
            window,
            ui,
            renderer_perf,
            element_runtime,
            hit_test,
            element_diag,
            semantics,
            self.cfg.max_gating_trace_entries,
            self.cfg.redact_text,
            self.cfg.max_debug_string_bytes,
        );
        debug.viewport_input = viewport_input;

        let app_snapshot = self
            .app_snapshot_provider
            .as_ref()
            .and_then(|provider| provider(app, window));

        let frame_clock = app
            .global::<fret_core::WindowFrameClockService>()
            .and_then(|svc| {
                let snapshot = svc.snapshot(window)?;
                let fixed_delta_ms = svc.effective_fixed_delta(window).map(|d| {
                    let ms = d.as_millis();
                    ms.min(u64::MAX as u128) as u64
                });
                Some(UiFrameClockSnapshotV1 {
                    now_monotonic_ms: {
                        let ms = snapshot.now_monotonic.as_millis();
                        ms.min(u64::MAX as u128) as u64
                    },
                    delta_ms: {
                        let ms = snapshot.delta.as_millis();
                        ms.min(u64::MAX as u128) as u64
                    },
                    fixed_delta_ms,
                })
            });

        let (safe_area_insets, occlusion_insets) = app
            .global::<fret_core::WindowMetricsService>()
            .map(|svc| {
                (
                    svc.safe_area_insets(window).map(ui_edges_from_edges),
                    svc.occlusion_insets(window).map(ui_edges_from_edges),
                )
            })
            .unwrap_or((None, None));

        let input_ctx = app
            .global::<fret_runtime::WindowInputContextService>()
            .and_then(|svc| svc.snapshot(window));

        let window_text_input_snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window));

        let clipboard = app
            .global::<fret_runtime::WindowClipboardDiagnosticsStore>()
            .and_then(|store| {
                let frame_id = app.frame_id();
                let last_read = store.last_read_for_window(window, frame_id);
                let last_write = store.last_write_for_window(window, frame_id);
                if last_read.is_none() && last_write.is_none() {
                    return None;
                }
                Some(UiClipboardDiagnosticsSnapshotV1 {
                    last_read_token: last_read.map(|e| e.token.0),
                    last_read_unavailable: last_read.map(|e| e.unavailable),
                    last_read_message: last_read.and_then(|e| e.message.clone()),
                    last_write_unavailable: last_write.map(|e| e.unavailable),
                    last_write_message: last_write.and_then(|e| e.message.clone()),
                })
            });

        let wgpu_adapter = app
            .global::<fret_render::WgpuAdapterSelectionSnapshot>()
            .and_then(|snapshot| serde_json::to_value(snapshot).ok());

        let window_snapshot_seq = ring.snapshot_seq;
        ring.snapshot_seq = ring.snapshot_seq.saturating_add(1);

        let snapshot = UiDiagnosticsSnapshotV1 {
            schema_version: 1,
            tick_id: app.tick_id().0,
            frame_id: app.frame_id().0,
            window_snapshot_seq,
            window: window.data().as_ffi(),
            timestamp_unix_ms: unix_ms_now(),
            scale_factor,
            window_bounds: RectV1::from(bounds),
            scene_ops: scene.ops_len() as u64,
            scene_fingerprint: scene.fingerprint(),
            semantics_fingerprint,
            debug,
            frame_clock,
            changed_models,
            changed_globals: std::mem::take(&mut ring.last_changed_globals),
            changed_model_sources_top,
            resource_caches,
            app_snapshot,
            safe_area_insets,
            occlusion_insets,
            focus_is_text_input: input_ctx.map(|c| c.focus_is_text_input),
            is_composing: window_text_input_snapshot.map(|s| s.is_composing),
            clipboard,
            primary_pointer_type: ring
                .last_pointer_type
                .map(|t| viewport_pointer_type_label(t).to_string()),
            caps: input_ctx.map(|c| UiPlatformCapabilitiesSummaryV1 {
                platform: c.platform.as_str().to_string(),
                ui_window_hover_detection: c.caps.ui.window_hover_detection.as_str().to_string(),
                clipboard_text: c.caps.clipboard.text.read && c.caps.clipboard.text.write,
                clipboard_text_read: c.caps.clipboard.text.read,
                clipboard_text_write: c.caps.clipboard.text.write,
                clipboard_primary_text: c.caps.clipboard.primary_text,
                ime: c.caps.ime.enabled,
                ime_set_cursor_area: c.caps.ime.set_cursor_area,
                fs_file_dialogs: c.caps.fs.file_dialogs,
                shell_share_sheet: c.caps.shell.share_sheet,
                shell_incoming_open: c.caps.shell.incoming_open,
            }),
            wgpu_adapter,
        };

        ring.push_snapshot(&self.cfg, snapshot);

        self.record_shortcut_routing_trace_for_window(app, window);

        if let Some(pending) = self.pending_pick.clone()
            && pending.window == window
        {
            self.resolve_pending_pick_for_window(
                window,
                pending.position,
                raw_semantics,
                ui,
                element_runtime,
            );
        }
    }

    fn record_shortcut_routing_trace_for_window(&mut self, app: &App, window: AppWindowId) {
        let Some(active) = self.active_scripts.get_mut(&window) else {
            return;
        };
        let Some(store) = app.global::<fret_runtime::WindowShortcutRoutingDiagnosticsStore>()
        else {
            return;
        };

        let step_index = active
            .last_injected_step
            .unwrap_or_else(|| active.next_step.min(u32::MAX as usize) as u32);

        let max_entries = MAX_SHORTCUT_ROUTING_TRACE_ENTRIES;
        let decisions = store.snapshot_since(window, active.last_shortcut_routing_seq, max_entries);
        if decisions.is_empty() {
            return;
        }

        for decision in decisions {
            active.last_shortcut_routing_seq = active
                .last_shortcut_routing_seq
                .max(decision.seq.saturating_add(1));

            let phase = match decision.phase {
                fret_runtime::ShortcutRoutingPhase::PreDispatch => "pre_dispatch",
                fret_runtime::ShortcutRoutingPhase::PostDispatch => "post_dispatch",
            };
            let outcome = match decision.outcome {
                fret_runtime::ShortcutRoutingOutcome::ReservedForIme => "reserved_for_ime",
                fret_runtime::ShortcutRoutingOutcome::ConsumedByWidget => "consumed_by_widget",
                fret_runtime::ShortcutRoutingOutcome::CommandDispatched => "command_dispatched",
                fret_runtime::ShortcutRoutingOutcome::CommandDisabled => "command_disabled",
                fret_runtime::ShortcutRoutingOutcome::SequenceContinuation => {
                    "sequence_continuation"
                }
                fret_runtime::ShortcutRoutingOutcome::SequenceReplay => "sequence_replay",
                fret_runtime::ShortcutRoutingOutcome::NoMatch => "no_match",
                fret_runtime::ShortcutRoutingOutcome::NoKeymap => "no_keymap",
            };

            push_shortcut_routing_trace(
                &mut active.shortcut_routing_trace,
                UiShortcutRoutingTraceEntryV1 {
                    step_index,
                    note: None,
                    frame_id: decision.frame_id.0,
                    phase: phase.to_string(),
                    deferred: decision.deferred,
                    focus_is_text_input: decision.focus_is_text_input,
                    ime_composing: decision.ime_composing,
                    key: format!("{:?}", decision.key),
                    modifiers: UiKeyModifiersV1::from_modifiers(decision.modifiers),
                    repeat: decision.repeat,
                    outcome: outcome.to_string(),
                    command: decision.command.as_ref().map(|c| c.as_str().to_string()),
                    command_enabled: decision.command_enabled,
                    pending_sequence_len: Some(decision.pending_sequence_len),
                },
            );
        }
    }

    fn dump_bundle(&mut self, label: Option<&str>) -> Option<PathBuf> {
        self.dump_bundle_with_options(label, None, None)
    }

    fn dump_bundle_with_options(
        &mut self,
        label: Option<&str>,
        dump_max_snapshots_override: Option<usize>,
        request_id: Option<u64>,
    ) -> Option<PathBuf> {
        bundle_dump::dump_bundle_with_options(self, label, dump_max_snapshots_override, request_id)
    }

    fn next_script_run_id(&mut self) -> u64 {
        let mut id = unix_ms_now();
        if id <= self.last_script_run_id {
            id = self.last_script_run_id.saturating_add(1);
        }
        self.last_script_run_id = id;
        id
    }
}

fn read_touch_stamp(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let text = std::str::from_utf8(&bytes).ok()?;
    text.lines()
        .rev()
        .find_map(|line| line.trim().parse::<u64>().ok())
}

#[derive(Debug, Clone)]
struct PendingPick {
    run_id: u64,
    window: AppWindowId,
    position: Point,
}

#[derive(Default)]
struct WindowRing {
    last_pointer_position: Option<Point>,
    last_pointer_type: Option<fret_core::PointerType>,
    events: VecDeque<RecordedUiEventV1>,
    snapshots: VecDeque<UiDiagnosticsSnapshotV1>,
    snapshot_seq: u64,
    viewport_input_this_frame: Vec<UiViewportInputEventV1>,
    last_changed_models: Vec<u64>,
    last_changed_globals: Vec<String>,
}

impl WindowRing {
    fn update_pointer_position(&mut self, event: &Event) {
        match event {
            Event::Pointer(e) => match e {
                fret_core::PointerEvent::Move {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::Down {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::Up {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::Wheel {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::PinchGesture {
                    position,
                    pointer_type,
                    ..
                } => {
                    self.last_pointer_position = Some(*position);
                    self.last_pointer_type = Some(*pointer_type);
                }
            },
            Event::PointerCancel(e) => {
                self.last_pointer_position = e.position;
                self.last_pointer_type = Some(e.pointer_type);
            }
            _ => {}
        }
    }

    fn clear(&mut self) {
        self.last_pointer_position = None;
        self.last_pointer_type = None;
        self.events.clear();
        self.snapshots.clear();
        self.snapshot_seq = 0;
        self.viewport_input_this_frame.clear();
        self.last_changed_models.clear();
        self.last_changed_globals.clear();
    }

    fn push_event(&mut self, cfg: &UiDiagnosticsConfig, event: RecordedUiEventV1) {
        self.events.push_back(event);
        while self.events.len() > cfg.max_events {
            self.events.pop_front();
        }
    }

    fn push_snapshot(&mut self, cfg: &UiDiagnosticsConfig, snapshot: UiDiagnosticsSnapshotV1) {
        self.snapshots.push_back(snapshot);
        while self.snapshots.len() > cfg.max_snapshots {
            self.snapshots.pop_front();
        }
    }
}

// Bundle serialization types live in `ui_diagnostics/bundle.rs`.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFrameClockSnapshotV1 {
    pub now_monotonic_ms: u64,
    pub delta_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_delta_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsSnapshotV1 {
    pub schema_version: u32,
    pub tick_id: u64,
    pub frame_id: u64,
    /// Per-window monotonic snapshot sequence (contiguous within a run).
    pub window_snapshot_seq: u64,
    pub window: u64,
    pub timestamp_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_clock: Option<UiFrameClockSnapshotV1>,
    pub scale_factor: f32,
    pub window_bounds: RectV1,
    pub scene_ops: u64,
    #[serde(default)]
    pub scene_fingerprint: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_fingerprint: Option<u64>,

    pub changed_models: Vec<u64>,
    pub changed_globals: Vec<String>,

    /// Aggregated writers for `changed_models`, derived from `ModelStore` debug info.
    ///
    /// This is best-effort and only populated in debug builds.
    #[serde(default)]
    pub changed_model_sources_top: Vec<UiChangedModelSourceHotspotV1>,

    #[serde(default)]
    pub resource_caches: Option<UiResourceCachesV1>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_snapshot: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area_insets: Option<UiEdgesV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_insets: Option<UiEdgesV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_is_text_input: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_composing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clipboard: Option<UiClipboardDiagnosticsSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_pointer_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caps: Option<UiPlatformCapabilitiesSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wgpu_adapter: Option<serde_json::Value>,

    pub debug: UiTreeDebugSnapshotV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiClipboardDiagnosticsSnapshotV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_token: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_unavailable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write_unavailable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPlatformCapabilitiesSummaryV1 {
    pub platform: String,
    pub ui_window_hover_detection: String,
    pub clipboard_text: bool,
    pub clipboard_text_read: bool,
    pub clipboard_text_write: bool,
    pub clipboard_primary_text: bool,
    pub ime: bool,
    pub ime_set_cursor_area: bool,
    pub fs_file_dialogs: bool,
    pub shell_share_sheet: bool,
    pub shell_incoming_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiChangedModelSourceHotspotV1 {
    pub type_name: String,
    pub changed_at: UiSourceLocationV1,
    pub count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiResourceCachesV1 {
    #[serde(default)]
    pub icon_svg_cache: Option<UiRetainedSvgCacheStatsV1>,
    #[serde(default)]
    pub canvas: Vec<UiCanvasCacheEntryV1>,
    #[serde(default)]
    pub render_text: Option<UiRendererTextPerfSnapshotV1>,
    #[serde(default)]
    pub render_text_font_trace: Option<UiRendererTextFontTraceSnapshotV1>,
    #[serde(default)]
    pub render_text_fallback_policy: Option<UiRendererTextFallbackPolicySnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRetainedSvgCacheStatsV1 {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: UiCacheStatsV1,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiCacheStatsV1 {
    pub get_calls: u64,
    pub get_hits: u64,
    pub get_misses: u64,
    pub prepare_calls: u64,
    pub prepare_hits: u64,
    pub prepare_misses: u64,
    pub prune_calls: u64,
    pub clear_calls: u64,
    pub evict_calls: u64,
    pub release_replaced: u64,
    pub release_prune_age: u64,
    pub release_prune_budget: u64,
    pub release_clear: u64,
    pub release_evict: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiRendererTextPerfSnapshotV1 {
    pub frame_id: u64,

    pub font_stack_key: u64,
    pub font_db_revision: u64,
    #[serde(default)]
    pub fallback_policy_key: u64,

    #[serde(default)]
    pub frame_missing_glyphs: u64,
    #[serde(default)]
    pub frame_texts_with_missing_glyphs: u64,

    pub blobs_live: u64,
    pub blob_cache_entries: u64,
    pub shape_cache_entries: u64,
    pub measure_cache_buckets: u64,

    #[serde(default)]
    pub unwrapped_layout_cache_entries: u64,
    #[serde(default)]
    pub frame_unwrapped_layout_cache_hits: u64,
    #[serde(default)]
    pub frame_unwrapped_layout_cache_misses: u64,
    #[serde(default)]
    pub frame_unwrapped_layouts_created: u64,

    pub frame_cache_resets: u64,
    pub frame_blob_cache_hits: u64,
    pub frame_blob_cache_misses: u64,
    pub frame_blobs_created: u64,
    pub frame_shape_cache_hits: u64,
    pub frame_shape_cache_misses: u64,
    pub frame_shapes_created: u64,

    pub mask_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
    pub color_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
    pub subpixel_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
}

impl UiRendererTextPerfSnapshotV1 {
    fn from_core(snapshot: fret_core::RendererTextPerfSnapshot) -> Self {
        Self {
            frame_id: snapshot.frame_id.0,
            font_stack_key: snapshot.font_stack_key,
            font_db_revision: snapshot.font_db_revision,
            fallback_policy_key: snapshot.fallback_policy_key,
            frame_missing_glyphs: snapshot.frame_missing_glyphs,
            frame_texts_with_missing_glyphs: snapshot.frame_texts_with_missing_glyphs,
            blobs_live: snapshot.blobs_live,
            blob_cache_entries: snapshot.blob_cache_entries,
            shape_cache_entries: snapshot.shape_cache_entries,
            measure_cache_buckets: snapshot.measure_cache_buckets,
            unwrapped_layout_cache_entries: snapshot.unwrapped_layout_cache_entries,
            frame_unwrapped_layout_cache_hits: snapshot.frame_unwrapped_layout_cache_hits,
            frame_unwrapped_layout_cache_misses: snapshot.frame_unwrapped_layout_cache_misses,
            frame_unwrapped_layouts_created: snapshot.frame_unwrapped_layouts_created,
            frame_cache_resets: snapshot.frame_cache_resets,
            frame_blob_cache_hits: snapshot.frame_blob_cache_hits,
            frame_blob_cache_misses: snapshot.frame_blob_cache_misses,
            frame_blobs_created: snapshot.frame_blobs_created,
            frame_shape_cache_hits: snapshot.frame_shape_cache_hits,
            frame_shape_cache_misses: snapshot.frame_shape_cache_misses,
            frame_shapes_created: snapshot.frame_shapes_created,
            mask_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.mask_atlas),
            color_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.color_atlas),
            subpixel_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.subpixel_atlas),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextCommonFallbackInjectionV1 {
    PlatformDefault,
    None,
    CommonFallback,
}

impl Default for UiTextCommonFallbackInjectionV1 {
    fn default() -> Self {
        Self::PlatformDefault
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFallbackPolicySnapshotV1 {
    pub frame_id: u64,
    pub font_stack_key: u64,
    pub font_db_revision: u64,
    pub fallback_policy_key: u64,

    #[serde(default)]
    pub system_fonts_enabled: bool,
    #[serde(default)]
    pub prefer_common_fallback: bool,

    #[serde(default)]
    pub common_fallback_injection: UiTextCommonFallbackInjectionV1,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale_bcp47: Option<String>,

    #[serde(default)]
    pub common_fallback_stack_suffix: String,
    #[serde(default)]
    pub common_fallback_candidates: Vec<String>,
}

impl UiRendererTextFallbackPolicySnapshotV1 {
    fn from_core(
        snapshot: fret_core::RendererTextFallbackPolicySnapshot,
        max_debug_string_bytes: usize,
    ) -> Self {
        fn injection_from_core(
            injection: fret_core::TextCommonFallbackInjection,
        ) -> UiTextCommonFallbackInjectionV1 {
            match injection {
                fret_core::TextCommonFallbackInjection::PlatformDefault => {
                    UiTextCommonFallbackInjectionV1::PlatformDefault
                }
                fret_core::TextCommonFallbackInjection::None => {
                    UiTextCommonFallbackInjectionV1::None
                }
                fret_core::TextCommonFallbackInjection::CommonFallback => {
                    UiTextCommonFallbackInjectionV1::CommonFallback
                }
            }
        }

        let mut locale_bcp47 = snapshot.locale_bcp47;
        if let Some(locale) = locale_bcp47.as_mut() {
            truncate_string_bytes(locale, max_debug_string_bytes);
        }

        let mut common_fallback_stack_suffix = snapshot.common_fallback_stack_suffix;
        truncate_string_bytes(&mut common_fallback_stack_suffix, max_debug_string_bytes);

        let mut common_fallback_candidates = snapshot.common_fallback_candidates;
        for s in &mut common_fallback_candidates {
            truncate_string_bytes(s, max_debug_string_bytes);
        }

        Self {
            frame_id: snapshot.frame_id.0,
            font_stack_key: snapshot.font_stack_key,
            font_db_revision: snapshot.font_db_revision,
            fallback_policy_key: snapshot.fallback_policy_key,
            system_fonts_enabled: snapshot.system_fonts_enabled,
            prefer_common_fallback: snapshot.prefer_common_fallback,
            common_fallback_injection: injection_from_core(snapshot.common_fallback_injection),
            locale_bcp47,
            common_fallback_stack_suffix,
            common_fallback_candidates,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFontTraceSnapshotV1 {
    pub frame_id: u64,
    #[serde(default)]
    pub entries: Vec<UiRendererTextFontTraceEntryV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFontTraceEntryV1 {
    pub text_preview: String,
    pub text_len_bytes: u32,

    pub font: String,
    pub font_size_px: f32,
    pub scale_factor: f32,

    pub wrap: String,
    pub overflow: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width_px: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale_bcp47: Option<String>,

    pub missing_glyphs: u32,

    #[serde(default)]
    pub families: Vec<UiRendererTextFontTraceFamilyUsageV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFontTraceFamilyUsageV1 {
    pub family: String,
    pub glyphs: u32,
    pub missing_glyphs: u32,
    pub class: UiRendererTextFontTraceFamilyClassV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRendererTextFontTraceFamilyClassV1 {
    Requested,
    CommonFallback,
    SystemFallback,
    Unknown,
}

impl Default for UiRendererTextFontTraceFamilyClassV1 {
    fn default() -> Self {
        Self::Unknown
    }
}

impl UiRendererTextFontTraceSnapshotV1 {
    fn from_core(
        snapshot: fret_core::RendererTextFontTraceSnapshot,
        redact_text: bool,
        max_debug_string_bytes: usize,
    ) -> Self {
        fn wrap_to_string(wrap: fret_core::TextWrap) -> &'static str {
            match wrap {
                fret_core::TextWrap::None => "none",
                fret_core::TextWrap::Word => "word",
                fret_core::TextWrap::WordBreak => "word_break",
                fret_core::TextWrap::Grapheme => "grapheme",
            }
        }

        fn overflow_to_string(overflow: fret_core::TextOverflow) -> &'static str {
            match overflow {
                fret_core::TextOverflow::Clip => "clip",
                fret_core::TextOverflow::Ellipsis => "ellipsis",
            }
        }

        fn font_id_to_string(font: &fret_core::FontId) -> String {
            match font {
                fret_core::FontId::Ui => "ui".to_string(),
                fret_core::FontId::Serif => "serif".to_string(),
                fret_core::FontId::Monospace => "monospace".to_string(),
                fret_core::FontId::Family(name) => format!("family:{name}"),
            }
        }

        fn class_from_core(
            class: fret_core::RendererTextFontTraceFamilyClass,
        ) -> UiRendererTextFontTraceFamilyClassV1 {
            match class {
                fret_core::RendererTextFontTraceFamilyClass::Requested => {
                    UiRendererTextFontTraceFamilyClassV1::Requested
                }
                fret_core::RendererTextFontTraceFamilyClass::CommonFallback => {
                    UiRendererTextFontTraceFamilyClassV1::CommonFallback
                }
                fret_core::RendererTextFontTraceFamilyClass::SystemFallback => {
                    UiRendererTextFontTraceFamilyClassV1::SystemFallback
                }
                fret_core::RendererTextFontTraceFamilyClass::Unknown => {
                    UiRendererTextFontTraceFamilyClassV1::Unknown
                }
            }
        }

        let mut entries = snapshot
            .entries
            .into_iter()
            .map(|mut e| {
                if redact_text {
                    e.text_preview = "<redacted>".to_string();
                }
                truncate_string_bytes(&mut e.text_preview, max_debug_string_bytes);
                if let Some(locale) = e.locale_bcp47.as_mut() {
                    truncate_string_bytes(locale, max_debug_string_bytes);
                }

                let mut families: Vec<UiRendererTextFontTraceFamilyUsageV1> = e
                    .families
                    .into_iter()
                    .map(|mut f| {
                        truncate_string_bytes(&mut f.family, max_debug_string_bytes);
                        UiRendererTextFontTraceFamilyUsageV1 {
                            family: f.family,
                            glyphs: f.glyphs,
                            missing_glyphs: f.missing_glyphs,
                            class: class_from_core(f.class),
                        }
                    })
                    .collect();
                families.sort_by(|a, b| {
                    b.glyphs
                        .cmp(&a.glyphs)
                        .then_with(|| a.family.cmp(&b.family))
                });

                UiRendererTextFontTraceEntryV1 {
                    text_preview: e.text_preview,
                    text_len_bytes: e.text_len_bytes,
                    font: font_id_to_string(&e.font),
                    font_size_px: e.font_size.0,
                    scale_factor: e.scale_factor,
                    wrap: wrap_to_string(e.wrap).to_string(),
                    overflow: overflow_to_string(e.overflow).to_string(),
                    max_width_px: e.max_width.map(|px| px.0),
                    locale_bcp47: e.locale_bcp47,
                    missing_glyphs: e.missing_glyphs,
                    families,
                }
            })
            .collect::<Vec<_>>();
        entries.truncate(4096);

        Self {
            frame_id: snapshot.frame_id.0,
            entries,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiRendererGlyphAtlasPerfSnapshotV1 {
    pub width: u32,
    pub height: u32,
    pub pages: u32,
    pub entries: u64,

    pub used_px: u64,
    pub capacity_px: u64,

    pub frame_hits: u64,
    pub frame_misses: u64,
    pub frame_inserts: u64,
    pub frame_evict_glyphs: u64,
    pub frame_evict_pages: u64,
    pub frame_out_of_space: u64,
    pub frame_too_large: u64,

    pub frame_pending_uploads: u64,
    pub frame_pending_upload_bytes: u64,
    pub frame_upload_bytes: u64,
}

impl UiRendererGlyphAtlasPerfSnapshotV1 {
    fn from_core(snapshot: fret_core::RendererGlyphAtlasPerfSnapshot) -> Self {
        Self {
            width: snapshot.width,
            height: snapshot.height,
            pages: snapshot.pages,
            entries: snapshot.entries,
            used_px: snapshot.used_px,
            capacity_px: snapshot.capacity_px,
            frame_hits: snapshot.frame_hits,
            frame_misses: snapshot.frame_misses,
            frame_inserts: snapshot.frame_inserts,
            frame_evict_glyphs: snapshot.frame_evict_glyphs,
            frame_evict_pages: snapshot.frame_evict_pages,
            frame_out_of_space: snapshot.frame_out_of_space,
            frame_too_large: snapshot.frame_too_large,
            frame_pending_uploads: snapshot.frame_pending_uploads,
            frame_pending_upload_bytes: snapshot.frame_pending_upload_bytes,
            frame_upload_bytes: snapshot.frame_upload_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiSceneOpTileCacheStatsV1 {
    pub calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub stored_tiles: u64,
    pub recorded_ops: u64,
    pub replayed_ops: u64,
    pub clear_calls: u64,
    pub prune_calls: u64,
    pub evict_calls: u64,
    pub evict_prune_age: u64,
    pub evict_prune_budget: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSceneOpTileCacheSnapshotV1 {
    pub entries: usize,
    #[serde(default)]
    pub requested_tiles: usize,
    #[serde(default)]
    pub budget_limit: u32,
    #[serde(default)]
    pub budget_used: u32,
    #[serde(default)]
    pub skipped_tiles: u32,
    pub stats: UiSceneOpTileCacheStatsV1,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiWorkBudgetSnapshotV1 {
    pub requested_units: u32,
    pub limit: u32,
    pub used: u32,
    pub skipped_units: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCanvasCacheEntryV1 {
    pub node: u64,
    pub name: String,
    #[serde(default)]
    pub path: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub svg: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub text: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub scene_op_tiles: Option<UiSceneOpTileCacheSnapshotV1>,
    #[serde(default)]
    pub work_budget: Option<UiWorkBudgetSnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCacheKindSnapshotV1 {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: UiCacheStatsV1,
}

#[cfg(feature = "preload-icon-svgs")]
fn icon_svg_cache_stats(app: &App) -> Option<UiRetainedSvgCacheStatsV1> {
    let stats = app.global::<fret_ui_kit::declarative::icon::IconSvgPreloadDiagnostics>()?;
    let entries = stats.entries;
    let bytes_ready = stats.bytes_ready;
    let register_calls = stats.register_calls;
    Some(UiRetainedSvgCacheStatsV1 {
        entries,
        bytes_ready,
        stats: UiCacheStatsV1 {
            prepare_calls: register_calls,
            ..Default::default()
        },
    })
}

#[cfg(not(feature = "preload-icon-svgs"))]
fn icon_svg_cache_stats(_app: &App) -> Option<UiRetainedSvgCacheStatsV1> {
    None
}

fn canvas_cache_stats_for_window(app: &App, window: u64) -> Vec<UiCanvasCacheEntryV1> {
    let Some(registry) = app.global::<fret_canvas::diagnostics::CanvasCacheStatsRegistry>() else {
        return Vec::new();
    };

    registry
        .iter()
        .filter_map(|(key, snap)| {
            ((key.window == window) || (key.window == 0)).then_some((key, snap))
        })
        .map(|(key, snap)| UiCanvasCacheEntryV1 {
            node: key.node,
            name: key.name.to_string(),
            path: snap.path.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            svg: snap.svg.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            text: snap.text.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            scene_op_tiles: snap.scene_op_tiles.map(|s| UiSceneOpTileCacheSnapshotV1 {
                entries: s.entries,
                requested_tiles: s.requested_tiles,
                budget_limit: s.budget_limit,
                budget_used: s.budget_used,
                skipped_tiles: s.skipped_tiles,
                stats: UiSceneOpTileCacheStatsV1 {
                    calls: s.stats.calls,
                    hits: s.stats.hits,
                    misses: s.stats.misses,
                    stored_tiles: s.stats.stored_tiles,
                    recorded_ops: s.stats.recorded_ops,
                    replayed_ops: s.stats.replayed_ops,
                    clear_calls: s.stats.clear_calls,
                    prune_calls: s.stats.prune_calls,
                    evict_calls: s.stats.evict_calls,
                    evict_prune_age: s.stats.evict_prune_age,
                    evict_prune_budget: s.stats.evict_prune_budget,
                },
            }),
            work_budget: snap.work_budget.map(|b| UiWorkBudgetSnapshotV1 {
                requested_units: b.requested_units,
                limit: b.limit,
                used: b.used,
                skipped_units: b.skipped_units,
            }),
        })
        .collect()
}

#[cfg(any())]
mod legacy_forked_script_protocol {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UiActionScriptV1 {
        pub schema_version: u32,
        pub steps: Vec<UiActionStepV1>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum UiActionStepV1 {
        Click {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_click_count")]
            click_count: u8,
        },
        ResetDiagnostics,
        MovePointer {
            target: UiSelectorV1,
        },
        DragPointer {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            delta_x: f32,
            delta_y: f32,
            #[serde(default = "default_drag_steps")]
            steps: u32,
        },
        Wheel {
            target: UiSelectorV1,
            #[serde(default)]
            delta_x: f32,
            #[serde(default)]
            delta_y: f32,
        },
        PressKey {
            key: String,
            #[serde(default)]
            modifiers: UiKeyModifiersV1,
            #[serde(default)]
            repeat: bool,
        },
        TypeText {
            text: String,
        },
        WaitFrames {
            n: u32,
        },
        WaitUntil {
            predicate: UiPredicateV1,
            timeout_frames: u32,
        },
        Assert {
            predicate: UiPredicateV1,
        },
        CaptureBundle {
            label: Option<String>,
        },
        CaptureScreenshot {
            label: Option<String>,
            #[serde(default = "default_capture_screenshot_timeout_frames")]
            timeout_frames: u32,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UiActionScriptV2 {
        pub schema_version: u32,
        pub steps: Vec<UiActionStepV2>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum UiActionStepV2 {
        // v1-compatible steps
        Click {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_click_count")]
            click_count: u8,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            modifiers: Option<UiKeyModifiersV1>,
        },
        ResetDiagnostics,
        MovePointer {
            target: UiSelectorV1,
        },
        DragPointer {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            delta_x: f32,
            delta_y: f32,
            #[serde(default = "default_drag_steps")]
            steps: u32,
        },
        /// Move the pointer along a straight line over multiple frames (one move event per frame).
        ///
        /// Prefer this over `drag_pointer` when measuring hit-test/dispatch time, because
        /// `drag_pointer` emits multiple pointer move events in a single frame.
        MovePointerSweep {
            target: UiSelectorV1,
            delta_x: f32,
            delta_y: f32,
            #[serde(default = "default_drag_steps")]
            steps: u32,
            #[serde(default = "default_move_frames_per_step")]
            frames_per_step: u32,
        },
        Wheel {
            target: UiSelectorV1,
            #[serde(default)]
            delta_x: f32,
            #[serde(default)]
            delta_y: f32,
        },
        PressKey {
            key: String,
            #[serde(default)]
            modifiers: UiKeyModifiersV1,
            #[serde(default)]
            repeat: bool,
        },
        TypeText {
            text: String,
        },
        WaitFrames {
            n: u32,
        },
        WaitUntil {
            predicate: UiPredicateV1,
            timeout_frames: u32,
        },
        Assert {
            predicate: UiPredicateV1,
        },
        CaptureBundle {
            label: Option<String>,
        },
        CaptureScreenshot {
            label: Option<String>,
            #[serde(default = "default_capture_screenshot_timeout_frames")]
            timeout_frames: u32,
        },

        // v2 intent-level steps
        /// Click a target only after its bounds have remained stable for `stable_frames`.
        ///
        /// This is useful for virtualized lists where a target's measured bounds can jump
        /// across frames (e.g. estimate -> measured), causing clicks to land at stale
        /// positions when using a single-frame snapshot.
        ClickStable {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_click_count")]
            click_count: u8,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            modifiers: Option<UiKeyModifiersV1>,
            #[serde(default = "default_click_stable_frames")]
            stable_frames: u32,
            #[serde(default = "default_click_stable_max_move_px")]
            max_move_px: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        EnsureVisible {
            target: UiSelectorV1,
            #[serde(default)]
            within_window: bool,
            #[serde(default)]
            padding_px: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        ScrollIntoView {
            container: UiSelectorV1,
            target: UiSelectorV1,
            #[serde(default)]
            delta_x: f32,
            #[serde(default = "default_scroll_delta_y")]
            delta_y: f32,
            #[serde(default)]
            require_fully_within_window: bool,
            #[serde(default)]
            padding_px: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        TypeTextInto {
            target: UiSelectorV1,
            text: String,
            #[serde(default)]
            clear_before_type: bool,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        MenuSelect {
            menu: UiSelectorV1,
            item: UiSelectorV1,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        DragTo {
            from: UiSelectorV1,
            to: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_drag_steps")]
            steps: u32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        SetSliderValue {
            target: UiSelectorV1,
            value: f32,
            #[serde(default = "default_slider_min")]
            min: f32,
            #[serde(default = "default_slider_max")]
            max: f32,
            #[serde(default = "default_slider_epsilon")]
            epsilon: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
            #[serde(default = "default_drag_steps")]
            drag_steps: u32,
        },
        /// Request a resize of the active window's inner size (logical px).
        ///
        /// This is intended for deterministic “resize stress” repro scripts and is best-effort:
        /// runners may ignore it on platforms where programmatic resizing is not supported.
        SetWindowInnerSize {
            width_px: f32,
            height_px: f32,
        },
    }

    impl From<UiActionStepV1> for UiActionStepV2 {
        fn from(value: UiActionStepV1) -> Self {
            match value {
                UiActionStepV1::Click {
                    target,
                    button,
                    click_count,
                } => Self::Click {
                    target,
                    button,
                    click_count,
                    modifiers: None,
                },
                UiActionStepV1::ResetDiagnostics => Self::ResetDiagnostics,
                UiActionStepV1::MovePointer { target } => Self::MovePointer { target },
                UiActionStepV1::DragPointer {
                    target,
                    button,
                    delta_x,
                    delta_y,
                    steps,
                } => Self::DragPointer {
                    target,
                    button,
                    delta_x,
                    delta_y,
                    steps,
                },
                UiActionStepV1::Wheel {
                    target,
                    delta_x,
                    delta_y,
                } => Self::Wheel {
                    target,
                    delta_x,
                    delta_y,
                },
                UiActionStepV1::PressKey {
                    key,
                    modifiers,
                    repeat,
                } => Self::PressKey {
                    key,
                    modifiers,
                    repeat,
                },
                UiActionStepV1::TypeText { text } => Self::TypeText { text },
                UiActionStepV1::WaitFrames { n } => Self::WaitFrames { n },
                UiActionStepV1::WaitUntil {
                    predicate,
                    timeout_frames,
                } => Self::WaitUntil {
                    predicate,
                    timeout_frames,
                },
                UiActionStepV1::Assert { predicate } => Self::Assert { predicate },
                UiActionStepV1::CaptureBundle { label } => Self::CaptureBundle { label },
                UiActionStepV1::CaptureScreenshot {
                    label,
                    timeout_frames,
                } => Self::CaptureScreenshot {
                    label,
                    timeout_frames,
                },
            }
        }
    }

    fn default_drag_steps() -> u32 {
        8
    }

    fn default_click_count() -> u8 {
        1
    }

    fn default_click_stable_frames() -> u32 {
        2
    }

    fn default_click_stable_max_move_px() -> f32 {
        1.0
    }

    fn default_move_frames_per_step() -> u32 {
        1
    }

    fn default_capture_screenshot_timeout_frames() -> u32 {
        300
    }

    fn default_action_timeout_frames() -> u32 {
        180
    }

    fn default_scroll_delta_y() -> f32 {
        -120.0
    }

    fn default_slider_min() -> f32 {
        0.0
    }

    fn default_slider_max() -> f32 {
        100.0
    }

    fn default_slider_epsilon() -> f32 {
        0.5
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UiMouseButtonV1 {
        Left,
        Right,
        Middle,
    }

    impl Default for UiMouseButtonV1 {
        fn default() -> Self {
            Self::Left
        }
    }

    impl UiMouseButtonV1 {
        fn from_button(button: fret_core::MouseButton) -> Self {
            match button {
                fret_core::MouseButton::Left => Self::Left,
                fret_core::MouseButton::Right => Self::Right,
                fret_core::MouseButton::Middle => Self::Middle,
                fret_core::MouseButton::Back
                | fret_core::MouseButton::Forward
                | fret_core::MouseButton::Other(_) => Self::Left,
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
    pub struct UiKeyModifiersV1 {
        #[serde(default)]
        pub shift: bool,
        #[serde(default)]
        pub ctrl: bool,
        #[serde(default)]
        pub alt: bool,
        #[serde(default)]
        pub meta: bool,
    }

    impl UiKeyModifiersV1 {
        fn from_modifiers(modifiers: fret_core::Modifiers) -> Self {
            Self {
                shift: modifiers.shift,
                ctrl: modifiers.ctrl,
                alt: modifiers.alt,
                meta: modifiers.meta,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum UiPredicateV1 {
        Exists {
            target: UiSelectorV1,
        },
        NotExists {
            target: UiSelectorV1,
        },
        FocusIs {
            target: UiSelectorV1,
        },
        /// Matches the current modal/pointer barrier root and focus barrier root (if any).
        ///
        /// This is intentionally coarse-grained: scripts should be able to assert that close
        /// transitions keep the pointer barrier active while releasing focus containment (or vice
        /// versa) without needing stable node ids.
        BarrierRoots {
            #[serde(default)]
            barrier_root: UiOptionalRootStateV1,
            #[serde(default)]
            focus_barrier_root: UiOptionalRootStateV1,
            /// When set, additionally enforces whether the two roots are equal.
            ///
            /// - `true`: requires `barrier_root == focus_barrier_root` (both `None`, or the same id).
            /// - `false`: requires `barrier_root != focus_barrier_root`.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            require_equal: Option<bool>,
        },
        /// True when the target exists and its semantics bounds intersect the active window bounds.
        ///
        /// This is useful for scroll-driven scenarios: it prevents scripts from “finding” an element
        /// that exists in the tree but is currently far off-screen due to an in-flight scroll/window
        /// update.
        VisibleInWindow {
            target: UiSelectorV1,
        },
        /// True when the target exists and its semantics bounds are fully contained within the active
        /// window bounds (optionally padded inward by `padding_px`).
        BoundsWithinWindow {
            target: UiSelectorV1,
            #[serde(default)]
            padding_px: f32,
            /// A small tolerance to account for subpixel rounding (e.g. 1 physical px at non-1.0 DPI).
            ///
            /// This does not replace `padding_px` (which shrinks the allowed region); it only relaxes
            /// strict edge containment checks by `eps_px`.
            #[serde(default)]
            eps_px: f32,
        },
        /// True when the target exists and its semantics bounds are at least the specified size.
        ///
        /// This is useful for demos where the content can legitimately be taller than the window
        /// (scrollable pages), but we still want to gate against "collapsed to ~0" layout regressions.
        BoundsMinSize {
            target: UiSelectorV1,
            #[serde(default)]
            min_w_px: f32,
            #[serde(default)]
            min_h_px: f32,
            /// A small tolerance to account for rounding / fractional layout units.
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds do not overlap.
        ///
        /// Use `eps_px` to tolerate tiny intersections caused by subpixel rounding (e.g. at 125% DPI).
        BoundsNonOverlapping {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds overlap.
        ///
        /// Use `eps_px` to require at least `eps_px` overlap in both dimensions (helps tolerate
        /// subpixel rounding at fractional DPI).
        BoundsOverlapping {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds overlap on the X axis.
        ///
        /// This is useful when two elements are intentionally vertically offset (e.g. a slider thumb
        /// and track), but we still want to assert horizontal alignment.
        BoundsOverlappingX {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds overlap on the Y axis.
        BoundsOverlappingY {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
    }

    #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UiOptionalRootStateV1 {
        /// Do not assert anything about the root (accept both `Some` and `None`).
        #[default]
        Any,
        None,
        Some,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum UiSelectorV1 {
        RoleAndName {
            role: String,
            name: String,
        },
        RoleAndPath {
            role: String,
            name: String,
            /// Ancestors ordered from outermost -> innermost.
            ancestors: Vec<UiRoleAndNameV1>,
        },
        TestId {
            id: String,
        },
        GlobalElementId {
            element: u64,
        },
        NodeId {
            node: u64,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UiRoleAndNameV1 {
        pub role: String,
        pub name: String,
    }
}

#[derive(Debug, Default)]
pub struct UiScriptFrameOutput {
    pub events: Vec<Event>,
    pub effects: Vec<Effect>,
    pub request_redraw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPickResultV1 {
    pub schema_version: u32,
    pub run_id: u64,
    pub updated_unix_ms: u64,
    pub window: Option<u64>,
    pub stage: UiPickStageV1,
    pub position: Option<PointV1>,
    pub selection: Option<UiPickSelectionV1>,
    pub reason: Option<String>,
    pub last_bundle_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPickStageV1 {
    Picked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPickSelectionV1 {
    pub node: UiSemanticsNodeV1,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub selectors: Vec<UiSelectorV1>,
}

impl UiPickSelectionV1 {
    fn from_node(
        snapshot: &fret_core::SemanticsSnapshot,
        node: &fret_core::SemanticsNode,
        element: Option<u64>,
        element_path: Option<String>,
        cfg: &UiDiagnosticsConfig,
    ) -> Self {
        let exported =
            UiSemanticsNodeV1::from_node(node, cfg.redact_text, cfg.max_debug_string_bytes);
        let selectors = suggest_selectors(snapshot, node, &exported, element, cfg);
        Self {
            node: exported,
            element,
            element_path,
            selectors,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInputArbitrationSnapshotV1 {
    #[serde(default)]
    pub modal_barrier_root: Option<u64>,
    #[serde(default)]
    pub focus_barrier_root: Option<u64>,
    #[serde(default)]
    pub pointer_occlusion: String,
    #[serde(default)]
    pub pointer_occlusion_layer_id: Option<u64>,
    #[serde(default)]
    pub pointer_capture_active: bool,
    #[serde(default)]
    pub pointer_capture_layer_id: Option<u64>,
    #[serde(default)]
    pub pointer_capture_multiple_layers: bool,
}

impl Default for UiInputArbitrationSnapshotV1 {
    fn default() -> Self {
        Self {
            modal_barrier_root: None,
            focus_barrier_root: None,
            pointer_occlusion: "none".to_string(),
            pointer_occlusion_layer_id: None,
            pointer_capture_active: false,
            pointer_capture_layer_id: None,
            pointer_capture_multiple_layers: false,
        }
    }
}

impl UiInputArbitrationSnapshotV1 {
    fn from_snapshot(snapshot: fret_ui::tree::UiInputArbitrationSnapshot) -> Self {
        Self {
            modal_barrier_root: snapshot.modal_barrier_root.map(key_to_u64),
            focus_barrier_root: snapshot.focus_barrier_root.map(key_to_u64),
            pointer_occlusion: pointer_occlusion_label(snapshot.pointer_occlusion),
            pointer_occlusion_layer_id: snapshot
                .pointer_occlusion_layer
                .map(|id| id.data().as_ffi()),
            pointer_capture_active: snapshot.pointer_capture_active,
            pointer_capture_layer_id: snapshot.pointer_capture_layer.map(|id| id.data().as_ffi()),
            pointer_capture_multiple_layers: snapshot.pointer_capture_multiple_layers,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTreeDebugSnapshotV1 {
    pub stats: UiFrameStatsV1,
    #[serde(default)]
    pub invalidation_walks: Vec<UiInvalidationWalkV1>,
    #[serde(default)]
    pub hover_declarative_invalidation_hotspots: Vec<UiHoverDeclarativeInvalidationHotspotV1>,
    #[serde(default)]
    pub dirty_views: Vec<UiDirtyViewV1>,
    #[serde(default)]
    pub notify_requests: Vec<UiNotifyRequestV1>,
    #[serde(default)]
    pub virtual_list_windows: Vec<UiVirtualListWindowV1>,
    #[serde(default)]
    pub virtual_list_window_shift_samples: Vec<UiVirtualListWindowShiftSampleV1>,
    #[serde(default)]
    pub windowed_rows_surfaces: Vec<UiWindowedRowsSurfaceWindowV1>,
    #[serde(default)]
    pub retained_virtual_list_reconciles: Vec<UiRetainedVirtualListReconcileV1>,
    #[serde(default)]
    pub scroll_handle_changes: Vec<UiScrollHandleChangeV1>,
    #[serde(default)]
    pub prepaint_actions: Vec<UiPrepaintActionV1>,
    #[serde(default)]
    pub model_change_hotspots: Vec<UiModelChangeHotspotV1>,
    #[serde(default)]
    pub model_change_unobserved: Vec<UiModelChangeUnobservedV1>,
    #[serde(default)]
    pub global_change_hotspots: Vec<UiGlobalChangeHotspotV1>,
    #[serde(default)]
    pub global_change_unobserved: Vec<UiGlobalChangeUnobservedV1>,
    #[serde(default)]
    pub cache_roots: Vec<UiCacheRootStatsV1>,
    #[serde(default)]
    pub overlay_synthesis: Vec<UiOverlaySynthesisEventV1>,
    /// Viewport input forwarding events observed during the current frame.
    ///
    /// This records `Effect::ViewportInput` deliveries (ADR 0132) so scripted diagnostics can
    /// gate on “viewport tooling input was actually exercised” without scraping logs.
    #[serde(default)]
    pub viewport_input: Vec<UiViewportInputEventV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_ime_bridge: Option<UiWebImeBridgeDebugSnapshotV1>,
    /// Docking interaction ownership snapshot (best-effort).
    ///
    /// This is sourced from a frame-local diagnostics store populated by policy-heavy ecosystem
    /// crates (e.g. docking), and is intended for debugging arbitration regressions without logs.
    #[serde(default)]
    pub docking_interaction: Option<UiDockingInteractionSnapshotV1>,
    #[serde(default)]
    pub removed_subtrees: Vec<UiRemovedSubtreeV1>,
    #[serde(default)]
    pub layout_engine_solves: Vec<UiLayoutEngineSolveV1>,
    #[serde(default)]
    pub layout_hotspots: Vec<UiLayoutHotspotV1>,
    #[serde(default)]
    pub widget_measure_hotspots: Vec<UiWidgetMeasureHotspotV1>,
    #[serde(default)]
    pub paint_widget_hotspots: Vec<UiPaintWidgetHotspotV1>,
    #[serde(default)]
    pub paint_text_prepare_hotspots: Vec<UiPaintTextPrepareHotspotV1>,
    #[serde(default)]
    pub input_arbitration: UiInputArbitrationSnapshotV1,
    /// Best-effort command gating decisions for a small set of "interesting" commands.
    ///
    /// This is intended for debugging cross-surface inconsistencies (menus vs palette vs buttons)
    /// without relying on ad-hoc logs.
    #[serde(default)]
    pub command_gating_trace: Vec<UiCommandGatingTraceEntryV1>,
    pub layers_in_paint_order: Vec<UiLayerInfoV1>,
    #[serde(default)]
    pub all_layer_roots: Vec<u64>,
    #[serde(default)]
    pub layer_visible_writes: Vec<UiLayerVisibleWriteV1>,
    #[serde(default)]
    pub overlay_policy_decisions: Vec<UiOverlayPolicyDecisionV1>,
    /// A committed per-window environment snapshot (ADR 0232), exported under `debug.environment`
    /// for easy diagnostics consumption.
    ///
    /// This duplicates the committed fields also present under `debug.element_runtime.environment`
    /// (when the element runtime snapshot is enabled), but keeps a stable schema path for tools
    /// that do not want to parse the entire element runtime snapshot payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ElementEnvironmentSnapshotV1>,
    /// Best-effort window insets snapshot (safe-area + occlusion) from `WindowMetricsService`.
    ///
    /// Unlike `debug.environment`, this does not require the element runtime snapshot to be
    /// enabled. It is intended as a quick "what does the runner think the insets are" anchor
    /// during mobile bring-up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_insets: Option<UiWindowInsetsSnapshotV1>,
    /// Best-effort platform text-input snapshot for the current window.
    ///
    /// This records `focus_is_text_input` and the last committed IME cursor area, which are
    /// frequently needed when diagnosing virtual keyboard avoidance and IME candidate placement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_input: Option<UiWindowTextInputSnapshotV1>,
    /// Runner surface lifecycle state, sourced from `RunnerSurfaceLifecycleDiagnosticsStore`.
    ///
    /// This is intended for Android/iOS bring-up to verify that background/foreground transitions
    /// are dropping and recreating surfaces as expected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_surface_lifecycle: Option<UiRunnerSurfaceLifecycleSnapshotV1>,
    /// Runner accessibility activation evidence, sourced from `RunnerAccessibilityDiagnosticsStore`.
    ///
    /// This records when the OS accessibility stack activates the AccessKit adapter for a window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_accessibility: Option<UiRunnerAccessibilitySnapshotV1>,
    pub hit_test: Option<UiHitTestSnapshotV1>,
    pub element_runtime: Option<ElementDiagnosticsSnapshotV1>,
    pub semantics: Option<UiSemanticsSnapshotV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiWindowInsetsSnapshotV1 {
    pub safe_area_known: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area_insets_px: Option<UiPaddingInsetsV1>,
    pub occlusion_known: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_insets_px: Option<UiPaddingInsetsV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWindowTextInputSnapshotV1 {
    pub focus_is_text_input: bool,
    pub is_composing: bool,
    /// Total length (UTF-16 code units) of the composed view.
    pub text_len_utf16: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_cursor_area: Option<RectV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiRunnerSurfaceLifecycleSnapshotV1 {
    pub can_create_surfaces_calls: u64,
    pub destroy_surfaces_calls: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_can_create_surfaces_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_destroy_surfaces_unix_ms: Option<u64>,
    pub surfaces_available: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiRunnerAccessibilitySnapshotV1 {
    pub activation_requests: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_activation_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_activation_frame_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWebImeBridgeDebugSnapshotV1 {
    pub enabled: bool,
    pub composing: bool,
    pub suppress_next_input: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_has_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_element_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mount_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_pixel_ratio: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_value_chars: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_start_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_end_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_client_width_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_client_height_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_scroll_width_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_scroll_height_px: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_beforeinput_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_data: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_key_code: Option<KeyCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_area: Option<RectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_anchor_px: Option<(f32, f32)>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_cursor_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_text: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_events: Vec<String>,

    pub beforeinput_seen: u64,
    pub input_seen: u64,
    pub suppressed_input_seen: u64,
    pub composition_start_seen: u64,
    pub composition_update_seen: u64,
    pub composition_end_seen: u64,
    pub cursor_area_set_seen: u64,
}

impl UiWebImeBridgeDebugSnapshotV1 {
    fn from_snapshot(
        snapshot: &fret_core::input::WebImeBridgeDebugSnapshot,
        redact_text: bool,
        max_debug_string_bytes: usize,
    ) -> Self {
        let mut recent_events = if redact_text {
            Vec::new()
        } else {
            snapshot.recent_events.clone()
        };
        for ev in &mut recent_events {
            truncate_string_bytes(ev, max_debug_string_bytes);
        }

        Self {
            enabled: snapshot.enabled,
            composing: snapshot.composing,
            suppress_next_input: snapshot.suppress_next_input,
            textarea_has_focus: snapshot.textarea_has_focus,
            active_element_tag: snapshot.active_element_tag.clone(),
            position_mode: snapshot.position_mode.clone(),
            mount_kind: snapshot.mount_kind.clone(),
            device_pixel_ratio: snapshot.device_pixel_ratio,
            textarea_value_chars: snapshot.textarea_value_chars,
            textarea_selection_start_utf16: snapshot.textarea_selection_start_utf16,
            textarea_selection_end_utf16: snapshot.textarea_selection_end_utf16,
            textarea_client_width_px: snapshot.textarea_client_width_px,
            textarea_client_height_px: snapshot.textarea_client_height_px,
            textarea_scroll_width_px: snapshot.textarea_scroll_width_px,
            textarea_scroll_height_px: snapshot.textarea_scroll_height_px,
            last_input_type: snapshot.last_input_type.clone(),
            last_beforeinput_data: (!redact_text)
                .then(|| snapshot.last_beforeinput_data.clone())
                .flatten(),
            last_input_data: (!redact_text)
                .then(|| snapshot.last_input_data.clone())
                .flatten(),
            last_key_code: snapshot.last_key_code,
            last_cursor_area: snapshot.last_cursor_area.map(RectV1::from),
            last_cursor_anchor_px: snapshot.last_cursor_anchor_px,
            last_preedit_text: (!redact_text)
                .then(|| snapshot.last_preedit_text.clone())
                .flatten(),
            last_preedit_cursor_utf16: snapshot.last_preedit_cursor_utf16,
            last_commit_text: (!redact_text)
                .then(|| snapshot.last_commit_text.clone())
                .flatten(),
            recent_events,
            beforeinput_seen: snapshot.beforeinput_seen,
            input_seen: snapshot.input_seen,
            suppressed_input_seen: snapshot.suppressed_input_seen,
            composition_start_seen: snapshot.composition_start_seen,
            composition_update_seen: snapshot.composition_update_seen,
            composition_end_seen: snapshot.composition_end_seen,
            cursor_area_set_seen: snapshot.cursor_area_set_seen,
        }
    }
}

impl UiTreeDebugSnapshotV1 {
    fn from_tree(
        app: &App,
        window: AppWindowId,
        ui: &UiTree<App>,
        renderer_perf: Option<fret_render::RendererPerfFrameSample>,
        element_runtime_state: Option<&ElementRuntime>,
        hit_test: Option<UiHitTestSnapshotV1>,
        element_runtime_snapshot: Option<ElementDiagnosticsSnapshotV1>,
        semantics: Option<UiSemanticsSnapshotV1>,
        max_gating_trace_entries: usize,
        redact_text: bool,
        max_debug_string_bytes: usize,
    ) -> Self {
        let contained_relayout_roots: HashSet<fret_core::NodeId> = ui
            .debug_view_cache_contained_relayout_roots()
            .iter()
            .copied()
            .collect();
        let environment = element_runtime_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.environment.clone());

        let window_insets = app.global::<fret_core::WindowMetricsService>().map(|svc| {
            let safe_area_known = svc.safe_area_insets_is_known(window);
            let safe_area_insets_px = svc.safe_area_insets(window).map(|e| UiPaddingInsetsV1 {
                left_px: e.left.0,
                top_px: e.top.0,
                right_px: e.right.0,
                bottom_px: e.bottom.0,
            });
            let occlusion_known = svc.occlusion_insets_is_known(window);
            let occlusion_insets_px = svc.occlusion_insets(window).map(|e| UiPaddingInsetsV1 {
                left_px: e.left.0,
                top_px: e.top.0,
                right_px: e.right.0,
                bottom_px: e.bottom.0,
            });
            UiWindowInsetsSnapshotV1 {
                safe_area_known,
                safe_area_insets_px,
                occlusion_known,
                occlusion_insets_px,
            }
        });

        let text_input = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .map(|snapshot| UiWindowTextInputSnapshotV1 {
                focus_is_text_input: snapshot.focus_is_text_input,
                is_composing: snapshot.is_composing,
                text_len_utf16: snapshot.text_len_utf16,
                selection_utf16: snapshot.selection_utf16,
                marked_utf16: snapshot.marked_utf16,
                ime_cursor_area: snapshot.ime_cursor_area.map(RectV1::from),
            });

        let runner_surface_lifecycle = app
            .global::<fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore>()
            .map(|store| store.snapshot())
            .map(|snapshot| UiRunnerSurfaceLifecycleSnapshotV1 {
                can_create_surfaces_calls: snapshot.can_create_surfaces_calls,
                destroy_surfaces_calls: snapshot.destroy_surfaces_calls,
                last_can_create_surfaces_unix_ms: snapshot.last_can_create_surfaces_unix_ms,
                last_destroy_surfaces_unix_ms: snapshot.last_destroy_surfaces_unix_ms,
                surfaces_available: snapshot.surfaces_available,
            });

        let runner_accessibility = Some({
            let snapshot = app
                .global::<fret_runtime::RunnerAccessibilityDiagnosticsStore>()
                .and_then(|store| store.snapshot(window))
                .unwrap_or_default();
            UiRunnerAccessibilitySnapshotV1 {
                activation_requests: snapshot.activation_requests,
                last_activation_unix_ms: snapshot.last_activation_unix_ms,
                last_activation_frame_id: snapshot.last_activation_frame_id.map(|id| id.0),
            }
        });

        let cache_roots: Vec<UiCacheRootStatsV1> = ui
            .debug_cache_root_stats()
            .iter()
            .map(|stats| {
                UiCacheRootStatsV1::from_stats(
                    window,
                    ui,
                    element_runtime_state,
                    semantics.as_ref(),
                    &contained_relayout_roots,
                    stats,
                    max_debug_string_bytes,
                )
            })
            .collect();

        let removed_subtrees: Vec<UiRemovedSubtreeV1> = ui
            .debug_removed_subtrees()
            .iter()
            .map(|r| {
                UiRemovedSubtreeV1::from_record(
                    window,
                    ui,
                    element_runtime_state,
                    r,
                    max_debug_string_bytes,
                )
            })
            .collect();

        let mut layout_engine_solves: Vec<UiLayoutEngineSolveV1> = ui
            .debug_layout_engine_solves()
            .iter()
            .map(UiLayoutEngineSolveV1::from_solve)
            .collect();
        for s in &mut layout_engine_solves {
            truncate_opt_string_bytes(&mut s.root_element_path, max_debug_string_bytes);
        }

        let mut layout_hotspots: Vec<UiLayoutHotspotV1> = ui
            .debug_layout_hotspots()
            .iter()
            .map(UiLayoutHotspotV1::from_hotspot)
            .collect();
        for h in &mut layout_hotspots {
            truncate_opt_string_bytes(&mut h.element_path, max_debug_string_bytes);
        }

        let mut widget_measure_hotspots: Vec<UiWidgetMeasureHotspotV1> = ui
            .debug_widget_measure_hotspots()
            .iter()
            .map(UiWidgetMeasureHotspotV1::from_hotspot)
            .collect();
        for h in &mut widget_measure_hotspots {
            truncate_opt_string_bytes(&mut h.element_path, max_debug_string_bytes);
        }

        Self {
            stats: UiFrameStatsV1::from_stats(ui.debug_stats(), renderer_perf),
            invalidation_walks: ui
                .debug_invalidation_walks()
                .iter()
                .map(|w| UiInvalidationWalkV1::from_walk(w, window, element_runtime_state))
                .collect(),
            hover_declarative_invalidation_hotspots: ui
                .debug_hover_declarative_invalidation_hotspots(20)
                .into_iter()
                .map(UiHoverDeclarativeInvalidationHotspotV1::from_hotspot)
                .collect(),
            dirty_views: ui
                .debug_dirty_views()
                .iter()
                .map(UiDirtyViewV1::from_dirty_view)
                .collect(),
            notify_requests: ui
                .debug_notify_requests()
                .iter()
                .map(UiNotifyRequestV1::from_notify_request)
                .collect(),
            virtual_list_windows: ui
                .debug_virtual_list_windows()
                .iter()
                .map(UiVirtualListWindowV1::from_window)
                .collect(),
            virtual_list_window_shift_samples: ui
                .debug_virtual_list_window_shift_samples()
                .iter()
                .map(UiVirtualListWindowShiftSampleV1::from_sample)
                .collect(),
            windowed_rows_surfaces: app
                .global::<fret_ui_kit::declarative::windowed_rows_surface::WindowedRowsSurfaceDiagnosticsStore>(
                )
                .and_then(|store| store.windows_for_window(window, app.frame_id()))
                .map(|windows| {
                    windows
                        .iter()
                        .map(UiWindowedRowsSurfaceWindowV1::from_telemetry)
                        .collect()
                })
                .unwrap_or_default(),
            retained_virtual_list_reconciles: ui
                .debug_retained_virtual_list_reconciles()
                .iter()
                .map(UiRetainedVirtualListReconcileV1::from_record)
                .collect(),
            scroll_handle_changes: ui
                .debug_scroll_handle_changes()
                .iter()
                .map(UiScrollHandleChangeV1::from_change)
                .collect(),
            prepaint_actions: ui
                .debug_prepaint_actions()
                .iter()
                .map(UiPrepaintActionV1::from_action)
                .collect(),
            model_change_hotspots: ui
                .debug_model_change_hotspots()
                .iter()
                .map(UiModelChangeHotspotV1::from_hotspot)
                .collect(),
            model_change_unobserved: ui
                .debug_model_change_unobserved()
                .iter()
                .map(UiModelChangeUnobservedV1::from_unobserved)
                .collect(),
            global_change_hotspots: ui
                .debug_global_change_hotspots()
                .iter()
                .map(|h| UiGlobalChangeHotspotV1::from_hotspot(app, h))
                .collect(),
            global_change_unobserved: ui
                .debug_global_change_unobserved()
                .iter()
                .map(|u| UiGlobalChangeUnobservedV1::from_unobserved(app, u))
                .collect(),
            cache_roots,
            overlay_synthesis: app
                .global::<fret_ui_kit::WindowOverlaySynthesisDiagnosticsStore>()
                .and_then(|diag| diag.events_for_window(window, app.frame_id()))
                .map(|events| {
                    events
                        .iter()
                        .copied()
                        .map(UiOverlaySynthesisEventV1::from_event)
                        .collect()
            })
                .unwrap_or_default(),
            viewport_input: Vec::new(),
            web_ime_bridge: app
                .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                .filter(|snapshot| **snapshot != fret_core::input::WebImeBridgeDebugSnapshot::default())
                .map(|snapshot| {
                    UiWebImeBridgeDebugSnapshotV1::from_snapshot(
                        snapshot,
                        redact_text,
                        max_debug_string_bytes,
                    )
                }),
            docking_interaction: app
                .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                .and_then(|store| store.docking_for_window(window, app.frame_id()))
                .map(UiDockingInteractionSnapshotV1::from_snapshot),
            removed_subtrees,
            layout_engine_solves,
            layout_hotspots,
            widget_measure_hotspots,
            paint_widget_hotspots: ui
                .debug_paint_widget_hotspots()
                .iter()
                .map(UiPaintWidgetHotspotV1::from_hotspot)
                .collect(),
            paint_text_prepare_hotspots: ui
                .debug_paint_text_prepare_hotspots()
                .iter()
                .map(UiPaintTextPrepareHotspotV1::from_hotspot)
                .collect(),
            input_arbitration: UiInputArbitrationSnapshotV1::from_snapshot(
                ui.input_arbitration_snapshot(),
            ),
            command_gating_trace: command_gating_trace_for_window(
                app,
                window,
                max_gating_trace_entries,
            ),
            layers_in_paint_order: ui
                .debug_layers_in_paint_order()
                .into_iter()
                .map(UiLayerInfoV1::from_layer)
                .collect(),
            all_layer_roots: ui
                .debug_layers_in_paint_order()
                .into_iter()
                .map(|l| l.root.data().as_ffi())
                .collect(),
            layer_visible_writes: ui
                .debug_layer_visible_writes()
                .iter()
                .map(UiLayerVisibleWriteV1::from_write)
                .collect(),
            overlay_policy_decisions: ui
                .debug_overlay_policy_decisions()
                .iter()
                .map(UiOverlayPolicyDecisionV1::from_decision)
                .collect(),
            environment,
            window_insets,
            text_input,
            runner_surface_lifecycle,
            runner_accessibility,
            hit_test,
            element_runtime: element_runtime_snapshot,
            semantics,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockingInteractionSnapshotV1 {
    #[serde(default)]
    pub dock_drag: Option<UiDockDragDiagnosticsV1>,
    #[serde(default)]
    pub dock_drop_resolve: Option<UiDockDropResolveDiagnosticsV1>,
    #[serde(default)]
    pub viewport_capture: Option<UiViewportCaptureDiagnosticsV1>,
    #[serde(default)]
    pub dock_graph_stats: Option<UiDockGraphStatsDiagnosticsV1>,
    #[serde(default)]
    pub dock_graph_signature: Option<UiDockGraphSignatureDiagnosticsV1>,
}

impl UiDockingInteractionSnapshotV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockingInteractionDiagnostics) -> Self {
        Self {
            dock_drag: snapshot
                .dock_drag
                .map(UiDockDragDiagnosticsV1::from_snapshot),
            dock_drop_resolve: snapshot
                .dock_drop_resolve
                .as_ref()
                .map(UiDockDropResolveDiagnosticsV1::from_snapshot),
            viewport_capture: snapshot
                .viewport_capture
                .map(UiViewportCaptureDiagnosticsV1::from_snapshot),
            dock_graph_stats: snapshot
                .dock_graph_stats
                .map(UiDockGraphStatsDiagnosticsV1::from_snapshot),
            dock_graph_signature: snapshot
                .dock_graph_signature
                .as_ref()
                .map(UiDockGraphSignatureDiagnosticsV1::from_snapshot),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockGraphSignatureDiagnosticsV1 {
    pub signature: String,
    pub fingerprint64: u64,
}

impl UiDockGraphSignatureDiagnosticsV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockGraphSignatureDiagnostics) -> Self {
        Self {
            signature: snapshot.signature.clone(),
            fingerprint64: snapshot.fingerprint64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockGraphStatsDiagnosticsV1 {
    pub node_count: u32,
    pub tabs_count: u32,
    pub split_count: u32,
    pub floating_count: u32,
    pub max_depth: u32,
    pub max_split_depth: u32,
    pub canonical_ok: bool,
    pub has_nested_same_axis_splits: bool,
}

impl UiDockGraphStatsDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockGraphStatsDiagnostics) -> Self {
        Self {
            node_count: snapshot.node_count,
            tabs_count: snapshot.tabs_count,
            split_count: snapshot.split_count,
            floating_count: snapshot.floating_count,
            max_depth: snapshot.max_depth,
            max_split_depth: snapshot.max_split_depth,
            canonical_ok: snapshot.canonical_ok,
            has_nested_same_axis_splits: snapshot.has_nested_same_axis_splits,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropResolveSourceV1 {
    InvertDocking,
    OutsideWindow,
    FloatZone,
    EmptyDockSpace,
    LayoutBoundsMiss,
    LatchedPreviousHover,
    TabBar,
    FloatingTitleBar,
    OuterHintRect,
    InnerHintRect,
    None,
}

impl UiDockDropResolveSourceV1 {
    fn from_source(source: fret_runtime::DockDropResolveSource) -> Self {
        match source {
            fret_runtime::DockDropResolveSource::InvertDocking => Self::InvertDocking,
            fret_runtime::DockDropResolveSource::OutsideWindow => Self::OutsideWindow,
            fret_runtime::DockDropResolveSource::FloatZone => Self::FloatZone,
            fret_runtime::DockDropResolveSource::EmptyDockSpace => Self::EmptyDockSpace,
            fret_runtime::DockDropResolveSource::LayoutBoundsMiss => Self::LayoutBoundsMiss,
            fret_runtime::DockDropResolveSource::LatchedPreviousHover => Self::LatchedPreviousHover,
            fret_runtime::DockDropResolveSource::TabBar => Self::TabBar,
            fret_runtime::DockDropResolveSource::FloatingTitleBar => Self::FloatingTitleBar,
            fret_runtime::DockDropResolveSource::OuterHintRect => Self::OuterHintRect,
            fret_runtime::DockDropResolveSource::InnerHintRect => Self::InnerHintRect,
            fret_runtime::DockDropResolveSource::None => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDockDropCandidateRectKindV1 {
    WindowBounds,
    DockBounds,
    FloatZone,
    LayoutBounds,
    RootRect,
    LeafTabsRect,
    TabBarRect,
    InnerHintRect,
    OuterHintRect,
}

impl UiDockDropCandidateRectKindV1 {
    fn from_kind(kind: fret_runtime::DockDropCandidateRectKind) -> Self {
        match kind {
            fret_runtime::DockDropCandidateRectKind::WindowBounds => Self::WindowBounds,
            fret_runtime::DockDropCandidateRectKind::DockBounds => Self::DockBounds,
            fret_runtime::DockDropCandidateRectKind::FloatZone => Self::FloatZone,
            fret_runtime::DockDropCandidateRectKind::LayoutBounds => Self::LayoutBounds,
            fret_runtime::DockDropCandidateRectKind::RootRect => Self::RootRect,
            fret_runtime::DockDropCandidateRectKind::LeafTabsRect => Self::LeafTabsRect,
            fret_runtime::DockDropCandidateRectKind::TabBarRect => Self::TabBarRect,
            fret_runtime::DockDropCandidateRectKind::InnerHintRect => Self::InnerHintRect,
            fret_runtime::DockDropCandidateRectKind::OuterHintRect => Self::OuterHintRect,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockDropCandidateRectDiagnosticsV1 {
    pub kind: UiDockDropCandidateRectKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<UiDropZoneV1>,
    pub rect: RectV1,
}

impl UiDockDropCandidateRectDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDropCandidateRectDiagnostics) -> Self {
        Self {
            kind: UiDockDropCandidateRectKindV1::from_kind(snapshot.kind),
            zone: snapshot.zone.map(UiDropZoneV1::from_zone),
            rect: RectV1::from(snapshot.rect),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDropZoneV1 {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

impl UiDropZoneV1 {
    fn from_zone(zone: fret_core::DropZone) -> Self {
        match zone {
            fret_core::DropZone::Center => Self::Center,
            fret_core::DropZone::Left => Self::Left,
            fret_core::DropZone::Right => Self::Right,
            fret_core::DropZone::Top => Self::Top,
            fret_core::DropZone::Bottom => Self::Bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockDropTargetDiagnosticsV1 {
    pub layout_root: u64,
    pub tabs: u64,
    pub zone: UiDropZoneV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insert_index: Option<u64>,
    pub outer: bool,
}

impl UiDockDropTargetDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDropTargetDiagnostics) -> Self {
        Self {
            layout_root: snapshot.layout_root.data().as_ffi(),
            tabs: snapshot.tabs.data().as_ffi(),
            zone: UiDropZoneV1::from_zone(snapshot.zone),
            insert_index: snapshot.insert_index.map(|v| v as u64),
            outer: snapshot.outer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiDockDropPreviewKindDiagnosticsV1 {
    WrapBinary,
    InsertIntoSplit {
        axis: String,
        split: u64,
        insert_index: u64,
    },
}

impl UiDockDropPreviewKindDiagnosticsV1 {
    fn from_kind(kind: fret_runtime::DockDropPreviewKindDiagnostics) -> Self {
        match kind {
            fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary => Self::WrapBinary,
            fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit {
                axis,
                split,
                insert_index,
            } => Self::InsertIntoSplit {
                axis: match axis {
                    fret_core::Axis::Horizontal => "horizontal",
                    fret_core::Axis::Vertical => "vertical",
                }
                .to_string(),
                split: split.data().as_ffi(),
                insert_index: insert_index as u64,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDropPreviewDiagnosticsV1 {
    pub kind: UiDockDropPreviewKindDiagnosticsV1,
}

impl UiDockDropPreviewDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDropPreviewDiagnostics) -> Self {
        Self {
            kind: UiDockDropPreviewKindDiagnosticsV1::from_kind(snapshot.kind),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDropResolveDiagnosticsV1 {
    pub pointer_id: u64,
    pub position: PointV1,
    pub window_bounds: RectV1,
    pub dock_bounds: RectV1,
    pub source: UiDockDropResolveSourceV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<UiDockDropTargetDiagnosticsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<UiDockDropPreviewDiagnosticsV1>,
    #[serde(default)]
    pub candidates: Vec<UiDockDropCandidateRectDiagnosticsV1>,
}

impl UiDockDropResolveDiagnosticsV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockDropResolveDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            position: PointV1::from(snapshot.position),
            window_bounds: RectV1::from(snapshot.window_bounds),
            dock_bounds: RectV1::from(snapshot.dock_bounds),
            source: UiDockDropResolveSourceV1::from_source(snapshot.source),
            resolved: snapshot
                .resolved
                .map(UiDockDropTargetDiagnosticsV1::from_snapshot),
            preview: snapshot
                .preview
                .map(UiDockDropPreviewDiagnosticsV1::from_snapshot),
            candidates: snapshot
                .candidates
                .iter()
                .copied()
                .map(UiDockDropCandidateRectDiagnosticsV1::from_snapshot)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockDragDiagnosticsV1 {
    pub pointer_id: u64,
    pub source_window: u64,
    pub current_window: u64,
    pub dragging: bool,
    pub cross_window_hover: bool,
    #[serde(default)]
    pub transparent_payload_applied: bool,
    #[serde(default)]
    pub window_under_cursor_source: String,
}

impl UiDockDragDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDragDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            source_window: snapshot.source_window.data().as_ffi(),
            current_window: snapshot.current_window.data().as_ffi(),
            dragging: snapshot.dragging,
            cross_window_hover: snapshot.cross_window_hover,
            transparent_payload_applied: snapshot.transparent_payload_applied,
            window_under_cursor_source: dock_drag_window_under_cursor_source_label(
                snapshot.window_under_cursor_source,
            )
            .to_string(),
        }
    }
}

fn dock_drag_window_under_cursor_source_label(
    source: fret_runtime::WindowUnderCursorSource,
) -> &'static str {
    use fret_runtime::WindowUnderCursorSource as Src;
    match source {
        Src::Unknown => "unknown",
        Src::PlatformWin32 => "platform_win32",
        Src::PlatformMacos => "platform_macos",
        Src::Latched => "latched",
        Src::HeuristicZOrder => "heuristic_z_order",
        Src::HeuristicRects => "heuristic_rects",
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiViewportCaptureDiagnosticsV1 {
    pub pointer_id: u64,
    pub target: u64,
}

impl UiViewportCaptureDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::ViewportCaptureDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            target: snapshot.target.data().as_ffi(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiViewportInputEventV1 {
    pub target: u64,
    pub pointer_id: u64,
    pub pointer_type: String,
    pub cursor_px: PointV1,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: UiViewportInputKindV1,
}

impl UiViewportInputEventV1 {
    fn from_event(event: fret_core::ViewportInputEvent) -> Self {
        Self {
            target: event.target.data().as_ffi(),
            pointer_id: event.pointer_id.0 as u64,
            pointer_type: viewport_pointer_type_label(event.pointer_type).to_string(),
            cursor_px: PointV1::from(event.cursor_px),
            uv: event.uv,
            target_px: event.target_px,
            kind: UiViewportInputKindV1::from_kind(event.kind),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiViewportInputKindV1 {
    PointerMove {
        buttons: UiMouseButtonsV1,
        modifiers: UiKeyModifiersV1,
    },
    PointerDown {
        button: UiMouseButtonV1,
        modifiers: UiKeyModifiersV1,
        click_count: u8,
    },
    PointerUp {
        button: UiMouseButtonV1,
        modifiers: UiKeyModifiersV1,
        is_click: bool,
        click_count: u8,
    },
    PointerCancel {
        buttons: UiMouseButtonsV1,
        modifiers: UiKeyModifiersV1,
        reason: String,
    },
    Wheel {
        delta: PointV1,
        modifiers: UiKeyModifiersV1,
    },
}

impl UiViewportInputKindV1 {
    fn from_kind(kind: fret_core::ViewportInputKind) -> Self {
        match kind {
            fret_core::ViewportInputKind::PointerMove { buttons, modifiers } => Self::PointerMove {
                buttons: UiMouseButtonsV1::from_buttons(buttons),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
            },
            fret_core::ViewportInputKind::PointerDown {
                button,
                modifiers,
                click_count,
            } => Self::PointerDown {
                button: UiMouseButtonV1::from_button(button),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                click_count,
            },
            fret_core::ViewportInputKind::PointerUp {
                button,
                modifiers,
                is_click,
                click_count,
            } => Self::PointerUp {
                button: UiMouseButtonV1::from_button(button),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                is_click,
                click_count,
            },
            fret_core::ViewportInputKind::PointerCancel {
                buttons,
                modifiers,
                reason,
            } => Self::PointerCancel {
                buttons: UiMouseButtonsV1::from_buttons(buttons),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                reason: viewport_cancel_reason_label(reason).to_string(),
            },
            fret_core::ViewportInputKind::Wheel { delta, modifiers } => Self::Wheel {
                delta: PointV1::from(delta),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiMouseButtonsV1 {
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub middle: bool,
}

impl UiMouseButtonsV1 {
    fn from_buttons(buttons: fret_core::MouseButtons) -> Self {
        Self {
            left: buttons.left,
            right: buttons.right,
            middle: buttons.middle,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAxisV1 {
    Horizontal,
    Vertical,
}

impl UiAxisV1 {
    fn from_axis(axis: fret_core::Axis) -> Self {
        match axis {
            fret_core::Axis::Horizontal => Self::Horizontal,
            fret_core::Axis::Vertical => Self::Vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisKindV1 {
    Modal,
    Popover,
    Hover,
    Tooltip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListMeasureModeV1 {
    Fixed,
    Measured,
    Known,
}

impl UiVirtualListMeasureModeV1 {
    fn from_mode(mode: fret_ui::element::VirtualListMeasureMode) -> Self {
        match mode {
            fret_ui::element::VirtualListMeasureMode::Fixed => Self::Fixed,
            fret_ui::element::VirtualListMeasureMode::Measured => Self::Measured,
            fret_ui::element::VirtualListMeasureMode::Known => Self::Known,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftKindV1 {
    None,
    Prefetch,
    Escape,
}

impl Default for UiVirtualListWindowShiftKindV1 {
    fn default() -> Self {
        Self::None
    }
}

impl UiVirtualListWindowShiftKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugVirtualListWindowShiftKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::None => Self::None,
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::Prefetch => Self::Prefetch,
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::Escape => Self::Escape,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftReasonV1 {
    ScrollOffset,
    ViewportResize,
    ItemsRevision,
    ScrollToItem,
    InputsChange,
    Unknown,
}

impl UiVirtualListWindowShiftReasonV1 {
    fn from_reason(reason: fret_ui::tree::UiDebugVirtualListWindowShiftReason) -> Self {
        match reason {
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset => Self::ScrollOffset,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ViewportResize => {
                Self::ViewportResize
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision => {
                Self::ItemsRevision
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem => Self::ScrollToItem,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::InputsChange => Self::InputsChange,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftApplyModeV1 {
    RetainedReconcile,
    NonRetainedRerender,
}

impl UiVirtualListWindowShiftApplyModeV1 {
    fn from_mode(mode: fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode) -> Self {
        match mode {
            fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile => {
                Self::RetainedReconcile
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender => {
                Self::NonRetainedRerender
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiVirtualRangeV1 {
    pub start_index: u64,
    pub end_index: u64,
    pub overscan: u64,
    pub count: u64,
}

impl UiVirtualRangeV1 {
    fn from_range(range: fret_ui::virtual_list::VirtualRange) -> Self {
        Self {
            start_index: range.start_index as u64,
            end_index: range.end_index as u64,
            overscan: range.overscan as u64,
            count: range.count as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiVirtualListWindowV1 {
    pub node: u64,
    pub element: u64,
    #[serde(default)]
    pub source: UiVirtualListWindowSourceV1,
    pub axis: UiAxisV1,
    #[serde(default)]
    pub is_probe_layout: bool,
    pub items_len: u64,
    pub items_revision: u64,
    pub prev_items_revision: u64,
    pub measure_mode: UiVirtualListMeasureModeV1,
    pub overscan: u64,
    #[serde(default)]
    pub policy_key: u64,
    #[serde(default)]
    pub inputs_key: u64,
    pub viewport: f32,
    pub prev_viewport: f32,
    pub offset: f32,
    pub prev_offset: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_window_range: Option<UiVirtualRangeV1>,
    #[serde(default)]
    pub deferred_scroll_to_item: bool,
    #[serde(default)]
    pub deferred_scroll_consumed: bool,
    #[serde(default)]
    pub window_mismatch: bool,
    #[serde(default)]
    pub window_shift_kind: UiVirtualListWindowShiftKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_reason: Option<UiVirtualListWindowShiftReasonV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_apply_mode: Option<UiVirtualListWindowShiftApplyModeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_invalidation_detail: Option<String>,
}

impl UiVirtualListWindowV1 {
    fn from_window(window: &fret_ui::tree::UiDebugVirtualListWindow) -> Self {
        Self {
            node: key_to_u64(window.node),
            element: window.element.0,
            source: UiVirtualListWindowSourceV1::from_source(window.source),
            axis: UiAxisV1::from_axis(window.axis),
            is_probe_layout: window.is_probe_layout,
            items_len: window.items_len as u64,
            items_revision: window.items_revision,
            prev_items_revision: window.prev_items_revision,
            measure_mode: UiVirtualListMeasureModeV1::from_mode(window.measure_mode),
            overscan: window.overscan as u64,
            policy_key: window.policy_key,
            inputs_key: window.inputs_key,
            viewport: window.viewport.0,
            prev_viewport: window.prev_viewport.0,
            offset: window.offset.0,
            prev_offset: window.prev_offset.0,
            window_range: window.window_range.map(UiVirtualRangeV1::from_range),
            prev_window_range: window.prev_window_range.map(UiVirtualRangeV1::from_range),
            render_window_range: window.render_window_range.map(UiVirtualRangeV1::from_range),
            deferred_scroll_to_item: window.deferred_scroll_to_item,
            deferred_scroll_consumed: window.deferred_scroll_consumed,
            window_mismatch: window.window_mismatch,
            window_shift_kind: UiVirtualListWindowShiftKindV1::from_kind(window.window_shift_kind),
            window_shift_reason: window
                .window_shift_reason
                .map(UiVirtualListWindowShiftReasonV1::from_reason),
            window_shift_apply_mode: window
                .window_shift_apply_mode
                .map(UiVirtualListWindowShiftApplyModeV1::from_mode),
            window_shift_invalidation_detail: window
                .window_shift_invalidation_detail
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiVirtualListWindowShiftSampleV1 {
    pub frame_id: u64,
    pub source: UiVirtualListWindowSourceV1,
    pub node: u64,
    pub element: u64,
    pub window_shift_kind: UiVirtualListWindowShiftKindV1,
    pub window_shift_reason: UiVirtualListWindowShiftReasonV1,
    pub window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_invalidation_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_window_range: Option<UiVirtualRangeV1>,
}

impl UiVirtualListWindowShiftSampleV1 {
    fn from_sample(sample: &fret_ui::tree::UiDebugVirtualListWindowShiftSample) -> Self {
        Self {
            frame_id: sample.frame_id.0,
            source: UiVirtualListWindowSourceV1::from_source(sample.source),
            node: key_to_u64(sample.node),
            element: sample.element.0,
            window_shift_kind: UiVirtualListWindowShiftKindV1::from_kind(sample.window_shift_kind),
            window_shift_reason: UiVirtualListWindowShiftReasonV1::from_reason(
                sample.window_shift_reason,
            ),
            window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1::from_mode(
                sample.window_shift_apply_mode,
            ),
            window_shift_invalidation_detail: sample
                .window_shift_invalidation_detail
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
            prev_window_range: sample.prev_window_range.map(UiVirtualRangeV1::from_range),
            window_range: sample.window_range.map(UiVirtualRangeV1::from_range),
            render_window_range: sample.render_window_range.map(UiVirtualRangeV1::from_range),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWindowedRowsSurfaceWindowV1 {
    pub callsite_id: u64,
    pub location: UiSourceLocationV1,

    pub len: u64,
    pub row_height: f32,
    pub overscan: u64,
    pub gap: f32,
    pub scroll_margin: f32,

    pub viewport_height: f32,
    pub offset_y: f32,
    pub content_height: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_start: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_end: Option<u64>,
    pub visible_count: u64,
}

impl UiWindowedRowsSurfaceWindowV1 {
    fn from_telemetry(
        telemetry: &fret_ui_kit::declarative::windowed_rows_surface::WindowedRowsSurfaceWindowTelemetry,
    ) -> Self {
        Self {
            callsite_id: telemetry.callsite_id,
            location: UiSourceLocationV1 {
                file: telemetry.file.to_string(),
                line: telemetry.line,
                column: telemetry.column,
            },
            len: telemetry.len,
            row_height: telemetry.row_height.0,
            overscan: telemetry.overscan,
            gap: telemetry.gap.0,
            scroll_margin: telemetry.scroll_margin.0,
            viewport_height: telemetry.viewport_height.0,
            offset_y: telemetry.offset_y.0,
            content_height: telemetry.content_height.0,
            visible_start: telemetry.visible_start,
            visible_end: telemetry.visible_end,
            visible_count: telemetry.visible_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRetainedVirtualListReconcileV1 {
    pub node: u64,
    pub element: u64,
    pub prev_items: u64,
    pub next_items: u64,
    pub preserved_items: u64,
    pub attached_items: u64,
    pub detached_items: u64,
    #[serde(default)]
    pub reused_from_keep_alive_items: u64,
    #[serde(default)]
    pub kept_alive_items: u64,
    #[serde(default)]
    pub evicted_keep_alive_items: u64,
    #[serde(default)]
    pub keep_alive_pool_len_before: u64,
    #[serde(default)]
    pub keep_alive_pool_len_after: u64,
}

impl UiRetainedVirtualListReconcileV1 {
    fn from_record(record: &fret_ui::tree::UiDebugRetainedVirtualListReconcile) -> Self {
        Self {
            node: key_to_u64(record.node),
            element: record.element.0,
            prev_items: record.prev_items as u64,
            next_items: record.next_items as u64,
            preserved_items: record.preserved_items as u64,
            attached_items: record.attached_items as u64,
            detached_items: record.detached_items as u64,
            reused_from_keep_alive_items: record.reused_from_keep_alive_items as u64,
            kept_alive_items: record.kept_alive_items as u64,
            evicted_keep_alive_items: record.evicted_keep_alive_items as u64,
            keep_alive_pool_len_before: record.keep_alive_pool_len_before as u64,
            keep_alive_pool_len_after: record.keep_alive_pool_len_after as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowSourceV1 {
    Prepaint,
    #[serde(other)]
    Layout,
}

impl Default for UiVirtualListWindowSourceV1 {
    fn default() -> Self {
        Self::Layout
    }
}

impl UiVirtualListWindowSourceV1 {
    fn from_source(source: fret_ui::tree::UiDebugVirtualListWindowSource) -> Self {
        match source {
            fret_ui::tree::UiDebugVirtualListWindowSource::Layout => Self::Layout,
            fret_ui::tree::UiDebugVirtualListWindowSource::Prepaint => Self::Prepaint,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisSourceV1 {
    CachedDeclaration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScrollHandleChangeKindV1 {
    Layout,
    HitTestOnly,
}

impl UiScrollHandleChangeKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugScrollHandleChangeKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugScrollHandleChangeKind::Layout => Self::Layout,
            fret_ui::tree::UiDebugScrollHandleChangeKind::HitTestOnly => Self::HitTestOnly,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollHandleChangeV1 {
    pub handle_key: u64,
    pub kind: UiScrollHandleChangeKindV1,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_revision: Option<u64>,
    pub offset_x: f32,
    pub offset_y: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_offset_x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_offset_y: Option<f32>,
    pub viewport_w: f32,
    pub viewport_h: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_viewport_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_viewport_h: Option<f32>,
    pub content_w: f32,
    pub content_h: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_content_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_content_h: Option<f32>,
    #[serde(default)]
    pub offset_changed: bool,
    #[serde(default)]
    pub viewport_changed: bool,
    #[serde(default)]
    pub content_changed: bool,
    #[serde(default)]
    pub bound_elements: u32,
    #[serde(default)]
    pub bound_nodes_sample: Vec<u64>,
    #[serde(default)]
    pub upgraded_to_layout_bindings: u32,
}

impl UiScrollHandleChangeV1 {
    fn from_change(change: &fret_ui::tree::UiDebugScrollHandleChange) -> Self {
        Self {
            handle_key: change.handle_key as u64,
            kind: UiScrollHandleChangeKindV1::from_kind(change.kind),
            revision: change.revision,
            prev_revision: change.prev_revision,
            offset_x: change.offset.x.0,
            offset_y: change.offset.y.0,
            prev_offset_x: change.prev_offset.map(|p| p.x.0),
            prev_offset_y: change.prev_offset.map(|p| p.y.0),
            viewport_w: change.viewport.width.0,
            viewport_h: change.viewport.height.0,
            prev_viewport_w: change.prev_viewport.map(|s| s.width.0),
            prev_viewport_h: change.prev_viewport.map(|s| s.height.0),
            content_w: change.content.width.0,
            content_h: change.content.height.0,
            prev_content_w: change.prev_content.map(|s| s.width.0),
            prev_content_h: change.prev_content.map(|s| s.height.0),
            offset_changed: change.offset_changed,
            viewport_changed: change.viewport_changed,
            content_changed: change.content_changed,
            bound_elements: change.bound_elements,
            bound_nodes_sample: change
                .bound_nodes_sample
                .iter()
                .copied()
                .map(key_to_u64)
                .collect(),
            upgraded_to_layout_bindings: change.upgraded_to_layout_bindings,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPrepaintActionKindV1 {
    Invalidate,
    RequestRedraw,
    RequestAnimationFrame,
    VirtualListWindowShift,
    ChartSamplingWindowShift,
    NodeGraphCullWindowShift,
}

impl UiPrepaintActionKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugPrepaintActionKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugPrepaintActionKind::Invalidate => Self::Invalidate,
            fret_ui::tree::UiDebugPrepaintActionKind::RequestRedraw => Self::RequestRedraw,
            fret_ui::tree::UiDebugPrepaintActionKind::RequestAnimationFrame => {
                Self::RequestAnimationFrame
            }
            fret_ui::tree::UiDebugPrepaintActionKind::VirtualListWindowShift => {
                Self::VirtualListWindowShift
            }
            fret_ui::tree::UiDebugPrepaintActionKind::ChartSamplingWindowShift => {
                Self::ChartSamplingWindowShift
            }
            fret_ui::tree::UiDebugPrepaintActionKind::NodeGraphCullWindowShift => {
                Self::NodeGraphCullWindowShift
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrepaintActionV1 {
    pub node: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node: Option<u64>,
    pub kind: UiPrepaintActionKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_list_window_shift_kind: Option<UiVirtualListWindowShiftKindV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_list_window_shift_reason: Option<UiVirtualListWindowShiftReasonV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart_sampling_window_key: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_graph_cull_window_key: Option<u64>,
    #[serde(default)]
    pub frame_id: u64,
}

impl UiPrepaintActionV1 {
    fn from_action(action: &fret_ui::tree::UiDebugPrepaintAction) -> Self {
        let invalidation = action.invalidation.map(|inv| match inv {
            fret_ui::Invalidation::Layout => "layout",
            fret_ui::Invalidation::Paint => "paint",
            fret_ui::Invalidation::HitTest => "hit_test",
            fret_ui::Invalidation::HitTestOnly => "hit_test_only",
        });

        Self {
            node: key_to_u64(action.node),
            target_node: action.target.map(key_to_u64),
            kind: UiPrepaintActionKindV1::from_kind(action.kind),
            invalidation: invalidation.map(|s| s.to_string()),
            element: action.element.map(|id| id.0),
            virtual_list_window_shift_kind: action
                .virtual_list_window_shift_kind
                .map(UiVirtualListWindowShiftKindV1::from_kind),
            virtual_list_window_shift_reason: action
                .virtual_list_window_shift_reason
                .map(UiVirtualListWindowShiftReasonV1::from_reason),
            chart_sampling_window_key: action.chart_sampling_window_key,
            node_graph_cull_window_key: action.node_graph_cull_window_key,
            frame_id: action.frame_id.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisOutcomeV1 {
    Synthesized,
    SuppressedMissingTrigger,
    SuppressedTriggerNotLiveInCurrentFrame,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiOverlaySynthesisEventV1 {
    pub kind: UiOverlaySynthesisKindV1,
    pub id: u64,
    pub source: UiOverlaySynthesisSourceV1,
    pub outcome: UiOverlaySynthesisOutcomeV1,
}

impl UiOverlaySynthesisEventV1 {
    fn from_event(e: fret_ui_kit::OverlaySynthesisEvent) -> Self {
        use fret_ui_kit::OverlaySynthesisKind;
        use fret_ui_kit::OverlaySynthesisOutcome;
        use fret_ui_kit::OverlaySynthesisSource;

        let kind = match e.kind {
            OverlaySynthesisKind::Modal => UiOverlaySynthesisKindV1::Modal,
            OverlaySynthesisKind::Popover => UiOverlaySynthesisKindV1::Popover,
            OverlaySynthesisKind::Hover => UiOverlaySynthesisKindV1::Hover,
            OverlaySynthesisKind::Tooltip => UiOverlaySynthesisKindV1::Tooltip,
        };
        let source = match e.source {
            OverlaySynthesisSource::CachedDeclaration => {
                UiOverlaySynthesisSourceV1::CachedDeclaration
            }
        };
        let outcome = match e.outcome {
            OverlaySynthesisOutcome::Synthesized => UiOverlaySynthesisOutcomeV1::Synthesized,
            OverlaySynthesisOutcome::SuppressedMissingTrigger => {
                UiOverlaySynthesisOutcomeV1::SuppressedMissingTrigger
            }
            OverlaySynthesisOutcome::SuppressedTriggerNotLiveInCurrentFrame => {
                UiOverlaySynthesisOutcomeV1::SuppressedTriggerNotLiveInCurrentFrame
            }
        };

        Self {
            kind,
            id: e.id.0,
            source,
            outcome,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCommandGatingTraceEntryV1 {
    pub command: String,
    pub enabled: bool,
    pub reason: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_path: Option<String>,
    /// Structured explanation of why the command is disabled (multiple blockers may apply).
    #[serde(default)]
    pub blocked_by: Vec<String>,
    /// Best-effort detail fields to make debugging inconsistent gating easier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_when: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_when: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_override: Option<bool>,
    #[serde(default)]
    pub command_registered: bool,
}

#[derive(Debug, Clone)]
struct UiCommandGatingTraceCandidate {
    command: fret_runtime::CommandId,
    source: &'static str,
    menu_path: Option<String>,
    menu_when: Option<fret_runtime::WhenExpr>,
}

fn command_gating_trace_for_window(
    app: &App,
    window: AppWindowId,
    max_entries: usize,
) -> Vec<UiCommandGatingTraceEntryV1> {
    let gating = fret_runtime::best_effort_snapshot_for_window(app, window);

    let mut candidates: Vec<UiCommandGatingTraceCandidate> = Vec::new();

    // 1) Explicit gating inputs (useful for verifying that snapshots are being published).
    for (cmd, _) in gating.enabled_overrides() {
        candidates.push(UiCommandGatingTraceCandidate {
            command: cmd.clone(),
            source: "enabled_overrides",
            menu_path: None,
            menu_when: None,
        });
    }
    if let Some(map) = gating.action_availability() {
        for (cmd, _) in map {
            candidates.push(UiCommandGatingTraceCandidate {
                command: cmd.clone(),
                source: "action_availability",
                menu_path: None,
                menu_when: None,
            });
        }
    }

    // 2) Effective OS menubar model (data-only). This is the closest source of truth for
    // "visible menu commands" from the app's perspective.
    if let Some(menu_bar) = fret_app::effective_menu_bar(app) {
        collect_menu_bar_commands(&menu_bar, &mut candidates);
    }

    // 3) Command palette catalog (best-effort). This approximates the set of entries derived from
    // host commands; the actual palette filters further by query/group options.
    for (id, meta) in app.commands().iter() {
        if meta.hidden {
            continue;
        }
        candidates.push(UiCommandGatingTraceCandidate {
            command: id.clone(),
            source: "command_palette_catalog",
            menu_path: None,
            menu_when: None,
        });
    }

    // Always include a core, cross-surface set even if the host didn't publish any snapshot yet.
    for &cmd in &[
        "edit.undo",
        "edit.redo",
        "edit.copy",
        "edit.cut",
        "edit.paste",
        "edit.select_all",
        "focus.menu_bar",
    ] {
        candidates.push(UiCommandGatingTraceCandidate {
            command: fret_runtime::CommandId::from(cmd),
            source: "core",
            menu_path: None,
            menu_when: None,
        });
    }

    // Deduplicate by (command, source, menu_path) so repeated insertions don't explode snapshots.
    let mut seen: HashSet<(String, &'static str, Option<String>)> = HashSet::new();
    candidates.retain(|c| {
        let key = (
            c.command.as_str().to_string(),
            c.source,
            c.menu_path.clone(),
        );
        if seen.contains(&key) {
            return false;
        }
        seen.insert(key);
        true
    });

    candidates.sort_by(|a, b| {
        a.source
            .cmp(b.source)
            .then_with(|| a.menu_path.cmp(&b.menu_path))
            .then_with(|| a.command.as_str().cmp(b.command.as_str()))
    });

    let max_entries = max_entries.min(2000);
    candidates
        .into_iter()
        .take(max_entries)
        .map(|c| {
            let decision =
                command_gating_decision_trace(app, &gating, &c.command, c.menu_when.as_ref());

            UiCommandGatingTraceEntryV1 {
                command: c.command.as_str().to_string(),
                enabled: decision.enabled,
                reason: decision.reason,
                scope: decision.scope,
                source: c.source.to_string(),
                menu_path: c.menu_path,
                blocked_by: decision.blocked_by,
                action_available: decision.action_available,
                command_when: decision.command_when,
                menu_when: decision.menu_when,
                enabled_override: decision.enabled_override,
                command_registered: decision.command_registered,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct UiCommandGatingDecisionTrace {
    enabled: bool,
    reason: String,
    scope: String,
    blocked_by: Vec<String>,
    action_available: Option<bool>,
    command_when: Option<bool>,
    menu_when: Option<bool>,
    enabled_override: Option<bool>,
    command_registered: bool,
}

fn command_gating_decision_trace(
    app: &App,
    gating: &fret_runtime::WindowCommandGatingSnapshot,
    command: &fret_runtime::CommandId,
    menu_when: Option<&fret_runtime::WhenExpr>,
) -> UiCommandGatingDecisionTrace {
    let meta = app.commands().get(command.clone());
    let scope = meta
        .map(|m| format!("{:?}", m.scope))
        .unwrap_or_else(|| "Unknown".to_string());

    let mut blocked_by: Vec<String> = Vec::new();

    let action_available = if let Some(meta) = meta
        && meta.scope == fret_runtime::CommandScope::Widget
        && let Some(map) = gating.action_availability()
        && let Some(is_available) = map.get(command).copied()
    {
        Some(is_available)
    } else {
        None
    };
    if action_available == Some(false) {
        blocked_by.push("action_availability".to_string());
    }

    let command_when = meta.and_then(|m| m.when.as_ref().map(|w| w.eval(gating.input_ctx())));
    if command_when == Some(false) {
        blocked_by.push("when".to_string());
    }

    let enabled_override = gating.enabled_overrides().get(command).copied();
    if enabled_override == Some(false) {
        blocked_by.push("enabled_override".to_string());
    }

    let menu_when = menu_when.map(|w| w.eval(gating.input_ctx()));
    if menu_when == Some(false) {
        blocked_by.push("menu_when".to_string());
    }

    let command_registered = meta.is_some();
    let enabled = blocked_by.is_empty();

    // Keep a stable "primary reason" string for backwards compatibility / easy grepping.
    let reason = if blocked_by.iter().any(|b| b == "action_availability") {
        "action_unavailable"
    } else if blocked_by.iter().any(|b| b == "when") {
        "when_false"
    } else if blocked_by.iter().any(|b| b == "enabled_override") {
        "disabled_override"
    } else if blocked_by.iter().any(|b| b == "menu_when") {
        "menu_when_false"
    } else if !command_registered {
        "unknown_command"
    } else {
        "enabled"
    }
    .to_string();

    UiCommandGatingDecisionTrace {
        enabled,
        reason,
        scope,
        blocked_by,
        action_available,
        command_when,
        menu_when,
        enabled_override,
        command_registered,
    }
}

fn collect_menu_bar_commands(
    menu_bar: &fret_runtime::MenuBar,
    out: &mut Vec<UiCommandGatingTraceCandidate>,
) {
    for menu in &menu_bar.menus {
        let menu_title = menu.title.as_ref().to_string();
        collect_menu_items(&menu_title, &menu.items, out);
    }
}

fn collect_menu_items(
    prefix: &str,
    items: &[fret_runtime::MenuItem],
    out: &mut Vec<UiCommandGatingTraceCandidate>,
) {
    for item in items {
        match item {
            fret_runtime::MenuItem::Command { command, when, .. } => {
                out.push(UiCommandGatingTraceCandidate {
                    command: command.clone(),
                    source: "menu_bar",
                    menu_path: Some(prefix.to_string()),
                    menu_when: when.clone(),
                });
            }
            fret_runtime::MenuItem::Label { .. } => {}
            fret_runtime::MenuItem::Separator | fret_runtime::MenuItem::SystemMenu { .. } => {}
            fret_runtime::MenuItem::Submenu {
                title,
                when: _,
                items,
            } => {
                let next = format!("{prefix} > {}", title.as_ref());
                collect_menu_items(&next, items, out);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHoverDeclarativeInvalidationHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub hit_test: u32,
    #[serde(default)]
    pub layout: u32,
    #[serde(default)]
    pub paint: u32,
}

impl UiHoverDeclarativeInvalidationHotspotV1 {
    fn from_hotspot(h: fret_ui::tree::UiDebugHoverDeclarativeInvalidationHotspot) -> Self {
        Self {
            node: key_to_u64(h.node),
            element: h.element.map(|e| e.0),
            hit_test: h.hit_test,
            layout: h.layout,
            paint: h.paint,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRemovedSubtreeV1 {
    pub root: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_parent_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_element_path: Option<String>,
    #[serde(default)]
    pub root_parent: Option<u64>,
    #[serde(default)]
    pub root_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_is_view_cache_reuse_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_frame_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default)]
    pub root_layer: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_layer_visible: Option<bool>,
    #[serde(default)]
    pub reachable_from_layer_roots: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reachable_from_view_cache_roots: Option<bool>,
    #[serde(default)]
    pub unreachable_from_liveness_roots: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liveness_layer_roots_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_cache_reuse_roots_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_cache_reuse_root_nodes_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_listed_under_reuse_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_root_path: Option<String>,
    #[serde(default)]
    pub root_children_len: u32,
    #[serde(default)]
    pub root_parent_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_contains_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_frame_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_frame_children_contains_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_frame_instance_present: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_frame_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path: Vec<u64>,
    #[serde(default)]
    pub root_path_truncated: bool,
    /// For each `root_path` edge (`child -> parent`), whether `UiTree` currently has the
    /// corresponding `parent.children` edge:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing node entry)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path_edge_ui_contains_child: Vec<u8>,
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path_edge_frame_contains_child: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub removed_nodes: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_tail: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl UiRemovedSubtreeV1 {
    fn from_record(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime_state: Option<&ElementRuntime>,
        r: &fret_ui::tree::UiDebugRemoveSubtreeRecord,
        max_debug_string_bytes: usize,
    ) -> Self {
        let outcome = match r.outcome {
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::SkippedLayerRoot => "skipped_layer_root",
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::RootMissing => "root_missing",
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::Removed => "removed",
        };

        let mut root_element_path = r.root_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut root_element_path, max_debug_string_bytes);

        let mut root_parent_element_path = r.root_parent_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut root_parent_element_path, max_debug_string_bytes);

        let mut trigger_element_path = r.trigger_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut trigger_element_path, max_debug_string_bytes);

        let mut trigger_element_root_path = r.trigger_element_root.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut trigger_element_root_path, max_debug_string_bytes);

        let root_path = r.root_path[..(r.root_path_len as usize).min(r.root_path.len())].to_vec();
        let root_path_edge_len = (r.root_path_edge_len as usize)
            .min(r.root_path_edge_ui_contains_child.len())
            .min(r.root_path_edge_frame_contains_child.len());
        let root_path_edge_ui_contains_child =
            r.root_path_edge_ui_contains_child[..root_path_edge_len].to_vec();
        let root_path_edge_frame_contains_child =
            r.root_path_edge_frame_contains_child[..root_path_edge_len].to_vec();

        let (
            root_parent_children_last_set_location,
            root_parent_children_last_set_old_len,
            root_parent_children_last_set_new_len,
            root_parent_children_last_set_frame_id,
        ) = r
            .root_parent
            .and_then(|parent| ui.debug_set_children_write_for(parent))
            .map(|w| {
                let mut location = Some(format!("{}:{}:{}", w.file, w.line, w.column));
                truncate_opt_string_bytes(&mut location, max_debug_string_bytes);
                (
                    location,
                    Some(w.old_len),
                    Some(w.new_len),
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((None, None, None, None));

        let (
            root_root_parent_sever_parent,
            root_root_parent_sever_parent_element,
            root_root_parent_sever_parent_path,
            root_root_parent_sever_parent_is_view_cache_reuse_root,
            root_root_parent_sever_location,
            root_root_parent_sever_frame_id,
            root_root_parent_sever_parent_children_last_set_old_elements_head,
            root_root_parent_sever_parent_children_last_set_old_elements_head_paths,
            root_root_parent_sever_parent_children_last_set_new_elements_head,
            root_root_parent_sever_parent_children_last_set_new_elements_head_paths,
        ) = r
            .root_root
            .and_then(|root| ui.debug_parent_sever_write_for(root))
            .map(|w| {
                let parent_element = element_runtime_state
                    .and_then(|runtime| runtime.element_for_node(window, w.parent));
                let mut parent_path = parent_element.and_then(|element| {
                    element_runtime_state
                        .and_then(|runtime| runtime.debug_path_for_element(window, element))
                });
                truncate_opt_string_bytes(&mut parent_path, max_debug_string_bytes);
                let parent_is_view_cache_reuse_root = parent_element.and_then(|element| {
                    element_runtime_state.and_then(|runtime| {
                        runtime
                            .diagnostics_snapshot(window)
                            .map(|s| s.view_cache_reuse_roots.contains(&element))
                    })
                });

                let mut old_elements_head: Vec<u64> = Vec::new();
                let mut old_elements_head_paths: Vec<String> = Vec::new();
                let mut new_elements_head: Vec<u64> = Vec::new();
                let mut new_elements_head_paths: Vec<String> = Vec::new();

                if let Some(write) = ui.debug_set_children_write_for(w.parent) {
                    for element in write.old_elements_head.into_iter().flatten() {
                        old_elements_head.push(element.0);
                        if let Some(path) = element_runtime_state
                            .and_then(|runtime| runtime.debug_path_for_element(window, element))
                        {
                            old_elements_head_paths.push(path);
                        }
                    }
                    for element in write.new_elements_head.into_iter().flatten() {
                        new_elements_head.push(element.0);
                        if let Some(path) = element_runtime_state
                            .and_then(|runtime| runtime.debug_path_for_element(window, element))
                        {
                            new_elements_head_paths.push(path);
                        }
                    }
                }

                truncate_vec_string_bytes(&mut old_elements_head_paths, max_debug_string_bytes);
                truncate_vec_string_bytes(&mut new_elements_head_paths, max_debug_string_bytes);

                let mut location = Some(format!("{}:{}:{}", w.file, w.line, w.column));
                truncate_opt_string_bytes(&mut location, max_debug_string_bytes);

                (
                    Some(key_to_u64(w.parent)),
                    parent_element.map(|e| e.0),
                    parent_path,
                    parent_is_view_cache_reuse_root,
                    location,
                    Some(w.frame_id.0),
                    old_elements_head,
                    old_elements_head_paths,
                    new_elements_head,
                    new_elements_head_paths,
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));

        Self {
            root: key_to_u64(r.root),
            root_element: r.root_element.map(|e| e.0),
            root_parent_element: r.root_parent_element.map(|e| e.0),
            root_parent_element_path,
            root_element_path,
            root_parent: r.root_parent.map(key_to_u64),
            root_root: r.root_root.map(key_to_u64),
            root_root_parent_sever_parent,
            root_root_parent_sever_parent_element,
            root_root_parent_sever_parent_path,
            root_root_parent_sever_parent_is_view_cache_reuse_root,
            root_root_parent_sever_location,
            root_root_parent_sever_frame_id,
            root_root_parent_sever_parent_children_last_set_old_elements_head,
            root_root_parent_sever_parent_children_last_set_old_elements_head_paths,
            root_root_parent_sever_parent_children_last_set_new_elements_head,
            root_root_parent_sever_parent_children_last_set_new_elements_head_paths,
            root_layer: r.root_layer.map(|id| id.data().as_ffi()),
            root_layer_visible: r.root_layer_visible,
            reachable_from_layer_roots: r.reachable_from_layer_roots,
            reachable_from_view_cache_roots: r.reachable_from_view_cache_roots,
            unreachable_from_liveness_roots: r.unreachable_from_liveness_roots,
            liveness_layer_roots_len: r.liveness_layer_roots_len,
            view_cache_reuse_roots_len: r.view_cache_reuse_roots_len,
            view_cache_reuse_root_nodes_len: r.view_cache_reuse_root_nodes_len,
            trigger_element: r.trigger_element.map(|e| e.0),
            trigger_element_root: r.trigger_element_root.map(|e| e.0),
            trigger_element_in_view_cache_keep_alive: r.trigger_element_in_view_cache_keep_alive,
            trigger_element_listed_under_reuse_root: r
                .trigger_element_listed_under_reuse_root
                .map(|id| id.0),
            trigger_element_path,
            trigger_element_root_path,
            root_children_len: r.root_children_len,
            root_parent_children_len: r.root_parent_children_len,
            root_parent_children_contains_root: r.root_parent_children_contains_root,
            root_parent_frame_children_len: r.root_parent_frame_children_len,
            root_parent_frame_children_contains_root: r.root_parent_frame_children_contains_root,
            root_frame_instance_present: r.root_frame_instance_present,
            root_frame_children_len: r.root_frame_children_len,
            root_path,
            root_path_truncated: r.root_path_truncated,
            root_path_edge_ui_contains_child,
            root_path_edge_frame_contains_child,
            root_parent_children_last_set_location,
            root_parent_children_last_set_old_len,
            root_parent_children_last_set_new_len,
            root_parent_children_last_set_frame_id,
            removed_nodes: r.removed_nodes,
            removed_head: r.removed_head[..(r.removed_head_len as usize).min(r.removed_head.len())]
                .to_vec(),
            removed_tail: r.removed_tail[..(r.removed_tail_len as usize).min(r.removed_tail.len())]
                .to_vec(),
            outcome: Some(outcome.to_string()),
            frame_id: Some(r.frame_id.0),
            location: {
                let mut location = Some(format!("{}:{}:{}", r.file, r.line, r.column));
                truncate_opt_string_bytes(&mut location, max_debug_string_bytes);
                location
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDirtyViewV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

impl UiDirtyViewV1 {
    fn from_dirty_view(dirty: &fret_ui::tree::UiDebugDirtyView) -> Self {
        let source = match dirty.source {
            fret_ui::tree::UiDebugInvalidationSource::ModelChange => "model_change",
            fret_ui::tree::UiDebugInvalidationSource::GlobalChange => "global_change",
            fret_ui::tree::UiDebugInvalidationSource::Notify => "notify",
            fret_ui::tree::UiDebugInvalidationSource::Hover => "hover",
            fret_ui::tree::UiDebugInvalidationSource::Focus => "focus",
            fret_ui::tree::UiDebugInvalidationSource::Other => "other",
        };

        Self {
            root_node: key_to_u64(dirty.view.0),
            root_element: dirty.element.map(|e| e.0),
            source: Some(source.to_string()),
            detail: dirty.detail.as_str().map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNotifyRequestV1 {
    pub frame_id: u64,
    pub caller_node: u64,
    pub target_view: u64,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl UiNotifyRequestV1 {
    fn from_notify_request(req: &fret_ui::tree::UiDebugNotifyRequest) -> Self {
        Self {
            frame_id: req.frame_id.0,
            caller_node: key_to_u64(req.caller_node),
            target_view: key_to_u64(req.target_view.0),
            file: req.file.to_string(),
            line: req.line,
            column: req.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCacheRootStatsV1 {
    pub root: u64,
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub reused: bool,
    pub contained_layout: bool,
    #[serde(default)]
    pub contained_relayout_in_frame: bool,
    pub paint_replayed_ops: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_child_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes_truncated_at: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_in_semantics: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub reuse_reason: Option<String>,
}

impl UiCacheRootStatsV1 {
    fn from_stats(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        semantics: Option<&UiSemanticsSnapshotV1>,
        contained_relayout_roots: &HashSet<fret_core::NodeId>,
        stats: &fret_ui::tree::UiDebugCacheRootStats,
        max_debug_string_bytes: usize,
    ) -> Self {
        let element_path = stats.element.and_then(|id| {
            element_runtime.and_then(|runtime| runtime.debug_path_for_element(window, id))
        });

        let direct_child_nodes = ui.children(stats.root).len().min(u32::MAX as usize) as u32;

        // Keep bundles bounded: cache roots can cover large subtrees in real apps.
        const MAX_SUBTREE_NODES: usize = 50_000;
        let mut subtree_nodes_truncated_at: Option<u32> = None;
        let mut seen: HashSet<fret_core::NodeId> = HashSet::new();
        let mut stack: Vec<fret_core::NodeId> = vec![stats.root];
        while let Some(node) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }
            if seen.len() > MAX_SUBTREE_NODES {
                subtree_nodes_truncated_at = Some(MAX_SUBTREE_NODES as u32);
                break;
            }
            for child in ui.children(node) {
                stack.push(child);
            }
        }

        let root_in_semantics = semantics.map(|snap| {
            let id = stats.root.data().as_ffi();
            snap.nodes.iter().any(|n| n.id == id)
        });
        let contained_relayout_in_frame = contained_relayout_roots.contains(&stats.root);

        let (
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
        ) = ui
            .debug_set_children_write_for(stats.root)
            .map(|w| {
                let old_elements_head: Vec<_> =
                    w.old_elements_head.iter().flatten().copied().collect();
                let new_elements_head: Vec<_> =
                    w.new_elements_head.iter().flatten().copied().collect();

                let old_paths: Vec<String> = old_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();
                let new_paths: Vec<String> = new_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();

                (
                    Some(format!("{}:{}:{}", w.file, w.line, w.column)),
                    Some(w.old_len),
                    Some(w.new_len),
                    old_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    new_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    old_paths,
                    new_paths,
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
            ));

        let mut out = Self {
            root: stats.root.data().as_ffi(),
            element: stats.element.map(|id| id.0),
            element_path,
            reused: stats.reused,
            contained_layout: stats.contained_layout,
            contained_relayout_in_frame,
            paint_replayed_ops: stats.paint_replayed_ops,
            direct_child_nodes: Some(direct_child_nodes),
            subtree_nodes: Some(seen.len().min(u32::MAX as usize) as u32),
            subtree_nodes_truncated_at,
            root_in_semantics,
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
            reuse_reason: Some(stats.reuse_reason.as_str().to_string()),
        };

        truncate_opt_string_bytes(&mut out.element_path, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.children_last_set_location, max_debug_string_bytes);
        truncate_vec_string_bytes(
            &mut out.children_last_set_old_elements_head_paths,
            max_debug_string_bytes,
        );
        truncate_vec_string_bytes(
            &mut out.children_last_set_new_elements_head_paths,
            max_debug_string_bytes,
        );
        truncate_opt_string_bytes(&mut out.reuse_reason, max_debug_string_bytes);

        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineSolveV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_element_kind: Option<String>,
    #[serde(default)]
    pub root_element_path: Option<String>,
    pub solve_time_us: u64,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    #[serde(default)]
    pub measure_time_us: u64,
    #[serde(default)]
    pub top_measures: Vec<UiLayoutEngineMeasureHotspotV1>,
}

impl UiLayoutEngineSolveV1 {
    fn from_solve(s: &fret_ui::tree::UiDebugLayoutEngineSolve) -> Self {
        Self {
            root_node: s.root.data().as_ffi(),
            root_element: s.root_element.map(|id| id.0),
            root_element_kind: s.root_element_kind.map(|s| s.to_string()),
            root_element_path: s.root_element_path.clone(),
            solve_time_us: s.solve_time.as_micros().min(u64::MAX as u128) as u64,
            measure_calls: s.measure_calls,
            measure_cache_hits: s.measure_cache_hits,
            measure_time_us: s.measure_time.as_micros().min(u64::MAX as u128) as u64,
            top_measures: s
                .top_measures
                .iter()
                .map(|m| UiLayoutEngineMeasureHotspotV1 {
                    node: m.node.data().as_ffi(),
                    measure_time_us: m.measure_time.as_micros().min(u64::MAX as u128) as u64,
                    calls: m.calls,
                    cache_hits: m.cache_hits,
                    element: m.element.map(|id| id.0),
                    element_kind: m.element_kind.map(|s| s.to_string()),
                    top_children: m
                        .top_children
                        .iter()
                        .map(|c| UiLayoutEngineMeasureChildHotspotV1 {
                            child: c.child.data().as_ffi(),
                            measure_time_us: c.measure_time.as_micros().min(u64::MAX as u128)
                                as u64,
                            calls: c.calls,
                            element: c.element.map(|id| id.0),
                            element_kind: c.element_kind.map(|s| s.to_string()),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub widget_type: String,
    pub layout_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiLayoutHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugLayoutHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            element_path: h.element_path.clone(),
            widget_type: h.widget_type.to_string(),
            layout_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWidgetMeasureHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub widget_type: String,
    pub measure_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiWidgetMeasureHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugWidgetMeasureHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            element_path: h.element_path.clone(),
            widget_type: h.widget_type.to_string(),
            measure_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintWidgetHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub widget_type: String,
    pub paint_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
    #[serde(default)]
    pub inclusive_scene_ops_delta: u32,
    #[serde(default)]
    pub exclusive_scene_ops_delta: u32,
}

impl UiPaintWidgetHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintWidgetHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            widget_type: h.widget_type.to_string(),
            paint_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_scene_ops_delta: h.inclusive_scene_ops_delta,
            exclusive_scene_ops_delta: h.exclusive_scene_ops_delta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintTextPrepareHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub prepare_time_us: u64,
    pub text_len: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_factor: Option<f32>,
    #[serde(default)]
    pub reasons_mask: u16,
}

impl UiPaintTextPrepareHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintTextPrepareHotspot) -> Self {
        fn wrap_as_str(wrap: fret_core::TextWrap) -> &'static str {
            match wrap {
                fret_core::TextWrap::None => "none",
                fret_core::TextWrap::Word => "word",
                fret_core::TextWrap::WordBreak => "word_break",
                fret_core::TextWrap::Grapheme => "grapheme",
            }
        }

        fn overflow_as_str(overflow: fret_core::TextOverflow) -> &'static str {
            match overflow {
                fret_core::TextOverflow::Clip => "clip",
                fret_core::TextOverflow::Ellipsis => "ellipsis",
            }
        }

        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: Some(h.element_kind.to_string()),
            prepare_time_us: h.prepare_time.as_micros().min(u64::MAX as u128) as u64,
            text_len: h.text_len,
            max_width: h.constraints.max_width.map(|v| v.0),
            wrap: Some(wrap_as_str(h.constraints.wrap).to_string()),
            overflow: Some(overflow_as_str(h.constraints.overflow).to_string()),
            scale_factor: Some(h.constraints.scale_factor),
            reasons_mask: h.reasons_mask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureHotspotV1 {
    pub node: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    pub cache_hits: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    #[serde(default)]
    pub top_children: Vec<UiLayoutEngineMeasureChildHotspotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureChildHotspotV1 {
    pub child: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModelChangeHotspotV1 {
    pub model: u64,
    pub observation_edges: u32,
    #[serde(default)]
    pub changed_type: Option<String>,
    #[serde(default)]
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiModelChangeHotspotV1 {
    fn from_hotspot(hotspot: &fret_ui::tree::UiDebugModelChangeHotspot) -> Self {
        let changed_type = hotspot.changed.map(|c| c.type_name.to_string());
        let changed_at = hotspot.changed.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });
        Self {
            model: hotspot.model.data().as_ffi(),
            observation_edges: hotspot.observation_edges,
            changed_type,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSourceLocationV1 {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModelChangeUnobservedV1 {
    pub model: u64,
    pub created_type: Option<String>,
    pub created_at: Option<UiSourceLocationV1>,
    #[serde(default)]
    pub changed_type: Option<String>,
    #[serde(default)]
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiModelChangeUnobservedV1 {
    fn from_unobserved(unobserved: &fret_ui::tree::UiDebugModelChangeUnobserved) -> Self {
        let created_type = unobserved.created.map(|c| c.type_name.to_string());
        let created_at = unobserved.created.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });
        let changed_type = unobserved.changed.map(|c| c.type_name.to_string());
        let changed_at = unobserved.changed.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });

        Self {
            model: unobserved.model.data().as_ffi(),
            created_type,
            created_at,
            changed_type,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiGlobalChangeHotspotV1 {
    pub type_name: String,
    pub observation_edges: u32,
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiGlobalChangeHotspotV1 {
    fn from_hotspot(app: &App, hotspot: &fret_ui::tree::UiDebugGlobalChangeHotspot) -> Self {
        let type_name = app
            .global_type_name(hotspot.global)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:?}", hotspot.global));
        let changed_at = app
            .global_changed_at(hotspot.global)
            .map(|at| UiSourceLocationV1 {
                file: at.file().to_string(),
                line: at.line(),
                column: at.column(),
            });

        Self {
            type_name,
            observation_edges: hotspot.observation_edges,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiGlobalChangeUnobservedV1 {
    pub type_name: String,
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiGlobalChangeUnobservedV1 {
    fn from_unobserved(
        app: &App,
        unobserved: &fret_ui::tree::UiDebugGlobalChangeUnobserved,
    ) -> Self {
        let type_name = app
            .global_type_name(unobserved.global)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:?}", unobserved.global));
        let changed_at = app
            .global_changed_at(unobserved.global)
            .map(|at| UiSourceLocationV1 {
                file: at.file().to_string(),
                line: at.line(),
                column: at.column(),
            });

        Self {
            type_name,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidationKindV1 {
    Paint,
    Layout,
    HitTest,
    HitTestOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidationSourceV1 {
    ModelChange,
    GlobalChange,
    Hover,
    Focus,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInvalidationWalkV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_element_path: Option<String>,
    pub kind: UiInvalidationKindV1,
    pub source: UiInvalidationSourceV1,
    #[serde(default)]
    pub detail: Option<String>,
    pub walked_nodes: u32,
    #[serde(default)]
    pub truncated_at: Option<u64>,
}

impl UiInvalidationWalkV1 {
    fn from_walk(
        walk: &fret_ui::tree::UiDebugInvalidationWalk,
        window: AppWindowId,
        element_runtime_state: Option<&ElementRuntime>,
    ) -> Self {
        let kind = match walk.inv {
            Invalidation::Paint => UiInvalidationKindV1::Paint,
            Invalidation::Layout => UiInvalidationKindV1::Layout,
            Invalidation::HitTest => UiInvalidationKindV1::HitTest,
            Invalidation::HitTestOnly => UiInvalidationKindV1::HitTestOnly,
        };
        let source = match walk.source {
            fret_ui::tree::UiDebugInvalidationSource::ModelChange => {
                UiInvalidationSourceV1::ModelChange
            }
            fret_ui::tree::UiDebugInvalidationSource::GlobalChange => {
                UiInvalidationSourceV1::GlobalChange
            }
            fret_ui::tree::UiDebugInvalidationSource::Notify => UiInvalidationSourceV1::Other,
            fret_ui::tree::UiDebugInvalidationSource::Hover => UiInvalidationSourceV1::Hover,
            fret_ui::tree::UiDebugInvalidationSource::Focus => UiInvalidationSourceV1::Focus,
            fret_ui::tree::UiDebugInvalidationSource::Other => UiInvalidationSourceV1::Other,
        };
        let root_element_path = walk.root_element.and_then(|element| {
            element_runtime_state.and_then(|rt| rt.debug_path_for_element(window, element))
        });
        Self {
            root_node: key_to_u64(walk.root),
            root_element: walk.root_element.map(|e| e.0),
            root_element_path,
            kind,
            source,
            detail: walk.detail.as_str().map(|s| s.to_string()),
            walked_nodes: walk.walked_nodes,
            truncated_at: walk.truncated_at.map(key_to_u64),
        }
    }
}

// Semantics bundle types live in `ui_diagnostics/semantics.rs`.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFrameStatsV1 {
    #[serde(default)]
    pub frame_arena_capacity_estimate_bytes: u64,
    #[serde(default)]
    pub frame_arena_grow_events: u32,
    #[serde(default)]
    pub element_children_vec_pool_reuses: u32,
    #[serde(default)]
    pub element_children_vec_pool_misses: u32,
    /// UI thread CPU time spent since the previous snapshot (approx frame CPU time).
    ///
    /// This is intended to distinguish "real work" from schedule noise: if wall time spikes but
    /// CPU time does not, the thread likely wasn't running (preempted / ReadyThread / DPC/ISR).
    #[serde(default)]
    pub ui_thread_cpu_time_us: u64,
    /// Cumulative UI thread CPU time since process start (kernel + user).
    #[serde(default)]
    pub ui_thread_cpu_total_time_us: u64,
    /// UI thread CPU cycle time since the previous snapshot (high-resolution signal on Windows).
    ///
    /// Prefer this over `ui_thread_cpu_time_us` when doing per-frame triage: `GetThreadTimes`
    /// resolution can be too coarse on some systems.
    #[serde(default)]
    pub ui_thread_cpu_cycle_time_delta_cycles: u64,
    /// Cumulative UI thread CPU cycle time since process start.
    #[serde(default)]
    pub ui_thread_cpu_cycle_time_total_cycles: u64,
    pub layout_time_us: u64,
    #[serde(default)]
    pub layout_collect_roots_time_us: u64,
    #[serde(default)]
    pub layout_invalidate_scroll_handle_bindings_time_us: u64,
    #[serde(default)]
    pub layout_expand_view_cache_invalidations_time_us: u64,
    #[serde(default)]
    pub layout_request_build_roots_time_us: u64,
    #[serde(default)]
    pub layout_pending_barrier_relayouts_time_us: u64,
    #[serde(default)]
    pub layout_repair_view_cache_bounds_time_us: u64,
    #[serde(default)]
    pub layout_contained_view_cache_roots_time_us: u64,
    #[serde(default)]
    pub layout_collapse_layout_observations_time_us: u64,
    #[serde(default)]
    pub layout_observation_record_time_us: u64,
    #[serde(default)]
    pub layout_observation_record_models_items: u32,
    #[serde(default)]
    pub layout_observation_record_globals_items: u32,
    #[serde(default)]
    pub layout_prepaint_after_layout_time_us: u64,
    #[serde(default)]
    pub layout_skipped_engine_frame: bool,
    #[serde(default)]
    pub layout_roots_time_us: u64,
    #[serde(default)]
    pub layout_barrier_relayouts_time_us: u64,
    #[serde(default)]
    pub layout_view_cache_time_us: u64,
    #[serde(default)]
    pub layout_semantics_refresh_time_us: u64,
    #[serde(default)]
    pub layout_focus_repair_time_us: u64,
    #[serde(default)]
    pub layout_deferred_cleanup_time_us: u64,
    #[serde(default)]
    pub prepaint_time_us: u64,
    pub paint_time_us: u64,
    #[serde(default)]
    pub paint_record_visual_bounds_time_us: u64,
    #[serde(default)]
    pub paint_record_visual_bounds_calls: u32,
    #[serde(default)]
    pub paint_cache_key_time_us: u64,
    #[serde(default)]
    pub paint_cache_hit_check_time_us: u64,
    #[serde(default)]
    pub paint_widget_time_us: u64,
    #[serde(default)]
    pub paint_observation_record_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_models_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_models_items: u32,
    #[serde(default)]
    pub paint_host_widget_observed_globals_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_globals_items: u32,
    #[serde(default)]
    pub paint_host_widget_instance_lookup_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_instance_lookup_calls: u32,
    #[serde(default)]
    pub paint_text_prepare_time_us: u64,
    #[serde(default)]
    pub paint_text_prepare_calls: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_blob_missing: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_scale_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_text_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_rich_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_style_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_wrap_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_overflow_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_width_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_font_stack_changed: u32,
    #[serde(default)]
    pub paint_input_context_time_us: u64,
    #[serde(default)]
    pub paint_scroll_handle_invalidation_time_us: u64,
    #[serde(default)]
    pub paint_collect_roots_time_us: u64,
    #[serde(default)]
    pub paint_publish_text_input_snapshot_time_us: u64,
    #[serde(default)]
    pub paint_collapse_observations_time_us: u64,
    #[serde(default)]
    pub dispatch_time_us: u64,
    #[serde(default)]
    pub dispatch_pointer_events: u32,
    #[serde(default)]
    pub dispatch_pointer_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_events: u32,
    #[serde(default)]
    pub dispatch_timer_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_targeted_events: u32,
    #[serde(default)]
    pub dispatch_timer_targeted_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_events: u32,
    #[serde(default)]
    pub dispatch_timer_broadcast_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_layers_visited: u32,
    #[serde(default)]
    pub dispatch_timer_broadcast_rebuild_visible_layers_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_loop_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_slowest_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_slowest_token: Option<u64>,
    #[serde(default)]
    pub dispatch_timer_slowest_was_broadcast: bool,
    #[serde(default)]
    pub dispatch_other_events: u32,
    #[serde(default)]
    pub dispatch_other_event_time_us: u64,
    #[serde(default)]
    pub hit_test_time_us: u64,
    #[serde(default)]
    pub dispatch_events: u32,
    #[serde(default)]
    pub hit_test_queries: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_queries: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_disabled: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_misses: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_hits: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_candidate_rejected: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_nodes_visited: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_nodes_pushed: u32,
    #[serde(default)]
    pub hit_test_path_cache_hits: u32,
    #[serde(default)]
    pub hit_test_path_cache_misses: u32,
    #[serde(default)]
    pub hit_test_cached_path_time_us: u64,
    #[serde(default)]
    pub hit_test_bounds_tree_query_time_us: u64,
    #[serde(default)]
    pub hit_test_candidate_self_only_time_us: u64,
    #[serde(default)]
    pub hit_test_fallback_traversal_time_us: u64,
    #[serde(default)]
    pub dispatch_hover_update_time_us: u64,
    #[serde(default)]
    pub dispatch_scroll_handle_invalidation_time_us: u64,
    #[serde(default)]
    pub dispatch_active_layers_time_us: u64,
    #[serde(default)]
    pub dispatch_input_context_time_us: u64,
    #[serde(default)]
    pub dispatch_event_chain_build_time_us: u64,
    #[serde(default)]
    pub dispatch_widget_capture_time_us: u64,
    #[serde(default)]
    pub dispatch_widget_bubble_time_us: u64,
    #[serde(default)]
    pub dispatch_cursor_query_time_us: u64,
    #[serde(default)]
    pub dispatch_pointer_move_layer_observers_time_us: u64,
    #[serde(default)]
    pub dispatch_synth_hover_observer_time_us: u64,
    #[serde(default)]
    pub dispatch_cursor_effect_time_us: u64,
    #[serde(default)]
    pub dispatch_post_dispatch_snapshot_time_us: u64,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    #[serde(default)]
    pub prepaint_nodes_visited: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    #[serde(default)]
    pub paint_cache_replay_time_us: u64,
    #[serde(default)]
    pub paint_cache_bounds_translate_time_us: u64,
    #[serde(default)]
    pub paint_cache_bounds_translated_nodes: u32,
    #[serde(default)]
    pub interaction_cache_hits: u32,
    #[serde(default)]
    pub interaction_cache_misses: u32,
    #[serde(default)]
    pub interaction_cache_replayed_records: u32,
    #[serde(default)]
    pub interaction_records: u32,
    pub layout_engine_solves: u64,
    pub layout_engine_solve_time_us: u64,
    pub layout_engine_widget_fallback_solves: u64,
    #[serde(default)]
    pub layout_fast_path_taken: bool,
    #[serde(default)]
    pub layout_invalidations_count: u32,
    #[serde(default)]
    pub model_change_invalidation_roots: u32,
    #[serde(default)]
    pub model_change_models: u32,
    #[serde(default)]
    pub model_change_observation_edges: u32,
    #[serde(default)]
    pub model_change_unobserved_models: u32,
    #[serde(default)]
    pub global_change_invalidation_roots: u32,
    #[serde(default)]
    pub global_change_globals: u32,
    #[serde(default)]
    pub global_change_observation_edges: u32,
    #[serde(default)]
    pub global_change_unobserved_globals: u32,
    #[serde(default)]
    pub invalidation_walk_nodes: u32,
    #[serde(default)]
    pub invalidation_walk_calls: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_model_change: u32,
    #[serde(default)]
    pub invalidation_walk_calls_model_change: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_global_change: u32,
    #[serde(default)]
    pub invalidation_walk_calls_global_change: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_hover: u32,
    #[serde(default)]
    pub invalidation_walk_calls_hover: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_focus: u32,
    #[serde(default)]
    pub invalidation_walk_calls_focus: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_other: u32,
    #[serde(default)]
    pub invalidation_walk_calls_other: u32,
    #[serde(default)]
    pub hover_pressable_target_changes: u32,
    #[serde(default)]
    pub hover_hover_region_target_changes: u32,
    #[serde(default)]
    pub hover_declarative_instance_changes: u32,
    #[serde(default)]
    pub hover_declarative_hit_test_invalidations: u32,
    #[serde(default)]
    pub hover_declarative_layout_invalidations: u32,
    #[serde(default)]
    pub hover_declarative_paint_invalidations: u32,
    #[serde(default)]
    pub view_cache_active: bool,
    #[serde(default)]
    pub view_cache_invalidation_truncations: u32,
    #[serde(default)]
    pub view_cache_contained_relayouts: u32,
    #[serde(default)]
    pub view_cache_roots_total: u32,
    #[serde(default)]
    pub view_cache_roots_reused: u32,
    #[serde(default)]
    pub view_cache_roots_first_mount: u32,
    #[serde(default)]
    pub view_cache_roots_node_recreated: u32,
    #[serde(default)]
    pub view_cache_roots_cache_key_mismatch: u32,
    #[serde(default)]
    pub view_cache_roots_not_marked_reuse_root: u32,
    #[serde(default)]
    pub view_cache_roots_needs_rerender: u32,
    #[serde(default)]
    pub view_cache_roots_layout_invalidated: u32,
    #[serde(default)]
    pub view_cache_roots_manual: u32,
    #[serde(default)]
    pub set_children_barrier_writes: u32,
    #[serde(default)]
    pub barrier_relayouts_scheduled: u32,
    #[serde(default)]
    pub barrier_relayouts_performed: u32,
    #[serde(default)]
    pub virtual_list_visible_range_checks: u32,
    #[serde(default)]
    pub virtual_list_visible_range_refreshes: u32,
    #[serde(default)]
    pub virtual_list_window_shifts_total: u32,
    #[serde(default)]
    pub virtual_list_window_shifts_non_retained: u32,
    #[serde(default)]
    pub retained_virtual_list_reconciles: u32,
    #[serde(default)]
    pub retained_virtual_list_attached_items: u32,
    #[serde(default)]
    pub retained_virtual_list_detached_items: u32,
    pub focused_node: Option<u64>,
    pub captured_node: Option<u64>,

    // Renderer (wgpu) perf sample (best-effort; may be absent or lag a frame).
    #[serde(default)]
    pub renderer_tick_id: u64,
    #[serde(default)]
    pub renderer_frame_id: u64,
    #[serde(default)]
    pub renderer_frames: u64,
    #[serde(default)]
    pub renderer_encode_scene_us: u64,
    #[serde(default)]
    pub renderer_ensure_pipelines_us: u64,
    #[serde(default)]
    pub renderer_plan_compile_us: u64,
    #[serde(default)]
    pub renderer_upload_us: u64,
    #[serde(default)]
    pub renderer_record_passes_us: u64,
    #[serde(default)]
    pub renderer_encoder_finish_us: u64,
    #[serde(default)]
    pub renderer_prepare_svg_us: u64,
    #[serde(default)]
    pub renderer_prepare_text_us: u64,
    #[serde(default)]
    pub renderer_svg_uploads: u64,
    #[serde(default)]
    pub renderer_svg_upload_bytes: u64,
    #[serde(default)]
    pub renderer_image_uploads: u64,
    #[serde(default)]
    pub renderer_image_upload_bytes: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_unknown: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_owned: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_external_zero_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_gpu_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_cpu_upload: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_unknown: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_owned: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_external_zero_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_gpu_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_cpu_upload: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_fallbacks: u64,
    #[serde(default)]
    pub renderer_render_target_metadata_degradations_color_encoding_dropped: u64,
    #[serde(default)]
    pub renderer_svg_raster_budget_bytes: u64,
    #[serde(default)]
    pub renderer_svg_rasters_live: u64,
    #[serde(default)]
    pub renderer_svg_standalone_bytes_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_pages_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_bytes_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_used_px: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_capacity_px: u64,
    #[serde(default)]
    pub renderer_svg_raster_cache_hits: u64,
    #[serde(default)]
    pub renderer_svg_raster_cache_misses: u64,
    #[serde(default)]
    pub renderer_svg_raster_budget_evictions: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_page_evictions: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_entries_evicted: u64,
    #[serde(default)]
    pub renderer_text_atlas_revision: u64,
    #[serde(default)]
    pub renderer_text_atlas_uploads: u64,
    #[serde(default)]
    pub renderer_text_atlas_upload_bytes: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_glyphs: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_pages: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_page_glyphs: u64,
    #[serde(default)]
    pub renderer_text_atlas_resets: u64,
    #[serde(default)]
    pub renderer_intermediate_budget_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_in_use_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_peak_in_use_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_release_targets: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_allocations: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_reuses: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_releases: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_evictions: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_free_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_free_textures: u64,
    #[serde(default)]
    pub renderer_draw_calls: u64,
    #[serde(default)]
    pub renderer_text_draw_calls: u64,
    #[serde(default)]
    pub renderer_quad_draw_calls: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_unknown: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_owned: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_external_zero_copy: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_gpu_copy: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_cpu_upload: u64,
    #[serde(default)]
    pub renderer_mask_draw_calls: u64,
    #[serde(default)]
    pub renderer_pipeline_switches: u64,
    #[serde(default)]
    pub renderer_bind_group_switches: u64,
    #[serde(default)]
    pub renderer_scissor_sets: u64,
    #[serde(default)]
    pub renderer_uniform_bytes: u64,
    #[serde(default)]
    pub renderer_instance_bytes: u64,
    #[serde(default)]
    pub renderer_vertex_bytes: u64,
    #[serde(default)]
    pub renderer_scene_encoding_cache_hits: u64,
    #[serde(default)]
    pub renderer_scene_encoding_cache_misses: u64,
    #[serde(default)]
    pub renderer_material_quad_ops: u64,
    #[serde(default)]
    pub renderer_material_sampled_quad_ops: u64,
    #[serde(default)]
    pub renderer_material_distinct: u64,
    #[serde(default)]
    pub renderer_material_unknown_ids: u64,
    #[serde(default)]
    pub renderer_material_degraded_due_to_budget: u64,
}

impl UiFrameStatsV1 {
    fn from_stats(
        stats: UiDebugFrameStats,
        renderer_perf: Option<fret_render::RendererPerfFrameSample>,
    ) -> Self {
        let cpu = ui_thread_cpu_time::sample_current_thread(stats.frame_id.0);

        let mut out = Self {
            frame_arena_capacity_estimate_bytes: stats.frame_arena_capacity_estimate_bytes,
            frame_arena_grow_events: stats.frame_arena_grow_events,
            element_children_vec_pool_reuses: stats.element_children_vec_pool_reuses,
            element_children_vec_pool_misses: stats.element_children_vec_pool_misses,
            ui_thread_cpu_time_us: cpu.delta_time_us,
            ui_thread_cpu_total_time_us: cpu.total_time_us,
            ui_thread_cpu_cycle_time_delta_cycles: cpu.delta_cycles,
            ui_thread_cpu_cycle_time_total_cycles: cpu.total_cycles,
            layout_time_us: stats.layout_time.as_micros() as u64,
            layout_collect_roots_time_us: stats.layout_collect_roots_time.as_micros() as u64,
            layout_invalidate_scroll_handle_bindings_time_us: stats
                .layout_invalidate_scroll_handle_bindings_time
                .as_micros() as u64,
            layout_expand_view_cache_invalidations_time_us: stats
                .layout_expand_view_cache_invalidations_time
                .as_micros() as u64,
            layout_request_build_roots_time_us: stats.layout_request_build_roots_time.as_micros()
                as u64,
            layout_pending_barrier_relayouts_time_us: stats
                .layout_pending_barrier_relayouts_time
                .as_micros() as u64,
            layout_repair_view_cache_bounds_time_us: stats
                .layout_repair_view_cache_bounds_time
                .as_micros() as u64,
            layout_contained_view_cache_roots_time_us: stats
                .layout_contained_view_cache_roots_time
                .as_micros() as u64,
            layout_collapse_layout_observations_time_us: stats
                .layout_collapse_layout_observations_time
                .as_micros() as u64,
            layout_observation_record_time_us: stats.layout_observation_record_time.as_micros()
                as u64,
            layout_observation_record_models_items: stats.layout_observation_record_models_items,
            layout_observation_record_globals_items: stats.layout_observation_record_globals_items,
            layout_prepaint_after_layout_time_us: stats
                .layout_prepaint_after_layout_time
                .as_micros() as u64,
            layout_skipped_engine_frame: stats.layout_skipped_engine_frame,
            layout_roots_time_us: stats.layout_roots_time.as_micros() as u64,
            layout_barrier_relayouts_time_us: stats.layout_barrier_relayouts_time.as_micros()
                as u64,
            layout_view_cache_time_us: stats.layout_view_cache_time.as_micros() as u64,
            layout_semantics_refresh_time_us: stats.layout_semantics_refresh_time.as_micros()
                as u64,
            layout_focus_repair_time_us: stats.layout_focus_repair_time.as_micros() as u64,
            layout_deferred_cleanup_time_us: stats.layout_deferred_cleanup_time.as_micros() as u64,
            prepaint_time_us: stats.prepaint_time.as_micros() as u64,
            paint_time_us: stats.paint_time.as_micros() as u64,
            paint_record_visual_bounds_time_us: stats.paint_record_visual_bounds_time.as_micros()
                as u64,
            paint_record_visual_bounds_calls: stats.paint_record_visual_bounds_calls,
            paint_cache_key_time_us: stats.paint_cache_key_time.as_micros() as u64,
            paint_cache_hit_check_time_us: stats.paint_cache_hit_check_time.as_micros() as u64,
            paint_widget_time_us: stats.paint_widget_time.as_micros() as u64,
            paint_observation_record_time_us: stats.paint_observation_record_time.as_micros()
                as u64,
            paint_host_widget_observed_models_time_us: stats
                .paint_host_widget_observed_models_time
                .as_micros() as u64,
            paint_host_widget_observed_models_items: stats.paint_host_widget_observed_models_items,
            paint_host_widget_observed_globals_time_us: stats
                .paint_host_widget_observed_globals_time
                .as_micros() as u64,
            paint_host_widget_observed_globals_items: stats
                .paint_host_widget_observed_globals_items,
            paint_host_widget_instance_lookup_time_us: stats
                .paint_host_widget_instance_lookup_time
                .as_micros() as u64,
            paint_host_widget_instance_lookup_calls: stats.paint_host_widget_instance_lookup_calls,
            paint_text_prepare_time_us: stats.paint_text_prepare_time.as_micros() as u64,
            paint_text_prepare_calls: stats.paint_text_prepare_calls,
            paint_text_prepare_reason_blob_missing: stats.paint_text_prepare_reason_blob_missing,
            paint_text_prepare_reason_scale_changed: stats.paint_text_prepare_reason_scale_changed,
            paint_text_prepare_reason_text_changed: stats.paint_text_prepare_reason_text_changed,
            paint_text_prepare_reason_rich_changed: stats.paint_text_prepare_reason_rich_changed,
            paint_text_prepare_reason_style_changed: stats.paint_text_prepare_reason_style_changed,
            paint_text_prepare_reason_wrap_changed: stats.paint_text_prepare_reason_wrap_changed,
            paint_text_prepare_reason_overflow_changed: stats
                .paint_text_prepare_reason_overflow_changed,
            paint_text_prepare_reason_width_changed: stats.paint_text_prepare_reason_width_changed,
            paint_text_prepare_reason_font_stack_changed: stats
                .paint_text_prepare_reason_font_stack_changed,
            paint_input_context_time_us: stats.paint_input_context_time.as_micros() as u64,
            paint_scroll_handle_invalidation_time_us: stats
                .paint_scroll_handle_invalidation_time
                .as_micros() as u64,
            paint_collect_roots_time_us: stats.paint_collect_roots_time.as_micros() as u64,
            paint_publish_text_input_snapshot_time_us: stats
                .paint_publish_text_input_snapshot_time
                .as_micros() as u64,
            paint_collapse_observations_time_us: stats.paint_collapse_observations_time.as_micros()
                as u64,
            dispatch_time_us: stats.dispatch_time.as_micros() as u64,
            dispatch_pointer_events: stats.dispatch_pointer_events,
            dispatch_pointer_event_time_us: stats.dispatch_pointer_event_time.as_micros() as u64,
            dispatch_timer_events: stats.dispatch_timer_events,
            dispatch_timer_event_time_us: stats.dispatch_timer_event_time.as_micros() as u64,
            dispatch_timer_targeted_events: stats.dispatch_timer_targeted_events,
            dispatch_timer_targeted_time_us: stats.dispatch_timer_targeted_time.as_micros() as u64,
            dispatch_timer_broadcast_events: stats.dispatch_timer_broadcast_events,
            dispatch_timer_broadcast_time_us: stats.dispatch_timer_broadcast_time.as_micros()
                as u64,
            dispatch_timer_broadcast_layers_visited: stats.dispatch_timer_broadcast_layers_visited,
            dispatch_timer_broadcast_rebuild_visible_layers_time_us: stats
                .dispatch_timer_broadcast_rebuild_visible_layers_time
                .as_micros()
                as u64,
            dispatch_timer_broadcast_loop_time_us: stats
                .dispatch_timer_broadcast_loop_time
                .as_micros() as u64,
            dispatch_timer_slowest_event_time_us: stats
                .dispatch_timer_slowest_event_time
                .as_micros() as u64,
            dispatch_timer_slowest_token: stats.dispatch_timer_slowest_token.map(|t| t.0),
            dispatch_timer_slowest_was_broadcast: stats.dispatch_timer_slowest_was_broadcast,
            dispatch_other_events: stats.dispatch_other_events,
            dispatch_other_event_time_us: stats.dispatch_other_event_time.as_micros() as u64,
            hit_test_time_us: stats.hit_test_time.as_micros() as u64,
            dispatch_events: stats.dispatch_events,
            hit_test_queries: stats.hit_test_queries,
            hit_test_bounds_tree_queries: stats.hit_test_bounds_tree_queries,
            hit_test_bounds_tree_disabled: stats.hit_test_bounds_tree_disabled,
            hit_test_bounds_tree_misses: stats.hit_test_bounds_tree_misses,
            hit_test_bounds_tree_hits: stats.hit_test_bounds_tree_hits,
            hit_test_bounds_tree_candidate_rejected: stats.hit_test_bounds_tree_candidate_rejected,
            hit_test_bounds_tree_nodes_visited: stats.hit_test_bounds_tree_nodes_visited,
            hit_test_bounds_tree_nodes_pushed: stats.hit_test_bounds_tree_nodes_pushed,
            hit_test_path_cache_hits: stats.hit_test_path_cache_hits,
            hit_test_path_cache_misses: stats.hit_test_path_cache_misses,
            hit_test_cached_path_time_us: stats.hit_test_cached_path_time.as_micros() as u64,
            hit_test_bounds_tree_query_time_us: stats.hit_test_bounds_tree_query_time.as_micros()
                as u64,
            hit_test_candidate_self_only_time_us: stats
                .hit_test_candidate_self_only_time
                .as_micros() as u64,
            hit_test_fallback_traversal_time_us: stats.hit_test_fallback_traversal_time.as_micros()
                as u64,
            dispatch_hover_update_time_us: stats.dispatch_hover_update_time.as_micros() as u64,
            dispatch_scroll_handle_invalidation_time_us: stats
                .dispatch_scroll_handle_invalidation_time
                .as_micros() as u64,
            dispatch_active_layers_time_us: stats.dispatch_active_layers_time.as_micros() as u64,
            dispatch_input_context_time_us: stats.dispatch_input_context_time.as_micros() as u64,
            dispatch_event_chain_build_time_us: stats.dispatch_event_chain_build_time.as_micros()
                as u64,
            dispatch_widget_capture_time_us: stats.dispatch_widget_capture_time.as_micros() as u64,
            dispatch_widget_bubble_time_us: stats.dispatch_widget_bubble_time.as_micros() as u64,
            dispatch_cursor_query_time_us: stats.dispatch_cursor_query_time.as_micros() as u64,
            dispatch_pointer_move_layer_observers_time_us: stats
                .dispatch_pointer_move_layer_observers_time
                .as_micros() as u64,
            dispatch_synth_hover_observer_time_us: stats
                .dispatch_synth_hover_observer_time
                .as_micros() as u64,
            dispatch_cursor_effect_time_us: stats.dispatch_cursor_effect_time.as_micros() as u64,
            dispatch_post_dispatch_snapshot_time_us: stats
                .dispatch_post_dispatch_snapshot_time
                .as_micros() as u64,
            layout_nodes_visited: stats.layout_nodes_visited,
            layout_nodes_performed: stats.layout_nodes_performed,
            prepaint_nodes_visited: stats.prepaint_nodes_visited,
            paint_nodes: stats.paint_nodes,
            paint_nodes_performed: stats.paint_nodes_performed,
            paint_cache_hits: stats.paint_cache_hits,
            paint_cache_misses: stats.paint_cache_misses,
            paint_cache_replayed_ops: stats.paint_cache_replayed_ops,
            paint_cache_replay_time_us: stats.paint_cache_replay_time.as_micros() as u64,
            paint_cache_bounds_translate_time_us: stats
                .paint_cache_bounds_translate_time
                .as_micros() as u64,
            paint_cache_bounds_translated_nodes: stats.paint_cache_bounds_translated_nodes,
            interaction_cache_hits: stats.interaction_cache_hits,
            interaction_cache_misses: stats.interaction_cache_misses,
            interaction_cache_replayed_records: stats.interaction_cache_replayed_records,
            interaction_records: stats.interaction_records,
            layout_engine_solves: stats.layout_engine_solves,
            layout_engine_solve_time_us: stats.layout_engine_solve_time.as_micros() as u64,
            layout_engine_widget_fallback_solves: stats.layout_engine_widget_fallback_solves,
            layout_fast_path_taken: stats.layout_fast_path_taken,
            layout_invalidations_count: stats.layout_invalidations_count,
            model_change_invalidation_roots: stats.model_change_invalidation_roots,
            model_change_models: stats.model_change_models,
            model_change_observation_edges: stats.model_change_observation_edges,
            model_change_unobserved_models: stats.model_change_unobserved_models,
            global_change_invalidation_roots: stats.global_change_invalidation_roots,
            global_change_globals: stats.global_change_globals,
            global_change_observation_edges: stats.global_change_observation_edges,
            global_change_unobserved_globals: stats.global_change_unobserved_globals,
            invalidation_walk_nodes: stats.invalidation_walk_nodes,
            invalidation_walk_calls: stats.invalidation_walk_calls,
            invalidation_walk_nodes_model_change: stats.invalidation_walk_nodes_model_change,
            invalidation_walk_calls_model_change: stats.invalidation_walk_calls_model_change,
            invalidation_walk_nodes_global_change: stats.invalidation_walk_nodes_global_change,
            invalidation_walk_calls_global_change: stats.invalidation_walk_calls_global_change,
            invalidation_walk_nodes_hover: stats.invalidation_walk_nodes_hover,
            invalidation_walk_calls_hover: stats.invalidation_walk_calls_hover,
            invalidation_walk_nodes_focus: stats.invalidation_walk_nodes_focus,
            invalidation_walk_calls_focus: stats.invalidation_walk_calls_focus,
            invalidation_walk_nodes_other: stats.invalidation_walk_nodes_other,
            invalidation_walk_calls_other: stats.invalidation_walk_calls_other,
            hover_pressable_target_changes: stats.hover_pressable_target_changes,
            hover_hover_region_target_changes: stats.hover_hover_region_target_changes,
            hover_declarative_instance_changes: stats.hover_declarative_instance_changes,
            hover_declarative_hit_test_invalidations: stats
                .hover_declarative_hit_test_invalidations,
            hover_declarative_layout_invalidations: stats.hover_declarative_layout_invalidations,
            hover_declarative_paint_invalidations: stats.hover_declarative_paint_invalidations,
            view_cache_active: stats.view_cache_active,
            view_cache_invalidation_truncations: stats.view_cache_invalidation_truncations,
            view_cache_contained_relayouts: stats.view_cache_contained_relayouts,
            view_cache_roots_total: stats.view_cache_roots_total,
            view_cache_roots_reused: stats.view_cache_roots_reused,
            view_cache_roots_first_mount: stats.view_cache_roots_first_mount,
            view_cache_roots_node_recreated: stats.view_cache_roots_node_recreated,
            view_cache_roots_cache_key_mismatch: stats.view_cache_roots_cache_key_mismatch,
            view_cache_roots_not_marked_reuse_root: stats.view_cache_roots_not_marked_reuse_root,
            view_cache_roots_needs_rerender: stats.view_cache_roots_needs_rerender,
            view_cache_roots_layout_invalidated: stats.view_cache_roots_layout_invalidated,
            view_cache_roots_manual: stats.view_cache_roots_manual,
            set_children_barrier_writes: stats.set_children_barrier_writes,
            barrier_relayouts_scheduled: stats.barrier_relayouts_scheduled,
            barrier_relayouts_performed: stats.barrier_relayouts_performed,
            virtual_list_visible_range_checks: stats.virtual_list_visible_range_checks,
            virtual_list_visible_range_refreshes: stats.virtual_list_visible_range_refreshes,
            virtual_list_window_shifts_total: stats.virtual_list_window_shifts_total,
            virtual_list_window_shifts_non_retained: stats.virtual_list_window_shifts_non_retained,
            retained_virtual_list_reconciles: stats.retained_virtual_list_reconciles,
            retained_virtual_list_attached_items: stats.retained_virtual_list_attached_items,
            retained_virtual_list_detached_items: stats.retained_virtual_list_detached_items,
            focused_node: stats.focus.map(key_to_u64),
            captured_node: stats.captured.map(key_to_u64),
            renderer_tick_id: 0,
            renderer_frame_id: 0,
            renderer_frames: 0,
            renderer_encode_scene_us: 0,
            renderer_ensure_pipelines_us: 0,
            renderer_plan_compile_us: 0,
            renderer_upload_us: 0,
            renderer_record_passes_us: 0,
            renderer_encoder_finish_us: 0,
            renderer_prepare_svg_us: 0,
            renderer_prepare_text_us: 0,
            renderer_svg_uploads: 0,
            renderer_svg_upload_bytes: 0,
            renderer_image_uploads: 0,
            renderer_image_upload_bytes: 0,
            renderer_render_target_updates_ingest_unknown: 0,
            renderer_render_target_updates_ingest_owned: 0,
            renderer_render_target_updates_ingest_external_zero_copy: 0,
            renderer_render_target_updates_ingest_gpu_copy: 0,
            renderer_render_target_updates_ingest_cpu_upload: 0,
            renderer_render_target_updates_requested_ingest_unknown: 0,
            renderer_render_target_updates_requested_ingest_owned: 0,
            renderer_render_target_updates_requested_ingest_external_zero_copy: 0,
            renderer_render_target_updates_requested_ingest_gpu_copy: 0,
            renderer_render_target_updates_requested_ingest_cpu_upload: 0,
            renderer_render_target_updates_ingest_fallbacks: 0,
            renderer_render_target_metadata_degradations_color_encoding_dropped: 0,
            renderer_svg_raster_budget_bytes: 0,
            renderer_svg_rasters_live: 0,
            renderer_svg_standalone_bytes_live: 0,
            renderer_svg_mask_atlas_pages_live: 0,
            renderer_svg_mask_atlas_bytes_live: 0,
            renderer_svg_mask_atlas_used_px: 0,
            renderer_svg_mask_atlas_capacity_px: 0,
            renderer_svg_raster_cache_hits: 0,
            renderer_svg_raster_cache_misses: 0,
            renderer_svg_raster_budget_evictions: 0,
            renderer_svg_mask_atlas_page_evictions: 0,
            renderer_svg_mask_atlas_entries_evicted: 0,
            renderer_text_atlas_revision: 0,
            renderer_text_atlas_uploads: 0,
            renderer_text_atlas_upload_bytes: 0,
            renderer_text_atlas_evicted_glyphs: 0,
            renderer_text_atlas_evicted_pages: 0,
            renderer_text_atlas_evicted_page_glyphs: 0,
            renderer_text_atlas_resets: 0,
            renderer_intermediate_budget_bytes: 0,
            renderer_intermediate_in_use_bytes: 0,
            renderer_intermediate_peak_in_use_bytes: 0,
            renderer_intermediate_release_targets: 0,
            renderer_intermediate_pool_allocations: 0,
            renderer_intermediate_pool_reuses: 0,
            renderer_intermediate_pool_releases: 0,
            renderer_intermediate_pool_evictions: 0,
            renderer_intermediate_pool_free_bytes: 0,
            renderer_intermediate_pool_free_textures: 0,
            renderer_draw_calls: 0,
            renderer_text_draw_calls: 0,
            renderer_quad_draw_calls: 0,
            renderer_viewport_draw_calls: 0,
            renderer_viewport_draw_calls_ingest_unknown: 0,
            renderer_viewport_draw_calls_ingest_owned: 0,
            renderer_viewport_draw_calls_ingest_external_zero_copy: 0,
            renderer_viewport_draw_calls_ingest_gpu_copy: 0,
            renderer_viewport_draw_calls_ingest_cpu_upload: 0,
            renderer_mask_draw_calls: 0,
            renderer_pipeline_switches: 0,
            renderer_bind_group_switches: 0,
            renderer_scissor_sets: 0,
            renderer_uniform_bytes: 0,
            renderer_instance_bytes: 0,
            renderer_vertex_bytes: 0,
            renderer_scene_encoding_cache_hits: 0,
            renderer_scene_encoding_cache_misses: 0,
            renderer_material_quad_ops: 0,
            renderer_material_sampled_quad_ops: 0,
            renderer_material_distinct: 0,
            renderer_material_unknown_ids: 0,
            renderer_material_degraded_due_to_budget: 0,
        };

        if let Some(sample) = renderer_perf {
            out.renderer_tick_id = sample.tick_id;
            out.renderer_frame_id = sample.frame_id;
            out.renderer_frames = sample.perf.frames;
            out.renderer_encode_scene_us = sample.perf.encode_scene_us;
            out.renderer_ensure_pipelines_us = sample.perf.ensure_pipelines_us;
            out.renderer_plan_compile_us = sample.perf.plan_compile_us;
            out.renderer_upload_us = sample.perf.upload_us;
            out.renderer_record_passes_us = sample.perf.record_passes_us;
            out.renderer_encoder_finish_us = sample.perf.encoder_finish_us;
            out.renderer_prepare_svg_us = sample.perf.prepare_svg_us;
            out.renderer_prepare_text_us = sample.perf.prepare_text_us;
            out.renderer_svg_uploads = sample.perf.svg_uploads;
            out.renderer_svg_upload_bytes = sample.perf.svg_upload_bytes;
            out.renderer_image_uploads = sample.perf.image_uploads;
            out.renderer_image_upload_bytes = sample.perf.image_upload_bytes;
            out.renderer_render_target_updates_ingest_unknown =
                sample.perf.render_target_updates_ingest_unknown;
            out.renderer_render_target_updates_ingest_owned =
                sample.perf.render_target_updates_ingest_owned;
            out.renderer_render_target_updates_ingest_external_zero_copy =
                sample.perf.render_target_updates_ingest_external_zero_copy;
            out.renderer_render_target_updates_ingest_gpu_copy =
                sample.perf.render_target_updates_ingest_gpu_copy;
            out.renderer_render_target_updates_ingest_cpu_upload =
                sample.perf.render_target_updates_ingest_cpu_upload;
            out.renderer_render_target_updates_requested_ingest_unknown =
                sample.perf.render_target_updates_requested_ingest_unknown;
            out.renderer_render_target_updates_requested_ingest_owned =
                sample.perf.render_target_updates_requested_ingest_owned;
            out.renderer_render_target_updates_requested_ingest_external_zero_copy = sample
                .perf
                .render_target_updates_requested_ingest_external_zero_copy;
            out.renderer_render_target_updates_requested_ingest_gpu_copy =
                sample.perf.render_target_updates_requested_ingest_gpu_copy;
            out.renderer_render_target_updates_requested_ingest_cpu_upload = sample
                .perf
                .render_target_updates_requested_ingest_cpu_upload;
            out.renderer_render_target_updates_ingest_fallbacks =
                sample.perf.render_target_updates_ingest_fallbacks;
            out.renderer_render_target_metadata_degradations_color_encoding_dropped = sample
                .perf
                .render_target_metadata_degradations_color_encoding_dropped;
            out.renderer_svg_raster_budget_bytes = sample.perf.svg_raster_budget_bytes;
            out.renderer_svg_rasters_live = sample.perf.svg_rasters_live;
            out.renderer_svg_standalone_bytes_live = sample.perf.svg_standalone_bytes_live;
            out.renderer_svg_mask_atlas_pages_live = sample.perf.svg_mask_atlas_pages_live;
            out.renderer_svg_mask_atlas_bytes_live = sample.perf.svg_mask_atlas_bytes_live;
            out.renderer_svg_mask_atlas_used_px = sample.perf.svg_mask_atlas_used_px;
            out.renderer_svg_mask_atlas_capacity_px = sample.perf.svg_mask_atlas_capacity_px;
            out.renderer_svg_raster_cache_hits = sample.perf.svg_raster_cache_hits;
            out.renderer_svg_raster_cache_misses = sample.perf.svg_raster_cache_misses;
            out.renderer_svg_raster_budget_evictions = sample.perf.svg_raster_budget_evictions;
            out.renderer_svg_mask_atlas_page_evictions = sample.perf.svg_mask_atlas_page_evictions;
            out.renderer_svg_mask_atlas_entries_evicted =
                sample.perf.svg_mask_atlas_entries_evicted;
            out.renderer_text_atlas_revision = sample.perf.text_atlas_revision;
            out.renderer_text_atlas_uploads = sample.perf.text_atlas_uploads;
            out.renderer_text_atlas_upload_bytes = sample.perf.text_atlas_upload_bytes;
            out.renderer_text_atlas_evicted_glyphs = sample.perf.text_atlas_evicted_glyphs;
            out.renderer_text_atlas_evicted_pages = sample.perf.text_atlas_evicted_pages;
            out.renderer_text_atlas_evicted_page_glyphs =
                sample.perf.text_atlas_evicted_page_glyphs;
            out.renderer_text_atlas_resets = sample.perf.text_atlas_resets;
            out.renderer_intermediate_budget_bytes = sample.perf.intermediate_budget_bytes;
            out.renderer_intermediate_in_use_bytes = sample.perf.intermediate_in_use_bytes;
            out.renderer_intermediate_peak_in_use_bytes =
                sample.perf.intermediate_peak_in_use_bytes;
            out.renderer_intermediate_release_targets = sample.perf.intermediate_release_targets;
            out.renderer_intermediate_pool_allocations = sample.perf.intermediate_pool_allocations;
            out.renderer_intermediate_pool_reuses = sample.perf.intermediate_pool_reuses;
            out.renderer_intermediate_pool_releases = sample.perf.intermediate_pool_releases;
            out.renderer_intermediate_pool_evictions = sample.perf.intermediate_pool_evictions;
            out.renderer_intermediate_pool_free_bytes = sample.perf.intermediate_pool_free_bytes;
            out.renderer_intermediate_pool_free_textures =
                sample.perf.intermediate_pool_free_textures;
            out.renderer_draw_calls = sample.perf.draw_calls;
            out.renderer_text_draw_calls = sample.perf.text_draw_calls;
            out.renderer_quad_draw_calls = sample.perf.quad_draw_calls;
            out.renderer_viewport_draw_calls = sample.perf.viewport_draw_calls;
            out.renderer_viewport_draw_calls_ingest_unknown =
                sample.perf.viewport_draw_calls_ingest_unknown;
            out.renderer_viewport_draw_calls_ingest_owned =
                sample.perf.viewport_draw_calls_ingest_owned;
            out.renderer_viewport_draw_calls_ingest_external_zero_copy =
                sample.perf.viewport_draw_calls_ingest_external_zero_copy;
            out.renderer_viewport_draw_calls_ingest_gpu_copy =
                sample.perf.viewport_draw_calls_ingest_gpu_copy;
            out.renderer_viewport_draw_calls_ingest_cpu_upload =
                sample.perf.viewport_draw_calls_ingest_cpu_upload;
            out.renderer_mask_draw_calls = sample.perf.mask_draw_calls;
            out.renderer_pipeline_switches = sample.perf.pipeline_switches;
            out.renderer_bind_group_switches = sample.perf.bind_group_switches;
            out.renderer_scissor_sets = sample.perf.scissor_sets;
            out.renderer_uniform_bytes = sample.perf.uniform_bytes;
            out.renderer_instance_bytes = sample.perf.instance_bytes;
            out.renderer_vertex_bytes = sample.perf.vertex_bytes;
            out.renderer_scene_encoding_cache_hits = sample.perf.scene_encoding_cache_hits;
            out.renderer_scene_encoding_cache_misses = sample.perf.scene_encoding_cache_misses;
            out.renderer_material_quad_ops = sample.perf.material_quad_ops;
            out.renderer_material_sampled_quad_ops = sample.perf.material_sampled_quad_ops;
            out.renderer_material_distinct = sample.perf.material_distinct;
            out.renderer_material_unknown_ids = sample.perf.material_unknown_ids;
            out.renderer_material_degraded_due_to_budget =
                sample.perf.material_degraded_due_to_budget;
        }

        out
    }
}

mod ui_thread_cpu_time {
    #[cfg(windows)]
    use std::cell::Cell;

    #[cfg(windows)]
    use windows_sys::Win32::Foundation::FILETIME;
    #[cfg(windows)]
    use windows_sys::Win32::Foundation::HANDLE;
    #[cfg(windows)]
    use windows_sys::Win32::System::Threading::{GetCurrentThread, GetThreadTimes};
    #[cfg(windows)]
    use windows_sys::core::BOOL;

    #[cfg(windows)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn QueryThreadCycleTime(thread: HANDLE, cycle_time: *mut u64) -> BOOL;
    }

    #[cfg(windows)]
    thread_local! {
        static LAST_THREAD_CPU_100NS: Cell<Option<u64>> = const { Cell::new(None) };
        static LAST_THREAD_CYCLES: Cell<Option<u64>> = const { Cell::new(None) };
        static CACHED_FRAME_ID: Cell<Option<u64>> = const { Cell::new(None) };
        static CACHED_DELTA_TIME_US: Cell<u64> = const { Cell::new(0) };
        static CACHED_TOTAL_TIME_US: Cell<u64> = const { Cell::new(0) };
        static CACHED_DELTA_CYCLES: Cell<u64> = const { Cell::new(0) };
        static CACHED_TOTAL_CYCLES: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) struct UiThreadCpuSample {
        pub(super) delta_time_us: u64,
        pub(super) total_time_us: u64,
        pub(super) delta_cycles: u64,
        pub(super) total_cycles: u64,
    }

    pub(super) fn reset() {
        #[cfg(windows)]
        {
            LAST_THREAD_CPU_100NS.with(|slot| slot.set(None));
            LAST_THREAD_CYCLES.with(|slot| slot.set(None));
            CACHED_FRAME_ID.with(|slot| slot.set(None));
            CACHED_DELTA_TIME_US.with(|slot| slot.set(0));
            CACHED_TOTAL_TIME_US.with(|slot| slot.set(0));
            CACHED_DELTA_CYCLES.with(|slot| slot.set(0));
            CACHED_TOTAL_CYCLES.with(|slot| slot.set(0));
        }
    }

    pub(super) fn sample_current_thread(frame_id: u64) -> UiThreadCpuSample {
        #[cfg(windows)]
        {
            if CACHED_FRAME_ID.with(|slot| slot.get()) == Some(frame_id) {
                return UiThreadCpuSample {
                    delta_time_us: CACHED_DELTA_TIME_US.with(|slot| slot.get()),
                    total_time_us: CACHED_TOTAL_TIME_US.with(|slot| slot.get()),
                    delta_cycles: CACHED_DELTA_CYCLES.with(|slot| slot.get()),
                    total_cycles: CACHED_TOTAL_CYCLES.with(|slot| slot.get()),
                };
            }

            fn filetime_to_100ns(ft: FILETIME) -> u64 {
                ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
            }

            let mut creation: FILETIME = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut exit: FILETIME = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut kernel: FILETIME = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut user: FILETIME = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };

            let ok = unsafe {
                GetThreadTimes(
                    GetCurrentThread(),
                    &mut creation,
                    &mut exit,
                    &mut kernel,
                    &mut user,
                )
            };
            if ok == 0 {
                return UiThreadCpuSample {
                    delta_time_us: 0,
                    total_time_us: 0,
                    delta_cycles: 0,
                    total_cycles: 0,
                };
            }

            let total_100ns = filetime_to_100ns(kernel).saturating_add(filetime_to_100ns(user));
            let total_us = total_100ns / 10;
            let delta_time_us = LAST_THREAD_CPU_100NS.with(|slot| {
                let prev = slot.get();
                slot.set(Some(total_100ns));
                prev.map_or(0, |prev| total_100ns.saturating_sub(prev) / 10)
            });

            let mut total_cycles: u64 = 0;
            let ok_cycles = unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut total_cycles) };
            let delta_cycles = if ok_cycles == 0 {
                0
            } else {
                LAST_THREAD_CYCLES.with(|slot| {
                    let prev = slot.get();
                    slot.set(Some(total_cycles));
                    prev.map_or(0, |prev| total_cycles.saturating_sub(prev))
                })
            };

            CACHED_FRAME_ID.with(|slot| slot.set(Some(frame_id)));
            CACHED_DELTA_TIME_US.with(|slot| slot.set(delta_time_us));
            CACHED_TOTAL_TIME_US.with(|slot| slot.set(total_us));
            CACHED_DELTA_CYCLES.with(|slot| slot.set(delta_cycles));
            CACHED_TOTAL_CYCLES.with(|slot| slot.set(total_cycles));
            return UiThreadCpuSample {
                delta_time_us,
                total_time_us: total_us,
                delta_cycles,
                total_cycles,
            };
        }

        #[cfg(not(windows))]
        {
            let _ = frame_id;
            UiThreadCpuSample {
                delta_time_us: 0,
                total_time_us: 0,
                delta_cycles: 0,
                total_cycles: 0,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayerInfoV1 {
    pub id: String,
    /// Numeric layer id (stable across `Debug` formatting changes; not stable between runs).
    #[serde(default)]
    pub layer_id: u64,
    pub root: u64,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    /// Pointer occlusion mode for this layer root (when applicable).
    #[serde(default)]
    pub pointer_occlusion: String,
    pub wants_pointer_down_outside_events: bool,
    #[serde(default)]
    pub consume_pointer_down_outside_events: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pointer_down_outside_branches: Vec<u64>,
    pub wants_pointer_move_events: bool,
    pub wants_timer_events: bool,
}

impl UiLayerInfoV1 {
    fn from_layer(layer: UiDebugLayerInfo) -> Self {
        Self {
            id: format!("{:?}", layer.id),
            layer_id: layer.id.data().as_ffi(),
            root: key_to_u64(layer.root),
            visible: layer.visible,
            blocks_underlay_input: layer.blocks_underlay_input,
            hit_testable: layer.hit_testable,
            pointer_occlusion: pointer_occlusion_label(layer.pointer_occlusion),
            wants_pointer_down_outside_events: layer.wants_pointer_down_outside_events,
            consume_pointer_down_outside_events: layer.consume_pointer_down_outside_events,
            pointer_down_outside_branches: layer
                .pointer_down_outside_branches
                .into_iter()
                .take(32)
                .map(key_to_u64)
                .collect(),
            wants_pointer_move_events: layer.wants_pointer_move_events,
            wants_timer_events: layer.wants_timer_events,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayerVisibleWriteV1 {
    pub layer: String,
    pub prev_visible: Option<bool>,
    pub visible: bool,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl UiLayerVisibleWriteV1 {
    fn from_write(write: &fret_ui::tree::UiDebugSetLayerVisibleWrite) -> Self {
        Self {
            layer: format!("{:?}", write.layer),
            prev_visible: write.prev_visible,
            visible: write.visible,
            file: write.file.to_string(),
            line: write.line,
            column: write.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiOverlayPolicyDecisionV1 {
    pub layer: String,
    pub kind: String,
    pub present: bool,
    pub interactive: bool,
    pub wants_timer_events: bool,
    pub reason: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u32,
    #[serde(default)]
    pub column: u32,
}

impl UiOverlayPolicyDecisionV1 {
    fn from_decision(d: &fret_ui::tree::UiDebugOverlayPolicyDecisionWrite) -> Self {
        Self {
            layer: format!("{:?}", d.layer),
            kind: d.kind.to_string(),
            present: d.present,
            interactive: d.interactive,
            wants_timer_events: d.wants_timer_events,
            reason: d.reason.to_string(),
            file: d.file.to_string(),
            line: d.line,
            column: d.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestSnapshotV1 {
    pub position: PointV1,
    pub hit: Option<u64>,
    pub active_layer_roots: Vec<u64>,
    pub barrier_root: Option<u64>,
    #[serde(default)]
    pub focus_barrier_root: Option<u64>,
    /// Stable, script-friendly labels for each scope root.
    ///
    /// Prefer this over `active_layer_roots` when validating behavior across refactors, since node
    /// ids are not stable between runs.
    #[serde(default)]
    pub scope_roots: Vec<UiHitTestScopeRootV1>,
}

impl UiHitTestSnapshotV1 {
    fn from_tree(position: Point, ui: &mut UiTree<App>) -> Self {
        let hit_test = ui.debug_hit_test_routing(position);
        let arbitration = ui.input_arbitration_snapshot();
        let layers = ui.debug_layers_in_paint_order();
        Self::from_hit_test_with_layers(position, hit_test, arbitration.focus_barrier_root, &layers)
    }

    fn from_hit_test_with_layers(
        position: Point,
        hit_test: UiDebugHitTest,
        focus_barrier_root: Option<NodeId>,
        layers: &[UiDebugLayerInfo],
    ) -> Self {
        let mut scope_roots = Vec::new();
        if let Some(root) = hit_test.barrier_root {
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "modal_barrier_root".to_string(),
                root: key_to_u64(root),
                layer_id: None,
                pointer_occlusion: None,
                blocks_underlay_input: None,
                hit_testable: None,
            });
        }

        let mut by_root: HashMap<NodeId, &UiDebugLayerInfo> = HashMap::new();
        for layer in layers {
            by_root.insert(layer.root, layer);
        }

        if let Some(root) = focus_barrier_root {
            let info = by_root.get(&root);
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "focus_barrier_root".to_string(),
                root: key_to_u64(root),
                layer_id: info.map(|l| l.id.data().as_ffi()),
                pointer_occlusion: info.map(|l| pointer_occlusion_label(l.pointer_occlusion)),
                blocks_underlay_input: info.map(|l| l.blocks_underlay_input),
                hit_testable: info.map(|l| l.hit_testable),
            });
        }

        for root in &hit_test.active_layer_roots {
            let info = by_root.get(root);
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "layer_root".to_string(),
                root: key_to_u64(*root),
                layer_id: info.map(|l| l.id.data().as_ffi()),
                pointer_occlusion: info.map(|l| pointer_occlusion_label(l.pointer_occlusion)),
                blocks_underlay_input: info.map(|l| l.blocks_underlay_input),
                hit_testable: info.map(|l| l.hit_testable),
            });
        }

        Self {
            position: PointV1::from(position),
            hit: hit_test.hit.map(key_to_u64),
            active_layer_roots: hit_test
                .active_layer_roots
                .into_iter()
                .map(key_to_u64)
                .collect(),
            barrier_root: hit_test.barrier_root.map(key_to_u64),
            focus_barrier_root: focus_barrier_root.map(key_to_u64),
            scope_roots,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestScopeRootV1 {
    /// Stable scope root kind (e.g. `modal_barrier_root`, `layer_root`).
    pub kind: String,
    /// Node id of the root (not stable between runs; treat as an in-run reference only).
    pub root: u64,
    /// When `kind=layer_root`, the corresponding layer id (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<u64>,
    /// Pointer occlusion mode for the layer root (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_underlay_input: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_testable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDiagnosticsSnapshotV1 {
    pub focused_element: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_path: Option<String>,
    pub focused_element_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_visual_bounds: Option<RectV1>,
    pub active_text_selection: Option<(u64, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_text_selection_path: Option<(String, String)>,
    pub hovered_pressable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_path: Option<String>,
    pub hovered_pressable_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_visual_bounds: Option<RectV1>,
    pub pressed_pressable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_path: Option<String>,
    pub pressed_pressable_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_visual_bounds: Option<RectV1>,
    pub hovered_hover_region: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_hover_region_path: Option<String>,
    pub wants_continuous_frames: bool,
    pub observed_models: Vec<ElementObservedModelsV1>,
    pub observed_globals: Vec<ElementObservedGlobalsV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_layout_queries: Vec<ElementObservedLayoutQueriesV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_query_regions: Vec<ElementLayoutQueryRegionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ElementEnvironmentSnapshotV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_environment: Vec<ElementObservedEnvironmentV1>,
    #[serde(default)]
    pub view_cache_reuse_roots: Vec<u64>,
    #[serde(default)]
    pub view_cache_reuse_root_element_counts: Vec<(u64, u32)>,
    #[serde(default)]
    pub view_cache_reuse_root_element_samples: Vec<ElementViewCacheReuseRootElementsSampleV1>,
    #[serde(default)]
    pub retained_keep_alive_roots_len: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_keep_alive_roots_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_keep_alive_roots_tail: Vec<u64>,
    #[serde(default)]
    pub node_entry_root_overwrites: Vec<ElementNodeEntryRootOverwriteV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementViewCacheReuseRootElementsSampleV1 {
    pub root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<u64>,
    pub elements_len: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements_tail: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementNodeEntryRootOverwriteV1 {
    pub frame_id: u64,
    pub element: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub old_root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_root_path: Option<String>,
    pub new_root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_root_path: Option<String>,
    pub old_node: u64,
    pub new_node: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<UiSourceLocationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedModelsV1 {
    pub element: u64,
    pub models: Vec<(u64, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedGlobalsV1 {
    pub element: u64,
    pub globals: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedLayoutQueriesV1 {
    pub element: u64,
    pub deps_fingerprint: u64,
    pub regions: Vec<ElementObservedLayoutQueryRegionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedLayoutQueryRegionV1 {
    pub region: u64,
    pub invalidation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_name: Option<String>,
    pub region_revision: u64,
    pub region_changed_this_frame: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_committed_bounds: Option<RectV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLayoutQueryRegionV1 {
    pub region: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub revision: u64,
    pub changed_this_frame: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_bounds: Option<RectV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementEnvironmentSnapshotV1 {
    pub viewport_bounds: RectV1,
    pub scale_factor: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_scale_factor: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_transparency: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<fret_core::Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contrast_preference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forced_colors_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_pointer_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area_insets: Option<UiEdgesV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_insets: Option<UiEdgesV1>,
}

impl ElementEnvironmentSnapshotV1 {
    fn from_diagnostics_snapshot(
        snapshot: &fret_ui::elements::EnvironmentQueryDiagnosticsSnapshot,
    ) -> Self {
        let edges_to_protocol = |value: fret_core::Edges| UiEdgesV1 {
            top_px: value.top.0,
            right_px: value.right.0,
            bottom_px: value.bottom.0,
            left_px: value.left.0,
        };
        Self {
            viewport_bounds: RectV1::from(snapshot.viewport_bounds),
            scale_factor: snapshot.scale_factor,
            color_scheme: snapshot
                .color_scheme
                .map(|s| color_scheme_label(s).to_string()),
            prefers_reduced_motion: snapshot.prefers_reduced_motion,
            text_scale_factor: snapshot.text_scale_factor,
            prefers_reduced_transparency: snapshot.prefers_reduced_transparency,
            accent_color: snapshot.accent_color,
            contrast_preference: snapshot
                .contrast_preference
                .map(|c| contrast_preference_label(c).to_string()),
            forced_colors_mode: snapshot
                .forced_colors_mode
                .map(|m| forced_colors_mode_label(m).to_string()),
            primary_pointer_type: Some(
                viewport_pointer_type_label(snapshot.primary_pointer_type).to_string(),
            ),
            safe_area_insets: snapshot.safe_area_insets.map(edges_to_protocol),
            occlusion_insets: snapshot.occlusion_insets.map(edges_to_protocol),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedEnvironmentV1 {
    pub element: u64,
    pub deps_fingerprint: u64,
    pub keys: Vec<ElementObservedEnvironmentKeyV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedEnvironmentKeyV1 {
    pub key: String,
    pub invalidation: String,
    pub key_revision: u64,
    pub key_changed_this_frame: bool,
}

impl ElementDiagnosticsSnapshotV1 {
    fn from_runtime(
        window: AppWindowId,
        runtime: &ElementRuntime,
        snapshot: fret_ui::elements::WindowElementDiagnosticsSnapshot,
        max_debug_string_bytes: usize,
    ) -> Self {
        let mut focused_element_path = snapshot
            .focused_element
            .and_then(|id| runtime.debug_path_for_element(window, id));
        truncate_opt_string_bytes(&mut focused_element_path, max_debug_string_bytes);

        let mut active_text_selection_path = snapshot.active_text_selection.and_then(|(a, b)| {
            let a = runtime.debug_path_for_element(window, a)?;
            let b = runtime.debug_path_for_element(window, b)?;
            Some((a, b))
        });
        if let Some((a, b)) = active_text_selection_path.as_mut() {
            truncate_string_bytes(a, max_debug_string_bytes);
            truncate_string_bytes(b, max_debug_string_bytes);
        }

        let mut hovered_pressable_path = snapshot
            .hovered_pressable
            .and_then(|id| runtime.debug_path_for_element(window, id));
        truncate_opt_string_bytes(&mut hovered_pressable_path, max_debug_string_bytes);

        let mut pressed_pressable_path = snapshot
            .pressed_pressable
            .and_then(|id| runtime.debug_path_for_element(window, id));
        truncate_opt_string_bytes(&mut pressed_pressable_path, max_debug_string_bytes);

        let mut hovered_hover_region_path = snapshot
            .hovered_hover_region
            .and_then(|id| runtime.debug_path_for_element(window, id));
        truncate_opt_string_bytes(&mut hovered_hover_region_path, max_debug_string_bytes);

        let mut out = Self {
            focused_element: snapshot.focused_element.map(|id| id.0),
            focused_element_path,
            focused_element_node: snapshot.focused_element_node.map(key_to_u64),
            focused_element_bounds: snapshot.focused_element_bounds.map(RectV1::from),
            focused_element_visual_bounds: snapshot.focused_element_visual_bounds.map(RectV1::from),
            active_text_selection: snapshot.active_text_selection.map(|(a, b)| (a.0, b.0)),
            active_text_selection_path,
            hovered_pressable: snapshot.hovered_pressable.map(|id| id.0),
            hovered_pressable_path,
            hovered_pressable_node: snapshot.hovered_pressable_node.map(key_to_u64),
            hovered_pressable_bounds: snapshot.hovered_pressable_bounds.map(RectV1::from),
            hovered_pressable_visual_bounds: snapshot
                .hovered_pressable_visual_bounds
                .map(RectV1::from),
            pressed_pressable: snapshot.pressed_pressable.map(|id| id.0),
            pressed_pressable_path,
            pressed_pressable_node: snapshot.pressed_pressable_node.map(key_to_u64),
            pressed_pressable_bounds: snapshot.pressed_pressable_bounds.map(RectV1::from),
            pressed_pressable_visual_bounds: snapshot
                .pressed_pressable_visual_bounds
                .map(RectV1::from),
            hovered_hover_region: snapshot.hovered_hover_region.map(|id| id.0),
            hovered_hover_region_path,
            wants_continuous_frames: snapshot.wants_continuous_frames,
            observed_models: snapshot
                .observed_models
                .into_iter()
                .map(|(element, list)| ElementObservedModelsV1 {
                    element: element.0,
                    models: list
                        .into_iter()
                        .map(|(id, inv)| (id, invalidation_label(inv).to_string()))
                        .collect(),
                })
                .collect(),
            observed_globals: snapshot
                .observed_globals
                .into_iter()
                .map(|(element, list)| ElementObservedGlobalsV1 {
                    element: element.0,
                    globals: list
                        .into_iter()
                        .map(|(id, inv)| (id, invalidation_label(inv).to_string()))
                        .collect(),
                })
                .collect(),
            observed_layout_queries: snapshot
                .observed_layout_queries
                .into_iter()
                .map(|entry| ElementObservedLayoutQueriesV1 {
                    element: entry.element.0,
                    deps_fingerprint: entry.deps_fingerprint,
                    regions: entry
                        .regions
                        .into_iter()
                        .map(|r| ElementObservedLayoutQueryRegionV1 {
                            region: r.region.0,
                            invalidation: invalidation_label(r.invalidation).to_string(),
                            region_name: r.region_name.map(|name| name.to_string()),
                            region_revision: r.region_revision,
                            region_changed_this_frame: r.region_changed_this_frame,
                            region_committed_bounds: r.region_committed_bounds.map(RectV1::from),
                        })
                        .collect(),
                })
                .collect(),
            layout_query_regions: snapshot
                .layout_query_regions
                .into_iter()
                .map(|r| ElementLayoutQueryRegionV1 {
                    region: r.region.0,
                    name: r.name.map(|name| name.to_string()),
                    revision: r.revision,
                    changed_this_frame: r.changed_this_frame,
                    committed_bounds: r.committed_bounds.map(RectV1::from),
                    current_bounds: r.current_bounds.map(RectV1::from),
                })
                .collect(),
            environment: Some(ElementEnvironmentSnapshotV1::from_diagnostics_snapshot(
                &snapshot.environment,
            )),
            observed_environment: snapshot
                .observed_environment
                .into_iter()
                .map(|entry| ElementObservedEnvironmentV1 {
                    element: entry.element.0,
                    deps_fingerprint: entry.deps_fingerprint,
                    keys: entry
                        .keys
                        .into_iter()
                        .map(|k| ElementObservedEnvironmentKeyV1 {
                            key: k.key.to_string(),
                            invalidation: invalidation_label(k.invalidation).to_string(),
                            key_revision: k.key_revision,
                            key_changed_this_frame: k.key_changed_this_frame,
                        })
                        .collect(),
                })
                .collect(),
            view_cache_reuse_roots: snapshot
                .view_cache_reuse_roots
                .into_iter()
                .map(|id| id.0)
                .collect(),
            view_cache_reuse_root_element_counts: snapshot
                .view_cache_reuse_root_element_counts
                .into_iter()
                .map(|(id, count)| (id.0, count))
                .collect(),
            view_cache_reuse_root_element_samples: snapshot
                .view_cache_reuse_root_element_samples
                .into_iter()
                .map(|s| ElementViewCacheReuseRootElementsSampleV1 {
                    root: s.root.0,
                    node: s.node.map(|n| n.data().as_ffi()),
                    elements_len: s.elements_len,
                    elements_head: s.elements_head.into_iter().map(|id| id.0).collect(),
                    elements_tail: s.elements_tail.into_iter().map(|id| id.0).collect(),
                })
                .collect(),
            retained_keep_alive_roots_len: snapshot.retained_keep_alive_roots_len,
            retained_keep_alive_roots_head: snapshot
                .retained_keep_alive_roots_head
                .into_iter()
                .map(|n| n.data().as_ffi())
                .collect(),
            retained_keep_alive_roots_tail: snapshot
                .retained_keep_alive_roots_tail
                .into_iter()
                .map(|n| n.data().as_ffi())
                .collect(),
            node_entry_root_overwrites: snapshot
                .node_entry_root_overwrites
                .into_iter()
                .map(|r| {
                    let mut element_path = runtime.debug_path_for_element(window, r.element);
                    let mut old_root_path = runtime.debug_path_for_element(window, r.old_root);
                    let mut new_root_path = runtime.debug_path_for_element(window, r.new_root);
                    truncate_opt_string_bytes(&mut element_path, max_debug_string_bytes);
                    truncate_opt_string_bytes(&mut old_root_path, max_debug_string_bytes);
                    truncate_opt_string_bytes(&mut new_root_path, max_debug_string_bytes);

                    let mut file = r.file.to_string();
                    truncate_string_bytes(&mut file, max_debug_string_bytes);

                    ElementNodeEntryRootOverwriteV1 {
                        frame_id: r.frame_id.0,
                        element: r.element.0,
                        element_path,
                        old_root: r.old_root.0,
                        old_root_path,
                        new_root: r.new_root.0,
                        new_root_path,
                        old_node: r.old_node.data().as_ffi(),
                        new_node: r.new_node.data().as_ffi(),
                        location: Some(UiSourceLocationV1 {
                            file,
                            line: r.line,
                            column: r.column,
                        }),
                    }
                })
                .collect(),
        };

        for entry in &mut out.observed_layout_queries {
            for region in &mut entry.regions {
                truncate_opt_string_bytes(&mut region.region_name, max_debug_string_bytes);
                truncate_string_bytes(&mut region.invalidation, max_debug_string_bytes);
            }
        }
        for region in &mut out.layout_query_regions {
            truncate_opt_string_bytes(&mut region.name, max_debug_string_bytes);
        }
        for entry in &mut out.observed_environment {
            for key in &mut entry.keys {
                truncate_string_bytes(&mut key.key, max_debug_string_bytes);
                truncate_string_bytes(&mut key.invalidation, max_debug_string_bytes);
            }
        }
        for entry in &mut out.observed_models {
            for (_, inv) in &mut entry.models {
                truncate_string_bytes(inv, max_debug_string_bytes);
            }
        }
        for entry in &mut out.observed_globals {
            for (_, inv) in &mut entry.globals {
                truncate_string_bytes(inv, max_debug_string_bytes);
            }
        }

        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedUiEventV1 {
    pub tick_id: u64,
    pub frame_id: u64,
    pub window: u64,
    pub kind: String,
    pub position: Option<PointV1>,
    pub debug: String,
}

impl RecordedUiEventV1 {
    fn from_event(app: &App, window: AppWindowId, event: &Event, redact_text: bool) -> Self {
        let kind = event_kind(event);
        let position = event.pointer_event().map(|p| PointV1::from(p.position()));
        let debug = event_debug_string(event, redact_text);

        Self {
            tick_id: app.tick_id().0,
            frame_id: app.frame_id().0,
            window: window.data().as_ffi(),
            kind,
            position,
            debug,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointV1 {
    pub x: f32,
    pub y: f32,
}

impl From<Point> for PointV1 {
    fn from(value: Point) -> Self {
        Self {
            x: value.x.0,
            y: value.y.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RectV1 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<Rect> for RectV1 {
    fn from(value: Rect) -> Self {
        Self {
            x: value.origin.x.0,
            y: value.origin.y.0,
            w: value.size.width.0,
            h: value.size.height.0,
        }
    }
}

fn invalidation_label(inv: Invalidation) -> &'static str {
    match inv {
        Invalidation::Paint => "paint",
        Invalidation::Layout => "layout",
        Invalidation::HitTest => "hit_test",
        Invalidation::HitTestOnly => "hit_test_only",
    }
}

fn pointer_occlusion_label(occlusion: fret_ui::tree::PointerOcclusion) -> String {
    match occlusion {
        fret_ui::tree::PointerOcclusion::None => "none",
        fret_ui::tree::PointerOcclusion::BlockMouse => "block_mouse",
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll => "block_mouse_except_scroll",
    }
    .to_string()
}

fn viewport_pointer_type_label(pointer_type: fret_core::PointerType) -> &'static str {
    match pointer_type {
        fret_core::PointerType::Mouse => "mouse",
        fret_core::PointerType::Touch => "touch",
        fret_core::PointerType::Pen => "pen",
        fret_core::PointerType::Unknown => "unknown",
    }
}

fn color_scheme_label(scheme: fret_core::ColorScheme) -> &'static str {
    match scheme {
        fret_core::ColorScheme::Light => "light",
        fret_core::ColorScheme::Dark => "dark",
    }
}

fn contrast_preference_label(preference: fret_core::ContrastPreference) -> &'static str {
    match preference {
        fret_core::ContrastPreference::NoPreference => "no_preference",
        fret_core::ContrastPreference::More => "more",
        fret_core::ContrastPreference::Less => "less",
        fret_core::ContrastPreference::Custom => "custom",
    }
}

fn forced_colors_mode_label(mode: fret_core::ForcedColorsMode) -> &'static str {
    match mode {
        fret_core::ForcedColorsMode::None => "none",
        fret_core::ForcedColorsMode::Active => "active",
    }
}

fn viewport_cancel_reason_label(reason: fret_core::PointerCancelReason) -> &'static str {
    match reason {
        fret_core::PointerCancelReason::LeftWindow => "left_window",
    }
}

fn event_kind(event: &Event) -> String {
    match event {
        Event::Pointer(p) => format!("pointer.{}", p.kind()),
        Event::KeyDown { .. } => "key.down".to_string(),
        Event::KeyUp { .. } => "key.up".to_string(),
        Event::TextInput(_) => "text.input".to_string(),
        Event::Ime(_) => "ime".to_string(),
        Event::Timer { .. } => "timer".to_string(),
        Event::WindowCloseRequested => "window.close_requested".to_string(),
        other => format!("{other:?}")
            .split_whitespace()
            .next()
            .unwrap_or("event")
            .to_string(),
    }
}

fn event_debug_string(event: &Event, redact_text: bool) -> String {
    if !redact_text {
        return format!("{event:?}");
    }

    match event {
        Event::TextInput(text) => format!("TextInput(len={})", text.len()),
        Event::Ime(_) => "Ime(<redacted>)".to_string(),
        _ => format!("{event:?}"),
    }
}

fn unix_ms_now() -> u64 {
    fret_core::time::SystemTime::now()
        .duration_since(fret_core::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

fn semantics_fingerprint_v1(
    snapshot: &fret_core::SemanticsSnapshot,
    redact_text: bool,
    max_string_bytes: usize,
) -> u64 {
    let mut hasher = Fnv1a64::new();
    hasher.write_u64(snapshot.window.data().as_ffi());

    for root in &snapshot.roots {
        hasher.write_u64(key_to_u64(root.root));
        hasher.write_bool(root.visible);
        hasher.write_bool(root.blocks_underlay_input);
        hasher.write_bool(root.hit_testable);
        hasher.write_u32(root.z_index);
    }

    hasher.write_opt_u64(snapshot.barrier_root.map(key_to_u64));
    hasher.write_opt_u64(snapshot.focus_barrier_root.map(key_to_u64));
    hasher.write_opt_u64(snapshot.focus.map(key_to_u64));
    hasher.write_opt_u64(snapshot.captured.map(key_to_u64));

    for node in &snapshot.nodes {
        hasher.write_u64(key_to_u64(node.id));
        hasher.write_opt_u64(node.parent.map(key_to_u64));
        hasher.write_str_bytes(semantics_role_label(node.role).as_bytes());

        hasher.write_f32(node.bounds.origin.x.0);
        hasher.write_f32(node.bounds.origin.y.0);
        hasher.write_f32(node.bounds.size.width.0);
        hasher.write_f32(node.bounds.size.height.0);

        hasher.write_bool(node.flags.focused);
        hasher.write_bool(node.flags.captured);
        hasher.write_bool(node.flags.disabled);
        hasher.write_bool(node.flags.selected);
        hasher.write_bool(node.flags.expanded);
        hasher.write_opt_bool(node.flags.checked);

        hasher.write_opt_str(node.test_id.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_u64(node.active_descendant.map(key_to_u64));
        hasher.write_opt_u32(node.pos_in_set);
        hasher.write_opt_u32(node.set_size);
        hasher.write_opt_str(node.label.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_str(node.value.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_pair_u32(node.text_selection);
        hasher.write_opt_pair_u32(node.text_composition);

        hasher.write_bool(node.actions.focus);
        hasher.write_bool(node.actions.invoke);
        hasher.write_bool(node.actions.set_value);
        hasher.write_bool(node.actions.set_text_selection);

        hasher.write_u32(node.labelled_by.len() as u32);
        for id in &node.labelled_by {
            hasher.write_u64(key_to_u64(*id));
        }
        hasher.write_u32(node.described_by.len() as u32);
        for id in &node.described_by {
            hasher.write_u64(key_to_u64(*id));
        }
        hasher.write_u32(node.controls.len() as u32);
        for id in &node.controls {
            hasher.write_u64(key_to_u64(*id));
        }
    }

    hasher.finish()
}

struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self {
            state: Self::OFFSET_BASIS,
        }
    }

    fn write_u8(&mut self, v: u8) {
        self.state ^= v as u64;
        self.state = self.state.wrapping_mul(Self::PRIME);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u8(b);
        }
    }

    fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    fn write_bool(&mut self, v: bool) {
        self.write_u8(if v { 1 } else { 0 });
    }

    fn write_opt_u64(&mut self, v: Option<u64>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_u64(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_u32(&mut self, v: Option<u32>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_u32(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_bool(&mut self, v: Option<bool>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_bool(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_pair_u32(&mut self, v: Option<(u32, u32)>) {
        match v {
            Some((a, b)) => {
                self.write_u8(1);
                self.write_u32(a);
                self.write_u32(b);
            }
            None => self.write_u8(0),
        }
    }

    fn write_str_bytes(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }

    fn write_opt_str(&mut self, s: Option<&str>, redact_text: bool, max_string_bytes: usize) {
        match s {
            Some(s) => {
                self.write_u8(1);
                if redact_text {
                    self.write_u32(s.len().min(u32::MAX as usize) as u32);
                } else {
                    let bytes = s.as_bytes();
                    self.write_u32(bytes.len().min(max_string_bytes) as u32);
                    self.write_bytes(&bytes[..bytes.len().min(max_string_bytes)]);
                }
            }
            None => self.write_u8(0),
        }
    }

    fn finish(self) -> u64 {
        self.state
    }
}

const MAX_SELECTOR_TRACE_ENTRIES: usize = 64;
const MAX_SELECTOR_TRACE_CANDIDATES: usize = 6;
const MAX_HIT_TEST_TRACE_ENTRIES: usize = 64;
const MAX_FOCUS_TRACE_ENTRIES: usize = 64;
const MAX_SHORTCUT_ROUTING_TRACE_ENTRIES: usize = 128;
const MAX_OVERLAY_PLACEMENT_TRACE_ENTRIES: usize = 128;
const MAX_WEB_IME_TRACE_ENTRIES: usize = 64;
const MAX_IME_EVENT_TRACE_ENTRIES: usize = 64;

fn selector_trace_eq(a: &UiSelectorV1, b: &UiSelectorV1) -> bool {
    match (a, b) {
        (
            UiSelectorV1::RoleAndName {
                role: a_role,
                name: a_name,
            },
            UiSelectorV1::RoleAndName {
                role: b_role,
                name: b_name,
            },
        ) => a_role == b_role && a_name == b_name,
        (
            UiSelectorV1::RoleAndPath {
                role: a_role,
                name: a_name,
                ancestors: a_ancestors,
            },
            UiSelectorV1::RoleAndPath {
                role: b_role,
                name: b_name,
                ancestors: b_ancestors,
            },
        ) => {
            a_role == b_role
                && a_name == b_name
                && a_ancestors.len() == b_ancestors.len()
                && a_ancestors
                    .iter()
                    .zip(b_ancestors.iter())
                    .all(|(a, b)| a.role == b.role && a.name == b.name)
        }
        (UiSelectorV1::TestId { id: a_id }, UiSelectorV1::TestId { id: b_id }) => a_id == b_id,
        (
            UiSelectorV1::GlobalElementId { element: a_el },
            UiSelectorV1::GlobalElementId { element: b_el },
        ) => a_el == b_el,
        (UiSelectorV1::NodeId { node: a_node }, UiSelectorV1::NodeId { node: b_node }) => {
            a_node == b_node
        }
        _ => false,
    }
}

fn hit_test_trace_entry_eq(a: &UiHitTestTraceEntryV1, b: &UiHitTestTraceEntryV1) -> bool {
    a.step_index == b.step_index
        && selector_trace_eq(&a.selector, &b.selector)
        && a.note == b.note
        && a.position.x_px == b.position.x_px
        && a.position.y_px == b.position.y_px
}

fn push_selector_resolution_trace(
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
    entry: UiSelectorResolutionTraceEntryV1,
) {
    if let Some(existing) = trace.iter_mut().rev().find(|e| {
        e.step_index == entry.step_index && selector_trace_eq(&e.selector, &entry.selector)
    }) {
        *existing = entry;
        return;
    }

    trace.push(entry);
    if trace.len() > MAX_SELECTOR_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_SELECTOR_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

fn push_hit_test_trace(trace: &mut Vec<UiHitTestTraceEntryV1>, entry: UiHitTestTraceEntryV1) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| hit_test_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_HIT_TEST_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_HIT_TEST_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

const MAX_BOUNDS_STABLE_TRACE_ENTRIES: usize = 32;
const MAX_CLICK_STABLE_TRACE_ENTRIES: usize = 32;

fn bounds_stable_trace_entry_eq(
    a: &UiBoundsStableTraceEntryV1,
    b: &UiBoundsStableTraceEntryV1,
) -> bool {
    a.step_index == b.step_index && selector_trace_eq(&a.selector, &b.selector)
}

fn push_bounds_stable_trace(
    trace: &mut Vec<UiBoundsStableTraceEntryV1>,
    entry: UiBoundsStableTraceEntryV1,
) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| bounds_stable_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_BOUNDS_STABLE_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_BOUNDS_STABLE_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

fn ui_rect_from_rect(rect: Rect) -> UiRectV1 {
    UiRectV1 {
        x_px: rect.origin.x.0,
        y_px: rect.origin.y.0,
        w_px: rect.size.width.0,
        h_px: rect.size.height.0,
    }
}

fn ui_size_from_size(size: fret_core::Size) -> UiSizeV1 {
    UiSizeV1 {
        w_px: size.width.0,
        h_px: size.height.0,
    }
}

fn ui_edges_from_edges(edges: fret_core::Edges) -> UiEdgesV1 {
    UiEdgesV1 {
        top_px: edges.top.0,
        right_px: edges.right.0,
        bottom_px: edges.bottom.0,
        left_px: edges.left.0,
    }
}

fn ui_layout_direction_from_dir(
    dir: fret_ui::overlay_placement::LayoutDirection,
) -> UiLayoutDirectionV1 {
    match dir {
        fret_ui::overlay_placement::LayoutDirection::Ltr => UiLayoutDirectionV1::Ltr,
        fret_ui::overlay_placement::LayoutDirection::Rtl => UiLayoutDirectionV1::Rtl,
    }
}

fn ui_overlay_side_from_side(side: fret_ui::overlay_placement::Side) -> UiOverlaySideV1 {
    match side {
        fret_ui::overlay_placement::Side::Top => UiOverlaySideV1::Top,
        fret_ui::overlay_placement::Side::Bottom => UiOverlaySideV1::Bottom,
        fret_ui::overlay_placement::Side::Left => UiOverlaySideV1::Left,
        fret_ui::overlay_placement::Side::Right => UiOverlaySideV1::Right,
    }
}

fn ui_overlay_align_from_align(align: fret_ui::overlay_placement::Align) -> UiOverlayAlignV1 {
    match align {
        fret_ui::overlay_placement::Align::Start => UiOverlayAlignV1::Start,
        fret_ui::overlay_placement::Align::Center => UiOverlayAlignV1::Center,
        fret_ui::overlay_placement::Align::End => UiOverlayAlignV1::End,
    }
}

fn ui_overlay_sticky_from_sticky(
    sticky: fret_ui::overlay_placement::StickyMode,
) -> UiOverlayStickyModeV1 {
    match sticky {
        fret_ui::overlay_placement::StickyMode::Partial => UiOverlayStickyModeV1::Partial,
        fret_ui::overlay_placement::StickyMode::Always => UiOverlayStickyModeV1::Always,
    }
}

fn test_id_for_element(
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    window: AppWindowId,
    element: fret_ui::elements::GlobalElementId,
) -> Option<String> {
    let (Some(rt), Some(snapshot)) = (element_runtime, semantics_snapshot) else {
        return None;
    };
    let node_id = rt.node_for_element(window, element)?;
    let node = snapshot
        .nodes
        .iter()
        .find(|n| n.id.data().as_ffi() == node_id.data().as_ffi())?;
    node.test_id.clone()
}

fn record_overlay_placement_trace(
    trace: &mut Vec<UiOverlayPlacementTraceEntryV1>,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    window: AppWindowId,
    step_index: u32,
    note: &str,
) {
    let snapshot = element_runtime.and_then(|rt| rt.diagnostics_snapshot(window));
    let Some(snapshot) = snapshot else {
        return;
    };

    for rec in snapshot.overlay_placement.iter() {
        match rec {
            fret_ui::elements::OverlayPlacementDiagnosticsRecord::AnchoredPanel(r) => {
                let anchor_test_id = r.anchor_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                let content_test_id = r.content_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                let t = r.trace;
                let options = t.options;

                let arrow = t.layout.arrow.map(|a| UiOverlayArrowLayoutV1 {
                    side: ui_overlay_side_from_side(a.side),
                    offset_px: a.offset.0,
                    alignment_offset_px: a.alignment_offset.0,
                    center_offset_px: a.center_offset.0,
                });

                push_overlay_placement_trace(
                    trace,
                    UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                        step_index,
                        note: Some(note.to_string()),
                        frame_id: r.frame_id.0,
                        overlay_root_name: r.overlay_root_name.as_deref().map(|s| s.to_string()),
                        anchor_element: r.anchor_element.map(|id| id.0),
                        anchor_test_id,
                        content_element: r.content_element.map(|id| id.0),
                        content_test_id,
                        outer_input: ui_rect_from_rect(t.outer_input),
                        outer_collision: ui_rect_from_rect(t.outer_collision),
                        anchor: ui_rect_from_rect(t.anchor),
                        desired: ui_size_from_size(t.desired),
                        side_offset_px: t.side_offset.0,
                        preferred_side: ui_overlay_side_from_side(t.preferred_side),
                        align: ui_overlay_align_from_align(t.align),
                        direction: ui_layout_direction_from_dir(options.direction),
                        sticky: ui_overlay_sticky_from_sticky(options.sticky),
                        offset: UiOverlayOffsetV1 {
                            main_axis_px: options.offset.main_axis.0,
                            cross_axis_px: options.offset.cross_axis.0,
                            alignment_axis_px: options.offset.alignment_axis.map(|v| v.0),
                        },
                        shift: UiOverlayShiftV1 {
                            main_axis: options.shift.main_axis,
                            cross_axis: options.shift.cross_axis,
                        },
                        collision_padding: ui_edges_from_edges(options.collision.padding),
                        collision_boundary: options.collision.boundary.map(ui_rect_from_rect),
                        gap_px: t.gap.0,
                        preferred_rect: ui_rect_from_rect(t.preferred_rect),
                        flipped_rect: ui_rect_from_rect(t.flipped_rect),
                        preferred_fits_without_main_clamp: t.preferred_fits_without_main_clamp,
                        flipped_fits_without_main_clamp: t.flipped_fits_without_main_clamp,
                        preferred_available_main_px: t.preferred_available_main_px,
                        flipped_available_main_px: t.flipped_available_main_px,
                        chosen_side: ui_overlay_side_from_side(t.chosen_side),
                        chosen_rect: ui_rect_from_rect(t.chosen_rect),
                        rect_after_shift: ui_rect_from_rect(t.rect_after_shift),
                        shift_delta: UiPointV1 {
                            x_px: t.shift_delta.x.0,
                            y_px: t.shift_delta.y.0,
                        },
                        final_rect: ui_rect_from_rect(t.layout.rect),
                        arrow,
                    },
                );
            }
            fret_ui::elements::OverlayPlacementDiagnosticsRecord::PlacedRect(r) => {
                let anchor_test_id = r.anchor_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                let content_test_id = r.content_element.and_then(|el| {
                    test_id_for_element(element_runtime, semantics_snapshot, window, el)
                });
                push_overlay_placement_trace(
                    trace,
                    UiOverlayPlacementTraceEntryV1::PlacedRect {
                        step_index,
                        note: Some(note.to_string()),
                        frame_id: r.frame_id.0,
                        overlay_root_name: r.overlay_root_name.as_deref().map(|s| s.to_string()),
                        anchor_element: r.anchor_element.map(|id| id.0),
                        anchor_test_id,
                        content_element: r.content_element.map(|id| id.0),
                        content_test_id,
                        outer: ui_rect_from_rect(r.outer),
                        anchor: ui_rect_from_rect(r.anchor),
                        placed: ui_rect_from_rect(r.placed),
                        side: r.side.map(ui_overlay_side_from_side),
                    },
                );
            }
        }
    }
}

fn record_focus_trace(
    trace: &mut Vec<UiFocusTraceEntryV1>,
    app: &App,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&UiTree<App>>,
    step_index: u32,
    expected_node_id: Option<u64>,
    expected_test_id: Option<&str>,
    note: &str,
) {
    let snapshot = element_runtime.and_then(|rt| rt.diagnostics_snapshot(window));
    let focused_element = snapshot
        .as_ref()
        .and_then(|s| s.focused_element)
        .map(|id| id.0);
    let focused_element_path = snapshot
        .as_ref()
        .and_then(|s| s.focused_element)
        .and_then(|id| element_runtime.and_then(|rt| rt.debug_path_for_element(window, id)));
    let focused_node_id = snapshot
        .as_ref()
        .and_then(|s| s.focused_element_node)
        .map(key_to_u64);

    let (focused_test_id, focused_role) =
        if let (Some(snapshot), Some(focused_node_id)) = (semantics_snapshot, focused_node_id) {
            let node = snapshot
                .nodes
                .iter()
                .find(|n| n.id.data().as_ffi() == focused_node_id);
            let test_id = node.and_then(|n| n.test_id.clone());
            let role = node.map(|n| semantics_role_label(n.role).to_string());
            (test_id, role)
        } else {
            (None, None)
        };

    let matches_expected = match (expected_node_id, expected_test_id) {
        (Some(expected_node_id), _) => focused_node_id.map(|id| id == expected_node_id),
        (None, Some(expected_test_id)) => {
            focused_test_id.as_deref().map(|id| id == expected_test_id)
        }
        _ => None,
    };

    let (
        modal_barrier_root,
        focus_barrier_root,
        pointer_occlusion,
        pointer_occlusion_layer_id,
        pointer_capture_active,
        pointer_capture_layer_id,
        pointer_capture_multiple_layers,
    ) = if let Some(ui) = ui {
        let arbitration = ui.input_arbitration_snapshot();
        (
            arbitration.modal_barrier_root.map(key_to_u64),
            arbitration.focus_barrier_root.map(key_to_u64),
            Some(pointer_occlusion_label(arbitration.pointer_occlusion)),
            arbitration
                .pointer_occlusion_layer
                .map(|l| l.data().as_ffi()),
            Some(arbitration.pointer_capture_active),
            arbitration.pointer_capture_layer.map(|l| l.data().as_ffi()),
            Some(arbitration.pointer_capture_multiple_layers),
        )
    } else {
        (None, None, None, None, None, None, None)
    };

    let reason_code = {
        if matches_expected == Some(true) {
            Some("focus.matches_expected".to_string())
        } else if let (Some(expected), Some(barrier), Some(snapshot)) =
            (expected_node_id, focus_barrier_root, semantics_snapshot)
        {
            let index = SemanticsIndex::new(snapshot);
            if !index.is_descendant_of_or_self(expected, barrier) {
                Some("focus.blocked_by_focus_barrier".to_string())
            } else {
                Some("focus.mismatch".to_string())
            }
        } else if let (Some(expected), Some(barrier), Some(snapshot)) =
            (expected_node_id, modal_barrier_root, semantics_snapshot)
        {
            let index = SemanticsIndex::new(snapshot);
            if !index.is_descendant_of_or_self(expected, barrier) {
                Some("focus.blocked_by_modal_barrier".to_string())
            } else {
                Some("focus.mismatch".to_string())
            }
        } else {
            Some("focus.mismatch".to_string())
        }
    };

    push_focus_trace(
        trace,
        UiFocusTraceEntryV1 {
            step_index,
            note: Some(note.to_string()),
            reason_code,
            text_input_snapshot: app
                .global::<fret_runtime::WindowTextInputSnapshotService>()
                .and_then(|svc| svc.snapshot(window).cloned())
                .map(|snapshot| UiTextInputSnapshotV1 {
                    focus_is_text_input: snapshot.focus_is_text_input,
                    is_composing: snapshot.is_composing,
                    text_len_utf16: snapshot.text_len_utf16,
                    selection_utf16: snapshot.selection_utf16,
                    marked_utf16: snapshot.marked_utf16,
                    ime_cursor_area: snapshot.ime_cursor_area.map(|r| UiRectV1 {
                        x_px: r.origin.x.0,
                        y_px: r.origin.y.0,
                        w_px: r.size.width.0,
                        h_px: r.size.height.0,
                    }),
                }),
            expected_node_id,
            expected_test_id: expected_test_id.map(|s| s.to_string()),
            modal_barrier_root,
            focus_barrier_root,
            pointer_occlusion,
            pointer_occlusion_layer_id,
            pointer_capture_active,
            pointer_capture_layer_id,
            pointer_capture_multiple_layers,
            focused_element,
            focused_element_path,
            focused_node_id,
            focused_test_id,
            focused_role,
            matches_expected,
        },
    );
}

fn record_web_ime_trace(
    trace: &mut Vec<UiWebImeTraceEntryV1>,
    app: &App,
    step_index: u32,
    note: &str,
) {
    let snapshot = app
        .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
        .filter(|snapshot| **snapshot != fret_core::input::WebImeBridgeDebugSnapshot::default());
    let Some(snapshot) = snapshot else {
        return;
    };

    let last_preedit_len = snapshot
        .last_preedit_text
        .as_deref()
        .map(|s| s.len().min(u32::MAX as usize) as u32);
    let last_commit_len = snapshot
        .last_commit_text
        .as_deref()
        .map(|s| s.len().min(u32::MAX as usize) as u32);

    push_web_ime_trace(
        trace,
        UiWebImeTraceEntryV1 {
            step_index,
            note: Some(note.to_string()),
            enabled: snapshot.enabled,
            composing: snapshot.composing,
            suppress_next_input: snapshot.suppress_next_input,
            textarea_has_focus: snapshot.textarea_has_focus,
            active_element_tag: snapshot.active_element_tag.clone(),
            position_mode: snapshot.position_mode.clone(),
            mount_kind: snapshot.mount_kind.clone(),
            device_pixel_ratio: snapshot.device_pixel_ratio,
            textarea_selection_start_utf16: snapshot.textarea_selection_start_utf16,
            textarea_selection_end_utf16: snapshot.textarea_selection_end_utf16,
            last_cursor_area: snapshot.last_cursor_area.map(|r| UiRectV1 {
                x_px: r.origin.x.0,
                y_px: r.origin.y.0,
                w_px: r.size.width.0,
                h_px: r.size.height.0,
            }),
            last_cursor_anchor_px: snapshot.last_cursor_anchor_px,
            last_input_type: snapshot.last_input_type.clone(),
            last_preedit_len,
            last_preedit_cursor_utf16: snapshot.last_preedit_cursor_utf16,
            last_commit_len,
            beforeinput_seen: snapshot.beforeinput_seen,
            input_seen: snapshot.input_seen,
            suppressed_input_seen: snapshot.suppressed_input_seen,
            composition_start_seen: snapshot.composition_start_seen,
            composition_update_seen: snapshot.composition_update_seen,
            composition_end_seen: snapshot.composition_end_seen,
            cursor_area_set_seen: snapshot.cursor_area_set_seen,
        },
    );
}

fn record_ime_event_trace(
    trace: &mut Vec<UiImeEventTraceEntryV1>,
    step_index: u32,
    note: &str,
    event: &fret_core::input::ImeEvent,
) {
    let mut preedit_len: Option<u32> = None;
    let mut preedit_cursor: Option<(u32, u32)> = None;
    let mut commit_len: Option<u32> = None;
    let mut delete_surrounding: Option<(u32, u32)> = None;

    let kind: &'static str = match event {
        fret_core::input::ImeEvent::Enabled => "enabled",
        fret_core::input::ImeEvent::Disabled => "disabled",
        fret_core::input::ImeEvent::Commit(text) => {
            commit_len = Some(text.len().min(u32::MAX as usize) as u32);
            "commit"
        }
        fret_core::input::ImeEvent::Preedit { text, cursor } => {
            preedit_len = Some(text.len().min(u32::MAX as usize) as u32);
            preedit_cursor = cursor.map(|(a, b)| {
                (
                    a.min(u32::MAX as usize) as u32,
                    b.min(u32::MAX as usize) as u32,
                )
            });
            "preedit"
        }
        fret_core::input::ImeEvent::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } => {
            delete_surrounding = Some((
                (*before_bytes).min(u32::MAX as usize) as u32,
                (*after_bytes).min(u32::MAX as usize) as u32,
            ));
            "delete_surrounding"
        }
    };

    push_ime_event_trace(
        trace,
        UiImeEventTraceEntryV1 {
            step_index,
            note: Some(note.to_string()),
            kind: kind.to_string(),
            preedit_len,
            preedit_cursor,
            commit_len,
            delete_surrounding,
        },
    );
}

fn hit_test_scope_roots_evidence(
    position: Point,
    ui: &mut UiTree<App>,
) -> (
    Option<NodeId>,
    Option<u64>,
    Option<u64>,
    Vec<UiHitTestScopeRootEvidenceV1>,
    fret_ui::tree::UiInputArbitrationSnapshot,
) {
    let snap = UiHitTestSnapshotV1::from_tree(position, ui);
    let arbitration = ui.input_arbitration_snapshot();
    let scope_roots = snap
        .scope_roots
        .into_iter()
        .map(|r| UiHitTestScopeRootEvidenceV1 {
            kind: r.kind,
            root: r.root,
            layer_id: r.layer_id,
            pointer_occlusion: r.pointer_occlusion,
            blocks_underlay_input: r.blocks_underlay_input,
            hit_testable: r.hit_testable,
        })
        .collect();
    (
        snap.hit
            .map(|id| NodeId::from(slotmap::KeyData::from_ffi(id))),
        snap.barrier_root,
        snap.focus_barrier_root,
        scope_roots,
        arbitration,
    )
}

fn record_hit_test_trace_for_selector(
    trace: &mut Vec<UiHitTestTraceEntryV1>,
    ui: &mut UiTree<App>,
    element_runtime: Option<&ElementRuntime>,
    window: AppWindowId,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    selector: &UiSelectorV1,
    step_index: u32,
    position: Point,
    intended: Option<&fret_core::SemanticsNode>,
    note: Option<&str>,
    max_debug_string_bytes: usize,
) {
    let entry = build_hit_test_trace_entry_for_selector(
        ui,
        element_runtime,
        window,
        semantics_snapshot,
        selector,
        step_index,
        position,
        intended,
        note,
        max_debug_string_bytes,
    );
    push_hit_test_trace(trace, entry);
}

fn build_hit_test_trace_entry_for_selector(
    ui: &mut UiTree<App>,
    element_runtime: Option<&ElementRuntime>,
    window: AppWindowId,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    selector: &UiSelectorV1,
    step_index: u32,
    position: Point,
    intended: Option<&fret_core::SemanticsNode>,
    note: Option<&str>,
    max_debug_string_bytes: usize,
) -> UiHitTestTraceEntryV1 {
    const MAX_HIT_NODE_PATH: usize = 64;

    let (hit_node, barrier_root, focus_barrier_root, scope_roots, arbitration) =
        hit_test_scope_roots_evidence(position, ui);

    let hit_semantics =
        semantics_snapshot.and_then(|snapshot| pick_semantics_node_at(snapshot, ui, position));
    let hit_semantics_node_id = hit_semantics.map(|n| n.id.data().as_ffi());
    let hit_semantics_test_id = hit_semantics.and_then(|n| n.test_id.clone());

    let intended_node_id = intended.map(|n| n.id.data().as_ffi());
    let intended_test_id = intended.and_then(|n| n.test_id.clone());
    let intended_bounds = intended.map(|n| UiRectV1 {
        x_px: n.bounds.origin.x.0,
        y_px: n.bounds.origin.y.0,
        w_px: n.bounds.size.width.0,
        h_px: n.bounds.size.height.0,
    });

    let hit_node_id = hit_node.map(|id| id.data().as_ffi());
    let hit_node_path: Vec<u64> = hit_node
        .map(|id| {
            ui.debug_node_path(id)
                .into_iter()
                .rev()
                .take(MAX_HIT_NODE_PATH)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|n| n.data().as_ffi())
                .collect()
        })
        .unwrap_or_default();

    let includes_intended = intended.map(|target| {
        if let Some(hit_id) = hit_semantics_node_id {
            if hit_id == target.id.data().as_ffi() {
                return true;
            }
        }
        if let (Some(want), Some(got)) =
            (target.test_id.as_deref(), hit_semantics_test_id.as_deref())
        {
            return want == got;
        }
        false
    });

    let hit_path_contains_intended = intended_node_id.map(|id| hit_node_path.contains(&id));

    let pointer_occlusion = pointer_occlusion_label(arbitration.pointer_occlusion).to_string();
    let pointer_occlusion_layer_id = arbitration
        .pointer_occlusion_layer
        .map(|id| id.data().as_ffi());
    let pointer_capture_layer_id = arbitration
        .pointer_capture_layer
        .map(|id| id.data().as_ffi());

    let pointer_occlusion_root = pointer_occlusion_layer_id.and_then(|layer_id| {
        scope_roots
            .iter()
            .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
            .map(|r| r.root)
    });
    let (
        pointer_occlusion_node_id,
        pointer_occlusion_test_id,
        pointer_occlusion_role,
        pointer_occlusion_bounds,
    ) = pointer_occlusion_root
        .map(|root| {
            let node = NodeId::from(KeyData::from_ffi(root));
            let mut test_id: Option<String> = None;
            let mut role: Option<String> = None;
            let mut bounds: Option<UiRectV1> = None;
            if let Some(snapshot) = semantics_snapshot {
                if let Some(n) = snapshot.nodes.iter().find(|n| n.id == node) {
                    test_id = n.test_id.clone();
                    role = Some(semantics_role_label(n.role).to_string());
                    bounds = Some(UiRectV1 {
                        x_px: n.bounds.origin.x.0,
                        y_px: n.bounds.origin.y.0,
                        w_px: n.bounds.size.width.0,
                        h_px: n.bounds.size.height.0,
                    });
                }
            }
            if bounds.is_none() {
                bounds = ui.debug_node_bounds(node).map(|r| UiRectV1 {
                    x_px: r.origin.x.0,
                    y_px: r.origin.y.0,
                    w_px: r.size.width.0,
                    h_px: r.size.height.0,
                });
            }
            (Some(root), test_id, role, bounds)
        })
        .unwrap_or((None, None, None, None));

    let is_ok = includes_intended == Some(true) || hit_path_contains_intended == Some(true);
    let (blocking_reason, blocking_root, blocking_layer_id) = if is_ok {
        (None, None, None)
    } else if barrier_root.is_some() {
        (Some("modal_barrier"), barrier_root, None)
    } else if focus_barrier_root.is_some() {
        let layer_id = scope_roots
            .iter()
            .find(|r| r.kind == "focus_barrier_root")
            .and_then(|r| r.layer_id);
        (Some("focus_barrier"), focus_barrier_root, layer_id)
    } else if arbitration.pointer_capture_active {
        let blocking_root = pointer_capture_layer_id.and_then(|layer_id| {
            scope_roots
                .iter()
                .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
                .map(|r| r.root)
        });
        (
            Some("pointer_capture"),
            blocking_root,
            pointer_capture_layer_id,
        )
    } else if pointer_occlusion != "none" {
        let blocking_root = pointer_occlusion_layer_id.and_then(|layer_id| {
            scope_roots
                .iter()
                .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
                .map(|r| r.root)
        });
        (
            Some("pointer_occlusion"),
            blocking_root,
            pointer_occlusion_layer_id,
        )
    } else if hit_node_id.is_none() {
        (Some("no_hit"), None, None)
    } else {
        (Some("miss"), None, None)
    };

    let (
        pointer_capture_node_id,
        pointer_capture_test_id,
        pointer_capture_role,
        pointer_capture_bounds,
        pointer_capture_element,
        pointer_capture_element_path,
    ) = ui
        .any_captured_node()
        .map(|captured| {
            let captured_id = captured.data().as_ffi();
            let mut test_id: Option<String> = None;
            let mut role: Option<String> = None;
            let mut bounds: Option<UiRectV1> = None;
            let mut element_id: Option<u64> = None;
            let mut element_path: Option<String> = None;
            if let Some(el) = ui.debug_node_element(captured) {
                element_id = Some(el.0);
                element_path = element_runtime
                    .and_then(|rt| rt.debug_path_for_element(window, el))
                    .map(|mut s| {
                        truncate_string_bytes(&mut s, max_debug_string_bytes);
                        s
                    });
            }
            if let Some(snapshot) = semantics_snapshot {
                if let Some(n) = snapshot.nodes.iter().find(|n| n.id == captured) {
                    test_id = n.test_id.clone();
                    role = Some(semantics_role_label(n.role).to_string());
                    bounds = Some(UiRectV1 {
                        x_px: n.bounds.origin.x.0,
                        y_px: n.bounds.origin.y.0,
                        w_px: n.bounds.size.width.0,
                        h_px: n.bounds.size.height.0,
                    });
                }
            }
            (
                Some(captured_id),
                test_id,
                role,
                bounds,
                element_id,
                element_path,
            )
        })
        .unwrap_or((None, None, None, None, None, None));
    let blocking_layer_hint = blocking_layer_id.and_then(|layer_id| {
        scope_roots
            .iter()
            .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
            .map(|r| {
                let mut parts: Vec<String> = Vec::new();
                parts.push(format!("layer_root={}", r.root));
                if let Some(v) = r.blocks_underlay_input {
                    parts.push(format!("blocks_underlay_input={v}"));
                }
                if let Some(v) = r.hit_testable {
                    parts.push(format!("hit_testable={v}"));
                }
                if let Some(v) = r.pointer_occlusion.as_deref() {
                    parts.push(format!("pointer_occlusion={v}"));
                }
                parts.join(" ")
            })
    });

    let routing_explain = if is_ok {
        None
    } else {
        let intended = intended_test_id
            .as_deref()
            .map(|t| format!("test_id={t}"))
            .or_else(|| intended_node_id.map(|id| format!("node_id={id}")))
            .unwrap_or_else(|| "<none>".to_string());
        let hit = hit_semantics_test_id
            .as_deref()
            .map(|t| format!("test_id={t}"))
            .or_else(|| hit_node_id.map(|id| format!("node_id={id}")))
            .unwrap_or_else(|| "<none>".to_string());

        match blocking_reason {
            Some("modal_barrier") => Some(format!(
                "blocked by modal barrier (barrier_root={}) intended={intended} hit={hit}",
                barrier_root.unwrap_or(0)
            )),
            Some("focus_barrier") => Some(format!(
                "blocked by focus barrier (focus_barrier_root={}) intended={intended} hit={hit}",
                focus_barrier_root.unwrap_or(0)
            )),
            Some("pointer_capture") => Some(format!(
                "blocked by pointer capture (layer_id={}, captured_node_id={}) {}{} intended={intended} hit={hit}",
                blocking_layer_id.unwrap_or(0),
                pointer_capture_node_id.unwrap_or(0),
                blocking_layer_hint.as_deref().unwrap_or(""),
                pointer_capture_element_path
                    .as_deref()
                    .map(|p| format!(" element_path={p}"))
                    .unwrap_or_default(),
            )),
            Some("pointer_occlusion") => Some(format!(
                "blocked by pointer occlusion ({pointer_occlusion}) (layer_id={}, root={}) {} intended={intended} hit={hit}",
                blocking_layer_id.unwrap_or(0),
                blocking_root.unwrap_or(0),
                blocking_layer_hint.as_deref().unwrap_or(""),
            )),
            Some("no_hit") => Some(format!("hit-test returned no node intended={intended}")),
            Some("miss") => Some(format!("hit-test missed intended={intended} hit={hit}")),
            _ => None,
        }
    };

    UiHitTestTraceEntryV1 {
        step_index,
        selector: selector.clone(),
        position: UiPointV1 {
            x_px: position.x.0,
            y_px: position.y.0,
        },
        intended_node_id,
        intended_test_id,
        intended_bounds,
        hit_node_id,
        hit_node_path,
        hit_semantics_node_id,
        hit_semantics_test_id,
        includes_intended,
        hit_path_contains_intended,
        blocking_reason: blocking_reason.map(|s| s.to_string()),
        blocking_root,
        blocking_layer_id,
        routing_explain,
        barrier_root,
        focus_barrier_root,
        pointer_occlusion: Some(pointer_occlusion),
        pointer_occlusion_layer_id,
        pointer_occlusion_node_id,
        pointer_occlusion_test_id,
        pointer_occlusion_role,
        pointer_occlusion_bounds,
        pointer_capture_active: Some(arbitration.pointer_capture_active),
        pointer_capture_layer_id,
        pointer_capture_multiple_layers: Some(arbitration.pointer_capture_multiple_layers),
        pointer_capture_node_id,
        pointer_capture_test_id,
        pointer_capture_role,
        pointer_capture_bounds,
        pointer_capture_element,
        pointer_capture_element_path,
        scope_roots,
        note: note.map(|s| s.to_string()),
    }
}

fn select_semantics_node_with_trace<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
    step_index: u32,
    redact_text: bool,
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);
    let mut matches: Vec<&'a fret_core::SemanticsNode> = Vec::new();
    let mut note: Option<String> = None;

    match selector {
        UiSelectorV1::NodeId { node } => {
            if let Some(n) = index
                .by_id
                .get(node)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
            {
                matches.push(n);
            }
        }
        UiSelectorV1::RoleAndName { role, name } => {
            let Some(role) = parse_semantics_role(role) else {
                note = Some("invalid_role".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };

            matches.extend(snapshot.nodes.iter().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id) && n.role == role && n.label.as_deref() == Some(name)
            }));
        }
        UiSelectorV1::RoleAndPath {
            role,
            name,
            ancestors,
        } => {
            let Some(role) = parse_semantics_role(role) else {
                note = Some("invalid_role".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };

            let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                Vec::with_capacity(ancestors.len());
            for a in ancestors {
                let Some(r) = parse_semantics_role(&a.role) else {
                    note = Some("invalid_ancestor_role".to_string());
                    push_selector_resolution_trace(
                        trace,
                        UiSelectorResolutionTraceEntryV1 {
                            step_index,
                            selector: selector.clone(),
                            match_count: 0,
                            chosen_node_id: None,
                            candidates: Vec::new(),
                            note,
                        },
                    );
                    return None;
                };
                parsed_ancestors.push((r, a.name.as_str()));
            }

            matches.extend(snapshot.nodes.iter().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id)
                    && n.role == role
                    && n.label.as_deref() == Some(name)
                    && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
            }));
        }
        UiSelectorV1::TestId { id } => {
            matches.extend(snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id) && n.test_id.as_deref() == Some(id)
            }));
            if matches.is_empty() {
                // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
                note = Some("fallback_hidden_nodes".to_string());
                matches.extend(
                    snapshot
                        .nodes
                        .iter()
                        .filter(|n| n.test_id.as_deref() == Some(id)),
                );
            }
        }
        UiSelectorV1::GlobalElementId { element } => {
            let Some(node) = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            }) else {
                note = Some("element_runtime_missing".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };
            let node_id = node.data().as_ffi();
            if let Some(n) = index
                .by_id
                .get(&node_id)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
            {
                matches.push(n);
            }
        }
    }

    let match_count = matches.len().min(u32::MAX as usize) as u32;
    let chosen = pick_best_match(matches.iter().copied(), &index);
    let chosen_node_id = chosen.map(|n| n.id.data().as_ffi());

    let mut ranked: Vec<((u32, u32, u64), &'a fret_core::SemanticsNode)> = matches
        .iter()
        .copied()
        .map(|n| {
            let id = n.id.data().as_ffi();
            ((index.root_z_for(id), index.depth_for(id), id), n)
        })
        .collect();
    ranked.sort_by(|(a, _), (b, _)| b.cmp(a));

    let candidates: Vec<UiSelectorResolutionCandidateV1> = ranked
        .into_iter()
        .take(MAX_SELECTOR_TRACE_CANDIDATES)
        .map(|(_rank, n)| UiSelectorResolutionCandidateV1 {
            node_id: n.id.data().as_ffi(),
            role: semantics_role_label(n.role).to_string(),
            name: if redact_text { None } else { n.label.clone() },
            test_id: n.test_id.clone(),
        })
        .collect();

    push_selector_resolution_trace(
        trace,
        UiSelectorResolutionTraceEntryV1 {
            step_index,
            selector: selector.clone(),
            match_count,
            chosen_node_id,
            candidates,
            note,
        },
    );

    chosen
}

fn reason_code_for_script_failure(reason: &str) -> Option<&'static str> {
    let reason = reason.trim();
    if reason.is_empty() {
        return None;
    }

    match reason {
        "no_semantics_snapshot" => Some("semantics.missing"),
        "assert_failed" => Some("assert.failed"),
        "window_target_unresolved" => Some("window.target_unresolved"),
        _ if reason.contains("focus") => Some("focus.mismatch"),
        _ if reason.ends_with("_timeout") => Some("timeout"),
        _ if reason.contains("no_semantics_match") || reason.contains("no_match") => {
            Some("selector.not_found")
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct DockDragRuntimeState {
    dragging: bool,
    source_window: AppWindowId,
    current_window: AppWindowId,
    moving_window: Option<AppWindowId>,
    window_under_moving_window: Option<AppWindowId>,
    window_under_moving_window_source: fret_runtime::WindowUnderCursorSource,
    transparent_payload_applied: bool,
    transparent_payload_mouse_passthrough_applied: bool,
    window_under_cursor_source: fret_runtime::WindowUnderCursorSource,
}

fn dock_drag_pointer_id_best_effort(
    app: &fret_app::App,
    known_windows: &[AppWindowId],
) -> Option<PointerId> {
    if let Some(pointer_id) = app.find_drag_pointer_id(|d| {
        (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && d.dragging
    }) {
        return Some(pointer_id);
    }

    let store = app.global::<fret_runtime::WindowInteractionDiagnosticsStore>()?;
    for window in known_windows.iter().rev().copied() {
        let docking = store.docking_latest_for_window(window)?;
        if let Some(drag) = docking.dock_drag
            && drag.dragging
        {
            // `docking_latest_for_window` is intentionally stable across frames, which makes it
            // useful for debugging but also means it can be stale. Only treat it as authoritative
            // when the drag session is still present in the live `App` drag registry.
            if app.drag(drag.pointer_id).is_some() {
                return Some(drag.pointer_id);
            }
        }
    }

    None
}

fn dock_drag_runtime_state(
    app: &fret_app::App,
    known_windows: &[AppWindowId],
) -> Option<DockDragRuntimeState> {
    if let Some(pointer_id) = dock_drag_pointer_id_best_effort(app, known_windows)
        && let Some(drag) = app.drag(pointer_id)
    {
        return Some(DockDragRuntimeState {
            dragging: drag.dragging,
            source_window: drag.source_window,
            current_window: drag.current_window,
            moving_window: drag.moving_window,
            window_under_moving_window: drag.window_under_moving_window,
            window_under_moving_window_source: drag.window_under_moving_window_source,
            transparent_payload_applied: drag.transparent_payload_applied,
            transparent_payload_mouse_passthrough_applied: drag
                .transparent_payload_mouse_passthrough_applied,
            window_under_cursor_source: drag.window_under_cursor_source,
        });
    }

    // If the drag session cannot be found in `App`, treat it as inactive. The per-window docking
    // diagnostics store may retain stale "latest" snapshots across frames (by design), which is
    // useful for debugging but unsuitable as a source of truth for scripted gates.
    None
}

fn dock_drag_window_under_cursor_source_is(
    have: fret_runtime::WindowUnderCursorSource,
    want: &str,
) -> bool {
    use fret_runtime::WindowUnderCursorSource as Src;
    match want {
        "platform" => matches!(have, Src::PlatformWin32 | Src::PlatformMacos),
        "platform_win32" => matches!(have, Src::PlatformWin32),
        "platform_macos" => matches!(have, Src::PlatformMacos),
        "latched" => matches!(have, Src::Latched),
        "heuristic" => matches!(have, Src::HeuristicZOrder | Src::HeuristicRects),
        "heuristic_z_order" => matches!(have, Src::HeuristicZOrder),
        "heuristic_rects" => matches!(have, Src::HeuristicRects),
        "unknown" => matches!(have, Src::Unknown),
        _ => false,
    }
}

fn eval_predicate_without_semantics(
    window: AppWindowId,
    known_windows: &[AppWindowId],
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    pred: &UiPredicateV1,
) -> Option<bool> {
    match pred {
        UiPredicateV1::KnownWindowCountGe { n } => Some((known_windows.len() as u32) >= *n),
        UiPredicateV1::KnownWindowCountIs { n } => Some((known_windows.len() as u32) == *n),
        UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => Some(
            platform_caps.is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str()),
        ),
        UiPredicateV1::DockDragCurrentWindowIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            Some(
                dock_drag_runtime
                    .is_some_and(|drag| drag.dragging && drag.current_window == target_window),
            )
        }
        UiPredicateV1::DockDragMovingWindowIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            Some(
                dock_drag_runtime
                    .is_some_and(|drag| drag.dragging && drag.moving_window == Some(target_window)),
            )
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            Some(dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.window_under_moving_window == Some(target_window)
            }))
        }
        UiPredicateV1::DockDragActiveIs { active } => {
            Some(dock_drag_runtime.is_some_and(|drag| drag.dragging) == *active)
        }
        UiPredicateV1::DockDragTransparentPayloadAppliedIs { applied } => Some(
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && drag.transparent_payload_applied == *applied)
                || (!*applied && dock_drag_runtime.is_none()),
        ),
        UiPredicateV1::DockDragTransparentPayloadMousePassthroughAppliedIs { applied } => Some(
            dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.transparent_payload_mouse_passthrough_applied == *applied
            }) || (!*applied && dock_drag_runtime.is_none()),
        ),
        UiPredicateV1::DockDragWindowUnderCursorSourceIs { source } => {
            Some(dock_drag_runtime.is_some_and(|drag| {
                dock_drag_window_under_cursor_source_is(drag.window_under_cursor_source, source)
            }))
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowSourceIs { source } => {
            Some(dock_drag_runtime.is_some_and(|drag| {
                dock_drag_window_under_cursor_source_is(
                    drag.window_under_moving_window_source,
                    source,
                )
            }))
        }
        UiPredicateV1::DockFloatingDragActiveIs { active } => {
            Some(match docking.and_then(|d| d.floating_drag) {
                Some(drag) => drag.activated == *active,
                None => !*active,
            })
        }
        UiPredicateV1::DockDropPreviewKindIs { preview_kind } => {
            let preview = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.preview.as_ref())?;
            let have = match preview.kind {
                fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary => "wrap_binary",
                fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit { .. } => {
                    "insert_into_split"
                }
            };
            Some(have == preview_kind.as_str())
        }
        UiPredicateV1::DockDropResolveSourceIs { source } => {
            let resolve = docking.and_then(|d| d.dock_drop_resolve.as_ref())?;
            let have = match resolve.source {
                fret_runtime::DockDropResolveSource::InvertDocking => "invert_docking",
                fret_runtime::DockDropResolveSource::OutsideWindow => "outside_window",
                fret_runtime::DockDropResolveSource::FloatZone => "float_zone",
                fret_runtime::DockDropResolveSource::EmptyDockSpace => "empty_dock_space",
                fret_runtime::DockDropResolveSource::LayoutBoundsMiss => "layout_bounds_miss",
                fret_runtime::DockDropResolveSource::LatchedPreviousHover => {
                    "latched_previous_hover"
                }
                fret_runtime::DockDropResolveSource::TabBar => "tab_bar",
                fret_runtime::DockDropResolveSource::FloatingTitleBar => "floating_title_bar",
                fret_runtime::DockDropResolveSource::OuterHintRect => "outer_hint_rect",
                fret_runtime::DockDropResolveSource::InnerHintRect => "inner_hint_rect",
                fret_runtime::DockDropResolveSource::None => "none",
            };
            Some(have == source.as_str())
        }
        UiPredicateV1::DockDropResolvedIsSome { some } => Some(
            docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .is_some_and(|d| d.resolved.is_some() == *some),
        ),
        UiPredicateV1::DockGraphCanonicalIs { canonical } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.canonical_ok == *canonical),
        ),
        UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.has_nested_same_axis_splits == *has_nested),
        ),
        UiPredicateV1::DockGraphNodeCountLe { max } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.node_count <= *max),
        ),
        UiPredicateV1::DockGraphMaxSplitDepthLe { max } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.max_split_depth <= *max),
        ),
        UiPredicateV1::DockGraphSignatureIs { signature } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.signature == *signature),
        ),
        UiPredicateV1::DockGraphSignatureContains { needle } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.signature.contains(needle)),
        ),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        ),
        _ => None,
    }
}

fn eval_predicate(
    snapshot: &fret_core::SemanticsSnapshot,
    window_bounds: Rect,
    window: AppWindowId,
    input_ctx: Option<&fret_runtime::InputContext>,
    element_runtime: Option<&ElementRuntime>,
    text_input_snapshot: Option<&fret_runtime::WindowTextInputSnapshot>,
    render_text: Option<fret_core::RendererTextPerfSnapshot>,
    render_text_font_trace: Option<&fret_core::RendererTextFontTraceSnapshot>,
    known_windows: &[AppWindowId],
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    pred: &UiPredicateV1,
) -> bool {
    match pred {
        UiPredicateV1::Exists { target } => {
            select_semantics_node(snapshot, window, element_runtime, target).is_some()
        }
        UiPredicateV1::NotExists { target } => {
            select_semantics_node(snapshot, window, element_runtime, target).is_none()
        }
        UiPredicateV1::FocusIs { target } => {
            let Some(focus) = snapshot.focus else {
                return false;
            };
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.id == focus
        }
        UiPredicateV1::RoleIs { target, role } => {
            let Some(want) = parse_semantics_role(role) else {
                return false;
            };
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.role == want
        }
        UiPredicateV1::CheckedIs { target, checked } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.checked == Some(*checked)
        }
        UiPredicateV1::SelectedIs { target, selected } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.selected == *selected
        }
        UiPredicateV1::TextCompositionIs { target, composing } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.text_composition.is_some() == *composing
        }
        UiPredicateV1::ImeCursorAreaIsSome { is_some } => {
            text_input_snapshot
                .and_then(|snapshot| snapshot.ime_cursor_area)
                .is_some()
                == *is_some
        }
        UiPredicateV1::ImeCursorAreaWithinWindow {
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(area) = text_input_snapshot.and_then(|snapshot| snapshot.ime_cursor_area)
            else {
                return false;
            };

            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let window_top = window_bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0
                - pad
                - pad_insets.right_px.max(0.0);
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0
                - pad
                - pad_insets.bottom_px.max(0.0);

            let area_left = area.origin.x.0;
            let area_top = area.origin.y.0;
            let area_right = area.origin.x.0 + area.size.width.0.max(0.0);
            let area_bottom = area.origin.y.0 + area.size.height.0.max(0.0);

            area_left >= window_left - eps
                && area_top >= window_top - eps
                && area_right <= window_right + eps
                && area_bottom <= window_bottom + eps
        }
        UiPredicateV1::ImeCursorAreaMinSize {
            min_w_px,
            min_h_px,
            eps_px,
        } => {
            let Some(area) = text_input_snapshot.and_then(|snapshot| snapshot.ime_cursor_area)
            else {
                return false;
            };

            let eps = eps_px.max(0.0);
            let min_w = min_w_px.max(0.0);
            let min_h = min_h_px.max(0.0);

            area.size.width.0.max(0.0) + eps >= min_w && area.size.height.0.max(0.0) + eps >= min_h
        }
        UiPredicateV1::CheckedIsNone { target } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.checked.is_none()
        }
        UiPredicateV1::ActiveItemIs { container, item } => {
            let Some(item_node) = select_semantics_node(snapshot, window, element_runtime, item)
            else {
                return false;
            };

            if snapshot.focus == Some(item_node.id) {
                return true;
            }

            let Some(container_node) =
                select_semantics_node(snapshot, window, element_runtime, container)
            else {
                return false;
            };

            container_node.active_descendant == Some(item_node.id)
        }
        UiPredicateV1::ActiveItemIsNone { container } => {
            let Some(container_node) =
                select_semantics_node(snapshot, window, element_runtime, container)
            else {
                return false;
            };

            if container_node.active_descendant.is_some() {
                return false;
            }

            let Some(focus_id) = snapshot.focus else {
                return true;
            };
            let Some(focus_node) = snapshot.nodes.iter().find(|n| n.id == focus_id) else {
                return true;
            };

            focus_node.role != SemanticsRole::ListBoxOption
        }
        UiPredicateV1::BarrierRoots {
            barrier_root,
            focus_barrier_root,
            require_equal,
        } => {
            let barrier = snapshot.barrier_root.map(|n| n.data().as_ffi());
            let focus_barrier = snapshot.focus_barrier_root.map(|n| n.data().as_ffi());

            let matches_root_state = |state: UiOptionalRootStateV1, value: Option<u64>| match state
            {
                UiOptionalRootStateV1::Any => true,
                UiOptionalRootStateV1::None => value.is_none(),
                UiOptionalRootStateV1::Some => value.is_some(),
            };

            if !matches_root_state(*barrier_root, barrier) {
                return false;
            }
            if !matches_root_state(*focus_barrier_root, focus_barrier) {
                return false;
            }

            match require_equal {
                None => true,
                Some(true) => barrier == focus_barrier,
                Some(false) => barrier != focus_barrier,
            }
        }
        UiPredicateV1::RenderTextMissingGlyphsIs { missing_glyphs } => {
            render_text.is_some_and(|snapshot| snapshot.frame_missing_glyphs == *missing_glyphs)
        }
        UiPredicateV1::RenderTextFontTraceCapturedWhenMissingGlyphs => {
            let Some(perf) = render_text else {
                return false;
            };
            if perf.frame_missing_glyphs == 0 {
                return true;
            }

            let Some(trace) = render_text_font_trace else {
                return false;
            };
            trace
                .entries
                .iter()
                .any(|e| e.missing_glyphs > 0 && !e.families.is_empty())
        }
        UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
            text_font_stack_key_stable_frames >= *stable_frames
        }
        UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
        UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
        UiPredicateV1::RunnerAccessibilityActivated => false,
        UiPredicateV1::VisibleInWindow { target } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            rects_intersect(node.bounds, window_bounds)
        }
        UiPredicateV1::BoundsWithinWindow {
            target,
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            let bounds = node.bounds;
            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let window_top = window_bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0
                - pad
                - pad_insets.right_px.max(0.0);
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0
                - pad
                - pad_insets.bottom_px.max(0.0);

            let node_left = bounds.origin.x.0;
            let node_top = bounds.origin.y.0;
            let node_right = bounds.origin.x.0 + bounds.size.width.0;
            let node_bottom = bounds.origin.y.0 + bounds.size.height.0;

            node_left >= window_left - eps
                && node_top >= window_top - eps
                && node_right <= window_right + eps
                && node_bottom <= window_bottom + eps
        }
        UiPredicateV1::TextInputImeCursorAreaWithinWindow {
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(text_input_snapshot) = text_input_snapshot else {
                return false;
            };
            let Some(cursor_area) = text_input_snapshot.ime_cursor_area else {
                return false;
            };
            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let window_top = window_bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0
                - pad
                - pad_insets.right_px.max(0.0);
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0
                - pad
                - pad_insets.bottom_px.max(0.0);

            let area_left = cursor_area.origin.x.0;
            let area_top = cursor_area.origin.y.0;
            let area_right = cursor_area.origin.x.0 + cursor_area.size.width.0;
            let area_bottom = cursor_area.origin.y.0 + cursor_area.size.height.0;

            area_left >= window_left - eps
                && area_top >= window_top - eps
                && area_right <= window_right + eps
                && area_bottom <= window_bottom + eps
        }
        UiPredicateV1::BoundsMinSize {
            target,
            min_w_px,
            min_h_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };

            let w = node.bounds.size.width.0.max(0.0);
            let h = node.bounds.size.height.0.max(0.0);

            let min_w = min_w_px.max(0.0);
            let min_h = min_h_px.max(0.0);
            let eps = eps_px.max(0.0);

            w + eps >= min_w && h + eps >= min_h
        }
        UiPredicateV1::BoundsMaxSize {
            target,
            max_w_px,
            max_h_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };

            let w = node.bounds.size.width.0.max(0.0);
            let h = node.bounds.size.height.0.max(0.0);

            let max_w = max_w_px.max(0.0);
            let max_h = max_h_px.max(0.0);
            let eps = eps_px.max(0.0);

            w <= max_w + eps && h <= max_h + eps
        }
        UiPredicateV1::BoundsApproxEqual { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax = a.bounds.origin.x.0;
            let ay = a.bounds.origin.y.0;
            let aw = a.bounds.size.width.0.max(0.0);
            let ah = a.bounds.size.height.0.max(0.0);

            let bx = b.bounds.origin.x.0;
            let by = b.bounds.origin.y.0;
            let bw = b.bounds.size.width.0.max(0.0);
            let bh = b.bounds.size.height.0.max(0.0);

            (ax - bx).abs() <= eps
                && (ay - by).abs() <= eps
                && (aw - bw).abs() <= eps
                && (ah - bh).abs() <= eps
        }
        UiPredicateV1::BoundsCenterApproxEqual { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax = a.bounds.origin.x.0;
            let ay = a.bounds.origin.y.0;
            let aw = a.bounds.size.width.0.max(0.0);
            let ah = a.bounds.size.height.0.max(0.0);
            let acx = ax + aw * 0.5;
            let acy = ay + ah * 0.5;

            let bx = b.bounds.origin.x.0;
            let by = b.bounds.origin.y.0;
            let bw = b.bounds.size.width.0.max(0.0);
            let bh = b.bounds.size.height.0.max(0.0);
            let bcx = bx + bw * 0.5;
            let bcy = by + bh * 0.5;

            (acx - bcx).abs() <= eps && (acy - bcy).abs() <= eps
        }
        UiPredicateV1::BoundsNonOverlapping { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ay0 = a.bounds.origin.y.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let by0 = b.bounds.origin.y.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);

            !(overlap_w > eps && overlap_h > eps)
        }
        UiPredicateV1::BoundsOverlapping { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ay0 = a.bounds.origin.y.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let by0 = b.bounds.origin.y.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);

            overlap_w > eps && overlap_h > eps
        }
        UiPredicateV1::BoundsOverlappingX { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            overlap_w > eps
        }
        UiPredicateV1::BoundsOverlappingY { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ay0 = a.bounds.origin.y.0;
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let by0 = b.bounds.origin.y.0;
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);
            overlap_h > eps
        }
        UiPredicateV1::KnownWindowCountGe { n } => (known_windows.len() as u32) >= *n,
        UiPredicateV1::KnownWindowCountIs { n } => (known_windows.len() as u32) == *n,
        UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => {
            if let Some(input_ctx) = input_ctx {
                input_ctx.caps.ui.window_hover_detection.as_str() == quality.as_str()
            } else {
                platform_caps
                    .is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str())
            }
        }
        UiPredicateV1::DockDragCurrentWindowIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && drag.current_window == target_window)
        }
        UiPredicateV1::DockDragMovingWindowIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && drag.moving_window == Some(target_window))
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.window_under_moving_window == Some(target_window)
            })
        }
        UiPredicateV1::DockDragActiveIs { active } => {
            let dragging = dock_drag_runtime.is_some_and(|drag| drag.dragging);
            dragging == *active
        }
        UiPredicateV1::DockDragTransparentPayloadAppliedIs { applied } => {
            if let Some(drag) = dock_drag_runtime {
                return drag.dragging && drag.transparent_payload_applied == *applied;
            }
            !*applied
        }
        UiPredicateV1::DockDragTransparentPayloadMousePassthroughAppliedIs { applied } => {
            if let Some(drag) = dock_drag_runtime {
                return drag.dragging
                    && drag.transparent_payload_mouse_passthrough_applied == *applied;
            }
            !*applied
        }
        UiPredicateV1::DockDragWindowUnderCursorSourceIs { source } => {
            let Some(drag) = dock_drag_runtime else {
                return false;
            };
            dock_drag_window_under_cursor_source_is(
                drag.window_under_cursor_source,
                source.as_str(),
            )
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowSourceIs { source } => {
            let Some(drag) = dock_drag_runtime else {
                return false;
            };
            dock_drag_window_under_cursor_source_is(
                drag.window_under_moving_window_source,
                source.as_str(),
            )
        }
        UiPredicateV1::DockFloatingDragActiveIs { active } => {
            match docking.and_then(|d| d.floating_drag) {
                Some(drag) => drag.activated == *active,
                None => !*active,
            }
        }
        UiPredicateV1::DockDropPreviewKindIs { preview_kind } => {
            let Some(preview) = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.preview.as_ref())
            else {
                return false;
            };
            let have = match preview.kind {
                fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary => "wrap_binary",
                fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit { .. } => {
                    "insert_into_split"
                }
            };
            have == preview_kind.as_str()
        }
        UiPredicateV1::DockDropResolveSourceIs { source } => {
            let Some(resolve) = docking.and_then(|d| d.dock_drop_resolve.as_ref()) else {
                return false;
            };
            let have = match resolve.source {
                fret_runtime::DockDropResolveSource::InvertDocking => "invert_docking",
                fret_runtime::DockDropResolveSource::OutsideWindow => "outside_window",
                fret_runtime::DockDropResolveSource::FloatZone => "float_zone",
                fret_runtime::DockDropResolveSource::EmptyDockSpace => "empty_dock_space",
                fret_runtime::DockDropResolveSource::LayoutBoundsMiss => "layout_bounds_miss",
                fret_runtime::DockDropResolveSource::LatchedPreviousHover => {
                    "latched_previous_hover"
                }
                fret_runtime::DockDropResolveSource::TabBar => "tab_bar",
                fret_runtime::DockDropResolveSource::FloatingTitleBar => "floating_title_bar",
                fret_runtime::DockDropResolveSource::OuterHintRect => "outer_hint_rect",
                fret_runtime::DockDropResolveSource::InnerHintRect => "inner_hint_rect",
                fret_runtime::DockDropResolveSource::None => "none",
            };
            have == source.as_str()
        }
        UiPredicateV1::DockDropResolvedIsSome { some } => docking
            .and_then(|d| d.dock_drop_resolve.as_ref())
            .is_some_and(|d| d.resolved.is_some() == *some),
        UiPredicateV1::DockGraphCanonicalIs { canonical } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.canonical_ok == *canonical),
        UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.has_nested_same_axis_splits == *has_nested),
        UiPredicateV1::DockGraphNodeCountLe { max } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.node_count <= *max),
        UiPredicateV1::DockGraphMaxSplitDepthLe { max } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.max_split_depth <= *max),
        UiPredicateV1::DockGraphSignatureIs { signature } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.signature == *signature),
        UiPredicateV1::DockGraphSignatureContains { needle } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.signature.contains(needle)),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        UiPredicateV1::EventKindSeen { event_kind: _ } => false,
    }
}

fn resolve_window_target_from_known_windows(
    current_window: AppWindowId,
    known_windows: &[AppWindowId],
    target: UiWindowTargetV1,
) -> Option<AppWindowId> {
    let first_seen = known_windows
        .iter()
        .copied()
        .min_by_key(|w| w.data().as_ffi());
    let last_seen = known_windows
        .iter()
        .copied()
        .max_by_key(|w| w.data().as_ffi());
    match target {
        UiWindowTargetV1::Current => Some(current_window),
        UiWindowTargetV1::FirstSeen => first_seen,
        UiWindowTargetV1::FirstSeenOther => known_windows
            .iter()
            .copied()
            .filter(|w| *w != current_window)
            .min_by_key(|w| w.data().as_ffi()),
        UiWindowTargetV1::LastSeen => last_seen,
        UiWindowTargetV1::LastSeenOther => known_windows
            .iter()
            .copied()
            .filter(|w| *w != current_window)
            .max_by_key(|w| w.data().as_ffi()),
        UiWindowTargetV1::WindowFfi { window } => {
            let want = AppWindowId::from(KeyData::from_ffi(window));
            known_windows.contains(&want).then_some(want)
        }
    }
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0.max(0.0);
    let ay1 = ay0 + a.size.height.0.max(0.0);

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0.max(0.0);
    let by1 = by0 + b.size.height.0.max(0.0);

    ax1 > bx0 && bx1 > ax0 && ay1 > by0 && by1 > ay0
}

fn center_of_rect(rect: Rect) -> Point {
    let x = rect.origin.x + rect.size.width * 0.5;
    let y = rect.origin.y + rect.size.height * 0.5;
    Point::new(x, y)
}

fn center_of_rect_clamped_to_rect(rect: Rect, clamp: Rect) -> Point {
    if !rects_intersect(rect, clamp) {
        return center_of_rect(rect);
    }

    let rx0 = rect.origin.x.0;
    let ry0 = rect.origin.y.0;
    let rx1 = rx0 + rect.size.width.0.max(0.0);
    let ry1 = ry0 + rect.size.height.0.max(0.0);

    let cx0 = clamp.origin.x.0;
    let cy0 = clamp.origin.y.0;
    let cx1 = cx0 + clamp.size.width.0.max(0.0);
    let cy1 = cy0 + clamp.size.height.0.max(0.0);

    let ix0 = rx0.max(cx0);
    let iy0 = ry0.max(cy0);
    let ix1 = rx1.min(cx1);
    let iy1 = ry1.min(cy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect(rect);
    }

    Point::new(
        fret_core::Px((ix0 + ix1) * 0.5),
        fret_core::Px((iy0 + iy1) * 0.5),
    )
}

fn wheel_position_prefer_intended_hit(
    snapshot: &fret_core::SemanticsSnapshot,
    ui: &UiTree<App>,
    intended: &fret_core::SemanticsNode,
    container_bounds: Rect,
    window_bounds: Rect,
) -> Point {
    let cx0 = window_bounds.origin.x.0;
    let cy0 = window_bounds.origin.y.0;
    let cx1 = cx0 + window_bounds.size.width.0.max(0.0);
    let cy1 = cy0 + window_bounds.size.height.0.max(0.0);

    let bx0 = container_bounds.origin.x.0;
    let by0 = container_bounds.origin.y.0;
    let bx1 = bx0 + container_bounds.size.width.0.max(0.0);
    let by1 = by0 + container_bounds.size.height.0.max(0.0);

    let ix0 = bx0.max(cx0);
    let iy0 = by0.max(cy0);
    let ix1 = bx1.min(cx1);
    let iy1 = by1.min(cy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect(container_bounds);
    }

    let w = (ix1 - ix0).max(0.0);
    let h = (iy1 - iy0).max(0.0);
    let pad_x = 8.0f32.min(w * 0.5);
    let pad_y = 8.0f32.min(h * 0.5);

    let x_mid = (ix0 + ix1) * 0.5;
    let y_mid = (iy0 + iy1) * 0.5;

    let candidates = [
        Point::new(fret_core::Px(x_mid), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(y_mid)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy0 + pad_y)),
        Point::new(fret_core::Px(x_mid), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix0 + pad_x), fret_core::Px(iy1 - pad_y)),
        Point::new(fret_core::Px(ix1 - pad_x), fret_core::Px(iy1 - pad_y)),
    ];

    for pos in candidates {
        if let Some(hit) = pick_semantics_node_at(snapshot, ui, pos)
            && hit.id.data().as_ffi() == intended.id.data().as_ffi()
        {
            return pos;
        }
    }

    candidates[0]
}

fn parse_semantics_numeric_value(value: &str) -> Option<f32> {
    let s = value.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(raw) = s.strip_suffix('%') {
        return raw.trim().parse::<f32>().ok();
    }
    if let Ok(v) = s.parse::<f32>() {
        return Some(v);
    }

    // Best-effort: extract the first float-ish token from the string.
    let mut token = String::new();
    let mut started = false;
    for ch in s.chars() {
        let keep = ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+');
        if keep {
            token.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    if token.is_empty() {
        return None;
    }
    token.parse::<f32>().ok()
}

fn move_pointer_event(position: Point) -> Event {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    })
}

fn wheel_event(position: Point, delta_x: f32, delta_y: f32) -> Event {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    Event::Pointer(PointerEvent::Wheel {
        pointer_id,
        position,
        delta: Point::new(fret_core::Px(delta_x), fret_core::Px(delta_y)),
        modifiers,
        pointer_type,
    })
}

fn click_events(position: Point, button: UiMouseButtonV1, click_count: u8) -> [Event; 3] {
    click_events_with_modifiers(position, button, click_count, Modifiers::default())
}

fn click_events_with_modifiers(
    position: Point,
    button: UiMouseButtonV1,
    click_count: u8,
    modifiers: Modifiers,
) -> [Event; 3] {
    let pointer_id = PointerId(0);
    let pointer_type = PointerType::Mouse;
    let click_count = click_count.max(1);

    let move_event = Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    });
    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };
    let down = Event::Pointer(PointerEvent::Down {
        pointer_id,
        position,
        button,
        modifiers,
        click_count,
        pointer_type,
    });
    let up = Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button,
        modifiers,
        is_click: true,
        click_count,
        pointer_type,
    });

    [move_event, down, up]
}

fn drag_events(start: Point, end: Point, button: UiMouseButtonV1, steps: u32) -> Vec<Event> {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };

    let mut pressed_buttons = MouseButtons::default();
    match button {
        MouseButton::Left => pressed_buttons.left = true,
        MouseButton::Right => pressed_buttons.right = true,
        MouseButton::Middle => pressed_buttons.middle = true,
        _ => {}
    }

    let mut out = Vec::with_capacity(3 + steps as usize);
    out.push(Event::Pointer(PointerEvent::Move {
        pointer_id,
        position: start,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    }));
    out.push(Event::Pointer(PointerEvent::Down {
        pointer_id,
        position: start,
        button,
        modifiers,
        click_count: 1,
        pointer_type,
    }));

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let x = start.x.0 + (end.x.0 - start.x.0) * t;
        let y = start.y.0 + (end.y.0 - start.y.0) * t;
        let position = Point::new(fret_core::Px(x), fret_core::Px(y));
        out.push(Event::Pointer(PointerEvent::Move {
            pointer_id,
            position,
            buttons: pressed_buttons,
            modifiers,
            pointer_type,
        }));

        // For scripted diagnostics, also emit `InternalDrag` events during pointer drags. The
        // runtime routes these to the active internal-drag anchor when a cross-window drag session
        // is active (e.g. docking tear-off / drop indicators).
        //
        // This is intentionally safe for generic scripts: `UiTree` ignores `InternalDrag` events
        // unless `app.drag(pointer_id)` exists and is marked `cross_window_hover`.
        out.push(Event::InternalDrag(fret_core::InternalDragEvent {
            pointer_id,
            position,
            kind: fret_core::InternalDragKind::Over,
            modifiers,
        }));
    }

    out.push(Event::Pointer(PointerEvent::Up {
        pointer_id,
        position: end,
        button,
        modifiers,
        is_click: false,
        click_count: 1,
        pointer_type,
    }));

    // Mirror the runner's "mouse-up routes a drop then clears hover" behavior for internal drags.
    out.push(Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position: end,
        kind: fret_core::InternalDragKind::Drop,
        modifiers,
    }));
    out
}

fn pointer_move_with_internal_over_events(button: UiMouseButtonV1, position: Point) -> [Event; 2] {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    let pressed_buttons = match button {
        UiMouseButtonV1::Left => MouseButtons {
            left: true,
            ..Default::default()
        },
        UiMouseButtonV1::Right => MouseButtons {
            right: true,
            ..Default::default()
        },
        UiMouseButtonV1::Middle => MouseButtons {
            middle: true,
            ..Default::default()
        },
    };

    let move_event = Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: pressed_buttons,
        modifiers,
        pointer_type,
    });
    let over = Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position,
        kind: fret_core::InternalDragKind::Over,
        modifiers,
    });
    [move_event, over]
}

fn pointer_up_with_internal_drop_events(button: UiMouseButtonV1, position: Point) -> [Event; 2] {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };

    let up = Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button,
        modifiers,
        is_click: false,
        click_count: 1,
        pointer_type,
    });
    let drop = Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position,
        kind: fret_core::InternalDragKind::Drop,
        modifiers,
    });
    [up, drop]
}

fn push_drag_playback_frame(state: &mut V2DragPointerState, events: &mut Vec<Event>) -> bool {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    let (button, pressed_buttons) = match state.button {
        UiMouseButtonV1::Left => (
            MouseButton::Left,
            MouseButtons {
                left: true,
                ..Default::default()
            },
        ),
        UiMouseButtonV1::Right => (
            MouseButton::Right,
            MouseButtons {
                right: true,
                ..Default::default()
            },
        ),
        UiMouseButtonV1::Middle => (
            MouseButton::Middle,
            MouseButtons {
                middle: true,
                ..Default::default()
            },
        ),
    };

    let steps = state.steps.max(1);
    let final_frame = steps.saturating_add(1);

    match state.frame {
        0 => {
            events.push(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position: state.start,
                buttons: MouseButtons::default(),
                modifiers,
                pointer_type,
            }));
            events.push(Event::Pointer(PointerEvent::Down {
                pointer_id,
                position: state.start,
                button,
                modifiers,
                click_count: 1,
                pointer_type,
            }));
            state.frame = 1;
            false
        }
        f if (1..=steps).contains(&f) => {
            let t = f as f32 / steps as f32;
            let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
            let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
            let position = Point::new(fret_core::Px(x), fret_core::Px(y));
            events.push(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position,
                buttons: pressed_buttons,
                modifiers,
                pointer_type,
            }));
            events.push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id,
                position,
                kind: fret_core::InternalDragKind::Over,
                modifiers,
            }));
            state.frame = state.frame.saturating_add(1);
            false
        }
        f if f >= final_frame => {
            events.push(Event::Pointer(PointerEvent::Up {
                pointer_id,
                position: state.end,
                button,
                modifiers,
                is_click: false,
                click_count: 1,
                pointer_type,
            }));
            events.push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id,
                position: state.end,
                kind: fret_core::InternalDragKind::Drop,
                modifiers,
            }));
            true
        }
        _ => true,
    }
}

fn drag_playback_last_position(state: &V2DragPointerState) -> Point {
    let steps = state.steps.max(1);
    let final_frame = steps.saturating_add(1);

    match state.frame {
        0 | 1 => state.start,
        f if (2..=final_frame).contains(&f) => {
            let move_frame = (f - 1).min(steps);
            let t = move_frame as f32 / steps as f32;
            let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
            let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
            Point::new(fret_core::Px(x), fret_core::Px(y))
        }
        _ => state.end,
    }
}

fn write_cursor_override_window_client_logical(
    out_dir: &Path,
    window: AppWindowId,
    x_px: f32,
    y_px: f32,
) -> Result<(), std::io::Error> {
    let payload = format!(
        "schema_version=1\nkind=window_client_logical\nwindow={}\nx_px={}\ny_px={}\n",
        window.data().as_ffi(),
        x_px,
        y_px
    );
    let text_path = out_dir.join("cursor_screen_pos.override.txt");
    let trigger_path = out_dir.join("cursor_screen_pos.touch");
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(text_path, payload)?;
    touch_file(&trigger_path)?;
    Ok(())
}

fn write_mouse_buttons_override_window_v1(
    out_dir: &Path,
    window: AppWindowId,
    left: Option<bool>,
    right: Option<bool>,
    middle: Option<bool>,
) -> Result<(), std::io::Error> {
    let mut payload = format!("schema_version=1\nwindow={}\n", window.data().as_ffi());
    if let Some(v) = left {
        payload.push_str(&format!("left={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = right {
        payload.push_str(&format!("right={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = middle {
        payload.push_str(&format!("middle={}\n", if v { 1 } else { 0 }));
    }
    let text_path = out_dir.join("mouse_buttons.override.txt");
    let trigger_path = out_dir.join("mouse_buttons.touch");
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(text_path, payload)?;
    touch_file(&trigger_path)?;
    Ok(())
}

fn write_mouse_buttons_override_all_windows_v1(
    out_dir: &Path,
    left: Option<bool>,
    right: Option<bool>,
    middle: Option<bool>,
) -> Result<(), std::io::Error> {
    let mut payload = "schema_version=1\n".to_string();
    if let Some(v) = left {
        payload.push_str(&format!("left={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = right {
        payload.push_str(&format!("right={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = middle {
        payload.push_str(&format!("middle={}\n", if v { 1 } else { 0 }));
    }
    let text_path = out_dir.join("mouse_buttons.override.txt");
    let trigger_path = out_dir.join("mouse_buttons.touch");
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(text_path, payload)?;
    touch_file(&trigger_path)?;
    Ok(())
}

fn press_key_events(key: KeyCode, modifiers: UiKeyModifiersV1, repeat: bool) -> [Event; 2] {
    let modifiers = core_modifiers_from_ui(Some(modifiers));
    let down = Event::KeyDown {
        key,
        modifiers,
        repeat,
    };
    let up = Event::KeyUp { key, modifiers };
    [down, up]
}

fn core_modifiers_from_ui(modifiers: Option<UiKeyModifiersV1>) -> Modifiers {
    let modifiers = modifiers.unwrap_or_default();
    Modifiers {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        meta: modifiers.meta,
        ..Modifiers::default()
    }
}

fn ime_event_kind_name(event: &UiImeEventV1) -> &'static str {
    match event {
        UiImeEventV1::Enabled => "enabled",
        UiImeEventV1::Disabled => "disabled",
        UiImeEventV1::Commit { .. } => "commit",
        UiImeEventV1::Preedit { .. } => "preedit",
        UiImeEventV1::DeleteSurrounding { .. } => "delete_surrounding",
    }
}

fn ime_event_from_v1(event: &UiImeEventV1) -> ImeEvent {
    match event {
        UiImeEventV1::Enabled => ImeEvent::Enabled,
        UiImeEventV1::Disabled => ImeEvent::Disabled,
        UiImeEventV1::Commit { text } => ImeEvent::Commit(text.clone()),
        UiImeEventV1::Preedit { text, cursor_bytes } => ImeEvent::Preedit {
            text: text.clone(),
            cursor: cursor_bytes.map(|(a, b)| (a as usize, b as usize)),
        },
        UiImeEventV1::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } => ImeEvent::DeleteSurrounding {
            before_bytes: (*before_bytes).min(usize::MAX as u32) as usize,
            after_bytes: (*after_bytes).min(usize::MAX as u32) as usize,
        },
    }
}

fn parse_shortcut(shortcut: &str) -> Option<(KeyCode, UiKeyModifiersV1)> {
    let mut parts = shortcut
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return None;
    }

    let key = parts.pop()?;

    let mut modifiers = UiKeyModifiersV1::default();
    for part in parts {
        match part.to_ascii_lowercase().as_str() {
            "shift" => modifiers.shift = true,
            "ctrl" | "control" => modifiers.ctrl = true,
            "alt" => modifiers.alt = true,
            "meta" | "cmd" | "command" | "super" => modifiers.meta = true,
            "primary" => {
                if cfg!(target_os = "macos") {
                    modifiers.meta = true;
                } else {
                    modifiers.ctrl = true;
                }
            }
            _ => return None,
        }
    }

    Some((parse_key_code(key)?, modifiers))
}

fn rect_inset(rect: Rect, insets: UiPaddingInsetsV1) -> Rect {
    let left = Px(insets.left_px.max(0.0));
    let top = Px(insets.top_px.max(0.0));
    let right = Px(insets.right_px.max(0.0));
    let bottom = Px(insets.bottom_px.max(0.0));

    let origin = Point {
        x: rect.origin.x + left,
        y: rect.origin.y + top,
    };
    let w = (rect.size.width.0 - left.0 - right.0).max(0.0);
    let h = (rect.size.height.0 - top.0 - bottom.0).max(0.0);
    Rect {
        origin,
        size: fret_core::Size {
            width: Px(w),
            height: Px(h),
        },
    }
}

fn rect_fully_contains(outer: Rect, inner: Rect) -> bool {
    let ox0 = outer.origin.x.0;
    let oy0 = outer.origin.y.0;
    let ox1 = ox0 + outer.size.width.0;
    let oy1 = oy0 + outer.size.height.0;

    let ix0 = inner.origin.x.0;
    let iy0 = inner.origin.y.0;
    let ix1 = ix0 + inner.size.width.0;
    let iy1 = iy0 + inner.size.height.0;

    ix0 >= ox0 && iy0 >= oy0 && ix1 <= ox1 && iy1 <= oy1
}

fn parse_key_code(key: &str) -> Option<KeyCode> {
    let key = key.trim().to_ascii_lowercase();
    match key.as_str() {
        "shift" => Some(KeyCode::ShiftLeft),
        "ctrl" | "control" => Some(KeyCode::ControlLeft),
        "alt" | "option" => Some(KeyCode::AltLeft),
        "meta" | "super" | "cmd" | "command" => Some(KeyCode::MetaLeft),
        "escape" | "esc" => Some(KeyCode::Escape),
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "space" => Some(KeyCode::Space),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f4" => Some(KeyCode::F4),
        "f5" => Some(KeyCode::F5),
        "f6" => Some(KeyCode::F6),
        "f7" => Some(KeyCode::F7),
        "f8" => Some(KeyCode::F8),
        "f9" => Some(KeyCode::F9),
        "f10" => Some(KeyCode::F10),
        "f11" => Some(KeyCode::F11),
        "f12" => Some(KeyCode::F12),
        "arrow_up" | "up" => Some(KeyCode::ArrowUp),
        "arrow_down" | "down" => Some(KeyCode::ArrowDown),
        "arrow_left" | "left" => Some(KeyCode::ArrowLeft),
        "arrow_right" | "right" => Some(KeyCode::ArrowRight),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "page_up" => Some(KeyCode::PageUp),
        "page_down" => Some(KeyCode::PageDown),
        _ => {
            if key.len() == 1 {
                return Some(match key.as_bytes()[0] {
                    b'a' => KeyCode::KeyA,
                    b'b' => KeyCode::KeyB,
                    b'c' => KeyCode::KeyC,
                    b'd' => KeyCode::KeyD,
                    b'e' => KeyCode::KeyE,
                    b'f' => KeyCode::KeyF,
                    b'g' => KeyCode::KeyG,
                    b'h' => KeyCode::KeyH,
                    b'i' => KeyCode::KeyI,
                    b'j' => KeyCode::KeyJ,
                    b'k' => KeyCode::KeyK,
                    b'l' => KeyCode::KeyL,
                    b'm' => KeyCode::KeyM,
                    b'n' => KeyCode::KeyN,
                    b'o' => KeyCode::KeyO,
                    b'p' => KeyCode::KeyP,
                    b'q' => KeyCode::KeyQ,
                    b'r' => KeyCode::KeyR,
                    b's' => KeyCode::KeyS,
                    b't' => KeyCode::KeyT,
                    b'u' => KeyCode::KeyU,
                    b'v' => KeyCode::KeyV,
                    b'w' => KeyCode::KeyW,
                    b'x' => KeyCode::KeyX,
                    b'y' => KeyCode::KeyY,
                    b'z' => KeyCode::KeyZ,
                    b'0' => KeyCode::Digit0,
                    b'1' => KeyCode::Digit1,
                    b'2' => KeyCode::Digit2,
                    b'3' => KeyCode::Digit3,
                    b'4' => KeyCode::Digit4,
                    b'5' => KeyCode::Digit5,
                    b'6' => KeyCode::Digit6,
                    b'7' => KeyCode::Digit7,
                    b'8' => KeyCode::Digit8,
                    b'9' => KeyCode::Digit9,
                    _ => return None,
                });
            }
            None
        }
    }
}

fn key_to_u64(key: NodeId) -> u64 {
    key.data().as_ffi()
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(value).unwrap_or_default();
    std::fs::write(path, bytes)
}

fn write_json_compact<T: Serialize>(path: PathBuf, value: &T) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    std::fs::write(path, bytes)
}

fn take_last_vecdeque<T: Clone>(items: &VecDeque<T>, max: usize) -> Vec<T> {
    if max == 0 {
        return Vec::new();
    }
    let len = items.len();
    let start = len.saturating_sub(max);
    items.iter().skip(start).cloned().collect()
}

fn truncate_string_bytes(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }
    if max_bytes == 0 {
        s.clear();
        return;
    }

    let suffix = "...";
    if max_bytes <= suffix.len() {
        let mut idx = max_bytes;
        while idx > 0 && !s.is_char_boundary(idx) {
            idx -= 1;
        }
        s.truncate(idx);
        return;
    }

    let mut idx = max_bytes - suffix.len();
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    s.truncate(idx);
    s.push_str(suffix);
}

fn truncate_opt_string_bytes(s: &mut Option<String>, max_bytes: usize) {
    let Some(v) = s.as_mut() else {
        return;
    };
    truncate_string_bytes(v, max_bytes);
}

fn truncate_vec_string_bytes(items: &mut Vec<String>, max_bytes: usize) {
    for s in items {
        truncate_string_bytes(s, max_bytes);
    }
}

fn write_latest_pointer(out_dir: &Path, export_dir: &Path) -> Result<(), std::io::Error> {
    let path = out_dir.join("latest.txt");
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let rel = export_dir.strip_prefix(out_dir).unwrap_or(export_dir);
    std::fs::write(path, rel.to_string_lossy().as_bytes())
}

fn touch_file(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // `fret-diag`'s filesystem transport uses a monotonically increasing stamp written into
    // `*.touch` files. `SystemTime` millisecond resolution is not sufficient on all platforms
    // (multiple writes can occur within the same millisecond), so ensure the stamp is strictly
    // increasing within the current process.
    //
    // The stamp is used only for edge detection (not for wall-clock semantics), so it's safe to
    // synthesize values above `unix_ms_now()` when needed.
    use std::sync::atomic::{AtomicU64, Ordering};
    static LAST_TOUCH_STAMP: AtomicU64 = AtomicU64::new(0);
    let mut stamp = unix_ms_now();
    loop {
        let prev = LAST_TOUCH_STAMP.load(Ordering::Relaxed);
        let next = stamp.max(prev.saturating_add(1));
        match LAST_TOUCH_STAMP.compare_exchange_weak(
            prev,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                stamp = next;
                break;
            }
            Err(_) => continue,
        }
    }
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    writeln!(f, "{stamp}")?;
    let _ = f.flush();
    Ok(())
}

#[cfg(feature = "diagnostics-ws")]
fn build_semantics_node_get_ack_v1(
    snapshot: Option<&fret_core::SemanticsSnapshot>,
    window_ffi: u64,
    node_id: u64,
    redact_text: bool,
    max_string_bytes: usize,
) -> UiSemanticsNodeGetAckV1 {
    let captured_unix_ms = Some(unix_ms_now());

    let Some(snapshot) = snapshot else {
        return UiSemanticsNodeGetAckV1 {
            schema_version: 1,
            status: "no_semantics".to_string(),
            reason: Some("no_semantics_snapshot".to_string()),
            window: window_ffi,
            node_id,
            semantics_fingerprint: None,
            node: None,
            children: Vec::new(),
            captured_unix_ms,
        };
    };

    let semantics_fingerprint = Some(semantics_fingerprint_v1(
        snapshot,
        redact_text,
        max_string_bytes,
    ));
    let want = NodeId::from(KeyData::from_ffi(node_id));

    let Some(node) = snapshot.nodes.iter().find(|n| n.id == want) else {
        return UiSemanticsNodeGetAckV1 {
            schema_version: 1,
            status: "not_found".to_string(),
            reason: None,
            window: window_ffi,
            node_id,
            semantics_fingerprint,
            node: None,
            children: Vec::new(),
            captured_unix_ms,
        };
    };

    let exported = UiSemanticsNodeV1::from_node(node, redact_text, max_string_bytes);
    let node = serde_json::to_value(exported).ok();
    let children = snapshot
        .nodes
        .iter()
        .filter(|n| n.parent == Some(want))
        .map(|n| key_to_u64(n.id))
        .collect::<Vec<_>>();

    UiSemanticsNodeGetAckV1 {
        schema_version: 1,
        status: "ok".to_string(),
        reason: None,
        window: window_ffi,
        node_id,
        semantics_fingerprint,
        node,
        children,
        captured_unix_ms,
    }
}

fn screenshot_request_completed(path: &Path, request_id: &str, window_ffi: u64) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let Ok(root) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(completed) = root.get("completed").and_then(|v| v.as_array()) else {
        return false;
    };
    completed.iter().any(|entry| {
        entry.get("request_id").and_then(|v| v.as_str()) == Some(request_id)
            && entry.get("window").and_then(|v| v.as_u64()) == Some(window_ffi)
    })
}

#[cfg(feature = "diagnostics-ws")]
fn read_screenshot_result_entry(
    path: &Path,
    request_id: &str,
    window_ffi: u64,
) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    let root = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    let completed = root.get("completed").and_then(|v| v.as_array())?;
    completed
        .iter()
        .find(|entry| {
            entry.get("request_id").and_then(|v| v.as_str()) == Some(request_id)
                && entry.get("window").and_then(|v| v.as_u64()) == Some(window_ffi)
        })
        .cloned()
}

fn display_path(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.to_string_lossy().to_string()
}

fn maybe_redact_string(s: &str, redact_text: bool) -> String {
    if !redact_text {
        return s.to_string();
    }
    format!("<redacted len={}>", s.len())
}

fn sanitize_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
            out.push(c);
        } else if matches!(c, ' ' | ':' | '/' | '\\') {
            out.push('_');
        }
    }
    if out.is_empty() {
        "bundle".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        AppWindowId, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRole,
        SemanticsRoot, SemanticsSnapshot, Size,
    };
    use fret_diag_protocol::UiActionStepV1;
    use slotmap::KeyData;

    fn eval_predicate(
        snapshot: &fret_core::SemanticsSnapshot,
        window_bounds: Rect,
        window: AppWindowId,
        element_runtime: Option<&ElementRuntime>,
        text_input_snapshot: Option<&fret_runtime::WindowTextInputSnapshot>,
        render_text: Option<fret_core::RendererTextPerfSnapshot>,
        render_text_font_trace: Option<&fret_core::RendererTextFontTraceSnapshot>,
        known_windows: &[AppWindowId],
        docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
        text_font_stack_key_stable_frames: u32,
        font_catalog_populated: bool,
        system_font_rescan_idle: bool,
        pred: &UiPredicateV1,
    ) -> bool {
        super::eval_predicate(
            snapshot,
            window_bounds,
            window,
            element_runtime,
            None,
            text_input_snapshot,
            render_text,
            render_text_font_trace,
            known_windows,
            docking,
            text_font_stack_key_stable_frames,
            font_catalog_populated,
            system_font_rescan_idle,
            pred,
        )
    }

    #[test]
    fn parse_key_code_supports_function_keys() {
        assert_eq!(parse_key_code("f1"), Some(KeyCode::F1));
        assert_eq!(parse_key_code("f10"), Some(KeyCode::F10));
        assert_eq!(parse_key_code("F12"), Some(KeyCode::F12));
    }

    #[test]
    fn environment_snapshot_exports_committed_preferences_and_insets() {
        use fret_core::{ColorScheme, ContrastPreference, Edges, ForcedColorsMode, PointerType};

        let snapshot = fret_ui::elements::EnvironmentQueryDiagnosticsSnapshot {
            viewport_bounds: rect(0.0, 0.0, 100.0, 200.0),
            scale_factor: 2.0,
            color_scheme: Some(ColorScheme::Dark),
            prefers_reduced_motion: Some(true),
            text_scale_factor: Some(1.25),
            prefers_reduced_transparency: Some(true),
            accent_color: Some(fret_core::Color {
                r: 1.0,
                g: 0.5,
                b: 0.25,
                a: 1.0,
            }),
            contrast_preference: Some(ContrastPreference::More),
            forced_colors_mode: Some(ForcedColorsMode::Active),
            primary_pointer_type: PointerType::Touch,
            safe_area_insets: Some(Edges {
                top: Px(1.0),
                right: Px(2.0),
                bottom: Px(3.0),
                left: Px(4.0),
            }),
            occlusion_insets: None,
        };

        let exported = ElementEnvironmentSnapshotV1::from_diagnostics_snapshot(&snapshot);
        assert_eq!(exported.color_scheme.as_deref(), Some("dark"));
        assert_eq!(exported.prefers_reduced_motion, Some(true));
        assert_eq!(exported.text_scale_factor, Some(1.25));
        assert_eq!(exported.prefers_reduced_transparency, Some(true));
        assert_eq!(
            exported.accent_color,
            Some(fret_core::Color {
                r: 1.0,
                g: 0.5,
                b: 0.25,
                a: 1.0,
            })
        );
        assert_eq!(exported.contrast_preference.as_deref(), Some("more"));
        assert_eq!(exported.forced_colors_mode.as_deref(), Some("active"));
        assert_eq!(exported.primary_pointer_type.as_deref(), Some("touch"));
        assert_eq!(exported.safe_area_insets.map(|e| e.top_px), Some(1.0));
        assert_eq!(exported.safe_area_insets.map(|e| e.right_px), Some(2.0));
        assert_eq!(exported.safe_area_insets.map(|e| e.bottom_px), Some(3.0));
        assert_eq!(exported.safe_area_insets.map(|e| e.left_px), Some(4.0));
    }

    fn node_id(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn window_id(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    fn dock_node_id(id: u64) -> fret_core::DockNodeId {
        fret_core::DockNodeId::from(KeyData::from_ffi(id))
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn semantics_node(
        id: u64,
        parent: Option<u64>,
        role: SemanticsRole,
        bounds: Rect,
        label: &str,
    ) -> SemanticsNode {
        SemanticsNode {
            id: node_id(id),
            parent: parent.map(node_id),
            role,
            bounds,
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: Some(label.to_string()),
            value: None,
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
        }
    }

    #[cfg(feature = "diagnostics-ws")]
    #[test]
    fn devtools_semantics_node_get_ack_includes_node_and_children() {
        let window = window_id(1);
        let root_id = 10;
        let child_id = 11;

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(root_id),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    root_id,
                    None,
                    SemanticsRole::Group,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node(
                    child_id,
                    Some(root_id),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 10.0, 10.0),
                    "child",
                ),
            ],
        };

        let ack = build_semantics_node_get_ack_v1(
            Some(&snapshot),
            window.data().as_ffi(),
            root_id,
            false,
            4096,
        );
        assert_eq!(ack.status, "ok");
        assert_eq!(ack.window, window.data().as_ffi());
        assert_eq!(ack.node_id, root_id);
        assert_eq!(ack.children, vec![key_to_u64(node_id(child_id))]);
        assert!(ack.node.is_some());
        assert!(ack.semantics_fingerprint.is_some());

        let ack = build_semantics_node_get_ack_v1(
            Some(&snapshot),
            window.data().as_ffi(),
            999,
            false,
            4096,
        );
        assert_eq!(ack.status, "not_found");

        let ack =
            build_semantics_node_get_ack_v1(None, window.data().as_ffi(), root_id, false, 4096);
        assert_eq!(ack.status, "no_semantics");
    }

    fn semantics_node_with_test_id(
        id: u64,
        parent: Option<u64>,
        role: SemanticsRole,
        bounds: Rect,
        label: &str,
        test_id: &str,
    ) -> SemanticsNode {
        let mut n = semantics_node(id, parent, role, bounds, label);
        n.test_id = Some(test_id.to_string());
        n
    }

    #[test]
    fn scripts_do_not_force_inspection_active() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.inspect_enabled = false;
        svc.pick_armed_run_id = None;
        svc.pending_pick = None;
        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-{}", unique));
        svc.cfg.pick_trigger_path = dir.join("pick.touch");
        svc.cfg.inspect_trigger_path = dir.join("inspect.touch");
        svc.cfg.inspect_path = dir.join("inspect.json");
        svc.pending_script = Some(PendingScript { steps: Vec::new() });

        assert!(
            !svc.wants_inspection_active(AppWindowId::default()),
            "scripts should not force inspection_active (allows view cache/paint cache during perf triage)"
        );
    }

    #[test]
    fn scripts_support_reset_diagnostics_step() {
        let parsed: UiActionScriptV1 =
            serde_json::from_str(r#"{"schema_version":1,"steps":[{"type":"reset_diagnostics"}]}"#)
                .expect("parse reset_diagnostics step");
        assert_eq!(parsed.schema_version, 1);
        assert!(
            matches!(parsed.steps.as_slice(), [UiActionStepV1::ResetDiagnostics]),
            "expected reset_diagnostics step"
        );
    }

    #[test]
    fn scripts_support_schema_v2_intent_steps() {
        let parsed: UiActionScriptV2 = serde_json::from_str(
            r#"{"schema_version":2,"steps":[{"type":"ensure_visible","target":{"kind":"test_id","id":"x"}}]}"#,
        )
        .expect("parse schema v2 script");
        assert_eq!(parsed.schema_version, 2);
        assert!(
            matches!(
                parsed.steps.as_slice(),
                [UiActionStepV2::EnsureVisible { .. }]
            ),
            "expected ensure_visible step"
        );
    }

    #[test]
    fn scripts_support_move_pointer_sweep_step() {
        let parsed: UiActionScriptV2 = serde_json::from_str(
            r#"{"schema_version":2,"steps":[{"type":"move_pointer_sweep","target":{"kind":"test_id","id":"x"},"delta_x":10.0,"delta_y":-5.0,"steps":3,"frames_per_step":2}]}"#,
        )
        .expect("parse move_pointer_sweep step");
        assert_eq!(parsed.schema_version, 2);
        assert!(
            matches!(
                parsed.steps.as_slice(),
                [UiActionStepV2::MovePointerSweep { .. }]
            ),
            "expected move_pointer_sweep step"
        );
    }

    #[test]
    fn pick_trigger_is_baselined_on_first_poll() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.pick_armed_run_id = None;

        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-{}", unique));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        svc.cfg.pick_trigger_path = dir.join("pick.touch");
        std::fs::write(&svc.cfg.pick_trigger_path, []).expect("create pick.touch");

        svc.last_pick_trigger_mtime = None;
        svc.poll_pick_trigger();

        assert!(
            svc.pick_armed_run_id.is_none(),
            "the first observed pick.touch mtime should be baselined, not treated as a pick trigger"
        );
        assert!(svc.last_pick_trigger_mtime.is_some());
    }

    #[test]
    fn inspect_trigger_is_baselined_on_first_poll() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.inspect_enabled = false;

        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-{}", unique));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        svc.cfg.inspect_trigger_path = dir.join("inspect.touch");
        svc.cfg.inspect_path = dir.join("inspect.json");
        std::fs::write(&svc.cfg.inspect_trigger_path, []).expect("create inspect.touch");

        svc.last_inspect_trigger_mtime = None;
        svc.poll_inspect_trigger();

        assert!(
            !svc.inspect_enabled,
            "the first observed inspect.touch mtime should be baselined, not treated as an inspect trigger"
        );
        assert!(svc.last_inspect_trigger_mtime.is_some());
    }

    #[test]
    fn pick_by_bounds_prefers_topmost_root_z() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(3),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-a",
                ),
                semantics_node(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "a",
                ),
                semantics_node(
                    3,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-b",
                ),
                semantics_node(
                    4,
                    Some(3),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "b",
                ),
            ],
        };

        let picked = pick_semantics_node_by_bounds(&snapshot, Point::new(Px(10.0), Px(10.0)))
            .expect("expected a pick");
        assert_eq!(picked.id, node_id(4));
    }

    #[test]
    fn select_by_test_id_prefers_topmost_root_z() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(3),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-a",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "a",
                    "open",
                ),
                semantics_node(
                    3,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-b",
                ),
                semantics_node_with_test_id(
                    4,
                    Some(3),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "b",
                    "open",
                ),
            ],
        };

        let selector = UiSelectorV1::TestId {
            id: "open".to_string(),
        };
        let picked = select_semantics_node(&snapshot, window_id(1), None, &selector)
            .expect("expected a pick");
        assert_eq!(picked.id, node_id(4));

        let cfg = UiDiagnosticsConfig::default();
        let best = best_selector_for_node(&snapshot, &snapshot.nodes[1], None, &cfg)
            .expect("expected a selector");
        match best {
            UiSelectorV1::TestId { id } => assert_eq!(id, "open"),
            other => panic!("expected TestId selector, got: {other:?}"),
        }
    }

    #[test]
    fn bounds_within_window_predicate_respects_padding() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "content",
                    "content",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsWithinWindow {
            target: UiSelectorV1::TestId {
                id: "content".to_string(),
            },
            padding_px: 0.0,
            padding_insets_px: None,
            eps_px: 0.0,
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            None,
            &[],
            None,
            None,
            None,
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::BoundsWithinWindow {
            target: UiSelectorV1::TestId {
                id: "content".to_string(),
            },
            padding_px: 12.0,
            padding_insets_px: None,
            eps_px: 0.0,
        };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected padding to shrink the allowed window rect"
        );
    }

    #[test]
    fn bounds_min_size_predicate_accepts_large_enough_nodes() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node_with_test_id(
                1,
                None,
                SemanticsRole::Panel,
                rect(10.0, 10.0, 320.0, 240.0),
                "resizable",
                "ui-gallery-resizable-panels",
            )],
        };

        let pred = UiPredicateV1::BoundsMinSize {
            target: UiSelectorV1::TestId {
                id: "ui-gallery-resizable-panels".to_string(),
            },
            min_w_px: 200.0,
            min_h_px: 200.0,
            eps_px: 0.0,
        };

        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected node to satisfy the min-size gate"
        );
    }

    #[test]
    fn active_item_is_predicate_matches_focus_or_active_descendant() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let mut root = semantics_node_with_test_id(
            1,
            None,
            SemanticsRole::ListBox,
            rect(0.0, 0.0, 100.0, 100.0),
            "listbox",
            "listbox",
        );
        let mut item_a = semantics_node_with_test_id(
            2,
            Some(1),
            SemanticsRole::ListBoxOption,
            rect(0.0, 0.0, 100.0, 20.0),
            "a",
            "a",
        );
        let item_b = semantics_node_with_test_id(
            3,
            Some(1),
            SemanticsRole::ListBoxOption,
            rect(0.0, 20.0, 100.0, 20.0),
            "b",
            "b",
        );

        // Model A: roving focus (focused item is active).
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(node_id(2)),
            captured: None,
            nodes: vec![root.clone(), item_a.clone(), item_b.clone()],
        };

        let pred = UiPredicateV1::ActiveItemIs {
            container: UiSelectorV1::TestId {
                id: "listbox".to_string(),
            },
            item: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected roving focus to satisfy active_item_is"
        );

        // Model B: composite focus + active_descendant.
        root.active_descendant = Some(node_id(3));
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(node_id(1)),
            captured: None,
            nodes: vec![root, item_a, item_b],
        };

        let pred = UiPredicateV1::ActiveItemIs {
            container: UiSelectorV1::TestId {
                id: "listbox".to_string(),
            },
            item: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected active_descendant to satisfy active_item_is"
        );
    }

    #[test]
    fn active_item_is_none_predicate_matches_when_no_active_descendant_and_focus_is_not_option() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let root = semantics_node_with_test_id(
            1,
            None,
            SemanticsRole::ListBox,
            rect(0.0, 0.0, 100.0, 100.0),
            "listbox",
            "listbox",
        );
        let item_a = semantics_node_with_test_id(
            2,
            Some(1),
            SemanticsRole::ListBoxOption,
            rect(0.0, 0.0, 100.0, 20.0),
            "a",
            "a",
        );

        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(node_id(1)),
            captured: None,
            nodes: vec![root, item_a],
        };

        let pred = UiPredicateV1::ActiveItemIsNone {
            container: UiSelectorV1::TestId {
                id: "listbox".to_string(),
            },
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            None,
            None,
            None,
            0,
            false,
            true,
            &pred
        ));
    }

    #[test]
    fn dock_drop_preview_kind_predicate_reads_from_docking_diagnostics() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: Vec::new(),
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: Vec::new(),
        };

        let mut docking = fret_runtime::DockingInteractionDiagnostics::default();
        docking.dock_drop_resolve = Some(fret_runtime::DockDropResolveDiagnostics {
            pointer_id: fret_core::PointerId(1),
            position: fret_core::geometry::Point::new(Px(1.0), Px(2.0)),
            window_bounds,
            dock_bounds: window_bounds,
            source: fret_runtime::DockDropResolveSource::None,
            resolved: None,
            preview: Some(fret_runtime::DockDropPreviewDiagnostics {
                kind: fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary,
            }),
            candidates: Vec::new(),
        });

        let pred = UiPredicateV1::DockDropPreviewKindIs {
            preview_kind: "wrap_binary".to_string(),
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        docking.dock_drop_resolve = Some(fret_runtime::DockDropResolveDiagnostics {
            preview: Some(fret_runtime::DockDropPreviewDiagnostics {
                kind: fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit {
                    axis: fret_core::Axis::Horizontal,
                    split: dock_node_id(1),
                    insert_index: 1,
                },
            }),
            ..docking.dock_drop_resolve.unwrap()
        });

        let pred = UiPredicateV1::DockDropPreviewKindIs {
            preview_kind: "insert_into_split".to_string(),
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));
    }

    #[test]
    fn dock_drop_resolve_source_and_resolved_presence_predicates_read_from_docking_diagnostics() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: Vec::new(),
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: Vec::new(),
        };

        let mut docking = fret_runtime::DockingInteractionDiagnostics::default();
        docking.dock_drop_resolve = Some(fret_runtime::DockDropResolveDiagnostics {
            pointer_id: fret_core::PointerId(1),
            position: fret_core::geometry::Point::new(Px(1.0), Px(2.0)),
            window_bounds,
            dock_bounds: window_bounds,
            source: fret_runtime::DockDropResolveSource::OuterHintRect,
            resolved: None,
            preview: None,
            candidates: Vec::new(),
        });

        let pred = UiPredicateV1::DockDropResolveSourceIs {
            source: "outer_hint_rect".to_string(),
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::DockDropResolvedIsSome { some: false };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        docking.dock_drop_resolve = Some(fret_runtime::DockDropResolveDiagnostics {
            resolved: Some(fret_runtime::DockDropTargetDiagnostics {
                layout_root: dock_node_id(1),
                tabs: dock_node_id(2),
                zone: fret_core::dock::DropZone::Left,
                insert_index: None,
                outer: true,
            }),
            ..docking.dock_drop_resolve.unwrap()
        });

        let pred = UiPredicateV1::DockDropResolvedIsSome { some: true };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));
    }

    #[test]
    fn dock_graph_stats_predicates_read_from_docking_diagnostics() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: Vec::new(),
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: Vec::new(),
        };

        let docking = fret_runtime::DockingInteractionDiagnostics {
            dock_graph_stats: Some(fret_runtime::DockGraphStatsDiagnostics {
                node_count: 10,
                tabs_count: 2,
                split_count: 3,
                floating_count: 1,
                max_depth: 4,
                max_split_depth: 3,
                canonical_ok: true,
                has_nested_same_axis_splits: false,
            }),
            dock_graph_signature: Some(fret_runtime::DockGraphSignatureDiagnostics {
                signature: "dock(root=split(v,[tabs([a]),tabs([b])]);floatings=[])".to_string(),
                fingerprint64: 42,
            }),
            ..Default::default()
        };

        let pred = UiPredicateV1::DockGraphCanonicalIs { canonical: true };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested: false };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::DockGraphNodeCountLe { max: 10 };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));
        let pred = UiPredicateV1::DockGraphNodeCountLe { max: 9 };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                Some(&docking),
                0,
                false,
                true,
                &pred
            ),
            "expected node_count <= 9 to fail when snapshot reports node_count=10"
        );

        let pred = UiPredicateV1::DockGraphMaxSplitDepthLe { max: 3 };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));
        let pred = UiPredicateV1::DockGraphMaxSplitDepthLe { max: 2 };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                Some(&docking),
                0,
                false,
                true,
                &pred
            ),
            "expected max_split_depth <= 2 to fail when snapshot reports max_split_depth=3"
        );

        let pred = UiPredicateV1::DockGraphSignatureIs {
            signature: "dock(root=split(v,[tabs([a]),tabs([b])]);floatings=[])".to_string(),
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::DockGraphSignatureContains {
            needle: "tabs([a])".to_string(),
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64: 42 };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            None,
            None,
            None,
            &[],
            Some(&docking),
            0,
            false,
            true,
            &pred
        ));

        let pred = UiPredicateV1::DockGraphCanonicalIs { canonical: false };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                Some(&docking),
                0,
                false,
                true,
                &pred
            ),
            "expected canonical=false to fail when snapshot reports canonical_ok=true"
        );
    }

    #[test]
    fn bounds_min_size_predicate_rejects_collapsed_nodes() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node_with_test_id(
                1,
                None,
                SemanticsRole::Panel,
                rect(10.0, 10.0, 320.0, 0.1),
                "resizable",
                "ui-gallery-resizable-panels",
            )],
        };

        let pred = UiPredicateV1::BoundsMinSize {
            target: UiSelectorV1::TestId {
                id: "ui-gallery-resizable-panels".to_string(),
            },
            min_w_px: 200.0,
            min_h_px: 200.0,
            eps_px: 0.0,
        };

        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "collapsed node should fail the min-size gate"
        );
    }

    #[test]
    fn bounds_non_overlapping_predicate_rejects_intersection() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(25.0, 10.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsNonOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected overlap (a right edge > b left edge) to fail"
        );

        let pred = UiPredicateV1::BoundsNonOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 16.0,
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected eps_px to tolerate a small overlap"
        );
    }

    #[test]
    fn not_exists_predicate_matches_absence() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node(
                1,
                None,
                SemanticsRole::Panel,
                rect(0.0, 0.0, 100.0, 100.0),
                "root",
            )],
        };

        let pred = UiPredicateV1::NotExists {
            target: UiSelectorV1::TestId {
                id: "missing".to_string(),
            },
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected missing test id to satisfy NotExists"
        );
    }

    #[test]
    fn bounds_overlapping_predicate_requires_intersection() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(25.0, 10.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected overlap (a right edge > b left edge) to pass"
        );

        let pred = UiPredicateV1::BoundsOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 16.0,
        };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected eps_px to require more overlap than available"
        );
    }

    #[test]
    fn bounds_overlapping_x_predicate_ignores_y() {
        let window_bounds = rect(0.0, 0.0, 100.0, 200.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 200.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(25.0, 150.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsOverlappingX {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected x overlap to pass even when y does not overlap"
        );

        let pred = UiPredicateV1::BoundsOverlappingX {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 8.0,
        };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected eps_px to require more x overlap than available"
        );
    }

    #[test]
    fn bounds_overlapping_y_predicate_ignores_x() {
        let window_bounds = rect(0.0, 0.0, 200.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(150.0, 25.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsOverlappingY {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected y overlap to pass even when x does not overlap"
        );

        let pred = UiPredicateV1::BoundsOverlappingY {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 8.0,
        };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window_id(1),
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected eps_px to require more y overlap than available"
        );
    }

    #[test]
    fn inspect_focus_shortcut_locks_to_semantics_focus() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(node_id(2)),
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "focus",
                    "focused-btn",
                ),
            ],
        };

        let window = window_id(1);
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.inspect_enabled = true;

        svc.inspect_pending_nav
            .insert(window, inspect::InspectNavCommand::Focus);
        svc.apply_inspect_navigation(window, Some(&snapshot), None);

        assert!(svc.inspect_is_locked(window));
        let focus_id = snapshot.focus.expect("focus").data().as_ffi();
        assert_eq!(svc.inspect_focus_node_id(window), Some(focus_id));
        assert!(
            svc.inspect_best_selector_json(window)
                .is_some_and(|s| s.contains("test_id"))
        );
    }

    #[test]
    fn pick_by_bounds_respects_modal_barrier() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(3),
                    visible: true,
                    blocks_underlay_input: true,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: Some(node_id(3)),
            focus_barrier_root: Some(node_id(3)),
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "underlay",
                ),
                semantics_node(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "underlay-button",
                ),
                semantics_node(
                    3,
                    None,
                    SemanticsRole::Dialog,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "modal",
                ),
                semantics_node(
                    4,
                    Some(3),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "modal-button",
                ),
            ],
        };

        let picked = pick_semantics_node_by_bounds(&snapshot, Point::new(Px(10.0), Px(10.0)))
            .expect("expected a pick");
        assert_eq!(picked.id, node_id(4));
    }

    #[test]
    fn scripts_can_assert_barrier_root_and_focus_barrier_root_independently() {
        let window = window_id(1);
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: true,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: Some(node_id(1)),
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node(
                1,
                None,
                SemanticsRole::Window,
                rect(0.0, 0.0, 100.0, 100.0),
                "root",
            )],
        };

        let pred = UiPredicateV1::BarrierRoots {
            barrier_root: UiOptionalRootStateV1::Some,
            focus_barrier_root: UiOptionalRootStateV1::None,
            require_equal: Some(false),
        };

        assert!(
            eval_predicate(
                &snapshot,
                window_bounds,
                window,
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected scripts to assert that the pointer barrier can remain active while focus containment is released"
        );

        let pred = UiPredicateV1::BarrierRoots {
            barrier_root: UiOptionalRootStateV1::Some,
            focus_barrier_root: UiOptionalRootStateV1::None,
            require_equal: Some(true),
        };
        assert!(
            !eval_predicate(
                &snapshot,
                window_bounds,
                window,
                None,
                None,
                None,
                None,
                &[],
                None,
                None,
                None,
                0,
                false,
                true,
                &pred
            ),
            "expected require_equal=true to fail when the roots differ"
        );
    }

    #[test]
    fn scripts_can_assert_barrier_roots_via_drive_script() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.cfg.script_auto_dump = false;

        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-script-{}", unique));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        svc.cfg.out_dir = dir.clone();
        svc.cfg.ready_path = dir.join("ready.touch");
        svc.cfg.script_path = dir.join("script.json");
        svc.cfg.script_trigger_path = dir.join("script.touch");
        svc.cfg.script_result_path = dir.join("script.result.json");
        svc.cfg.script_result_trigger_path = dir.join("script.result.touch");

        let window = AppWindowId::default();
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: true,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: Some(node_id(1)),
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node(
                1,
                None,
                SemanticsRole::Window,
                rect(0.0, 0.0, 100.0, 100.0),
                "root",
            )],
        };

        let script: UiActionScriptV1 = serde_json::from_str(
            r#"{
                "schema_version": 1,
                "steps": [
                    {
                        "type": "assert",
                        "predicate": {
                            "kind": "barrier_roots",
                            "barrier_root": "some",
                            "focus_barrier_root": "none",
                            "require_equal": false
                        }
                    }
                ]
            }"#,
        )
        .expect("parse barrier_roots predicate");
        svc.pending_script = PendingScript::from_v1(script);
        assert!(
            svc.pending_script.is_some(),
            "script schema_version should be valid"
        );
        svc.pending_script_run_id = Some(1);

        let mut app = App::new();
        let _ = svc.drive_script_for_window(
            &mut app,
            window,
            window_bounds,
            1.0,
            None,
            Some(&snapshot),
        );

        let bytes =
            std::fs::read(&svc.cfg.script_result_path).expect("read script result json file");
        let result: UiScriptResultV1 =
            serde_json::from_slice(&bytes).expect("parse UiScriptResultV1");
        assert!(
            matches!(result.stage, UiScriptStageV1::Passed),
            "expected drive_script to persist the passed result"
        );
    }

    #[test]
    fn hit_test_snapshot_exposes_focus_barrier_root() {
        let position = Point::new(Px(1.0), Px(2.0));
        let hit_test = UiDebugHitTest {
            hit: None,
            active_layer_roots: vec![node_id(10)],
            barrier_root: Some(node_id(10)),
        };

        let snap = UiHitTestSnapshotV1::from_hit_test_with_layers(
            position,
            hit_test,
            Some(node_id(11)),
            &[],
        );

        assert_eq!(snap.barrier_root, Some(key_to_u64(node_id(10))));
        assert_eq!(snap.focus_barrier_root, Some(key_to_u64(node_id(11))));
        assert!(
            snap.scope_roots
                .iter()
                .any(|r| r.kind == "modal_barrier_root" && r.root == key_to_u64(node_id(10)))
        );
        assert!(
            snap.scope_roots
                .iter()
                .any(|r| { r.kind == "focus_barrier_root" && r.root == key_to_u64(node_id(11)) })
        );
    }
}

fn sanitize_path_for_bundle(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

trait PointerEventExt {
    fn kind(&self) -> &'static str;
    fn position(&self) -> Point;
}

impl PointerEventExt for fret_core::PointerEvent {
    fn kind(&self) -> &'static str {
        match self {
            fret_core::PointerEvent::Down { .. } => "down",
            fret_core::PointerEvent::Up { .. } => "up",
            fret_core::PointerEvent::Move { .. } => "move",
            fret_core::PointerEvent::Wheel { .. } => "wheel",
            fret_core::PointerEvent::PinchGesture { .. } => "pinch_gesture",
        }
    }

    fn position(&self) -> Point {
        match self {
            fret_core::PointerEvent::Down { position, .. } => *position,
            fret_core::PointerEvent::Up { position, .. } => *position,
            fret_core::PointerEvent::Move { position, .. } => *position,
            fret_core::PointerEvent::Wheel { position, .. } => *position,
            fret_core::PointerEvent::PinchGesture { position, .. } => *position,
        }
    }
}

trait EventPointerExt {
    fn pointer_event(&self) -> Option<&fret_core::PointerEvent>;
}

impl EventPointerExt for Event {
    fn pointer_event(&self) -> Option<&fret_core::PointerEvent> {
        match self {
            Event::Pointer(p) => Some(p),
            _ => None,
        }
    }
}
