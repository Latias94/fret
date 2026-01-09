use std::sync::Arc;

use fret_core::{ClipboardToken, Modifiers, Point, Rect};
use fret_runtime::TimerToken;

use crate::core::{CanvasPoint, EdgeId, GroupId, NodeId as GraphNodeId, NodeKindKey, PortId};
use crate::rules::{DiagnosticSeverity, EdgeEndpoint};
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
    pub(crate) last_conversion: Option<LastConversionContext>,
    pub(crate) panning: bool,
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
    pub(crate) edge_drag: Option<EdgeDrag>,
    pub(crate) hover_edge: Option<EdgeId>,
    pub(crate) hover_port: Option<PortId>,
    pub(crate) hover_port_valid: bool,
    pub(crate) hover_port_convertible: bool,
    pub(crate) context_menu: Option<ContextMenuState>,
    pub(crate) searcher: Option<SearcherState>,
    pub(crate) toast: Option<ToastState>,
    pub(crate) auto_pan_timer: Option<TimerToken>,
    pub(crate) pending_paste: Option<PendingPaste>,
    pub(crate) snap_guides: Option<SnapGuides>,

    pub(crate) sticky_wire: bool,
    pub(crate) sticky_wire_ignore_next_up: bool,

    pub(crate) recent_kinds: Vec<NodeKindKey>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingNodeSelectAction {
    None,
    Toggle,
    Add,
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
    pub(crate) nodes: Vec<(GraphNodeId, CanvasPoint)>,
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
}

#[derive(Debug, Clone)]
pub(crate) struct PendingNodeResize {
    pub(crate) node: GraphNodeId,
    pub(crate) start_pos: Point,
    pub(crate) start_size: crate::core::CanvasSize,
    pub(crate) start_size_opt: Option<crate::core::CanvasSize>,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeResize {
    pub(crate) node: GraphNodeId,
    pub(crate) start_pos: Point,
    pub(crate) start_size: crate::core::CanvasSize,
    pub(crate) start_size_opt: Option<crate::core::CanvasSize>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MarqueeMode {
    Replace,
    Add,
    Toggle,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingMarqueeDrag {
    pub(crate) start_pos: Point,
    pub(crate) base_nodes: Vec<GraphNodeId>,
    pub(crate) mode: MarqueeMode,
}

#[derive(Debug, Clone)]
pub(crate) struct MarqueeDrag {
    pub(crate) start_pos: Point,
    pub(crate) pos: Point,
    pub(crate) base_nodes: Vec<GraphNodeId>,
    pub(crate) mode: MarqueeMode,
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
pub(crate) struct GeometryCacheKey {
    pub(crate) graph_rev: u64,
    pub(crate) zoom_bits: u32,
    pub(crate) draw_order_hash: u64,
    pub(crate) presenter_rev: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InternalsCacheKey {
    pub(crate) graph_rev: u64,
    pub(crate) zoom_bits: u32,
    pub(crate) draw_order_hash: u64,
    pub(crate) presenter_rev: u64,
    pub(crate) pan_x_bits: u32,
    pub(crate) pan_y_bits: u32,
    pub(crate) bounds_x_bits: u32,
    pub(crate) bounds_y_bits: u32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GeometryCache {
    pub(crate) key: Option<GeometryCacheKey>,
    pub(crate) geom: Arc<super::geometry::CanvasGeometry>,
    pub(crate) index: Arc<super::spatial::CanvasSpatialIndex>,
}

#[derive(Debug, Clone)]
pub(crate) struct LastConversionContext {
    pub(crate) from: PortId,
    pub(crate) to: PortId,
    pub(crate) at: CanvasPoint,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
}
