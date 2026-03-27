use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{GridOptions, HorizontalOptions, ImUiFacade, ScrollOptions, VerticalOptions};

pub(super) fn build_imui_children_with_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let mut ui = ImUiFacade {
        cx,
        out,
        build_focus,
    };
    f(&mut ui);
}

pub(super) fn horizontal_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: HorizontalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::h_flex_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    builder.into_element(cx)
}

pub(super) fn vertical_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VerticalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::v_flex_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    builder.into_element(cx)
}

pub(super) fn scroll_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ScrollOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::scroll_area_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .axis(options.axis)
        .show_scrollbars(options.show_scrollbar_x, options.show_scrollbar_y);
    if let Some(handle) = options.handle {
        builder = builder.handle(handle);
    }
    builder.into_element(cx)
}

pub(super) fn grid_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: GridOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
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

    crate::ui::v_flex(move |_cx| rows)
        .gap_metric(options.row_gap)
        .justify(crate::Justify::Start)
        .items(crate::Items::Stretch)
        .no_wrap()
        .into_element(cx)
}
