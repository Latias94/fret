use fret_ui::element::{AnyElement, StackProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::preview::fill_preview_layout;

mod gradient;
mod thumb;

use gradient::vertical_hue_gradient_overlay;
use thumb::vertical_bar_thumb_overlay;

pub(in crate::controls::color_edit::popup) fn hue_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hue: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                vertical_hue_gradient_overlay(cx),
                vertical_bar_thumb_overlay(cx, hue),
            ]
        },
    )
}
