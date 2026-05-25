//! Node graph substrate for Fret.
//!
//! This crate provides a long-lived, serializable graph model with typed connections and
//! editor-grade contracts (migrations, diagnostics, deterministic persistence).
//!
//! UI integration is optional and lives behind the default `fret-ui` feature.

#![deny(unsafe_code)]

/// Reserved builtin node kind for a schema-less wire reroute node.
pub const REROUTE_KIND: &str = "fret.reroute";

pub mod core;
pub mod interaction;
pub mod io;
#[cfg(feature = "kit")]
pub mod kit;
pub mod ops;
pub mod profile;
pub mod rules;
pub mod runtime;
pub mod schema;
pub mod types;

#[cfg(feature = "fret-ui")]
pub mod ui;

pub use core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    StickyNote, StickyNoteId, Symbol, SymbolId,
};
pub use interaction::{
    NodeGraphConnectionMode, NodeGraphDragHandleMode, NodeGraphModifierKey,
    NodeGraphZoomActivationKey,
};
pub use rules::{ConnectPlan, Diagnostic, DiagnosticSeverity};
pub use types::{TypeDesc, TypeVarId};

#[cfg(feature = "app-integration")]
pub mod advanced;
#[cfg(feature = "app-integration")]
pub mod app;

#[cfg(test)]
mod surface_policy_tests {
    use std::path::{Path, PathBuf};

    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    const APP_RS: &str = include_str!("app.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");
    const UI_BINDING_RS: &str = include_str!("ui/binding.rs");
    const UI_BINDING_QUERIES_RS: &str = include_str!("ui/binding_queries.rs");
    const UI_BINDING_STORE_SYNC_RS: &str = include_str!("ui/binding_store_sync.rs");
    const UI_BINDING_VIEWPORT_RS: &str = include_str!("ui/binding_viewport.rs");
    const UI_CANVAS_RS: &str = include_str!("ui/canvas/widget/widget_surface.rs");
    const UI_CANVAS_BUILDERS_RS: &str = include_str!("ui/canvas/widget/widget_surface/builders.rs");
    const UI_CONTROLLER_RS: &str = include_str!("ui/controller.rs");
    const UI_CONTROLLER_UPDATES_RS: &str = include_str!("ui/controller_updates.rs");
    const UI_CONTROLLER_VIEWPORT_RS: &str = include_str!("ui/controller_viewport.rs");
    const UI_DECLARATIVE_MOD_RS: &str = include_str!("ui/declarative/mod.rs");
    const UI_EDITORS_MOD_RS: &str = include_str!("ui/editors/mod.rs");
    const UI_EDITOR_PORTAL_NUMBER_RS: &str = include_str!("ui/editors/portal_number.rs");
    const UI_EDITOR_PORTAL_TEXT_RS: &str = include_str!("ui/editors/portal_text.rs");
    const UI_MOD_RS: &str = include_str!("ui/mod.rs");
    const UI_OVERLAYS_MOD_RS: &str = include_str!("ui/overlays/mod.rs");
    const UI_OVERLAY_CONTROLS_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/controls_declarative.rs");
    const UI_OVERLAY_CONTROLS_HOST_POLICY_RS: &str =
        include_str!("ui/overlays/controls_host_policy.rs");
    const UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS: &str =
        include_str!("ui/overlays/controls_interaction_policy.rs");
    const UI_OVERLAY_CONTROLS_PAINT_PLAN_RS: &str =
        include_str!("ui/overlays/controls_paint_plan.rs");
    const UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/blackboard_declarative.rs");
    const UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS: &str =
        include_str!("ui/overlays/blackboard_interaction_policy.rs");
    const UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS: &str =
        include_str!("ui/overlays/blackboard_paint_plan.rs");
    const UI_OVERLAY_MINIMAP_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/minimap_declarative.rs");
    const UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS: &str =
        include_str!("ui/overlays/minimap_interaction_policy.rs");
    const UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS: &str =
        include_str!("ui/overlays/toolbar_layout_policy.rs");
    const UI_OVERLAY_TOOLBARS_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/toolbars_declarative.rs");
    const UI_OVERLAY_RENAME_DECLARATIVE_RS: &str =
        include_str!("ui/overlays/rename_declarative.rs");
    const UI_OVERLAY_RENAME_COMMAND_RS: &str = include_str!("ui/overlays/rename_command.rs");
    const UI_OVERLAY_RENAME_LIFECYCLE_RS: &str = include_str!("ui/overlays/rename_lifecycle.rs");
    const UI_VIEWPORT_OPTIONS_RS: &str = include_str!("ui/viewport_options.rs");
    const UI_CANVAS_WIDGET_MARQUEE_BEGIN_RS: &str =
        include_str!("ui/canvas/widget/marquee_begin.rs");
    const UI_CANVAS_WIDGET_MARQUEE_CX_RS: &str = include_str!("ui/canvas/widget/marquee_cx.rs");
    const UI_CANVAS_WIDGET_MARQUEE_FINISH_RS: &str =
        include_str!("ui/canvas/widget/marquee_finish.rs");
    const UI_CANVAS_WIDGET_MARQUEE_PENDING_RS: &str =
        include_str!("ui/canvas/widget/marquee_pending.rs");
    const UI_CANVAS_WIDGET_MARQUEE_RS: &str = include_str!("ui/canvas/widget/marquee.rs");
    const UI_CANVAS_WIDGET_MARQUEE_SELECTION_RS: &str =
        include_str!("ui/canvas/widget/marquee_selection.rs");
    const UI_CANVAS_WIDGET_PAN_ZOOM_BEGIN_RS: &str =
        include_str!("ui/canvas/widget/pan_zoom_begin.rs");
    const UI_CANVAS_WIDGET_PAN_ZOOM_BEGIN_CX_RS: &str =
        include_str!("ui/canvas/widget/pan_zoom_begin_cx.rs");
    const UI_CANVAS_WIDGET_PAN_ZOOM_MOVE_RS: &str =
        include_str!("ui/canvas/widget/pan_zoom_move.rs");
    const UI_CANVAS_WIDGET_PAINT_INVALIDATION_RS: &str =
        include_str!("ui/canvas/widget/paint_invalidation.rs");
    const UI_CANVAS_WIDGET_REDRAW_REQUEST_RS: &str =
        include_str!("ui/canvas/widget/redraw_request.rs");
    const UI_CANVAS_COMMAND_ADAPTER_RS: &str = include_str!("ui/canvas/widget/command_adapter.rs");
    const UI_CANVAS_RETAINED_COMMAND_ADAPTER_RS: &str =
        include_str!("ui/canvas/widget/retained_command_adapter.rs");
    const UI_CANVAS_LOW_LEVEL_ADAPTER_RS: &str =
        include_str!("ui/canvas/widget/low_level_adapter.rs");
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
    const UI_CANVAS_WIDGET_POINTER_UP_RS: &str = include_str!("ui/canvas/widget/pointer_up.rs");
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
    const UI_CANVAS_WIDGET_CANCEL_RS: &str = include_str!("ui/canvas/widget/cancel.rs");
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
    const UI_CANVAS_WIDGET_EDGE_DRAG_RS: &str = include_str!("ui/canvas/widget/edge_drag/mod.rs");
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
    const UI_CANVAS_WIDGET_STICKY_WIRE_RS: &str = include_str!("ui/canvas/widget/sticky_wire.rs");
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
    const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_UP_RS: &str =
        include_str!("ui/canvas/widget/event_router_pointer_button/up.rs");
    const UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_MOVE_RS: &str =
        include_str!("ui/canvas/widget/event_router_pointer_button/move_event.rs");
    const UI_CANVAS_WIDGET_EVENT_ROUTER_RS: &str = include_str!("ui/canvas/widget/event_router.rs");
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
    const UI_CANVAS_WIDGET_CURSOR_RS: &str = include_str!("ui/canvas/widget/cursor.rs");
    const UI_CANVAS_WIDGET_CURSOR_CX_RS: &str = include_str!("ui/canvas/widget/cursor_cx.rs");
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
    const UI_CANVAS_WIDGET_HOVER_RS: &str = include_str!("ui/canvas/widget/hover.rs");
    const UI_CANVAS_WIDGET_HOVER_MOVE_CX_RS: &str =
        include_str!("ui/canvas/widget/hover_move_cx.rs");
    const UI_CANVAS_WIDGET_CANCEL_CLEANUP_RS: &str =
        include_str!("ui/canvas/widget/cancel_cleanup.rs");
    const UI_CANVAS_WIDGET_STICKY_WIRE_TARGETS_RS: &str =
        include_str!("ui/canvas/widget/sticky_wire_targets.rs");
    const UI_CANVAS_WIDGET_STICKY_WIRE_TARGETS_PICKER_RS: &str =
        include_str!("ui/canvas/widget/sticky_wire_targets/picker.rs");
    const UI_CANVAS_WIDGET_GROUP_DRAG_RS: &str = include_str!("ui/canvas/widget/group_drag.rs");
    const UI_CANVAS_WIDGET_GROUP_DRAG_TAIL_RS: &str =
        include_str!("ui/canvas/widget/group_drag/tail.rs");
    const UI_CANVAS_WIDGET_GROUP_RESIZE_RS: &str = include_str!("ui/canvas/widget/group_resize.rs");
    const UI_CANVAS_WIDGET_GROUP_RESIZE_TAIL_RS: &str =
        include_str!("ui/canvas/widget/group_resize/tail.rs");
    const UI_CANVAS_WIDGET_GROUP_PREVIEW_MOVE_CX_RS: &str =
        include_str!("ui/canvas/widget/group_preview_move_cx.rs");
    const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_RS: &str =
        include_str!("ui/canvas/widget/keyboard_shortcuts.rs");
    const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_RETAINED_CX_RS: &str =
        include_str!("ui/canvas/widget/keyboard_shortcuts_retained_cx.rs");
    const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_COMMANDS_RS: &str =
        include_str!("ui/canvas/widget/keyboard_shortcuts_commands.rs");
    const UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_OVERLAY_RS: &str =
        include_str!("ui/canvas/widget/keyboard_shortcuts_overlay.rs");
    const UI_CANVAS_WIDGET_EVENT_KEYBOARD_ROUTE_RS: &str =
        include_str!("ui/canvas/widget/event_keyboard_route.rs");
    const UI_CANVAS_WIDGET_EVENT_KEYBOARD_RS: &str =
        include_str!("ui/canvas/widget/event_keyboard.rs");
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
    const UI_CANVAS_WIDGET_NODE_DRAG_SNAP_RS: &str =
        include_str!("ui/canvas/widget/node_drag_snap.rs");
    const UI_CANVAS_WIDGET_NODE_DRAG_RS: &str = include_str!("ui/canvas/widget/node_drag.rs");
    const UI_CANVAS_WIDGET_NODE_DRAG_MOVE_CX_RS: &str =
        include_str!("ui/canvas/widget/node_drag_move_cx.rs");
    const UI_CANVAS_WIDGET_NODE_DRAG_TAIL_RS: &str =
        include_str!("ui/canvas/widget/node_drag/tail.rs");
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
    const UI_CANVAS_WIDGET_PENDING_DRAG_RS: &str = include_str!("ui/canvas/widget/pending_drag.rs");
    const UI_CANVAS_WIDGET_PENDING_NODE_DRAG_ACTIVATION_CX_RS: &str =
        include_str!("ui/canvas/widget/pending_node_drag_activation_cx.rs");
    const UI_CANVAS_WIDGET_PENDING_RESIZE_RS: &str =
        include_str!("ui/canvas/widget/pending_resize.rs");
    const UI_CANVAS_WIDGET_PENDING_WIRE_DRAG_RS: &str =
        include_str!("ui/canvas/widget/pending_wire_drag.rs");
    const UI_CANVAS_WIDGET_PENDING_NODE_DRAG_RELEASE_CX_RS: &str =
        include_str!("ui/canvas/widget/pending_node_drag_release_cx.rs");
    const UI_CANVAS_WIDGET_POINTER_UP_PENDING_CLICK_SELECT_RS: &str =
        include_str!("ui/canvas/widget/pointer_up_pending/click_select.rs");
    const UI_CANVAS_WIDGET_POINTER_DOWN_DOUBLE_CLICK_EDGE_FINISH_RS: &str =
        include_str!("ui/canvas/widget/pointer_down_double_click_edge/finish.rs");
    const UI_CANVAS_WIDGET_CONTEXT_MENU_RS: &str =
        include_str!("ui/canvas/widget/context_menu/mod.rs");
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
    const UI_CANVAS_WIDGET_RIGHT_CLICK_RS: &str = include_str!("ui/canvas/widget/right_click.rs");
    const UI_CANVAS_WIDGET_RIGHT_CLICK_PENDING_RS: &str =
        include_str!("ui/canvas/widget/right_click/pending.rs");
    const UI_CANVAS_WIDGET_SEARCHER_RS: &str = include_str!("ui/canvas/widget/searcher.rs");
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
    const UI_CANVAS_WIDGET_SEARCHER_INPUT_RS: &str =
        include_str!("ui/canvas/widget/searcher_input.rs");
    const UI_CANVAS_WIDGET_SEARCHER_INPUT_DISPATCH_RS: &str =
        include_str!("ui/canvas/widget/searcher_input/dispatch.rs");
    const UI_CANVAS_WIDGET_SEARCHER_INPUT_QUERY_RS: &str =
        include_str!("ui/canvas/widget/searcher_input_query.rs");
    const UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_RS: &str =
        include_str!("ui/canvas/widget/searcher_row_activation.rs");
    const UI_CANVAS_WIDGET_SEARCHER_POINTER_RS: &str =
        include_str!("ui/canvas/widget/searcher_pointer.rs");
    const UI_CANVAS_WIDGET_SEARCHER_POINTER_MOVE_EVENT_RS: &str =
        include_str!("ui/canvas/widget/searcher_pointer/move_event.rs");
    const UI_CANVAS_WIDGET_SEARCHER_POINTER_WHEEL_EVENT_RS: &str =
        include_str!("ui/canvas/widget/searcher_pointer/wheel_event.rs");
    const UI_CANVAS_WIDGET_SEARCHER_UI_RS: &str = include_str!("ui/canvas/widget/searcher_ui.rs");
    const UI_CANVAS_WIDGET_SEARCHER_UI_EVENT_RS: &str =
        include_str!("ui/canvas/widget/searcher_ui/event.rs");
    const UI_CANVAS_WIDGET_TIMER_MOTION_SHARED_RS: &str =
        include_str!("ui/canvas/widget/timer_motion_shared.rs");
    const UI_CANVAS_WIDGET_EVENT_TIMER_RS: &str = include_str!("ui/canvas/widget/event_timer.rs");
    const UI_CANVAS_WIDGET_EVENT_TIMER_ROUTE_RS: &str =
        include_str!("ui/canvas/widget/event_timer_route.rs");
    const UI_CANVAS_WIDGET_TIMER_MOTION_RS: &str = include_str!("ui/canvas/widget/timer_motion.rs");
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
    const UI_CANVAS_WIDGET_VIEWPORT_MOTION_CX_RS: &str =
        include_str!("ui/canvas/widget/viewport_motion_cx.rs");
    const UI_VIEW_QUEUE_RS: &str = include_str!("ui/canvas/widget/view_queue.rs");
    const FRET_EXAMPLES_CARGO_TOML: &str = include_str!("../../../apps/fret-examples/Cargo.toml");
    const FRET_EXAMPLES_LIB_RS: &str = include_str!("../../../apps/fret-examples/src/lib.rs");
    const FRET_DEMO_CARGO_TOML: &str = include_str!("../../../apps/fret-demo/Cargo.toml");
    const FRETBOARD_NATIVE_RS: &str = include_str!("../../../apps/fretboard/src/dev/native.rs");
    const NODE_GRAPH_DEMO_RS: &str =
        include_str!("../../../apps/fret-examples/src/node_graph_demo.rs");
    const UI_GALLERY_CARGO_TOML: &str = include_str!("../../../apps/fret-ui-gallery/Cargo.toml");
    const UI_GALLERY_NODE_GRAPH_CULL_TORTURE_RS: &str = include_str!(
        "../../../apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs"
    );
    const WORKFLOW_NODE_GRAPH_DEMO_RS: &str = include_str!(
        "../../../apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs"
    );

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
        assert!(CARGO_TOML.contains(
            "compat-retained-canvas = [\"fret-ui\", \"fret-ui/compat-retained-widgets\"]"
        ));
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
    fn retained_canvas_low_level_adapter_policy_helpers_stay_off_retained_bridge() {
        let adapter_policy_sources = [
            UI_CANVAS_WIDGET_PAINT_INVALIDATION_RS,
            UI_CANVAS_WIDGET_REDRAW_REQUEST_RS,
            UI_CANVAS_LOW_LEVEL_ADAPTER_RS,
            UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS,
            UI_CANVAS_WIDGET_POINTER_UP_FINISH_RS,
            UI_CANVAS_WIDGET_POINTER_UP_SESSION_CLEANUP_RS,
            UI_CANVAS_WIDGET_POINTER_UP_SESSION_RELEASE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_PENDING_RELEASE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_PENDING_RELEASE_GROUP_RS,
            UI_CANVAS_WIDGET_POINTER_UP_PENDING_RELEASE_NODE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_PENDING_WIRE_DRAG_RS,
            UI_CANVAS_WIDGET_STICKY_WIRE_CONNECT_FINISH_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_DRAG_TAIL_RS,
            UI_CANVAS_WIDGET_CANCEL_CLEANUP_RS,
            UI_CANVAS_WIDGET_STICKY_WIRE_TARGETS_RS,
            UI_CANVAS_WIDGET_STICKY_WIRE_TARGETS_PICKER_RS,
            UI_CANVAS_WIDGET_GROUP_DRAG_TAIL_RS,
            UI_CANVAS_WIDGET_GROUP_RESIZE_TAIL_RS,
        ]
        .join("\n");

        for forbidden in [
            "widget_tail",
            "retained_widget_tail",
            "WidgetRedrawCx",
            "WidgetPaintInvalidationCx",
            "WidgetHandledCx",
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !adapter_policy_sources.contains(forbidden),
                "canvas low-level adapter policy helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(UI_CANVAS_LOW_LEVEL_ADAPTER_RS.contains("trait CanvasRedrawCx"));
        assert!(UI_CANVAS_LOW_LEVEL_ADAPTER_RS.contains("trait CanvasPaintInvalidationCx"));
        assert!(UI_CANVAS_LOW_LEVEL_ADAPTER_RS.contains("trait CanvasHandledCx"));
        assert!(UI_CANVAS_LOW_LEVEL_ADAPTER_RS.contains("trait CanvasPointerCaptureReleaseCx"));
        assert!(
            UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS
                .contains("trait WireCommitCx<H>:\n    CanvasPointerCaptureReleaseCx<H>")
        );
        for forbidden in [
            "    fn release_pointer_capture(",
            "    fn request_redraw(",
            "    fn invalidate_paint(",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS
                    .split("#[cfg(test)]")
                    .next()
                    .unwrap_or(UI_CANVAS_WIDGET_WIRE_DRAG_COMMIT_CX_RS)
                    .contains(forbidden),
                "wire commit low-level operations must stay inherited from low_level_adapter; found `{forbidden}`"
            );
        }
        assert!(
            UI_CANVAS_RETAINED_LOW_LEVEL_ADAPTER_RS
                .contains("impl<H: UiHost> low_level_adapter::CanvasRedrawCx<H> for EventCx")
        );
        assert!(UI_CANVAS_RETAINED_LOW_LEVEL_ADAPTER_RS.contains(
            "impl<H: UiHost> low_level_adapter::CanvasPointerCaptureReleaseCx<H> for CommandCx"
        ));
    }

    #[test]
    fn retained_canvas_command_dispatch_adapter_replaces_close_button_retained_edge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_COMMAND_ADAPTER_RS.contains(forbidden),
                "canvas command adapter contract must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(UI_CANVAS_COMMAND_ADAPTER_RS.contains("trait CanvasCommandDispatchCx"));
        assert!(
            UI_CANVAS_RETAINED_COMMAND_ADAPTER_RS
                .contains("impl<H: UiHost> command_adapter::CanvasCommandDispatchCx for EventCx")
        );
        assert!(
            UI_CANVAS_WIDGET_POINTER_DOWN_CLOSE_BUTTON_CX_RS
                .contains("CanvasHandledCx<H> + CanvasCommandDispatchCx")
        );
        assert!(
            !UI_CANVAS_WIDGET_POINTER_DOWN_CLOSE_BUTTON_CX_RS
                .split("#[cfg(test)]")
                .next()
                .unwrap_or(UI_CANVAS_WIDGET_POINTER_DOWN_CLOSE_BUTTON_CX_RS)
                .contains("dispatch_close_command"),
            "close-button cx must inherit command dispatch from command_adapter"
        );
        assert!(
            !UI_CANVAS_RS.contains("pointer_down_close_button_retained_cx"),
            "close-button must not keep a dedicated retained command adapter"
        );
    }

    #[test]
    fn sticky_wire_pointer_down_route_stays_off_retained_bridge() {
        let sticky_wire_sources = [
            UI_CANVAS_WIDGET_STICKY_WIRE_RS,
            UI_CANVAS_WIDGET_STICKY_WIRE_CONNECT_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !sticky_wire_sources.contains(forbidden),
                "sticky-wire pointer-down route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn wire_drag_helpers_stay_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_WIRE_DRAG_HELPERS_RS.contains(forbidden),
                "wire-drag helpers must stay behind retained-agnostic Cx seams; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn group_preview_move_handlers_stay_off_retained_bridge() {
        let group_preview_move_sources = [
            UI_CANVAS_WIDGET_GROUP_DRAG_RS,
            UI_CANVAS_WIDGET_GROUP_RESIZE_RS,
            UI_CANVAS_WIDGET_GROUP_PREVIEW_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !group_preview_move_sources.contains(forbidden),
                "group preview move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pending_group_activation_handlers_stay_off_retained_bridge() {
        let pending_group_activation_sources = [
            UI_CANVAS_WIDGET_PENDING_GROUP_DRAG_RS,
            UI_CANVAS_WIDGET_PENDING_GROUP_RESIZE_RS,
            UI_CANVAS_WIDGET_PENDING_GROUP_ACTIVATION_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pending_group_activation_sources.contains(forbidden),
                "pending group activation handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pending_node_resize_move_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_PENDING_RESIZE_RS.contains(forbidden),
                "pending node resize move helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pending_node_drag_activation_handlers_stay_off_retained_bridge() {
        let pending_node_drag_activation_sources = [
            UI_CANVAS_WIDGET_PENDING_DRAG_RS,
            UI_CANVAS_WIDGET_PENDING_NODE_DRAG_ACTIVATION_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pending_node_drag_activation_sources.contains(forbidden),
                "pending node drag activation handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pending_node_drag_release_handlers_stay_off_retained_bridge() {
        let pending_node_drag_release_sources = [
            UI_CANVAS_WIDGET_POINTER_UP_PENDING_CLICK_SELECT_RS,
            UI_CANVAS_WIDGET_PENDING_NODE_DRAG_RELEASE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pending_node_drag_release_sources.contains(forbidden),
                "pending node drag release handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn wire_drag_move_handlers_stay_off_retained_bridge() {
        let wire_drag_move_sources = [
            UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_CX_RS,
            UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_UPDATE_RS,
            UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_UPDATE_AUTO_PAN_RS,
            UI_CANVAS_WIDGET_WIRE_DRAG_MOVE_UPDATE_PRELUDE_RS,
            UI_CANVAS_WIDGET_PENDING_WIRE_DRAG_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !wire_drag_move_sources.contains(forbidden),
                "wire drag move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn edge_insert_move_handlers_stay_off_retained_bridge() {
        let edge_insert_move_sources = [
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_DRAG_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_PENDING_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_PENDING_ACTIVATE_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_PRELUDE_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_DRAG_TAIL_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !edge_insert_move_sources.contains(forbidden),
                "edge insert move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn edge_drag_move_handlers_stay_off_retained_bridge() {
        let edge_drag_move_sources = [
            UI_CANVAS_WIDGET_EDGE_DRAG_RS,
            UI_CANVAS_WIDGET_EDGE_DRAG_MOVE_START_RS,
            UI_CANVAS_WIDGET_EDGE_DRAG_PRELUDE_RS,
            UI_CANVAS_WIDGET_EDGE_DRAG_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !edge_drag_move_sources.contains(forbidden),
                "edge drag move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn insert_node_drag_move_handlers_stay_off_retained_bridge() {
        let insert_node_drag_move_sources = [
            UI_CANVAS_WIDGET_INSERT_NODE_DRAG_RS,
            UI_CANVAS_WIDGET_INSERT_NODE_DRAG_PENDING_RS,
            UI_CANVAS_WIDGET_INSERT_NODE_DRAG_PRELUDE_RS,
            UI_CANVAS_WIDGET_INSERT_NODE_DRAG_SESSION_RS,
            UI_CANVAS_WIDGET_INSERT_NODE_DRAG_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !insert_node_drag_move_sources.contains(forbidden),
                "insert-node drag move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn edge_insert_menu_and_insert_routes_stay_off_retained_bridge() {
        let edge_insert_sources = [
            UI_CANVAS_WIDGET_EDGE_INSERT_PRELUDE_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_INSERT_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_CONTEXT_MENU_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !edge_insert_sources.contains(forbidden),
                "edge-insert menu/insert routes should stay behind retained-agnostic Cx seams; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn node_drag_move_handlers_stay_off_retained_bridge() {
        let node_drag_move_sources = [
            UI_CANVAS_WIDGET_NODE_DRAG_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_MOVE_CX_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_TAIL_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_MOVE_TAIL_CX_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_CONSTRAINTS_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_CONSTRAINTS_EXTENT_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_GEOMETRY_CX_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_SNAP_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_COMPUTE_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !node_drag_move_sources.contains(forbidden),
                "node drag move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn node_resize_move_handlers_stay_off_retained_bridge() {
        let node_resize_move_sources = [
            UI_CANVAS_WIDGET_NODE_RESIZE_MOVE_RS,
            UI_CANVAS_WIDGET_NODE_RESIZE_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !node_resize_move_sources.contains(forbidden),
                "node resize move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_commit_handlers_stay_off_retained_bridge() {
        let pointer_up_commit_sources = [
            UI_CANVAS_WIDGET_POINTER_UP_COMMIT_CX_RS,
            UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RS,
            UI_CANVAS_WIDGET_POINTER_UP_NODE_DRAG_RS,
            UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_COMMIT_RS,
            UI_CANVAS_WIDGET_POINTER_UP_COMMIT_GROUP_DRAG_RS,
            UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RESIZE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RESIZE_GROUP_RS,
            UI_CANVAS_WIDGET_POINTER_UP_COMMIT_RESIZE_NODE_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pointer_up_commit_sources.contains(forbidden),
                "pointer-up commit handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_pending_dispatch_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_PENDING_RS.contains(forbidden),
                "pointer-up pending dispatch must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_active_dispatch_stays_off_retained_bridge() {
        let pointer_up_active_sources = [
            UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_ACTIVE_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_POINTER_UP_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_POINTER_UP_ACTIVE_RS,
            UI_CANVAS_WIDGET_EDGE_INSERT_DRAG_POINTER_UP_PENDING_RS,
            UI_CANVAS_WIDGET_EDGE_DRAG_POINTER_UP_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pointer_up_active_sources.contains(forbidden),
                "pointer-up active dispatch must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_route_wrappers_stay_off_retained_bridge() {
        let pointer_up_route_sources = [
            UI_CANVAS_WIDGET_POINTER_UP_RS,
            UI_CANVAS_WIDGET_POINTER_UP_LEFT_RS,
            UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DISPATCH_RS,
            UI_CANVAS_WIDGET_MARQUEE_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pointer_up_route_sources.contains(forbidden),
                "pointer-up route wrappers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn edge_double_click_finish_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_DOWN_DOUBLE_CLICK_EDGE_FINISH_RS.contains(forbidden),
                "edge double-click finish helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn searcher_top_level_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_SEARCHER_RS.contains(forbidden),
                "searcher top-level route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_top_level_route_stays_off_retained_bridge() {
        let context_menu_top_level_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_INPUT_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_POINTER_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_top_level_sources.contains(forbidden),
                "context menu top-level route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_opening_route_stays_off_retained_bridge() {
        let context_menu_opening_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_BACKGROUND_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_EDGE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_OPENING_GROUP_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_opening_sources.contains(forbidden),
                "context menu opening route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_activation_route_stays_off_retained_bridge() {
        let context_menu_activation_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_COMMAND_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_ACTIVATE_TARGET_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_activation_sources.contains(forbidden),
                "context menu activation route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_background_execution_stays_off_retained_bridge() {
        let context_menu_background_execution_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_BACKGROUND_EXECUTION_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_BACKGROUND_EXECUTION_ACTIVATE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_BACKGROUND_EXECUTION_APPLY_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_background_execution_sources.contains(forbidden),
                "context menu background execution must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_edge_execution_stays_off_retained_bridge() {
        let context_menu_edge_execution_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_OPEN_INSERT_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_REROUTE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_DELETE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_EDGE_EXECUTION_CUSTOM_ACTION_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_edge_execution_sources.contains(forbidden),
                "context menu edge execution must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_connection_insert_execution_stays_off_retained_bridge() {
        let context_menu_connection_insert_execution_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_ACTIVATE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_APPLY_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_INSERT_RECOVERY_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_connection_insert_execution_sources.contains(forbidden),
                "context menu connection insert execution must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_connection_conversion_execution_stays_off_retained_bridge() {
        let context_menu_connection_conversion_execution_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_CONVERSION_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_CONVERSION_ACTIVATE_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_CONNECTION_EXECUTION_CONVERSION_APPLY_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_connection_conversion_execution_sources.contains(forbidden),
                "context menu connection conversion execution must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_ui_tail_stays_off_retained_bridge() {
        let context_menu_ui_tail_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_UI_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_UI_EVENT_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_ui_tail_sources.contains(forbidden),
                "context menu UI tail helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_pointer_move_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_CONTEXT_MENU_KEY_NAVIGATION_POINTER_MOVE_RS.contains(forbidden),
                "context menu pointer-move helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_key_down_route_stays_off_retained_bridge() {
        let context_menu_key_down_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_KEY_NAVIGATION_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_KEY_NAVIGATION_KEY_DOWN_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_key_down_sources.contains(forbidden),
                "context menu key-down route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn context_menu_selection_activation_route_stays_off_retained_bridge() {
        let context_menu_selection_sources = [
            UI_CANVAS_WIDGET_CONTEXT_MENU_SELECTION_ACTIVATION_RS,
            UI_CANVAS_WIDGET_CONTEXT_MENU_SELECTION_ACTIVATION_POINTER_DOWN_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !context_menu_selection_sources.contains(forbidden),
                "context menu selection activation route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn right_click_route_stays_off_retained_bridge() {
        let right_click_sources = [
            UI_CANVAS_WIDGET_RIGHT_CLICK_RS,
            UI_CANVAS_WIDGET_RIGHT_CLICK_PENDING_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !right_click_sources.contains(forbidden),
                "right-click route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_guard_dispatch_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_POINTER_UP_DISPATCH_RS.contains(forbidden),
                "pointer-up guard dispatch must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_event_entry_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_POINTER_UP_RS.contains(forbidden),
                "pointer-up event entry must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_button_router_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_UP_RS.contains(forbidden),
                "pointer-up button router must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_release_route_stays_off_retained_bridge() {
        let pointer_up_release_sources = [
            UI_CANVAS_WIDGET_POINTER_UP_RELEASE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_STATE_RELEASE_RS,
            UI_CANVAS_WIDGET_POINTER_UP_RELEASE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pointer_up_release_sources.contains(forbidden),
                "pointer-up release route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_up_left_double_click_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_UP_LEFT_ROUTE_DOUBLE_CLICK_RS.contains(forbidden),
                "pointer-up left double-click route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn cancel_lifecycle_helpers_stay_off_retained_bridge() {
        let cancel_sources = [
            UI_CANVAS_WIDGET_CANCEL_RS,
            UI_CANVAS_WIDGET_CANCEL_VIEWPORT_STATE_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !cancel_sources.contains(forbidden),
                "cancel lifecycle helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn searcher_dismiss_tail_helpers_stay_off_retained_bridge() {
        let searcher_dismiss_sources = [
            UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_RS,
            UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_POINTER_DOWN_RS,
            UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_POINTER_UP_RS,
            UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_STATE_ARM_RS,
            UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_STATE_CLEAR_RS,
            UI_CANVAS_WIDGET_SEARCHER_ACTIVATION_STATE_RELEASE_RS,
            UI_CANVAS_WIDGET_SEARCHER_INPUT_RS,
            UI_CANVAS_WIDGET_SEARCHER_INPUT_DISPATCH_RS,
            UI_CANVAS_WIDGET_SEARCHER_INPUT_QUERY_RS,
            UI_CANVAS_WIDGET_SEARCHER_POINTER_RS,
            UI_CANVAS_WIDGET_SEARCHER_POINTER_MOVE_EVENT_RS,
            UI_CANVAS_WIDGET_SEARCHER_POINTER_WHEEL_EVENT_RS,
            UI_CANVAS_WIDGET_SEARCHER_UI_RS,
            UI_CANVAS_WIDGET_SEARCHER_UI_EVENT_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !searcher_dismiss_sources.contains(forbidden),
                "searcher dismiss tail helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn searcher_row_activation_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_SEARCHER_ROW_ACTIVATION_RS.contains(forbidden),
                "searcher row activation route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn node_drag_move_tail_stays_off_retained_bridge() {
        let node_drag_move_tail_sources = [
            UI_CANVAS_WIDGET_NODE_DRAG_TAIL_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_MOVE_TAIL_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !node_drag_move_tail_sources.contains(forbidden),
                "node drag move tail helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn node_drag_geometry_helpers_stay_off_retained_bridge() {
        let node_drag_geometry_sources = [
            UI_CANVAS_WIDGET_NODE_DRAG_CONSTRAINTS_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_CONSTRAINTS_EXTENT_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_GEOMETRY_CX_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_SNAP_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !node_drag_geometry_sources.contains(forbidden),
                "node drag geometry helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn keyboard_shortcut_command_helpers_stay_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_COMMANDS_RS.contains(forbidden),
                "keyboard shortcut command helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_RS.contains("trait KeyboardShortcutCommandSink"),
            "keyboard shortcut retained adapter should stay isolated behind a named command seam"
        );
    }

    #[test]
    fn keyboard_shortcut_wrapper_stays_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_RS.contains(forbidden),
                "keyboard shortcut wrapper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_RETAINED_CX_RS.contains("EventCx"),
            "keyboard shortcut retained adapter should stay explicit and isolated"
        );
    }

    #[test]
    fn keyboard_overlay_helpers_stay_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_OVERLAY_RS.contains(forbidden),
                "keyboard overlay helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_KEYBOARD_SHORTCUTS_OVERLAY_RS.contains("trait KeyboardOverlayCx"),
            "keyboard overlay route should stay behind the composed searcher/context-menu/cancel seam"
        );
    }

    #[test]
    fn keyboard_event_route_stays_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_KEYBOARD_ROUTE_RS.contains(forbidden),
                "keyboard event route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_EVENT_KEYBOARD_ROUTE_RS.contains("trait KeyboardRouteCx"),
            "keyboard event route should stay behind a composed retained-agnostic route seam"
        );
    }

    #[test]
    fn keyboard_system_input_route_stays_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_INPUT_RS.contains(forbidden),
                "keyboard system input route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_EVENT_KEYBOARD_RS.contains("trait KeyboardInputSink"),
            "keyboard retained adapter should expose a narrow input sink seam"
        );
    }

    #[test]
    fn keyboard_input_focus_helper_stays_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_KEYBOARD_RS.contains(forbidden),
                "keyboard input focus helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_EVENT_KEYBOARD_RS.contains("trait KeyboardInputFocusCx"),
            "keyboard input focus helper should expose a narrow retained-agnostic focus seam"
        );
        assert!(
            UI_CANVAS_WIDGET_EVENT_KEYBOARD_RETAINED_CX_RS.contains("EventCx"),
            "keyboard input focus retained adapter should stay explicit and isolated"
        );
    }

    #[test]
    fn system_non_pointer_route_stays_off_retained_cx() {
        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_RS.contains(forbidden),
                "system non-pointer route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_RS.contains("trait SystemRouteCx"),
            "system non-pointer route should stay behind a composed retained-agnostic route seam"
        );
    }

    #[test]
    fn system_lifecycle_route_helpers_stay_off_retained_cx() {
        let lifecycle_sources = [
            UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_LIFECYCLE_RS,
            UI_CANVAS_WIDGET_EVENT_CLIPBOARD_RS,
            UI_CANVAS_WIDGET_INTERNAL_DRAG_EVENT_RS,
            UI_CANVAS_WIDGET_INTERNAL_DRAG_MOVE_RS,
            UI_CANVAS_WIDGET_INTERNAL_DRAG_DROP_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !lifecycle_sources.contains(forbidden),
                "system lifecycle route helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }

        assert!(
            UI_CANVAS_WIDGET_EVENT_ROUTER_SYSTEM_LIFECYCLE_RS.contains("trait SystemLifecycleCx"),
            "system lifecycle route should stay behind a composed retained-agnostic route seam"
        );
        assert!(
            UI_CANVAS_WIDGET_EVENT_CLIPBOARD_RS.contains("trait ClipboardTextCx"),
            "clipboard lifecycle handling should expose a retained-agnostic text seam"
        );
    }

    #[test]
    fn keyboard_pan_activation_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_KEYBOARD_PAN_ACTIVATION_RS.contains(forbidden),
                "keyboard pan activation helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn feedback_motion_helpers_stay_off_retained_bridge() {
        let feedback_motion_sources = [
            UI_CANVAS_WIDGET_EVENT_CLIPBOARD_FEEDBACK_RS,
            UI_CANVAS_WIDGET_EVENT_CLIPBOARD_FEEDBACK_CX_RS,
            UI_CANVAS_WIDGET_EVENT_TIMER_TOAST_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_SHARED_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !feedback_motion_sources.contains(forbidden),
                "feedback and motion helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn node_drag_preview_compute_stays_off_retained_bridge() {
        let node_drag_preview_sources = [
            UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_COMPUTE_RS,
            UI_CANVAS_WIDGET_NODE_DRAG_PREVIEW_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !node_drag_preview_sources.contains(forbidden),
                "node drag preview compute helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn marquee_begin_finish_stays_off_retained_bridge() {
        let marquee_sources = [
            UI_CANVAS_WIDGET_MARQUEE_BEGIN_RS,
            UI_CANVAS_WIDGET_MARQUEE_CX_RS,
            UI_CANVAS_WIDGET_MARQUEE_FINISH_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !marquee_sources.contains(forbidden),
                "marquee begin/finish helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn marquee_move_handlers_stay_off_retained_bridge() {
        let marquee_sources = [
            UI_CANVAS_WIDGET_MARQUEE_PENDING_RS,
            UI_CANVAS_WIDGET_MARQUEE_SELECTION_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !marquee_sources.contains(forbidden),
                "marquee move handlers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pan_zoom_begin_helpers_stay_off_retained_bridge() {
        let pan_zoom_sources = [
            UI_CANVAS_WIDGET_PAN_ZOOM_BEGIN_RS,
            UI_CANVAS_WIDGET_PAN_ZOOM_BEGIN_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pan_zoom_sources.contains(forbidden),
                "pan-zoom begin helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pan_zoom_move_helpers_stay_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_PAN_ZOOM_MOVE_RS.contains(forbidden),
                "pan-zoom move helpers must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_primary_surface_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_SURFACE_RS.contains(forbidden),
                "pointer-move primary surface route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_primary_group_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_GROUP_RS.contains(forbidden),
                "pointer-move primary group route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_primary_node_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_NODE_RS.contains(forbidden),
                "pointer-move primary node route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_primary_connection_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_CONNECTION_RS.contains(forbidden),
                "pointer-move primary connection route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_primary_route_wrapper_stays_off_retained_bridge() {
        let primary_route_sources = [
            UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_PRIMARY_RS,
            UI_CANVAS_WIDGET_PRIMARY_POINTER_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !primary_route_sources.contains(forbidden),
                "pointer-move primary route wrapper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_secondary_route_wrapper_stays_off_retained_bridge() {
        let secondary_route_sources = [
            UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_RS,
            UI_CANVAS_WIDGET_SECONDARY_POINTER_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !secondary_route_sources.contains(forbidden),
                "pointer-move secondary route wrapper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_secondary_node_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_NODE_RS.contains(forbidden),
                "pointer-move secondary node route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_secondary_connection_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_CONNECTION_RS.contains(forbidden),
                "pointer-move secondary connection route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_secondary_insert_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_SECONDARY_INSERT_RS.contains(forbidden),
                "pointer-move secondary insert route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_overlay_route_stays_off_retained_bridge() {
        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_OVERLAY_RS.contains(forbidden),
                "pointer-move overlay route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_hover_fallback_stays_off_retained_bridge() {
        let hover_sources =
            [UI_CANVAS_WIDGET_HOVER_RS, UI_CANVAS_WIDGET_HOVER_MOVE_CX_RS].join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !hover_sources.contains(forbidden),
                "pointer-move hover fallback must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_route_wrapper_stays_off_retained_bridge() {
        let pointer_move_sources = [
            UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_MOVE_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_RELEASE_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_ROUTE_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_DISPATCH_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !pointer_move_sources.contains(forbidden),
                "pointer-move route wrapper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_cursor_update_stays_off_retained_bridge() {
        let cursor_sources = [
            UI_CANVAS_WIDGET_CURSOR_RS,
            UI_CANVAS_WIDGET_CURSOR_CX_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_CURSOR_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !cursor_sources.contains(forbidden),
                "pointer-move cursor update must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_auto_pan_timer_stays_off_retained_bridge() {
        let timer_sources = [
            UI_CANVAS_WIDGET_AUTO_PAN_TIMER_CX_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_TIMER_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !timer_sources.contains(forbidden),
                "pointer-move auto-pan timer helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_tail_wrapper_stays_off_retained_bridge() {
        let tail_sources = [
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_TAIL_CX_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !tail_sources.contains(forbidden),
                "pointer-move tail wrapper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_release_route_stays_off_retained_bridge() {
        let release_sources = [
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_RELEASE_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_MOVE_TAIL_ROUTE_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_PAN_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_PAN_MISSING_RS,
            UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_PAN_PENDING_RIGHT_CLICK_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !release_sources.contains(forbidden),
                "pointer-move release route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_move_missing_left_release_stays_off_retained_bridge() {
        let release_sources = [UI_CANVAS_WIDGET_POINTER_MOVE_RELEASE_LEFT_RS].join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !release_sources.contains(forbidden),
                "pointer-move missing-left-release helper must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn pointer_wheel_route_stays_off_retained_bridge() {
        let wheel_sources = [
            UI_CANVAS_WIDGET_EVENT_POINTER_WHEEL_RS,
            UI_CANVAS_WIDGET_EVENT_POINTER_WHEEL_ROUTE_RS,
            UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_WHEEL_RS,
            UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_WHEEL_CX_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_CX_RS,
            UI_CANVAS_WIDGET_VIEWPORT_MOTION_CX_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_MOTION_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_PAN_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_PAN_APPLY_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_VIEWPORT_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_APPLY_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_PINCH_RS,
            UI_CANVAS_WIDGET_POINTER_WHEEL_ZOOM_WHEEL_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !wheel_sources.contains(forbidden),
                "pointer-wheel route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn timer_motion_route_stays_off_retained_bridge() {
        let timer_sources = [
            UI_CANVAS_WIDGET_EVENT_TIMER_RS,
            UI_CANVAS_WIDGET_EVENT_TIMER_ROUTE_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_CX_RS,
            UI_CANVAS_WIDGET_VIEWPORT_MOTION_CX_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_AUTO_PAN_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_AUTO_PAN_DISPATCH_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_PAN_INERTIA_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_VIEWPORT_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_VIEWPORT_ANIMATION_RS,
            UI_CANVAS_WIDGET_TIMER_MOTION_VIEWPORT_DEBOUNCE_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !timer_sources.contains(forbidden),
                "timer-motion route must stay retained-Cx agnostic; found `{forbidden}`"
            );
        }
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
        assert!(
            !UI_CONTROLLER_VIEWPORT_RS.contains("pub fn fit_view_nodes_with_options_action_host(")
        );
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
        assert!(
            binding_surface.contains("pub fn fit_view_nodes_in_bounds_with_options_action_host(")
        );
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds<"));
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_action_host("));
        assert!(binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options<"));
        assert!(
            binding_surface.contains("pub fn fit_canvas_rect_in_bounds_with_options_action_host(")
        );
        assert!(binding_surface.contains("pub fn screen_to_canvas<"));
        assert!(binding_surface.contains("pub fn canvas_to_screen<"));
    }

    #[test]
    fn binding_surface_covers_instance_style_sync_and_history_helpers() {
        let binding_surface = binding_surface();
        assert!(binding_surface.contains(
            "pub struct NodeGraphSurfaceBinding {\n    graph: Model<Graph>,\n    view_state: Model<NodeGraphViewState>,\n    editor_config: Model<NodeGraphEditorConfig>,\n    store: Model<NodeGraphStore>,\n    internals: Arc<NodeGraphInternalsStore>,\n}"
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
            binding_surface
                .contains("pub fn internals_store(&self) -> Arc<NodeGraphInternalsStore> {")
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
    fn root_ui_surface_re_exports_store_first_viewport_option_types_but_not_raw_view_queue_module()
    {
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
        assert!(
            UI_VIEW_QUEUE_RS.contains("pub(crate) struct NodeGraphViewQueueSetViewportOptions")
        );
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
        assert!(UI_CANVAS_BUILDERS_RS.contains(
            "pub fn with_controller(mut self, controller: NodeGraphController) -> Self {"
        ));
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
    fn left_click_route_stays_off_retained_bridge() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let left_click_root = ui_root.join("canvas/widget/left_click");
        let mut files = Vec::new();
        collect_rs_files(&left_click_root, &mut files);

        let retained_terms = [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ];

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if !retained_terms.iter().any(|term| source.contains(term)) {
                continue;
            }

            offenders.push(source_rel_path(&path, &ui_root));
        }

        assert!(
            offenders.is_empty(),
            "left-click routing should stay behind retained-agnostic Cx seams:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn pointer_down_double_click_route_stays_off_retained_bridge() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let route_roots = [
            ui_root.join("canvas/widget/pointer_down_double_click_background"),
            ui_root.join("canvas/widget/pointer_down_double_click_edge"),
        ];
        let mut files = vec![ui_root.join("canvas/widget/pointer_down_double_click.rs")];
        files.push(ui_root.join("canvas/widget/event_pointer_down_route/double_click.rs"));
        files.push(ui_root.join("canvas/widget/event_pointer_down_state.rs"));
        for root in route_roots {
            collect_rs_files(&root, &mut files);
        }

        let retained_terms = [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ];

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if !retained_terms.iter().any(|term| source.contains(term)) {
                continue;
            }

            offenders.push(source_rel_path(&path, &ui_root));
        }

        assert!(
            offenders.is_empty(),
            "pointer-down double-click routing should stay behind retained-agnostic Cx seams:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn pointer_down_preflight_route_stays_off_retained_bridge() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let files = [
            ui_root.join("canvas/widget/event_pointer_down_route/preflight.rs"),
            ui_root.join("canvas/widget/pointer_down_gesture_start/close_button.rs"),
        ];

        let retained_terms = [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ];

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if !retained_terms.iter().any(|term| source.contains(term)) {
                continue;
            }

            offenders.push(source_rel_path(&path, &ui_root));
        }

        assert!(
            offenders.is_empty(),
            "pointer-down preflight routing should stay behind retained-agnostic Cx seams:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn pointer_down_starts_route_stays_off_retained_bridge() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let files = [
            ui_root.join("canvas/widget/event_pointer_down_route/starts.rs"),
            ui_root.join("canvas/widget/pointer_down_gesture_start.rs"),
            ui_root.join("canvas/widget/pointer_down_gesture_start/menu.rs"),
            ui_root.join("canvas/widget/pointer_down_gesture_start/pending_right_click.rs"),
            ui_root.join("canvas/widget/pointer_down_gesture_start/pan_start.rs"),
            ui_root.join("canvas/widget/pointer_down_gesture_start/sticky.rs"),
        ];

        let retained_terms = [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ];

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if !retained_terms.iter().any(|term| source.contains(term)) {
                continue;
            }

            offenders.push(source_rel_path(&path, &ui_root));
        }

        assert!(
            offenders.is_empty(),
            "pointer-down start routing should stay behind retained-agnostic Cx seams:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn pointer_down_tail_route_stays_off_retained_bridge() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let files = [
            ui_root.join("canvas/widget/event_pointer_down.rs"),
            ui_root.join("canvas/widget/event_pointer_down_route.rs"),
            ui_root.join("canvas/widget/event_pointer_down_route/dispatch.rs"),
            ui_root.join("canvas/widget/event_router_pointer_button/down.rs"),
        ];

        let retained_terms = [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ];

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if !retained_terms.iter().any(|term| source.contains(term)) {
                continue;
            }

            offenders.push(source_rel_path(&path, &ui_root));
        }

        assert!(
            offenders.is_empty(),
            "pointer-down tail routing should stay behind retained-agnostic Cx seams:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn top_level_event_router_stays_off_retained_bridge() {
        let router_sources = [
            UI_CANVAS_WIDGET_EVENT_ROUTER_RS,
            UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_RS,
            UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_RS,
            UI_CANVAS_WIDGET_EVENT_ROUTER_POINTER_BUTTON_DOWN_RS,
        ]
        .join("\n");

        for forbidden in [
            "retained_bridge",
            "compat_retained_canvas",
            "EventCx",
            "CommandCx",
            "LayoutCx",
            "PaintCx",
        ] {
            assert!(
                !router_sources.contains(forbidden),
                "top-level event routing should stay behind retained-agnostic Cx seams; found `{forbidden}`"
            );
        }
    }

    #[test]
    fn retained_canvas_facade_usage_stays_explicit_not_globbed() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::*") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "compat_retained_canvas facade imports must stay explicit; glob imports hide which retained bridge exports remain needed:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn retained_canvas_widget_trait_stays_on_feature_gated_widget_export() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::widget") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "widget trait imports must use the feature-gated fret_ui::Widget export, not the deleted compat facade:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn retained_canvas_command_contexts_stay_on_feature_gated_command_export() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::command") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "command ctx imports must use the feature-gated fret_ui::CommandCx export, not the deleted compat facade:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn retained_canvas_event_contexts_stay_on_feature_gated_event_export() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::event") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "event ctx imports must use the feature-gated fret_ui::EventCx export, not the deleted compat facade:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn retained_canvas_layout_contexts_stay_on_feature_gated_layout_export() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::layout") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "layout ctx imports must use the feature-gated fret_ui::LayoutCx export, not the deleted compat facade:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn retained_canvas_paint_contexts_stay_on_feature_gated_paint_export() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::paint") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "paint ctx imports must use the feature-gated fret_ui::PaintCx export, not the deleted compat facade:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn retained_canvas_frame_contexts_stay_on_feature_gated_frame_export() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_root = manifest_dir.join("src/ui");
        let mut files = Vec::new();
        collect_rs_files(&ui_root, &mut files);

        let mut offenders = Vec::new();
        for path in files {
            let source =
                std::fs::read_to_string(&path).expect("source file should be readable as UTF-8");
            if source.contains("compat_retained_canvas::frame") {
                offenders.push(source_rel_path(&path, &ui_root));
            }
        }

        assert!(
            offenders.is_empty(),
            "frame ctx imports must use the feature-gated fret_ui::PrepaintCx / fret_ui::SemanticsCx exports, not the deleted compat facade:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn overlay_policy_modules_compile_without_retained_canvas_compat() {
        assert!(UI_MOD_RS.contains("mod overlays;"));
        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod overlays;"));
        assert!(UI_MOD_RS.contains("mod screen_space_placement;"));
        assert!(
            !UI_MOD_RS.contains(
                "#[cfg(feature = \"compat-retained-canvas\")]\nmod screen_space_placement;"
            )
        );

        for module in [
            "mod blackboard_declarative;",
            "mod blackboard_interaction_policy;",
            "mod blackboard_layout;",
            "mod blackboard_paint_plan;",
            "mod blackboard_policy;",
            "mod controls_declarative;",
            "mod controls_host_policy;",
            "mod controls_interaction_policy;",
            "mod controls_layout;",
            "mod controls_paint_plan;",
            "mod controls_policy;",
            "mod minimap_drag_policy;",
            "mod minimap_interaction_policy;",
            "mod minimap_declarative;",
            "mod minimap_navigation_policy;",
            "mod minimap_policy;",
            "mod minimap_projection;",
            "mod panel_item_state;",
            "mod panel_navigation_policy;",
            "mod panel_pointer_policy;",
            "mod rename_command;",
            "mod rename_host_layout;",
            "mod rename_lifecycle;",
            "mod rename_declarative;",
            "mod rename_policy;",
            "mod toolbar_layout_policy;",
            "mod toolbar_policy;",
            "mod toolbars_declarative;",
        ] {
            assert!(
                UI_OVERLAYS_MOD_RS.contains(module),
                "overlay policy module should compile outside compat-retained-canvas: {module}"
            );
        }

        assert!(!UI_OVERLAYS_MOD_RS.contains("mod panel_button_paint;"));
    }

    #[test]
    fn editor_chrome_compiles_without_retained_canvas_compat() {
        assert!(UI_MOD_RS.contains("mod editors;"));
        assert!(!UI_MOD_RS.contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod editors;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod chrome;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_command_policy;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_command_session;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_number;"));
        assert!(UI_EDITORS_MOD_RS.contains("mod portal_text;"));
        assert!(
            !UI_EDITORS_MOD_RS
                .contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod portal_number;")
        );
        assert!(
            !UI_EDITORS_MOD_RS
                .contains("#[cfg(feature = \"compat-retained-canvas\")]\nmod portal_text;")
        );

        for retained_bridge_free_editor in [UI_EDITOR_PORTAL_NUMBER_RS, UI_EDITOR_PORTAL_TEXT_RS] {
            assert!(
                !retained_bridge_free_editor.contains("retained_bridge")
                    && !retained_bridge_free_editor.contains("CommandCx"),
                "default portal editor modules must not depend on retained bridge command adapters"
            );
        }
    }

    #[test]
    fn default_overlay_policy_surfaces_stay_off_retained_bridge() {
        assert!(
            !UI_OVERLAY_CONTROLS_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_DECLARATIVE_RS.contains("Widget<"),
            "declarative controls composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_CONTROLS_HOST_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_HOST_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_HOST_POLICY_RS.contains("Widget<"),
            "default controls host policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_INTERACTION_POLICY_RS.contains("Widget<"),
            "default controls interaction policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_CONTROLS_PAINT_PLAN_RS.contains("retained_bridge")
                && !UI_OVERLAY_CONTROLS_PAINT_PLAN_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_CONTROLS_PAINT_PLAN_RS.contains("Widget<"),
            "default controls paint plan must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_BLACKBOARD_DECLARATIVE_RS.contains("Widget<"),
            "declarative blackboard composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_BLACKBOARD_INTERACTION_POLICY_RS.contains("Widget<"),
            "default blackboard interaction policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS.contains("retained_bridge")
                && !UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_BLACKBOARD_PAINT_PLAN_RS.contains("Widget<"),
            "default blackboard paint plan must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_MINIMAP_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_MINIMAP_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_MINIMAP_DECLARATIVE_RS.contains("Widget<"),
            "declarative minimap composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_MINIMAP_INTERACTION_POLICY_RS.contains("Widget<"),
            "default minimap interaction policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("retained_bridge")
                && !UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_TOOLBAR_LAYOUT_POLICY_RS.contains("Widget<"),
            "default toolbar layout policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_TOOLBARS_DECLARATIVE_RS.contains("Widget<"),
            "declarative toolbar composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_RENAME_DECLARATIVE_RS.contains("retained_bridge")
                && !UI_OVERLAY_RENAME_DECLARATIVE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_RENAME_DECLARATIVE_RS.contains("Widget<"),
            "declarative rename composition must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_RENAME_COMMAND_RS.contains("retained_bridge")
                && !UI_OVERLAY_RENAME_COMMAND_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_RENAME_COMMAND_RS.contains("Widget<"),
            "default rename command/session policy must not take a retained dependency"
        );
        assert!(
            !UI_OVERLAY_RENAME_LIFECYCLE_RS.contains("retained_bridge")
                && !UI_OVERLAY_RENAME_LIFECYCLE_RS.contains("RetainedSubtreeProps")
                && !UI_OVERLAY_RENAME_LIFECYCLE_RS.contains("Widget<"),
            "default rename lifecycle policy must not take a retained dependency"
        );
    }

    #[test]
    fn workflow_gallery_surface_stays_binding_first_for_viewport_controls() {
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.observe(cx);"));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("LayoutQueryRegionProps::default()"));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("node_graph_surface(cx, props)"));
        assert!(WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.set_viewport_action_host("));
        assert!(
            WORKFLOW_NODE_GRAPH_DEMO_RS.contains("binding.fit_view_nodes_in_bounds_action_host(")
        );
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
            FRET_DEMO_CARGO_TOML
                .contains("node-graph-demos = [\"fret-examples/node-graph-demos\"]")
        );
        assert!(FRET_EXAMPLES_LIB_RS.contains("pub mod node_graph_demo;"));
        assert!(NODE_GRAPH_DEMO_RS.contains("NodeGraphSurfaceBinding::new("));
        assert!(NODE_GRAPH_DEMO_RS.contains("node_graph_surface_in(cx, props)"));
        assert!(!NODE_GRAPH_DEMO_RS.contains("NodeGraphController"));
        assert!(!NODE_GRAPH_DEMO_RS.contains("binding.controller()"));
    }
}
