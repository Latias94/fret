use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::cell::empty_cell;
use crate::imui::{ResponseExt, TableColumn, TableColumnResizeResponse, TableOptions};

mod cell;
mod labels;
mod resize;
mod trigger;

use self::cell::wrap_table_header_cell;
pub(super) use labels::{
    column_is_sortable, table_header_label_text, table_sort_indicator_text, visible_header_label,
};
use labels::{sortable_header_a11y_label, table_header_content_box};
use trigger::{BuiltHeaderTrigger, header_trigger_surface, sortable_header_visual};

pub(super) struct BuiltHeaderCell {
    pub(super) element: AnyElement,
    pub(super) trigger: ResponseExt,
}

pub(super) fn wrap_sortable_header_cell<H: UiHost>(
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

pub(super) fn wrap_plain_header_cell<H: UiHost>(
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
