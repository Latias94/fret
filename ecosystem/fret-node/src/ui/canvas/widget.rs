use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fret_canvas::budget::{InteractionBudget, WorkBudget};
use fret_canvas::cache::{
    SceneOpTileCache, TileCacheKeyBuilder, TileCoord, TileGrid2D, warm_scene_op_tiles_u64,
};
use fret_canvas::diagnostics::{CanvasCacheKey, CanvasCacheStatsRegistry};
use fret_canvas::scale::{canvas_units_from_screen_px, effective_scale_factor};
use fret_canvas::view::{CanvasViewport2D, PanZoom2D};
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextBlobId, TextConstraints, TextOverflow, TextWrap, Transform2D,
};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::{UiHost, retained_bridge::*};
use slotmap::Key;

use crate::REROUTE_KIND;
use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, Graph, NodeId as GraphNodeId, NodeKindKey,
    PortDirection, PortId,
};
use crate::interaction::NodeGraphConnectionMode;
use crate::io::{NodeGraphInteractionState, NodeGraphViewState};
use crate::ops::{
    GraphFragment, GraphHistory, GraphOp, GraphOpBuilderExt, GraphTransaction, IdRemapSeed,
    IdRemapper, PasteTuning, apply_transaction,
};
use crate::profile::{ApplyPipelineError, apply_transaction_with_profile};
use crate::rules::{ConnectDecision, Diagnostic, DiagnosticSeverity, EdgeEndpoint};
use crate::runtime::callbacks::{
    ConnectDragKind, ConnectEnd, ConnectEndOutcome, ConnectStart, NodeDragEnd, NodeDragEndOutcome,
    NodeDragStart, ViewportMoveEnd, ViewportMoveEndOutcome, ViewportMoveKind, ViewportMoveStart,
};
use crate::runtime::callbacks::{NodeGraphCallbacks, connection_changes_from_transaction};
use crate::runtime::changes::NodeGraphChanges;
use crate::runtime::events::ViewChange;
use crate::runtime::store::NodeGraphStore;

use crate::ui::commands::{
    CMD_NODE_GRAPH_ACTIVATE, CMD_NODE_GRAPH_ALIGN_BOTTOM, CMD_NODE_GRAPH_ALIGN_CENTER_X,
    CMD_NODE_GRAPH_ALIGN_CENTER_Y, CMD_NODE_GRAPH_ALIGN_LEFT, CMD_NODE_GRAPH_ALIGN_RIGHT,
    CMD_NODE_GRAPH_ALIGN_TOP, CMD_NODE_GRAPH_COPY, CMD_NODE_GRAPH_CREATE_GROUP, CMD_NODE_GRAPH_CUT,
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_DISTRIBUTE_X, CMD_NODE_GRAPH_DISTRIBUTE_Y,
    CMD_NODE_GRAPH_DUPLICATE, CMD_NODE_GRAPH_FOCUS_NEXT, CMD_NODE_GRAPH_FOCUS_NEXT_EDGE,
    CMD_NODE_GRAPH_FOCUS_NEXT_PORT, CMD_NODE_GRAPH_FOCUS_PORT_DOWN, CMD_NODE_GRAPH_FOCUS_PORT_LEFT,
    CMD_NODE_GRAPH_FOCUS_PORT_RIGHT, CMD_NODE_GRAPH_FOCUS_PORT_UP, CMD_NODE_GRAPH_FOCUS_PREV,
    CMD_NODE_GRAPH_FOCUS_PREV_EDGE, CMD_NODE_GRAPH_FOCUS_PREV_PORT, CMD_NODE_GRAPH_FRAME_ALL,
    CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT,
    CMD_NODE_GRAPH_GROUP_RENAME, CMD_NODE_GRAPH_GROUP_SEND_TO_BACK, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_NUDGE_DOWN, CMD_NODE_GRAPH_NUDGE_DOWN_FAST, CMD_NODE_GRAPH_NUDGE_LEFT,
    CMD_NODE_GRAPH_NUDGE_LEFT_FAST, CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_NUDGE_RIGHT_FAST,
    CMD_NODE_GRAPH_NUDGE_UP, CMD_NODE_GRAPH_NUDGE_UP_FAST, CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER,
    CMD_NODE_GRAPH_OPEN_INSERT_NODE, CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
    CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_REDO, CMD_NODE_GRAPH_RESET_VIEW,
    CMD_NODE_GRAPH_SELECT_ALL, CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE, CMD_NODE_GRAPH_UNDO,
    CMD_NODE_GRAPH_ZOOM_IN, CMD_NODE_GRAPH_ZOOM_OUT,
};
use crate::ui::presenter::{
    DefaultNodeGraphPresenter, EdgeRenderHint, EdgeRouteKind, InsertNodeCandidate,
    NodeGraphContextMenuAction, NodeGraphContextMenuItem, NodeGraphPresenter, NodeResizeHandleSet,
    PortAnchorHint,
};
use crate::ui::style::NodeGraphStyle;
use crate::ui::{
    FallbackMeasuredNodeGraphPresenter, GroupRenameOverlay, MeasuredGeometryStore,
    NodeGraphCanvasTransform, NodeGraphEdgeTypes, NodeGraphEditQueue, NodeGraphInternalsSnapshot,
    NodeGraphInternalsStore, NodeGraphOverlayState,
};

use super::middleware::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasEventOutcome,
    NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareCx, NoopNodeGraphCanvasMiddleware,
};
use super::paint::CanvasPaintCache;
use super::state::ViewportMoveDebounceState;

mod auto_measure;
mod callbacks;
mod cancel;
mod clipboard;
mod command_router;
mod commit;
mod commit_legacy;
mod context_menu;
mod cursor;
mod delete;
mod edge_drag;
mod edge_insert;
mod edge_insert_drag;
mod event_router;
mod focus;
mod group_drag;
mod group_resize;
mod hit_test;
mod hover;
mod insert_node_drag;
mod left_click;
mod marquee;
mod move_ops;
mod node_drag;
mod node_resize;
mod overlay_hit;
mod paint_root;
mod pan_zoom;
mod pending_drag;
mod pending_group_drag;
mod pending_group_resize;
mod pending_resize;
mod pending_wire_drag;
mod pointer_up;
mod preview;
mod rect_math;
mod right_click;
mod searcher;
mod selection;
mod sticky_wire;
mod threshold;
mod toast;
mod view_math;
mod view_state;
mod wire_drag;
mod wire_math;

use overlay_hit::{
    context_menu_rect_at, hit_context_menu_item, hit_searcher_row, searcher_rect_at,
    searcher_visible_rows,
};
use rect_math::{edge_bounds_rect, inflate_rect, rect_from_points, rect_union, rects_intersect};
use wire_math::{
    closest_point_on_edge_route, dist2_point_to_segment, step_wire_distance2, wire_distance2,
};

use super::conversion;
use super::geometry::group_order;
use super::geometry::{CanvasGeometry, node_ports};
use super::route_math::{
    cubic_bezier, cubic_bezier_derivative, edge_route_end_tangent, edge_route_start_tangent,
    normal_from_tangent, wire_ctrl_points,
};
use super::searcher::{SEARCHER_MAX_VISIBLE_ROWS, SearcherRow, SearcherRowKind};
use super::snaplines::SnapGuides;
use super::spatial::CanvasSpatialIndex;
use super::state::{
    ContextMenuState, ContextMenuTarget, DragPreviewCache, DragPreviewKind, GeometryCache,
    GeometryCacheKey, InteractionState, InternalsCacheKey, MarqueeDrag, NodeResizeHandle,
    PanInertiaState, PasteSeries, PendingPaste, SearcherState, ToastState, ViewSnapshot, WireDrag,
    WireDragKind,
};
use super::workflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignDistributeMode {
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    AlignCenterX,
    AlignCenterY,
    DistributeX,
    DistributeY,
}

#[derive(Debug, Clone, Copy)]
enum PortNavDir {
    Left,
    Right,
    Up,
    Down,
}

/// Retained node-graph canvas widget (MVP).
///
/// This draws nodes and wires and supports:
/// - pan (MMB drag, wheel without Ctrl),
/// - zoom (Ctrl+wheel, centered),
/// - node drag (LMB),
/// - connect ports (LMB drag pin -> pin).
pub type NodeGraphCanvas = NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>;

pub struct NodeGraphCanvasWith<M> {
    graph: Model<Graph>,
    view_state: Model<NodeGraphViewState>,
    store: Option<Model<NodeGraphStore>>,
    store_rev: Option<u64>,
    presenter: Box<dyn NodeGraphPresenter>,
    edge_types: Option<NodeGraphEdgeTypes>,
    callbacks: Option<Box<dyn NodeGraphCallbacks>>,
    middleware: M,
    style: NodeGraphStyle,
    close_command: Option<CommandId>,

    auto_measured: Arc<MeasuredGeometryStore>,
    auto_measured_key: Option<(u64, u32)>,

    edit_queue: Option<Model<NodeGraphEditQueue>>,
    edit_queue_key: Option<u64>,
    overlays: Option<Model<NodeGraphOverlayState>>,

    measured_output: Option<Arc<MeasuredGeometryStore>>,
    measured_output_key: Option<GeometryCacheKey>,

    internals: Option<Arc<NodeGraphInternalsStore>>,
    internals_key: Option<InternalsCacheKey>,

    cached_pan: CanvasPoint,
    cached_zoom: f32,
    history: GraphHistory,
    geometry: GeometryCache,

    paint_cache: CanvasPaintCache,
    grid_scene_cache: SceneOpTileCache<u64>,
    grid_tiles_scratch: Vec<TileCoord>,
    text_blobs: Vec<TextBlobId>,
    interaction: InteractionState,
}

impl NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        Self::new_with_middleware(graph, view_state, NoopNodeGraphCanvasMiddleware)
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    const REROUTE_INPUTS: usize = 1;
    const REROUTE_OUTPUTS: usize = 1;
    const AUTO_PAN_TICK_HZ: f32 = 60.0;
    const AUTO_PAN_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::AUTO_PAN_TICK_HZ) as u64);
    const PAN_INERTIA_TICK_HZ: f32 = 60.0;
    const PAN_INERTIA_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::PAN_INERTIA_TICK_HZ) as u64);
    const VIEWPORT_MOVE_END_DEBOUNCE: Duration = Duration::from_millis(180);
    const EDGE_FOCUS_ANCHOR_SIZE_SCREEN: f32 = 16.0;
    const EDGE_FOCUS_ANCHOR_PAD_SCREEN: f32 = 1.0;
    const EDGE_FOCUS_ANCHOR_BORDER_SCREEN: f32 = 2.0;
    const EDGE_FOCUS_ANCHOR_OFFSET_SCREEN: f32 = 18.0;
    const GRID_TILE_SIZE_SCREEN_PX: f32 = 2048.0;
    const GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget = InteractionBudget::new(32, 8);
    const EDGE_MARKER_BUILD_BUDGET_PER_FRAME: InteractionBudget = InteractionBudget::new(96, 24);
    const EDGE_LABEL_BUILD_BUDGET_PER_FRAME: InteractionBudget = InteractionBudget::new(16, 4);

    fn view_interacting(&self) -> bool {
        self.interaction.viewport_move_debounce.is_some()
            || self.interaction.panning
            || self.interaction.pan_inertia.is_some()
            || self.interaction.pending_marquee.is_some()
            || self.interaction.marquee.is_some()
            || self.interaction.pending_node_drag.is_some()
            || self.interaction.node_drag.is_some()
            || self.interaction.pending_group_drag.is_some()
            || self.interaction.group_drag.is_some()
            || self.interaction.pending_group_resize.is_some()
            || self.interaction.group_resize.is_some()
            || self.interaction.pending_node_resize.is_some()
            || self.interaction.node_resize.is_some()
            || self.interaction.pending_wire_drag.is_some()
            || self.interaction.wire_drag.is_some()
            || self.interaction.suspended_wire_drag.is_some()
            || self.interaction.pending_edge_insert_drag.is_some()
            || self.interaction.edge_insert_drag.is_some()
            || self.interaction.edge_drag.is_some()
            || self.interaction.pending_insert_node_drag.is_some()
            || self.interaction.insert_node_drag_preview.is_some()
            || self.interaction.context_menu.is_some()
            || self.interaction.searcher.is_some()
    }

    fn edge_render_hint(&self, graph: &Graph, edge_id: EdgeId) -> EdgeRenderHint {
        let base = self.presenter.edge_render_hint(graph, edge_id, &self.style);
        if let Some(edge_types) = self.edge_types.as_ref() {
            edge_types.apply(graph, edge_id, &self.style, base)
        } else {
            base
        }
    }

    fn group_rect_with_preview(
        &self,
        group_id: crate::core::GroupId,
        base: crate::core::CanvasRect,
    ) -> crate::core::CanvasRect {
        if let Some(resize) = self
            .interaction
            .group_resize
            .as_ref()
            .filter(|r| r.group == group_id)
        {
            return resize.current_rect;
        }
        if let Some(drag) = self
            .interaction
            .group_drag
            .as_ref()
            .filter(|d| d.group == group_id)
        {
            return drag.current_rect;
        }
        if let Some(rect) = self.interaction.node_resize.as_ref().and_then(|r| {
            r.current_groups
                .iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, rect)| *rect)
        }) {
            return rect;
        }
        if let Some(rect) = self.interaction.node_drag.as_ref().and_then(|d| {
            d.current_groups
                .iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, r)| *r)
        }) {
            return rect;
        }
        base
    }

    fn canvas_geometry<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> Arc<CanvasGeometry> {
        let zoom = snapshot.zoom;
        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let key = GeometryCacheKey {
            graph_rev,
            zoom_bits: snapshot.zoom.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev,
        };

        if self.geometry.key != Some(key) {
            self.geometry.drag_preview = None;
            let style = self.style.clone();
            let draw_order = snapshot.draw_order.clone();
            let graph = self.graph.clone();
            let presenter = &mut *self.presenter;
            let (geom, index) = graph
                .read_ref(host, |graph| {
                    let geom = CanvasGeometry::build_with_presenter(
                        graph,
                        &draw_order,
                        &style,
                        zoom,
                        presenter,
                    );
                    let z = zoom.max(1.0e-6);
                    let tuning = snapshot.interaction.spatial_index;
                    let cell_size_canvas = (tuning.cell_size_screen_px / z)
                        .max(tuning.min_cell_size_screen_px / z)
                        .max(1.0);
                    let max_hit_pad_canvas = (tuning.edge_aabb_pad_screen_px / z).max(0.0);
                    let index = CanvasSpatialIndex::build(
                        graph,
                        &geom,
                        zoom,
                        max_hit_pad_canvas,
                        cell_size_canvas,
                    );
                    (geom, index)
                })
                .ok()
                .unwrap_or_else(|| (CanvasGeometry::default(), CanvasSpatialIndex::empty()));
            self.geometry.key = Some(key);
            self.geometry.geom = Arc::new(geom);
            self.geometry.index = Arc::new(index);
        }

        self.geometry.geom.clone()
    }

    fn edge_focus_anchor_rect(center: Point, zoom: f32) -> Rect {
        let z = zoom.max(1.0e-6);
        let half = 0.5 * Self::EDGE_FOCUS_ANCHOR_SIZE_SCREEN / z;
        let pad = Self::EDGE_FOCUS_ANCHOR_PAD_SCREEN / z;
        let size = 2.0 * (half + pad);
        Rect::new(
            Point::new(Px(center.x.0 - half - pad), Px(center.y.0 - half - pad)),
            Size::new(Px(size), Px(size)),
        )
    }

    fn edge_focus_anchor_centers(
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> (Point, Point) {
        fn norm_or_fallback(v: Point, fallback: Point) -> Point {
            let len = (v.x.0 * v.x.0 + v.y.0 * v.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(v.x.0 / len), Px(v.y.0 / len));
            }
            let len = (fallback.x.0 * fallback.x.0 + fallback.y.0 * fallback.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(fallback.x.0 / len), Px(fallback.y.0 / len));
            }
            Point::new(Px(1.0), Px(0.0))
        }

        let z = zoom.max(1.0e-6);
        let off = Self::EDGE_FOCUS_ANCHOR_OFFSET_SCREEN / z;
        let fallback = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));

        let start_dir = norm_or_fallback(edge_route_start_tangent(route, from, to, zoom), fallback);
        let end_dir = norm_or_fallback(edge_route_end_tangent(route, from, to, zoom), fallback);

        let start = Point::new(
            Px(from.x.0 + start_dir.x.0 * off),
            Px(from.y.0 + start_dir.y.0 * off),
        );
        let end = Point::new(
            Px(to.x.0 - end_dir.x.0 * off),
            Px(to.y.0 - end_dir.y.0 * off),
        );
        (start, end)
    }

    pub(super) fn canvas_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> (Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>) {
        let geom = self.canvas_geometry(host, snapshot);
        let index = self.geometry.index.clone();
        let node_drag = self.interaction.node_drag.clone();
        let group_drag = self.interaction.group_drag.clone();
        let node_resize = self.interaction.node_resize.clone();

        if let Some(drag) = node_drag.as_ref() {
            if let Some((geom, index)) = self.drag_preview_derived(
                host,
                snapshot,
                DragPreviewKind::NodeDrag,
                drag.preview_rev,
                &drag.current_nodes,
            ) {
                return (geom, index);
            }
        } else if let Some(drag) = group_drag.as_ref() {
            if let Some((geom, index)) = self.drag_preview_derived(
                host,
                snapshot,
                DragPreviewKind::GroupDrag,
                drag.preview_rev,
                &drag.current_nodes,
            ) {
                return (geom, index);
            }
        } else if let Some(resize) = node_resize.as_ref() {
            if let Some((geom, index)) = self.node_resize_preview_derived(
                host,
                snapshot,
                resize.preview_rev,
                resize.node,
                resize.current_node_pos,
                resize.current_size_opt,
            ) {
                return (geom, index);
            }
        } else {
            self.geometry.drag_preview = None;
        }

        (geom, index)
    }

    fn update_ports_for_node_rect_change(
        geom: &mut CanvasGeometry,
        index: &mut CanvasSpatialIndex,
        node_id: GraphNodeId,
        prev_rect: Rect,
        next_rect: Rect,
        ports: &[PortId],
    ) {
        let eps = 1.0e-3;
        let prev_w = prev_rect.size.width.0;
        let next_w = next_rect.size.width.0;

        for port_id in ports {
            let Some(handle) = geom.ports.get_mut(port_id) else {
                continue;
            };
            if handle.node != node_id {
                continue;
            }

            let local_x = handle.center.x.0 - prev_rect.origin.x.0;
            let local_y = handle.center.y.0 - prev_rect.origin.y.0;
            let mut next_local_x = local_x;
            match handle.dir {
                PortDirection::In => {
                    if (local_x - 0.0).abs() <= eps {
                        next_local_x = 0.0;
                    }
                }
                PortDirection::Out => {
                    if (local_x - prev_w).abs() <= eps {
                        next_local_x = next_w;
                    }
                }
            }

            let center = Point::new(
                Px(next_rect.origin.x.0 + next_local_x),
                Px(next_rect.origin.y.0 + local_y),
            );
            let half_w = 0.5 * handle.bounds.size.width.0;
            let half_h = 0.5 * handle.bounds.size.height.0;
            let bounds = Rect::new(
                Point::new(Px(center.x.0 - half_w), Px(center.y.0 - half_h)),
                handle.bounds.size,
            );
            handle.center = center;
            handle.bounds = bounds;
            index.update_port_rect(*port_id, bounds);
        }
    }

    fn update_edges_for_ports(
        geom: &mut CanvasGeometry,
        index: &mut CanvasSpatialIndex,
        zoom: f32,
        ports: &[PortId],
        resolve_edges: impl FnOnce(&HashSet<EdgeId>) -> Vec<(EdgeId, PortId, PortId)>,
    ) {
        let mut edge_ids: HashSet<EdgeId> = HashSet::new();
        for port in ports {
            if let Some(edges) = index.edges_for_port(*port) {
                edge_ids.extend(edges.iter().copied());
            }
        }
        if edge_ids.is_empty() {
            return;
        }

        let endpoints = resolve_edges(&edge_ids);
        for (edge_id, from, to) in endpoints {
            let Some(p0) = geom.port_center(from) else {
                continue;
            };
            let Some(p1) = geom.port_center(to) else {
                continue;
            };
            let rect = index.edge_aabb(p0, p1, zoom);
            index.update_edge_rect(edge_id, rect);
        }
    }

    pub fn new_with_middleware(
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        middleware: M,
    ) -> Self {
        let auto_measured = Arc::new(MeasuredGeometryStore::new());
        Self {
            graph,
            view_state,
            store: None,
            store_rev: None,
            presenter: Box::new(FallbackMeasuredNodeGraphPresenter::new(
                DefaultNodeGraphPresenter::default(),
                auto_measured.clone(),
            )),
            edge_types: None,
            callbacks: None,
            middleware,
            style: NodeGraphStyle::default(),
            close_command: None,
            auto_measured,
            auto_measured_key: None,
            edit_queue: None,
            edit_queue_key: None,
            overlays: None,
            measured_output: None,
            measured_output_key: None,
            internals: None,
            internals_key: None,
            cached_pan: CanvasPoint::default(),
            cached_zoom: 1.0,
            history: GraphHistory::default(),
            geometry: GeometryCache::default(),
            paint_cache: CanvasPaintCache::default(),
            grid_scene_cache: SceneOpTileCache::default(),
            grid_tiles_scratch: Vec::new(),
            text_blobs: Vec::new(),
            interaction: InteractionState::default(),
        }
    }

    pub fn with_presenter(mut self, presenter: impl NodeGraphPresenter + 'static) -> Self {
        self.presenter = Box::new(FallbackMeasuredNodeGraphPresenter::new(
            presenter,
            self.auto_measured.clone(),
        ));
        self
    }

    /// Attaches a B-layer `edgeTypes` registry to override edge render hints.
    pub fn with_edge_types(mut self, edge_types: NodeGraphEdgeTypes) -> Self {
        self.edge_types = Some(edge_types);
        self
    }

    /// Attaches B-layer callbacks for controlled/uncontrolled integrations (ReactFlow-style).
    ///
    /// This is a convenience surface: callbacks are invoked for commits originating from this
    /// canvas (including undo/redo).
    pub fn with_callbacks(mut self, callbacks: impl NodeGraphCallbacks) -> Self {
        self.callbacks = Some(Box::new(callbacks));
        self
    }

    pub fn with_middleware<M2: NodeGraphCanvasMiddleware>(
        self,
        middleware: M2,
    ) -> NodeGraphCanvasWith<M2> {
        NodeGraphCanvasWith {
            graph: self.graph,
            view_state: self.view_state,
            store: self.store,
            store_rev: self.store_rev,
            presenter: self.presenter,
            edge_types: self.edge_types,
            callbacks: self.callbacks,
            middleware,
            style: self.style,
            close_command: self.close_command,
            auto_measured: self.auto_measured,
            auto_measured_key: self.auto_measured_key,
            edit_queue: self.edit_queue,
            edit_queue_key: self.edit_queue_key,
            overlays: self.overlays,
            measured_output: self.measured_output,
            measured_output_key: self.measured_output_key,
            internals: self.internals,
            internals_key: self.internals_key,
            cached_pan: self.cached_pan,
            cached_zoom: self.cached_zoom,
            history: self.history,
            geometry: self.geometry,
            paint_cache: self.paint_cache,
            grid_scene_cache: self.grid_scene_cache,
            grid_tiles_scratch: self.grid_tiles_scratch,
            text_blobs: self.text_blobs,
            interaction: self.interaction,
        }
    }

    /// Configures a store to receive derived measured geometry each frame.
    ///
    /// This is an output-only surface (similar to XyFlow "internals"):
    /// - the graph model stays pure data,
    /// - derived geometry is published for overlays/tooling,
    /// - the store is not consulted by this canvas unless you explicitly pass it into a presenter.
    pub fn with_measured_output_store(mut self, store: Arc<MeasuredGeometryStore>) -> Self {
        self.measured_output = Some(store);
        self.measured_output_key = None;
        self
    }

    pub fn with_internals_store(mut self, store: Arc<NodeGraphInternalsStore>) -> Self {
        self.internals = Some(store);
        self
    }

    pub fn with_style(mut self, style: NodeGraphStyle) -> Self {
        self.style = style;
        self.geometry.key = None;
        self
    }

    /// Adds a screen-space close button overlay that dispatches a command when clicked.
    ///
    /// This is intended for demos and tool windows; production apps typically wire close actions
    /// via docking/tab chrome instead.
    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = Some(command);
        self
    }

    /// Attaches a UI-side edit queue (`Model<NodeGraphEditQueue>`).
    ///
    /// The canvas drains queued transactions during layout and commits them through its normal
    /// apply + history pipeline (undo/redo).
    pub fn with_edit_queue(mut self, queue: Model<NodeGraphEditQueue>) -> Self {
        self.edit_queue = Some(queue);
        self.edit_queue_key = None;
        self
    }

    /// Attaches an overlay state model (`Model<NodeGraphOverlayState>`).
    pub fn with_overlay_state(mut self, overlays: Model<NodeGraphOverlayState>) -> Self {
        self.overlays = Some(overlays);
        self
    }

    /// Attaches a B-layer runtime store.
    ///
    /// When set, viewport and selection updates are written into the store and pulled back into
    /// `view_state` on demand.
    pub fn with_store(mut self, store: Model<NodeGraphStore>) -> Self {
        self.store = Some(store);
        self.store_rev = None;
        self
    }

    fn update_internals_store<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        bounds: Rect,
        geom: &CanvasGeometry,
    ) {
        let Some(store) = self.internals.as_ref() else {
            return;
        };

        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let key = InternalsCacheKey {
            graph_rev,
            zoom_bits: snapshot.zoom.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev,
            pan_x_bits: snapshot.pan.x.to_bits(),
            pan_y_bits: snapshot.pan.y.to_bits(),
            bounds_x_bits: bounds.origin.x.0.to_bits(),
            bounds_y_bits: bounds.origin.y.0.to_bits(),
            bounds_w_bits: bounds.size.width.0.to_bits(),
            bounds_h_bits: bounds.size.height.0.to_bits(),
        };

        if self.internals_key == Some(key) {
            return;
        }
        self.internals_key = Some(key);

        let transform = NodeGraphCanvasTransform {
            bounds_origin: bounds.origin,
            bounds_size: bounds.size,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        };

        let mut next = NodeGraphInternalsSnapshot {
            transform,
            ..NodeGraphInternalsSnapshot::default()
        };

        for (&node, node_geom) in &geom.nodes {
            next.nodes_window
                .insert(node, transform.canvas_rect_to_window(node_geom.rect));
        }
        for (&port, handle) in &geom.ports {
            next.ports_window
                .insert(port, transform.canvas_rect_to_window(handle.bounds));
            next.port_centers_window
                .insert(port, transform.canvas_point_to_window(handle.center));
        }

        next.focused_node = self.interaction.focused_node;
        next.focused_port = self.interaction.focused_port;
        next.focused_edge = self.interaction.focused_edge;
        next.connecting = self.interaction.wire_drag.is_some();

        let style = self.style.clone();
        let focused_node = self.interaction.focused_node;
        let focused_port = self.interaction.focused_port;
        let focused_edge = self.interaction.focused_edge;
        let labels = self
            .graph
            .read_ref(host, |graph| {
                let node_label = focused_node
                    .and_then(|node| self.presenter.a11y_node_label(graph, node))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_node.map(|node| format!("{:?}", node)));

                let port_label = focused_port
                    .and_then(|port| self.presenter.a11y_port_label(graph, port))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_port.map(|port| format!("{:?}", port)));

                let edge_label = focused_edge
                    .and_then(|edge| self.presenter.a11y_edge_label(graph, edge, &style))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_edge.map(|edge| format!("{:?}", edge)));

                (node_label, port_label, edge_label)
            })
            .ok()
            .unwrap_or_default();

        next.a11y_focused_node_label = labels.0.clone().map(|label| format!("Node {}", label));
        next.a11y_focused_port_label = labels.1.clone().map(|label| format!("Port {}", label));
        next.a11y_focused_edge_label = labels.2.clone().map(|label| format!("Edge {}", label));
        next.a11y_active_descendant_label = next
            .a11y_focused_port_label
            .clone()
            .or_else(|| next.a11y_focused_edge_label.clone())
            .or_else(|| next.a11y_focused_node_label.clone());

        store.update(next);
    }

    fn update_measured_output_store(&mut self, zoom: f32, geom: &CanvasGeometry) {
        let Some(store) = self.measured_output.as_ref() else {
            return;
        };
        let Some(key) = self.geometry.key else {
            return;
        };
        if self.measured_output_key == Some(key) {
            return;
        }
        self.measured_output_key = Some(key);

        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let quant = |v: f32| {
            (v / crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX).round()
                * crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX
        };

        let mut node_sizes: Vec<(GraphNodeId, (f32, f32))> = Vec::with_capacity(geom.nodes.len());
        for (&node, node_geom) in &geom.nodes {
            let w = quant(node_geom.rect.size.width.0 * zoom);
            let h = quant(node_geom.rect.size.height.0 * zoom);
            node_sizes.push((node, (w, h)));
        }

        let mut port_anchors: Vec<(PortId, PortAnchorHint)> = Vec::with_capacity(geom.ports.len());
        for (&port, handle) in &geom.ports {
            let Some(node_geom) = geom.nodes.get(&handle.node) else {
                continue;
            };
            let ox = node_geom.rect.origin.x.0;
            let oy = node_geom.rect.origin.y.0;

            let cx = quant((handle.center.x.0 - ox) * zoom);
            let cy = quant((handle.center.y.0 - oy) * zoom);
            let bx = quant((handle.bounds.origin.x.0 - ox) * zoom);
            let by = quant((handle.bounds.origin.y.0 - oy) * zoom);
            let bw = quant(handle.bounds.size.width.0 * zoom);
            let bh = quant(handle.bounds.size.height.0 * zoom);

            let center = Point::new(Px(cx), Px(cy));
            let bounds = Rect::new(Point::new(Px(bx), Px(by)), Size::new(Px(bw), Px(bh)));
            port_anchors.push((port, PortAnchorHint { center, bounds }));
        }

        let _ = store.apply_exclusive_batch_if_changed(
            crate::ui::measured::MeasuredGeometryExclusiveBatch {
                node_sizes_px: node_sizes,
                port_anchors_px: port_anchors,
            },
            crate::ui::measured::MeasuredGeometryApplyOptions::default(),
        );
    }

    fn apply_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        ops: Vec<GraphOp>,
    ) {
        let _ = self.apply_ops_result(host, window, ops);
    }

    fn apply_ops_result<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        ops: Vec<GraphOp>,
    ) -> bool {
        self.commit_ops(host, window, None, ops)
    }

    fn start_sticky_wire_drag_from_port<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        pos: Point,
    ) {
        self.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from,
                bundle: Vec::new(),
            },
            pos,
        });
        self.interaction.sticky_wire = true;
        self.interaction.sticky_wire_ignore_next_up = true;
        self.interaction.hover_port = None;
        self.interaction.hover_port_valid = false;
        self.interaction.hover_port_convertible = false;
        cx.capture_pointer(cx.node);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
    }

    fn restore_suspended_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: Option<PortId>,
        fallback_pos: Point,
    ) {
        if let Some(wire_drag) = self.interaction.suspended_wire_drag.take() {
            self.interaction.wire_drag = Some(wire_drag);
            self.interaction.sticky_wire = true;
            self.interaction.sticky_wire_ignore_next_up = true;
            self.interaction.hover_port = None;
            self.interaction.hover_port_valid = false;
            self.interaction.hover_port_convertible = false;
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        if let Some(from) = fallback_from {
            self.start_sticky_wire_drag_from_port(cx, from, fallback_pos);
        }
    }

    pub(super) fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
        fn snap_axis(value: f32, grid: f32) -> f32 {
            if !value.is_finite() {
                return value;
            }
            if !grid.is_finite() || grid <= 0.0 {
                return value;
            }
            (value / grid).round() * grid
        }

        CanvasPoint {
            x: snap_axis(pos.x, grid.width),
            y: snap_axis(pos.y, grid.height),
        }
    }

    pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return CanvasPoint::default();
        }

        let margin_screen = snapshot.interaction.auto_pan.margin;
        let speed_screen_per_s = snapshot.interaction.auto_pan.speed;
        if !margin_screen.is_finite() || margin_screen <= 0.0 {
            return CanvasPoint::default();
        }
        if !speed_screen_per_s.is_finite() || speed_screen_per_s <= 0.0 {
            return CanvasPoint::default();
        }

        let viewport_w = bounds.size.width.0;
        let viewport_h = bounds.size.height.0;
        if !viewport_w.is_finite()
            || viewport_w <= 0.0
            || !viewport_h.is_finite()
            || viewport_h <= 0.0
        {
            return CanvasPoint::default();
        }

        let pan = snapshot.pan;
        let pos_screen_x = (pos.x.0 + pan.x) * zoom;
        let pos_screen_y = (pos.y.0 + pan.y) * zoom;

        let dist_left = pos_screen_x;
        let dist_right = viewport_w - pos_screen_x;
        let dist_top = pos_screen_y;
        let dist_bottom = viewport_h - pos_screen_y;

        let step_screen = speed_screen_per_s / Self::AUTO_PAN_TICK_HZ;
        let step_graph = step_screen / zoom;

        let mut delta_x = 0.0;
        let mut delta_y = 0.0;

        if dist_left.is_finite() && dist_left < margin_screen {
            let factor = ((margin_screen - dist_left) / margin_screen).clamp(0.0, 1.0);
            delta_x += step_graph * factor;
        }
        if dist_right.is_finite() && dist_right < margin_screen {
            let factor = ((margin_screen - dist_right) / margin_screen).clamp(0.0, 1.0);
            delta_x -= step_graph * factor;
        }
        if dist_top.is_finite() && dist_top < margin_screen {
            let factor = ((margin_screen - dist_top) / margin_screen).clamp(0.0, 1.0);
            delta_y += step_graph * factor;
        }
        if dist_bottom.is_finite() && dist_bottom < margin_screen {
            let factor = ((margin_screen - dist_bottom) / margin_screen).clamp(0.0, 1.0);
            delta_y -= step_graph * factor;
        }

        if !delta_x.is_finite() || !delta_y.is_finite() {
            return CanvasPoint::default();
        }

        CanvasPoint {
            x: delta_x,
            y: delta_y,
        }
    }

    fn wire_drag_suppresses_edge(kind: &WireDragKind, edge_id: EdgeId) -> bool {
        match kind {
            WireDragKind::Reconnect { edge, .. } => *edge == edge_id,
            WireDragKind::ReconnectMany { edges } => {
                edges.iter().any(|(edge, ..)| *edge == edge_id)
            }
            _ => false,
        }
    }

    fn clamp_context_menu_origin(
        &self,
        desired: Point,
        item_count: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = context_menu_rect_at(&self.style, desired, item_count, snapshot.zoom);

        let viewport_w = bounds.size.width.0 / snapshot.zoom;
        let viewport_h = bounds.size.height.0 / snapshot.zoom;
        let viewport_origin_x = -snapshot.pan.x;
        let viewport_origin_y = -snapshot.pan.y;

        let min_x = viewport_origin_x;
        let min_y = viewport_origin_y;
        let max_x = viewport_origin_x + (viewport_w - rect.size.width.0).max(0.0);
        let max_y = viewport_origin_y + (viewport_h - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }

    fn clamp_searcher_origin(
        &self,
        desired: Point,
        visible_rows: usize,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> Point {
        let rect = searcher_rect_at(&self.style, desired, visible_rows, snapshot.zoom);

        let viewport_w = bounds.size.width.0 / snapshot.zoom;
        let viewport_h = bounds.size.height.0 / snapshot.zoom;
        let viewport_origin_x = -snapshot.pan.x;
        let viewport_origin_y = -snapshot.pan.y;

        let min_x = viewport_origin_x;
        let min_y = viewport_origin_y;
        let max_x = viewport_origin_x + (viewport_w - rect.size.width.0).max(0.0);
        let max_y = viewport_origin_y + (viewport_h - rect.size.height.0).max(0.0);

        Point::new(
            Px(desired.x.0.clamp(min_x, max_x)),
            Px(desired.y.0.clamp(min_y, max_y)),
        )
    }

    fn node_default_size_for_ports(&self, inputs: usize, outputs: usize) -> (f32, f32) {
        let rows = inputs.max(outputs) as f32;
        let base = self.style.node_header_height + 2.0 * self.style.node_padding;
        let pin_area = rows * self.style.pin_row_height;
        (self.style.node_width, base + pin_area)
    }

    fn reroute_pos_for_invoked_at(&self, invoked_at: Point) -> CanvasPoint {
        let (w, h) = self.node_default_size_for_ports(Self::REROUTE_INPUTS, Self::REROUTE_OUTPUTS);
        CanvasPoint {
            x: invoked_at.x.0 - 0.5 * w,
            y: invoked_at.y.0 - 0.5 * h,
        }
    }

    fn first_added_node_id(ops: &[GraphOp]) -> Option<GraphNodeId> {
        for op in ops {
            if let GraphOp::AddNode { id, .. } = op {
                return Some(*id);
            }
        }
        None
    }

    fn build_reroute_create_ops(at: CanvasPoint) -> Vec<GraphOp> {
        let node_id = GraphNodeId::new();
        let in_port_id = PortId::new();
        let out_port_id = PortId::new();

        let node = crate::core::Node {
            kind: NodeKindKey::new(REROUTE_KIND),
            kind_version: 1,
            pos: at,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        };

        let in_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("in"),
            dir: PortDirection::In,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        };

        let out_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("out"),
            dir: PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        };

        vec![
            GraphOp::AddNode { id: node_id, node },
            GraphOp::AddPort {
                id: in_port_id,
                port: in_port,
            },
            GraphOp::AddPort {
                id: out_port_id,
                port: out_port,
            },
            GraphOp::SetNodePorts {
                id: node_id,
                from: Vec::new(),
                to: vec![in_port_id, out_port_id],
            },
        ]
    }

    fn create_group_at<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        let size = crate::core::CanvasSize {
            width: 480.0,
            height: 320.0,
        };
        let origin = crate::core::CanvasPoint {
            x: at.x - 0.5 * size.width,
            y: at.y - 0.5 * size.height,
        };
        let group = crate::core::Group {
            title: "Group".to_string(),
            rect: crate::core::CanvasRect { origin, size },
            color: None,
        };
        let group_id = crate::core::GroupId::new();
        let ops = vec![GraphOp::AddGroup {
            id: group_id,
            group,
        }];
        if self.commit_ops(host, window, Some("Create Group"), ops) {
            self.update_view_state(host, |s| {
                s.selected_nodes.clear();
                s.selected_edges.clear();
                s.selected_groups.clear();
                s.selected_groups.push(group_id);
                s.group_draw_order.retain(|id| *id != group_id);
                s.group_draw_order.push(group_id);
            });
        }
    }

    fn record_recent_kind(&mut self, kind: &NodeKindKey) {
        const MAX_RECENT: usize = 20;

        self.interaction.recent_kinds.retain(|k| k != kind);
        self.interaction.recent_kinds.insert(0, kind.clone());
        if self.interaction.recent_kinds.len() > MAX_RECENT {
            self.interaction.recent_kinds.truncate(MAX_RECENT);
        }
    }

    fn searcher_is_selectable_row(row: &SearcherRow) -> bool {
        matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
    }

    fn searcher_first_selectable_row(rows: &[SearcherRow]) -> usize {
        rows.iter()
            .position(Self::searcher_is_selectable_row)
            .unwrap_or(0)
    }

    fn rebuild_searcher_rows(searcher: &mut SearcherState) {
        let rows = match &searcher.target {
            ContextMenuTarget::ConnectionConvertPicker { .. } => {
                super::searcher::build_rows_flat(&searcher.candidates, &searcher.query)
            }
            _ => super::searcher::build_rows(
                &searcher.candidates,
                &searcher.query,
                &searcher.recent_kinds,
            ),
        };

        searcher.rows = rows;
        searcher.scroll = searcher.scroll.min(
            searcher
                .rows
                .len()
                .saturating_sub(SEARCHER_MAX_VISIBLE_ROWS),
        );
        searcher.active_row = Self::searcher_first_selectable_row(&searcher.rows)
            .min(searcher.rows.len().saturating_sub(1));
        Self::ensure_searcher_active_visible(searcher);
    }

    fn ensure_searcher_active_visible(searcher: &mut SearcherState) {
        let n = searcher.rows.len();
        if n == 0 {
            searcher.active_row = 0;
            searcher.scroll = 0;
            return;
        }

        let visible = SEARCHER_MAX_VISIBLE_ROWS.min(n);
        let max_scroll = n.saturating_sub(visible);
        searcher.scroll = searcher.scroll.min(max_scroll);

        if searcher.active_row < searcher.scroll {
            searcher.scroll = searcher.active_row;
        } else if searcher.active_row >= searcher.scroll + visible {
            searcher.scroll = (searcher.active_row + 1).saturating_sub(visible);
        }
        searcher.scroll = searcher.scroll.min(max_scroll);
    }

    fn try_activate_searcher_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
    ) -> bool {
        let Some(searcher) = self.interaction.searcher.take() else {
            return false;
        };

        let Some(row) = searcher.rows.get(row_ix).cloned() else {
            self.interaction.searcher = Some(searcher);
            return false;
        };

        let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
            self.interaction.searcher = Some(searcher);
            return false;
        };
        if !row.enabled {
            self.interaction.searcher = Some(searcher);
            return false;
        }

        let item = NodeGraphContextMenuItem {
            label: row.label,
            enabled: true,
            action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
        };
        self.activate_context_menu_item(
            cx,
            &searcher.target,
            searcher.invoked_at,
            item,
            &searcher.candidates,
        );
        true
    }

    fn open_insert_node_picker<H: UiHost>(&mut self, host: &mut H, at: CanvasPoint) {
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| presenter.list_insertable_nodes(graph))
                .ok()
                .unwrap_or_default()
        };

        let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
        menu_candidates.push(InsertNodeCandidate {
            kind: NodeKindKey::new(REROUTE_KIND),
            label: Arc::<str>::from("Reroute"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        });
        menu_candidates.extend(candidates);

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let rows =
            super::searcher::build_rows(&menu_candidates, "", &self.interaction.recent_kinds);
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin =
            self.clamp_searcher_origin(Point::new(Px(at.x), Px(at.y)), visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at: Point::new(Px(at.x), Px(at.y)),
            target: ContextMenuTarget::BackgroundInsertNodePicker { at },
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
    }

    fn open_connection_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
    ) {
        let candidates: Vec<InsertNodeCandidate> = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| {
                    presenter.list_insertable_nodes_for_connection(graph, from)
                })
                .ok()
                .unwrap_or_default()
        };

        let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
        menu_candidates.push(InsertNodeCandidate {
            kind: NodeKindKey::new(REROUTE_KIND),
            label: Arc::<str>::from("Reroute"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        });
        menu_candidates.extend(candidates);

        let snapshot = self.sync_view_state(host);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let rows =
            super::searcher::build_rows(&menu_candidates, "", &self.interaction.recent_kinds);
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let origin =
            self.clamp_searcher_origin(Point::new(Px(at.x), Px(at.y)), visible, bounds, &snapshot);
        let active_row = rows
            .iter()
            .position(|r| matches!(r.kind, SearcherRowKind::Candidate { .. }) && r.enabled)
            .unwrap_or(0);

        self.interaction.context_menu = None;
        self.interaction.searcher = Some(SearcherState {
            origin,
            invoked_at: Point::new(Px(at.x), Px(at.y)),
            target: ContextMenuTarget::ConnectionInsertNodePicker { from, at },
            query: String::new(),
            candidates: menu_candidates,
            recent_kinds: self.interaction.recent_kinds.clone(),
            rows,
            hovered_row: None,
            active_row,
            scroll: 0,
        });
    }

    fn open_edge_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        edge: EdgeId,
        invoked_at: Point,
    ) {
        edge_insert::open_edge_insert_node_picker(self, host, window, edge, invoked_at);
    }

    fn paint_context_menu<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        menu: &ContextMenuState,
        zoom: f32,
    ) {
        let rect = context_menu_rect_at(&self.style, menu.origin, menu.items.len(), zoom);
        let border_w = Px(1.0 / zoom);
        let radius = Px(self.style.context_menu_corner_radius / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(50),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(border_w),
            border_color: self.style.context_menu_border,
            corner_radii: Corners::all(radius),
        });

        let pad = self.style.context_menu_padding / zoom;
        let item_h = self.style.context_menu_item_height / zoom;
        let inner_x = rect.origin.x.0 + pad;
        let inner_y = rect.origin.y.0 + pad;
        let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(inner_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: effective_scale_factor(cx.scale_factor, zoom),
        };

        for (ix, item) in menu.items.iter().enumerate() {
            let item_rect = Rect::new(
                Point::new(Px(inner_x), Px(inner_y + ix as f32 * item_h)),
                Size::new(Px(inner_w), Px(item_h)),
            );

            let is_active = menu.active_item == ix;
            let is_hovered = menu.hovered_item == Some(ix);
            if (is_hovered || is_active) && item.enabled {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(51),
                    rect: item_rect,
                    background: self.style.context_menu_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(4.0 / zoom)),
                });
            }

            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                item.label.clone(),
                &text_style,
                constraints,
            );

            let text_x = item_rect.origin.x;
            let inner_y =
                item_rect.origin.y.0 + (item_rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            let color = if item.enabled {
                self.style.context_menu_text
            } else {
                self.style.context_menu_text_disabled
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(52),
                origin: Point::new(text_x, text_y),
                text: blob,
                color,
            });
        }
    }

    fn paint_marquee<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        marquee: &MarqueeDrag,
        zoom: f32,
    ) {
        let rect = rect_from_points(marquee.start_pos, marquee.pos);
        let border_w = Px(self.style.marquee_border_width / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(49),
            rect,
            background: self.style.marquee_fill,
            border: Edges::all(border_w),
            border_color: self.style.marquee_border,
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    fn paint_snap_guides<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        guides: &SnapGuides,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        let w = Px((self.style.snapline_width / zoom).max(0.5 / zoom));
        let half = 0.5 * w.0;

        if let Some(x) = guides.x {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(48),
                rect: Rect::new(
                    Point::new(Px(x - half), Px(viewport_origin_y)),
                    Size::new(w, Px(viewport_h)),
                ),
                background: self.style.snapline_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if let Some(y) = guides.y {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(48),
                rect: Rect::new(
                    Point::new(Px(viewport_origin_x), Px(y - half)),
                    Size::new(Px(viewport_w), w),
                ),
                background: self.style.snapline_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }
    }

    fn paint_toast<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        toast: &ToastState,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_h: f32,
    ) {
        let margin = 12.0 / zoom;
        let pad = 10.0 / zoom;
        let max_w = 420.0 / zoom;

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(max_w - 2.0 * pad)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: effective_scale_factor(cx.scale_factor, zoom),
        };

        let (blob, metrics) = self.paint_cache.text_blob(
            cx.services,
            toast.message.clone(),
            &text_style,
            constraints,
        );

        let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(120.0 / zoom, max_w);
        let box_h = metrics.size.height.0 + 2.0 * pad;

        let x = viewport_origin_x + margin;
        let y = viewport_origin_y + viewport_h - box_h - margin;
        let rect = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(box_w), Px(box_h)));

        let border_color = match toast.severity {
            DiagnosticSeverity::Info => Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
            DiagnosticSeverity::Warning => Color {
                r: 0.95,
                g: 0.75,
                b: 0.20,
                a: 1.0,
            },
            DiagnosticSeverity::Error => Color {
                r: 0.90,
                g: 0.35,
                b: 0.35,
                a: 1.0,
            },
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(70),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0 / zoom)),
            border_color,
            corner_radii: Corners::all(Px(6.0 / zoom)),
        });

        let text_x = Px(rect.origin.x.0 + pad);
        let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(71),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.style.context_menu_text,
        });
    }

    fn paint_wire_drag_hint<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        wire_drag: &WireDrag,
        zoom: f32,
    ) {
        let text = match &wire_drag.kind {
            WireDragKind::New { bundle, .. } if bundle.len() > 1 => {
                Arc::<str>::from(format!("Bundle: {}", bundle.len()))
            }
            WireDragKind::ReconnectMany { edges } if edges.len() > 1 => {
                Arc::<str>::from(format!("Yank: {}", edges.len()))
            }
            _ => return,
        };

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let pad = 8.0 / zoom;
        let max_w = 220.0 / zoom;
        let constraints = TextConstraints {
            max_width: Some(Px(max_w - 2.0 * pad)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: effective_scale_factor(cx.scale_factor, zoom),
        };

        let (blob, metrics) =
            self.paint_cache
                .text_blob(cx.services, text, &text_style, constraints);

        let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(72.0 / zoom, max_w);
        let box_h = metrics.size.height.0 + 2.0 * pad;

        let offset_x = 14.0 / zoom;
        let offset_y = 12.0 / zoom;
        let rect = Rect::new(
            Point::new(
                Px(wire_drag.pos.x.0 + offset_x),
                Px(wire_drag.pos.y.0 + offset_y),
            ),
            Size::new(Px(box_w), Px(box_h)),
        );

        let border_color = if snapshot.interaction.connection_mode == NodeGraphConnectionMode::Loose
            && self.interaction.hover_port.is_some()
            && !self.interaction.hover_port_valid
        {
            if self.interaction.hover_port_convertible {
                Color {
                    r: 0.95,
                    g: 0.75,
                    b: 0.20,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.90,
                    g: 0.35,
                    b: 0.35,
                    a: 1.0,
                }
            }
        } else {
            self.style.context_menu_border
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(69),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(Px(1.0 / zoom)),
            border_color,
            corner_radii: Corners::all(Px(6.0 / zoom)),
        });

        let text_x = Px(rect.origin.x.0 + pad);
        let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(70),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.style.context_menu_text,
        });
    }

    fn yank_edges_from_port(graph: &Graph, port: PortId) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        let Some(p) = graph.ports.get(&port) else {
            return Vec::new();
        };

        let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();
        match p.dir {
            PortDirection::Out => {
                for (edge_id, edge) in &graph.edges {
                    if edge.from == port {
                        out.push((*edge_id, EdgeEndpoint::From, edge.to));
                    }
                }
            }
            PortDirection::In => {
                for (edge_id, edge) in &graph.edges {
                    if edge.to == port {
                        out.push((*edge_id, EdgeEndpoint::To, edge.from));
                    }
                }
            }
        }
        out
    }

    fn yank_reconnectable_edges_from_port<H: UiHost>(
        &self,
        host: &mut H,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        if let Some(store) = self.store.as_ref() {
            if let Ok(out) = store.read_ref(host, |s| {
                use crate::runtime::lookups::ConnectionSide;

                let graph = s.graph();
                let Some(p) = graph.ports.get(&port) else {
                    return Vec::new();
                };

                let node = p.node;
                let side = ConnectionSide::from_port_dir(p.dir);
                let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();

                let Some(conns) = s.lookups().connections_for_port(node, side, port) else {
                    return out;
                };

                for (edge_id, conn) in conns {
                    let (endpoint, fixed) = match p.dir {
                        PortDirection::Out => (EdgeEndpoint::From, conn.target_port),
                        PortDirection::In => (EdgeEndpoint::To, conn.source_port),
                    };
                    if Self::edge_endpoint_is_reconnectable(graph, interaction, *edge_id, endpoint)
                    {
                        out.push((*edge_id, endpoint, fixed));
                    }
                }

                out.sort_by_key(|(edge_id, _, _)| *edge_id);
                out
            }) {
                return out;
            }
        }

        self.graph
            .read_ref(host, |graph| {
                let mut edges = Self::yank_edges_from_port(graph, port);
                edges.retain(|(edge_id, endpoint, _fixed)| {
                    Self::edge_endpoint_is_reconnectable(graph, interaction, *edge_id, *endpoint)
                });
                edges
            })
            .ok()
            .unwrap_or_default()
    }

    fn edge_is_deletable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
    ) -> bool {
        if !interaction.edges_deletable {
            return false;
        }
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        edge.deletable.unwrap_or(true)
    }

    fn edge_reconnectable_flags(
        edge: &crate::core::Edge,
        interaction: &NodeGraphInteractionState,
    ) -> (bool, bool) {
        match edge.reconnectable {
            Some(crate::core::EdgeReconnectable::Bool(false)) => (false, false),
            Some(crate::core::EdgeReconnectable::Bool(true)) => (true, true),
            Some(crate::core::EdgeReconnectable::Endpoint(
                crate::core::EdgeReconnectableEndpoint::Source,
            )) => (true, false),
            Some(crate::core::EdgeReconnectable::Endpoint(
                crate::core::EdgeReconnectableEndpoint::Target,
            )) => (false, true),
            None => {
                let allow = interaction.edges_reconnectable;
                (allow, allow)
            }
        }
    }

    fn edge_endpoint_is_reconnectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
    ) -> bool {
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        let (allow_source, allow_target) = Self::edge_reconnectable_flags(edge, interaction);
        match endpoint {
            EdgeEndpoint::From => allow_source,
            EdgeEndpoint::To => allow_target,
        }
    }

    fn node_is_draggable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.nodes_draggable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.draggable.unwrap_or(true)
    }

    fn port_is_connectable_base(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port) = graph.ports.get(&port) else {
            return false;
        };
        let node_connectable = Self::node_is_connectable(graph, interaction, port.node);
        port.connectable.unwrap_or(node_connectable)
    }

    fn port_is_connectable_start(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port_value) = graph.ports.get(&port) else {
            return false;
        };
        if !Self::port_is_connectable_base(graph, interaction, port) {
            return false;
        }
        port_value.connectable_start.unwrap_or(true)
    }

    fn port_is_connectable_end(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port_value) = graph.ports.get(&port) else {
            return false;
        };
        if !Self::port_is_connectable_base(graph, interaction, port) {
            return false;
        }
        port_value.connectable_end.unwrap_or(true)
    }

    fn node_is_connectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.connectable.unwrap_or(interaction.nodes_connectable)
    }

    fn node_is_deletable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.nodes_deletable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.deletable.unwrap_or(true)
    }

    fn should_add_bundle_port(
        graph: &Graph,
        from: PortId,
        bundle: &[PortId],
        candidate: PortId,
    ) -> bool {
        if candidate == from || bundle.contains(&candidate) {
            return false;
        }
        let Some(from_port) = graph.ports.get(&from) else {
            return false;
        };
        let Some(candidate_port) = graph.ports.get(&candidate) else {
            return false;
        };
        candidate_port.dir == from_port.dir
    }

    fn zoom_about_center_factor(&mut self, bounds: Rect, factor: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }

        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let mut view = PanZoom2D {
            pan: Point::new(Px(self.cached_pan.x), Px(self.cached_pan.y)),
            zoom,
        };
        let center = Point::new(
            Px(0.5 * bounds.size.width.0),
            Px(0.5 * bounds.size.height.0),
        );
        view.zoom_about_screen_point(bounds, center, new_zoom);
        self.cached_pan = CanvasPoint {
            x: view.pan.x.0,
            y: view.pan.y.0,
        };
        self.cached_zoom = view.zoom;
    }

    fn zoom_about_pointer_factor(&mut self, position: Point, factor: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        if !position.x.0.is_finite() || !position.y.0.is_finite() {
            return;
        }

        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let pan_x = self.cached_pan.x;
        let pan_y = self.cached_pan.y;

        // `position` is in the widget's local (canvas) coordinates.
        // Compute the pivot in screen coordinates (relative to bounds origin) to keep the
        // graph point under the cursor stable.
        let pivot_screen_x = (position.x.0 + pan_x) * zoom;
        let pivot_screen_y = (position.y.0 + pan_y) * zoom;

        let g0_x = pivot_screen_x / zoom - pan_x;
        let g0_y = pivot_screen_y / zoom - pan_y;

        let new_pan_x = pivot_screen_x / new_zoom - g0_x;
        let new_pan_y = pivot_screen_y / new_zoom - g0_y;

        self.cached_pan = CanvasPoint {
            x: new_pan_x,
            y: new_pan_y,
        };
        self.cached_zoom = new_zoom;
    }

    fn stop_auto_pan_timer<H: UiHost>(&mut self, host: &mut H) {
        let Some(timer) = self.interaction.auto_pan_timer.take() else {
            return;
        };
        host.push_effect(Effect::CancelTimer { token: timer });
    }

    fn stop_pan_inertia_timer<H: UiHost>(&mut self, host: &mut H) {
        let Some(inertia) = self.interaction.pan_inertia.take() else {
            return;
        };
        host.push_effect(Effect::CancelTimer {
            token: inertia.timer,
        });
    }

    fn bump_viewport_move_debounce<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
    ) {
        if let Some(prev) = self.interaction.viewport_move_debounce.take() {
            host.push_effect(Effect::CancelTimer { token: prev.timer });
            if prev.kind != kind {
                self.emit_move_end(snapshot, prev.kind, ViewportMoveEndOutcome::Ended);
                self.emit_move_start(snapshot, kind);
            }
        } else {
            self.emit_move_start(snapshot, kind);
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::VIEWPORT_MOVE_END_DEBOUNCE,
            repeat: None,
        });
        self.interaction.viewport_move_debounce = Some(ViewportMoveDebounceState { kind, timer });
    }

    fn pan_inertia_should_tick(&self) -> bool {
        if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
            return false;
        }
        if self.interaction.panning {
            return false;
        }
        self.interaction.pending_marquee.is_none()
            && self.interaction.marquee.is_none()
            && self.interaction.pending_node_drag.is_none()
            && self.interaction.node_drag.is_none()
            && self.interaction.pending_group_drag.is_none()
            && self.interaction.group_drag.is_none()
            && self.interaction.pending_group_resize.is_none()
            && self.interaction.group_resize.is_none()
            && self.interaction.pending_node_resize.is_none()
            && self.interaction.node_resize.is_none()
            && self.interaction.pending_wire_drag.is_none()
            && self.interaction.wire_drag.is_none()
            && self.interaction.edge_drag.is_none()
    }

    fn maybe_start_pan_inertia_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.stop_pan_inertia_timer(host);

        let tuning = &snapshot.interaction.pan_inertia;
        if !tuning.enabled {
            return false;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return false;
        }

        let mut velocity = self.interaction.pan_velocity;
        if !velocity.x.is_finite() || !velocity.y.is_finite() {
            return false;
        }

        let speed_screen = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt() * zoom;
        let min_speed = tuning.min_speed.max(0.0);
        if !speed_screen.is_finite() || speed_screen < min_speed {
            return false;
        }

        let max_speed = tuning.max_speed.max(min_speed);
        if max_speed.is_finite() && max_speed > 0.0 {
            let max_speed_canvas = max_speed / zoom;
            let speed_canvas = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
            if speed_canvas.is_finite() && speed_canvas > max_speed_canvas && speed_canvas > 0.0 {
                let scale = max_speed_canvas / speed_canvas;
                velocity.x *= scale;
                velocity.y *= scale;
            }
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::PAN_INERTIA_TICK_INTERVAL,
            repeat: Some(Self::PAN_INERTIA_TICK_INTERVAL),
        });
        self.interaction.pan_inertia = Some(PanInertiaState {
            timer,
            velocity,
            last_tick_at: Instant::now(),
        });
        true
    }

    fn ensure_auto_pan_timer_running<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) {
        if self.interaction.auto_pan_timer.is_some() {
            return;
        }
        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::AUTO_PAN_TICK_INTERVAL,
            repeat: Some(Self::AUTO_PAN_TICK_INTERVAL),
        });
        self.interaction.auto_pan_timer = Some(timer);
    }

    fn auto_pan_should_tick(&self, snapshot: &ViewSnapshot, bounds: Rect) -> bool {
        if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
            return false;
        }
        let Some(pos) = self.interaction.last_pos else {
            return false;
        };

        let wants_node_drag = snapshot.interaction.auto_pan.on_node_drag
            && (self.interaction.node_drag.is_some()
                || self.interaction.group_drag.is_some()
                || self.interaction.group_resize.is_some());
        let wants_connect =
            snapshot.interaction.auto_pan.on_connect && self.interaction.wire_drag.is_some();

        if !wants_node_drag && !wants_connect {
            return false;
        }

        let delta = Self::auto_pan_delta(snapshot, pos, bounds);
        delta.x != 0.0 || delta.y != 0.0
    }

    fn sync_auto_pan_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) {
        if self.auto_pan_should_tick(snapshot, bounds) {
            self.ensure_auto_pan_timer_running(host, window);
        } else {
            self.stop_auto_pan_timer(host);
        }
    }

    fn focus_next_edge<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable
            || !snapshot.interaction.edges_selectable
            || !snapshot.interaction.edges_focusable
        {
            return false;
        }

        let mut edges: Vec<EdgeId> = self
            .graph
            .read_ref(host, |g| {
                g.edges
                    .keys()
                    .copied()
                    .filter(|id| Self::edge_is_selectable(g, &snapshot.interaction, *id))
                    .collect()
            })
            .ok()
            .unwrap_or_default();
        if edges.is_empty() {
            return false;
        }
        edges.sort_unstable();

        let current = self
            .interaction
            .focused_edge
            .or_else(|| snapshot.selected_edges.first().copied());

        let next = match current.and_then(|id| edges.iter().position(|e| *e == id)) {
            Some(ix) => {
                let len = edges.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                edges[next_ix]
            }
            None => {
                if forward {
                    edges[0]
                } else {
                    edges[edges.len() - 1]
                }
            }
        };

        self.interaction.focused_edge = Some(next);
        self.interaction.focused_node = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(host, |s| {
            s.selected_nodes.clear();
            s.selected_groups.clear();
            s.selected_edges.clear();
            s.selected_edges.push(next);
        });
        true
    }

    fn focus_next_node<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let ordered: Vec<GraphNodeId> = self
            .graph
            .read_ref(host, |g| {
                let mut out: Vec<GraphNodeId> = Vec::new();
                let mut used: HashSet<GraphNodeId> = HashSet::new();

                for id in &snapshot.draw_order {
                    if Self::node_is_selectable(g, &snapshot.interaction, *id) && used.insert(*id) {
                        out.push(*id);
                    }
                }

                let mut rest: Vec<GraphNodeId> = g
                    .nodes
                    .keys()
                    .copied()
                    .filter(|id| Self::node_is_selectable(g, &snapshot.interaction, *id))
                    .filter(|id| used.insert(*id))
                    .collect();
                rest.sort_unstable();
                out.extend(rest);
                out
            })
            .ok()
            .unwrap_or_default();

        if ordered.is_empty() {
            return false;
        }

        let current = self
            .interaction
            .focused_node
            .or_else(|| snapshot.selected_nodes.first().copied());

        let next = match current.and_then(|id| ordered.iter().position(|e| *e == id)) {
            Some(ix) => {
                let len = ordered.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                ordered[next_ix]
            }
            None => {
                if forward {
                    ordered[0]
                } else {
                    ordered[ordered.len() - 1]
                }
            }
        };

        self.interaction.focused_node = Some(next);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(next);
            s.draw_order.retain(|id| *id != next);
            s.draw_order.push(next);
        });
        true
    }

    fn refresh_focused_port_hints<H: UiHost>(&mut self, host: &mut H) {
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;

        let snapshot = self.sync_view_state(host);
        let mode = snapshot.interaction.connection_mode;

        let Some(target) = self.interaction.focused_port else {
            return;
        };
        let Some(wire_drag) = self.interaction.wire_drag.clone() else {
            return;
        };

        let presenter = &mut *self.presenter;
        let (valid, convertible) = self
            .graph
            .read_ref(host, |graph| {
                let mut scratch = graph.clone();

                let valid = match &wire_drag.kind {
                    WireDragKind::New { from, bundle } => {
                        let sources = if bundle.is_empty() {
                            std::slice::from_ref(from)
                        } else {
                            bundle.as_slice()
                        };
                        let mut any_accept = false;
                        for src in sources {
                            let plan = presenter.plan_connect(&scratch, *src, target, mode);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                    WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
                        presenter
                            .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode)
                            .decision,
                        ConnectDecision::Accept
                    ),
                    WireDragKind::ReconnectMany { edges } => {
                        let mut any_accept = false;
                        for (edge, endpoint, _fixed) in edges {
                            let plan = presenter
                                .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                };

                let convertible = if !valid {
                    match &wire_drag.kind {
                        WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                            conversion::is_convertible(presenter, &scratch, *from, target)
                        }
                        _ => false,
                    }
                } else {
                    false
                };

                (valid, convertible)
            })
            .ok()
            .unwrap_or((false, false));

        if self.interaction.wire_drag.is_some() && self.interaction.focused_port == Some(target) {
            self.interaction.focused_port_valid = valid;
            self.interaction.focused_port_convertible = convertible;
        }
    }

    fn focus_next_port<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let focused_node = self
            .interaction
            .focused_node
            .or_else(|| snapshot.selected_nodes.first().copied())
            .or_else(|| {
                self.graph
                    .read_ref(host, |g| g.nodes.keys().next().copied())
                    .ok()
                    .flatten()
            });

        let Some(focused_node) = focused_node else {
            return false;
        };

        let wire_dir = self.interaction.wire_drag.as_ref().and_then(|w| {
            let from_port = match &w.kind {
                WireDragKind::New { from, .. } => Some(*from),
                WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
            }?;
            self.graph
                .read_ref(host, |g| g.ports.get(&from_port).map(|p| p.dir))
                .ok()
                .flatten()
        });

        let ports = self
            .graph
            .read_ref(host, |g| {
                let (inputs, outputs) = node_ports(g, focused_node);
                let mut ports = Vec::with_capacity(inputs.len() + outputs.len());
                ports.extend(inputs);
                ports.extend(outputs);

                if let Some(wire_dir) = wire_dir {
                    let want = match wire_dir {
                        PortDirection::In => PortDirection::Out,
                        PortDirection::Out => PortDirection::In,
                    };
                    ports.retain(|id| g.ports.get(id).is_some_and(|p| p.dir == want));
                }

                ports
            })
            .ok()
            .unwrap_or_default();

        if ports.is_empty() {
            return false;
        }

        let current = self
            .interaction
            .focused_port
            .filter(|id| ports.iter().any(|p| *p == *id));

        let next = match current.and_then(|id| ports.iter().position(|p| *p == id)) {
            Some(ix) => {
                let len = ports.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                ports[next_ix]
            }
            None => {
                if forward {
                    ports[0]
                } else {
                    ports[ports.len() - 1]
                }
            }
        };

        self.interaction.focused_node = Some(focused_node);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = Some(next);
        self.refresh_focused_port_hints(host);
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(focused_node);
        });
        true
    }

    fn port_center_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        port: PortId,
    ) -> Option<CanvasPoint> {
        let (geom, _) = self.canvas_derived(&*host, snapshot);
        geom.ports.get(&port).map(|h| CanvasPoint {
            x: h.center.x.0,
            y: h.center.y.0,
        })
    }

    fn activate_focused_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let Some(port) = self
            .interaction
            .focused_port
            .or(self.interaction.hover_port)
        else {
            return false;
        };

        let pos = self
            .port_center_canvas(cx.app, snapshot, port)
            .map(|p| Point::new(Px(p.x), Px(p.y)))
            .or(self.interaction.last_pos)
            .unwrap_or_else(|| {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                Point::new(
                    Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
                    Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
                )
            });

        if self.interaction.wire_drag.is_none() {
            self.interaction.wire_drag = Some(WireDrag {
                kind: WireDragKind::New {
                    from: port,
                    bundle: Vec::new(),
                },
                pos,
            });
            self.interaction.click_connect = true;
            self.interaction.pending_wire_drag = None;
            self.interaction.suspended_wire_drag = None;
            self.interaction.sticky_wire = false;
            self.interaction.sticky_wire_ignore_next_up = false;
            self.interaction.focused_edge = None;
            self.interaction.focused_port = None;
            self.interaction.focused_port_valid = false;
            self.interaction.focused_port_convertible = false;
            self.interaction.hover_port = None;
            self.interaction.hover_port_valid = false;
            self.interaction.hover_port_convertible = false;
            return true;
        }

        if let Some(mut w) = self.interaction.wire_drag.take() {
            w.pos = pos;
            self.interaction.wire_drag = Some(w);
        }

        let _ = wire_drag::handle_wire_left_up_with_forced_target(
            self,
            cx,
            snapshot,
            snapshot.zoom,
            Some(port),
        );
        self.refresh_focused_port_hints(cx.app);
        true
    }
}

impl<H: UiHost, M: NodeGraphCanvasMiddleware> Widget<H> for NodeGraphCanvasWith<M> {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.paint_cache.clear(services);
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let snapshot = self.sync_view_state(cx.app);
        if cx.input_ctx.focus_is_text_input && command.as_str().starts_with("node_graph.") {
            return false;
        }

        let mw_outcome = {
            let mw_ctx = NodeGraphCanvasMiddlewareCx {
                graph: &self.graph,
                view_state: &self.view_state,
                style: &self.style,
                bounds: self.interaction.last_bounds,
                pan: snapshot.pan,
                zoom: snapshot.zoom,
            };
            self.middleware.handle_command(cx, &mw_ctx, command)
        };
        if mw_outcome == NodeGraphCanvasCommandOutcome::Handled {
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return true;
        }

        self.handle_command(cx, &snapshot, command)
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        self.interaction.last_bounds = Some(cx.bounds);
        let snapshot = self.sync_view_state(cx.app);

        cx.set_role(fret_core::SemanticsRole::Viewport);
        cx.set_focusable(true);
        cx.set_label(self.presenter.a11y_canvas_label().as_ref());

        let active_descendant = match (
            self.interaction.focused_port.is_some(),
            self.interaction.focused_edge.is_some(),
            self.interaction.focused_node.is_some(),
        ) {
            (true, _, _) => cx.children.get(0).copied(),
            (false, true, _) => cx.children.get(1).copied(),
            (false, false, true) => cx.children.get(2).copied(),
            _ => None,
        };
        cx.set_active_descendant(active_descendant);

        let (focused_node, focused_port, focused_edge) = (
            self.interaction.focused_node,
            self.interaction.focused_port,
            self.interaction.focused_edge,
        );

        let style = self.style.clone();
        let value = self
            .graph
            .read_ref(cx.app, |graph| {
                let mut parts: Vec<String> = Vec::new();
                parts.push(format!("zoom {:.3}", snapshot.zoom));
                parts.push(format!(
                    "selected nodes {}, edges {}, groups {}",
                    snapshot.selected_nodes.len(),
                    snapshot.selected_edges.len(),
                    snapshot.selected_groups.len(),
                ));

                if self.interaction.wire_drag.is_some() {
                    parts.push("connecting".to_string());
                }

                if let Some(node) = focused_node {
                    if let Some(label) = self.presenter.a11y_node_label(graph, node) {
                        parts.push(format!("focused node {}", label));
                    } else {
                        parts.push(format!("focused node {:?}", node));
                    }
                }

                if let Some(port) = focused_port {
                    if let Some(label) = self.presenter.a11y_port_label(graph, port) {
                        parts.push(format!("focused port {}", label));
                    } else {
                        parts.push(format!("focused port {:?}", port));
                    }
                }

                if let Some(edge) = focused_edge {
                    if let Some(label) = self.presenter.a11y_edge_label(graph, edge, &style) {
                        parts.push(format!("focused edge {}", label));
                    } else {
                        parts.push(format!("focused edge {:?}", edge));
                    }
                }

                parts.join("; ")
            })
            .ok()
            .unwrap_or_else(|| format!("zoom {:.3}", snapshot.zoom));

        cx.set_value(value);
    }

    fn render_transform(&self, bounds: Rect) -> Option<Transform2D> {
        let view = PanZoom2D {
            pan: Point::new(Px(self.cached_pan.x), Px(self.cached_pan.y)),
            zoom: self.cached_zoom,
        };
        view.render_transform(bounds)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);
        if let Some(queue) = self.edit_queue.as_ref() {
            cx.observe_model(queue, Invalidation::Layout);
        }
        for &child in cx.children {
            cx.layout_in(child, cx.bounds);
        }
        self.interaction.last_bounds = Some(cx.bounds);
        self.sync_view_state(cx.app);
        self.drain_edit_queue(cx.app, cx.window);
        self.update_auto_measured_node_sizes(cx);
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let snapshot = self.sync_view_state(cx.app);
        self.interaction.last_bounds = Some(cx.bounds);

        let mw_outcome = {
            let mw_ctx = NodeGraphCanvasMiddlewareCx {
                graph: &self.graph,
                view_state: &self.view_state,
                style: &self.style,
                bounds: Some(cx.bounds),
                pan: snapshot.pan,
                zoom: snapshot.zoom,
            };
            self.middleware.handle_event(cx, &mw_ctx, event)
        };
        if mw_outcome == NodeGraphCanvasEventOutcome::Handled {
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        self.handle_event(cx, event, &snapshot);
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paint_root(cx);
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px, Rect,
        Size, TextBlobId,
    };
    use fret_runtime::CommandId;
    use fret_runtime::ui_host::{
        CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, DragKindId, DragSession, DragSessionId, Effect, FrameId,
    };
    use fret_runtime::{ModelHost, ModelStore, TickId, TimerToken};
    use fret_ui::retained_bridge::Widget as _;
    use serde_json::Value;
    use std::any::{Any, TypeId};
    use std::collections::{HashMap, HashSet};
    use std::time::Instant;

    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group,
        GroupId, Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey,
        PortKind,
    };

    mod callbacks_conformance;
    mod connect_conformance;
    mod connection_mode_conformance;
    mod hit_testing_conformance;
    mod insert_node_drag_conformance;
    mod interaction_conformance;
    mod internals_conformance;
    mod invalidation_ordering_conformance;
    mod middleware_conformance;
    mod perf_cache;
    mod portal_conformance;
    mod portal_keyboard_conformance;
    mod portal_lifecycle_conformance;
    mod portal_pointer_passthrough_conformance;

    #[test]
    fn inflate_rect_expands_by_margin() {
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let inflated = super::inflate_rect(rect, 5.0);
        assert_eq!(
            inflated,
            Rect::new(Point::new(Px(5.0), Px(15.0)), Size::new(Px(40.0), Px(50.0)))
        );
    }

    #[test]
    fn edge_bounds_rect_applies_padding() {
        let from = Point::new(Px(10.0), Px(10.0));
        let to = Point::new(Px(30.0), Px(20.0));
        let bounds = super::edge_bounds_rect(
            crate::ui::presenter::EdgeRouteKind::Straight,
            from,
            to,
            1.0,
            2.0,
        );
        assert_eq!(
            bounds,
            Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(24.0), Px(14.0)))
        );
    }

    #[test]
    fn middle_mouse_panning_tracks_screen_delta_under_render_transform() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let mut snapshot = canvas.sync_view_state(cx.app);
        assert_eq!(snapshot.zoom, 1.0);
        assert_eq!(snapshot.pan, CanvasPoint::default());

        canvas.interaction.panning = true;
        canvas.interaction.pan_last_sample_at = Some(Instant::now());
        canvas.interaction.pan_last_screen_pos = None;

        let screen_positions = [
            Point::new(Px(100.0), Px(100.0)),
            Point::new(Px(140.0), Px(100.0)),
            Point::new(Px(190.0), Px(100.0)),
        ];

        for screen in screen_positions {
            let zoom = snapshot.zoom;
            let pan = snapshot.pan;
            let local = Point::new(
                Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
                Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
            );
            assert!(super::pan_zoom::handle_panning_move(
                &mut canvas,
                &mut cx,
                &snapshot,
                local,
            ));
            snapshot = canvas.sync_view_state(cx.app);
        }

        let expected_pan_x = screen_positions.last().unwrap().x.0 - screen_positions[0].x.0;
        assert!((snapshot.pan.x - expected_pan_x).abs() <= 1.0e-3);
        assert!((snapshot.pan.y - 0.0).abs() <= 1.0e-3);
    }

    #[test]
    fn space_to_pan_starts_left_mouse_panning_and_updates_viewport() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let mut snapshot = canvas.sync_view_state(cx.app);
        assert!(snapshot.interaction.space_to_pan);
        assert_eq!(snapshot.zoom, 1.0);
        assert_eq!(snapshot.pan, CanvasPoint::default());

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(100.0), Px(100.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(canvas.interaction.panning);
        assert_eq!(canvas.interaction.panning_button, Some(MouseButton::Left));
        assert!(canvas.interaction.pending_marquee.is_none());
        assert!(canvas.interaction.marquee.is_none());

        let screen_positions = [
            Point::new(Px(100.0), Px(100.0)),
            Point::new(Px(140.0), Px(100.0)),
            Point::new(Px(190.0), Px(100.0)),
        ];
        for screen in screen_positions {
            let zoom = snapshot.zoom;
            let pan = snapshot.pan;
            let local = Point::new(
                Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
                Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
            );
            canvas.event(
                &mut cx,
                &Event::Pointer(PointerEvent::Move {
                    pointer_id: fret_core::PointerId::default(),
                    position: local,
                    buttons: MouseButtons {
                        left: true,
                        ..MouseButtons::default()
                    },
                    modifiers: Modifiers::default(),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            snapshot = canvas.sync_view_state(cx.app);
        }

        let expected_pan_x = screen_positions.last().unwrap().x.0 - screen_positions[0].x.0;
        assert!((snapshot.pan.x - expected_pan_x).abs() <= 1.0e-3);
        assert!((snapshot.pan.y - 0.0).abs() <= 1.0e-3);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId::default(),
                position: *screen_positions.last().unwrap(),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(!canvas.interaction.panning);

        canvas.event(
            &mut cx,
            &Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );
        assert!(!canvas.interaction.pan_activation_key_held);
        assert_eq!(canvas.history.undo_len(), 0);
    }

    #[test]
    fn pan_activation_key_code_must_match_to_enable_space_to_pan() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.space_to_pan = true;
            s.interaction.pan_activation_key_code =
                Some(crate::io::NodeGraphKeyCode(fret_core::KeyCode::KeyP));
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let _snapshot = canvas.sync_view_state(cx.app);
        assert!(!canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            !canvas.interaction.pan_activation_key_held,
            "Space should not activate panning when pan_activation_key_code is KeyP"
        );

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::KeyP,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::KeyUp {
                key: fret_core::KeyCode::KeyP,
                modifiers: Modifiers::default(),
            },
        );
        assert!(!canvas.interaction.pan_activation_key_held);
    }

    #[test]
    fn pan_activation_key_code_none_disables_space_to_pan_activation() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.space_to_pan = true;
            s.interaction.pan_activation_key_code = None;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let _snapshot = canvas.sync_view_state(cx.app);

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            !canvas.interaction.pan_activation_key_held,
            "pan_activation_key_code=None should disable activation"
        );
    }

    #[test]
    fn pan_on_scroll_mode_horizontal_ignores_vertical_wheel_delta() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.pan_on_scroll = true;
            s.interaction.pan_on_scroll_speed = 1.0;
            s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Horizontal;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let before = canvas.sync_view_state(cx.app).pan;
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after = canvas.sync_view_state(cx.app).pan;
        assert_eq!(before, after);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(80.0), Px(0.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after2 = canvas.sync_view_state(cx.app).pan;
        assert!((after2.x - after.x - 80.0).abs() <= 1.0e-3);
        assert!((after2.y - after.y).abs() <= 1.0e-3);
    }

    #[test]
    fn pan_on_scroll_shift_maps_vertical_wheel_to_horizontal_on_windows() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.pan_on_scroll = true;
            s.interaction.pan_on_scroll_speed = 1.0;
            s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);
        cx.input_ctx.platform = fret_runtime::Platform::Windows;

        let before = canvas.sync_view_state(cx.app).pan;
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after = canvas.sync_view_state(cx.app).pan;
        assert!((after.x - before.x - 120.0).abs() <= 1.0e-3);
        assert!((after.y - before.y).abs() <= 1.0e-3);
    }

    #[test]
    fn space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.pan_on_scroll = false;
            s.interaction.pan_on_scroll_speed = 1.0;
            s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
            s.interaction.zoom_on_scroll = false;
            s.interaction.space_to_pan = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let before = canvas.sync_view_state(cx.app).pan;
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after = canvas.sync_view_state(cx.app).pan;
        assert_eq!(before, after);

        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(canvas.interaction.pan_activation_key_held);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let after2 = canvas.sync_view_state(cx.app).pan;
        assert!((after2.y - after.y - 120.0).abs() <= 1.0e-3);
    }

    #[test]
    fn pinch_gesture_zooms_in_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_pinch = true;
            s.interaction.zoom_on_pinch_speed = 1.0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(100.0), Px(100.0));
        let before = canvas.sync_view_state(cx.app);
        assert_eq!(before.zoom, 1.0);
        assert_eq!(before.pan, CanvasPoint::default());

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::PinchGesture {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                delta: 1.0,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 2.0).abs() <= 1.0e-6);
        assert!((after.pan.x - -50.0).abs() <= 1.0e-3);
        assert!((after.pan.y - -50.0).abs() <= 1.0e-3);
    }

    #[test]
    fn pinch_gesture_respects_toggle() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_pinch = false;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::PinchGesture {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(100.0), Px(100.0)),
                delta: 1.0,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert_eq!(after.zoom, 1.0);
        assert_eq!(after.pan, CanvasPoint::default());
    }

    #[test]
    fn wheel_zoom_zooms_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_scroll = true;
            s.interaction.zoom_on_scroll_speed = 1.0;
            s.interaction.zoom_activation_key = crate::io::NodeGraphZoomActivationKey::None;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(100.0), Px(100.0));
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                delta: Point::new(Px(0.0), Px(-120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 1.18).abs() <= 1.0e-4);
        assert!((after.pan.x - -15.254).abs() <= 1.0e-3);
        assert!((after.pan.y - -15.254).abs() <= 1.0e-3);
    }

    #[test]
    fn delete_key_defaults_to_backspace() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());
        let mut canvas = NodeGraphCanvas::new(graph, view);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Delete,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }
        assert!(
            host.effects.is_empty(),
            "Delete should not dispatch delete-selection by default (XYFlow default is Backspace)"
        );

        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Backspace,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }
        assert!(
            host.effects.iter().any(|e| matches!(
                e,
                Effect::Command { command, .. }
                    if *command == CommandId::from(crate::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
            )),
            "Backspace should dispatch delete-selection by default"
        );
    }

    #[test]
    fn disable_keyboard_a11y_does_not_block_delete_shortcut() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.disable_keyboard_a11y = true;
            s.interaction.delete_key = crate::io::NodeGraphDeleteKey::BackspaceOrDelete;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();

        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Backspace,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        assert!(
            host.effects.iter().any(|e| matches!(
                e,
                Effect::Command { command, .. }
                    if *command == CommandId::from(crate::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
            )),
            "delete shortcut should still work when disable_keyboard_a11y is enabled (XYFlow parity)"
        );
    }

    #[test]
    fn disable_keyboard_a11y_blocks_tab_focus_traversal() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.disable_keyboard_a11y = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();

        {
            let mut cx = event_cx(&mut host, &mut services, bounds);
            cx.window = Some(AppWindowId::default());
            canvas.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Tab,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        assert!(
            host.effects.is_empty(),
            "Tab focus traversal should not dispatch focus commands when disable_keyboard_a11y is enabled"
        );
    }

    #[test]
    fn double_click_background_zooms_in_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_double_click = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(600.0), Px(500.0));
        let before = canvas.sync_view_state(cx.app);
        assert_eq!(before.zoom, 1.0);
        assert_eq!(before.pan, CanvasPoint::default());

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 2.0).abs() <= 1.0e-6);
        assert!((after.pan.x - -300.0).abs() <= 1.0e-3);
        assert!((after.pan.y - -250.0).abs() <= 1.0e-3);
        assert!(canvas.interaction.pending_marquee.is_none());
        assert!(canvas.interaction.marquee.is_none());
    }

    #[test]
    fn shift_double_click_background_zooms_out_about_pointer() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_double_click = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let pos = Point::new(Px(600.0), Px(500.0));
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let after = canvas.sync_view_state(cx.app);
        assert!((after.zoom - 0.5).abs() <= 1.0e-6);
        assert!((after.pan.x - 600.0).abs() <= 1.0e-3);
        assert!((after.pan.y - 500.0).abs() <= 1.0e-3);
    }

    #[test]
    fn double_click_edge_inserts_reroute_when_enabled() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(420.0);
        let edge_id = EdgeId::new();
        graph_value.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.zoom_on_double_click = true;
            s.interaction.reroute_on_edge_double_click = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let snap = canvas.sync_view_state(cx.app);
        let (geom, _index) = canvas.canvas_derived(&*cx.app, &snap);
        let from = geom.port_center(a_out).expect("from port center");
        let to = geom.port_center(b_in).expect("to port center");
        let (c1, c2) = super::wire_ctrl_points(from, to, snap.zoom);
        let pos = super::cubic_bezier(from, c1, c2, to, 0.5);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
        let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
        assert_eq!(nodes_len, 3);
        assert_eq!(edges_len, 2);
        assert!(
            graph
                .read_ref(cx.app, |g| g.edges.contains_key(&edge_id))
                .unwrap_or(false)
        );
        assert!(
            graph
                .read_ref(cx.app, |g| g
                    .nodes
                    .values()
                    .any(|n| n.kind.0 == crate::REROUTE_KIND))
                .unwrap_or(false)
        );

        let after = canvas.sync_view_state(cx.app);
        assert_eq!(after.selected_edges.len(), 0);
        assert_eq!(after.selected_nodes.len(), 1);
        assert_eq!(after.zoom, 1.0);
    }

    #[test]
    fn alt_double_click_edge_opens_insert_node_picker() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(420.0);
        let edge_id = EdgeId::new();
        graph_value.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let snap = canvas.sync_view_state(cx.app);
        let (geom, _index) = canvas.canvas_derived(&*cx.app, &snap);
        let from = geom.port_center(a_out).expect("from port center");
        let to = geom.port_center(b_in).expect("to port center");
        let (c1, c2) = super::wire_ctrl_points(from, to, snap.zoom);
        let pos = super::cubic_bezier(from, c1, c2, to, 0.5);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    alt: true,
                    ..Modifiers::default()
                },
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
        let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
        assert_eq!(nodes_len, 2);
        assert_eq!(edges_len, 1);

        let Some(searcher) = canvas.interaction.searcher.as_ref() else {
            panic!("expected searcher to be open");
        };
        assert!(matches!(
            searcher.target,
            super::super::state::ContextMenuTarget::EdgeInsertNodePicker(e) if e == edge_id
        ));
    }

    #[test]
    fn alt_drag_edge_opens_insert_node_picker_when_enabled() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(420.0);
        let edge_id = EdgeId::new();
        graph_value.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.edge_insert_on_alt_drag = true;
        });

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let snap = canvas.sync_view_state(cx.app);
        let (geom, _index) = canvas.canvas_derived(&*cx.app, &snap);
        let from = geom.port_center(a_out).expect("from port center");
        let to = geom.port_center(b_in).expect("to port center");
        let (c1, c2) = super::wire_ctrl_points(from, to, snap.zoom);
        let edge_pos = super::cubic_bezier(from, c1, c2, to, 0.5);

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: edge_pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    alt: true,
                    ..Modifiers::default()
                },
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(edge_pos.x.0 + 16.0), edge_pos.y),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers {
                    alt: true,
                    ..Modifiers::default()
                },
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(edge_pos.x.0 + 16.0), edge_pos.y),
                button: MouseButton::Left,
                modifiers: Modifiers {
                    alt: true,
                    ..Modifiers::default()
                },
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
        let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
        assert_eq!(nodes_len, 2);
        assert_eq!(edges_len, 1);

        let Some(searcher) = canvas.interaction.searcher.as_ref() else {
            panic!("expected searcher to be open");
        };
        assert!(matches!(
            searcher.target,
            super::super::state::ContextMenuTarget::EdgeInsertNodePicker(e) if e == edge_id
        ));
    }

    #[test]
    fn internal_drag_drop_candidate_on_edge_splits_edge() {
        use std::sync::Arc;

        use crate::core::{PortCapacity, PortDirection, PortKey, PortKind};
        use crate::rules::{InsertNodeTemplate, PortTemplate};
        use crate::ui::presenter::InsertNodeCandidate;
        use fret_core::{InternalDragEvent, InternalDragKind};

        let mut host = TestUiHostImpl::default();
        let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(420.0);
        let edge_id = EdgeId::new();
        graph_value.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let template_kind = NodeKindKey::new("test.mid");
        let template = InsertNodeTemplate {
            kind: template_kind.clone(),
            kind_version: 1,
            collapsed: false,
            data: Value::Null,
            ports: vec![
                PortTemplate {
                    key: PortKey::new("in"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: None,
                    data: Value::Null,
                },
                PortTemplate {
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: None,
                    data: Value::Null,
                },
            ],
            input: PortKey::new("in"),
            output: PortKey::new("out"),
        };
        let candidate = InsertNodeCandidate {
            kind: template_kind.clone(),
            label: Arc::<str>::from("Mid"),
            enabled: true,
            template: Some(template),
            payload: Value::Null,
        };

        host.drag = Some(DragSession::new_cross_window(
            DragSessionId(1),
            fret_core::PointerId(0),
            AppWindowId::default(),
            super::insert_node_drag::DRAG_KIND_INSERT_NODE,
            Point::new(Px(0.0), Px(0.0)),
            super::insert_node_drag::InsertNodeDragPayload { candidate },
        ));

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        let snap = canvas.sync_view_state(cx.app);
        let (geom, _index) = canvas.canvas_derived(&*cx.app, &snap);
        let from = geom.port_center(a_out).expect("from port center");
        let to = geom.port_center(b_in).expect("to port center");
        let (c1, c2) = super::wire_ctrl_points(from, to, snap.zoom);
        let pos = super::cubic_bezier(from, c1, c2, to, 0.5);

        canvas.event(
            &mut cx,
            &Event::InternalDrag(InternalDragEvent {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                kind: InternalDragKind::Drop,
                modifiers: Modifiers::default(),
            }),
        );

        let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
        let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
        assert_eq!(nodes_len, 3);
        assert_eq!(edges_len, 2);
        assert!(
            graph
                .read_ref(cx.app, |g| g.edges.contains_key(&edge_id))
                .unwrap_or(false)
        );
        assert!(
            graph
                .read_ref(cx.app, |g| g
                    .nodes
                    .values()
                    .any(|n| n.kind == template_kind))
                .unwrap_or(false)
        );

        let after = canvas.sync_view_state(cx.app);
        assert_eq!(after.selected_nodes.len(), 1);
        assert_eq!(after.selected_edges.len(), 0);
    }
    use crate::rules::EdgeEndpoint;
    use crate::ui::commands::{
        CMD_NODE_GRAPH_ACTIVATE, CMD_NODE_GRAPH_ALIGN_CENTER_X, CMD_NODE_GRAPH_ALIGN_LEFT,
        CMD_NODE_GRAPH_ALIGN_RIGHT, CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_DISTRIBUTE_X,
        CMD_NODE_GRAPH_FOCUS_NEXT, CMD_NODE_GRAPH_FOCUS_NEXT_PORT, CMD_NODE_GRAPH_FOCUS_PORT_LEFT,
        CMD_NODE_GRAPH_FOCUS_PORT_RIGHT, CMD_NODE_GRAPH_FOCUS_PREV, CMD_NODE_GRAPH_FOCUS_PREV_PORT,
        CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_NUDGE_RIGHT_FAST, CMD_NODE_GRAPH_SELECT_ALL,
    };

    use super::super::state::{NodeDrag, ViewSnapshot, WireDrag, WireDragKind};
    use super::NodeGraphCanvas;

    #[derive(Default)]
    struct NullServices;

    impl fret_core::TextService for NullServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (TextBlobId, fret_core::TextMetrics) {
            (
                TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl fret_core::PathService for NullServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for NullServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct TestUiHostImpl {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        redraw: HashSet<AppWindowId>,
        effects: Vec<Effect>,
        drag: Option<DragSession>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHostImpl {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|b| b.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            if !self.globals.contains_key(&type_id) {
                self.globals.insert(type_id, Box::new(init()));
            }

            // Avoid aliasing `&mut self` by temporarily removing the value.
            let boxed = self
                .globals
                .remove(&type_id)
                .expect("global must exist")
                .downcast::<T>()
                .ok()
                .expect("global has wrong type");
            let mut value = *boxed;

            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestUiHostImpl {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestUiHostImpl {
        fn take_changed_models(&mut self) -> Vec<fret_runtime::ModelId> {
            self.models.take_changed_models()
        }
    }

    impl CommandsHost for TestUiHostImpl {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestUiHostImpl {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraw.insert(window);
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for TestUiHostImpl {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            TimerToken(self.next_timer_token)
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            ClipboardToken(self.next_clipboard_token)
        }

        fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            fret_runtime::ImageUploadToken(self.next_image_upload_token)
        }
    }

    impl DragHost for TestUiHostImpl {
        fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
            self.drag
                .as_ref()
                .filter(|drag| drag.pointer_id == pointer_id)
        }

        fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
            self.drag
                .as_mut()
                .filter(|drag| drag.pointer_id == pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
            if self.drag(pointer_id).is_some() {
                self.drag = None;
            }
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new(
                DragSessionId(1),
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ));
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.drag = Some(DragSession::new_cross_window(
                DragSessionId(1),
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ));
        }
    }

    fn event_cx<'a>(
        host: &'a mut TestUiHostImpl,
        services: &'a mut NullServices,
        bounds: Rect,
    ) -> fret_ui::retained_bridge::EventCx<'a, TestUiHostImpl> {
        fret_ui::retained_bridge::EventCx {
            app: host,
            services,
            node: fret_core::NodeId::default(),
            window: None,
            input_ctx: fret_runtime::InputContext::default(),
            pointer_id: None,
            children: &[],
            focus: None,
            captured: None,
            bounds,
            invalidations: Vec::new(),
            requested_focus: None,
            requested_capture: None,
            requested_cursor: None,
            stop_propagation: false,
        }
    }

    fn command_cx<'a>(
        host: &'a mut TestUiHostImpl,
        services: &'a mut NullServices,
        tree: &'a mut fret_ui::UiTree<TestUiHostImpl>,
    ) -> fret_ui::retained_bridge::CommandCx<'a, TestUiHostImpl> {
        fret_ui::retained_bridge::CommandCx {
            app: host,
            services,
            tree,
            node: fret_core::NodeId::default(),
            window: None,
            input_ctx: fret_runtime::InputContext::default(),
            focus: None,
            invalidations: Vec::new(),
            requested_focus: None,
            stop_propagation: false,
        }
    }

    fn make_test_graph_two_nodes() -> (Graph, NodeId, NodeId) {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();

        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        (graph, a, b)
    }

    fn make_test_graph_two_nodes_with_size() -> (Graph, NodeId, NodeId) {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();

        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 5.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        (graph, a, b)
    }

    fn make_test_graph_two_nodes_with_ports() -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let a_in = PortId::new();
        let a_out = PortId::new();
        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![a_in, a_out],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            a_in,
            Port {
                node: a,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            a_out,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let b = NodeId::new();
        let b_in = PortId::new();
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 200.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![b_in],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            b_in,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        (graph, a, a_in, a_out, b, b_in)
    }

    fn make_test_graph_two_nodes_with_ports_spaced_x(
        dx: f32,
    ) -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
        let (mut graph, a, a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
        graph
            .nodes
            .entry(b)
            .and_modify(|n| n.pos = CanvasPoint { x: dx, y: 0.0 });
        (graph, a, a_in, a_out, b, b_in)
    }

    fn read_node_pos(
        host: &mut TestUiHostImpl,
        model: &fret_runtime::Model<Graph>,
        id: NodeId,
    ) -> CanvasPoint {
        model
            .read_ref(host, |g| g.nodes.get(&id).map(|n| n.pos))
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    #[test]
    fn distance_sq_point_to_rect_is_zero_inside_and_positive_outside() {
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(100.0), Px(50.0)),
        );
        let inside = Point::new(Px(50.0), Px(40.0));
        assert!(NodeGraphCanvas::rect_contains_point(rect, inside));
        assert_eq!(
            NodeGraphCanvas::distance_sq_point_to_rect(inside, rect),
            0.0
        );

        let outside = Point::new(Px(0.0), Px(0.0));
        assert!(!NodeGraphCanvas::rect_contains_point(rect, outside));
        assert!(NodeGraphCanvas::distance_sq_point_to_rect(outside, rect) > 0.0);
    }

    #[test]
    fn yank_edges_from_port_returns_all_incident_edges() {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let n1 = NodeId::new();
        let p_out = PortId::new();
        graph.nodes.insert(
            n1,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![p_out],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_out,
            Port {
                node: n1,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let n2 = NodeId::new();
        let p_in1 = PortId::new();
        let p_in2 = PortId::new();
        graph.nodes.insert(
            n2,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![p_in1, p_in2],
                data: Value::Null,
            },
        );
        for (id, key) in [(p_in1, "in1"), (p_in2, "in2")] {
            graph.ports.insert(
                id,
                Port {
                    node: n2,
                    key: PortKey::new(key),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: None,
                    data: Value::Null,
                },
            );
        }

        let e1 = EdgeId::new();
        let e2 = EdgeId::new();
        graph.edges.insert(
            e1,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in1,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            e2,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in2,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let from_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_out);
        assert_eq!(from_edges.len(), 2);
        assert!(from_edges.contains(&(e1, EdgeEndpoint::From, p_in1)));
        assert!(from_edges.contains(&(e2, EdgeEndpoint::From, p_in2)));

        let to_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_in1);
        assert_eq!(to_edges, vec![(e1, EdgeEndpoint::To, p_out)]);
    }

    #[test]
    fn should_add_bundle_port_requires_same_side_and_dedupes() {
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");
        let n1 = NodeId::new();

        let p_out1 = PortId::new();
        let p_out2 = PortId::new();
        let p_in = PortId::new();

        graph.nodes.insert(
            n1,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![p_out1, p_out2, p_in],
                data: Value::Null,
            },
        );

        graph.ports.insert(
            p_out1,
            Port {
                node: n1,
                key: PortKey::new("out1"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_out2,
            Port {
                node: n1,
                key: PortKey::new("out2"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_in,
            Port {
                node: n1,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        assert!(NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[p_out1],
            p_out2
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[p_out2],
            p_out2
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[],
            p_out1
        ));
        assert!(!NodeGraphCanvas::should_add_bundle_port(
            &graph,
            p_out1,
            &[],
            p_in
        ));
    }

    #[test]
    fn node_drag_records_single_history_entry_for_multi_node_move() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
        let snapshot = canvas.sync_view_state(&mut host);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        canvas.interaction.node_drag = Some(NodeDrag {
            primary: a,
            node_ids: vec![a, b],
            nodes: vec![
                (a, CanvasPoint { x: 0.0, y: 0.0 }),
                (b, CanvasPoint { x: 10.0, y: 0.0 }),
            ],
            current_nodes: vec![
                (a, CanvasPoint { x: 0.0, y: 0.0 }),
                (b, CanvasPoint { x: 10.0, y: 0.0 }),
            ],
            current_groups: Vec::new(),
            preview_rev: 0,
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_pos: Point::new(Px(0.0), Px(0.0)),
        });

        for pos in [
            Point::new(Px(20.0), Px(5.0)),
            Point::new(Px(40.0), Px(10.0)),
            Point::new(Px(60.0), Px(10.0)),
        ] {
            let did = super::node_drag::handle_node_drag_move(
                &mut canvas,
                &mut cx,
                &snapshot,
                pos,
                fret_core::Modifiers::default(),
                snapshot.zoom,
            );
            assert!(did);
            assert_eq!(canvas.history.undo_len(), 0);
        }

        let did_up = super::pointer_up::handle_pointer_up(
            &mut canvas,
            &mut cx,
            &snapshot,
            Point::new(Px(60.0), Px(10.0)),
            fret_core::MouseButton::Left,
            1,
            fret_core::Modifiers::default(),
            snapshot.zoom,
        );
        assert!(did_up);
        assert_eq!(canvas.history.undo_len(), 1);

        assert!(canvas.undo_last(&mut host, None));
        assert_eq!(
            read_node_pos(&mut host, &graph, a),
            CanvasPoint { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            read_node_pos(&mut host, &graph, b),
            CanvasPoint { x: 10.0, y: 0.0 }
        );
    }

    #[test]
    fn connect_bundle_records_single_history_entry() {
        let mut host = TestUiHostImpl::default();
        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let n1 = NodeId::new();
        let out1 = PortId::new();
        let out2 = PortId::new();
        graph.nodes.insert(
            n1,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![out1, out2],
                data: Value::Null,
            },
        );
        for (id, key) in [(out1, "out1"), (out2, "out2")] {
            graph.ports.insert(
                id,
                Port {
                    node: n1,
                    key: PortKey::new(key),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: None,
                    data: Value::Null,
                },
            );
        }

        let n2 = NodeId::new();
        let inn = PortId::new();
        graph.nodes.insert(
            n2,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 100.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: vec![inn],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            inn,
            Port {
                node: n2,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let graph_model = host.models.insert(graph);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph_model.clone(), view);
        let snapshot: ViewSnapshot = canvas.sync_view_state(&mut host);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut cx = event_cx(&mut host, &mut services, bounds);

        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: out1,
                bundle: vec![out1, out2],
            },
            pos: Point::new(Px(0.0), Px(0.0)),
        });

        let did = super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(inn),
        );
        assert!(did);
        assert_eq!(canvas.history.undo_len(), 1);
        let edges_len = graph_model
            .read_ref(&mut host, |g| g.edges.len())
            .unwrap_or(0);
        assert_eq!(edges_len, 2);

        assert!(canvas.undo_last(&mut host, None));
        let edges_len = graph_model
            .read_ref(&mut host, |g| g.edges.len())
            .unwrap_or(0);
        assert_eq!(edges_len, 0);
    }

    #[test]
    fn nudge_moves_selection_and_records_history_entry() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, a).x, 1.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 11.0);

        assert!(canvas.undo_last(&mut host, None));
        assert_eq!(
            read_node_pos(&mut host, &graph, a),
            CanvasPoint { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            read_node_pos(&mut host, &graph, b),
            CanvasPoint { x: 10.0, y: 0.0 }
        );
    }

    #[test]
    fn nudge_multi_selection_respects_node_extent_by_selection_bounds() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
            s.interaction.node_extent = Some(CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 50.0,
                    height: 100.0,
                },
            });
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
        assert_eq!(canvas.history.undo_len(), 0);

        assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 10.0);
    }

    #[test]
    fn nudge_respects_per_node_extent_rect() {
        let mut host = TestUiHostImpl::default();

        let mut graph_value = Graph::new(GraphId::new());
        let node_id = NodeId::new();
        graph_value.nodes.insert(
            node_id,
            Node {
                kind: NodeKindKey::new("test.node"),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: Some(crate::core::NodeExtent::Rect {
                    rect: CanvasRect {
                        origin: CanvasPoint { x: 0.0, y: 0.0 },
                        size: CanvasSize {
                            width: 45.0,
                            height: 100.0,
                        },
                    },
                }),
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![node_id];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT_FAST)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, node_id).x, 5.0);
    }

    #[test]
    fn select_all_selects_nodes_groups_and_edges_and_respects_edge_selectable() {
        let mut host = TestUiHostImpl::default();

        let mut graph = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();
        graph.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 0.0 },
                selectable: Some(false),
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let p_out = PortId::new();
        let p_in = PortId::new();
        graph.ports.insert(
            p_out,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            p_in,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.nodes.get_mut(&a).unwrap().ports.push(p_out);
        graph.nodes.get_mut(&b).unwrap().ports.push(p_in);

        let e_ok = EdgeId::new();
        let e_no = EdgeId::new();
        graph.edges.insert(
            e_ok,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            e_no,
            Edge {
                kind: EdgeKind::Data,
                from: p_out,
                to: p_in,
                selectable: Some(false),
                deletable: None,
                reconnectable: None,
            },
        );

        let g0 = GroupId::new();
        graph.groups.insert(
            g0,
            Group {
                title: "Group".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 80.0,
                    },
                },
                color: None,
            },
        );

        let graph = host.models.insert(graph);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.interaction.elements_selectable = true;
            s.interaction.edges_selectable = true;
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        canvas.interaction.focused_edge = Some(e_ok);
        canvas.interaction.focused_node = Some(a);
        canvas.interaction.focused_port = Some(p_out);
        canvas.interaction.focused_port_valid = true;
        canvas.interaction.focused_port_convertible = true;

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_SELECT_ALL)));

        assert!(canvas.interaction.focused_edge.is_none());
        assert!(canvas.interaction.focused_node.is_none());
        assert!(canvas.interaction.focused_port.is_none());
        assert!(!canvas.interaction.focused_port_valid);
        assert!(!canvas.interaction.focused_port_convertible);

        let mut selected_nodes = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        selected_nodes.sort();
        assert_eq!(selected_nodes, vec![a]);

        let mut selected_groups = view
            .read_ref(&host, |s| s.selected_groups.clone())
            .unwrap_or_default();
        selected_groups.sort();
        assert_eq!(selected_groups, vec![g0]);

        let mut selected_edges = view
            .read_ref(&host, |s| s.selected_edges.clone())
            .unwrap_or_default();
        selected_edges.sort();
        assert_eq!(selected_edges, vec![e_ok]);
    }

    #[test]
    fn delete_selection_respects_node_deletable_and_keeps_undeletable_selected() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, a, b) = make_test_graph_two_nodes();
        graph_value
            .nodes
            .get_mut(&a)
            .expect("node must exist")
            .deletable = Some(false);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.interaction.nodes_deletable = true;
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION)));

        assert!(
            graph
                .read_ref(&mut host, |g| g.nodes.contains_key(&a))
                .unwrap_or(false)
        );
        assert!(
            !graph
                .read_ref(&mut host, |g| g.nodes.contains_key(&b))
                .unwrap_or(true)
        );

        let selected_nodes = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected_nodes, vec![a]);
        assert_eq!(canvas.history.undo_len(), 1);
    }

    #[test]
    fn delete_selection_respects_edge_deletable_and_keeps_undeletable_selected() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();

        let edge = EdgeId::new();
        graph_value.edges.insert(
            edge,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: Some(false),
                reconnectable: None,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes.clear();
            s.selected_groups.clear();
            s.selected_edges = vec![edge];
            s.interaction.edges_deletable = true;
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION)));

        assert_eq!(graph.read_ref(&mut host, |g| g.edges.len()).unwrap_or(0), 1);
        let selected_edges = view
            .read_ref(&host, |s| s.selected_edges.clone())
            .unwrap_or_default();
        assert_eq!(selected_edges, vec![edge]);
        assert_eq!(canvas.history.undo_len(), 0);
    }

    #[test]
    fn align_left_moves_selected_nodes_and_records_history_entry() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_LEFT)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 0.0);

        assert!(canvas.undo_last(&mut host, None));
        assert_eq!(
            read_node_pos(&mut host, &graph, a),
            CanvasPoint { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            read_node_pos(&mut host, &graph, b),
            CanvasPoint { x: 10.0, y: 5.0 }
        );
    }

    #[test]
    fn align_right_respects_per_node_extent_rect() {
        let mut host = TestUiHostImpl::default();

        let mut graph_value = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();
        graph_value.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: Some(crate::core::NodeExtent::Rect {
                    rect: CanvasRect {
                        origin: CanvasPoint { x: 0.0, y: 0.0 },
                        size: CanvasSize {
                            width: 40.0,
                            height: 100.0,
                        },
                    },
                }),
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 20.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_RIGHT)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, a).x, 30.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 20.0);
    }

    #[test]
    fn align_center_x_preserves_alignment_under_node_extent_bounds() {
        let mut host = TestUiHostImpl::default();

        let mut graph_value = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();
        graph_value.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 90.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            b,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 150.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b];
            s.interaction.node_extent = Some(CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            });
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_CENTER_X)));
        assert_eq!(canvas.history.undo_len(), 1);

        let pos_a = read_node_pos(&mut host, &graph, a);
        let pos_b = read_node_pos(&mut host, &graph, b);
        assert_eq!(pos_a.x, 75.0);
        assert_eq!(pos_b.x, 60.0);

        let center_a = pos_a.x + 5.0;
        let center_b = pos_b.x + 20.0;
        assert!((center_a - center_b).abs() <= 1.0e-6);
    }

    #[test]
    fn distribute_x_clamps_nodes_to_node_extent_rect_like_xyflow() {
        let mut host = TestUiHostImpl::default();

        let mut graph_value = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let a = NodeId::new();
        let b = NodeId::new();
        let c = NodeId::new();
        let d = NodeId::new();

        graph_value.nodes.insert(
            a,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            b,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 10.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            c,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 60.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 80.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            d,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 90.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a, b, c, d];
            s.interaction.node_extent = Some(CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            });
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X)));
        assert_eq!(canvas.history.undo_len(), 1);

        assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
        assert_eq!(read_node_pos(&mut host, &graph, b).x, 30.0);

        // Desired position would be x=25, but node extent clamps to max_x=20 for a 80px-wide node.
        assert_eq!(read_node_pos(&mut host, &graph, c).x, 20.0);
        assert_eq!(read_node_pos(&mut host, &graph, d).x, 90.0);
    }

    #[test]
    fn distribute_x_clamps_selected_group_children_to_node_extent_rect_like_xyflow() {
        let mut host = TestUiHostImpl::default();

        let mut graph_value = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let left = NodeId::new();
        let right = NodeId::new();
        let child = NodeId::new();

        let group_id = GroupId::new();
        graph_value.groups.insert(
            group_id,
            Group {
                title: "G".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 10.0, y: 0.0 },
                    size: CanvasSize {
                        width: 20.0,
                        height: 20.0,
                    },
                },
                color: None,
            },
        );

        graph_value.nodes.insert(
            left,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            right,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 90.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        graph_value.nodes.insert(
            child,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 50.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: Some(group_id),
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![left, right];
            s.selected_groups = vec![group_id];
            s.interaction.node_extent = Some(CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            });
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X)));
        assert_eq!(canvas.history.undo_len(), 1);

        // Left/right are the endpoints and remain fixed; the group is the interior element.
        assert_eq!(read_node_pos(&mut host, &graph, left).x, 0.0);
        assert_eq!(read_node_pos(&mut host, &graph, right).x, 90.0);

        // The group's desired shift would move the child to x=80. Node extent clamps to max_x=60.
        assert_eq!(read_node_pos(&mut host, &graph, child).x, 60.0);
        let group_origin_x = graph
            .read_ref(&mut host, |g| {
                g.groups.get(&group_id).map(|gr| gr.rect.origin.x)
            })
            .ok()
            .flatten()
            .unwrap_or_default();
        assert_eq!(group_origin_x, 20.0);
    }

    #[test]
    fn distribute_x_clamps_selected_group_children_to_node_extent_rect_from_node_extents() {
        let mut host = TestUiHostImpl::default();

        let mut graph_value = Graph::new(GraphId::new());
        let kind = NodeKindKey::new("test.node");

        let left = NodeId::new();
        let right = NodeId::new();
        let child = NodeId::new();

        let group_id = GroupId::new();
        graph_value.groups.insert(
            group_id,
            Group {
                title: "G".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 10.0, y: 0.0 },
                    size: CanvasSize {
                        width: 20.0,
                        height: 20.0,
                    },
                },
                color: None,
            },
        );

        graph_value.nodes.insert(
            left,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );
        graph_value.nodes.insert(
            right,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint { x: 90.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 10.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        graph_value.nodes.insert(
            child,
            Node {
                kind,
                kind_version: 1,
                pos: CanvasPoint { x: 50.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: Some(group_id),
                extent: Some(crate::core::NodeExtent::Rect {
                    rect: CanvasRect {
                        origin: CanvasPoint { x: 40.0, y: 0.0 },
                        size: CanvasSize {
                            width: 60.0,
                            height: 100.0,
                        },
                    },
                }),
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 40.0,
                    height: 20.0,
                }),
                collapsed: false,
                ports: Vec::new(),
                data: Value::Null,
            },
        );

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![left, right];
            s.selected_groups = vec![group_id];
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X)));
        assert_eq!(canvas.history.undo_len(), 1);
        assert_eq!(read_node_pos(&mut host, &graph, child).x, 60.0);

        let group_origin_x = graph
            .read_ref(&mut host, |g| {
                g.groups.get(&group_id).map(|gr| gr.rect.origin.x)
            })
            .ok()
            .flatten()
            .unwrap_or_default();
        assert_eq!(group_origin_x, 20.0);
    }

    #[test]
    fn focus_next_cycles_nodes_and_updates_selection() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, b) = make_test_graph_two_nodes();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.draw_order = vec![a, b];
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
        }
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![a]);

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
        }
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![b]);

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV)));
        }
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![a]);
    }

    #[test]
    fn focus_next_skips_unselectable_nodes() {
        let mut host = TestUiHostImpl::default();
        let (mut graph_value, a, b) = make_test_graph_two_nodes();
        graph_value
            .nodes
            .get_mut(&a)
            .expect("node exists")
            .selectable = Some(false);

        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.interaction.elements_selectable = true;
            s.draw_order = vec![a, b];
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
        let selected = view
            .read_ref(&host, |s| s.selected_nodes.clone())
            .unwrap_or_default();
        assert_eq!(selected, vec![b]);
    }

    #[test]
    fn focus_next_port_cycles_ports_within_focused_node() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a];
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
        }
        assert_eq!(canvas.interaction.focused_port, Some(a_in));

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
        }
        assert_eq!(canvas.interaction.focused_port, Some(a_out));

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV_PORT)));
        }
        assert_eq!(canvas.interaction.focused_port, Some(a_in));
    }

    #[test]
    fn focus_next_port_filters_by_wire_direction() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        view.update(&mut host, |s, _cx| {
            s.selected_nodes = vec![a];
            s.selected_edges.clear();
            s.selected_groups.clear();
        })
        .unwrap();

        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: Vec::new(),
            },
            pos: Point::new(Px(0.0), Px(0.0)),
        });

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
        assert_eq!(canvas.interaction.focused_port, Some(a_in));
    }

    #[test]
    fn activate_starts_and_commits_wire_drag() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
        let graph_model = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph_model.clone(), view);
        canvas.sync_view_state(&mut host);

        canvas.interaction.focused_port = Some(a_out);

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ACTIVATE)));
        }
        assert!(canvas.interaction.wire_drag.is_some());
        assert!(canvas.interaction.click_connect);
        assert!(canvas.interaction.focused_port.is_none());

        canvas.interaction.focused_port = Some(b_in);
        {
            let mut cx = command_cx(&mut host, &mut services, &mut tree);
            assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ACTIVATE)));
        }
        assert!(canvas.interaction.wire_drag.is_none());

        let edges_len = graph_model
            .read_ref(&mut host, |g| g.edges.len())
            .unwrap_or(0);
        assert_eq!(edges_len, 1);
    }

    #[test]
    fn focus_port_right_moves_to_neighbor_node() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _a_in, a_out, b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(500.0);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        canvas.interaction.focused_port = Some(a_out);
        canvas.interaction.focused_node = None;

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PORT_RIGHT)));
        assert_eq!(canvas.interaction.focused_node, Some(b));
        assert_eq!(canvas.interaction.focused_port, Some(b_in));
    }

    #[test]
    fn focus_port_left_moves_back() {
        let mut host = TestUiHostImpl::default();
        let (graph_value, a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(500.0);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(crate::io::NodeGraphViewState::default());

        let mut canvas = NodeGraphCanvas::new(graph, view.clone());
        canvas.sync_view_state(&mut host);

        canvas.interaction.focused_port = Some(b_in);

        let mut services = NullServices::default();
        let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
        let mut cx = command_cx(&mut host, &mut services, &mut tree);

        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PORT_LEFT)));
        assert_eq!(canvas.interaction.focused_node, Some(a));
        assert_eq!(canvas.interaction.focused_port, Some(a_out));
    }
}
