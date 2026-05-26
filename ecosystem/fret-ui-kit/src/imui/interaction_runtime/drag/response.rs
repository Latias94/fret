use fret_core::Point;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Default)]
struct DragReportState {
    last_position: Option<Point>,
}

pub(in crate::imui) fn populate_pressable_drag_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    response: &mut crate::imui::ResponseExt,
) {
    let started = cx.take_transient_for(id, crate::imui::KEY_DRAG_STARTED);
    let stopped = cx.take_transient_for(id, crate::imui::KEY_DRAG_STOPPED);
    response.drag_mut().set_started(started);
    response.drag_mut().set_stopped(stopped);
    response.drag_mut().clear();

    let kind = super::drag_kind_for_element(id);
    let pointer_id = cx.app.find_drag_pointer_id(|d| {
        d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
    });
    let drag_snapshot = pointer_id.and_then(|pointer_id| {
        cx.app
            .drag(pointer_id)
            .filter(|drag| drag.kind == kind)
            .map(|drag| (drag.dragging, drag.position, drag.start_position))
    });
    if let Some((dragging, current, start)) = drag_snapshot {
        response.drag_mut().set_dragging(dragging);
        let (delta, total) = cx.state_for(id, DragReportState::default, |st| {
            let prev = st.last_position.unwrap_or(current);
            st.last_position = Some(current);
            (
                crate::imui::point_sub(current, prev),
                crate::imui::point_sub(current, start),
            )
        });
        if dragging {
            response.drag_mut().set_motion(delta, total);
        }
    } else {
        cx.state_for(id, DragReportState::default, |st| {
            st.last_position = None;
        });
    }
}
