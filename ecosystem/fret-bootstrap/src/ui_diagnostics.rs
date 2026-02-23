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
mod bundle_sidecars;
mod fs_triggers;
mod inspect;
mod pick;
mod pick_flow;
mod snapshot_recording;
mod snapshot_types;
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

mod trace_helpers;
use trace_helpers::{
    MAX_CLICK_STABLE_TRACE_ENTRIES, MAX_FOCUS_TRACE_ENTRIES, MAX_IME_EVENT_TRACE_ENTRIES,
    MAX_OVERLAY_PLACEMENT_TRACE_ENTRIES, MAX_SELECTOR_TRACE_CANDIDATES,
    MAX_SHORTCUT_ROUTING_TRACE_ENTRIES, MAX_WEB_IME_TRACE_ENTRIES, push_bounds_stable_trace,
    push_hit_test_trace, push_selector_resolution_trace,
};

// Split out the DevTools WS wiring to reduce churn in this file.
#[path = "ui_diagnostics/ui_diagnostics_devtools_ws.rs"]
mod ui_diagnostics_devtools_ws;

use snapshot_types::WindowRing;
pub use snapshot_types::*;

mod config;
pub use config::UiDiagnosticsConfig;
include!("ui_diagnostics/service.rs");

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

// Bundle serialization types live in `ui_diagnostics/bundle.rs`.

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

include!("ui_diagnostics/pick_output_types.rs");

include!("ui_diagnostics/debug_snapshot_types.rs");

include!("ui_diagnostics/debug_snapshot_impl.rs");

include!("ui_diagnostics/docking_diagnostics.rs");

include!("ui_diagnostics/viewport_input_types.rs");

include!("ui_diagnostics/overlay_synthesis_diagnostics.rs");

include!("ui_diagnostics/virtual_list_diagnostics.rs");

include!("ui_diagnostics/scroll_handle_diagnostics.rs");

include!("ui_diagnostics/prepaint_diagnostics.rs");

include!("ui_diagnostics/command_gating_trace.rs");

include!("ui_diagnostics/invalidation_diagnostics.rs");

include!("ui_diagnostics/removed_subtree_diagnostics.rs");

include!("ui_diagnostics/cache_root_diagnostics.rs");

include!("ui_diagnostics/layout_paint_hotspot_diagnostics.rs");

include!("ui_diagnostics/model_global_change_diagnostics.rs");

// Semantics bundle types live in `ui_diagnostics/semantics.rs`.
include!("ui_diagnostics/frame_stats.rs");

#[path = "ui_diagnostics/ui_thread_cpu_time.rs"]
mod ui_thread_cpu_time;

include!("ui_diagnostics/layer_diagnostics.rs");

include!("ui_diagnostics/hit_test_diagnostics.rs");

include!("ui_diagnostics/element_runtime_diagnostics.rs");

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

include!("ui_diagnostics/recorded_event_types.rs");

include!("ui_diagnostics/labels.rs");

include!("ui_diagnostics/fingerprint.rs");

include!("ui_diagnostics/overlay_placement_trace_recording.rs");

include!("ui_diagnostics/focus_and_ime_trace_recording.rs");

include!("ui_diagnostics/hit_test_trace_recording.rs");

include!("ui_diagnostics/selector_resolution_trace_recording.rs");

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

include!("ui_diagnostics/window_target_and_geometry_helpers.rs");

include!("ui_diagnostics/input_event_synthesis.rs");

include!("ui_diagnostics/json_utils.rs");

include!("ui_diagnostics/truncation.rs");

include!("ui_diagnostics/touch_stamp.rs");
include!("ui_diagnostics/devtools_ws_helpers.rs");

include!("ui_diagnostics/path_utils.rs");

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
