use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::interaction_runtime::ImUiActiveItemState;

mod keyboard;
mod pointer;

#[derive(Clone, Copy, Debug)]
pub(super) struct SliderInteractionRange {
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) step: f32,
}

pub(super) fn install_slider_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    enabled: bool,
    model: fret_runtime::Model<f32>,
    min: f32,
    max: f32,
    step: f32,
) -> fret_runtime::Model<ImUiActiveItemState> {
    cx.pressable_clear_on_pointer_down();
    cx.pressable_clear_on_pointer_move();
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::super::active_item_model_for_window(cx);
    let lifecycle_model = super::super::lifecycle_session_model_for(cx, id);
    let range = SliderInteractionRange { min, max, step };

    pointer::install_slider_pointer_handlers(
        cx,
        id,
        model.clone(),
        range,
        active_item_model.clone(),
        lifecycle_model.clone(),
    );
    keyboard::install_slider_keyboard_handler(cx, id, enabled, model, range, lifecycle_model);

    active_item_model
}
