use std::sync::Arc;

use fret_ui::{ElementContext, UiHost};

use crate::imui::{TableColumn, TableColumnResizeResponse, TableOptions};

use super::super::cell::empty_cell;
use super::BuiltHeaderCell;
use super::cell::wrap_table_header_cell;
use super::labels::{table_header_content_box, table_header_label_text};
use super::trigger::{BuiltHeaderTrigger, header_trigger_surface};

pub(in crate::imui::table_controls) fn wrap_plain_header_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    visible_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    options: &TableOptions,
    resize_response: &mut TableColumnResizeResponse,
) -> BuiltHeaderCell {
    let column_key = column
        .id_arc()
        .unwrap_or_else(|| Arc::from(column_index.to_string()));
    let a11y_label = visible_label
        .clone()
        .or_else(|| column.id_arc())
        .or_else(|| Some(Arc::from(format!("Column {}", column_index + 1))));

    let BuiltHeaderTrigger {
        element: trigger_element,
        trigger,
    } = header_trigger_surface(
        cx,
        ("plain-header-cell", column_key),
        a11y_label,
        false,
        move |cx, _enabled, _state| {
            let content = visible_label
                .clone()
                .map(|label| table_header_label_text(cx, label))
                .unwrap_or_else(|| empty_cell(cx));
            vec![table_header_content_box(cx, content)]
        },
    );

    let element = wrap_table_header_cell(
        cx,
        column,
        column_index,
        trigger_element,
        test_id,
        options,
        resize_response,
    );

    BuiltHeaderCell { element, trigger }
}
