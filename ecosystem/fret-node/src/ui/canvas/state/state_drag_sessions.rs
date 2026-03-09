use std::sync::Arc;

use fret_core::{Point, PointerId};
use fret_runtime::TickId;

use crate::core::{CanvasPoint, EdgeId, GroupId, NodeId as GraphNodeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::NodeResizeHandle;
use crate::ui::presenter::InsertNodeCandidate;

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
    pub(crate) group: GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
}

#[derive(Debug, Clone)]
pub(crate) struct GroupDrag {
    pub(crate) group: GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
    pub(crate) nodes: Vec<(GraphNodeId, CanvasPoint)>,
    pub(crate) current_rect: crate::core::CanvasRect,
    pub(crate) current_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    pub(crate) preview_rev: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingGroupResize {
    pub(crate) group: GroupId,
    pub(crate) start_pos: Point,
    pub(crate) start_rect: crate::core::CanvasRect,
}

#[derive(Debug, Clone)]
pub(crate) struct GroupResize {
    pub(crate) group: GroupId,
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
