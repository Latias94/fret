use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fret_canvas::budget::{InteractionBudget, WorkBudget};
use fret_canvas::cache::{
    SceneOpTileCache, TileCacheKeyBuilder, TileCoord, TileGrid2D, tile_cache_key,
    warm_scene_op_tiles_u64_with,
};
use fret_canvas::diagnostics::{CanvasCacheKey, CanvasCacheStatsRegistry};
use fret_canvas::scale::{canvas_units_from_screen_px, effective_scale_factor};
use fret_canvas::view::{CanvasViewport2D, PanZoom2D};
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextConstraints, TextOverflow, TextWrap, Transform2D,
};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::{Theme, UiHost, retained_bridge::*};
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
use crate::ui::style::{NodeGraphColorMode, NodeGraphStyle};
use crate::ui::{
    FallbackMeasuredNodeGraphPresenter, GroupRenameOverlay, MeasuredGeometryStore,
    NodeGraphCanvasTransform, NodeGraphEdgeTypes, NodeGraphEditQueue, NodeGraphFitViewOptions,
    NodeGraphInternalsSnapshot, NodeGraphInternalsStore, NodeGraphOverlayState, NodeGraphViewQueue,
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
use rect_math::{
    edge_bounds_rect, inflate_rect, path_bounds_rect, rect_from_points, rect_union, rects_intersect,
};
use wire_math::{
    closest_point_on_edge_route, closest_point_on_path, dist2_point_to_segment,
    path_midpoint_and_normal, path_start_end_tangents, step_wire_distance2, wire_distance2,
    wire_distance2_path,
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
    color_mode: Option<NodeGraphColorMode>,
    color_mode_last: Option<NodeGraphColorMode>,
    color_mode_theme_rev: Option<u64>,
    close_command: Option<CommandId>,

    auto_measured: Arc<MeasuredGeometryStore>,
    auto_measured_key: Option<(u64, u32)>,

    edit_queue: Option<Model<NodeGraphEditQueue>>,
    edit_queue_key: Option<u64>,
    view_queue: Option<Model<NodeGraphViewQueue>>,
    view_queue_key: Option<u64>,
    overlays: Option<Model<NodeGraphOverlayState>>,

    fit_view_on_mount: Option<NodeGraphFitViewOptions>,
    did_fit_view_on_mount: bool,

    measured_output: Option<Arc<MeasuredGeometryStore>>,
    measured_output_key: Option<GeometryCacheKey>,

    internals: Option<Arc<NodeGraphInternalsStore>>,
    internals_key: Option<InternalsCacheKey>,

    cached_pan: CanvasPoint,
    cached_zoom: f32,
    last_cull_window_key: Option<u64>,
    history: GraphHistory,
    geometry: GeometryCache,

    paint_cache: CanvasPaintCache,
    grid_scene_cache: SceneOpTileCache<u64>,
    grid_tiles_scratch: Vec<TileCoord>,
    edges_tiles_scratch: Vec<TileCoord>,
    edges_tile_keys_scratch: Vec<u64>,
    edge_labels_tile_keys_scratch: Vec<u64>,
    groups_scene_cache: SceneOpTileCache<u64>,
    nodes_scene_cache: SceneOpTileCache<u64>,
    edges_scene_cache: SceneOpTileCache<u64>,
    edge_labels_scene_cache: SceneOpTileCache<u64>,
    edges_build_states: HashMap<u64, EdgesBuildState>,
    edge_labels_build_states: HashMap<u64, EdgeLabelsBuildState>,
    edge_labels_build_state: Option<EdgeLabelsBuildState>,
    interaction: InteractionState,
}

#[derive(Debug, Clone)]
struct EdgesBuildState {
    ops: Vec<SceneOp>,
    edges: Vec<paint_render_data::EdgeRender>,
    next_edge: usize,
}

#[derive(Debug, Clone)]
struct EdgeLabelsBuildState {
    key: u64,
    ops: Vec<SceneOp>,
    edges: Vec<paint_render_data::EdgeRender>,
    next_edge: usize,
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
    const EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget = InteractionBudget::new(4, 1);
    const EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(2, 1);
    const EDGE_WIRE_BUILD_BUDGET_PER_FRAME: InteractionBudget = InteractionBudget::new(256, 64);
    const EDGE_MARKER_BUILD_BUDGET_PER_FRAME: InteractionBudget = InteractionBudget::new(96, 24);
    const EDGE_LABEL_BUILD_BUDGET_PER_FRAME: InteractionBudget = InteractionBudget::new(16, 4);
    const STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES: u64 = 60 * 30;
    const STATIC_SCENE_TILE_CACHE_MAX_ENTRIES: usize = 16;

    fn view_interacting(&self) -> bool {
        self.interaction.viewport_move_debounce.is_some()
            || self.interaction.panning
            || self.interaction.pan_inertia.is_some()
            || self.interaction.viewport_animation.is_some()
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

    fn edge_custom_path(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> Option<crate::ui::edge_types::EdgeCustomPath> {
        self.edge_types.as_ref()?.custom_path(
            graph,
            edge_id,
            &self.style,
            hint,
            crate::ui::edge_types::EdgePathInput { from, to, zoom },
        )
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
            color_mode: None,
            color_mode_last: None,
            color_mode_theme_rev: None,
            close_command: None,
            auto_measured,
            auto_measured_key: None,
            edit_queue: None,
            edit_queue_key: None,
            view_queue: None,
            view_queue_key: None,
            overlays: None,
            fit_view_on_mount: None,
            did_fit_view_on_mount: false,
            measured_output: None,
            measured_output_key: None,
            internals: None,
            internals_key: None,
            cached_pan: CanvasPoint::default(),
            cached_zoom: 1.0,
            last_cull_window_key: None,
            history: GraphHistory::default(),
            geometry: GeometryCache::default(),
            paint_cache: CanvasPaintCache::default(),
            grid_scene_cache: SceneOpTileCache::default(),
            grid_tiles_scratch: Vec::new(),
            edges_tiles_scratch: Vec::new(),
            edges_tile_keys_scratch: Vec::new(),
            edge_labels_tile_keys_scratch: Vec::new(),
            groups_scene_cache: SceneOpTileCache::default(),
            nodes_scene_cache: SceneOpTileCache::default(),
            edges_scene_cache: SceneOpTileCache::default(),
            edge_labels_scene_cache: SceneOpTileCache::default(),
            edges_build_states: HashMap::new(),
            edge_labels_build_states: HashMap::new(),
            edge_labels_build_state: None,
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
            color_mode: self.color_mode,
            color_mode_last: self.color_mode_last,
            color_mode_theme_rev: self.color_mode_theme_rev,
            close_command: self.close_command,
            auto_measured: self.auto_measured,
            auto_measured_key: self.auto_measured_key,
            edit_queue: self.edit_queue,
            edit_queue_key: self.edit_queue_key,
            view_queue: self.view_queue,
            view_queue_key: self.view_queue_key,
            overlays: self.overlays,
            fit_view_on_mount: self.fit_view_on_mount,
            did_fit_view_on_mount: self.did_fit_view_on_mount,
            measured_output: self.measured_output,
            measured_output_key: self.measured_output_key,
            internals: self.internals,
            internals_key: self.internals_key,
            cached_pan: self.cached_pan,
            cached_zoom: self.cached_zoom,
            last_cull_window_key: self.last_cull_window_key,
            history: self.history,
            geometry: self.geometry,
            paint_cache: self.paint_cache,
            grid_scene_cache: self.grid_scene_cache,
            grid_tiles_scratch: self.grid_tiles_scratch,
            edges_tiles_scratch: self.edges_tiles_scratch,
            edges_tile_keys_scratch: self.edges_tile_keys_scratch,
            edge_labels_tile_keys_scratch: self.edge_labels_tile_keys_scratch,
            groups_scene_cache: self.groups_scene_cache,
            nodes_scene_cache: self.nodes_scene_cache,
            edges_scene_cache: self.edges_scene_cache,
            edge_labels_scene_cache: self.edge_labels_scene_cache,
            edges_build_states: self.edges_build_states,
            edge_labels_build_states: self.edge_labels_build_states,
            edge_labels_build_state: self.edge_labels_build_state,
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
        self.color_mode = None;
        self.color_mode_last = None;
        self.color_mode_theme_rev = None;
        self.geometry.key = None;
        self
    }

    pub fn with_color_mode(mut self, mode: NodeGraphColorMode) -> Self {
        self.color_mode = Some(mode);
        self.color_mode_last = None;
        self.color_mode_theme_rev = None;
        self.geometry.key = None;
        self
    }

    fn sync_style_from_color_mode(
        &mut self,
        theme: fret_ui::ThemeSnapshot,
        services: Option<&mut dyn fret_core::UiServices>,
    ) {
        let Some(mode) = self.color_mode else {
            return;
        };

        let needs_update = match mode {
            NodeGraphColorMode::System => {
                let rev = theme.revision;
                self.color_mode_last != Some(mode) || self.color_mode_theme_rev != Some(rev)
            }
            NodeGraphColorMode::Light | NodeGraphColorMode::Dark => {
                self.color_mode_last != Some(mode)
            }
        };

        if !needs_update {
            return;
        }

        self.color_mode_last = Some(mode);
        self.color_mode_theme_rev = match mode {
            NodeGraphColorMode::System => Some(theme.revision),
            NodeGraphColorMode::Light | NodeGraphColorMode::Dark => None,
        };

        self.style = NodeGraphStyle::from_snapshot_with_color_mode(theme, mode);
        self.geometry.key = None;

        if let Some(services) = services {
            self.paint_cache.clear(services);
        }

        self.groups_scene_cache.clear();
        self.nodes_scene_cache.clear();
        self.edges_scene_cache.clear();
        self.edge_labels_scene_cache.clear();
        self.edges_build_states.clear();
        self.edge_labels_build_states.clear();
        self.edge_labels_build_state = None;
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

    /// Attaches a UI-side view queue (`Model<NodeGraphViewQueue>`).
    ///
    /// This is a message-passing surface for viewport commands that need arguments (e.g. framing a
    /// specific node set).
    pub fn with_view_queue(mut self, queue: Model<NodeGraphViewQueue>) -> Self {
        self.view_queue = Some(queue);
        self.view_queue_key = None;
        self
    }

    /// Enables a one-shot initial fit-view on mount (XyFlow `fitView` mental model).
    pub fn with_fit_view_on_mount(self) -> Self {
        self.with_fit_view_on_mount_options(NodeGraphFitViewOptions::default())
    }

    /// Enables a one-shot initial fit-view on mount with custom options (XyFlow `fitViewOptions`).
    pub fn with_fit_view_on_mount_options(mut self, options: NodeGraphFitViewOptions) -> Self {
        self.fit_view_on_mount = Some(options);
        self.did_fit_view_on_mount = false;
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

    fn maybe_fit_view_on_mount<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        bounds: Rect,
        did_drain_view_queue: bool,
    ) -> bool {
        if did_drain_view_queue || self.did_fit_view_on_mount {
            return false;
        }

        let Some(options) = self.fit_view_on_mount.clone() else {
            return false;
        };

        let include_hidden_nodes = options.include_hidden_nodes;
        let node_ids: Vec<GraphNodeId> = self
            .graph
            .read_ref(host, |graph| {
                graph
                    .nodes
                    .iter()
                    .filter_map(|(id, node)| {
                        if node.hidden && !include_hidden_nodes {
                            None
                        } else {
                            Some(*id)
                        }
                    })
                    .collect()
            })
            .ok()
            .unwrap_or_default();

        if node_ids.is_empty() {
            return false;
        }

        let did =
            self.frame_nodes_in_view_with_options(host, window, bounds, &node_ids, Some(&options));
        if did {
            self.did_fit_view_on_mount = true;
        }
        did
    }
}

impl<H: UiHost, M: NodeGraphCanvasMiddleware> Widget<H> for NodeGraphCanvasWith<M> {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.paint_cache.clear(services);
        self.groups_scene_cache.clear();
        self.nodes_scene_cache.clear();
        self.edges_scene_cache.clear();
        self.edge_labels_scene_cache.clear();
        self.edges_build_states.clear();
        self.edge_labels_build_states.clear();
        self.edge_labels_build_state = None;
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        let snapshot = self.sync_view_state(cx.app);
        if cx.input_ctx.focus_is_text_input
            && (command.as_str().starts_with("node_graph.")
                || matches!(
                    command.as_str(),
                    "edit.copy" | "edit.cut" | "edit.paste" | "edit.select_all"
                ))
        {
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

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> CommandAvailability {
        if cx.focus != Some(cx.node) {
            return CommandAvailability::NotHandled;
        }

        let clipboard_text = cx.input_ctx.caps.clipboard.text;
        match command.as_str() {
            "edit.copy" | CMD_NODE_GRAPH_COPY => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }

                let has_copyable_selection = self
                    .view_state
                    .read_ref(cx.app, |state| {
                        !state.selected_nodes.is_empty() || !state.selected_groups.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);

                if has_copyable_selection {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            "edit.cut" | CMD_NODE_GRAPH_CUT => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }

                let has_any_selection = self
                    .view_state
                    .read_ref(cx.app, |state| {
                        !state.selected_nodes.is_empty()
                            || !state.selected_edges.is_empty()
                            || !state.selected_groups.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);

                if has_any_selection {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            "edit.paste" | CMD_NODE_GRAPH_PASTE => {
                if !clipboard_text || cx.window.is_none() {
                    return CommandAvailability::Blocked;
                }
                CommandAvailability::Available
            }
            "edit.select_all" | CMD_NODE_GRAPH_SELECT_ALL => {
                let has_any_content = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        !graph.nodes.is_empty() || !graph.groups.is_empty()
                    })
                    .ok()
                    .unwrap_or(false);

                if has_any_content {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            _ => CommandAvailability::NotHandled,
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let theme = Theme::global(&*cx.app).snapshot();
        self.sync_style_from_color_mode(theme, None);
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
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);
        if let Some(queue) = self.edit_queue.as_ref() {
            cx.observe_model(queue, Invalidation::Layout);
        }
        if let Some(queue) = self.view_queue.as_ref() {
            cx.observe_model(queue, Invalidation::Layout);
        }
        for &child in cx.children {
            cx.layout_in(child, cx.bounds);
        }
        self.interaction.last_bounds = Some(cx.bounds);
        self.sync_view_state(cx.app);
        self.drain_edit_queue(cx.app, cx.window);
        self.update_auto_measured_node_sizes(cx);
        let did_view_queue = self.drain_view_queue(cx.app, cx.window);
        let did_fit_on_mount =
            self.maybe_fit_view_on_mount(cx.app, cx.window, cx.bounds, did_view_queue);
        if did_view_queue || did_fit_on_mount {
            cx.request_redraw();
        }
        cx.available
    }

    fn prepaint(&mut self, cx: &mut PrepaintCx<'_, H>) {
        let snapshot = self.sync_view_state(cx.app);
        if !snapshot.interaction.only_render_visible_elements {
            self.last_cull_window_key = None;
            return;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 1.0e-6 {
            return;
        }

        let viewport_max_screen_px = cx.bounds.size.width.0.max(cx.bounds.size.height.0);
        if !viewport_max_screen_px.is_finite() || viewport_max_screen_px <= 0.0 {
            return;
        }

        const STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN: u32 = 1024;
        const STATIC_NODES_TILE_MUL: f32 = 2.0;

        fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
            let target = value.ceil().max(1.0) as u32;
            let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
            pow2.max(min)
        }

        let nodes_tile_size_screen_px = next_power_of_two_at_least(
            STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN,
            viewport_max_screen_px * STATIC_NODES_TILE_MUL,
        );
        let nodes_cache_tile_size_canvas = (nodes_tile_size_screen_px as f32 / zoom).max(1.0);

        let viewport = CanvasViewport2D::new(
            cx.bounds,
            PanZoom2D {
                pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
                zoom,
            },
        );
        let viewport_rect = viewport.visible_canvas_rect();
        let center_x = viewport_rect.origin.x.0 + 0.5 * viewport_rect.size.width.0;
        let center_y = viewport_rect.origin.y.0 + 0.5 * viewport_rect.size.height.0;
        if !center_x.is_finite() || !center_y.is_finite() {
            return;
        }

        let tile_x = (center_x / nodes_cache_tile_size_canvas).floor() as i32;
        let tile_y = (center_y / nodes_cache_tile_size_canvas).floor() as i32;

        let mut b = TileCacheKeyBuilder::new("fret-node.canvas.cull_window.v1");
        b.add_u32(nodes_tile_size_screen_px)
            .add_f32_bits(zoom)
            .add_i32(tile_x)
            .add_i32(tile_y);
        let next_key = b.finish();

        match self.last_cull_window_key {
            None => {
                // Initialize the baseline key without counting it as a "shift".
                self.last_cull_window_key = Some(next_key);
            }
            Some(prev_key) if prev_key != next_key => {
                cx.debug_record_node_graph_cull_window_shift(next_key);
                self.last_cull_window_key = Some(next_key);
            }
            _ => {}
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
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
        let theme = cx.theme().snapshot();
        self.sync_style_from_color_mode(theme, Some(cx.services));
        self.paint_root(cx);
    }
}

#[cfg(test)]
mod tests;
