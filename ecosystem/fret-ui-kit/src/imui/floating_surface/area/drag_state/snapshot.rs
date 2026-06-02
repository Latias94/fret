use fret_core::Point;
use fret_ui::{ElementContext, UiHost};

#[derive(Clone, Copy)]
pub(super) struct FloatingAreaDragSnapshot {
    pub(super) dragging: bool,
    pub(super) position: Point,
    pub(super) start_position: Point,
}

pub(super) fn floating_area_drag_snapshot<H: UiHost>(
    cx: &ElementContext<'_, H>,
    drag_kind: fret_runtime::DragKindId,
) -> Option<FloatingAreaDragSnapshot> {
    cx.app
        .find_drag_pointer_id(|drag| {
            drag.kind == drag_kind
                && drag.source_window == cx.window
                && drag.current_window == cx.window
        })
        .and_then(|pointer_id| cx.app.drag(pointer_id))
        .filter(|drag| drag.kind == drag_kind)
        .map(|drag| FloatingAreaDragSnapshot {
            dragging: drag.dragging,
            position: drag.position,
            start_position: drag.start_position,
        })
}
