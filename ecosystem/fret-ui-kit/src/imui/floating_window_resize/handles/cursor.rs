use fret_core::CursorIcon;

use super::super::super::FloatWindowResizeHandle;

pub(super) fn resize_handle_cursor(handle: FloatWindowResizeHandle) -> CursorIcon {
    match handle {
        FloatWindowResizeHandle::Left | FloatWindowResizeHandle::Right => CursorIcon::ColResize,
        FloatWindowResizeHandle::Top | FloatWindowResizeHandle::Bottom => CursorIcon::RowResize,
        FloatWindowResizeHandle::TopLeft | FloatWindowResizeHandle::BottomRight => {
            CursorIcon::NwseResize
        }
        FloatWindowResizeHandle::TopRight | FloatWindowResizeHandle::BottomLeft => {
            CursorIcon::NeswResize
        }
    }
}
