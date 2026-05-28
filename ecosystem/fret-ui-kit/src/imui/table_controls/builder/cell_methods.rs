use std::sync::Arc;

use fret_ui::UiHost;

use super::{BuiltTableCell, ImUiTableRow, test_ids};
use crate::imui::{ImUiFacade, TableCellOptions, containers::build_imui_children_with_focus};

impl<'cx, 'a, H: UiHost> ImUiTableRow<'cx, 'a, H> {
    pub fn cell(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.cell_with_options(TableCellOptions::default(), f);
    }

    pub fn cell_with_options(
        &mut self,
        options: TableCellOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let cell_index = self.cells.len();
        let mut out = Vec::new();
        build_imui_children_with_focus(self.cx, &mut out, self.build_focus.clone(), f);
        let content = super::super::cell::pack_cell_children(self.cx, out);
        let test_id = test_ids::cell_test_id(self.row_test_id.as_ref(), cell_index);
        self.cells.push(BuiltTableCell {
            test_id,
            explicit_test_id: options.test_id,
            background: options.background,
            content,
        });
    }

    pub fn cell_text(&mut self, text: impl Into<Arc<str>>) {
        self.cell_text_with_options(text, TableCellOptions::default());
    }

    pub fn cell_text_with_options(&mut self, text: impl Into<Arc<str>>, options: TableCellOptions) {
        let text = text.into();
        self.cell_with_options(options, move |ui| {
            let element = crate::declarative::text::text_table_cell(ui.cx_mut(), text.clone());
            ui.add(element);
        });
    }
}
