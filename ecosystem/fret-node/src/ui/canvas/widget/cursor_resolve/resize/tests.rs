use super::*;

#[test]
fn resize_handle_cursor_icon_matches_axis_handles() {
    assert_eq!(
        cursor_icon_for_resize_handle(NodeResizeHandle::Top),
        CursorIcon::RowResize
    );
    assert_eq!(
        cursor_icon_for_resize_handle(NodeResizeHandle::Bottom),
        CursorIcon::RowResize
    );
    assert_eq!(
        cursor_icon_for_resize_handle(NodeResizeHandle::Left),
        CursorIcon::ColResize
    );
    assert_eq!(
        cursor_icon_for_resize_handle(NodeResizeHandle::Right),
        CursorIcon::ColResize
    );
}
