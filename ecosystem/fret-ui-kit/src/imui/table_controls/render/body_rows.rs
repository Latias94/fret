use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::super::{BuiltTableCell, BuiltTableRow, TableColumn, TableOptions, body, cell};

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
                let mut iter = row.cells.into_iter();
                let mut cells =
                    Vec::with_capacity(columns.iter().filter(|column| column.visible()).count());
                for (column_index, column) in columns.iter().enumerate() {
                    let built = iter.next().unwrap_or_else(|| BuiltTableCell {
                        test_id: None,
                        explicit_test_id: None,
                        background: None,
                        content: cell::empty_cell(cx),
                    });
                    if !column.visible() {
                        continue;
                    }
                    let default_test_id = row
                        .test_id
                        .as_ref()
                        .map(|base| {
                            Arc::from(format!(
                                "{base}.cell.{}",
                                column_test_id_suffixes[column_index]
                            ))
                        })
                        .or(built.test_id);
                    let test_id = built.explicit_test_id.or(default_test_id);
                    cells.push(body::PreparedTableCell {
                        column: column.clone(),
                        element: body::wrap_table_cell(
                            cx,
                            column,
                            built.content,
                            test_id,
                            false,
                            built.background,
                            options,
                        ),
                    });
                }
                debug_assert!(
                    iter.next().is_none(),
                    "imui table rows must emit exactly one cell per declared column"
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
