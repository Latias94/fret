use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{TableColumn, TableOptions, TableResponse};

mod body;
mod builder;
mod cell;
mod header;
mod header_row;
mod palette;
mod render;
mod row_groups;
mod test_ids;

use builder::{BuiltTableCell, BuiltTableRow};
pub use builder::{ImUiTable, ImUiTableRow};

pub(super) fn table_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: &[TableColumn],
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TableOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> (AnyElement, TableResponse) {
    let columns = columns.to_vec();
    let rows = builder::build_table_rows(cx, options.test_id.clone(), build_focus, f);

    render::render_table(cx, id, columns, rows, options)
}

#[cfg(test)]
mod tests;
