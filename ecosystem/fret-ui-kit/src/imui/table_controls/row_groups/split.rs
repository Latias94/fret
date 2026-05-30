use fret_ui::element::AnyElement;

use super::super::body::PreparedTableCell;
use crate::imui::TableColumnPin;

pub(super) struct PinnedTableGroups {
    pub(super) left: Vec<AnyElement>,
    pub(super) center: Vec<AnyElement>,
    pub(super) right: Vec<AnyElement>,
}

pub(super) fn has_pinned_table_cells(cells: &[PreparedTableCell]) -> bool {
    cells
        .iter()
        .any(|cell| cell.column.pin() != TableColumnPin::None)
}

pub(super) fn split_pinned_table_cells(cells: Vec<PreparedTableCell>) -> PinnedTableGroups {
    let mut left = Vec::new();
    let mut center = Vec::new();
    let mut right = Vec::new();

    for cell in cells {
        match cell.column.pin() {
            TableColumnPin::Left => left.push(cell.element),
            TableColumnPin::Right => right.push(cell.element),
            TableColumnPin::None => center.push(cell.element),
        }
    }

    PinnedTableGroups {
        left,
        center,
        right,
    }
}
