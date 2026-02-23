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

include!("ui_diagnostics/fingerprint.rs");

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
