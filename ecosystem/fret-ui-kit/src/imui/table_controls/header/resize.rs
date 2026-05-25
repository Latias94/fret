use std::sync::Arc;

use fret_core::{CursorIcon, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length, PointerRegionProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::imui::{
    DragResponse, ResponseExt, TableColumn, TableColumnResizeResponse, drag_kind_for_element,
    drag_threshold_for, finish_pointer_region_drag, handle_pointer_region_drag_move_with_threshold,
    imui_is_disabled, populate_pressable_drag_response, prepare_pointer_region_drag_on_left_down,
};

const TABLE_RESIZE_HANDLE_HIT_WIDTH: Px = Px(12.0);
const TABLE_RESIZE_HANDLE_MIN_HEIGHT: Px = Px(24.0);
const TABLE_RESIZE_HANDLE_VISUAL_WIDTH: Px = Px(1.0);

#[derive(Default)]
struct TableResizeHandleDragState {
    was_dragging: bool,
}

pub(super) fn table_resize_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    response: &mut TableColumnResizeResponse,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let column_key = column
        .id_arc()
        .or_else(|| column.header_arc())
        .unwrap_or_else(|| Arc::from(format!("column-{column_index}")));
    let enabled = !imui_is_disabled(cx);
    response.enabled = enabled;

    let handle = cx.keyed(("table-column-resize", column_key, column_index), |cx| {
        let mut props = PointerRegionProps::default();
        props.enabled = enabled;
        props.layout.size.width = Length::Px(TABLE_RESIZE_HANDLE_HIT_WIDTH);
        props.layout.size.height = Length::Auto;
        props.layout.size.min_height = Some(Length::Px(TABLE_RESIZE_HANDLE_MIN_HEIGHT));
        props.layout.flex.shrink = 0.0;

        cx.pointer_region(props, move |cx| {
            let region_id = cx.root_id();
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
                handle_pointer_region_drag_move_with_threshold(
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
                finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
            }));

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

            vec![table_resize_handle_visual(cx, enabled)]
        })
    });

    if let Some(test_id) = test_id {
        handle.test_id(test_id)
    } else {
        handle
    }
}

fn table_resize_handle_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let mut color = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"));
    if !enabled {
        color.a *= 0.45;
    }

    let mut grip = ContainerProps::default();
    grip.background = Some(color);
    grip.layout.size.width = Length::Px(TABLE_RESIZE_HANDLE_VISUAL_WIDTH);
    grip.layout.size.height = Length::Px(TABLE_RESIZE_HANDLE_MIN_HEIGHT);
    grip.layout.flex.shrink = 0.0;

    crate::ui::h_flex(move |cx| vec![cx.container(grip, |_cx| Vec::new())])
        .gap_metric(crate::MetricRef::space(crate::Space::N0))
        .justify(crate::Justify::Center)
        .items(crate::Items::Stretch)
        .no_wrap()
        .into_element(cx)
}
