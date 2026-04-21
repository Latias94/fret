//! Internal immediate-mode interaction runtime helpers.

mod disabled;
mod drag;
mod hover;
mod lifecycle;
mod models;

pub(super) use disabled::{
    DisabledScopeGuard, disabled_alpha_for, imui_is_disabled, sanitize_response_for_enabled,
};
pub(super) use drag::{
    clear_active_item_on_left_pointer_up, drag_kind_for_element, drag_threshold_for,
    finish_pointer_region_drag, finish_pressable_drag_on_pointer_up,
    handle_pointer_region_drag_move_with_threshold, handle_pressable_drag_move_with_threshold,
    mark_active_item_on_left_pointer_down, populate_pressable_drag_response,
    prepare_pointer_region_drag_on_left_down, prepare_pressable_drag_on_pointer_down,
};
#[allow(unused_imports)]
pub(super) use hover::{
    HoverQueryDelayRead, hover_blocked_by_active_item_for, install_hover_query_hooks_for_pressable,
};
pub(super) use lifecycle::{
    mark_lifecycle_activated_on_left_pointer_down, mark_lifecycle_deactivated_on_left_pointer_up,
    mark_lifecycle_edit, mark_lifecycle_instant_if_inactive,
    populate_response_lifecycle_from_active_state, populate_response_lifecycle_transients,
};
pub(super) use models::{
    ImUiActiveItemState, ImUiLifecycleSessionState, LongPressSignalState,
    active_item_model_for_window, context_menu_anchor_model_for, disabled_scope_depth_for,
    float_window_collapsed_model_for, lifecycle_session_model_for, long_press_signal_model_for,
    pointer_click_modifiers_model_for,
};
