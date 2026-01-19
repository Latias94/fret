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

mod apply;
mod auto_measure;
mod callbacks;
mod cancel;
mod clipboard;
mod command_edit;
mod command_focus;
mod command_history;
mod command_mode;
mod command_move;
mod command_open;
mod command_router;
mod command_selection;
mod command_view;
mod commit;
mod commit_legacy;
mod context_menu;
mod cursor;
mod delete;
mod derived_geometry;
mod edge_drag;
mod edge_insert;
mod edge_insert_drag;
mod event_clipboard;
mod event_keyboard;
mod event_pointer_down;
mod event_pointer_move;
mod event_pointer_up;
mod event_pointer_wheel;
mod event_router;
mod event_timer;
mod focus;
mod focus_nav;
mod graph_construction;
mod group_drag;
mod group_resize;
mod hit_test;
mod hover;
mod insert_node_drag;
mod interaction_policy;
mod left_click;
mod marquee;
mod move_ops;
mod node_drag;
mod node_layout;
mod node_resize;
mod overlay_hit;
mod overlay_layout;
mod paint_edge_anchors;
mod paint_edges;
mod paint_grid;
mod paint_groups;
mod paint_nodes;
mod paint_overlay_elements;
mod paint_overlays;
mod paint_render_data;
mod paint_root;
mod pan_zoom;
mod pending_drag;
mod pending_group_drag;
mod pending_group_resize;
mod pending_resize;
mod pending_wire_drag;
mod pointer_up;
mod preview;
mod reconnect;
mod rect_math;
mod right_click;
mod searcher;
mod searcher_logic;
mod selection;
mod sticky_wire;
mod stores;
mod threshold;
mod toast;
mod view_math;
mod view_state;
mod viewport_timers;
mod wire_drag;
mod wire_drag_helpers;
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
mod tests;
