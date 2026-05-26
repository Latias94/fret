use fret_core::Px;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, ScrollAxis, ScrollProps,
    SpacingEdges, SpacingLength,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use super::body::PreparedTableCell;
use crate::imui::{TableColumnPin, TableOptions};

struct PinnedTableGroups {
    left: Vec<AnyElement>,
    center: Vec<AnyElement>,
    right: Vec<AnyElement>,
}

pub(super) fn wrap_pinned_table_row_groups<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<PreparedTableCell>,
    options: &TableOptions,
    scroll_x: Option<ScrollHandle>,
) -> AnyElement {
    let has_pinned_cells = cells
        .iter()
        .any(|cell| cell.column.pin() != TableColumnPin::None);
    if !has_pinned_cells {
        let cells = cells.into_iter().map(|cell| cell.element).collect();
        return if scroll_x.is_some() {
            let center = table_scroll_content_row_group(cx, cells, options);
            wrap_table_center_scroll(cx, scroll_x, center)
        } else {
            table_fill_row_group(cx, cells, options)
        };
    }

    let groups = split_pinned_table_cells(cells);
    let mut children = Vec::new();
    if !groups.left.is_empty() {
        children.push(table_pinned_row_group(cx, groups.left, options));
    }
    if !groups.center.is_empty() {
        let center = table_scroll_content_row_group(cx, groups.center, options);
        children.push(wrap_table_center_scroll(cx, scroll_x, center));
    }
    if !groups.right.is_empty() {
        children.push(table_pinned_row_group(cx, groups.right, options));
    }

    table_row_outer_group(cx, children)
}

fn split_pinned_table_cells(cells: Vec<PreparedTableCell>) -> PinnedTableGroups {
    let mut left = Vec::new();
    let mut center = Vec::new();
    let mut right = Vec::new();

    for cell in cells {
        match cell.column.pin() {
            TableColumnPin::Left => left.push(cell.element),
            TableColumnPin::Right => right.push(cell.element),
            TableColumnPin::None => center.push(cell.element),
        }
    }

    PinnedTableGroups {
        left,
        center,
        right,
    }
}

fn table_row_outer_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout.flex.basis = Length::Px(Px(0.0));

    table_h_flex(
        cx,
        children,
        crate::MetricRef::space(crate::Space::N0),
        layout,
    )
}

fn table_fill_row_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<AnyElement>,
    options: &TableOptions,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout.flex.basis = Length::Px(Px(0.0));

    table_h_flex(cx, cells, options.column_gap.clone(), layout)
}

fn table_pinned_row_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<AnyElement>,
    options: &TableOptions,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.flex.shrink = 0.0;

    table_h_flex(cx, cells, options.column_gap.clone(), layout)
}

fn table_scroll_content_row_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<AnyElement>,
    options: &TableOptions,
) -> AnyElement {
    table_h_flex(
        cx,
        cells,
        options.column_gap.clone(),
        LayoutStyle::default(),
    )
}

fn wrap_table_center_scroll<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    scroll_x: Option<ScrollHandle>,
    row: AnyElement,
) -> AnyElement {
    if let Some(scroll_x) = scroll_x {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        layout.flex.grow = 1.0;
        layout.flex.shrink = 1.0;
        layout.flex.basis = Length::Px(Px(0.0));
        cx.scroll(
            ScrollProps {
                axis: ScrollAxis::X,
                scroll_handle: Some(scroll_x),
                layout,
                ..Default::default()
            },
            |_cx| vec![row],
        )
    } else {
        row
    }
}

fn table_h_flex<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
    gap: crate::MetricRef,
    layout: LayoutStyle,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    cx.flex(
        FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: SpacingLength::Px(gap.resolve(theme)),
            padding: SpacingEdges::all(SpacingLength::Px(Px(0.0))),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        |_cx| children,
    )
}
