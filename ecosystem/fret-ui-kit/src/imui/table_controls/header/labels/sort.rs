use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{TableColumn, TableSortDirection};

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

pub(in crate::imui::table_controls::header) fn sortable_header_a11y_label(
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
