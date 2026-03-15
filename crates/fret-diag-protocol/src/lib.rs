//! Stable, serializable protocol types for Fret diagnostics and scripted UI automation.
//!
//! The diagnostics pipeline intentionally uses explicit schema versions (e.g. `*V1`, `*V2`) so
//! tooling can evolve without breaking older bundles/scripts.
//!
//! Most users interact with this crate indirectly via `fretboard diag` and the JSON artifacts in
//! `tools/diag-scripts/`.

use serde::{Deserialize, Serialize};

pub mod builder;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Envelope message for diagnostics/devtools transports.
///
/// Transports (e.g. WebSockets) send a `type` discriminator and a free-form JSON `payload`.
/// Higher-level tooling is responsible for validating the schema version and payload structure.
pub struct DiagTransportMessageV1 {
    pub schema_version: u32,
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Hello message sent by a client when attaching to a devtools server.
pub struct DevtoolsHelloV1 {
    pub client_kind: String,
    pub client_version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Acknowledgement message returned by the devtools server after receiving [`DevtoolsHelloV1`].
pub struct DevtoolsHelloAckV1 {
    pub server_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub server_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsSessionDescriptorV1 {
    pub session_id: String,
    pub client_kind: String,
    pub client_version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsSessionListV1 {
    pub sessions: Vec<DevtoolsSessionDescriptorV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsSessionAddedV1 {
    pub session: DevtoolsSessionDescriptorV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsSessionRemovedV1 {
    pub session_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiScriptMetaV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiImeEventV1 {
    Enabled,
    Disabled,
    Commit {
        text: String,
    },
    /// IME preedit update.
    ///
    /// `cursor_bytes` is a byte-indexed range in the preedit string (begin, end).
    /// When `None`, the cursor should be hidden.
    Preedit {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cursor_bytes: Option<(u32, u32)>,
    },
    /// Delete text surrounding the cursor or selection.
    ///
    /// Offsets are expressed in UTF-8 bytes.
    DeleteSurrounding {
        before_bytes: u32,
        after_bytes: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Scripted UI interaction plan (schema v1).
///
/// Used by `fretboard diag` to drive automated UI actions and assertions.
pub struct UiActionScriptV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<UiScriptMetaV1>,
    pub steps: Vec<UiActionStepV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiActionStepV1 {
    Click {
        target: UiSelectorV1,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(
            default = "default_click_count",
            skip_serializing_if = "is_default_click_count"
        )]
        click_count: u8,
    },
    ResetDiagnostics,
    MovePointer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_snapshots: Option<u32>,
    },
    CaptureScreenshot {
        label: Option<String>,
        #[serde(default = "default_capture_screenshot_timeout_frames")]
        timeout_frames: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Scripted UI interaction plan (schema v2).
///
/// This is the preferred schema for new scripts and generators.
pub struct UiActionScriptV2 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<UiScriptMetaV1>,
    pub steps: Vec<UiActionStepV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCapabilitiesHintsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_script_schema_v1: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_bundle_schema2: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCapabilitiesV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    /// Optional runner identity for auditability (additive; tooling must treat as hints).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_version: Option<String>,
    /// Optional schema/config hints for tooling and triage (additive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hints: Option<FilesystemCapabilitiesHintsV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiDiagnosticsConfigPathsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_path: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_request_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_trigger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_result_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_result_trigger_path: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_trigger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_result_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_result_trigger_path: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick_trigger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick_result_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick_result_trigger_path: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspect_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspect_trigger_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiDiagnosticsConfigFileV1 {
    pub schema_version: u32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<UiDiagnosticsConfigPathsV1>,

    /// Whether the diagnostics runtime should accept script schema v1 inputs.
    ///
    /// When `None`, the runtime uses its default policy (currently: allow in manual flows; tooling
    /// typically writes an explicit `false` for launched runs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_script_schema_v1: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_keepalive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_auto_dump: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick_auto_dump: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_events: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_snapshots: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_dump_max_snapshots: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_semantics: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_semantics_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_test_ids_only: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshots_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_on_dump: Option<bool>,

    /// Whether the diagnostics runtime should write the large raw bundle artifact (`bundle.json`)
    /// during dumps.
    ///
    /// Tooling typically sets this to `false` for launched runs so default artifacts stay
    /// small-by-default (manifest + sidecars + optional compact bundle view).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_bundle_json: Option<bool>,
    /// Whether the diagnostics runtime should write the compact bundle view (`bundle.schema2.json`)
    /// alongside sidecars during dumps.
    ///
    /// This is intended for schema2-first + AI/sidecar-first workflows to avoid requiring tooling
    /// to parse large raw bundles just to produce a portable artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_bundle_schema2: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redact_text: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_debug_string_bytes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_gating_trace_entries: Option<u32>,

    /// When enabled, ignore external pointer input events (mouse/touch/pen) while a diagnostics
    /// script is running.
    ///
    /// This is intended to keep scripted runs deterministic when a user accidentally moves or
    /// clicks the real mouse during playback (especially for cross-window docking/tear-off).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolate_external_pointer_input_while_script_running: Option<bool>,

    /// When enabled, ignore external keyboard/text/IME events while a diagnostics script is
    /// running.
    ///
    /// This is intended to keep scripted runs deterministic when a user accidentally types while
    /// playback is in progress (especially when scripts are asserting shortcut routing or text
    /// input outcomes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolate_external_keyboard_input_while_script_running: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_clock_fixed_delta_ms: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devtools_embed_bundle: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiPaddingInsetsV1 {
    pub left_px: f32,
    pub top_px: f32,
    pub right_px: f32,
    pub bottom_px: f32,
}

impl UiPaddingInsetsV1 {
    pub fn uniform(padding_px: f32) -> Self {
        let p = padding_px.max(0.0);
        Self {
            left_px: p,
            top_px: p,
            right_px: p,
            bottom_px: p,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiWindowTargetV1 {
    /// Target the window currently driving the script step.
    Current,
    /// Target the first window observed by the diagnostics runtime.
    FirstSeen,
    /// Target the first observed window that is not the current window.
    FirstSeenOther,
    /// Target the most recently observed window.
    LastSeen,
    /// Target the most recently observed window that is not the current window.
    LastSeenOther,
    /// Target a specific window by its FFI handle/id as reported in bundles and script results.
    WindowFfi { window: u64 },
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiInsetsOverrideV1 {
    #[default]
    NoChange,
    Clear,
    Set {
        insets_px: UiPaddingInsetsV1,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiIncomingOpenInjectItemV1 {
    /// Diagnostics-only UTF-8 file payload.
    ///
    /// This is intended for CI fixtures and does not model binary files or platform handles.
    FileUtf8 {
        name: String,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiActionStepV2 {
    // v1-compatible steps
    Click {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(
            default = "default_click_count",
            skip_serializing_if = "is_default_click_count"
        )]
        click_count: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modifiers: Option<UiKeyModifiersV1>,
    },
    /// A high-level “tap” gesture (touch-first) resolved via semantics selectors.
    ///
    /// This is intended for mobile-style interaction policies where "click" is an imprecise term.
    /// Runtime injection still maps to unified pointer events with `PointerType::Touch` by default.
    Tap {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        /// Optional override; when omitted, defaults to `touch`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modifiers: Option<UiKeyModifiersV1>,
    },
    /// A high-level “long press” gesture (touch-first) resolved via semantics selectors.
    ///
    /// Runtime injection emits a `pointer_down`, holds until `duration_ms` elapses, then emits
    /// `pointer_up` with `is_click=false`.
    LongPress {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        /// Optional override; when omitted, defaults to `touch`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(
            default = "default_long_press_duration_ms",
            skip_serializing_if = "is_default_long_press_duration_ms"
        )]
        duration_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modifiers: Option<UiKeyModifiersV1>,
    },
    /// A high-level “swipe” gesture (touch-first) resolved via semantics selectors.
    ///
    /// Runtime injection emits a `pointer_down` at the target's center, then a sequence of
    /// `pointer_move` events to the end position, then a `pointer_up` with `is_click=false`.
    Swipe {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        /// Optional override; when omitted, defaults to `touch`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        delta_x: f32,
        delta_y: f32,
        #[serde(
            default = "default_drag_steps",
            skip_serializing_if = "is_default_drag_steps"
        )]
        steps: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modifiers: Option<UiKeyModifiersV1>,
    },
    /// A pinch/zoom gesture emitted at the target's center.
    ///
    /// `delta` is positive for zoom in and negative for zoom out (matches `PointerEvent::PinchGesture`).
    Pinch {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        /// Optional override; when omitted, defaults to `touch`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        /// Total delta across all steps.
        delta: f32,
        #[serde(
            default = "default_drag_steps",
            skip_serializing_if = "is_default_drag_steps"
        )]
        steps: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modifiers: Option<UiKeyModifiersV1>,
    },
    ResetDiagnostics,
    /// Set a “base reference” for subsequent selector-driven steps.
    ///
    /// When a base reference is set, the runtime may scope later selector resolution to the
    /// base node's subtree (window-local) until [`UiActionStepV2::ClearBaseRef`] is executed.
    ///
    /// This is an ergonomics feature (ImGui `SetRef(...)`-style outcome) intended to reduce
    /// repetition and diff noise in long scripts.
    SetBaseRef {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
    },
    /// Clear the active base reference, restoring global selector resolution.
    ClearBaseRef,
    /// Semantically activate a target using the runtime accessibility action surface.
    ///
    /// This bypasses pointer hit-testing and is primarily intended for diagnosis: it helps
    /// distinguish "semantics can activate" from "pointer cannot hit the target".
    Activate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
    },
    /// Move accessibility focus to a target without invoking it.
    ///
    /// This is intended for diagnosis and parity checks where you want to distinguish
    /// focusability from pointer hit-testing and activation behavior.
    Focus {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
    },
    MovePointer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
    },
    /// Move the pointer to a target and issue a pointer down, keeping the session active across
    /// subsequent steps (until `pointer_up`).
    ///
    /// This is intended for scripted "drag + key" flows (e.g. press Escape while dragging).
    PointerDown {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modifiers: Option<UiKeyModifiersV1>,
    },
    DragPointer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(default = "default_true")]
        clamp_to_window_bounds: bool,
        delta_x: f32,
        delta_y: f32,
        #[serde(default = "default_drag_steps")]
        steps: u32,
    },
    /// Move the pointer while a `pointer_down` session is active.
    ///
    /// This emits `PointerEvent::Move` with pressed buttons and also mirrors internal drag routing
    /// by emitting `InternalDrag::Over` events (safe unless a cross-window internal-drag session is
    /// active).
    PointerMove {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        delta_x: f32,
        delta_y: f32,
        #[serde(default = "default_drag_steps")]
        steps: u32,
    },
    /// Release an active `pointer_down` session.
    ///
    /// This emits `PointerEvent::Up` and mirrors internal drag routing by emitting
    /// `InternalDrag::Drop`.
    PointerUp {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button: Option<UiMouseButtonV1>,
    },
    /// Cancel an active `pointer_down` session.
    ///
    /// This emits `Event::PointerCancel` and mirrors internal drag routing by emitting
    /// `InternalDrag::Cancel`.
    PointerCancel {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
    },
    MovePointerSweep {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        delta_x: f32,
        delta_y: f32,
        #[serde(default = "default_drag_steps")]
        steps: u32,
        #[serde(default = "default_move_frames_per_step")]
        frames_per_step: u32,
    },
    Wheel {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        delta_x: f32,
        #[serde(default)]
        delta_y: f32,
    },
    /// Inject a burst of wheel events in a single frame via the native runner (best-effort).
    ///
    /// Unlike [`UiActionStepV2::Wheel`], which injects a single `pointer.wheel` event directly into
    /// the UI event stream, this step is intended to exercise runner-level wheel coalescing by
    /// synthesizing multiple raw wheel inputs in the same frame.
    ///
    /// Requires capability `diag.wheel_burst_inject`.
    WheelBurst {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        delta_x: f32,
        #[serde(default)]
        delta_y: f32,
        #[serde(
            default = "default_wheel_burst_count",
            skip_serializing_if = "is_default_wheel_burst_count"
        )]
        count: u32,
    },
    PressKey {
        key: String,
        #[serde(default)]
        modifiers: UiKeyModifiersV1,
        #[serde(default)]
        repeat: bool,
    },
    PressShortcut {
        shortcut: String,
        #[serde(default)]
        repeat: bool,
    },
    TypeText {
        text: String,
    },
    /// Inject an IME event into the focused text surface.
    ///
    /// This is intended for deterministic regression scripts that need to exercise text/IME
    /// composition without depending on platform IME integrations.
    Ime {
        event: UiImeEventV1,
    },
    WaitFrames {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        n: u32,
    },
    /// Wait for a fixed duration (in milliseconds).
    ///
    /// This is intended as a last-resort stabilization step when no semantic predicate exists.
    /// Prefer `wait_until` for deterministic gates.
    WaitMs {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        n_ms: u32,
    },
    WaitUntil {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        predicate: UiPredicateV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u32>,
    },
    /// Wait until the shortcut routing diagnostics trace contains an entry matching `query`.
    ///
    /// This is intended for deterministic scripts that need to assert keyboard routing outcomes
    /// (e.g. reserved-for-IME) without depending on screenshots or ad-hoc logs.
    WaitShortcutRoutingTrace {
        query: UiShortcutRoutingTraceQueryV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u32>,
    },
    /// Wait until the command dispatch trace contains an entry matching `query`.
    ///
    /// This is intended to gate action-first convergence work: pointer triggers, keymap shortcuts,
    /// and command palette/menus should all produce explainable dispatch outcomes.
    WaitCommandDispatchTrace {
        query: UiCommandDispatchTraceQueryV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u32>,
    },
    /// Wait until the overlay placement trace contains an entry matching `query`.
    ///
    /// This is intended for overlay-driven components (Select/Combobox/Menus) where correctness
    /// depends on collision/flip/shift behavior and we want failures to be explainable without
    /// relying on screenshots.
    WaitOverlayPlacementTrace {
        query: UiOverlayPlacementTraceQueryV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u32>,
    },
    Assert {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        predicate: UiPredicateV1,
    },
    CaptureBundle {
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_snapshots: Option<u32>,
    },
    CaptureScreenshot {
        label: Option<String>,
        #[serde(default = "default_capture_screenshot_timeout_frames")]
        timeout_frames: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u32>,
    },
    /// Capture a layout sidecar (native-only, best-effort).
    ///
    /// This is intended to make layout regressions explainable via a bundle-scoped sidecar file
    /// (e.g. `layout.taffy.v1.json`) rather than ad-hoc debug UI in demos.
    ///
    /// Tooling should treat missing sidecars as warnings, not failures.
    CaptureLayoutSidecar {
        /// Optional label used to name the bundle directory for this capture step.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        /// Optional debug label filter for selecting a subtree root before dumping.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root_label_filter: Option<String>,
    },

    // v2 intent-level steps
    /// Click a target only after its bounds have remained stable for `stable_frames`.
    ///
    /// This is useful for virtualized lists where a target's measured bounds can jump
    /// across frames (e.g. estimate -> measured), causing clicks to land at stale
    /// positions when using a single-frame snapshot.
    ClickStable {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(
            default = "default_click_count",
            skip_serializing_if = "is_default_click_count"
        )]
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
    /// Click an interactive span (by `tag`) inside a `SelectableText` target after its computed
    /// span bounds remain stable for `stable_frames`.
    ///
    /// This is intended for rich text surfaces where the clickable region is smaller than the
    /// semantics node bounds (e.g. link spans inside a paragraph), and where clicking the center
    /// of the node can miss the span.
    ClickSelectableTextSpanStable {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        tag: String,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(
            default = "default_click_count",
            skip_serializing_if = "is_default_click_count"
        )]
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
    /// Wait until a target's semantics bounds have remained stable for `stable_frames`.
    ///
    /// This is useful for overlays/virtualized surfaces where measured bounds can jump across
    /// frames (estimate -> measured, placement flip/shift, scroll settle), and you want a
    /// deterministic “ready” point without relying on wall-clock sleeps.
    WaitBoundsStable {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
        #[serde(default = "default_bounds_stable_frames")]
        stable_frames: u32,
        #[serde(default = "default_bounds_stable_max_move_px")]
        max_move_px: f32,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    /// Wait until a target's structured semantics scroll field has remained stable for
    /// `stable_frames`.
    ///
    /// This is intended for scrollable surfaces whose content extent converges across a few
    /// post-layout frames (for example after switching tabs or appending content at the bottom).
    WaitSemanticsScrollStable {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
        field: UiSemanticsScrollFieldV1,
        #[serde(default = "default_semantics_scroll_stable_frames")]
        stable_frames: u32,
        #[serde(default = "default_semantics_scroll_stable_max_delta")]
        max_delta: f64,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    EnsureVisible {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
        #[serde(default)]
        within_window: bool,
        #[serde(default)]
        padding_px: f32,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    ScrollIntoView {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        container: UiSelectorV1,
        target: UiSelectorV1,
        #[serde(default)]
        delta_x: f32,
        #[serde(default = "default_scroll_delta_y")]
        delta_y: f32,
        #[serde(default)]
        require_fully_within_container: bool,
        #[serde(default)]
        require_fully_within_window: bool,
        #[serde(default)]
        padding_px: f32,
        #[serde(default)]
        padding_insets_px: Option<UiPaddingInsetsV1>,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    TypeTextInto {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        text: String,
        #[serde(default)]
        clear_before_type: bool,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    /// Set the full text value of a target text surface via the accessibility SetValue path.
    ///
    /// Unlike [`UiActionStepV2::TypeTextInto`], this step does not depend on click-to-focus
    /// behavior. It resolves the target from semantics, requests focus for that node, selects the
    /// current text, then dispatches a single text input payload with `text`.
    ///
    /// Notes:
    ///
    /// - Intended for diagnostics gates that need stable text entry across policy-layer widgets.
    /// - Targets should expose `actions.set_value=true`; disabled targets fail the step.
    SetTextValue {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        target: UiSelectorV1,
        text: String,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    /// Paste `text` into a target text surface by:
    ///
    /// 1) clicking the target to focus it,
    /// 2) setting the OS clipboard text (best-effort),
    /// 3) issuing the platform "paste" shortcut (`Primary+V`).
    ///
    /// This is intentionally higher-level than `set_clipboard_text + press_shortcut` so scripts
    /// can gate paste-specific code paths with less boilerplate.
    ///
    /// Notes:
    ///
    /// - The clipboard write is best-effort and runner/platform dependent.
    /// - `clear_before_paste` uses `SetTextSelection { anchor=0, focus=u32::MAX }` (not `Ctrl+A`).
    PasteTextInto {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        text: String,
        #[serde(default)]
        clear_before_paste: bool,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    MenuSelect {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        menu: UiSelectorV1,
        item: UiSelectorV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    MenuSelectPath {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        path: Vec<UiSelectorV1>,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    DragTo {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
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
    SetWindowInnerSize {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        width_px: f32,
        height_px: f32,
    },
    /// Best-effort request to update OS window style facets at runtime (patch semantics).
    ///
    /// This is intended for diagnostics-only repros and regression gates for utility window
    /// postures (frameless/transparent/materials/hit-test policies).
    ///
    /// Capability-gated behind `diag.window_style_patch_v1`.
    ///
    /// Note: as of 2026-03-04 this capability is Windows-only in the default in-tree runner.
    /// Supported patch fields in the default runner are currently limited to:
    /// - `z_level`
    /// - `background_material`
    /// - `hit_test`
    /// - `opacity_alpha_u8`
    SetWindowStyle {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        style: UiWindowStylePatchV1,
    },
    SetWindowInsets {
        #[serde(default)]
        safe_area_insets: UiInsetsOverrideV1,
        #[serde(default)]
        occlusion_insets: UiInsetsOverrideV1,
    },
    /// Diagnostics-only clipboard override to simulate clipboard read denial/unavailability.
    ///
    /// This is intended to gate “paste request fails gracefully” behavior under mobile privacy
    /// constraints without requiring a real mobile runner.
    SetClipboardForceUnavailable {
        enabled: bool,
    },
    /// Set the OS clipboard text payload (best-effort).
    ///
    /// This is intended to make "paste" flows deterministic in scripted diagnostics by ensuring
    /// the clipboard contents are known.
    ///
    /// Requires capability `diag.clipboard_text`.
    SetClipboardText {
        text: String,
    },
    /// Assert that the OS clipboard text payload equals `text` (best-effort).
    ///
    /// This is intended to make clipboard-driven regression scripts explainable without relying
    /// on screenshots.
    ///
    /// Requires capability `diag.clipboard_text`.
    AssertClipboardText {
        text: String,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    /// In-app inspector helper: open help, search for `query`, lock the best match, and request
    /// copying the best selector JSON to the OS clipboard.
    ///
    /// This is intended to gate that the in-app inspector UX is still functional under
    /// tool-launched scripted runs (`fretboard diag run/suite --launch`) without relying on
    /// keyboard shortcut injection.
    ///
    /// Behavior notes:
    ///
    /// - Matching prefers `test_id`, then `label` when text redaction is disabled.
    /// - The runtime may keep the help overlay open after the step.
    InspectHelpLockBestMatchAndCopySelector {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        query: String,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    /// In-app inspector helper: open help, ensure the semantics tree panel is visible, select the
    /// best match for `query` in the tree, lock it, and copy the best selector JSON to the OS
    /// clipboard.
    ///
    /// This is intended to gate that the help-mode tree browser remains functional under
    /// tool-launched scripted runs (`fretboard diag run/suite --launch`) without relying on
    /// keyboard shortcut injection.
    InspectHelpTreeLockBestMatchAndCopySelector {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        query: String,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    /// Diagnostics-only incoming-open injection (best-effort).
    ///
    /// This simulates “open in…” / share-target flows by injecting an `IncomingOpenRequest` event.
    InjectIncomingOpen {
        items: Vec<UiIncomingOpenInjectItemV1>,
    },
    /// Set the OS window outer position (screen-space logical pixels).
    ///
    /// This is intended for deterministically arranging windows in scripted repros and for
    /// best-effort placement restoration (ADR 0017).
    SetWindowOuterPosition {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        x_px: f32,
        y_px: f32,
    },
    /// Set a runner-level cursor screen position override (screen-space physical pixels).
    ///
    /// Desktop runners may use this during scripted diagnostics to drive hover routing that is
    /// normally owned by OS cursor events (e.g. cross-window docking).
    ///
    /// Requires capability `diag.cursor_screen_pos_override`.
    SetCursorScreenPos {
        x_px: f32,
        y_px: f32,
    },
    /// Set a runner-level cursor screen position override using window-local client coordinates.
    ///
    /// This is intended for cross-window scripted diagnostics where the runner must synthesize a
    /// global cursor location from window-local input.
    ///
    /// Coordinates are in window-client **physical pixels**.
    ///
    /// Requires capability `diag.cursor_screen_pos_override`.
    SetCursorInWindow {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        x_px: f32,
        y_px: f32,
    },
    /// Set a runner-level cursor screen position override using window-local client coordinates.
    ///
    /// This is identical to `set_cursor_in_window`, except the coordinates are in window-client
    /// **logical pixels** (pre-DPI scale). The runner converts to physical pixels using the
    /// current window scale factor.
    ///
    /// Prefer this for deterministic scripts that already express geometry in logical pixels.
    ///
    /// Requires capability `diag.cursor_screen_pos_override`.
    SetCursorInWindowLogical {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        x_px: f32,
        y_px: f32,
    },
    /// Set a runner-level mouse button state override.
    ///
    /// This is intended for scripted diagnostics that need to exercise runner-level fallback
    /// behavior that depends on OS button state (e.g. "release outside all windows" poll-up
    /// paths) without requiring real OS input.
    ///
    /// Desktop runners may choose to apply this only while certain interactions are active
    /// (e.g. cross-window dock drags).
    SetMouseButtons {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        left: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        right: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        middle: Option<bool>,
    },
    RaiseWindow {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
    },
    /// Drag with pointer down across frames until `predicate` passes, or timeout.
    ///
    /// This is intended for runner-owned cross-window routing: scripts can keep a drag session
    /// active while polling diagnostics predicates that are only updated between frames.
    DragPointerUntil {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<UiWindowTargetV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pointer_kind: Option<UiPointerKindV1>,
        target: UiSelectorV1,
        #[serde(default)]
        button: UiMouseButtonV1,
        #[serde(default = "default_true")]
        release_on_success: bool,
        delta_x: f32,
        delta_y: f32,
        #[serde(default = "default_drag_steps")]
        steps: u32,
        predicate: UiPredicateV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
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
                window: None,
                pointer_kind: None,
                target,
                button,
                click_count,
                modifiers: None,
            },
            UiActionStepV1::ResetDiagnostics => Self::ResetDiagnostics,
            UiActionStepV1::MovePointer { window, target } => Self::MovePointer {
                window,
                pointer_kind: None,
                target,
            },
            UiActionStepV1::DragPointer {
                target,
                button,
                delta_x,
                delta_y,
                steps,
            } => Self::DragPointer {
                window: None,
                pointer_kind: None,
                target,
                button,
                clamp_to_window_bounds: true,
                delta_x,
                delta_y,
                steps,
            },
            UiActionStepV1::Wheel {
                target,
                delta_x,
                delta_y,
            } => Self::Wheel {
                window: None,
                pointer_kind: None,
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
            UiActionStepV1::WaitFrames { n } => Self::WaitFrames { window: None, n },
            UiActionStepV1::WaitUntil {
                predicate,
                timeout_frames,
            } => Self::WaitUntil {
                window: None,
                predicate,
                timeout_frames,
                timeout_ms: None,
            },
            UiActionStepV1::Assert { predicate } => Self::Assert {
                window: None,
                predicate,
            },
            UiActionStepV1::CaptureBundle {
                label,
                max_snapshots,
            } => Self::CaptureBundle {
                label,
                max_snapshots,
            },
            UiActionStepV1::CaptureScreenshot {
                label,
                timeout_frames,
            } => Self::CaptureScreenshot {
                label,
                timeout_frames,
                timeout_ms: None,
            },
        }
    }
}

fn default_drag_steps() -> u32 {
    8
}

fn is_default_drag_steps(v: &u32) -> bool {
    *v == default_drag_steps()
}

fn default_wheel_burst_count() -> u32 {
    8
}

fn is_default_wheel_burst_count(v: &u32) -> bool {
    *v == default_wheel_burst_count()
}

fn default_move_frames_per_step() -> u32 {
    1
}

fn default_click_count() -> u8 {
    1
}

fn is_default_click_count(v: &u8) -> bool {
    *v == 1
}

fn default_long_press_duration_ms() -> u64 {
    500
}

fn is_default_long_press_duration_ms(v: &u64) -> bool {
    *v == 500
}

fn default_click_stable_frames() -> u32 {
    2
}

fn default_click_stable_max_move_px() -> f32 {
    1.0
}

fn default_bounds_stable_frames() -> u32 {
    2
}

fn default_bounds_stable_max_move_px() -> f32 {
    1.0
}

fn default_semantics_scroll_stable_frames() -> u32 {
    2
}

fn default_semantics_scroll_stable_max_delta() -> f64 {
    1.0
}

fn default_capture_screenshot_timeout_frames() -> u32 {
    300
}

fn default_action_timeout_frames() -> u32 {
    180
}

fn default_true() -> bool {
    true
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiMouseButtonV1 {
    #[default]
    Left,
    Right,
    Middle,
}

impl UiMouseButtonV1 {
    pub fn from_button(button: fret_core::MouseButton) -> Self {
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPointerKindV1 {
    #[default]
    Mouse,
    Touch,
    Pen,
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
    pub fn from_modifiers(modifiers: fret_core::Modifiers) -> Self {
        Self {
            shift: modifiers.shift,
            ctrl: modifiers.ctrl,
            alt: modifiers.alt,
            meta: modifiers.meta,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWindowDecorationsRequestV1 {
    System,
    None,
    Server,
    Client,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTaskbarVisibilityV1 {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiActivationPolicyV1 {
    Activates,
    NonActivating,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWindowZLevelV1 {
    Normal,
    AlwaysOnTop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWindowHitTestRequestV1 {
    Normal,
    PassthroughAll,
    PassthroughRegions,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWindowAppearanceV1 {
    Opaque,
    CompositedNoBackdrop,
    CompositedBackdrop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWindowBackgroundMaterialRequestV1 {
    None,
    SystemDefault,
    Mica,
    Acrylic,
    Vibrancy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiWindowStyleMatchV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorations: Option<UiWindowDecorationsRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resizable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transparent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_transparent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<UiWindowAppearanceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taskbar: Option<UiTaskbarVisibilityV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation: Option<UiActivationPolicyV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z_level: Option<UiWindowZLevelV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_test: Option<UiWindowHitTestRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_test_regions_fingerprint64: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiWindowStylePatchV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taskbar: Option<UiTaskbarVisibilityV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation: Option<UiActivationPolicyV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z_level: Option<UiWindowZLevelV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorations: Option<UiWindowDecorationsRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resizable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transparent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_material: Option<UiWindowBackgroundMaterialRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_test: Option<UiWindowHitTestPatchV1>,
    /// Global window opacity hint (0..=255), best-effort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_alpha_u8: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiWindowHitTestPatchV1 {
    Normal,
    PassthroughAll,
    PassthroughRegions {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        regions: Vec<UiWindowHitTestRegionV1>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiWindowHitTestRegionV1 {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    RRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    },
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
    /// True when `target` resolves to a node that is a descendant of (or equal to) `scope`.
    ///
    /// This is primarily used to disambiguate selectors when multiple similar widgets exist in a
    /// window (and to keep scripts resilient under overlay/root splits).
    ExistsUnder {
        scope: UiSelectorV1,
        target: UiSelectorV1,
    },
    /// True when `scope` exists and `target` does *not* exist under it.
    ///
    /// Note: if `scope` does not exist, this predicate returns false.
    NotExistsUnder {
        scope: UiSelectorV1,
        target: UiSelectorV1,
    },
    /// True when the currently focused semantics node equals `target` and is a descendant of
    /// (or equal to) `scope`.
    ///
    /// This is a convenience predicate for focus-trap / focus-restore assertions:
    /// - "focus stays within this dialog root"
    /// - "after open, focus moves to the dialog's close button"
    FocusedDescendantIs {
        scope: UiSelectorV1,
        target: UiSelectorV1,
    },
    FocusIs {
        target: UiSelectorV1,
    },
    RoleIs {
        target: UiSelectorV1,
        role: String,
    },
    /// True when the target exists and its semantics `label` contains `text` as a substring.
    LabelContains {
        target: UiSelectorV1,
        text: String,
    },
    /// True when the target exists and its semantics `label` length (UTF-8 bytes) matches `len_bytes`.
    ///
    /// This is intended to remain stable under diagnostics text redaction, where labels may be
    /// replaced with placeholders like `<redacted len=123>`.
    LabelLenIs {
        target: UiSelectorV1,
        len_bytes: u32,
    },
    /// True when the target exists and its semantics `label` length (UTF-8 bytes) is at least `min_len_bytes`.
    LabelLenGe {
        target: UiSelectorV1,
        min_len_bytes: u32,
    },
    /// True when the target exists and its semantics `value` contains `text` as a substring.
    ValueContains {
        target: UiSelectorV1,
        text: String,
    },
    /// True when the target exists and its semantics `value` equals `text`.
    ///
    /// Caution: some widgets use locale-dependent `value` strings; prefer structured predicates
    /// (`SemanticsNumericApproxEq`, `SemanticsScrollApproxEq`, ...) when available.
    ValueEquals {
        target: UiSelectorV1,
        text: String,
    },
    /// True when the target exists and its semantics `value` length (UTF-8 bytes) matches `len_bytes`.
    ///
    /// This is intended to remain stable under diagnostics text redaction, where values may be
    /// replaced with placeholders like `<redacted len=123>`.
    ValueLenIs {
        target: UiSelectorV1,
        len_bytes: u32,
    },
    /// True when the target exists and its semantics `value` length (UTF-8 bytes) is at least `min_len_bytes`.
    ValueLenGe {
        target: UiSelectorV1,
        min_len_bytes: u32,
    },
    /// True when the target exists and its semantics `pos_in_set` equals `pos_in_set`.
    PosInSetIs {
        target: UiSelectorV1,
        pos_in_set: u32,
    },
    /// True when the target exists and its semantics `set_size` equals `set_size`.
    SetSizeIs {
        target: UiSelectorV1,
        set_size: u32,
    },
    CheckedIs {
        target: UiSelectorV1,
        checked: bool,
    },
    SelectedIs {
        target: UiSelectorV1,
        selected: bool,
    },
    /// True when the target exists and its structured semantics numeric field is approximately
    /// equal to the specified value.
    ///
    /// This is intended for range controls (slider/progress-like semantics) which should prefer
    /// `SemanticsNode.extra.numeric.*` over locale-dependent `value` strings.
    SemanticsNumericApproxEq {
        target: UiSelectorV1,
        field: UiSemanticsNumericFieldV1,
        value: f64,
        #[serde(default)]
        eps: f64,
    },
    /// True when the target exists and its structured semantics scroll field is present and finite.
    ///
    /// This is a lightweight gate to ensure `SemanticsNode.extra.scroll.*` is emitted for scroll
    /// containers.
    SemanticsScrollIsFinite {
        target: UiSelectorV1,
        field: UiSemanticsScrollFieldV1,
    },
    /// True when the target exists and its structured semantics scroll field is approximately
    /// equal to the specified value.
    SemanticsScrollApproxEq {
        target: UiSelectorV1,
        field: UiSemanticsScrollFieldV1,
        value: f64,
        #[serde(default)]
        eps: f64,
    },
    /// True when the target exists and its structured semantics scroll field is not approximately
    /// equal to the specified value.
    SemanticsScrollNotApproxEq {
        target: UiSelectorV1,
        field: UiSemanticsScrollFieldV1,
        value: f64,
        #[serde(default)]
        eps: f64,
    },
    /// True when the target exists and its semantics reports whether it currently has an IME
    /// composition range.
    ///
    /// Notes:
    /// - This checks whether `SemanticsNode.text_composition` is `Some(_)`.
    /// - Some platforms/widgets may omit composition ranges even while composing; treat this
    ///   predicate as best-effort and gate it behind appropriate suites.
    TextCompositionIs {
        target: UiSelectorV1,
        composing: bool,
    },
    /// True when the diagnostics runtime has a window-level IME cursor area snapshot.
    ///
    /// Notes:
    /// - This reads `WindowTextInputSnapshot.ime_cursor_area`.
    /// - Coordinates are window logical pixels.
    ImeCursorAreaIsSome {
        is_some: bool,
    },
    /// True when the window-level IME cursor area snapshot is within the current window bounds.
    ///
    /// This is a coarse regression gate for IME geometry bugs (caret/candidate window
    /// teleportation, negative coordinates, far-offscreen rects).
    ImeCursorAreaWithinWindow {
        #[serde(default)]
        padding_px: f32,
        /// Optional per-edge padding (added on top of `padding_px`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        padding_insets_px: Option<UiPaddingInsetsV1>,
        #[serde(default)]
        eps_px: f32,
    },
    /// True when the window-level IME cursor area snapshot has at least the specified size.
    ///
    /// This can catch "zero rect" bugs where the IME caret geometry is missing meaningful size.
    ImeCursorAreaMinSize {
        #[serde(default)]
        min_w_px: f32,
        #[serde(default)]
        min_h_px: f32,
        #[serde(default)]
        eps_px: f32,
    },
    /// True when the diagnostics runtime has a window-level IME surrounding text excerpt.
    ///
    /// Notes:
    /// - This reads `WindowTextInputSnapshot.surrounding_text`.
    /// - Offsets are UTF-8 byte offsets within the excerpt and should be on char boundaries.
    ImeSurroundingTextIsSome {
        is_some: bool,
    },
    /// True when the window-level IME surrounding text excerpt is present and internally valid.
    ///
    /// This is a coarse regression gate for platform text-input interop (winit `ImeSurroundingText`
    /// constraints: max bytes, offsets within range, char boundaries).
    ImeSurroundingTextValid,
    CheckedIsNone {
        target: UiSelectorV1,
    },
    /// True when the current active item is the specified `item`.
    ///
    /// This supports both common semantics models:
    ///
    /// - Composite widgets that retain focus on a container and express the highlighted row via
    ///   `active_descendant` (DOM-style `aria-activedescendant`).
    /// - Widgets that use roving focus (the focused node itself is the active item).
    ActiveItemIs {
        /// Container node (e.g. listbox). Used when the widget uses `active_descendant`.
        container: UiSelectorV1,
        /// The expected active item (highlighted option / row).
        item: UiSelectorV1,
    },
    /// True when there is no active item (neither roving focus nor `active_descendant`).
    ///
    /// This is primarily intended for combobox/listbox recipes that should not implicitly
    /// highlight the first option on open unless `auto_highlight` is enabled.
    ActiveItemIsNone {
        /// Container node used for composite focus + `active_descendant` models (typically the
        /// focused input or listbox root).
        container: UiSelectorV1,
    },
    BarrierRoots {
        #[serde(default)]
        barrier_root: UiOptionalRootStateV1,
        #[serde(default)]
        focus_barrier_root: UiOptionalRootStateV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        require_equal: Option<bool>,
    },
    RenderTextMissingGlyphsIs {
        missing_glyphs: u64,
    },
    /// Ensures that when the renderer reports missing/tofu glyphs for the current frame, a
    /// renderer-owned font fallback trace has been captured and is non-empty.
    ///
    /// This predicate is meant to keep "tofu regressions" debuggable: if missing glyphs happen,
    /// the diagnostics bundle should contain an audit trail of the selected families.
    RenderTextFontTraceCapturedWhenMissingGlyphs,
    /// True when the runner-owned `TextFontStackKey` has not changed for `stable_frames`
    /// consecutive frames.
    ///
    /// This is primarily used to keep perf suites from including one-time system font catalog
    /// rescans (which bump `TextFontStackKey` and can trigger large relayouts) inside a measured
    /// window.
    TextFontStackKeyStable {
        stable_frames: u32,
    },
    /// True when the runner-owned `FontCatalog` has been populated with at least one family.
    ///
    /// On desktop, the runner may seed an empty catalog at startup and populate it asynchronously
    /// via the system font rescan pipeline. This predicate lets scripts wait for that one-time
    /// async work to complete before entering a measured window.
    FontCatalogPopulated,
    /// True when the runner-owned system font rescan pipeline is idle (no work in flight and no
    /// pending restart).
    ///
    /// Desktop runners may perform a one-time async system font rescan at startup. Applying the
    /// result can bump `TextFontStackKey` and trigger large relayouts; this predicate lets perf
    /// suites wait for that one-time work to complete before entering a measured window.
    SystemFontRescanIdle,
    /// True when `debug.resource_loading.asset_load.missing_bundle_asset_requests >= min`.
    ///
    /// This is intended for negative-path diagnostics scripts that deliberately trigger missing
    /// bundle assets and want a structured gate instead of grepping logs.
    AssetLoadMissingBundleAssetRequestsGe {
        min: u64,
    },
    /// True when `debug.resource_loading.asset_load.stale_manifest_requests >= min`.
    ///
    /// This is intended for native/package-dev file-backed manifest lanes where the logical
    /// bundle/key mapping still exists but the manifest-backed file path has gone stale.
    AssetLoadStaleManifestRequestsGe {
        min: u64,
    },
    /// True when `debug.resource_loading.asset_load.unsupported_file_requests >= min`.
    ///
    /// This is intended to gate portable capability degradations where file locators must stay
    /// unsupported on targets like wasm.
    AssetLoadUnsupportedFileRequestsGe {
        min: u64,
    },
    /// True when `debug.resource_loading.asset_load.unsupported_url_requests >= min`.
    AssetLoadUnsupportedUrlRequestsGe {
        min: u64,
    },
    /// True when
    /// `debug.resource_loading.asset_load.external_reference_unavailable_requests >= min`.
    ///
    /// This is intended for byte-only asset surfaces that should not silently claim an external
    /// reference exists.
    AssetLoadExternalReferenceUnavailableRequestsGe {
        min: u64,
    },
    /// True when `debug.resource_loading.asset_load.revision_change_requests >= min`.
    ///
    /// This is intended for hot-reload / invalidation flows where we want to observe that a
    /// locator revision actually changed across snapshots.
    AssetLoadRevisionChangeRequestsGe {
        min: u64,
    },
    /// True when `debug.resource_loading.asset_load.recent[*].outcome_kind` contains
    /// `outcome_kind`.
    ///
    /// Supported values currently mirror the debug snapshot surface:
    /// - `resolved`
    /// - `missing`
    /// - `stale_manifest`
    /// - `unsupported_locator_kind`
    /// - `external_reference_unavailable`
    /// - `resolver_unavailable`
    /// - `access_denied`
    /// - `message`
    AssetLoadRecentOutcomeSeen {
        outcome_kind: String,
    },
    /// True when `debug.resource_loading.asset_load.recent[*].revision_transition` contains
    /// `transition`.
    ///
    /// Supported values currently mirror the debug snapshot surface:
    /// - `initial`
    /// - `stable`
    /// - `changed`
    AssetLoadRecentRevisionTransitionSeen {
        transition: String,
    },
    /// True when `debug.resource_loading.font_environment.bundled_baseline_source == source`.
    ///
    /// Supported values currently mirror the debug snapshot surface:
    /// - `none`
    /// - `bundled_profile`
    BundledFontBaselineSourceIs {
        source: String,
    },
    /// True when the runner has observed an OS accessibility activation request for the current
    /// window.
    ///
    /// This is intended to gate “AccessKit ↔ OS AX is actually live” rather than only asserting
    /// that the app has an internal semantics tree.
    RunnerAccessibilityActivated,
    VisibleInWindow {
        target: UiSelectorV1,
    },
    BoundsWithinWindow {
        target: UiSelectorV1,
        #[serde(default)]
        padding_px: f32,
        /// Optional per-edge padding (added on top of `padding_px`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        padding_insets_px: Option<UiPaddingInsetsV1>,
        #[serde(default)]
        eps_px: f32,
    },
    /// True when the runtime-published IME cursor area for the focused text input is fully within
    /// the window bounds (minus the specified padding).
    ///
    /// This is intended as a stable regression gate for keyboard-avoidance policies: after
    /// occlusion insets change, the focused caret/cursor area should remain inside the visible
    /// rect derived from safe-area + occlusion.
    TextInputImeCursorAreaWithinWindow {
        #[serde(default)]
        padding_px: f32,
        /// Optional per-edge padding (added on top of `padding_px`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        padding_insets_px: Option<UiPaddingInsetsV1>,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsMinSize {
        target: UiSelectorV1,
        #[serde(default)]
        min_w_px: f32,
        #[serde(default)]
        min_h_px: f32,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsMaxSize {
        target: UiSelectorV1,
        #[serde(default)]
        max_w_px: f32,
        #[serde(default)]
        max_h_px: f32,
        #[serde(default)]
        eps_px: f32,
    },
    /// True when both targets exist and their bounds match within `eps_px`.
    ///
    /// This is primarily used to gate “hit box vs visual chrome” regressions where a pressable
    /// can stretch but an inner chrome surface must continue to fill the same box.
    BoundsApproxEqual {
        a: UiSelectorV1,
        b: UiSelectorV1,
        #[serde(default)]
        eps_px: f32,
    },
    /// True when both targets exist and their bounds centers match within `eps_px`.
    ///
    /// This is primarily used to gate “stretched hit box + centered fixed chrome” contracts where
    /// the interactive surface can grow via flex/grid/min touch target, but the inner visual chrome
    /// remains fixed-size and centered.
    BoundsCenterApproxEqual {
        a: UiSelectorV1,
        b: UiSelectorV1,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsNonOverlapping {
        a: UiSelectorV1,
        b: UiSelectorV1,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsOverlapping {
        a: UiSelectorV1,
        b: UiSelectorV1,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsOverlappingX {
        a: UiSelectorV1,
        b: UiSelectorV1,
        #[serde(default)]
        eps_px: f32,
    },
    BoundsOverlappingY {
        a: UiSelectorV1,
        b: UiSelectorV1,
        #[serde(default)]
        eps_px: f32,
    },
    /// True when the diagnostics event ring contains an event whose recorded kind equals `kind`.
    ///
    /// This is intentionally a coarse predicate: it is meant to gate “a platform completion was
    /// delivered” without requiring a dedicated predicate per event type.
    EventKindSeen {
        event_kind: String,
    },
    /// True when the app snapshot field addressed by JSON Pointer `pointer` equals `value`.
    ///
    /// This predicate reads the best-effort `app_snapshot` payload published by the app into
    /// diagnostics snapshots. The pointer uses RFC 6901 JSON Pointer syntax (for example:
    /// `/shell/settings_open` or `/shell/last_action`).
    ///
    /// If the app does not publish an `app_snapshot`, or the pointer does not resolve to a value,
    /// this predicate evaluates to false.
    AppSnapshotFieldEquals {
        pointer: String,
        value: serde_json::Value,
    },
    /// True when the diagnostics runtime has observed at least `n` windows.
    ///
    /// This is intended for multi-window scripted repros (tear-off, auxiliary windows).
    KnownWindowCountGe {
        n: u32,
    },
    /// True when the diagnostics runtime has observed exactly `n` windows.
    ///
    /// This is useful for degradation gates where creating additional windows must be prevented
    /// (e.g. Wayland-safe docking tear-off degradation).
    KnownWindowCountIs {
        n: u32,
    },
    /// True when the latest diagnostics snapshot includes platform capability information and it
    /// reports `ui.window_hover_detection == quality`.
    ///
    /// Supported qualities:
    /// - `none`
    /// - `best_effort`
    /// - `reliable`
    PlatformUiWindowHoverDetectionIs {
        quality: String,
    },
    /// True when the platform-level "receiver window at cursor" probe reports that the active
    /// cursor position would be routed to `window` (best-effort).
    ///
    /// This predicate is intended to gate runner-level hit-test passthrough behavior (e.g.
    /// `WM_NCHITTEST` on Win32) without relying on pixel screenshots.
    ///
    /// Capability-gated behind `diag.platform_window_receiver_at_cursor_v1`.
    PlatformWindowReceiverAtCursorIs {
        window: UiWindowTargetV1,
    },
    /// True when the effective (clamped) OS window style for `window` matches the provided facets.
    ///
    /// This predicate is capability-gated and intended for non-pixel regression gates for utility
    /// windows (frameless/transparent/always-on-top posture).
    WindowStyleEffectiveIs {
        window: UiWindowTargetV1,
        style: UiWindowStyleMatchV1,
    },
    /// True when the effective (clamped) OS window background material for `window` matches `material`.
    ///
    /// This predicate is capability-gated and intended to gate deterministic degradation paths
    /// when OS materials are unsupported.
    WindowBackgroundMaterialEffectiveIs {
        window: UiWindowTargetV1,
        material: UiWindowBackgroundMaterialRequestV1,
    },
    /// True when the latest docking diagnostics report an active dock drag whose `current_window`
    /// matches `window`.
    DockDragCurrentWindowIs {
        window: UiWindowTargetV1,
    },
    /// True when the latest diagnostics report an active dock drag whose drag kind matches `kind`.
    ///
    /// Supported kinds:
    /// - `dock_panel`
    /// - `dock_tabs`
    DockDragKindIs {
        drag_kind: String,
    },
    /// True when the latest docking diagnostics report an active dock drag whose runner-owned
    /// moving window matches `window`.
    ///
    /// This is intended for ImGui-style multi-window docking where a torn-off window follows the
    /// cursor while dragging.
    DockDragMovingWindowIs {
        window: UiWindowTargetV1,
    },
    /// True when the latest docking diagnostics report an active dock drag whose
    /// "window under moving window" matches `window`.
    ///
    /// This allows scripts to gate "peek-behind" selection paths without reinterpreting
    /// `dock_drag_current_window_is` (which remains the runner's primary hover/drop routing
    /// target).
    DockDragWindowUnderMovingWindowIs {
        window: UiWindowTargetV1,
    },
    /// True when the latest docking diagnostics report an active dock drag session.
    DockDragActiveIs {
        active: bool,
    },
    /// True when the latest docking diagnostics report a dock drag session with an ImGui-style
    /// "transparent payload" applied to the moving window (e.g. reduced opacity and/or
    /// click-through hit-test passthrough while the dock-floating window follows the cursor).
    DockDragTransparentPayloadAppliedIs {
        applied: bool,
    },
    /// True when the latest docking diagnostics report that the runner successfully applied
    /// click-through hit-test passthrough for the moving window during transparent payload.
    DockDragTransparentPayloadHitTestPassthroughAppliedIs {
        applied: bool,
    },
    /// True when the latest docking diagnostics report a dock drag session whose hovered-window
    /// selection source matches `source`.
    ///
    /// This is primarily intended to gate multi-window docking hand-feel regressions: on
    /// platforms that claim `ui.window_hover_detection=reliable`, we want to ensure the runner is
    /// using an OS-backed "window under cursor" provider rather than a heuristic fallback.
    ///
    /// Supported sources:
    /// - `platform`: any OS-backed platform hover provider
    /// - `platform_win32`
    /// - `platform_macos`
    /// - `latched`
    /// - `heuristic`: any heuristic fallback
    /// - `heuristic_z_order`
    /// - `heuristic_rects`
    /// - `unknown`
    DockDragWindowUnderCursorSourceIs {
        source: String,
    },
    /// True when the latest docking diagnostics report a dock drag session whose
    /// "window under moving window" selection source matches `source`.
    ///
    /// Supported sources:
    /// - `platform`: any OS-backed platform hover provider
    /// - `platform_win32`
    /// - `platform_macos`
    /// - `latched`
    /// - `heuristic`: any heuristic fallback
    /// - `heuristic_z_order`
    /// - `heuristic_rects`
    /// - `unknown`
    DockDragWindowUnderMovingWindowSourceIs {
        source: String,
    },
    /// True when the latest docking diagnostics report an active in-window floating drag session.
    ///
    /// This is intended to gate "floating window" hand-feel regressions without relying on pixels.
    DockFloatingDragActiveIs {
        active: bool,
    },
    /// True when the current docking drop preview kind matches `kind`.
    ///
    /// This predicate reads the window-local `DockDropResolveDiagnostics` snapshot published into
    /// `WindowInteractionDiagnosticsStore` by policy-heavy ecosystem crates (e.g. docking).
    ///
    /// Supported kinds:
    /// - `wrap_binary`
    /// - `insert_into_split`
    DockDropPreviewKindIs {
        preview_kind: String,
    },
    /// True when the current docking drop resolve source matches `source`.
    ///
    /// This predicate reads the window-local `DockDropResolveDiagnostics` snapshot published into
    /// `WindowInteractionDiagnosticsStore` by policy-heavy ecosystem crates (e.g. docking).
    ///
    /// Supported sources:
    /// - `invert_docking`
    /// - `outside_window`
    /// - `float_zone`
    /// - `layout_bounds_miss`
    /// - `latched_previous_hover`
    /// - `tab_bar`
    /// - `floating_title_bar`
    /// - `outer_hint_rect`
    /// - `inner_hint_rect`
    /// - `none`
    DockDropResolveSourceIs {
        source: String,
    },
    /// True when the current docking drop resolve has (or does not have) a resolved target.
    ///
    /// This is useful for policy-gated no-drop zones: scripts can assert that the pointer is over
    /// a *candidate* region (via `dock_drop_resolve_source_is`) while `resolved` stays `None`.
    DockDropResolvedIsSome {
        some: bool,
    },
    /// True when the current docking drop resolve has a resolved target whose `zone` matches
    /// `zone`.
    ///
    /// Supported zones:
    /// - `center`
    /// - `left`
    /// - `right`
    /// - `top`
    /// - `bottom`
    DockDropResolvedZoneIs {
        zone: String,
    },
    /// True when the current docking drop resolve has a resolved target whose `insert_index`
    /// matches `index`.
    ///
    /// This is intended to gate "drop at end" semantics (e.g. `index == tab_count`) without
    /// relying on pixels.
    DockDropResolvedInsertIndexIs {
        index: u32,
    },
    /// True when the latest docking diagnostics report whether the active tab strip is overflowed.
    ///
    /// This predicate reads the best-effort `tab_strip_active_visibility` snapshot recorded by
    /// docking into `WindowInteractionDiagnosticsStore`.
    DockTabStripActiveOverflowIs {
        overflow: bool,
    },
    /// True when the latest docking diagnostics report whether the active tab is visible at the
    /// current tab scroll position.
    ///
    /// This predicate is intended to gate the editor-grade invariant:
    /// "selecting a tab (including via overflow menu) must scroll it into view".
    DockTabStripActiveVisibleIs {
        visible: bool,
    },
    /// True when the latest docking diagnostics report `tab_strip_active_visibility.scroll >= px`.
    ///
    /// This predicate is intended to gate edge auto-scroll during tab drags in overflowed tab
    /// strips, without relying on pixels.
    DockTabStripActiveScrollPxGe {
        px: f32,
    },
    /// True when the latest docking diagnostics report `tab_strip_active_visibility.scroll <= px`.
    ///
    /// This is primarily intended to assert the initial scroll state in scripted regressions.
    DockTabStripActiveScrollPxLe {
        px: f32,
    },
    /// True when the latest workspace diagnostics report whether the active tab strip is overflowed.
    ///
    /// This predicate reads the best-effort `workspace_interaction.tab_strip_active_visibility`
    /// snapshot recorded into `WindowInteractionDiagnosticsStore`.
    WorkspaceTabStripActiveOverflowIs {
        overflow: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pane_id: Option<String>,
    },
    /// True when the latest workspace diagnostics report whether the active tab is visible at the
    /// current tab scroll position.
    ///
    /// This predicate is intended to gate the editor-grade invariant:
    /// "selecting a tab (including via overflow menu) must scroll it into view".
    WorkspaceTabStripActiveVisibleIs {
        visible: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pane_id: Option<String>,
    },
    /// True when the latest workspace diagnostics report `tab_strip_active_visibility.scroll_x >= px`.
    ///
    /// This predicate reads the best-effort `workspace_interaction.tab_strip_active_visibility`
    /// snapshot recorded into `WindowInteractionDiagnosticsStore`.
    WorkspaceTabStripActiveScrollPxGe {
        px: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pane_id: Option<String>,
    },
    /// True when the latest workspace diagnostics report `tab_strip_active_visibility.scroll_x <= px`.
    ///
    /// This predicate reads the best-effort `workspace_interaction.tab_strip_active_visibility`
    /// snapshot recorded into `WindowInteractionDiagnosticsStore`.
    WorkspaceTabStripActiveScrollPxLe {
        px: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pane_id: Option<String>,
    },
    /// True when the latest workspace diagnostics report an active tab strip drag session.
    ///
    /// This predicate reads the best-effort `workspace_interaction.tab_strip_drag` snapshot
    /// recorded into `WindowInteractionDiagnosticsStore`.
    WorkspaceTabStripDragActiveIs {
        active: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pane_id: Option<String>,
    },
    /// True when the latest workspace diagnostics report whether a tab strip drag session is armed
    /// (i.e. tracking a pointer that may become a drag on move threshold).
    ///
    /// This predicate reads the best-effort `workspace_interaction.tab_strip_drag` snapshot
    /// recorded into `WindowInteractionDiagnosticsStore`.
    WorkspaceTabStripDragArmedIs {
        armed: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pane_id: Option<String>,
    },
    /// True when the latest dock graph stats snapshot reports a canonical-form layout.
    DockGraphCanonicalIs {
        canonical: bool,
    },
    /// True when the latest dock graph stats snapshot reports nested same-axis split children.
    DockGraphHasNestedSameAxisSplitsIs {
        has_nested: bool,
    },
    /// True when the latest dock graph stats snapshot reports `node_count <= max`.
    ///
    /// This is intended for scripted regression gates that want to ensure repeated dock operations
    /// do not accidentally allocate unbounded structure (e.g. legacy "wrap" behavior that deepens
    /// the split tree).
    DockGraphNodeCountLe {
        max: u32,
    },
    /// True when the latest dock graph stats snapshot reports `max_split_depth <= max`.
    DockGraphMaxSplitDepthLe {
        max: u32,
    },
    /// True when the latest dock graph signature snapshot matches `signature`.
    ///
    /// This signature is intended to be stable across runs and platforms:
    /// - it does not include split fractions (pointer-driven and DPI-sensitive),
    /// - it does not include floating window rects (platform-dependent).
    DockGraphSignatureIs {
        signature: String,
    },
    /// True when the latest dock graph signature snapshot contains `needle` as a substring.
    ///
    /// This is useful for large layouts where asserting the entire signature string would be too
    /// verbose.
    DockGraphSignatureContains {
        needle: String,
    },
    /// True when the latest dock graph signature snapshot does **not** contain `needle` as a
    /// substring.
    DockGraphSignatureNotContains {
        needle: String,
    },
    /// True when the latest dock graph signature fingerprint matches `fingerprint64`.
    DockGraphSignatureFingerprint64Is {
        fingerprint64: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSemanticsNumericFieldV1 {
    Value,
    Min,
    Max,
    Step,
    Jump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSemanticsScrollFieldV1 {
    X,
    XMin,
    XMax,
    Y,
    YMin,
    YMax,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOptionalRootStateV1 {
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root_z_index: Option<u32>,
    },
    RoleAndPath {
        role: String,
        name: String,
        ancestors: Vec<UiRoleAndNameV1>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root_z_index: Option<u32>,
    },
    TestId {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root_z_index: Option<u32>,
    },
    GlobalElementId {
        element: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root_z_index: Option<u32>,
    },
    NodeId {
        node: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root_z_index: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRoleAndNameV1 {
    pub role: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsNodeGetV1 {
    pub schema_version: u32,
    pub window: u64,
    pub node_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsNodeGetAckV1 {
    pub schema_version: u32,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub window: u64,
    pub node_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_fingerprint: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<serde_json::Value>,
    #[serde(default)]
    pub children: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestExplainV1 {
    pub schema_version: u32,
    pub window: u64,
    pub target: UiSelectorV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestExplainAckV1 {
    pub schema_version: u32,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub window: u64,
    pub target: UiSelectorV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_fingerprint: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hittable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_test: Option<UiHitTestTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInspectConfigV1 {
    pub schema_version: u32,
    pub enabled: bool,
    #[serde(default = "serde_default_true")]
    pub consume_clicks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsBundleDumpV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Optional per-dump cap on how many snapshots are included in the exported bundle.
    ///
    /// When omitted, the runtime uses its configured dump cap (typically
    /// `FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS` for script-driven dumps, and
    /// `FRET_DIAG_MAX_SNAPSHOTS` for manual dumps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_snapshots: Option<u32>,
}

/// Request that the app exits as soon as possible.
///
/// This is intended for transport-neutral "exit after run" behavior in CI / scripted automation
/// flows where relying on large timeouts is undesirable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsAppExitRequestV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional delay before triggering exit, expressed in wall-clock milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsBundleDumpedV1 {
    pub schema_version: u32,
    pub exported_unix_ms: u64,
    pub out_dir: String,
    pub dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<serde_json::Value>,
    /// Optional chunked representation of the embedded bundle JSON.
    ///
    /// When present, the runtime may send multiple `bundle.dumped` messages (same `exported_unix_ms`
    /// + `dir`) each carrying one chunk. Tooling should reassemble chunks in order to reconstruct
    ///   the full JSON payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_json_chunk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_json_chunk_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_json_chunk_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsScreenshotRequestV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default = "default_capture_screenshot_timeout_frames")]
    pub timeout_frames: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsScreenshotResultV1 {
    pub schema_version: u32,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub request_id: String,
    pub window: u64,
    pub bundle_dir_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshots_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<serde_json::Value>,
}

/// GPU screenshot request written by the in-app diagnostics runtime, consumed by desktop runners.
///
/// This is the transport between:
///
/// - `ecosystem/fret-bootstrap` (writer; script steps + DevTools WS bridge), and
/// - `crates/fret-launch` (reader; runner-owned GPU readback + PNG encoding).
///
/// Keeping this schema in `fret-diag-protocol` avoids "forked" JSON parsing logic across crates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagScreenshotRequestV1 {
    pub schema_version: u32,
    pub out_dir: String,
    pub bundle_dir_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub windows: Vec<DiagScreenshotWindowRequestV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagScreenshotWindowRequestV1 {
    pub window: u64,
    pub tick_id: u64,
    pub frame_id: u64,
    #[serde(default = "serde_default_one_f64")]
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagScreenshotResultFileV1 {
    #[serde(default = "default_diag_screenshot_schema_version")]
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_unix_ms: Option<u64>,
    #[serde(default)]
    pub completed: Vec<DiagScreenshotResultEntryV1>,
}

impl Default for DiagScreenshotResultFileV1 {
    fn default() -> Self {
        Self {
            schema_version: default_diag_screenshot_schema_version(),
            updated_unix_ms: None,
            completed: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagScreenshotResultEntryV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub bundle_dir_name: String,
    pub window: u64,
    pub tick_id: u64,
    pub frame_id: u64,
    pub scale_factor: f32,
    pub file: String,
    pub width_px: u32,
    pub height_px: u32,
    pub completed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiArtifactStatsV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_json_bytes: Option<u64>,
    #[serde(default)]
    pub window_count: u64,
    #[serde(default)]
    pub event_count: u64,
    #[serde(default)]
    pub snapshot_count: u64,
    #[serde(default)]
    pub max_snapshots: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dump_max_snapshots: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScriptResultV1 {
    pub schema_version: u32,
    pub run_id: u64,
    pub updated_unix_ms: u64,
    pub window: Option<u64>,
    pub stage: UiScriptStageV1,
    pub step_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<UiScriptEvidenceV1>,
    pub last_bundle_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_bundle_artifact: Option<UiArtifactStatsV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiScriptEvidenceV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_log: Vec<UiScriptEventLogEntryV1>,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub event_log_dropped: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities_check: Option<UiCapabilitiesCheckV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selector_resolution_trace: Vec<UiSelectorResolutionTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hit_test_trace: Vec<UiHitTestTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub click_stable_trace: Vec<UiClickStableTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounds_stable_trace: Vec<UiBoundsStableTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub focus_trace: Vec<UiFocusTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shortcut_routing_trace: Vec<UiShortcutRoutingTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command_dispatch_trace: Vec<UiCommandDispatchTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlay_placement_trace: Vec<UiOverlayPlacementTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub web_ime_trace: Vec<UiWebImeTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ime_event_trace: Vec<UiImeEventTraceEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCapabilitiesCheckV1 {
    pub schema_version: u32,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub available: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScriptEventLogEntryV1 {
    pub unix_ms: u64,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_dir: Option<String>,
    /// When available, identifies the window that observed/emitted this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<u64>,
    /// When available, the app tick id at the time of the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_id: Option<u64>,
    /// When available, the app frame id at the time of the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<u64>,
    /// Optional per-window snapshot sequence hint (may be resolved by tooling from `bundle.index.json`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_snapshot_seq: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSelectorResolutionTraceEntryV1 {
    pub step_index: u32,
    pub selector: UiSelectorV1,
    #[serde(default)]
    pub match_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<UiSelectorResolutionCandidateV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSelectorResolutionCandidateV1 {
    pub node_id: u64,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiPointV1 {
    pub x_px: f32,
    pub y_px: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiRectV1 {
    pub x_px: f32,
    pub y_px: f32,
    pub w_px: f32,
    pub h_px: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestTraceEntryV1 {
    pub step_index: u32,
    pub selector: UiSelectorV1,
    pub position: UiPointV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_bounds: Option<UiRectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_node_id: Option<u64>,
    /// Debug-only path from the root to `hit_node_id` (inclusive).
    ///
    /// Treat node ids as in-run references only; they are not stable across runs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hit_node_path: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_semantics_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_semantics_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes_intended: Option<bool>,
    /// Best-effort: whether the hit-test path contains the intended node id.
    ///
    /// Useful for diagnosing “clicked the right region but an overlay/capture blocked delivery”.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_path_contains_intended: Option<bool>,
    /// Best-effort attribution for why the intended target did not receive injected input.
    ///
    /// This is a convenience field intended for triage tools and AI. Prefer inspecting the raw
    /// evidence fields when debugging novel cases.
    ///
    /// Stable strings (start small; expand only when evidence becomes more actionable):
    /// - `modal_barrier` (a modal barrier is active)
    /// - `focus_barrier` (a focus barrier is active)
    /// - `pointer_capture` (pointer capture is active)
    /// - `pointer_occlusion` (pointer occlusion blocks underlay input)
    /// - `no_hit` (hit-test produced no node)
    /// - `miss` (hit-test landed on a different node)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_reason: Option<String>,
    /// Best-effort in-run root reference associated with `blocking_reason` (when applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_root: Option<u64>,
    /// Best-effort layer id associated with `blocking_reason` (when applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_layer_id: Option<u64>,
    /// Best-effort human-readable explanation for `blocking_reason`.
    ///
    /// This is intended for fast triage and AI; treat it as a hint rather than a contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_explain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barrier_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_barrier_root: Option<u64>,
    /// The input arbitration snapshot at the time this trace entry was recorded.
    ///
    /// These fields are primarily useful for explaining why injected input did not reach the
    /// underlay (pointer occlusion/capture/focus barriers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion_layer_id: Option<u64>,
    /// Best-effort pointer occlusion owner (in-run references only).
    ///
    /// When `pointer_occlusion_layer_id` is present, these fields attempt to resolve the layer
    /// root to a semantics node for easier triage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion_bounds: Option<UiRectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_layer_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_multiple_layers: Option<bool>,
    /// Best-effort pointer capture owner (in-run references only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_bounds: Option<UiRectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope_roots: Vec<UiHitTestScopeRootEvidenceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiClickStableTraceEntryV1 {
    pub step_index: u32,
    pub stable_required: u32,
    pub stable_count: u32,
    pub moved_px: f32,
    pub max_move_px: f32,
    pub remaining_frames: u32,
    pub hit_test: UiHitTestTraceEntryV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiBoundsStableTraceEntryV1 {
    pub step_index: u32,
    pub selector: UiSelectorV1,
    pub stable_required: u32,
    pub stable_count: u32,
    pub moved_px: f32,
    pub max_move_px: f32,
    pub remaining_frames: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<UiRectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestScopeRootEvidenceV1 {
    pub kind: String,
    pub root: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks_underlay_input: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit_testable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFocusTraceEntryV1 {
    pub step_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_input_snapshot: Option<UiTextInputSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modal_barrier_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_barrier_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion_layer_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_layer_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_capture_multiple_layers: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_node_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matches_expected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTextInputSnapshotV1 {
    #[serde(default)]
    pub focus_is_text_input: bool,
    #[serde(default)]
    pub is_composing: bool,
    #[serde(default)]
    pub text_len_utf16: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_cursor_area: Option<UiRectV1>,
    /// Optional IME surrounding text excerpt metadata (bytes).
    ///
    /// This is derived from `WindowTextInputSnapshot.surrounding_text` and is intended for
    /// lightweight debugging without embedding potentially sensitive text contents in bundles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_surrounding_text_len_bytes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_surrounding_cursor_bytes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_surrounding_anchor_bytes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiShortcutRoutingTraceEntryV1 {
    pub step_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default)]
    pub frame_id: u64,
    pub phase: String,
    #[serde(default)]
    pub deferred: bool,
    #[serde(default)]
    pub focus_is_text_input: bool,
    #[serde(default)]
    pub ime_composing: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_contexts: Vec<String>,
    pub key: String,
    pub modifiers: UiKeyModifiersV1,
    pub repeat: bool,
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_sequence_len: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlayPlacementTraceKindV1 {
    AnchoredPanel,
    PlacedRect,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiShortcutRoutingTraceQueryV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_composing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_is_text_input: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCommandDispatchTraceEntryV1 {
    pub step_index: u32,
    pub frame_id: u64,
    pub command: String,
    pub handled: bool,
    /// Best-effort handler scope classification (ADR 0307).
    ///
    /// Expected values: `"widget"`, `"window"`, `"app"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_scope: Option<String>,
    /// Whether the command was handled by a runner/driver integration layer (not by a UI element).
    #[serde(default)]
    pub handled_by_driver: bool,
    #[serde(default)]
    pub stopped: bool,
    #[serde(default)]
    pub source_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_element: Option<u64>,
    /// Best-effort stable selector attribution for pointer-triggered dispatch.
    ///
    /// This is intended to help scripted diagnostics answer:
    /// “which `test_id` caused this command to dispatch?”
    ///
    /// Notes:
    /// - This is a best-effort hint (additive). Tooling should fall back to correlating
    ///   `source_element` with the semantics snapshot if needed.
    /// - When available, this is usually populated from the hit-test trace recorded for the
    ///   injected pointer step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_element: Option<u64>,
    /// Best-effort stable selector attribution for the first widget that handled the command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_test_id: Option<String>,
    #[serde(default)]
    pub started_from_focus: bool,
    #[serde(default)]
    pub used_default_root_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiCommandDispatchTraceQueryV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_driver: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_from_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_default_root_fallback: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutDirectionV1 {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySideV1 {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlayAlignV1 {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlayStickyModeV1 {
    Partial,
    Always,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiEdgesV1 {
    pub top_px: f32,
    pub right_px: f32,
    pub bottom_px: f32,
    pub left_px: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiSizeV1 {
    pub w_px: f32,
    pub h_px: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiOverlayOffsetV1 {
    pub main_axis_px: f32,
    pub cross_axis_px: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment_axis_px: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiOverlayShiftV1 {
    pub main_axis: bool,
    pub cross_axis: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiOverlayArrowLayoutV1 {
    pub side: UiOverlaySideV1,
    pub offset_px: f32,
    pub alignment_offset_px: f32,
    pub center_offset_px: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiOverlayPlacementTraceEntryV1 {
    AnchoredPanel {
        step_index: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        #[serde(default)]
        frame_id: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        overlay_root_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor_element: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor_test_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_element: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_test_id: Option<String>,

        outer_input: UiRectV1,
        outer_collision: UiRectV1,
        anchor: UiRectV1,
        desired: UiSizeV1,
        side_offset_px: f32,
        preferred_side: UiOverlaySideV1,
        align: UiOverlayAlignV1,
        direction: UiLayoutDirectionV1,
        sticky: UiOverlayStickyModeV1,
        offset: UiOverlayOffsetV1,
        shift: UiOverlayShiftV1,
        collision_padding: UiEdgesV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        collision_boundary: Option<UiRectV1>,
        gap_px: f32,

        preferred_rect: UiRectV1,
        flipped_rect: UiRectV1,
        #[serde(default)]
        preferred_fits_without_main_clamp: bool,
        #[serde(default)]
        flipped_fits_without_main_clamp: bool,
        #[serde(default)]
        preferred_available_main_px: f32,
        #[serde(default)]
        flipped_available_main_px: f32,
        chosen_side: UiOverlaySideV1,
        chosen_rect: UiRectV1,
        rect_after_shift: UiRectV1,
        shift_delta: UiPointV1,
        final_rect: UiRectV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arrow: Option<UiOverlayArrowLayoutV1>,
    },
    PlacedRect {
        step_index: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        #[serde(default)]
        frame_id: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        overlay_root_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor_element: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor_test_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_element: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_test_id: Option<String>,
        outer: UiRectV1,
        anchor: UiRectV1,
        placed: UiRectV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        side: Option<UiOverlaySideV1>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiOverlayPlacementTraceQueryV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<UiOverlayPlacementTraceKindV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay_root_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_test_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_side: Option<UiOverlaySideV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_side: Option<UiOverlaySideV1>,
    /// For `kind=anchored_panel`, whether the solver flipped away from `preferred_side`.
    /// Equivalent to `chosen_side != preferred_side` when both are available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flipped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<UiOverlayAlignV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticky: Option<UiOverlayStickyModeV1>,
}

/// Debug-only snapshot for the wasm textarea IME bridge (ADR 0180).
///
/// This is intended for diagnostics evidence and is not a normative contract surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWebImeTraceEntryV1 {
    pub step_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub composing: bool,
    #[serde(default)]
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
    pub textarea_selection_start_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_end_utf16: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_area: Option<UiRectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_anchor_px: Option<(f32, f32)>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_cursor_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_len: Option<u32>,

    #[serde(default)]
    pub beforeinput_seen: u64,
    #[serde(default)]
    pub input_seen: u64,
    #[serde(default)]
    pub suppressed_input_seen: u64,
    #[serde(default)]
    pub composition_start_seen: u64,
    #[serde(default)]
    pub composition_update_seen: u64,
    #[serde(default)]
    pub composition_end_seen: u64,
    #[serde(default)]
    pub cursor_area_set_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiImeEventTraceEntryV1 {
    pub step_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preedit_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preedit_cursor: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_surrounding: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScriptStageV1 {
    Queued,
    Running,
    Passed,
    Failed,
}

fn serde_default_true() -> bool {
    true
}

fn serde_default_one_f64() -> f64 {
    1.0
}

fn default_diag_screenshot_schema_version() -> u32 {
    1
}

fn is_zero_u64(v: &u64) -> bool {
    *v == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devtools_app_exit_request_serializes_minimally() {
        let value = serde_json::to_value(DevtoolsAppExitRequestV1 {
            schema_version: 1,
            reason: None,
            delay_ms: None,
        })
        .unwrap();
        assert_eq!(value, serde_json::json!({ "schema_version": 1 }));
    }

    #[test]
    fn predicate_runner_accessibility_activated_serializes_and_deserializes() {
        let value = serde_json::to_value(UiPredicateV1::RunnerAccessibilityActivated).unwrap();
        assert_eq!(
            value,
            serde_json::json!({ "kind": "runner_accessibility_activated" })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::RunnerAccessibilityActivated
        ));
    }

    #[test]
    fn predicate_app_snapshot_field_equals_serializes_minimally() {
        let value = serde_json::to_value(UiPredicateV1::AppSnapshotFieldEquals {
            pointer: "/shell/settings_open".to_string(),
            value: serde_json::json!(true),
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "app_snapshot_field_equals",
                "pointer": "/shell/settings_open",
                "value": true,
            })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::AppSnapshotFieldEquals { .. }
        ));
    }

    #[test]
    fn predicate_bounds_approx_equal_serializes_and_deserializes() {
        let value = serde_json::to_value(UiPredicateV1::BoundsApproxEqual {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
                root_z_index: None,
            },
            eps_px: 1.0,
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "bounds_approx_equal",
                "a": { "kind": "test_id", "id": "a" },
                "b": { "kind": "test_id", "id": "b" },
                "eps_px": 1.0
            })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(roundtrip, UiPredicateV1::BoundsApproxEqual { .. }));
    }

    #[test]
    fn predicate_bounds_center_approx_equal_serializes_and_deserializes() {
        let value = serde_json::to_value(UiPredicateV1::BoundsCenterApproxEqual {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
                root_z_index: None,
            },
            eps_px: 1.0,
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "bounds_center_approx_equal",
                "a": { "kind": "test_id", "id": "a" },
                "b": { "kind": "test_id", "id": "b" },
                "eps_px": 1.0
            })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::BoundsCenterApproxEqual { .. }
        ));
    }

    #[test]
    fn predicate_exists_under_serializes_minimally() {
        let value = serde_json::to_value(UiPredicateV1::ExistsUnder {
            scope: UiSelectorV1::TestId {
                id: "scope".to_string(),
                root_z_index: None,
            },
            target: UiSelectorV1::TestId {
                id: "target".to_string(),
                root_z_index: None,
            },
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "exists_under",
                "scope": { "kind": "test_id", "id": "scope" },
                "target": { "kind": "test_id", "id": "target" },
            })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(roundtrip, UiPredicateV1::ExistsUnder { .. }));
    }

    #[test]
    fn predicate_value_equals_serializes_minimally() {
        let value = serde_json::to_value(UiPredicateV1::ValueEquals {
            target: UiSelectorV1::TestId {
                id: "name".to_string(),
                root_z_index: None,
            },
            text: "Alice".to_string(),
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "value_equals",
                "target": { "kind": "test_id", "id": "name" },
                "text": "Alice",
            })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(roundtrip, UiPredicateV1::ValueEquals { .. }));
    }

    #[test]
    fn predicate_focused_descendant_is_serializes_minimally() {
        let value = serde_json::to_value(UiPredicateV1::FocusedDescendantIs {
            scope: UiSelectorV1::TestId {
                id: "dialog".to_string(),
                root_z_index: None,
            },
            target: UiSelectorV1::TestId {
                id: "close".to_string(),
                root_z_index: None,
            },
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "focused_descendant_is",
                "scope": { "kind": "test_id", "id": "dialog" },
                "target": { "kind": "test_id", "id": "close" },
            })
        );

        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::FocusedDescendantIs { .. }
        ));
    }

    #[test]
    fn predicate_dock_tab_strip_scroll_predicates_serialize_and_deserialize() {
        let value =
            serde_json::to_value(UiPredicateV1::DockTabStripActiveScrollPxGe { px: 12.0 }).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "kind": "dock_tab_strip_active_scroll_px_ge",
                "px": 12.0
            })
        );
        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::DockTabStripActiveScrollPxGe { .. }
        ));

        let value =
            serde_json::to_value(UiPredicateV1::DockTabStripActiveScrollPxLe { px: 0.0 }).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "kind": "dock_tab_strip_active_scroll_px_le",
                "px": 0.0
            })
        );
        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::DockTabStripActiveScrollPxLe { .. }
        ));
    }

    #[test]
    fn predicate_workspace_tab_strip_scroll_predicates_serialize_and_deserialize() {
        let value = serde_json::to_value(UiPredicateV1::WorkspaceTabStripActiveScrollPxGe {
            px: 12.0,
            pane_id: None,
        })
        .unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "kind": "workspace_tab_strip_active_scroll_px_ge",
                "px": 12.0
            })
        );
        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::WorkspaceTabStripActiveScrollPxGe { .. }
        ));

        let value = serde_json::to_value(UiPredicateV1::WorkspaceTabStripActiveScrollPxLe {
            px: 0.0,
            pane_id: Some("pane-a".to_string()),
        })
        .unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "kind": "workspace_tab_strip_active_scroll_px_le",
                "px": 0.0,
                "pane_id": "pane-a",
            })
        );
        let roundtrip: UiPredicateV1 = serde_json::from_value(value).unwrap();
        assert!(matches!(
            roundtrip,
            UiPredicateV1::WorkspaceTabStripActiveScrollPxLe { .. }
        ));
    }

    #[test]
    fn diag_screenshot_request_round_trips_and_defaults_scale_factor() {
        let json = serde_json::json!({
            "schema_version": 1,
            "out_dir": "target/fret-diag",
            "bundle_dir_name": "1700000-bundle",
            "request_id": "req-1",
            "windows": [{
                "window": 123,
                "tick_id": 1,
                "frame_id": 2
            }]
        });
        let parsed: DiagScreenshotRequestV1 = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.windows.len(), 1);
        assert_eq!(parsed.windows[0].scale_factor, 1.0);

        let value = serde_json::to_value(parsed).unwrap();
        assert_eq!(value["schema_version"].as_u64(), Some(1));
    }

    #[test]
    fn diag_screenshot_result_defaults_schema_version_to_1() {
        let value = serde_json::json!({
            "updated_unix_ms": 1700000,
            "completed": [],
        });
        let parsed: DiagScreenshotResultFileV1 = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.schema_version, 1);
        assert_eq!(DiagScreenshotResultFileV1::default().schema_version, 1);
    }

    #[test]
    fn click_step_pointer_kind_round_trips_and_omits_none() {
        let step = UiActionStepV2::Click {
            window: None,
            pointer_kind: None,
            target: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            button: UiMouseButtonV1::Left,
            click_count: 1,
            modifiers: None,
        };
        let value = serde_json::to_value(step.clone()).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
              "type": "click",
              "target": {"kind":"test_id","id":"a"},
              "button": "left"
            })
        );

        let parsed: UiActionStepV2 = serde_json::from_value(serde_json::json!({
          "type": "click",
          "pointer_kind": "touch",
          "target": {"kind":"test_id","id":"a"},
          "button": "left",
          "click_count": 1
        }))
        .unwrap();
        assert!(matches!(
            parsed,
            UiActionStepV2::Click {
                pointer_kind: Some(UiPointerKindV1::Touch),
                ..
            }
        ));
    }

    #[test]
    fn tap_step_pointer_kind_round_trips_and_omits_none() {
        let step = UiActionStepV2::Tap {
            window: None,
            pointer_kind: None,
            target: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            modifiers: None,
        };
        let value = serde_json::to_value(step.clone()).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
              "type": "tap",
              "target": {"kind":"test_id","id":"a"}
            })
        );

        let parsed: UiActionStepV2 = serde_json::from_value(serde_json::json!({
          "type": "tap",
          "pointer_kind": "pen",
          "target": {"kind":"test_id","id":"a"}
        }))
        .unwrap();
        assert!(matches!(
            parsed,
            UiActionStepV2::Tap {
                pointer_kind: Some(UiPointerKindV1::Pen),
                ..
            }
        ));
    }

    #[test]
    fn long_press_step_round_trips_and_omits_defaults() {
        let step = UiActionStepV2::LongPress {
            window: None,
            pointer_kind: None,
            target: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            duration_ms: default_long_press_duration_ms(),
            modifiers: None,
        };
        let value = serde_json::to_value(step.clone()).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
              "type": "long_press",
              "target": {"kind":"test_id","id":"a"}
            })
        );

        let parsed: UiActionStepV2 = serde_json::from_value(serde_json::json!({
          "type": "long_press",
          "pointer_kind": "pen",
          "target": {"kind":"test_id","id":"a"},
          "duration_ms": 125
        }))
        .unwrap();
        assert!(matches!(
            parsed,
            UiActionStepV2::LongPress {
                pointer_kind: Some(UiPointerKindV1::Pen),
                duration_ms: 125,
                ..
            }
        ));
    }

    #[test]
    fn swipe_step_round_trips_and_omits_defaults() {
        let step = UiActionStepV2::Swipe {
            window: None,
            pointer_kind: None,
            target: UiSelectorV1::TestId {
                id: "a".to_string(),
                root_z_index: None,
            },
            delta_x: 12.0,
            delta_y: -8.0,
            steps: default_drag_steps(),
            modifiers: None,
        };
        let value = serde_json::to_value(step.clone()).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
              "type": "swipe",
              "target": {"kind":"test_id","id":"a"},
              "delta_x": 12.0,
              "delta_y": -8.0
            })
        );

        let parsed: UiActionStepV2 = serde_json::from_value(serde_json::json!({
          "type": "swipe",
          "pointer_kind": "pen",
          "target": {"kind":"test_id","id":"a"},
          "delta_x": 1.0,
          "delta_y": 2.0,
          "steps": 3
        }))
        .unwrap();
        assert!(matches!(
            parsed,
            UiActionStepV2::Swipe {
                pointer_kind: Some(UiPointerKindV1::Pen),
                steps: 3,
                ..
            }
        ));
    }

    #[test]
    fn step_activate_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "activate",
            "target": { "kind": "test_id", "id": "trigger" }
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::Activate { window, target } => {
                assert!(window.is_none());
                assert!(matches!(target, UiSelectorV1::TestId { .. }));
            }
            _ => panic!("expected activate"),
        }
    }

    #[test]
    fn step_focus_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "focus",
            "target": { "kind": "test_id", "id": "trigger" }
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::Focus { window, target } => {
                assert!(window.is_none());
                assert!(matches!(target, UiSelectorV1::TestId { .. }));
            }
            _ => panic!("expected focus"),
        }
    }

    #[test]
    fn hit_test_explain_request_round_trips() {
        let value = serde_json::json!({
            "schema_version": 1,
            "window": 7,
            "target": { "kind": "test_id", "id": "trigger" }
        });
        let req: UiHitTestExplainV1 = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(req.window, 7);
        assert_eq!(serde_json::to_value(req).unwrap(), value);
    }

    #[test]
    fn step_paste_text_into_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "paste_text_into",
            "target": { "kind": "test_id", "id": "field" },
            "text": "Hello"
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::PasteTextInto {
                window,
                pointer_kind,
                target,
                text,
                clear_before_paste,
                timeout_frames,
            } => {
                assert!(window.is_none());
                assert!(pointer_kind.is_none());
                assert!(matches!(target, UiSelectorV1::TestId { .. }));
                assert_eq!(text, "Hello");
                assert!(!clear_before_paste);
                assert_eq!(timeout_frames, default_action_timeout_frames());
            }
            _ => panic!("expected paste_text_into"),
        }
    }

    #[test]
    fn step_set_text_value_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "set_text_value",
            "target": { "kind": "test_id", "id": "field" },
            "text": "#112233"
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::SetTextValue {
                window,
                target,
                text,
                timeout_frames,
            } => {
                assert!(window.is_none());
                assert!(matches!(target, UiSelectorV1::TestId { .. }));
                assert_eq!(text, "#112233");
                assert_eq!(timeout_frames, default_action_timeout_frames());
            }
            _ => panic!("expected set_text_value"),
        }
    }

    #[test]
    fn step_inspect_help_lock_best_match_and_copy_selector_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "inspect_help_lock_best_match_and_copy_selector",
            "query": "ui-gallery-nav-search"
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::InspectHelpLockBestMatchAndCopySelector {
                window,
                query,
                timeout_frames,
            } => {
                assert!(window.is_none());
                assert_eq!(query, "ui-gallery-nav-search");
                assert_eq!(timeout_frames, default_action_timeout_frames());
            }
            _ => panic!("expected inspect_help_lock_best_match_and_copy_selector"),
        }
    }

    #[test]
    fn step_inspect_help_tree_lock_best_match_and_copy_selector_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "inspect_help_tree_lock_best_match_and_copy_selector",
            "query": "ui-gallery-nav-search"
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::InspectHelpTreeLockBestMatchAndCopySelector {
                window,
                query,
                timeout_frames,
            } => {
                assert!(window.is_none());
                assert_eq!(query, "ui-gallery-nav-search");
                assert_eq!(timeout_frames, default_action_timeout_frames());
            }
            _ => panic!("expected inspect_help_tree_lock_best_match_and_copy_selector"),
        }
    }

    #[test]
    fn step_wait_until_deserializes_with_default_timeout_frames() {
        let value = serde_json::json!({
            "type": "wait_until",
            "predicate": {
                "kind": "exists",
                "target": { "kind": "test_id", "id": "ui-gallery-nav-search" }
            }
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::WaitUntil {
                window,
                predicate,
                timeout_frames,
                timeout_ms,
            } => {
                assert!(window.is_none());
                assert!(
                    matches!(predicate, UiPredicateV1::Exists { .. }),
                    "expected exists predicate"
                );
                assert_eq!(timeout_frames, default_action_timeout_frames());
                assert!(timeout_ms.is_none());
            }
            _ => panic!("expected wait_until"),
        }
    }

    #[test]
    fn step_wait_semantics_scroll_stable_deserializes_with_defaults() {
        let value = serde_json::json!({
            "type": "wait_semantics_scroll_stable",
            "target": { "kind": "test_id", "id": "ui-gallery-content-viewport" },
            "field": "y_max"
        });

        let step: UiActionStepV2 = serde_json::from_value(value).unwrap();
        match step {
            UiActionStepV2::WaitSemanticsScrollStable {
                window,
                target,
                field,
                stable_frames,
                max_delta,
                timeout_frames,
            } => {
                assert!(window.is_none());
                assert!(matches!(target, UiSelectorV1::TestId { .. }));
                assert_eq!(field, UiSemanticsScrollFieldV1::YMax);
                assert_eq!(stable_frames, default_semantics_scroll_stable_frames());
                assert_eq!(max_delta, default_semantics_scroll_stable_max_delta());
                assert_eq!(timeout_frames, default_action_timeout_frames());
            }
            _ => panic!("expected wait_semantics_scroll_stable"),
        }
    }
}
