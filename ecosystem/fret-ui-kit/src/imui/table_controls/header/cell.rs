use std::sync::Arc;

use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, UiHost};

use super::resize::table_resize_handle;
use crate::imui::{TableColumn, TableColumnResizeResponse, TableOptions};

pub(super) fn wrap_table_header_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    content: AnyElement,
    test_id: Option<Arc<str>>,
    options: &TableOptions,
    resize_response: &mut TableColumnResizeResponse,
) -> AnyElement {
    let resize_handle = column.resize_options().map(|_| {
        let handle_test_id = test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.resize")));
        table_resize_handle(cx, column, column_index, resize_response, handle_test_id)
    });

    let mut cell = ContainerProps::default();
    cell.layout = super::super::cell::table_cell_layout(column.width(), options.clip_cells);

    let cell = cx.container(cell, move |cx| {
        let mut children = vec![content];
        if let Some(handle) = resize_handle {
            children.push(handle);
        }
        vec![
            crate::ui::h_flex(move |_cx| children)
                .gap_metric(crate::MetricRef::space(crate::Space::N0))
                .justify(crate::Justify::Start)
                .items(crate::Items::Stretch)
                .no_wrap()
                .into_element(cx),
        ]
    });

    if let Some(test_id) = test_id {
        cell.test_id(test_id)
    } else {
        cell
    }
}
