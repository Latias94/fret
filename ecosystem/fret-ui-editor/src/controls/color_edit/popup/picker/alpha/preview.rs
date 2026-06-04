use fret_ui::element::{AnyElement, StackProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::preview::{checkerboard_grid, fill_preview_layout};

mod gradient;
mod thumb;

use gradient::{alpha_gradient_overlay, vertical_alpha_gradient_overlay};
use thumb::{horizontal_bar_thumb_overlay, vertical_bar_thumb_overlay};

pub(super) fn vertical_alpha_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                checkerboard_grid(cx),
                vertical_alpha_gradient_overlay(cx, rgb),
                vertical_bar_thumb_overlay(cx, 1.0 - alpha.clamp(0.0, 1.0)),
            ]
        },
    )
}

pub(super) fn alpha_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                checkerboard_grid(cx),
                alpha_gradient_overlay(cx, rgb),
                horizontal_bar_thumb_overlay(cx, alpha),
            ]
        },
    )
}
