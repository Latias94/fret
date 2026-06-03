use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length};
use fret_ui::{ElementContext, UiHost};

use crate::imui::TableOptions;

mod horizontal;

use horizontal::table_h_flex;

pub(super) fn table_row_outer_group<H: UiHost>(
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

pub(super) fn table_fill_row_group<H: UiHost>(
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

pub(super) fn table_pinned_row_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<AnyElement>,
    options: &TableOptions,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.flex.shrink = 0.0;

    table_h_flex(cx, cells, options.column_gap.clone(), layout)
}

pub(super) fn table_scroll_content_row_group<H: UiHost>(
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
