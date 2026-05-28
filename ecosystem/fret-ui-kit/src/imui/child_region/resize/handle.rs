use std::sync::Arc;

use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

use crate::imui::{DragResponse, ResponseExt};

use super::axis::ChildRegionResizeAxis;

#[derive(Default)]
struct ChildRegionResizeDragState {
    was_dragging: bool,
}

pub(super) fn child_region_resize_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    axis: ChildRegionResizeAxis,
    enabled: bool,
    test_id: Option<Arc<str>>,
    drag: &mut DragResponse,
) -> AnyElement {
    let handle = cx.keyed((axis.key(), id), |cx| {
        let props = PointerRegionProps {
            enabled,
            layout: axis.layout(),
            ..Default::default()
        };

        cx.pointer_region(props, move |cx| {
            let region_id = cx.root_id();
            let drag_kind = super::super::super::drag_kind_for_element(region_id);
            let drag_threshold = super::super::super::drag_threshold_for(cx);
            let cursor = axis.cursor();

            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                super::super::super::prepare_pointer_region_drag_on_left_down(
                    host,
                    acx,
                    down,
                    enabled.then_some(drag_kind),
                    Some(cursor),
                )
            }));
            cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                if !enabled {
                    return false;
                }
                host.set_cursor_icon(cursor);
                super::super::super::handle_pointer_region_drag_move_with_threshold(
                    host,
                    acx,
                    mv,
                    drag_kind,
                    drag_threshold,
                )
            }));
            cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                if !enabled {
                    return false;
                }
                super::super::super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
            }));

            let mut drag_response = ResponseExt::default();
            super::super::super::populate_pressable_drag_response(
                cx,
                region_id,
                &mut drag_response,
            );
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

            Vec::new()
        })
    });

    if let Some(test_id) = test_id {
        handle.test_id(test_id)
    } else {
        handle
    }
}
