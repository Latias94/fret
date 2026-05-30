use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{GridOptions, ImUiFacade};
use super::children::build_imui_children_with_focus;

pub(in crate::imui) fn grid_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: GridOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let mut cells: Vec<AnyElement> = Vec::new();
    build_imui_children_with_focus(cx, &mut cells, build_focus, f);

    let columns = options.columns.max(1);
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut row_index = 0usize;
    let mut iter = cells.into_iter();

    loop {
        let mut row_cells: Vec<AnyElement> = Vec::with_capacity(columns);
        for _ in 0..columns {
            let Some(cell) = iter.next() else {
                break;
            };
            row_cells.push(cell);
        }
        if row_cells.is_empty() {
            break;
        }

        let row = cx.keyed(row_index, |cx| {
            crate::ui::h_flex(move |_cx| row_cells)
                .gap_metric(options.column_gap.clone())
                .justify(options.row_justify)
                .items(options.row_items)
                .no_wrap()
                .into_element(cx)
        });
        rows.push(row);
        row_index += 1;
    }

    let mut builder = crate::ui::v_flex(move |_cx| rows)
        .layout(layout)
        .gap_metric(options.row_gap)
        .justify(crate::Justify::Start)
        .items(crate::Items::Stretch)
        .no_wrap();
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}
