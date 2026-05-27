use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::body::PreparedTableCell;
use crate::imui::TableOptions;

mod layout;
mod scroll;
mod split;

pub(super) fn wrap_pinned_table_row_groups<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<PreparedTableCell>,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> AnyElement {
    let has_pinned_cells = split::has_pinned_table_cells(&cells);
    if !has_pinned_cells {
        let cells = cells.into_iter().map(|cell| cell.element).collect();
        return if scroll_x.is_some() {
            let center = layout::table_scroll_content_row_group(cx, cells, options);
            scroll::wrap_table_center_scroll(cx, scroll_x, center)
        } else {
            layout::table_fill_row_group(cx, cells, options)
        };
    }

    let groups = split::split_pinned_table_cells(cells);
    let mut children = Vec::new();
    if !groups.left.is_empty() {
        children.push(layout::table_pinned_row_group(cx, groups.left, options));
    }
    if !groups.center.is_empty() {
        let center = layout::table_scroll_content_row_group(cx, groups.center, options);
        children.push(scroll::wrap_table_center_scroll(cx, scroll_x, center));
    }
    if !groups.right.is_empty() {
        children.push(layout::table_pinned_row_group(cx, groups.right, options));
    }

    layout::table_row_outer_group(cx, children)
}
