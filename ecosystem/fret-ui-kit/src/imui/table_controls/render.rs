use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use super::{BuiltTableRow, TableColumn, TableOptions, TableResponse};
use super::{header_row, palette};
use plan::build_table_render_plan;

mod body_rows;
mod plan;
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
    let render_plan = build_table_render_plan(cx, &columns, &options);
    let mut header_responses = Vec::new();
    let header = if render_plan.show_header {
        Some(header_row::render_table_header(
            cx,
            id,
            &columns,
            &render_plan.column_test_id_suffixes,
            root_test_id.as_ref(),
            &palette,
            &options,
            render_plan.scroll_x.clone(),
            &mut header_responses,
        ))
    } else {
        None
    };

    let body_rows = body_rows::render_table_body_rows(
        cx,
        &columns,
        rows,
        &render_plan.column_test_id_suffixes,
        &palette,
        &options,
        render_plan.scroll_x.clone(),
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
