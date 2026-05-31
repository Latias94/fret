use fret_core::Color;
use fret_ui::element::AnyElement;

use crate::imui::TableColumn;

mod cell;
mod row;

pub(super) use cell::wrap_table_cell;
pub(super) use row::wrap_table_row;

pub(super) struct PreparedTableCell {
    pub(super) column: TableColumn,
    pub(super) element: AnyElement,
}

pub(super) struct TablePalette {
    pub(super) table_bg: Color,
    pub(super) border: Color,
    pub(super) header_bg: Color,
    pub(super) striped_bg: Color,
}
