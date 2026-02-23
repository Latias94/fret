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
