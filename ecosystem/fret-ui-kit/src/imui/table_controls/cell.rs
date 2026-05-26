use fret_core::{Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow};
use fret_ui::{ElementContext, UiHost};

use crate::imui::TableColumnWidth;

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
