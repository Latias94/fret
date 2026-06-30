use crate::primitives::dismissable_layer::OnDismissRequest;
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::layout;
use crate::primitives::dialog;

pub(super) fn popup_modal_backdrop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dim: Color,
    close_on_outside_press: bool,
    on_dismiss_request: OnDismissRequest,
) -> AnyElement {
    let backdrop_visual = cx.container(layout::modal_backdrop_props(dim), |_cx| {
        Vec::<AnyElement>::new()
    });
    dialog::modal_barrier_with_dismiss_handler(
        cx,
        open,
        close_on_outside_press,
        Some(on_dismiss_request),
        [backdrop_visual],
    )
}
