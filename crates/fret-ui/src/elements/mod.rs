mod access;
mod cx;
mod hash;
mod id;
mod queries;
mod runtime;

pub use access::{dismissible_has_pointer_move_handler, take_element_state, with_element_state};
pub use cx::ElementContext;
pub use hash::global_root;
pub use id::GlobalElementId;
pub use queries::{
    bounds_for_element, element_is_live_in_current_frame, node_for_element, peek_node_for_element,
    root_bounds_for_element, visual_bounds_for_element, with_element_cx,
};
pub use runtime::{ContinuousFrames, ElementRuntime, WindowElementState};
#[cfg(feature = "diagnostics")]
pub use runtime::{NodeEntryRootOverwrite, WindowElementDiagnosticsSnapshot};

pub(crate) use access::{
    clear_timer_target, is_pressed_pressable, observed_globals_for_element,
    observed_models_for_element, record_timer_target, set_pressed_pressable, timer_has_target,
    timer_target_node, update_hovered_hover_region, update_hovered_pressable, with_window_state,
};
pub(crate) use queries::{record_bounds_for_element, record_visual_bounds_for_element};
pub(crate) use runtime::{ActiveTextSelection, NodeEntry};
