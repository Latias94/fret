use std::sync::Arc;

use fret_core::CursorIcon;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    DragResponse, ResponseExt, TableColumnResizeResponse, drag_kind_for_element,
    drag_threshold_for, finish_pointer_region_drag, handle_pointer_region_drag_move_with_threshold,
    populate_pressable_drag_response, prepare_pointer_region_drag_on_left_down,
};

#[derive(Default)]
struct TableResizeHandleDragState {
    was_dragging: bool,
}

pub(super) fn install_table_resize_handle_drag<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    enabled: bool,
) {
    let drag_kind = drag_kind_for_element(region_id);
    let drag_threshold = drag_threshold_for(cx);

    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
        prepare_pointer_region_drag_on_left_down(
            host,
            acx,
            down,
            enabled.then_some(drag_kind),
            Some(CursorIcon::ColResize),
        )
    }));
    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
        if !enabled {
            return false;
        }
        host.set_cursor_icon(CursorIcon::ColResize);
        handle_pointer_region_drag_move_with_threshold(host, acx, mv, drag_kind, drag_threshold)
    }));
    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
        if !enabled {
            return false;
        }
        finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
    }));
}

pub(super) fn populate_table_resize_drag_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    response: &mut TableColumnResizeResponse,
) {
    let mut drag_response = ResponseExt::default();
    populate_pressable_drag_response(cx, region_id, &mut drag_response);
    response.drag = drag_response.drag();
    let dragging = response.drag.dragging();
    let (started, stopped) =
        cx.state_for(region_id, TableResizeHandleDragState::default, |state| {
            let started = dragging && !state.was_dragging;
            let stopped = !dragging && state.was_dragging;
            state.was_dragging = dragging;
            (started, stopped)
        });
    response.drag.merge_edges({
        let mut edges = DragResponse::default();
        edges.set_started(started);
        edges.set_stopped(stopped);
        edges
    });
}
