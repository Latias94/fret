use std::sync::Arc;

use fret_ui::UiHost;

use super::{BuiltTableCell, BuiltTableRow, ImUiTable, ImUiTableRow, test_ids};
use crate::imui::TableRowOptions;

impl<'cx, 'a, H: UiHost> ImUiTable<'cx, 'a, H> {
    pub fn row(
        &mut self,
        key: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTableRow<'cx2, 'a2, H>),
    ) {
        self.row_with_options(key, TableRowOptions::default(), f);
    }

    pub fn row_with_options(
        &mut self,
        key: impl Into<Arc<str>>,
        options: TableRowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTableRow<'cx2, 'a2, H>),
    ) {
        let key = key.into();
        let row_index = self.rows.len();
        let row_test_id =
            test_ids::row_test_id(options.test_id, self.root_test_id.as_ref(), row_index);
        let mut cells = Vec::<BuiltTableCell>::new();
        let build_focus = self.build_focus.clone();
        self.cx.keyed(key.clone(), |cx| {
            let mut row = ImUiTableRow {
                cx,
                cells: &mut cells,
                row_test_id: row_test_id.clone(),
                build_focus,
            };
            f(&mut row);
        });
        self.rows.push(BuiltTableRow {
            key,
            test_id: row_test_id,
            background: options.background,
            cells,
        });
    }
}
