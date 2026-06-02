use fret_core::{Corners, Edges, Px};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign};

use super::super::super::super::control_chrome::{self, ImUiControlPalette};

pub(in crate::imui::boolean_controls) fn radio_indicator<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    palette: ImUiControlPalette,
    selected: bool,
) -> AnyElement {
    let mut outer = ContainerProps::default();
    outer.layout.size.width = Length::Px(control_chrome::RADIO_INDICATOR_SIZE);
    outer.layout.size.height = Length::Px(control_chrome::RADIO_INDICATOR_SIZE);
    outer.border = Edges::all(Px(1.0));
    outer.border_color = Some(if selected {
        palette.accent_background
    } else {
        palette.border
    });
    outer.corner_radii = Corners::all(Px(999.0));

    cx.container(outer, move |cx| {
        if !selected {
            return Vec::new();
        }

        let mut center = FlexProps::default();
        center.layout.size.width = Length::Fill;
        center.layout.size.height = Length::Fill;
        center.justify = MainAlign::Center;
        center.align = CrossAlign::Center;

        let mut dot = ContainerProps::default();
        dot.layout.size.width = Length::Px(control_chrome::RADIO_DOT_SIZE);
        dot.layout.size.height = Length::Px(control_chrome::RADIO_DOT_SIZE);
        dot.background = Some(palette.accent_background);
        dot.corner_radii = Corners::all(Px(999.0));

        vec![cx.flex(center, move |cx| vec![cx.container(dot, |_| Vec::new())])]
    })
}
