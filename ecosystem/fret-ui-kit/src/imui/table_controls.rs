use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Color;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::containers::build_imui_children_with_focus;
use super::{
    ImUiFacade, TableCellOptions, TableColumn, TableHeaderResponse, TableOptions, TableResponse,
    TableRowOptions,
};

use super::TableColumnResizeResponse;

mod body;
mod header;
mod render;

struct BuiltTableRow {
    key: Arc<str>,
    test_id: Option<Arc<str>>,
    background: Option<Color>,
    cells: Vec<BuiltTableCell>,
}

struct BuiltTableCell {
    test_id: Option<Arc<str>>,
    explicit_test_id: Option<Arc<str>>,
    background: Option<Color>,
    content: AnyElement,
}

pub struct ImUiTable<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    rows: &'cx mut Vec<BuiltTableRow>,
    root_test_id: Option<Arc<str>>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

pub struct ImUiTableRow<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    cells: &'cx mut Vec<BuiltTableCell>,
    row_test_id: Option<Arc<str>>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

pub(super) fn table_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: &[TableColumn],
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TableOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> (AnyElement, TableResponse) {
    let columns = columns.to_vec();
    let mut rows = Vec::new();
    {
        let mut table = ImUiTable {
            cx,
            rows: &mut rows,
            root_test_id: options.test_id.clone(),
            build_focus,
        };
        f(&mut table);
    }

    render::render_table(cx, id, columns, rows, options)
}

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
        let default_test_id = self
            .root_test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.row.{row_index}")));
        let row_test_id = options.test_id.or(default_test_id);
        let mut cells = Vec::new();
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
        let content = render::pack_cell_children(self.cx, out);
        let test_id = self
            .row_test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.cell.{cell_index}")));
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

#[cfg(test)]
mod tests;
