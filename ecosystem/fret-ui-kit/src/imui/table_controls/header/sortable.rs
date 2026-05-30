use std::sync::Arc;

use fret_ui::{ElementContext, UiHost};

use crate::imui::{TableColumn, TableColumnResizeResponse, TableOptions};

use super::BuiltHeaderCell;
use super::cell::wrap_table_header_cell;
use super::labels::sortable_header_a11y_label;
use super::trigger::{BuiltHeaderTrigger, header_trigger_surface, sortable_header_visual};

pub(in crate::imui::table_controls) fn wrap_sortable_header_cell<H: UiHost>(
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
    let sort_direction = column.sort_direction();
    let a11y_label = sortable_header_a11y_label(column, visible_label.as_ref(), column_index);

    let BuiltHeaderTrigger {
        element: trigger_element,
        trigger,
    } = header_trigger_surface(
        cx,
        ("sortable-header-cell", column_key),
        Some(a11y_label),
        true,
        move |cx, enabled, state| {
            vec![sortable_header_visual(
                cx,
                visible_label.clone(),
                sort_direction,
                enabled,
                state,
            )]
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
