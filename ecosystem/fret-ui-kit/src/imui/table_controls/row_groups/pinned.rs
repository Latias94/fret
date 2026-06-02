use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::split::PinnedTableGroups;
use super::{layout, scroll};
use crate::imui::TableOptions;

pub(super) fn wrap_split_table_row_groups<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    groups: PinnedTableGroups,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> AnyElement {
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
