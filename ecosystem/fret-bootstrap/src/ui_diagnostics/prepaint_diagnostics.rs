
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
