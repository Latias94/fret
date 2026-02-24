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
