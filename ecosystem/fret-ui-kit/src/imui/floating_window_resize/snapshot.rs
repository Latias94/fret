use fret_core::Point;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Clone, Copy)]
pub(in crate::imui) struct FloatingWindowResizeSnapshot {
    pub(super) handle: super::super::FloatWindowResizeHandle,
    pub(super) dragging: bool,
    pub(super) position: Point,
    pub(super) start_position: Point,
}

pub(in crate::imui) fn current_resize_snapshot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    resize_enabled: bool,
) -> Option<FloatingWindowResizeSnapshot> {
    if !resize_enabled {
        return None;
    }

    [
        super::super::FloatWindowResizeHandle::Left,
        super::super::FloatWindowResizeHandle::Right,
        super::super::FloatWindowResizeHandle::Top,
        super::super::FloatWindowResizeHandle::Bottom,
        super::super::FloatWindowResizeHandle::TopLeft,
        super::super::FloatWindowResizeHandle::TopRight,
        super::super::FloatWindowResizeHandle::BottomLeft,
        super::super::FloatWindowResizeHandle::BottomRight,
    ]
    .into_iter()
    .find_map(|handle| {
        let kind = super::super::float_window_resize_kind_for_element(window_id, handle);
        cx.app
            .find_drag_pointer_id(|d| {
                d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
            })
            .and_then(|pointer_id| cx.app.drag(pointer_id))
            .filter(|drag| drag.kind == kind)
            .map(|drag| FloatingWindowResizeSnapshot {
                handle,
                dragging: drag.dragging,
                position: drag.position,
                start_position: drag.start_position,
            })
    })
}
