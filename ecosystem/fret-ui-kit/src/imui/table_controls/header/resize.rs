use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{TableColumn, TableColumnResizeResponse, imui_is_disabled};

mod behavior;
mod props;
mod visual;

use visual::table_resize_handle_visual;

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
        cx.pointer_region(props::table_resize_handle_props(enabled), move |cx| {
            let region_id = cx.root_id();
            behavior::install_table_resize_handle_drag(cx, region_id, enabled);
            behavior::populate_table_resize_drag_response(cx, region_id, response);

            vec![table_resize_handle_visual(cx, enabled)]
        })
    });

    if let Some(test_id) = test_id {
        handle.test_id(test_id)
    } else {
        handle
    }
}
