//! Drag-and-drop integration glue for `fret-dnd`.
//!
//! This module intentionally stays UI-kit scoped:
//! - `fret-dnd` remains a headless policy toolbox (sensors/collisions/modifiers).
//! - `fret-ui-kit::dnd` provides per-window, per-frame registries and controller helpers that
//!   translate app events into headless updates.
//!
//! Coordinate contract: when used with `fret-ui` events/layout, the canonical space is
//! **window-local logical pixels** (ADR 0017 / ADR 0149 / ADR 0157).

mod activation_handoff;
mod activation_probe;
mod controller;
mod forwarders;
mod registry;
mod service;
mod types;

pub use activation_handoff::{
    complete_cross_window_drag_activation, complete_cross_window_drag_activation_for_action_host,
    try_begin_cross_window_drag_on_activation,
};
pub use activation_probe::{DndActivationProbe, DndActivationProbeConfig};
pub use controller::{
    clear_pointer, clear_pointer_default_scope, clear_pointer_in_scope, handle_pointer_cancel,
    handle_pointer_cancel_default_scope, handle_pointer_cancel_in_scope, handle_pointer_down,
    handle_pointer_down_default_scope, handle_pointer_down_in_scope, handle_pointer_move,
    handle_pointer_move_default_scope, handle_pointer_move_in_scope,
    handle_pointer_move_or_init_in_scope, handle_pointer_up, handle_pointer_up_default_scope,
    handle_pointer_up_in_scope, handle_sensor_move_or_init_in_scope,
    pointer_is_tracking_any_sensor,
};
pub use forwarders::{
    DndPointerActivationConstraints, DndPointerForwarders, DndPointerForwardersConfig,
};
pub use registry::{
    droppable_rect_in_scope, register_draggable_rect, register_draggable_rect_default_scope,
    register_draggable_rect_in_scope, register_droppable_rect,
    register_droppable_rect_default_scope, register_droppable_rect_in_scope,
};
pub use service::{DndServiceModel, dnd_service_model, dnd_service_model_global};
pub use types::{DND_SCOPE_DEFAULT, DndScopeId, DndUpdate};

pub use fret_dnd::{
    ActivationConstraint, AutoScrollConfig, AutoScrollRequest, Axis, CollisionStrategy,
    DndCollision, DndItemId, InsertionSide, SensorOutput, insertion_side_for_pointer,
};

#[cfg(test)]
mod tests;
