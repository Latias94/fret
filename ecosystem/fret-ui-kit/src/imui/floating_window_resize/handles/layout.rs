use fret_ui::element::LayoutStyle;

use super::super::super::FloatWindowResizeHandle;

mod corner;
mod edge;

pub(super) fn resize_handle_layout(handle: FloatWindowResizeHandle) -> LayoutStyle {
    match handle {
        FloatWindowResizeHandle::Left
        | FloatWindowResizeHandle::Right
        | FloatWindowResizeHandle::Top
        | FloatWindowResizeHandle::Bottom => edge::edge_resize_handle_layout(handle),
        FloatWindowResizeHandle::TopLeft
        | FloatWindowResizeHandle::TopRight
        | FloatWindowResizeHandle::BottomLeft
        | FloatWindowResizeHandle::BottomRight => corner::corner_resize_handle_layout(handle),
    }
}
