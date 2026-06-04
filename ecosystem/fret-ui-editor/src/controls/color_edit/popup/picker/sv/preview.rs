use fret_ui::element::{AnyElement, StackProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::model::HsvColor;
use super::super::super::preview::fill_preview_layout;

mod grid;
mod thumb;

use grid::sv_picker_grid;
use thumb::sv_picker_thumb_overlay;

pub(in crate::controls::color_edit::popup) fn sv_picker_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                sv_picker_grid(cx, hsv.hue),
                sv_picker_thumb_overlay(cx, hsv.saturation, hsv.value),
            ]
        },
    )
}
