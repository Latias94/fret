mod access;
mod cx;
mod hash;
mod id;
mod queries;
mod runtime;

pub use access::{take_element_state, with_element_state};
pub use cx::ElementCx;
pub use hash::global_root;
pub use id::GlobalElementId;
pub use queries::{
    bounds_for_element, node_for_element, root_bounds_for_element, visual_bounds_for_element,
    with_element_cx,
};
pub use runtime::{ContinuousFrames, ElementRuntime, WindowElementState};

pub(crate) use access::{
    is_pressed_pressable, observed_globals_for_element, observed_models_for_element,
    set_pressed_pressable, update_hovered_hover_region, update_hovered_pressable,
    with_window_state,
};
pub(crate) use queries::{record_bounds_for_element, record_visual_bounds_for_element};
pub(crate) use runtime::NodeEntry;
