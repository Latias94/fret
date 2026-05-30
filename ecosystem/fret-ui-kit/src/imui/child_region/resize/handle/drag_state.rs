use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{DragResponse, ResponseExt, populate_pressable_drag_response};

#[derive(Default)]
struct ChildRegionResizeDragState {
    was_dragging: bool,
}

pub(super) fn populate_child_region_resize_drag_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    drag: &mut DragResponse,
) {
    let mut drag_response = ResponseExt::default();
    populate_pressable_drag_response(cx, region_id, &mut drag_response);
    *drag = drag_response.drag();

    let dragging = drag.dragging();
    let (started, stopped) =
        cx.state_for(region_id, ChildRegionResizeDragState::default, |state| {
            let started = dragging && !state.was_dragging;
            let stopped = !dragging && state.was_dragging;
            state.was_dragging = dragging;
            (started, stopped)
        });
    drag.merge_edges({
        let mut edges = DragResponse::default();
        edges.set_started(started);
        edges.set_stopped(stopped);
        edges
    });
}
