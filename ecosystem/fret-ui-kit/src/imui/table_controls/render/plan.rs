use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::super::{TableColumn, TableOptions, test_ids};
use crate::imui::TableColumnPin;

pub(super) struct TableRenderPlan {
    pub(super) column_test_id_suffixes: Vec<String>,
    pub(super) scroll_x: Option<ScrollHandle>,
    pub(super) show_header: bool,
}

pub(super) fn build_table_render_plan<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    columns: &[TableColumn],
    options: &TableOptions,
) -> TableRenderPlan {
    let visible_columns = columns
        .iter()
        .enumerate()
        .filter(|(_, column)| column.visible())
        .collect::<Vec<_>>();
    let has_pinned_columns = visible_columns
        .iter()
        .any(|(_, column)| column.pin() != TableColumnPin::None);
    let scroll_x = resolve_table_scroll_handle(cx, has_pinned_columns, options);
    let show_header = options.show_header
        && visible_columns
            .iter()
            .any(|(_, column)| column.header().is_some());
    let column_test_id_suffixes = columns
        .iter()
        .enumerate()
        .map(|(index, column)| test_ids::column_test_id_suffix(column, index))
        .collect::<Vec<_>>();

    TableRenderPlan {
        column_test_id_suffixes,
        scroll_x,
        show_header,
    }
}

fn resolve_table_scroll_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    has_pinned_columns: bool,
    options: &TableOptions,
) -> Option<ScrollHandle> {
    if has_pinned_columns || options.horizontal_scroll.is_some() {
        options
            .horizontal_scroll
            .clone()
            .or_else(|| Some(cx.slot_state(ScrollHandle::default, |h| h.clone())))
    } else {
        None
    }
}
