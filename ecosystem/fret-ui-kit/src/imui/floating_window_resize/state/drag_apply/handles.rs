use fret_core::Point;

use super::super::super::super::{FloatWindowResizeHandle, FloatWindowState};
use super::bounds::ResizeDragBounds;
use corner::apply_corner_resize_handle_delta;
use edge::apply_edge_resize_handle_delta;

mod corner;
mod edge;

pub(super) fn apply_resize_handle_delta(
    st: &mut FloatWindowState,
    position: &mut Point,
    handle: FloatWindowResizeHandle,
    delta: Point,
    bounds: &ResizeDragBounds,
) {
    match handle {
        FloatWindowResizeHandle::Left
        | FloatWindowResizeHandle::Right
        | FloatWindowResizeHandle::Top
        | FloatWindowResizeHandle::Bottom => {
            apply_edge_resize_handle_delta(st, position, handle, delta, bounds);
        }
        FloatWindowResizeHandle::TopLeft
        | FloatWindowResizeHandle::TopRight
        | FloatWindowResizeHandle::BottomLeft
        | FloatWindowResizeHandle::BottomRight => {
            apply_corner_resize_handle_delta(st, position, handle, delta, bounds);
        }
    }
}
