use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::SliderInteractionRange;
use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

mod down;
mod move_handler;
mod up;
mod value_update;

pub(super) fn install_slider_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    model: Model<f32>,
    range: SliderInteractionRange,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_move = active_item_model.clone();
    let active_item_model_for_up = active_item_model;
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_move = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model;

    let model_for_down = model.clone();
    down::install_slider_pointer_down_handler(
        cx,
        model_for_down,
        range,
        active_item_model_for_down,
        lifecycle_model_for_down,
    );

    let model_for_move = model;
    move_handler::install_slider_pointer_move_handler(
        cx,
        model_for_move,
        range,
        active_item_model_for_move,
        lifecycle_model_for_move,
    );

    up::install_slider_pointer_up_handler(cx, id, active_item_model_for_up, lifecycle_model_for_up);
}
