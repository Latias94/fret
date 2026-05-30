use fret_runtime::Model;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ResponseExt, context_menu_anchor_model_for};
use super::super::spec::DisclosureSpec;

mod activation;
mod keyboard;
mod pointer;
mod response;

pub(super) fn install_disclosure_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: &PressableState,
    trigger_id: GlobalElementId,
    spec: &DisclosureSpec,
    open_model: Model<bool>,
    enabled: bool,
    trigger_response: &mut ResponseExt,
) {
    let context_anchor_model = context_menu_anchor_model_for(cx, trigger_id);
    let context_anchor_model_for_report = context_anchor_model.clone();
    cx.pressable_clear_on_pointer_down();
    cx.pressable_clear_on_pointer_move();
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(trigger_id);

    let has_children = spec.has_children();
    activation::install_disclosure_trigger_activation(cx, open_model.clone(), has_children);

    if enabled {
        keyboard::install_disclosure_trigger_keyboard(cx, trigger_id, open_model, spec);
    }

    pointer::install_disclosure_trigger_pointer(cx, context_anchor_model);

    response::populate_disclosure_trigger_response(
        cx,
        trigger_id,
        state,
        context_anchor_model_for_report,
        enabled,
        trigger_response,
    );
}
