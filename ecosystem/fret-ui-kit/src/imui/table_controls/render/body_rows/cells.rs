use std::sync::Arc;

use fret_ui::{ElementContext, UiHost};

use super::super::super::{BuiltTableCell, TableColumn, TableOptions, body, cell};

pub(super) struct BodyRowCellsInput<'a> {
    pub(super) columns: &'a [TableColumn],
    pub(super) cells: Vec<BuiltTableCell>,
    pub(super) row_test_id: Option<Arc<str>>,
    pub(super) column_test_id_suffixes: &'a [String],
    pub(super) options: &'a TableOptions,
}

pub(super) fn prepare_body_row_cells<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: BodyRowCellsInput<'_>,
) -> Vec<body::PreparedTableCell> {
    let BodyRowCellsInput {
        columns,
        cells,
        row_test_id,
        column_test_id_suffixes,
        options,
    } = input;

    let mut iter = cells.into_iter();
    let mut prepared = Vec::with_capacity(columns.iter().filter(|column| column.visible()).count());
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
        let default_test_id = row_test_id
            .as_ref()
            .map(|base| {
                Arc::from(format!(
                    "{base}.cell.{}",
                    column_test_id_suffixes[column_index]
                ))
            })
            .or(built.test_id);
        let test_id = built.explicit_test_id.or(default_test_id);
        prepared.push(body::PreparedTableCell {
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

    prepared
}
