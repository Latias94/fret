use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::super::{BuiltTableRow, TableColumn, TableOptions, body};
use cells::{BodyRowCellsInput, prepare_body_row_cells};

mod cells;

pub(super) fn render_table_body_rows<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    columns: &[TableColumn],
    rows: Vec<BuiltTableRow>,
    column_test_id_suffixes: &[String],
    palette: &body::TablePalette,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> Vec<AnyElement> {
    rows.into_iter()
        .enumerate()
        .map(|(row_index, row)| {
            let striped = options.striped && row_index % 2 == 1;
            cx.keyed(row.key.clone(), |cx| {
                let cells = prepare_body_row_cells(
                    cx,
                    BodyRowCellsInput {
                        columns,
                        cells: row.cells,
                        row_test_id: row.test_id.clone(),
                        column_test_id_suffixes,
                        options,
                    },
                );
                body::wrap_table_row(
                    cx,
                    cells,
                    row.test_id,
                    false,
                    striped,
                    row.background,
                    palette,
                    options,
                    scroll_x.clone(),
                )
            })
        })
        .collect()
}
