use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::body;
use crate::imui::{TableColumn, TableHeaderResponse, TableOptions};
use cells::build_header_cells;

mod cells;

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
        let cells = build_header_cells(
            cx,
            columns,
            column_test_id_suffixes,
            root_test_id,
            options,
            header_responses,
        );
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
