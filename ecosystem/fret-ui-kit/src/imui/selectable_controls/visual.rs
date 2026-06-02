use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, Theme, UiHost};

mod palette;

pub(super) use palette::resolve_selectable_palette;

pub(super) fn selectable_row_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    enabled: bool,
    selected: bool,
    highlighted: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let palette = resolve_selectable_palette(
        Theme::global(&*cx.app),
        enabled,
        selected,
        highlighted || state.hovered || state.focused,
        state.pressed,
    );

    let mut row = ContainerProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    row.background = palette.bg;
    row.corner_radii = Corners::all(super::super::control_chrome::CONTROL_RADIUS);

    cx.container(row, move |cx| {
        vec![
            crate::declarative::text::text_list_row_label(cx, label.clone())
                .inherit_foreground(palette.fg),
        ]
    })
}
