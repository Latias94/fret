use std::sync::Arc;

use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, UiHost};

use crate::imui::label_identity::parse_label_identity;
use crate::imui::{TableColumn, TableSortDirection};

use super::super::cell::table_cell_padding;

pub(in crate::imui::table_controls) fn visible_header_label(
    column: &TableColumn,
) -> Option<Arc<str>> {
    column.header().map(|label| {
        let parts = parse_label_identity(label);
        Arc::<str>::from(parts.visible)
    })
}

pub(in crate::imui::table_controls) fn column_is_sortable(column: &TableColumn) -> bool {
    column.is_sortable()
}

fn sort_direction_indicator(direction: TableSortDirection) -> &'static str {
    match direction {
        TableSortDirection::Ascending => "^",
        TableSortDirection::Descending => "v",
    }
}

pub(in crate::imui::table_controls) fn table_sort_indicator_text<H: UiHost>(
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

pub(super) fn sortable_header_a11y_label(
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

pub(super) fn table_header_content_box<H: UiHost>(
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

pub(in crate::imui::table_controls) fn table_header_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
) -> AnyElement {
    crate::declarative::text::text_table_cell(cx, label)
}
