use std::sync::Arc;

use fret_core::{Color, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, SemanticsDecoration};
use fret_ui::{ElementContext, UiHost};

use super::super::cell::{table_cell_layout, table_cell_padding};
use super::super::{TableColumn, TableOptions};

pub(in super::super) fn wrap_table_cell<H: UiHost>(
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
