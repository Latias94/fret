use fret_core::time::Instant;
use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, Point, Rect};
use fret_runtime::TimerToken;

pub use super::resize_handle::NodeResizeHandle;
use super::snaplines::SnapGuides;
use crate::core::{CanvasPoint, EdgeId, GroupId, NodeId as GraphNodeId, NodeKindKey, PortId};
use crate::rules::{DiagnosticSeverity, EdgeEndpoint};
use crate::runtime::callbacks::ViewportMoveKind;

mod state_drag_sessions;
mod state_geometry_cache;
mod state_overlay_policy;
mod state_overlay_sessions;
mod state_paste_series;
mod state_preview_cache;
mod state_viewport_animation;

pub(crate) use state_drag_sessions::{
    EdgeDrag, EdgeInsertDrag, GroupDrag, GroupResize, InsertNodeDragPreview, MarqueeDrag, NodeDrag,
    NodeResize, PendingEdgeInsertDrag, PendingGroupDrag, PendingGroupResize, PendingInsertNodeDrag,
    PendingMarqueeDrag, PendingNodeDrag, PendingNodeResize, PendingNodeSelectAction,
    PendingWireDrag, WireDrag, WireDragKind,
};
pub(crate) use state_overlay_policy::{ContextMenuTarget, SearcherRowsMode};
pub(crate) use state_overlay_sessions::{
    ContextMenuState, LastConversionContext, PendingPaste, SearcherState, ToastState,
};
#[cfg(test)]
pub(crate) use state_preview_cache::DerivedBuildCounters;
pub(crate) use state_preview_cache::{
    DerivedBaseKey, DragPreviewCache, DragPreviewCacheMetaMut, DragPreviewKind,
    DrawOrderFingerprint, GeometryCache, GeometryCacheKey, InternalsCacheKey, InternalsViewKey,
    SpatialIndexCacheKey,
};

#[derive(Debug, Clone)]
pub(crate) struct ViewSnapshot {
    pub(crate) pan: CanvasPoint,
    pub(crate) zoom: f32,
    pub(crate) selected_nodes: Vec<GraphNodeId>,
    pub(crate) selected_edges: Vec<EdgeId>,
    pub(crate) selected_groups: Vec<GroupId>,
    pub(crate) draw_order: Vec<GraphNodeId>,
    pub(crate) group_draw_order: Vec<GroupId>,
    pub(crate) interaction: crate::io::NodeGraphInteractionState,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct InteractionState {
    pub(crate) last_pos: Option<Point>,
    pub(crate) last_canvas_pos: Option<CanvasPoint>,
    pub(crate) last_bounds: Option<Rect>,
    pub(crate) last_modifiers: Modifiers,
    /// Whether multi-selection mode is currently active (XyFlow `multiSelectionActive`).
    pub(crate) multi_selection_active: bool,
    pub(crate) last_conversion: Option<LastConversionContext>,
    pub(crate) pan_activation_key_held: bool,
    pub(crate) pending_right_click: Option<PendingRightClick>,
    pub(crate) panning: bool,
    pub(crate) panning_button: Option<MouseButton>,
    /// Last pointer sample in screen space (stable while pan/zoom changes).
    ///
    /// Needed because `NodeGraphCanvas` uses `render_transform` for pan/zoom, so pointer event
    /// positions are in the node's local (canvas) coordinates and change when pan/zoom changes.
    pub(crate) pan_last_screen_pos: Option<Point>,
    pub(crate) pan_last_sample_at: Option<Instant>,
    pub(crate) pan_velocity: CanvasPoint,
    pub(crate) pan_inertia: Option<PanInertiaState>,
    pub(crate) viewport_animation: Option<ViewportAnimationState>,
    pub(crate) viewport_move_debounce: Option<ViewportMoveDebounceState>,
    /// Whether the current `wire_drag` session was initiated via click-to-connect.
    ///
    /// When set, the next handle click should attempt to finalize the connection and then clear
    /// this flag (regardless of validity), matching XyFlow's `connectOnClick` behavior.
    pub(crate) click_connect: bool,
    pub(crate) pending_marquee: Option<PendingMarqueeDrag>,
    pub(crate) marquee: Option<MarqueeDrag>,
    pub(crate) pending_node_drag: Option<PendingNodeDrag>,
    pub(crate) node_drag: Option<NodeDrag>,
    pub(crate) pending_group_drag: Option<PendingGroupDrag>,
    pub(crate) group_drag: Option<GroupDrag>,
    pub(crate) pending_group_resize: Option<PendingGroupResize>,
    pub(crate) group_resize: Option<GroupResize>,
    pub(crate) pending_node_resize: Option<PendingNodeResize>,
    pub(crate) node_resize: Option<NodeResize>,
    pub(crate) pending_wire_drag: Option<PendingWireDrag>,
    pub(crate) wire_drag: Option<WireDrag>,
    /// When a multi-step connection workflow opens a picker (conversion/search), the active wire
    /// drag is suspended so it can be restored if the picker action is rejected.
    pub(crate) suspended_wire_drag: Option<WireDrag>,
    pub(crate) pending_edge_insert_drag: Option<PendingEdgeInsertDrag>,
    pub(crate) edge_insert_drag: Option<EdgeInsertDrag>,
    pub(crate) edge_drag: Option<EdgeDrag>,
    pub(crate) hover_edge: Option<EdgeId>,
    pub(crate) hover_edge_anchor: Option<(EdgeId, EdgeEndpoint)>,
    pub(crate) focused_edge: Option<EdgeId>,
    pub(crate) focused_node: Option<GraphNodeId>,
    pub(crate) focused_port: Option<PortId>,
    pub(crate) focused_port_valid: bool,
    pub(crate) focused_port_convertible: bool,
    pub(crate) hover_port: Option<PortId>,
    pub(crate) hover_port_valid: bool,
    pub(crate) hover_port_convertible: bool,
    pub(crate) hover_port_diagnostic: Option<(DiagnosticSeverity, Arc<str>)>,
    pub(crate) context_menu: Option<ContextMenuState>,
    pub(crate) searcher: Option<SearcherState>,
    pub(crate) pending_insert_node_drag: Option<PendingInsertNodeDrag>,
    pub(crate) insert_node_drag_preview: Option<InsertNodeDragPreview>,
    pub(crate) toast: Option<ToastState>,
    pub(crate) auto_pan_timer: Option<TimerToken>,
    pub(crate) pending_paste: Option<PendingPaste>,
    pub(crate) paste_series: Option<PasteSeries>,
    pub(crate) snap_guides: Option<SnapGuides>,

    pub(crate) sticky_wire: bool,
    pub(crate) sticky_wire_ignore_next_up: bool,

    pub(crate) recent_kinds: Vec<NodeKindKey>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingRightClick {
    pub(crate) start_pos: Point,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PasteSeries {
    pub(crate) anchor: CanvasPoint,
    pub(crate) count: u32,
}

#[cfg(test)]
mod tests {
    use super::{CanvasPoint, PasteSeries, ViewportAnimationEase};

    #[test]
    fn paste_series_increments_when_anchor_is_stable() {
        let anchor = CanvasPoint { x: 10.0, y: 20.0 };

        let (s1, at1) = PasteSeries::next(None, anchor, 1.0);
        assert_eq!(s1.count, 0);
        assert_eq!(at1, anchor);

        let (s2, at2) = PasteSeries::next(Some(s1), anchor, 1.0);
        assert_eq!(s2.count, 1);
        assert_eq!(at2, CanvasPoint { x: 34.0, y: 44.0 });

        let (s3, at3) = PasteSeries::next(Some(s2), anchor, 1.0);
        assert_eq!(s3.count, 2);
        assert_eq!(at3, CanvasPoint { x: 58.0, y: 68.0 });
    }

    #[test]
    fn paste_series_resets_when_anchor_moves_farther_than_threshold() {
        let anchor = CanvasPoint { x: 10.0, y: 20.0 };
        let (s1, _) = PasteSeries::next(None, anchor, 1.0);
        let (s2, _) = PasteSeries::next(Some(s1), anchor, 1.0);
        assert_eq!(s2.count, 1);

        // Threshold is 6px at zoom=1, so moving by 7px should reset.
        let moved = CanvasPoint { x: 17.0, y: 20.0 };
        let (s3, at3) = PasteSeries::next(Some(s2), moved, 1.0);
        assert_eq!(s3.count, 0);
        assert_eq!(at3, moved);
    }

    #[test]
    fn paste_series_scales_threshold_and_step_by_zoom() {
        let anchor = CanvasPoint { x: 0.0, y: 0.0 };

        // At zoom=2, step should be 12 canvas units (24/2).
        let (s1, at1) = PasteSeries::next(None, anchor, 2.0);
        let (s2, at2) = PasteSeries::next(Some(s1), anchor, 2.0);
        assert_eq!(s2.count, 1);
        assert_eq!(at1, anchor);
        assert_eq!(at2, CanvasPoint { x: 12.0, y: 12.0 });
    }

    #[test]
    fn viewport_ease_preserves_endpoints_and_midpoint() {
        for ease in [
            ViewportAnimationEase::Linear,
            ViewportAnimationEase::Smoothstep,
            ViewportAnimationEase::CubicInOut,
        ] {
            assert!((ease.apply(0.0) - 0.0).abs() <= 1.0e-6);
            assert!((ease.apply(1.0) - 1.0).abs() <= 1.0e-6);
            assert!((ease.apply(0.5) - 0.5).abs() <= 1.0e-6);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PanInertiaState {
    pub(crate) timer: TimerToken,
    pub(crate) velocity: CanvasPoint,
    pub(crate) last_tick_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewportAnimationInterpolate {
    Linear,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewportAnimationEase {
    Linear,
    Smoothstep,
    CubicInOut,
}

#[derive(Debug, Clone)]
pub(crate) struct ViewportAnimationState {
    pub(crate) timer: TimerToken,
    pub(crate) from_pan: CanvasPoint,
    pub(crate) from_zoom: f32,
    pub(crate) to_pan: CanvasPoint,
    pub(crate) to_zoom: f32,
    pub(crate) interpolate: ViewportAnimationInterpolate,
    pub(crate) ease: Option<ViewportAnimationEase>,
    pub(crate) duration: std::time::Duration,
    pub(crate) elapsed: std::time::Duration,
    pub(crate) last_tick_at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct ViewportMoveDebounceState {
    pub(crate) kind: ViewportMoveKind,
    pub(crate) timer: TimerToken,
}
