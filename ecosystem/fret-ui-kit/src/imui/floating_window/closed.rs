use fret_core::{Point, Size};
use fret_ui::GlobalElementId;

use super::super::{FloatingAreaResponse, FloatingWindowResponse};

pub(super) fn closed_floating_window_response(
    initial_position: Point,
    initial_size: Option<Size>,
) -> FloatingWindowResponse {
    FloatingWindowResponse {
        area: FloatingAreaResponse {
            id: GlobalElementId(0),
            rect: None,
            position: initial_position,
            dragging: false,
            drag_kind: super::super::float_window_drag_kind_for_element(GlobalElementId(0)),
        },
        size: initial_size,
        resizing: false,
        collapsed: false,
    }
}
