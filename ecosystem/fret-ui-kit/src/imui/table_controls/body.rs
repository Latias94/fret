use std::sync::Arc;

use fret_core::{Color, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsDecoration, SemanticsProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::cell::{table_cell_layout, table_cell_padding};
use super::row_groups;
use crate::imui::{TableColumn, TableOptions};

pub(super) struct PreparedTableCell {
    pub(super) column: TableColumn,
    pub(super) element: AnyElement,
}

pub(super) struct TablePalette {
    pub(super) table_bg: Color,
    pub(super) border: Color,
    pub(super) header_bg: Color,
    pub(super) striped_bg: Color,
}

pub(super) fn wrap_table_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<PreparedTableCell>,
    test_id: Option<Arc<str>>,
    header: bool,
    striped: bool,
    background: Option<Color>,
    palette: &TablePalette,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> AnyElement {
    let background = background.or_else(|| {
        if header {
            Some(palette.header_bg)
        } else if striped {
            Some(palette.striped_bg)
        } else {
            None
        }
    });

    let mut row = ContainerProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.background = background;

    let row = cx.container(row, move |cx| {
        vec![row_groups::wrap_pinned_table_row_groups(
            cx, cells, options, scroll_x,
        )]
    });

    if let Some(test_id) = test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::Group;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![row])
    } else {
        row
    }
}

pub(super) fn wrap_table_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    content: AnyElement,
    test_id: Option<Arc<str>>,
    header: bool,
    background: Option<Color>,
    options: &TableOptions,
) -> AnyElement {
    let mut cell = ContainerProps::default();
    cell.layout = table_cell_layout(column.width(), options.clip_cells);
    cell.padding = table_cell_padding().into();
    cell.background = background;
    let cell = cx.container(cell, move |_cx| vec![content]);
    if let Some(test_id) = test_id {
        cell.attach_semantics(
            SemanticsDecoration::default()
                .role(if header {
                    SemanticsRole::Heading
                } else {
                    SemanticsRole::Group
                })
                .test_id(test_id),
        )
    } else {
        cell
    }
}
