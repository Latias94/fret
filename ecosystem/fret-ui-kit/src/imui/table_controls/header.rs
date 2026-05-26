use std::sync::Arc;

use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, UiHost};

use super::cell::{empty_cell, table_cell_layout, table_cell_padding};
use crate::imui::label_identity::parse_label_identity;
use crate::imui::{
    ResponseExt, TableColumn, TableColumnResizeResponse, TableOptions, TableSortDirection,
};

mod resize;
mod trigger;

use resize::table_resize_handle;
use trigger::{BuiltHeaderTrigger, header_trigger_surface, sortable_header_visual};

pub(super) struct BuiltHeaderCell {
    pub(super) element: AnyElement,
    pub(super) trigger: ResponseExt,
}

pub(super) fn visible_header_label(column: &TableColumn) -> Option<Arc<str>> {
    column.header().map(|label| {
        let parts = parse_label_identity(label);
        Arc::<str>::from(parts.visible)
    })
}

pub(super) fn column_is_sortable(column: &TableColumn) -> bool {
    column.is_sortable()
}

fn sort_direction_indicator(direction: TableSortDirection) -> &'static str {
    match direction {
        TableSortDirection::Ascending => "^",
        TableSortDirection::Descending => "v",
    }
}

pub(super) fn table_sort_indicator_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    direction: TableSortDirection,
) -> AnyElement {
    crate::declarative::text::text_chrome_glyph(
        cx,
        Arc::<str>::from(sort_direction_indicator(direction)),
    )
}

fn sort_direction_a11y_label(direction: TableSortDirection) -> &'static str {
    match direction {
        TableSortDirection::Ascending => "ascending",
        TableSortDirection::Descending => "descending",
    }
}

fn sortable_header_a11y_label(
    column: &TableColumn,
    visible_label: Option<&Arc<str>>,
    column_index: usize,
) -> Arc<str> {
    let base = visible_label
        .cloned()
        .or_else(|| column.id_arc())
        .unwrap_or_else(|| Arc::from(format!("Column {}", column_index + 1)));
    match column.sort_direction() {
        Some(direction) => Arc::from(format!(
            "{base}, sorted {}",
            sort_direction_a11y_label(direction)
        )),
        None => Arc::from(format!("{base}, sortable")),
    }
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

fn table_header_content_box<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: AnyElement,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.layout.flex.grow = 1.0;
    props.layout.flex.shrink = 1.0;
    props.padding = table_cell_padding().into();
    cx.container(props, move |_cx| vec![content])
}

pub(super) fn table_header_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
) -> AnyElement {
    crate::declarative::text::text_table_cell(cx, label)
}

fn wrap_table_header_cell<H: UiHost>(
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
    cell.layout = table_cell_layout(column.width(), options.clip_cells);

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
