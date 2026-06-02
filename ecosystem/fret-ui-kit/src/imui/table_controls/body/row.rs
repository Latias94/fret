use std::sync::Arc;

use fret_core::{Color, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::super::{TableOptions, row_groups};
use super::{PreparedTableCell, TablePalette};

pub(in super::super) fn wrap_table_row<H: UiHost>(
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
