mod area;
mod drag_surface;
mod kinds;
mod layer;
mod state;

pub(super) use area::floating_area_element;
pub(super) use drag_surface::floating_area_drag_surface_element;
pub(super) use kinds::{
    FloatWindowResizeHandle, KEY_FLOAT_WINDOW_ACTIVATE, KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED,
    OnFloatingAreaLeftDoubleClick, float_window_drag_kind_for_element,
    float_window_resize_kind_for_element,
};
pub(super) use layer::{float_layer_bring_to_front_if_activated, floating_layer_element};
pub(super) use state::{FloatWindowState, FloatingAreaState, FloatingWindowChromeResponse};
