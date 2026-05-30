//! Shared facade-local support helpers for the `fret-ui-kit::imui` root hub.

mod constants;
mod geometry;
mod runtime;
mod slider_math;
mod state;
mod ui_writer;

pub(super) use constants::{
    DEFAULT_DISABLED_ALPHA, DEFAULT_DRAG_THRESHOLD_PX, DRAG_KIND_MASK, HOVER_DELAY_NORMAL,
    HOVER_DELAY_SHORT, HOVER_STATIONARY_DELAY, KEY_ACTIVATED, KEY_CHANGED, KEY_CLICKED,
    KEY_CONTEXT_MENU_REQUESTED, KEY_DEACTIVATED, KEY_DEACTIVATED_AFTER_EDIT, KEY_DOUBLE_CLICKED,
    KEY_DRAG_STARTED, KEY_DRAG_STOPPED, KEY_HOVER_DELAY_NORMAL_MET, KEY_HOVER_DELAY_SHORT_MET,
    KEY_HOVER_STATIONARY_MET, KEY_LONG_PRESSED, KEY_POINTER_CLICKED, KEY_SECONDARY_CLICKED,
    KEY_SELECT_ALL_ON_FOCUS, LONG_PRESS_DELAY, fnv1a64,
};
pub(super) use geometry::{
    point_add, point_sub, snap_point_to_device_pixels, snap_size_to_device_pixels,
};
pub(super) use runtime::prepare_imui_runtime_for_frame;
pub(super) use slider_math::{
    slider_clamp_and_snap, slider_normalize_range, slider_step_or_default,
    slider_value_from_pointer,
};
pub(super) use state::model_value_changed_for;
pub use ui_writer::UiWriterUiKitExt;
