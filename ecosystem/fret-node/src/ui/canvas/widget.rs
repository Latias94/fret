use fret_core::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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
use crate::ui::edit_queue::NodeGraphEditQueue;
use crate::ui::presenter::{
    DefaultNodeGraphPresenter, EdgeRenderHint, EdgeRouteKind, InsertNodeCandidate,
    NodeGraphContextMenuAction, NodeGraphContextMenuItem, NodeGraphPresenter, NodeResizeHandleSet,
    PortAnchorHint,
};
use crate::ui::style::{NodeGraphBackgroundStyle, NodeGraphColorMode, NodeGraphStyle};
use crate::ui::view_queue::{NodeGraphFitViewOptions, NodeGraphViewQueue};
use crate::ui::{
    FallbackMeasuredNodeGraphPresenter, GroupRenameOverlay, MeasuredGeometryStore,
    NodeGraphCanvasTransform, NodeGraphController, NodeGraphEdgeTypes,
    NodeGraphGeometryOverridesRef, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
    NodeGraphOverlayState, NodeGraphPaintOverridesRef, NodeGraphSkinRef,
};

use super::middleware::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasEventOutcome,
    NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareCx, NoopNodeGraphCanvasMiddleware,
};
use super::paint::CanvasPaintCache;
use super::state::ViewportMoveDebounceState;

mod apply;
mod auto_measure;
mod auto_measure_apply;
mod auto_measure_collect;
mod callbacks;
mod callbacks_connect;
mod callbacks_graph;
mod callbacks_view;
mod cancel;
mod cancel_cleanup;
mod cancel_gesture_state;
mod cancel_session;
mod cancel_viewport_state;
mod clipboard;
mod clipboard_anchor;
mod clipboard_paste;
mod clipboard_paste_parse;
mod clipboard_paste_selection;
mod clipboard_paste_transaction;
mod clipboard_transfer;
mod command_edit;
mod command_edit_remove;
mod command_focus;
mod command_focus_cycle;
mod command_focus_port;
mod command_history;
mod command_mode;
mod command_move;
mod command_open;
mod command_open_conversion;
mod command_open_edge;
mod command_open_group;
mod command_open_insert;
mod command_router;
mod command_router_align;
mod command_router_nudge;
mod command_selection;
mod command_ui;
mod command_view;
mod commit;
mod commit_legacy;
mod context_menu;
mod cursor;
mod cursor_gate;
mod cursor_resolve;
mod delete;
mod delete_ops_builder;
mod delete_predicates;
mod delete_removed_ids;
mod derived_geometry;
mod edge_drag;
mod edge_insert;
mod edge_insert_drag;
mod edge_path_ctx;
mod event_clipboard;
mod event_clipboard_feedback;
mod event_clipboard_pending;
mod event_keyboard;
mod event_keyboard_route;
mod event_keyboard_state;
mod event_pointer_down;
mod event_pointer_down_route;
mod event_pointer_down_state;
mod event_pointer_move;
mod event_pointer_move_state;
mod event_pointer_move_tail;
mod event_pointer_up;
mod event_pointer_wheel;
mod event_pointer_wheel_route;
mod event_pointer_wheel_state;
mod event_router;
mod event_router_pointer;
mod event_router_pointer_button;
mod event_router_pointer_wheel;
mod event_router_system;
mod event_router_system_input;
mod event_router_system_lifecycle;
mod event_timer;
mod event_timer_route;
mod event_timer_toast;
mod focus;
mod focus_draw_order;
mod focus_edge_repair;
mod focus_nav;
mod focus_nav_ports;
mod focus_nav_ports_activation;
mod focus_nav_ports_center;
mod focus_nav_ports_hints;
mod focus_nav_traversal;
mod focus_nav_traversal_edge;
mod focus_nav_traversal_node;
mod focus_nav_traversal_port;
mod focus_port_direction;
mod focus_port_direction_apply;
mod focus_port_direction_candidate;
mod focus_port_direction_rank;
mod focus_port_direction_wire;
mod focus_session;
mod graph_construction;
mod group_drag;
mod group_draw_order;
mod group_resize;
mod group_resize_apply;
mod group_resize_hit;
mod hit_test;
mod pointer_down_double_click;
mod pointer_down_double_click_background;
mod pointer_down_double_click_edge;
mod pointer_down_gesture_start;
mod pointer_move_dispatch;
mod pointer_move_pointer_state;
mod pointer_move_release;
mod pointer_move_release_left;
mod pointer_move_release_pan;
mod pointer_up_session;
mod pointer_wheel_motion;
mod pointer_wheel_pan;
mod pointer_wheel_viewport;
mod pointer_wheel_zoom;
mod press_session;
use hit_test::{HitTestCtx, HitTestScratch};
mod hover;
mod insert_candidates;
mod insert_execution;
mod insert_execution_feedback;
mod insert_execution_plan;
mod insert_execution_point;
mod insert_node_drag;
mod interaction_gate;
mod interaction_policy;
mod keyboard_pan_activation;
mod keyboard_shortcuts;
mod keyboard_shortcuts_commands;
mod keyboard_shortcuts_gate;
mod keyboard_shortcuts_map;
mod keyboard_shortcuts_overlay;
mod left_click;
mod marquee;
mod marquee_begin;
mod marquee_finish;
mod marquee_pending;
mod marquee_selection;
mod marquee_selection_apply;
mod marquee_selection_query;
mod menu_session;
mod move_ops;
mod node_drag;
mod node_drag_constraints;
mod node_drag_constraints_anchor;
mod node_drag_constraints_extent;
mod node_drag_preview;
mod node_drag_snap;
mod node_layout;
mod node_resize;
mod overlay_hit;
mod overlay_layout;
mod paint_edge_anchors;
mod paint_edges;
mod paint_grid;
mod paint_grid_cache;
mod paint_grid_plan;
mod paint_grid_plan_support;
mod paint_grid_stats;
mod paint_grid_tiles;
mod paint_grid_tiles_cross;
mod paint_grid_tiles_dots;
mod paint_grid_tiles_lines;
mod paint_groups;
mod paint_nodes;
mod paint_overlay_elements;
mod paint_overlay_feedback;
mod paint_overlay_guides;
mod paint_overlay_menu;
mod paint_overlay_toast;
mod paint_overlay_wire_hint;
mod paint_overlays;
mod paint_render_data;
mod paint_root;
mod paint_root_helpers;
mod paint_searcher;
mod paint_searcher_query;
mod paint_searcher_rows;
mod pan_zoom;
mod pan_zoom_begin;
mod pan_zoom_move;
mod pan_zoom_zoom;
mod pending_connection_session;
mod pending_drag;
mod pending_drag_session;
mod pending_group_drag;
mod pending_group_resize;
mod pending_resize;
mod pending_resize_session;
mod pending_wire_drag;
mod pointer_up;
mod pointer_up_commit;
mod pointer_up_commit_group_drag;
mod pointer_up_commit_resize;
mod pointer_up_finish;
mod pointer_up_left_route;
mod pointer_up_node_drag;
mod pointer_up_node_drag_ops;
mod pointer_up_node_drag_parent;
mod pointer_up_pending;
mod pointer_up_state;
mod preview;
mod reconnect;
mod rect_math;
mod rect_math_core;
mod rect_math_path;
mod retained_widget;
mod retained_widget_command_availability;
mod retained_widget_command_availability_gate;
mod retained_widget_command_availability_query;
mod retained_widget_cull_window;
mod retained_widget_cull_window_key;
mod retained_widget_cull_window_shift;
mod retained_widget_frame;
mod retained_widget_layout;
mod retained_widget_layout_children;
mod retained_widget_layout_drain;
mod retained_widget_layout_observe;
mod retained_widget_layout_publish;
mod retained_widget_runtime;
mod retained_widget_runtime_command;
mod retained_widget_runtime_event;
mod retained_widget_runtime_paint;
mod retained_widget_runtime_shared;
mod retained_widget_semantics;
mod retained_widget_semantics_focus;
mod retained_widget_semantics_value;
mod right_click;
mod searcher;
mod searcher_activation;
mod searcher_activation_hit;
mod searcher_activation_state;
mod searcher_input;
mod searcher_input_nav;
mod searcher_input_query;
mod searcher_logic;
mod searcher_picker;
mod searcher_pointer;
mod searcher_pointer_hover;
mod searcher_pointer_wheel;
mod searcher_row_activation;
mod searcher_rows;
mod searcher_ui;
mod selection;
mod split_edge_execution;
mod sticky_wire;
mod sticky_wire_connect;
mod sticky_wire_targets;
mod stores;
mod threshold;
mod timer_motion;
mod timer_motion_auto_pan;
mod timer_motion_pan_inertia;
mod timer_motion_shared;
mod timer_motion_viewport;
mod toast;
mod view_math;
mod view_math_rect;
mod view_math_viewport;
mod view_state;
mod viewport_timer_animation;
mod viewport_timer_auto_pan;
mod viewport_timer_inertia;
mod viewport_timers;
mod widget_surface;
mod wire_drag;
mod wire_drag_helpers;
mod wire_math;

use edge_path_ctx::EdgePathContext;
use insert_candidates::build_insert_candidate_menu_items;
use insert_execution::is_reroute_insert_candidate;
use menu_session::{build_context_menu_state, build_searcher_rows, build_searcher_state};
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
use super::spatial::CanvasSpatialDerived;
use super::state::{
    ContextMenuState, ContextMenuTarget, DerivedBaseKey, DragPreviewCache, DragPreviewKind,
    GeometryCache, GeometryCacheKey, InteractionState, InternalsCacheKey, InternalsViewKey,
    MarqueeDrag, NodeResizeHandle, PanInertiaState, PasteSeries, PendingPaste, SearcherRowsMode,
    SearcherState, SpatialIndexCacheKey, ToastState, ViewSnapshot, WireDrag, WireDragKind,
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
    skin: Option<NodeGraphSkinRef>,
    skin_last_rev: Option<u64>,
    geometry_overrides: Option<NodeGraphGeometryOverridesRef>,
    paint_overrides: Option<NodeGraphPaintOverridesRef>,
    paint_overrides_last_rev: Option<u64>,
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

    diagnostics_anchor_ports: Option<DiagnosticsAnchorPorts>,

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
struct DiagnosticsAnchorPorts {
    child_offset: usize,
    ports: Vec<PortId>,
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

#[cfg(test)]
mod tests;
