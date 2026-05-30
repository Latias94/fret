use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Color;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod cell_methods;
mod row_methods;
mod test_ids;

pub(super) struct BuiltTableRow {
    pub(super) key: Arc<str>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) background: Option<Color>,
    pub(super) cells: Vec<BuiltTableCell>,
}

pub(super) struct BuiltTableCell {
    pub(super) test_id: Option<Arc<str>>,
    pub(super) explicit_test_id: Option<Arc<str>>,
    pub(super) background: Option<Color>,
    pub(super) content: AnyElement,
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

pub(super) fn build_table_rows<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_test_id: Option<Arc<str>>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> Vec<BuiltTableRow> {
    let mut rows = Vec::new();
    {
        let mut table = ImUiTable {
            cx,
            rows: &mut rows,
            root_test_id,
            build_focus,
        };
        f(&mut table);
    }
    rows
}
