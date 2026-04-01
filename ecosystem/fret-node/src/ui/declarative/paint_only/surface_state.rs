use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PortalDebugFlags {
    /// Diagnostics-only: when true, disable portal hosting and clear `PortalBoundsStore` so overlay
    /// consumers can exercise their fallback paths.
    pub(super) disable_portals: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DragState {
    pub(super) button: MouseButton,
    pub(super) last_pos: Point,
}

#[derive(Debug, Clone)]
/// Local surface-state for marquee preview/arming; never persisted into `NodeGraphViewState`.
pub(super) struct MarqueeDragState {
    pub(super) start_screen: Point,
    pub(super) current_screen: Point,
    pub(super) active: bool,
    pub(super) toggle: bool,
    pub(super) base_selected_nodes: Arc<[crate::core::NodeId]>,
    pub(super) preview_selected_nodes: Arc<[crate::core::NodeId]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeDragPhase {
    Armed,
    Active,
    Canceled,
}

#[derive(Debug, Clone)]
/// Local surface-state for node-drag preview/arming; committed graph edits still flow through
/// controller/store transactions.
pub(super) struct NodeDragState {
    pub(super) start_screen: Point,
    pub(super) current_screen: Point,
    pub(super) phase: NodeDragPhase,
    pub(super) nodes_sorted: Arc<[crate::core::NodeId]>,
}

impl NodeDragState {
    pub(super) fn is_armed(&self) -> bool {
        matches!(self.phase, NodeDragPhase::Armed)
    }

    pub(super) fn is_active(&self) -> bool {
        matches!(self.phase, NodeDragPhase::Active)
    }

    pub(super) fn is_canceled(&self) -> bool {
        matches!(self.phase, NodeDragPhase::Canceled)
    }

    pub(super) fn is_live(&self) -> bool {
        !self.is_canceled()
    }

    pub(super) fn activate(&mut self, current_screen: Point) -> bool {
        if !self.is_armed() {
            return false;
        }
        self.phase = NodeDragPhase::Active;
        self.current_screen = current_screen;
        true
    }

    pub(super) fn cancel(&mut self) {
        self.phase = NodeDragPhase::Canceled;
    }

    pub(super) fn update_active_position(&mut self, current_screen: Point) -> bool {
        if !self.is_active() || self.current_screen == current_screen {
            return false;
        }
        self.current_screen = current_screen;
        true
    }
}

#[derive(Debug, Clone)]
/// Local click-selection preview that should only override paint until commit/cancel time.
pub(super) struct PendingSelectionState {
    pub(super) nodes: Arc<[crate::core::NodeId]>,
    pub(super) clear_edges: bool,
    pub(super) clear_groups: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AuthoritativeSurfaceBoundarySnapshot {
    pub(super) graph_id: crate::core::GraphId,
    pub(super) graph_rev: u64,
    pub(super) selected_nodes_hash: u64,
    pub(super) selected_edges_hash: u64,
    pub(super) selected_groups_hash: u64,
}

#[derive(Debug, Clone)]
pub(super) struct GridPaintCacheState {
    /// Last known bounds for the surface (updated from pointer hooks, and optionally from
    /// `last_bounds_for_element`).
    pub(super) bounds: Rect,
    pub(super) key: Option<CanvasKey>,
    pub(super) rebuilds: u64,
    pub(super) ops: Option<Arc<Vec<fret_core::SceneOp>>>,
}

impl Default for GridPaintCacheState {
    fn default() -> Self {
        Self {
            bounds: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                fret_core::Size::new(Px(0.0), Px(0.0)),
            ),
            key: None,
            rebuilds: 0,
            ops: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct GridPaintCacheKeyV2 {
    pub(super) bounds_x_q: i32,
    pub(super) bounds_y_q: i32,
    pub(super) bounds_w_q: i32,
    pub(super) bounds_h_q: i32,
    pub(super) zoom_q: i32,
    pub(super) ix0: i32,
    pub(super) ix1: i32,
    pub(super) iy0: i32,
    pub(super) iy1: i32,
    pub(super) bg_r_bits: u32,
    pub(super) bg_g_bits: u32,
    pub(super) bg_b_bits: u32,
    pub(super) bg_a_bits: u32,
    pub(super) grid_r_bits: u32,
    pub(super) grid_g_bits: u32,
    pub(super) grid_b_bits: u32,
    pub(super) grid_a_bits: u32,
}
