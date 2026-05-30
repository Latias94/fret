use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length, ScrollAxis, ScrollProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

pub(super) fn wrap_table_center_scroll<H: UiHost>(
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
