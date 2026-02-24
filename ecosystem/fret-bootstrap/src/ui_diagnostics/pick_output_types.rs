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
