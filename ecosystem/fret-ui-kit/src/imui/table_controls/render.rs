use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use super::{BuiltTableRow, TableColumn, TableOptions, TableResponse};
use super::{header_row, palette, test_ids};
use crate::imui::TableColumnPin;

mod body_rows;
mod root;

pub(super) fn render_table<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: Vec<TableColumn>,
    rows: Vec<BuiltTableRow>,
    options: TableOptions,
) -> (AnyElement, TableResponse) {
    let palette = palette::resolve_table_palette(Theme::global(&*cx.app));
    let root_test_id = options.test_id.clone();
    let visible_columns = columns
        .iter()
        .enumerate()
        .filter(|(_, column)| column.visible())
        .collect::<Vec<_>>();
    let has_pinned_columns = visible_columns
        .iter()
        .any(|(_, column)| column.pin() != TableColumnPin::None);
    let scroll_x = if has_pinned_columns || options.horizontal_scroll.is_some() {
        options
            .horizontal_scroll
            .clone()
            .or_else(|| Some(cx.slot_state(ScrollHandle::default, |h| h.clone())))
    } else {
        None
    };
    let show_header = options.show_header
        && visible_columns
            .iter()
            .any(|(_, column)| column.header().is_some());
    let column_test_id_suffixes = columns
        .iter()
        .enumerate()
        .map(|(index, column)| test_ids::column_test_id_suffix(column, index))
        .collect::<Vec<_>>();
    let mut header_responses = Vec::new();
    let header = if show_header {
        Some(header_row::render_table_header(
            cx,
            id,
            &columns,
            &column_test_id_suffixes,
            root_test_id.as_ref(),
            &palette,
            &options,
            scroll_x.clone(),
            &mut header_responses,
        ))
    } else {
        None
    };

    let body_rows = body_rows::render_table_body_rows(
        cx,
        &columns,
        rows,
        &column_test_id_suffixes,
        &palette,
        &options,
        scroll_x.clone(),
    );

    let mut children = Vec::new();
    if let Some(header) = header {
        children.push(header);
    }
    children.extend(body_rows);

    let element = root::table_root_element(cx, children, &palette, options);

    (
        element,
        TableResponse {
            headers: header_responses,
        },
    )
}
