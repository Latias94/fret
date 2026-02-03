use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use fret_core::{ClipboardToken, Modifiers, MouseButton, Point, PointerId, Rect};
use fret_runtime::{TickId, TimerToken};

use crate::core::{CanvasPoint, EdgeId, GroupId, NodeId as GraphNodeId, NodeKindKey, PortId};
use crate::rules::{DiagnosticSeverity, EdgeEndpoint};
use crate::runtime::callbacks::ViewportMoveKind;
use crate::ui::presenter::{InsertNodeCandidate, NodeGraphContextMenuItem};

use super::searcher::SearcherRow;
use super::snaplines::SnapGuides;

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

impl PasteSeries {
    pub(crate) fn next(prev: Option<Self>, anchor: CanvasPoint, zoom: f32) -> (Self, CanvasPoint) {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        let threshold = 6.0 / zoom;
        let step = 24.0 / zoom;

        let mut count = 0u32;
        if let Some(series) = prev {
            let dx = anchor.x - series.anchor.x;
            let dy = anchor.y - series.anchor.y;
            let d2 = dx * dx + dy * dy;
            if d2.is_finite() && d2 <= threshold * threshold {
                count = series.count.saturating_add(1);
            }
        }

        let next = Self { anchor, count };
        let at = CanvasPoint {
            x: anchor.x + count as f32 * step,
            y: anchor.y + count as f32 * step,
        };
        (next, at)
    }
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

impl ViewportAnimationEase {
    pub(crate) fn apply(self, t: f32) -> f32 {
        let t = if t.is_finite() {
            t.clamp(0.0, 1.0)
        } else {
            0.0
        };
        match self {
            Self::Linear => t,
            Self::Smoothstep => t * t * (3.0 - 2.0 * t),
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }
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

#[derive(Debug, Clone)]
pub(crate) struct SearcherState {
    pub(crate) origin: Point,
    pub(crate) invoked_at: Point,
    pub(crate) target: ContextMenuTarget,
    pub(crate) query: String,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
    pub(crate) recent_kinds: Vec<NodeKindKey>,
    pub(crate) rows: Vec<SearcherRow>,
    pub(crate) hovered_row: Option<usize>,
    pub(crate) active_row: usize,
    pub(crate) scroll: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingInsertNodeDrag {
    pub(crate) candidate: InsertNodeCandidate,
    pub(crate) start_pos: Point,
    pub(crate) pointer_id: PointerId,
    pub(crate) start_tick: TickId,
}

#[derive(Debug, Clone)]
pub(crate) struct InsertNodeDragPreview {
    pub(crate) label: Arc<str>,
    pub(crate) pos: Point,
    pub(crate) edge: Option<EdgeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingNodeSelectAction {
    None,
    Toggle,
}

impl Default for PendingNodeSelectAction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PendingNodeDrag {
    pub(crate) primary: GraphNodeId,
    pub(crate) nodes: Vec<GraphNodeId>,
    pub(crate) grab_offset: Point,
    pub(crate) start_pos: Point,
    pub(crate) select_action: PendingNodeSelectAction,
    pub(crate) drag_enabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeDrag {
    pub(crate) primary: GraphNodeId,
    pub(crate) node_ids: Vec<GraphNodeId>,
    pub(crate) nodes: Vec<(GraphNodeId, CanvasPoint)>,
    pub(crate) current_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    pub(crate) current_groups: Vec<(GroupId, crate::core::CanvasRect)>,
    pub(crate) preview_rev: u64,
    pub(crate) grab_offset: Point,
    pub(crate) start_pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingGroupDrag {
    pub(crate) group: crate::core::GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
}

#[derive(Debug, Clone)]
pub(crate) struct GroupDrag {
    pub(crate) group: crate::core::GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
    pub(crate) nodes: Vec<(GraphNodeId, CanvasPoint)>,
    pub(crate) current_rect: crate::core::CanvasRect,
    pub(crate) current_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    pub(crate) preview_rev: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingGroupResize {
    pub(crate) group: crate::core::GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
}

#[derive(Debug, Clone)]
pub(crate) struct GroupResize {
    pub(crate) group: crate::core::GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
    pub(crate) current_rect: crate::core::CanvasRect,
    pub(crate) preview_rev: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingNodeResize {
    pub(crate) node: GraphNodeId,
    pub(crate) handle: NodeResizeHandle,
    pub(crate) start_pos: Point,
    pub(crate) start_node_pos: CanvasPoint,
    pub(crate) start_size: crate::core::CanvasSize,
    pub(crate) start_size_opt: Option<crate::core::CanvasSize>,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeResize {
    pub(crate) node: GraphNodeId,
    pub(crate) handle: NodeResizeHandle,
    pub(crate) start_pos: Point,
    pub(crate) start_node_pos: CanvasPoint,
    pub(crate) start_size: crate::core::CanvasSize,
    pub(crate) start_size_opt: Option<crate::core::CanvasSize>,
    pub(crate) current_node_pos: CanvasPoint,
    pub(crate) current_size_opt: Option<crate::core::CanvasSize>,
    pub(crate) current_groups: Vec<(GroupId, crate::core::CanvasRect)>,
    pub(crate) preview_rev: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl NodeResizeHandle {
    pub const ALL: [Self; 8] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Right,
        Self::BottomRight,
        Self::Bottom,
        Self::BottomLeft,
        Self::Left,
    ];

    pub fn affects_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::Left | Self::BottomLeft)
    }

    pub fn affects_right(self) -> bool {
        matches!(self, Self::TopRight | Self::Right | Self::BottomRight)
    }

    pub fn affects_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::Top | Self::TopRight)
    }

    pub fn affects_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::Bottom | Self::BottomRight)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PendingMarqueeDrag {
    pub(crate) start_pos: Point,
    pub(crate) clear_selection_on_up: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct MarqueeDrag {
    pub(crate) start_pos: Point,
    pub(crate) pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) enum WireDragKind {
    New {
        from: PortId,
        bundle: Vec<PortId>,
    },
    Reconnect {
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        fixed: PortId,
    },
    ReconnectMany {
        edges: Vec<(EdgeId, EdgeEndpoint, PortId)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct WireDrag {
    pub(crate) kind: WireDragKind,
    pub(crate) pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingWireDrag {
    pub(crate) kind: WireDragKind,
    pub(crate) start_pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingEdgeInsertDrag {
    pub(crate) edge: EdgeId,
    pub(crate) start_pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct EdgeInsertDrag {
    pub(crate) edge: EdgeId,
    pub(crate) pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct EdgeDrag {
    pub(crate) edge: EdgeId,
    pub(crate) start_pos: Point,
}

#[derive(Debug, Clone)]
pub(crate) enum ContextMenuTarget {
    Background,
    BackgroundInsertNodePicker {
        at: CanvasPoint,
    },
    ConnectionInsertNodePicker {
        from: PortId,
        at: CanvasPoint,
    },
    Edge(EdgeId),
    EdgeInsertNodePicker(EdgeId),
    ConnectionConvertPicker {
        from: PortId,
        to: PortId,
        at: CanvasPoint,
    },
    Group(crate::core::GroupId),
}

#[derive(Debug, Clone)]
pub(crate) struct ContextMenuState {
    pub(crate) origin: Point,
    pub(crate) invoked_at: Point,
    pub(crate) target: ContextMenuTarget,
    pub(crate) items: Vec<NodeGraphContextMenuItem>,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
    pub(crate) hovered_item: Option<usize>,
    pub(crate) active_item: usize,
    pub(crate) typeahead: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ToastState {
    pub(crate) timer: TimerToken,
    pub(crate) severity: DiagnosticSeverity,
    pub(crate) message: Arc<str>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingPaste {
    pub(crate) token: ClipboardToken,
    pub(crate) at: CanvasPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DerivedBaseKey {
    pub(crate) graph_rev: u64,
    pub(crate) zoom_bits: u32,
    pub(crate) node_origin_x_bits: u32,
    pub(crate) node_origin_y_bits: u32,
    pub(crate) draw_order_hash: u64,
    pub(crate) presenter_rev: u64,
    pub(crate) edge_types_rev: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GeometryCacheKey {
    pub(crate) base: DerivedBaseKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SpatialIndexCacheKey {
    pub(crate) geom: GeometryCacheKey,
    pub(crate) cell_size_screen_bits: u32,
    pub(crate) min_cell_size_screen_bits: u32,
    pub(crate) edge_aabb_pad_screen_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InternalsViewKey {
    pub(crate) pan_x_bits: u32,
    pub(crate) pan_y_bits: u32,
    pub(crate) bounds_x_bits: u32,
    pub(crate) bounds_y_bits: u32,
    pub(crate) bounds_w_bits: u32,
    pub(crate) bounds_h_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InternalsCacheKey {
    pub(crate) base: DerivedBaseKey,
    pub(crate) view: InternalsViewKey,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GeometryCache {
    pub(crate) geom_key: Option<GeometryCacheKey>,
    pub(crate) index_key: Option<SpatialIndexCacheKey>,
    pub(crate) geom: Arc<super::geometry::CanvasGeometry>,
    pub(crate) index: Arc<super::spatial::CanvasSpatialIndex>,
    pub(crate) drag_preview: Option<DragPreviewCache>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragPreviewKind {
    NodeDrag,
    GroupDrag,
    NodeResize,
}

#[derive(Debug, Clone)]
pub(crate) struct DragPreviewCache {
    pub(crate) kind: DragPreviewKind,
    pub(crate) base_index_key: SpatialIndexCacheKey,
    pub(crate) preview_rev: u64,
    pub(crate) geom: Arc<super::geometry::CanvasGeometry>,
    pub(crate) index: Arc<super::spatial::CanvasSpatialIndex>,
    pub(crate) node_positions: HashMap<GraphNodeId, CanvasPoint>,
    pub(crate) node_rects: HashMap<GraphNodeId, Rect>,
    pub(crate) node_ports: HashMap<GraphNodeId, Vec<PortId>>,
}

#[derive(Debug, Clone)]
pub(crate) struct LastConversionContext {
    pub(crate) from: PortId,
    pub(crate) to: PortId,
    pub(crate) at: CanvasPoint,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
}
