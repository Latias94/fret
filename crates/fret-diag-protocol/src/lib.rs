use serde::{Deserialize, Serialize};

pub mod builder;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct DevtoolsHelloV1 {
    pub client_kind: String,
    pub client_version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<UiScriptMetaV1>,
    pub steps: Vec<UiActionStepV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCapabilitiesV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiActionStepV2 {
    // v1-compatible steps
    Click {
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
        n: u32,
    },
    WaitUntil {
        predicate: UiPredicateV1,
        timeout_frames: u32,
    },
    /// Wait until the shortcut routing diagnostics trace contains an entry matching `query`.
    ///
    /// This is intended for deterministic scripts that need to assert keyboard routing outcomes
    /// (e.g. reserved-for-IME) without depending on screenshots or ad-hoc logs.
    WaitShortcutRoutingTrace {
        query: UiShortcutRoutingTraceQueryV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
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
        target: UiSelectorV1,
        #[serde(default = "default_bounds_stable_frames")]
        stable_frames: u32,
        #[serde(default = "default_bounds_stable_max_move_px")]
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
        target: UiSelectorV1,
        text: String,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    MenuSelect {
        menu: UiSelectorV1,
        item: UiSelectorV1,
        #[serde(default = "default_action_timeout_frames")]
        timeout_frames: u32,
    },
    MenuSelectPath {
        path: Vec<UiSelectorV1>,
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

fn default_move_frames_per_step() -> u32 {
    1
}

fn default_click_count() -> u8 {
    1
}

fn is_default_click_count(v: &u8) -> bool {
    *v == 1
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

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
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
    RoleIs {
        target: UiSelectorV1,
        role: String,
    },
    CheckedIs {
        target: UiSelectorV1,
        checked: bool,
    },
    SelectedIs {
        target: UiSelectorV1,
        selected: bool,
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
    BarrierRoots {
        #[serde(default)]
        barrier_root: UiOptionalRootStateV1,
        #[serde(default)]
        focus_barrier_root: UiOptionalRootStateV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        require_equal: Option<bool>,
    },
    VisibleInWindow {
        target: UiSelectorV1,
    },
    BoundsWithinWindow {
        target: UiSelectorV1,
        #[serde(default)]
        padding_px: f32,
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
    },
    RoleAndPath {
        role: String,
        name: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevtoolsBundleDumpedV1 {
    pub schema_version: u32,
    pub exported_unix_ms: u64,
    pub out_dir: String,
    pub dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<serde_json::Value>,
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiScriptEvidenceV1 {
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
    pub overlay_placement_trace: Vec<UiOverlayPlacementTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub web_ime_trace: Vec<UiWebImeTraceEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ime_event_trace: Vec<UiImeEventTraceEntryV1>,
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
