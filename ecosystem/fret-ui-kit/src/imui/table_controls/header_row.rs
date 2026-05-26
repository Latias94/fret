use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::{body, header};
use crate::imui::{TableColumn, TableColumnResizeResponse, TableHeaderResponse, TableOptions};

pub(super) fn render_table_header<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: &[TableColumn],
    column_test_id_suffixes: &[String],
    root_test_id: Option<&Arc<str>>,
    palette: &body::TablePalette,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
    header_responses: &mut Vec<TableHeaderResponse>,
) -> AnyElement {
    cx.keyed(format!("{id}.header"), |cx| {
        let cells = columns
            .iter()
            .enumerate()
            .filter(|(_, column)| column.visible())
            .map(|(index, column)| {
                let visible_label = header::visible_header_label(column);
                let test_id = root_test_id.map(|base| {
                    Arc::from(format!(
                        "{base}.header.cell.{}",
                        column_test_id_suffixes[index]
                    ))
                });
                let sortable = header::column_is_sortable(column);
                let resize_options = column.resize_options();
                let mut resize = TableColumnResizeResponse {
                    column_index: index,
                    column_id: column.id_arc(),
                    enabled: resize_options.is_some(),
                    min_width: resize_options.and_then(|options| options.min_width),
                    max_width: resize_options.and_then(|options| options.max_width),
                    drag: Default::default(),
                };
                let built = if sortable {
                    header::wrap_sortable_header_cell(
                        cx,
                        column,
                        index,
                        visible_label.clone(),
                        test_id,
                        options,
                        &mut resize,
                    )
                } else {
                    header::wrap_plain_header_cell(
                        cx,
                        column,
                        index,
                        visible_label,
                        test_id,
                        options,
                        &mut resize,
                    )
                };
                header_responses.push(TableHeaderResponse {
                    column_index: index,
                    column_id: column.id_arc(),
                    sortable,
                    sort_direction: column.sort_direction(),
                    trigger: built.trigger,
                    resize,
                });
                body::PreparedTableCell {
                    column: column.clone(),
                    element: built.element,
                }
            })
            .collect::<Vec<_>>();
        body::wrap_table_row(
            cx,
            cells,
            root_test_id.map(|base| Arc::from(format!("{base}.header"))),
            true,
            false,
            None,
            palette,
            options,
            scroll_x.clone(),
        )
    })
}
