use std::path::{Path, PathBuf};

const LIB_RS: &str = include_str!("lib.rs");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const APP_RS: &str = include_str!("app.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const UI_BINDING_RS: &str = include_str!("ui/binding.rs");
const UI_BINDING_QUERIES_RS: &str = include_str!("ui/binding_queries.rs");
const UI_BINDING_STORE_SYNC_RS: &str = include_str!("ui/binding_store_sync.rs");
const UI_BINDING_VIEWPORT_RS: &str = include_str!("ui/binding_viewport.rs");
const UI_CANVAS_WIDGET_RS: &str = include_str!("ui/canvas/widget.rs");
const UI_CANVAS_STATE_RS: &str = include_str!("ui/canvas/state.rs");
const UI_CANVAS_STATE_OVERLAY_POLICY_RS: &str =
    include_str!("ui/canvas/state/state_overlay_policy.rs");
const UI_CANVAS_STATE_OVERLAY_SESSIONS_RS: &str =
    include_str!("ui/canvas/state/state_overlay_sessions.rs");
const UI_CANVAS_RS: &str = include_str!("ui/canvas/widget/widget_surface.rs");
const UI_CANVAS_BUILDERS_RS: &str = include_str!("ui/canvas/widget/widget_surface/builders.rs");
const UI_CANVAS_WIDGET_COMMIT_RS: &str = include_str!("ui/canvas/widget/commit/mod.rs");
const UI_CONTROLLER_RS: &str = include_str!("ui/controller.rs");
const UI_CONTROLLER_STORE_SYNC_RS: &str = include_str!("ui/controller_store_sync.rs");
const UI_CONTROLLER_UPDATES_RS: &str = include_str!("ui/controller_updates.rs");
const UI_CONTROLLER_VIEWPORT_RS: &str = include_str!("ui/controller_viewport.rs");
const UI_DECLARATIVE_MOD_RS: &str = include_str!("ui/declarative/mod.rs");
const UI_MOD_RS: &str = include_str!("ui/mod.rs");
const UI_OVERLAYS_MOD_RS: &str = include_str!("ui/overlays/mod.rs");
const UI_OVERLAY_CONTROLS_DECLARATIVE_RS: &str =
    include_str!("ui/overlays/controls_declarative.rs");
const UI_OVERLAY_CONTROLS_HOST_POLICY_RS: &str =
    include_str!("ui/overlays/controls_host_policy.rs");
const UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS: &str =
    include_str!("ui/overlays/controls_interaction_policy.rs");
const UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS: &str =
    include_str!("ui/overlays/blackboard_declarative.rs");
const UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS: &str =
    include_str!("ui/overlays/blackboard_interaction_policy.rs");
const UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS: &str =
    include_str!("ui/overlays/blackboard_paint_plan.rs");
const UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS: &str =
    include_str!("ui/overlays/minimap_interaction_policy.rs");
const UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS: &str =
    include_str!("ui/overlays/toolbar_layout_policy.rs");
const UI_OVERLAY_TOOLBAR_POLICY_RS: &str = include_str!("ui/overlays/toolbar_policy.rs");
const UI_OVERLAY_TOOLBARS_DECLARATIVE_RS: &str =
    include_str!("ui/overlays/toolbars_declarative.rs");
const UI_VIEWPORT_OPTIONS_RS: &str = include_str!("ui/viewport_options.rs");
const UI_CANVAS_WIDGET_MARQUEE_PENDING_RS: &str =
    include_str!("ui/canvas/widget/marquee_pending.rs");
const UI_CANVAS_WIDGET_MARQUEE_SELECTION_RS: &str =
    include_str!("ui/canvas/widget/marquee_selection.rs");
const UI_CANVAS_WIDGET_PAN_ZOOM_BEGIN_CX_RS: &str =
    include_str!("ui/canvas/widget/pan_zoom_begin_cx.rs");
const UI_CANVAS_WIDGET_PAINT_INVALIDATION_RS: &str =
    include_str!("ui/canvas/widget/paint_invalidation.rs");
const UI_CANVAS_WIDGET_PREPAINT_CULL_WINDOW_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/prepaint_cull_window_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_CACHE_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_cache.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_CACHE_WARM_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_cache/warm.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_CACHE_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_cache_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_CACHE_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_cache_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_DIAGNOSTICS_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_diagnostics_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_DIAGNOSTICS_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_diagnostics_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_GRID_STATS_RS: &str =
    include_str!("ui/canvas/widget/paint_grid_stats.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHE_PLAN_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cache_plan.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHE_PLAN_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cache_plan_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHE_PLAN_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cache_plan_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_GROUPS_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_groups.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_NODES_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_nodes.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_ANCHOR_TARGET_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/anchor_target.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_ANCHOR_TARGET_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/anchor_target_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_ANCHOR_TARGET_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/anchor_target_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_FALLBACK_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/fallback.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_FALLBACK_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/fallback_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_FALLBACK_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/fallback_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_EDGES_FALLBACK_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/edges/fallback.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_KEYS_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/keys.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_REPLAY_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/replay_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_REPLAY_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/replay_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_REPLAY_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/edges/replay.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGE_LABELS_REPLAY_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/labels/replay.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_BUILD_STATE_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/build_state_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_BUILD_STATE_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/build_state_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_BUILD_STATE_STEP_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/build_state/step.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_BUILD_STATE_TEMP_SCENE_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/build_state/temp_scene.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_BUILD_STATE_OPS_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/build_state/ops.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_BUILD_STATE_CLIP_OPS_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/build_state/clip_ops.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_SINGLE_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/edges/single.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_TILED_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGE_LABELS_BUILD_STATE_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/label_build_state_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGE_LABELS_BUILD_STATE_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/label_build_state_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGE_LABELS_SINGLE_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/labels/single.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGE_LABELS_TILED_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_OVERLAY_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/overlay_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_OVERLAY_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/overlay_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_SINGLE_RECT_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/single_rect.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_EDGES_TILE_PATH_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_edges/tile_path.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_STATIC_SCENE_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_static_scene_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_STATIC_SCENE_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_static_scene_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_CACHE_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame/cache.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_BACKGROUND_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame/background.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_BACKGROUND_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_background_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_BACKGROUND_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_background_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_CLIP_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_clip_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_CLIP_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_clip_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_DIAGNOSTICS_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_diagnostics_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_DIAGNOSTICS_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_diagnostics_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_VIEWPORT_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_viewport_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_FRAME_VIEWPORT_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/frame_viewport_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_IMMEDIATE_PASS_RS: &str =
    include_str!("ui/canvas/widget/paint_root/immediate_pass.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_CACHED_PASS_RS: &str =
    include_str!("ui/canvas/widget/paint_root/cached_pass.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_PASS_SCENE_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/pass_scene_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_PASS_SCENE_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/pass_scene_retained_cx.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_STATIC_CACHE_RS: &str =
    include_str!("ui/canvas/widget/paint_root/static_cache.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_STATIC_LAYER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/static_layer.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_TAIL_RS: &str =
    include_str!("ui/canvas/widget/paint_root/tail.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_TAIL_CLEANUP_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/paint_root/tail_cleanup_adapter.rs");
const UI_CANVAS_WIDGET_PAINT_ROOT_TAIL_CLEANUP_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/paint_root/tail_cleanup_retained_cx.rs");
const UI_CANVAS_RETAINED_COMMAND_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/retained_command_adapter.rs");
const UI_CANVAS_RETAINED_LOW_LEVEL_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/retained_low_level_adapter.rs");
const UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS: &str =
    include_str!("ui/canvas/widget/wire_drag/commit_cx.rs");
const UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/wire_drag_move_cx.rs");
const UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_UPDATE_RS: &str =
    include_str!("ui/canvas/widget/wire_drag/move_update/mod.rs");
const UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_UPDATE_AUTO_PAN_RS: &str =
    include_str!("ui/canvas/widget/wire_drag/move_update/auto_pan.rs");
const UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_UPDATE_PRELUDE_RS: &str =
    include_str!("ui/canvas/widget/wire_drag/move_update/prelude.rs");
const UI_CANVAS_WIDGET_POINTER_UP_FINISH_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_finish.rs");
const UI_CANVAS_WIDGET_POINTER_UP_SESSION_CLEANUP_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_session/cleanup.rs");
const UI_CANVAS_WIDGET_POINTER_UP_SESSION_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_session/release.rs");
const UI_CANVAS_WIDGET_POINTER_UP_PENDING_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_pending/release.rs");
const UI_CANVAS_WIDGET_POINTER_UP_PENDING_RELEASE_GROUP_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_pending/release/group.rs");
const UI_CANVAS_WIDGET_POINTER_UP_PENDING_RELEASE_NODE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_pending/release/node.rs");
const UI_CANVAS_WIDGET_POINTER_UP_PENDING_WIRE_DRAG_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_pending/wire_drag.rs");
const UI_CANVAS_WIDGET_POINTER_UP_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up/release.rs");
const UI_CANVAS_WIDGET_POINTER_UP_STATE_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_state/release.rs");
const UI_CANVAS_WIDGET_POINTER_UP_RELEASE_CX_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_release_cx.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_RS: &str =
    include_str!("ui/canvas/widget/pointer_up/left.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_left_route.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_left_route/dispatch.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DOUBLE_CLICK_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_left_route/double_click.rs");
const UI_CANVAS_WIDGET_POINTER_DOWN_CLOSE_BUTTON_CX_RS: &str =
    include_str!("ui/canvas/widget/pointer_down_close_button_cx.rs");
const UI_CANVAS_WIDGET_POINTER_DOWN_GESTURE_START_CLOSE_BUTTON_RS: &str =
    include_str!("ui/canvas/widget/pointer_down_gesture_start/close_button.rs");
const UI_CANVAS_WIDGET_CANCEL_VIEWPORT_STATE_RS: &str =
    include_str!("ui/canvas/widget/cancel_viewport_state.rs");
const UI_CANVAS_WIDGET_POINTER_UP_COMMIT_CX_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_commit_cx.rs");
const UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_commit.rs");
const UI_CANVAS_WIDGET_POINTER_UP_NODE_DRAG_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_node_drag.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_COMMIT_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_PENDING_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs");
const UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_ACTIVE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_left_route/dispatch/active.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_POINTER_UP_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/pointer_up.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_POINTER_UP_ACTIVE_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/pointer_up/active.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_POINTER_UP_PENDING_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs");
const UI_CANVAS_WIDGET_EDGE_DRAG_MOVE_START_RS: &str =
    include_str!("ui/canvas/widget/edge_drag/move_start.rs");
const UI_CANVAS_WIDGET_EDGE_DRAG_PRELUDE_RS: &str =
    include_str!("ui/canvas/widget/edge_drag/prelude.rs");
const UI_CANVAS_WIDGET_EDGE_DRAG_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/edge_drag_move_cx.rs");
const UI_CANVAS_WIDGET_EDGE_DRAG_POINTER_UP_RS: &str =
    include_str!("ui/canvas/widget/edge_drag/pointer_up.rs");
const UI_CANVAS_WIDGET_POINTER_UP_COMMIT_GROUP_DRAG_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_commit/group_drag.rs");
const UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RESIZE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_commit/resize.rs");
const UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RESIZE_GROUP_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_commit/resize/group.rs");
const UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RESIZE_NODE_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_commit/resize/node.rs");
const UI_CANVAS_WIDGET_STICKY_WIRE_CONNECT_RS: &str =
    include_str!("ui/canvas/widget/sticky_wire_connect.rs");
const UI_CANVAS_WIDGET_WIRE_DRAG_HELPERS_RS: &str =
    include_str!("ui/canvas/widget/wire_drag_helpers.rs");
const UI_CANVAS_WIDGET_STICKY_WIRE_CONNECT_FINISH_RS: &str =
    include_str!("ui/canvas/widget/sticky_wire_connect/finish.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_DRAG_TAIL_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/drag/tail.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_DRAG_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/drag.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_PENDING_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/pending.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_PENDING_ACTIVATE_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/pending/activate.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_PRELUDE_RS: &str =
    include_str!("ui/canvas/widget/edge_insert_drag/prelude.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_PRELUDE_RS: &str =
    include_str!("ui/canvas/widget/edge_insert/prelude.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_INSERT_RS: &str =
    include_str!("ui/canvas/widget/edge_insert/insert.rs");
const UI_CANVAS_WIDGET_EDGE_INSERT_CONTEXT_MENU_RS: &str =
    include_str!("ui/canvas/widget/edge_insert/context_menu.rs");
const UI_CANVAS_WIDGET_INSERT_NODE_DRAG_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/mod.rs");
const UI_CANVAS_WIDGET_INSERT_NODE_DRAG_PENDING_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/pending.rs");
const UI_CANVAS_WIDGET_INSERT_NODE_DRAG_PRELUDE_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/prelude.rs");
const UI_CANVAS_WIDGET_INSERT_NODE_DRAG_SESSION_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/session.rs");
const UI_CANVAS_WIDGET_INSERT_NODE_DRAG_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag_move_cx.rs");
const UI_CANVAS_WIDGET_EVENT_CLIPBOARD_FEEDBACK_RS: &str =
    include_str!("ui/canvas/widget/event_clipboard_feedback.rs");
const UI_CANVAS_WIDGET_EVENT_CLIPBOARD_FEEDBACK_CX_RS: &str =
    include_str!("ui/canvas/widget/event_clipboard_feedback_cx.rs");
const UI_CANVAS_WIDGET_EVENT_TIMER_TOAST_RS: &str =
    include_str!("ui/canvas/widget/event_timer_toast.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_UP_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_up.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_UP_DISPATCH_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_up/dispatch.rs");
const UI_CANVAS_WIDGET_EVENT_RUNTIME_ADAPTER_RS: &str =
    include_str!("ui/canvas/widget/event_runtime_adapter.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_UP_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer_button/up.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_MOVE_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer_button/move_event.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_CX_RS: &str =
    include_str!("ui/canvas/widget/event_router_cx.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer_button.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_DOWN_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer_button/down.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_move.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_move/release.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_ROUTE_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_move/tail.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_CURSOR_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_move_tail/cursor.rs");
const UI_CANVAS_WIDGET_AUTO_PAN_TIMER_CX_RS: &str =
    include_str!("ui/canvas/widget/auto_pan_timer_cx.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_move_tail.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_TAIL_CX_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_tail_cx.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_TIMER_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_move_tail/timer.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_release.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_PAN_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_release_pan.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_PAN_MISSING_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_release_pan/missing_release.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_PAN_PENDING_RIGHT_CLICK_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_LEFT_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_release_left.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_WHEEL_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_wheel.rs");
const UI_CANVAS_WIDGET_EVENT_POINTER_WHEEL_ROUTE_RS: &str =
    include_str!("ui/canvas/widget/event_pointer_wheel_route.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_WHEEL_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer_wheel.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_WHEEL_CX_RS: &str =
    include_str!("ui/canvas/widget/event_router_pointer_wheel_cx.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_CX_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_cx.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_MOTION_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_motion.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_PAN_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_pan.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_PAN_APPLY_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_pan/apply.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_VIEWPORT_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_viewport.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_zoom.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_APPLY_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_zoom/apply.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_PINCH_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_zoom/pinch.rs");
const UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_WHEEL_RS: &str =
    include_str!("ui/canvas/widget/pointer_wheel_zoom/wheel.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_cx.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/primary.rs");
const UI_CANVAS_WIDGET_PRIMARY_POINTER_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/primary_pointer_move_cx.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_SURFACE_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/primary/surface.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_GROUP_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/primary/group.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_NODE_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/primary/node.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_CONNECTION_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/primary/connection.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/secondary.rs");
const UI_CANVAS_WIDGET_SECONDARY_POINTER_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/secondary_pointer_move_cx.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_NODE_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/secondary/node.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_CONNECTION_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_INSERT_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/secondary/insert.rs");
const UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_OVERLAY_RS: &str =
    include_str!("ui/canvas/widget/pointer_move_dispatch/overlay.rs");
const UI_CANVAS_WIDGET_STICKY_WIRE_TARGETS_RS: &str =
    include_str!("ui/canvas/widget/sticky_wire_targets.rs");
const UI_CANVAS_WIDGET_STICKY_WIRE_TARGETS_PICKER_RS: &str =
    include_str!("ui/canvas/widget/sticky_wire_targets/picker.rs");
const UI_CANVAS_WIDGET_GROUP_DRAG_TAIL_RS: &str =
    include_str!("ui/canvas/widget/group_drag/tail.rs");
const UI_CANVAS_WIDGET_GROUP_RESIZE_TAIL_RS: &str =
    include_str!("ui/canvas/widget/group_resize/tail.rs");
const UI_CANVAS_WIDGET_GROUP_PREVIEW_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/group_preview_move_cx.rs");
const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_RS: &str =
    include_str!("ui/canvas/widget/keyboard_shortcuts.rs");
const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_COMMANDS_RS: &str =
    include_str!("ui/canvas/widget/keyboard_shortcuts_commands.rs");
const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_OVERLAY_RS: &str =
    include_str!("ui/canvas/widget/keyboard_shortcuts_overlay.rs");
const UI_CANVAS_WIDGET_EVENT_KEYBOARD_ROUTE_RS: &str =
    include_str!("ui/canvas/widget/event_keyboard_route.rs");
const UI_CANVAS_WIDGET_EVENT_KEYBOARD_RETAINED_CX_RS: &str =
    include_str!("ui/canvas/widget/event_keyboard_retained_cx.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_INPUT_RS: &str =
    include_str!("ui/canvas/widget/event_router_system_input.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_RS: &str =
    include_str!("ui/canvas/widget/event_router_system.rs");
const UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_LIFECYCLE_RS: &str =
    include_str!("ui/canvas/widget/event_router_system_lifecycle.rs");
const UI_CANVAS_WIDGET_EVENT_CLIPBOARD_RS: &str =
    include_str!("ui/canvas/widget/event_clipboard.rs");
const UI_CANVAS_WIDGET_INTERNAL_DRAG_EVENT_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/internal_event.rs");
const UI_CANVAS_WIDGET_INTERNAL_DRAG_MOVE_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/internal_move.rs");
const UI_CANVAS_WIDGET_INTERNAL_DRAG_DROP_RS: &str =
    include_str!("ui/canvas/widget/insert_node_drag/internal_drop.rs");
const UI_CANVAS_WIDGET_KEYBOARD_PAN_ACTIVATION_RS: &str =
    include_str!("ui/canvas/widget/keyboard_pan_activation.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_CONSTRAINTS_RS: &str =
    include_str!("ui/canvas/widget/node_drag_constraints.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_CONSTRAINTS_EXTENT_RS: &str =
    include_str!("ui/canvas/widget/node_drag_constraints_extent.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_GEOMETRY_CX_RS: &str =
    include_str!("ui/canvas/widget/node_drag_geometry_cx.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/node_drag_move_cx.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_MOVE_TAIL_CX_RS: &str =
    include_str!("ui/canvas/widget/node_drag_move_tail_cx.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_RS: &str =
    include_str!("ui/canvas/widget/node_drag_preview.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_COMPUTE_RS: &str =
    include_str!("ui/canvas/widget/node_drag_preview/compute.rs");
const UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_CX_RS: &str =
    include_str!("ui/canvas/widget/node_drag_preview_cx.rs");
const UI_CANVAS_WIDGET_NODE_RESIZE_MOVE_RS: &str =
    include_str!("ui/canvas/widget/node_resize/move_update.rs");
const UI_CANVAS_WIDGET_NODE_RESIZE_MOVE_CX_RS: &str =
    include_str!("ui/canvas/widget/node_resize_move_cx.rs");
const UI_CANVAS_WIDGET_PENDING_GROUP_DRAG_RS: &str =
    include_str!("ui/canvas/widget/pending_group_drag.rs");
const UI_CANVAS_WIDGET_PENDING_GROUP_RESIZE_RS: &str =
    include_str!("ui/canvas/widget/pending_group_resize.rs");
const UI_CANVAS_WIDGET_PENDING_GROUP_ACTIVATION_CX_RS: &str =
    include_str!("ui/canvas/widget/pending_group_activation_cx.rs");
const UI_CANVAS_WIDGET_PENDING_NODE_DRAG_ACTIVATION_CX_RS: &str =
    include_str!("ui/canvas/widget/pending_node_drag_activation_cx.rs");
const UI_CANVAS_WIDGET_PENDING_WIRE_DRAG_RS: &str =
    include_str!("ui/canvas/widget/pending_wire_drag.rs");
const UI_CANVAS_WIDGET_PENDING_NODE_DRAG_RELEASE_CX_RS: &str =
    include_str!("ui/canvas/widget/pending_node_drag_release_cx.rs");
const UI_CANVAS_WIDGET_POINTER_UP_PENDING_CLICK_SELECT_RS: &str =
    include_str!("ui/canvas/widget/pointer_up_pending/click_select.rs");
const UI_CANVAS_WIDGET_POINTER_DOWN_DOUBLE_CLICK_EDGE_FINISH_RS: &str =
    include_str!("ui/canvas/widget/pointer_down_double_click_edge/finish.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_INPUT_RS: &str =
    include_str!("ui/canvas/widget/context_menu/input.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_POINTER_RS: &str =
    include_str!("ui/canvas/widget/context_menu/pointer.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/activate.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_COMMAND_RS: &str =
    include_str!("ui/canvas/widget/context_menu/activate/command.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_TARGET_RS: &str =
    include_str!("ui/canvas/widget/context_menu/activate/target.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_BACKGROUND_EXECUTION_RS: &str =
    include_str!("ui/canvas/widget/context_menu/background_execution.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_BACKGROUND_EXECUTION_ACTIVATE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/background_execution/activate.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_BACKGROUND_EXECUTION_APPLY_RS: &str =
    include_str!("ui/canvas/widget/context_menu/background_execution/apply.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_insert.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_ACTIVATE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_insert/activate.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_APPLY_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_insert/apply.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_RECOVERY_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_CONVERSION_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_conversion.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_CONVERSION_ACTIVATE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_CONVERSION_APPLY_RS: &str =
    include_str!("ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_RS: &str =
    include_str!("ui/canvas/widget/context_menu/edge_execution.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_OPEN_INSERT_RS: &str =
    include_str!("ui/canvas/widget/context_menu/edge_execution/open_insert.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_REROUTE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/edge_execution/reroute.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_DELETE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/edge_execution/delete.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_CUSTOM_ACTION_RS: &str =
    include_str!("ui/canvas/widget/context_menu/edge_execution/custom_action.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_RS: &str =
    include_str!("ui/canvas/widget/context_menu/opening.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_BACKGROUND_RS: &str =
    include_str!("ui/canvas/widget/context_menu/opening/background.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_EDGE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/opening/edge.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_GROUP_RS: &str =
    include_str!("ui/canvas/widget/context_menu/opening/group.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_UI_RS: &str =
    include_str!("ui/canvas/widget/context_menu/ui.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_UI_OVERLAY_RS: &str =
    include_str!("ui/canvas/widget/context_menu/ui/overlay.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_UI_EVENT_RS: &str =
    include_str!("ui/canvas/widget/context_menu/ui/event.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_KEY_NAVIGATION_RS: &str =
    include_str!("ui/canvas/widget/context_menu/key_navigation.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_KEY_NAVIGATION_KEY_DOWN_RS: &str =
    include_str!("ui/canvas/widget/context_menu/key_navigation/key_down.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_KEY_NAVIGATION_POINTER_MOVE_RS: &str =
    include_str!("ui/canvas/widget/context_menu/key_navigation/pointer_move.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_SELECTION_ACTIVATION_RS: &str =
    include_str!("ui/canvas/widget/context_menu/selection_activation.rs");
const UI_CANVAS_WIDGET_CONTEXT_MENU_SELECTION_ACTIVATION_POINTER_DOWN_RS: &str =
    include_str!("ui/canvas/widget/context_menu/selection_activation/pointer_down.rs");
const UI_CANVAS_WIDGET_RIGHT_CLICK_PENDING_RS: &str =
    include_str!("ui/canvas/widget/right_click/pending.rs");
const UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_RS: &str =
    include_str!("ui/canvas/widget/searcher_activation.rs");
const UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_POINTER_DOWN_RS: &str =
    include_str!("ui/canvas/widget/searcher_activation/pointer_down.rs");
const UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_POINTER_UP_RS: &str =
    include_str!("ui/canvas/widget/searcher_activation/pointer_up.rs");
const UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_STATE_ARM_RS: &str =
    include_str!("ui/canvas/widget/searcher_activation_state/arm.rs");
const UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_STATE_CLEAR_RS: &str =
    include_str!("ui/canvas/widget/searcher_activation_state/clear.rs");
const UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_STATE_RELEASE_RS: &str =
    include_str!("ui/canvas/widget/searcher_activation_state/release.rs");
const UI_CANVAS_WIDGET_SEARCHER_INPUT_DISPATCH_RS: &str =
    include_str!("ui/canvas/widget/searcher_input/dispatch.rs");
const UI_CANVAS_WIDGET_SEARCHER_INPUT_QUERY_RS: &str =
    include_str!("ui/canvas/widget/searcher_input_query.rs");
const UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_RS: &str =
    include_str!("ui/canvas/widget/searcher_row_activation.rs");
const UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_ITEM_RS: &str =
    include_str!("ui/canvas/widget/searcher_row_activation/item.rs");
const UI_CANVAS_WIDGET_INSERT_CANDIDATES_MENU_RS: &str =
    include_str!("ui/canvas/widget/insert_candidates/menu.rs");
const UI_CANVAS_WIDGET_SEARCHER_PICKER_REQUEST_RS: &str =
    include_str!("ui/canvas/widget/searcher_picker/request.rs");
const UI_CANVAS_WIDGET_SEARCHER_ROWS_RS: &str = include_str!("ui/canvas/widget/searcher_rows.rs");
const UI_CANVAS_WIDGET_SEARCHER_POINTER_RS: &str =
    include_str!("ui/canvas/widget/searcher_pointer.rs");
const UI_CANVAS_WIDGET_SEARCHER_POINTER_MOVE_EVENT_RS: &str =
    include_str!("ui/canvas/widget/searcher_pointer/move_event.rs");
const UI_CANVAS_WIDGET_SEARCHER_POINTER_WHEEL_EVENT_RS: &str =
    include_str!("ui/canvas/widget/searcher_pointer/wheel_event.rs");
const UI_CANVAS_WIDGET_SEARCHER_UI_RS: &str = include_str!("ui/canvas/widget/searcher_ui.rs");
const UI_CANVAS_WIDGET_SEARCHER_UI_OVERLAY_RS: &str =
    include_str!("ui/canvas/widget/searcher_ui/overlay.rs");
const UI_CANVAS_WIDGET_SEARCHER_UI_EVENT_RS: &str =
    include_str!("ui/canvas/widget/searcher_ui/event.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_SHARED_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_shared.rs");
const UI_CANVAS_WIDGET_EVENT_TIMER_ROUTE_RS: &str =
    include_str!("ui/canvas/widget/event_timer_route.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_CX_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_cx.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_AUTO_PAN_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_auto_pan.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_AUTO_PAN_DISPATCH_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_auto_pan/dispatch.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_PAN_INERTIA_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_pan_inertia.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_VIEWPORT_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_viewport.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_VIEWPORT_ANIMATION_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_viewport/animation.rs");
const UI_CANVAS_WIDGET_TIMER_MOTION_VIEWPORT_DEBOUNCE_RS: &str =
    include_str!("ui/canvas/widget/timer_motion_viewport/debounce.rs");
const UI_CANVAS_WIDGET_RETAINED_RUNTIME_EVENT_RS: &str =
    include_str!("ui/canvas/widget/retained_widget_runtime_event.rs");
const UI_CANVAS_WIDGET_RETAINED_CULL_WINDOW_RS: &str =
    include_str!("ui/canvas/widget/retained_widget_cull_window.rs");
const UI_CANVAS_WIDGET_RETAINED_CULL_WINDOW_SHIFT_RS: &str =
    include_str!("ui/canvas/widget/retained_widget_cull_window_shift.rs");
const UI_CANVAS_WIDGET_VIEWPORT_MOTION_CX_RS: &str =
    include_str!("ui/canvas/widget/viewport_motion_cx.rs");
const UI_VIEW_QUEUE_RS: &str = include_str!("ui/canvas/widget/view_queue.rs");
const FRET_EXAMPLES_CARGO_TOML: &str = include_str!("../../../apps/fret-examples/Cargo.toml");
const FRET_EXAMPLES_LIB_RS: &str = include_str!("../../../apps/fret-examples/src/lib.rs");
const FRET_DEMO_CARGO_TOML: &str = include_str!("../../../apps/fret-demo/Cargo.toml");
const FRETBOARD_NATIVE_RS: &str = include_str!("../../../apps/fretboard/src/dev/native.rs");
const FRET_NODE_README_MD: &str = include_str!("../README.md");
const NODE_GRAPH_XYFLOW_GUIDE_MD: &str =
    include_str!("../../../docs/node-graph-how-to-build-like-xyflow.md");
const NODE_GRAPH_CONTROLLED_MODE_MD: &str =
    include_str!("../../../docs/node-graph-controlled-mode.md");
const NODE_GRAPH_DEMO_RS: &str = include_str!("../../../apps/fret-examples/src/node_graph_demo.rs");
const UI_GALLERY_CARGO_TOML: &str = include_str!("../../../apps/fret-ui-gallery/Cargo.toml");
const UI_GALLERY_NODE_GRAPH_CULL_TORTURE_RS: &str = include_str!(
    "../../../apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs"
);
const WORKFLOW_NODE_GRAPH_DEMO_RS: &str =
    include_str!("../../../apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs");

fn public_surface() -> &'static str {
    LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
}

fn binding_surface() -> String {
    [
        UI_BINDING_RS,
        UI_BINDING_QUERIES_RS,
        UI_BINDING_STORE_SYNC_RS,
        UI_BINDING_VIEWPORT_RS,
    ]
    .join("\n")
}

fn source_without_tests(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

fn struct_body<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source
        .find(&format!("struct {name} {{"))
        .unwrap_or_else(|| panic!("missing struct `{name}`"));
    let body_start = source[start..]
        .find('{')
        .map(|offset| start + offset + 1)
        .expect("struct should have a body");
    let body_end = source[body_start..]
        .find("\n}")
        .map(|offset| body_start + offset)
        .expect("struct body should close on its own line");
    &source[body_start..body_end]
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("source directory should be readable") {
        let entry = entry.expect("source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn source_rel_path(path: &Path, root: &Path) -> String {
    let rel = path
        .strip_prefix(root)
        .expect("source file should be under scan root")
        .to_string_lossy()
        .replace('\\', "/");
    format!("src/ui/{rel}")
}

#[test]
fn app_integration_stays_under_explicit_app_module() {
    let public_surface = public_surface();
    assert!(public_surface.contains("pub mod app;"));
    assert!(public_surface.contains("pub mod advanced;"));
    assert!(!public_surface.contains("pub use app::"));
    assert!(!public_surface.contains("pub use advanced::"));
    assert!(!public_surface.contains("pub fn install("));
    assert!(!public_surface.contains("pub fn install_with_ui_services("));
    assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
    assert!(!APP_RS.contains("install_with_ui_services"));
    assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
}

#[test]
fn retained_compatibility_surface_stays_declarative_only() {
    let public_surface = public_surface();
    assert!(!public_surface.contains("pub mod imui;"));
    assert!(!CARGO_TOML.contains("\nimui = ["));
    assert!(!CARGO_TOML.contains("fret-authoring"));
    assert!(!CARGO_TOML.contains("compat-retained-bridge"));
    assert!(
        CARGO_TOML.contains(
            "compat-retained-canvas = [\"fret-ui\", \"fret-ui/compat-retained-widgets\"]"
        )
    );
    assert!(
        !CARGO_TOML.contains("fret-ui/unstable-retained-bridge"),
        "fret-node compat-retained-canvas must no longer enable the deleted fret-ui bridge"
    );
    assert!(
        !UI_DECLARATIVE_MOD_RS.contains("node_graph_surface_compat_retained")
            && !UI_DECLARATIVE_MOD_RS.contains("NodeGraphSurfaceCompatRetainedProps"),
        "`fret-node` declarative surface must not expose a retained-subtree compatibility entry point"
    );
    assert!(
        !UI_MOD_RS.contains("node_graph_surface_compat_retained")
            && !UI_MOD_RS.contains("NodeGraphSurfaceCompatRetainedProps"),
        "`fret-node::ui` must not re-export retained-subtree declarative compatibility"
    );
    assert!(
        !UI_DECLARATIVE_MOD_RS.contains("RetainedSubtreeProps")
            && !UI_MOD_RS.contains("RetainedSubtreeProps"),
        "retained subtree compatibility must stay out of the public declarative node graph path"
    );
}

#[test]
fn public_node_graph_guides_teach_binding_first_surface() {
    assert!(FRET_NODE_README_MD.contains("## Recommended usage (declarative-first)"));
    assert!(FRET_NODE_README_MD.contains("NodeGraphSurfaceBinding"));
    assert!(FRET_NODE_README_MD.contains("node_graph_surface(...)"));
    assert!(!FRET_NODE_README_MD.contains("NodeGraphCanvas::new("));
    assert!(!FRET_NODE_README_MD.contains("NodeGraphCanvas::with_store"));

    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("## Recommended (binding-first) integration"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("NodeGraphSurfaceBinding::new("));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("node_graph_surface(cx, surface.surface_props())"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("NodeGraphController::new(surface.store_model())"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("dispatch_transaction*"));
    assert!(NODE_GRAPH_XYFLOW_GUIDE_MD.contains("fit_view_nodes_in_bounds*"));

    for forbidden in [
        "The UI consumes:",
        "`Model<Graph>` (for painting and hit-testing)",
        "`Model<NodeGraphViewState>` (pan/zoom/selection)",
        "optional `Model<NodeGraphStore>`",
        "NodeGraphCanvas::new(",
        "NodeGraphCanvas::with_store",
    ] {
        assert!(
            !NODE_GRAPH_XYFLOW_GUIDE_MD.contains(forbidden),
            "XyFlow-style guide must stay binding-first; found stale teaching text `{forbidden}`"
        );
    }
}

#[test]
fn retained_canvas_mirror_owner_quarantines_external_models() {
    let mirrors = struct_body(UI_CANVAS_WIDGET_RS, "NodeGraphCanvasMirrors");
    assert!(mirrors.contains("graph: Model<Graph>,"));
    assert!(mirrors.contains("view_state: Model<NodeGraphViewState>,"));
    assert!(mirrors.contains("editor_config: Option<Model<NodeGraphEditorConfig>>,"));

    let canvas = struct_body(UI_CANVAS_WIDGET_RS, "NodeGraphCanvasWith<M>");
    assert!(canvas.contains("mirrors: NodeGraphCanvasMirrors,"));
    assert!(!canvas.contains("\n    graph: Model<Graph>,"));
    assert!(!canvas.contains("\n    view_state: Model<NodeGraphViewState>,"));
    assert!(!canvas.contains("\n    editor_config_model: Option<Model<NodeGraphEditorConfig>>,"));
}

#[test]
fn retained_canvas_commit_pipeline_has_no_legacy_mirror_writer() {
    assert!(!UI_CANVAS_WIDGET_RS.contains("mod commit_legacy;"));
    assert!(!UI_CANVAS_WIDGET_RS.contains("commit_ops_legacy"));
    assert!(!UI_CANVAS_WIDGET_RS.contains("commit_transaction_legacy"));
    assert!(!UI_CANVAS_WIDGET_RS.contains("apply_transaction_result_legacy"));
    assert!(UI_CANVAS_WIDGET_COMMIT_RS.contains("mod apply;"));
    assert!(UI_CANVAS_WIDGET_COMMIT_RS.contains("mod commit;"));
}

#[test]
fn raw_transport_surface_stays_crate_internal() {
    assert!(!UI_MOD_RS.contains("pub mod advanced;"));
    assert!(!UI_MOD_RS.contains("pub mod edit_queue;"));
    assert!(!UI_MOD_RS.contains("NodeGraphEditQueue"));
    assert!(!UI_MOD_RS.contains("bind_controller_edit_queue_transport"));
    assert!(!UI_MOD_RS.contains("NodeGraphViewQueue"));
}

#[test]
fn controller_surface_stays_store_first_without_embedded_transport_state() {
    assert!(!UI_CONTROLLER_RS.contains("edit_queue: Option<"));
    assert!(!UI_CONTROLLER_RS.contains("view_queue: Option<"));
    assert!(!UI_CONTROLLER_RS.contains("bind_edit_queue_transport"));
    assert!(!UI_CONTROLLER_RS.contains("bind_view_queue_transport"));
    assert!(!UI_CONTROLLER_RS.contains("transport_edit_queue"));
    assert!(!UI_CONTROLLER_RS.contains("transport_view_queue"));
}

#[test]
fn fit_view_surface_stays_bounds_first() {
    let binding_surface = binding_surface();
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes("));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_action_host("));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options("));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options_action_host("));
    assert!(!binding_surface.contains("pub fn fit_view_nodes("));
    assert!(UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_in_bounds<"));
    assert!(UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_canvas_rect_in_bounds<"));
    assert!(binding_surface.contains("pub fn fit_view_nodes_in_bounds<"));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds<"));
}

#[test]
fn binding_surface_covers_instance_style_viewport_helpers() {
    let binding_surface = binding_surface();
    assert!(binding_surface.contains("pub fn set_viewport_with_options<"));
    assert!(binding_surface.contains("pub fn set_viewport_with_options_action_host("));
    assert!(binding_surface.contains("pub fn set_center_in_bounds<"));
    assert!(binding_surface.contains("pub fn set_center_in_bounds_action_host("));
    assert!(binding_surface.contains("pub fn set_center_in_bounds_with_options<"));
    assert!(binding_surface.contains("pub fn set_center_in_bounds_with_options_action_host("));
    assert!(binding_surface.contains("pub fn fit_view_nodes_in_bounds_with_options<"));
    assert!(binding_surface.contains("pub fn fit_view_nodes_in_bounds_with_options_action_host("));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds<"));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_action_host("));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options<"));
    assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options_action_host("));
    assert!(binding_surface.contains("pub fn screen_to_canvas<"));
    assert!(binding_surface.contains("pub fn canvas_to_screen<"));
}

#[test]
fn binding_surface_covers_instance_style_sync_and_history_helpers() {
    let binding_surface = binding_surface();
    assert!(binding_surface.contains(
            "struct NodeGraphSurfaceMirrors {\n    graph: Model<Graph>,\n    view_state: Model<NodeGraphViewState>,\n    editor_config: Model<NodeGraphEditorConfig>,\n}"
        ));
    assert!(binding_surface.contains(
            "pub struct NodeGraphSurfaceBinding {\n    mirrors: NodeGraphSurfaceMirrors,\n    store: Model<NodeGraphStore>,\n    internals: Arc<NodeGraphInternalsStore>,\n}"
        ));
    assert!(binding_surface.contains("pub fn from_models_and_controller("));
    assert!(!binding_surface.contains("pub fn from_models_and_controller_with_editor_config("));
    assert!(!binding_surface.contains("pub fn from_models("));
    assert!(binding_surface.contains("pub fn dispatch_transaction<"));
    assert!(binding_surface.contains("pub fn dispatch_transaction_action_host("));
    assert!(binding_surface.contains("pub fn submit_transaction<"));
    assert!(binding_surface.contains("pub fn submit_transaction_action_host("));
    assert!(binding_surface.contains("pub fn update_node<"));
    assert!(binding_surface.contains("pub fn update_node_action_host<"));
    assert!(binding_surface.contains("pub fn update_edge<"));
    assert!(binding_surface.contains("pub fn update_edge_action_host<"));
    assert!(binding_surface.contains("FnOnce(&mut NodeGraphNodeUpdate)"));
    assert!(binding_surface.contains("FnOnce(&mut NodeGraphEdgeUpdate)"));
    assert!(binding_surface.contains("pub fn store_model(&self) -> Model<NodeGraphStore> {"));
    assert!(
        binding_surface.contains("pub fn internals_store(&self) -> Arc<NodeGraphInternalsStore> {")
    );
    assert!(!binding_surface.contains("pub fn controller(&self) -> NodeGraphController {"));
    assert!(binding_surface.contains("pub fn replace_graph_action_host("));
    assert!(binding_surface.contains("pub fn replace_document_action_host("));
    assert!(binding_surface.contains("pub fn replace_view_state_action_host("));
    assert!(binding_surface.contains("pub fn set_selection_action_host("));
    assert!(binding_surface.contains("pub fn undo_action_host("));
    assert!(binding_surface.contains("pub fn redo_action_host("));
}

#[test]
fn controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper() {
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("### Current replace policy"));
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("**full replace first**"));
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("NodeGraphSurfaceBinding::replace_document("));
    assert!(
        NODE_GRAPH_CONTROLLED_MODE_MD
            .contains("NodeGraphController::replace_document_and_sync_models(")
    );
    assert!(NODE_GRAPH_CONTROLLED_MODE_MD.contains("replace_graph(...)"));
    assert!(
        NODE_GRAPH_CONTROLLED_MODE_MD.contains(
            "Diff-first replace helpers remain intentionally deferred until we have a concrete"
        ),
        "controlled-mode docs must keep the public helper decision explicit"
    );

    let binding_surface = binding_surface();
    let controlled_sync_sources =
        [binding_surface.as_str(), UI_CONTROLLER_STORE_SYNC_RS].join("\n");

    assert!(controlled_sync_sources.contains("pub fn replace_graph<"));
    assert!(controlled_sync_sources.contains("pub fn replace_document<"));
    assert!(controlled_sync_sources.contains("pub fn replace_graph_and_sync_models<"));
    assert!(controlled_sync_sources.contains("pub fn replace_document_and_sync_models<"));
    assert!(
        !controlled_sync_sources.contains("graph_diff"),
        "public controlled sync helpers should not hide diff-first replace semantics"
    );

    for forbidden in [
        "pub fn replace_graph_with_diff",
        "pub fn replace_document_with_diff",
        "pub fn replace_graph_diff",
        "pub fn replace_document_diff",
        "pub fn apply_graph_diff",
        "pub fn sync_graph_diff",
    ] {
        assert!(
            !controlled_sync_sources.contains(forbidden),
            "diff-first controlled sync remains deferred; found `{forbidden}`"
        );
    }
}

#[test]
fn update_helpers_hide_structural_fields_behind_explicit_transactions() {
    assert!(UI_CONTROLLER_UPDATES_RS.contains("pub struct NodeGraphNodeUpdate"));
    assert!(UI_CONTROLLER_UPDATES_RS.contains("pub struct NodeGraphEdgeUpdate"));
    assert!(!UI_CONTROLLER_UPDATES_RS.contains("pub ports:"));
    assert!(!UI_CONTROLLER_UPDATES_RS.contains("pub from:"));
    assert!(!UI_CONTROLLER_UPDATES_RS.contains("pub to:"));
    assert!(UI_CONTROLLER_UPDATES_RS.contains("Use explicit transactions for port"));
    assert!(UI_CONTROLLER_UPDATES_RS.contains("Use explicit transactions for reconnects"));
}

#[test]
fn root_ui_surface_re_exports_store_first_viewport_option_types_but_not_raw_view_queue_module() {
    assert!(!UI_MOD_RS.contains("mod view_queue;"));
    assert!(UI_MOD_RS.contains("mod viewport_options;"));
    assert!(!UI_MOD_RS.contains("pub mod view_queue;"));
    assert!(UI_MOD_RS.contains(
        "pub use viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};"
    ));
}

#[test]
fn public_viewport_option_surface_stays_store_first() {
    assert!(UI_VIEWPORT_OPTIONS_RS.contains("pub struct NodeGraphFitViewOptions"));
    assert!(UI_VIEWPORT_OPTIONS_RS.contains("pub struct NodeGraphSetViewportOptions"));
    assert!(!UI_VIEWPORT_OPTIONS_RS.contains("duration_ms"));
    assert!(!UI_VIEWPORT_OPTIONS_RS.contains("interpolate"));
    assert!(!UI_VIEWPORT_OPTIONS_RS.contains("ease"));
    assert!(UI_VIEW_QUEUE_RS.contains("pub(crate) struct NodeGraphViewQueueFitViewOptions"));
    assert!(UI_VIEW_QUEUE_RS.contains("pub(crate) struct NodeGraphViewQueueSetViewportOptions"));
    assert!(UI_VIEW_QUEUE_RS.contains("duration_ms"));
    assert!(UI_VIEW_QUEUE_RS.contains("interpolate"));
    assert!(UI_VIEW_QUEUE_RS.contains("ease"));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("duration_ms"));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("interpolate"));
    assert!(!UI_CONTROLLER_VIEWPORT_RS.contains("ease"));
}

#[test]
fn retained_widget_compat_island_stays_crate_private_and_controller_bound() {
    assert!(UI_MOD_RS.contains("mod canvas;"));
    assert!(!UI_MOD_RS.contains("pub mod canvas;"));
    assert!(!UI_MOD_RS.contains("pub use canvas::NodeGraphCanvas"));
    assert!(!UI_MOD_RS.contains("pub(crate) use canvas::NodeGraphCanvas"));
    assert!(!UI_MOD_RS.contains("pub use canvas::NodeGraphCanvasWith"));
    assert!(!UI_MOD_RS.contains("pub(crate) use canvas::NodeGraphCanvasWith"));
    assert!(!UI_MOD_RS.contains("pub mod a11y;"));
    assert!(!UI_MOD_RS.contains("pub mod editor;"));
    assert!(!UI_MOD_RS.contains("pub mod editors;"));
    assert!(!UI_MOD_RS.contains("pub mod overlays;"));
    assert!(!UI_MOD_RS.contains("pub mod panel;"));
    assert!(!UI_MOD_RS.contains("pub mod portal;"));
    assert!(!UI_MOD_RS.contains("pub use editor::NodeGraphEditor"));
    assert!(!UI_MOD_RS.contains("pub use panel::{NodeGraphPanel"));
    assert!(!UI_MOD_RS.contains("pub use portal::{"));
    assert!(!UI_MOD_RS.contains("pub use overlays::{"));
    assert!(UI_MOD_RS.contains("#[cfg(all(test, feature = \"compat-retained-canvas\"))]"));
    assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod editor;"));
    assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod panel;"));

    assert!(UI_CANVAS_RS.contains(
            "pub fn new(\n        graph: Model<Graph>,\n        view_state: Model<NodeGraphViewState>,\n        editor_config: Model<NodeGraphEditorConfig>,\n    ) -> Self {"
        ));
    assert!(
        UI_CANVAS_BUILDERS_RS.contains(
            "pub fn with_controller(mut self, controller: NodeGraphController) -> Self {"
        )
    );
    assert!(!UI_CANVAS_BUILDERS_RS.contains("with_editor_config_model("));
    assert!(UI_CANVAS_BUILDERS_RS.contains("pub(crate) fn with_view_queue("));
    assert!(UI_CANVAS_BUILDERS_RS.contains("retained compatibility plumbing"));
    assert!(UI_CANVAS_BUILDERS_RS.contains("declarative node graph surface"));

    assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod portal;"));
}

#[test]
fn pure_geometry_and_route_math_helpers_are_available_without_compat_gating() {
    let geometry_mod = include_str!("ui/canvas/geometry/mod.rs");
    let route_math_mod = include_str!("ui/canvas/route_math.rs");

    for forbidden in [
        "#[cfg(any(test, feature = \"compat-retained-canvas\"))]\npub(crate) use order::group_order;",
        "#[cfg(any(test, feature = \"compat-retained-canvas\"))]\npub(crate) use origin::{node_anchor_from_rect_origin, node_rect_origin_from_anchor};",
        "#[cfg(feature = \"compat-retained-canvas\")]\npub(crate) use route_math_curve::{cubic_bezier, normal_from_tangent};",
        "#[cfg(feature = \"compat-retained-canvas\")]\npub(crate) use route_math_tangent::{edge_route_end_tangent, edge_route_start_tangent};",
    ] {
        assert!(
            !geometry_mod.contains(forbidden) && !route_math_mod.contains(forbidden),
            "pure node geometry/route math helpers should not be compat-gated: {forbidden}"
        );
    }
}

#[test]
fn retained_bridge_source_usage_stays_on_the_migration_ledger() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_root = manifest_dir.join("src/ui");
    let mut files = Vec::new();
    collect_rs_files(&ui_root, &mut files);

    let allowed_exact = ["src/ui/canvas/widget.rs"];
    let allowed_prefixes = ["src/ui/canvas/widget/"];
    let retained_terms = [
        "use fret_ui::compat_retained_canvas",
        "use fret_ui::{UiHost, compat_retained_canvas",
        "fret_ui::compat_retained_canvas::",
        "RetainedSubtreeProps",
        "UiTreeRetainedExt",
    ];

    let mut offenders = Vec::new();
    for path in files {
        let source =
            std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
        if !retained_terms.iter().any(|term| source.contains(term)) {
            continue;
        }

        let rel = source_rel_path(&path, &ui_root);
        let allowed = allowed_exact.contains(&rel.as_str())
            || allowed_prefixes
                .iter()
                .any(|prefix| rel.starts_with(prefix));
        if !allowed {
            offenders.push(rel);
        }
    }

    assert!(
        offenders.is_empty(),
        "retained bridge source usage must stay on the explicit compat-retained-canvas migration ledger:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn retained_canvas_deleted_compat_facade_stays_out_of_ui_sources() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_root = manifest_dir.join("src/ui");
    let mut files = Vec::new();
    collect_rs_files(&ui_root, &mut files);

    let mut offenders = Vec::new();
    for path in files {
        let source =
            std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
        for forbidden in [
            "compat_retained_canvas::*",
            "compat_retained_canvas::widget",
            "compat_retained_canvas::command",
            "compat_retained_canvas::event",
            "compat_retained_canvas::layout",
            "compat_retained_canvas::paint",
            "compat_retained_canvas::frame",
        ] {
            if source.contains(forbidden) {
                offenders.push(format!(
                    "{} contains `{forbidden}`",
                    source_rel_path(&path, &ui_root)
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "deleted compat_retained_canvas facade imports must stay out of UI sources; use the feature-gated fret_ui exports or explicit adapters instead:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn overlay_menu_toolbar_policy_ownership_stays_on_named_seams() {
    assert!(UI_CANVAS_STATE_RS.contains("mod state_overlay_policy;"));
    assert!(
        UI_CANVAS_STATE_RS.contains(
            "pub(crate) use state_overlay_policy::{ContextMenuTarget, SearcherRowsMode};"
        )
    );
    assert!(UI_CANVAS_STATE_OVERLAY_POLICY_RS.contains("enum ContextMenuTarget"));
    assert!(UI_CANVAS_STATE_OVERLAY_POLICY_RS.contains("enum SearcherRowsMode"));
    assert!(
        UI_CANVAS_STATE_OVERLAY_SESSIONS_RS
            .contains("use super::state_overlay_policy::{ContextMenuTarget, SearcherRowsMode};")
    );
    assert!(
        !source_without_tests(UI_CANVAS_STATE_OVERLAY_SESSIONS_RS)
            .contains("enum ContextMenuTarget")
    );
    assert!(
        !source_without_tests(UI_CANVAS_STATE_OVERLAY_SESSIONS_RS)
            .contains("enum SearcherRowsMode")
    );

    assert!(UI_OVERLAYS_MOD_RS.contains("mod toolbar_policy;"));
    assert!(UI_OVERLAYS_MOD_RS.contains("mod toolbar_layout_policy;"));
    assert!(UI_OVERLAYS_MOD_RS.contains("mod toolbars_declarative;"));
    for required in [
        "pub enum NodeGraphToolbarVisibility",
        "pub enum NodeGraphToolbarPosition",
        "pub enum NodeGraphToolbarAlign",
        "pub enum NodeGraphToolbarSize",
        "resolve_node_toolbar_window_target",
        "resolve_edge_toolbar_window_target",
    ] {
        assert!(
            UI_OVERLAY_TOOLBAR_POLICY_RS.contains(required),
            "toolbar public policy surface should stay in toolbar_policy.rs: {required}"
        );
    }
    for forbidden in [
        "pub enum NodeGraphToolbarVisibility",
        "pub enum NodeGraphToolbarPosition",
        "pub enum NodeGraphToolbarAlign",
        "pub enum NodeGraphToolbarSize",
        "fn resolve_node_toolbar_window_target",
        "fn resolve_edge_toolbar_window_target",
    ] {
        assert!(
            !source_without_tests(UI_OVERLAY_TOOLBARS_DECLARATIVE_RS).contains(forbidden),
            "declarative toolbar composition should consume toolbar policy, not own it: {forbidden}"
        );
    }
    assert!(
        UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("use super::toolbar_policy::{"),
        "declarative toolbar composition should import the policy seam"
    );
    assert!(
        UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("use super::toolbar_layout_policy::{"),
        "declarative toolbar composition should import the layout-policy seam"
    );

    assert!(
        UI_CANVAS_WIDGET_CONTEXT_MENU_UI_OVERLAY_RS.contains("enum ContextMenuHoverEdgePolicy")
    );
    assert!(UI_CANVAS_WIDGET_CONTEXT_MENU_UI_RS.contains("mod overlay;"));
    assert!(
        UI_CANVAS_WIDGET_CONTEXT_MENU_UI_RS
            .contains("pub(in crate::ui::canvas::widget) use overlay::ContextMenuHoverEdgePolicy;")
    );
    assert!(
        UI_CANVAS_WIDGET_CONTEXT_MENU_UI_RS.contains(
            "overlay::apply_context_menu_open_state(interaction, menu, hover_edge_policy);"
        )
    );
    assert!(
        UI_CANVAS_WIDGET_EDGE_INSERT_CONTEXT_MENU_RS
            .contains("context_menu::apply_context_menu_open_state(")
    );
    assert!(
        UI_CANVAS_WIDGET_EDGE_INSERT_CONTEXT_MENU_RS
            .contains("context_menu::ContextMenuHoverEdgePolicy::Preserve")
    );
    assert!(
        !source_without_tests(UI_CANVAS_WIDGET_EDGE_INSERT_CONTEXT_MENU_RS)
            .contains("interaction.context_menu = Some"),
        "edge-insert context menu reopening should use context_menu/ui/overlay.rs"
    );

    assert!(UI_CANVAS_WIDGET_SEARCHER_UI_RS.contains("mod overlay;"));
    assert!(
        UI_CANVAS_WIDGET_SEARCHER_UI_RS
            .contains("overlay::install_searcher_overlay(self, searcher);")
    );
    assert!(UI_CANVAS_WIDGET_SEARCHER_UI_OVERLAY_RS.contains("fn apply_searcher_overlay_state("));
    assert!(UI_CANVAS_WIDGET_SEARCHER_UI_OVERLAY_RS.contains("context_menu::clear_context_menu("));
    assert!(UI_CANVAS_WIDGET_SEARCHER_PICKER_REQUEST_RS.contains("struct SearcherPickerRequest"));
    assert!(UI_CANVAS_WIDGET_SEARCHER_PICKER_REQUEST_RS.contains("rows_mode: SearcherRowsMode"));
    assert!(
        UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_RS.contains("searcher_ui::take_searcher_overlay")
    );
    assert!(
        UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_RS
            .contains("searcher_ui::restore_searcher_overlay")
    );
    assert!(
        UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_ITEM_RS
            .contains("build_insert_candidate_menu_item")
    );
    assert!(
        UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_ITEM_RS.contains("searcher_is_selectable_row")
    );
    assert!(
        UI_CANVAS_WIDGET_INSERT_CANDIDATES_MENU_RS
            .contains("NodeGraphContextMenuAction::InsertNodeCandidate")
    );
    assert!(UI_CANVAS_WIDGET_SEARCHER_ROWS_RS.contains("searcher_is_selectable_row"));

    for (name, source) in [
        (
            "searcher row activation",
            UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_RS,
        ),
        (
            "context menu command activation",
            UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_COMMAND_RS,
        ),
    ] {
        assert!(
            !source_without_tests(source).contains("interaction.searcher = None")
                && !source_without_tests(source).contains("interaction.searcher.take()")
                && !source_without_tests(source).contains("interaction.context_menu = None")
                && !source_without_tests(source).contains("interaction.context_menu.take()"),
            "{name} should use the named overlay lifecycle helpers"
        );
    }
}

#[test]
fn workflow_gallery_surface_stays_binding_first_for_viewport_controls() {
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.observe(cx);"));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("LayoutQueryRegionProps::default()"));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("node_graph_surface(cx, props)"));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.set_viewport_action_host("));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds_action_host("));
    assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds(cx.app,"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("RetainedSubtreeProps"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("retained_bridge"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphCanvas::new"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphEditor::new"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("create_node_retained"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("retained_subtree"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphController"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.controller()"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains(".with_controller(binding.controller())"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphViewQueue"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("bind_controller_view_queue_transport"));
    assert!(!WORKFLOW_NODE_GRAPH_DEMO_RS.contains("use fret_node::ui::advanced::{"));
}

#[test]
fn first_party_gallery_node_graph_pages_stay_off_retained_canvas() {
    assert!(
        UI_GALLERY_CARGO_TOML
            .contains("fret-node = { path = \"../../ecosystem/fret-node\", optional = true }")
    );
    assert!(!UI_GALLERY_CARGO_TOML.contains("fret-node/compat-retained-canvas"));
    assert!(!UI_GALLERY_CARGO_TOML.contains(
            "fret-node = { path = \"../../ecosystem/fret-node\", optional = true, features = [\"compat-retained-canvas\"] }"
        ));

    for source in [
        WORKFLOW_NODE_GRAPH_DEMO_RS,
        UI_GALLERY_NODE_GRAPH_CULL_TORTURE_RS,
    ] {
        assert!(source.contains("NodeGraphSurfaceBinding::new("));
        assert!(source.contains("node_graph_surface"));
        assert!(!source.contains("RetainedSubtreeProps"));
        assert!(!source.contains("retained_bridge"));
        assert!(!source.contains("NodeGraphCanvas::new"));
        assert!(!source.contains("NodeGraphEditor::new"));
        assert!(!source.contains("create_node_retained"));
        assert!(!source.contains("retained_subtree"));
    }
}

#[test]
fn first_party_node_graph_demos_stay_declarative_only() {
    for source in [
        FRET_EXAMPLES_CARGO_TOML,
        FRET_EXAMPLES_LIB_RS,
        FRET_DEMO_CARGO_TOML,
        FRETBOARD_NATIVE_RS,
        NODE_GRAPH_DEMO_RS,
    ] {
        assert!(!source.contains("node-graph-demos-legacy"));
        assert!(!source.contains("fret-node/compat-retained-canvas"));
        assert!(!source.contains("node_graph_legacy_demo"));
        assert!(!source.contains("node_graph_domain_demo"));
        assert!(!source.contains("imui_node_graph_demo"));
        assert!(!source.contains("node_graph_tuning_overlay"));
        assert!(!source.contains("RetainedSubtreeProps"));
        assert!(!source.contains("retained_bridge"));
        assert!(!source.contains("NodeGraphCanvas::new"));
        assert!(!source.contains("NodeGraphEditor::new"));
        assert!(!source.contains("create_node_retained"));
        assert!(!source.contains("retained_subtree"));
    }
    assert!(FRET_EXAMPLES_CARGO_TOML.contains("node-graph-demos = []"));
    assert!(
        FRET_DEMO_CARGO_TOML.contains("node-graph-demos = [\"fret-examples/node-graph-demos\"]")
    );
    assert!(FRET_EXAMPLES_LIB_RS.contains("pub mod node_graph_demo;"));
    assert!(NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
    assert!(NODE_GRAPH_DEMO_RS.contains("node_graph_surface_in(cx, props)"));
    assert!(!NODE_GRAPH_DEMO_RS.contains("NodeGraphController"));
    assert!(!NODE_GRAPH_DEMO_RS.contains("binding.controller()"));
}
