use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::ActiveTriggerBehaviorOptions;
use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

mod down;
mod up;

pub(super) fn install_active_trigger_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
    context_anchor_model: Model<Option<fret_core::Point>>,
    options: ActiveTriggerBehaviorOptions,
) {
    down::install_active_trigger_pointer_down(
        cx,
        active_item_model.clone(),
        lifecycle_model.clone(),
        options.primary_active,
        options.request_focus_on_press,
    );
    up::install_active_trigger_pointer_up(
        cx,
        active_item_model,
        lifecycle_model,
        context_anchor_model,
        options.primary_active,
    );
}
