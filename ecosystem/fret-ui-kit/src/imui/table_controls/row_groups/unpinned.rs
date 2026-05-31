use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::super::body::PreparedTableCell;
use super::{layout, scroll};
use crate::imui::TableOptions;

pub(super) fn wrap_unpinned_table_row_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<PreparedTableCell>,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> AnyElement {
    let cells = cells.into_iter().map(|cell| cell.element).collect();
    if scroll_x.is_some() {
        let center = layout::table_scroll_content_row_group(cx, cells, options);
        scroll::wrap_table_center_scroll(cx, scroll_x, center)
    } else {
        layout::table_fill_row_group(cx, cells, options)
    }
}
