//! Hue-wheel color model geometry and interaction helpers.

mod geometry;
mod interaction;
mod triangle;

pub(in crate::controls::color_edit) use geometry::{HueWheelGeometry, hue_wheel_geometry};
pub(in crate::controls::color_edit) use interaction::{
    HueWheelDragTarget, hsv_with_hue_wheel_position, hue_wheel_target_from_local_position,
};
pub(in crate::controls::color_edit) use triangle::{
    HueWheelTriangle, hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position,
};
