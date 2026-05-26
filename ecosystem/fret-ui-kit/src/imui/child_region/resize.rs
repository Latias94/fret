use std::sync::Arc;

use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

use crate::imui::{
    ChildRegionResizeXOptions, ChildRegionResizeYOptions, ChildRegionResponse, DragResponse,
    ResponseExt,
};

mod axis;

use axis::ChildRegionResizeAxis;

#[derive(Default)]
struct ChildRegionResizeDragState {
    was_dragging: bool,
}

pub(super) fn child_region_resize_x_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    options: ChildRegionResizeXOptions,
    test_id: Option<Arc<str>>,
    response: &mut ChildRegionResponse,
) -> AnyElement {
    let enabled = !super::super::imui_is_disabled(cx);
    let resize_response = response.resize_x_mut();
    resize_response.enabled = enabled;
    resize_response.min_width = options.min_width;
    resize_response.max_width = options.max_width;

    child_region_resize_handle(
        cx,
        id,
        ChildRegionResizeAxis::X,
        enabled,
        test_id,
        &mut resize_response.drag,
    )
}

pub(super) fn child_region_resize_y_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    options: ChildRegionResizeYOptions,
    test_id: Option<Arc<str>>,
    response: &mut ChildRegionResponse,
) -> AnyElement {
    let enabled = !super::super::imui_is_disabled(cx);
    let resize_response = response.resize_y_mut();
    resize_response.enabled = enabled;
    resize_response.min_height = options.min_height;
    resize_response.max_height = options.max_height;

    child_region_resize_handle(
        cx,
        id,
        ChildRegionResizeAxis::Y,
        enabled,
        test_id,
        &mut resize_response.drag,
    )
}

fn child_region_resize_handle<H: UiHost>(
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
            let drag_kind = super::super::drag_kind_for_element(region_id);
            let drag_threshold = super::super::drag_threshold_for(cx);
            let cursor = axis.cursor();

            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                super::super::prepare_pointer_region_drag_on_left_down(
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
                super::super::handle_pointer_region_drag_move_with_threshold(
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
                super::super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
            }));

            let mut drag_response = ResponseExt::default();
            super::super::populate_pressable_drag_response(cx, region_id, &mut drag_response);
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
