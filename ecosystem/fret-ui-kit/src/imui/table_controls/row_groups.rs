use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::body::PreparedTableCell;
use crate::imui::TableOptions;

mod layout;
mod pinned;
mod scroll;
mod split;
mod unpinned;

use pinned::wrap_split_table_row_groups;
use unpinned::wrap_unpinned_table_row_group;

pub(super) fn wrap_pinned_table_row_groups<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<PreparedTableCell>,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> AnyElement {
    let has_pinned_cells = split::has_pinned_table_cells(&cells);
    if !has_pinned_cells {
        return wrap_unpinned_table_row_group(cx, cells, options, scroll_x);
    }

    let groups = split::split_pinned_table_cells(cells);
    wrap_split_table_row_groups(cx, groups, options, scroll_x)
}
