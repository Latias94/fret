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
