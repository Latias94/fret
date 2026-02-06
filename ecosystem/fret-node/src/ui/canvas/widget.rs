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
use crate::ui::style::{NodeGraphBackgroundStyle, NodeGraphColorMode, NodeGraphStyle};
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
mod edge_path_ctx;
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
use hit_test::{HitTestCtx, HitTestScratch};
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
mod paint_root_helpers;
mod paint_searcher;
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
mod retained_widget;
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

use edge_path_ctx::EdgePathContext;
use overlay_hit::{
    context_menu_rect_at, context_menu_size_at_zoom, hit_context_menu_item, hit_searcher_row,
    searcher_rect_at, searcher_size_at_zoom, searcher_visible_rows,
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
    ContextMenuState, ContextMenuTarget, DerivedBaseKey, DragPreviewCache, DragPreviewKind,
    GeometryCache, GeometryCacheKey, InteractionState, InternalsCacheKey, InternalsViewKey,
    MarqueeDrag, NodeResizeHandle, PanInertiaState, PasteSeries, PendingPaste, SearcherState,
    SpatialIndexCacheKey, ToastState, ViewSnapshot, WireDrag, WireDragKind,
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
    background_override: Option<NodeGraphBackgroundStyle>,
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
        EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_render_hint(graph, edge_id)
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
        EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_custom_path(graph, edge_id, hint, from, to, zoom)
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
            background_override: None,
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
            background_override: self.background_override,
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
        self.background_override = None;
        self.color_mode = None;
        self.color_mode_last = None;
        self.color_mode_theme_rev = None;
        self.geometry.geom_key = None;
        self.geometry.index_key = None;
        self.geometry.drag_preview = None;
        self
    }

    pub fn background_style(&self) -> NodeGraphBackgroundStyle {
        self.style.background_style()
    }

    pub fn with_background_style(mut self, background: NodeGraphBackgroundStyle) -> Self {
        self.style = self.style.with_background_style(background);
        self.background_override = Some(background);
        // Background theming must not rebuild derived geometry; it is a paint-only concern.
        // Clear the grid cache to avoid retaining tiles for unused background variants.
        self.grid_scene_cache.clear();
        self
    }

    pub fn with_color_mode(mut self, mode: NodeGraphColorMode) -> Self {
        self.color_mode = Some(mode);
        self.color_mode_last = None;
        self.color_mode_theme_rev = None;
        self.geometry.geom_key = None;
        self.geometry.index_key = None;
        self.geometry.drag_preview = None;
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
        if let Some(background) = self.background_override {
            let style = std::mem::take(&mut self.style);
            self.style = style.with_background_style(background);
        }
        self.geometry.geom_key = None;
        self.geometry.index_key = None;
        self.geometry.drag_preview = None;

        if let Some(services) = services {
            self.paint_cache.clear(services);
        }

        self.grid_scene_cache.clear();
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

#[cfg(test)]
mod tests;
