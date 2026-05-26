use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use super::{
    BuiltTableCell, BuiltTableRow, TableColumn, TableColumnResizeResponse, TableHeaderResponse,
    TableOptions, TableResponse,
};
use super::{body, cell, header, palette, test_ids};
use crate::imui::TableColumnPin;

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
        let column_test_id_suffixes = column_test_id_suffixes.clone();
        Some(cx.keyed(format!("{id}.header"), |cx| {
            let cells = columns
                .iter()
                .enumerate()
                .filter(|(_, column)| column.visible())
                .map(|(index, column)| {
                    let visible_label = header::visible_header_label(column);
                    let test_id = root_test_id.as_ref().map(|base| {
                        Arc::from(format!(
                            "{base}.header.cell.{}",
                            column_test_id_suffixes[index]
                        ))
                    });
                    let sortable = header::column_is_sortable(column);
                    let resize_options = column.resize_options();
                    let mut resize = TableColumnResizeResponse {
                        column_index: index,
                        column_id: column.id_arc(),
                        enabled: resize_options.is_some(),
                        min_width: resize_options.and_then(|options| options.min_width),
                        max_width: resize_options.and_then(|options| options.max_width),
                        drag: Default::default(),
                    };
                    let built = if sortable {
                        header::wrap_sortable_header_cell(
                            cx,
                            column,
                            index,
                            visible_label.clone(),
                            test_id,
                            &options,
                            &mut resize,
                        )
                    } else {
                        header::wrap_plain_header_cell(
                            cx,
                            column,
                            index,
                            visible_label,
                            test_id,
                            &options,
                            &mut resize,
                        )
                    };
                    header_responses.push(TableHeaderResponse {
                        column_index: index,
                        column_id: column.id_arc(),
                        sortable,
                        sort_direction: column.sort_direction(),
                        trigger: built.trigger,
                        resize,
                    });
                    body::PreparedTableCell {
                        column: column.clone(),
                        element: built.element,
                    }
                })
                .collect::<Vec<_>>();
            body::wrap_table_row(
                cx,
                cells,
                root_test_id
                    .as_ref()
                    .map(|base| Arc::from(format!("{base}.header"))),
                true,
                false,
                None,
                &palette,
                &options,
                scroll_x.clone(),
            )
        }))
    } else {
        None
    };

    let body_rows = rows
        .into_iter()
        .enumerate()
        .map(|(row_index, row)| {
            let striped = options.striped && row_index % 2 == 1;
            let column_test_id_suffixes = column_test_id_suffixes.clone();
            cx.keyed(row.key.clone(), |cx| {
                let mut iter = row.cells.into_iter();
                let mut cells = Vec::with_capacity(visible_columns.len());
                for (column_index, column) in columns.iter().enumerate() {
                    let built = iter.next().unwrap_or_else(|| BuiltTableCell {
                        test_id: None,
                        explicit_test_id: None,
                        background: None,
                        content: cell::empty_cell(cx),
                    });
                    if !column.visible() {
                        continue;
                    }
                    let default_test_id = row
                        .test_id
                        .as_ref()
                        .map(|base| {
                            Arc::from(format!(
                                "{base}.cell.{}",
                                column_test_id_suffixes[column_index]
                            ))
                        })
                        .or(built.test_id);
                    let test_id = built.explicit_test_id.or(default_test_id);
                    cells.push(body::PreparedTableCell {
                        column: column.clone(),
                        element: body::wrap_table_cell(
                            cx,
                            column,
                            built.content,
                            test_id,
                            false,
                            built.background,
                            &options,
                        ),
                    });
                }
                debug_assert!(
                    iter.next().is_none(),
                    "imui table rows must emit exactly one cell per declared column"
                );
                body::wrap_table_row(
                    cx,
                    cells,
                    row.test_id,
                    false,
                    striped,
                    row.background,
                    &palette,
                    &options,
                    scroll_x.clone(),
                )
            })
        })
        .collect::<Vec<_>>();

    let mut children = Vec::new();
    if let Some(header) = header {
        children.push(header);
    }
    children.extend(body_rows);

    let mut root = ContainerProps::default();
    root.layout.size.width = Length::Fill;
    root.layout.size.height = Length::Auto;
    root.background = Some(palette.table_bg);
    root.border = Edges::all(Px(1.0));
    root.border_color = Some(palette.border);
    root.corner_radii = Corners::all(Px(6.0));

    let table = cx.container(root, move |cx| {
        vec![
            crate::ui::v_flex(move |_cx| children)
                .gap_metric(options.row_gap.clone())
                .justify(crate::Justify::Start)
                .items(crate::Items::Stretch)
                .no_wrap()
                .into_element(cx),
        ]
    });

    let element = if let Some(test_id) = options.test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::Group;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![table])
    } else {
        table
    };

    (
        element,
        TableResponse {
            headers: header_responses,
        },
    )
}
