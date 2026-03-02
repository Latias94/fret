use super::super::*;
use fret_runtime::ModelCreatedDebugInfo;

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugHoverDeclarativeInvalidationHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub hit_test: u32,
    pub layout: u32,
    pub paint: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub(in crate::tree) struct UiDebugHoverDeclarativeInvalidationCounts {
    pub(in crate::tree) hit_test: u32,
    pub(in crate::tree) layout: u32,
    pub(in crate::tree) paint: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugModelChangeHotspot {
    pub model: ModelId,
    pub observation_edges: u32,
    pub changed: Option<fret_runtime::model::ModelChangedDebugInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugModelChangeUnobserved {
    pub model: ModelId,
    pub created: Option<ModelCreatedDebugInfo>,
    pub changed: Option<fret_runtime::model::ModelChangedDebugInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugGlobalChangeHotspot {
    pub global: TypeId,
    pub observation_edges: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugGlobalChangeUnobserved {
    pub global: TypeId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugInvalidationSource {
    ModelChange,
    GlobalChange,
    Notify,
    Hover,
    Focus,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugInvalidationDetail {
    Unknown,
    ModelObservation,
    GlobalObservation,
    NotifyCall,
    HoverEvent,
    /// A hover edge that must re-run declarative view-cache subtrees.
    ///
    /// `HoverRegion` is a mechanism-layer primitive that provides a `hovered: bool` signal to
    /// component code. Under view-cache reuse, paint-only invalidations do not re-run child render
    /// closures by design, so hover edges must be treated as "view dirty" to preserve the
    /// contract for hover-driven policies (e.g. hover cards).
    HoverRegionEdge,
    FocusEvent,
    ScrollHandleHitTestOnly,
    ScrollHandleLayout,
    ScrollHandleWindowUpdate,
    ScrollDeferredProbe,
    ScrollHandleScrollToItemWindowUpdate,
    ScrollHandleViewportResizeWindowUpdate,
    ScrollHandleItemsRevisionWindowUpdate,
    ScrollHandlePrefetchWindowUpdate,
    FocusVisiblePolicy,
    InputModalityPolicy,
    AnimationFrameRequest,
}

impl UiDebugInvalidationDetail {
    pub fn from_source(source: UiDebugInvalidationSource) -> Self {
        match source {
            UiDebugInvalidationSource::ModelChange => Self::ModelObservation,
            UiDebugInvalidationSource::GlobalChange => Self::GlobalObservation,
            UiDebugInvalidationSource::Notify => Self::NotifyCall,
            UiDebugInvalidationSource::Hover => Self::HoverEvent,
            UiDebugInvalidationSource::Focus => Self::FocusEvent,
            UiDebugInvalidationSource::Other => Self::Unknown,
        }
    }

    pub fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            Self::ModelObservation => Some("model_observation"),
            Self::GlobalObservation => Some("global_observation"),
            Self::NotifyCall => Some("notify_call"),
            Self::HoverEvent => Some("hover_event"),
            Self::HoverRegionEdge => Some("hover_region_edge"),
            Self::FocusEvent => Some("focus_event"),
            Self::ScrollHandleHitTestOnly => Some("scroll_handle_hit_test_only"),
            Self::ScrollHandleLayout => Some("scroll_handle_layout"),
            Self::ScrollHandleWindowUpdate => Some("scroll_handle_window_update"),
            Self::ScrollDeferredProbe => Some("scroll_deferred_probe"),
            Self::ScrollHandleScrollToItemWindowUpdate => {
                Some("scroll_handle_scroll_to_item_window_update")
            }
            Self::ScrollHandleViewportResizeWindowUpdate => {
                Some("scroll_handle_viewport_resize_window_update")
            }
            Self::ScrollHandleItemsRevisionWindowUpdate => {
                Some("scroll_handle_items_revision_window_update")
            }
            Self::ScrollHandlePrefetchWindowUpdate => Some("scroll_handle_prefetch_window_update"),
            Self::FocusVisiblePolicy => Some("focus_visible_policy"),
            Self::InputModalityPolicy => Some("input_modality_policy"),
            Self::AnimationFrameRequest => Some("animation_frame_request"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugDirtyView {
    pub view: ViewId,
    pub element: Option<GlobalElementId>,
    pub source: UiDebugInvalidationSource,
    pub detail: UiDebugInvalidationDetail,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugNotifyRequest {
    pub frame_id: FrameId,
    pub caller_node: NodeId,
    pub target_view: ViewId,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugInvalidationWalk {
    pub root: NodeId,
    pub root_element: Option<GlobalElementId>,
    pub inv: Invalidation,
    pub source: UiDebugInvalidationSource,
    pub detail: UiDebugInvalidationDetail,
    pub walked_nodes: u32,
    pub truncated_at: Option<NodeId>,
}
