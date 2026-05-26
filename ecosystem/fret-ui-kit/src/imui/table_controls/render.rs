use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SemanticsProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use super::{
    BuiltTableCell, BuiltTableRow, TableColumn, TableColumnResizeResponse, TableHeaderResponse,
    TableOptions, TableResponse,
};
use super::{body, header};
use crate::imui::{TableColumnPin, TableColumnWidth};

pub(super) fn render_table<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: Vec<TableColumn>,
    rows: Vec<BuiltTableRow>,
    options: TableOptions,
) -> (AnyElement, TableResponse) {
    let palette = resolve_table_palette(Theme::global(&*cx.app));
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
        .map(|(index, column)| column_test_id_suffix(column, index))
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
                        content: empty_cell(cx),
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

fn column_test_id_suffix(column: &TableColumn, index: usize) -> String {
    column
        .id()
        .map(test_id_slug)
        .filter(|slug| !slug.is_empty())
        .unwrap_or_else(|| index.to_string())
}

fn test_id_slug(s: &str) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;

    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !out.is_empty() && !last_was_separator {
            out.push('-');
            last_was_separator = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }

    out
}

pub(super) fn table_cell_padding() -> Edges {
    Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
}

pub(super) fn pack_cell_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> AnyElement {
    match children.len() {
        0 => empty_cell(cx),
        1 => children.into_iter().next().expect("single cell child"),
        _ => crate::ui::v_flex(move |_cx| children)
            .gap_metric(crate::MetricRef::space(crate::Space::N0))
            .justify(crate::Justify::Start)
            .items(crate::Items::Stretch)
            .no_wrap()
            .into_element(cx),
    }
}

pub(super) fn empty_cell<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.container(ContainerProps::default(), |_cx| Vec::new())
}

pub(super) fn table_cell_layout(width: TableColumnWidth, clip_cells: bool) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.height = Length::Auto;
    if clip_cells {
        layout.overflow = Overflow::Clip;
    }

    match width {
        TableColumnWidth::Px(width) => {
            layout.size.width = Length::Px(width);
            layout.size.min_width = Some(Length::Px(width));
            layout.size.max_width = Some(Length::Px(width));
            layout.flex.shrink = 0.0;
        }
        TableColumnWidth::Fill(weight) => {
            let grow = if weight.is_finite() && weight > 0.0 {
                weight
            } else {
                1.0
            };
            layout.size.width = Length::Px(Px(0.0));
            layout.flex.grow = grow;
            layout.flex.shrink = 1.0;
            layout.flex.basis = Length::Px(Px(0.0));
        }
    }

    layout
}

fn resolve_table_palette(theme: &Theme) -> body::TablePalette {
    let table_bg = theme
        .color_by_key("table.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"));
    let header_bg = theme
        .color_by_key("table.header.background")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"));
    let mut striped_bg = theme
        .color_by_key("table.row.striped")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"));
    striped_bg.a *= 0.35;

    body::TablePalette {
        table_bg,
        border,
        header_bg,
        striped_bg,
    }
}
