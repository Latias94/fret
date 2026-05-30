//! Private shared behavior for active-only immediate-mode triggers.

use fret_runtime::Model;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

mod keyboard;
mod pointer;
mod response;

pub(super) struct ActiveTriggerBehavior {
    pub(super) active_item_model: Model<ImUiActiveItemState>,
    pub(super) context_anchor_model: Model<Option<fret_core::Point>>,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ActiveTriggerBehaviorOptions {
    pub(super) primary_active: bool,
    pub(super) request_focus_on_press: bool,
    pub(super) clear_pointer_move: bool,
}

impl Default for ActiveTriggerBehaviorOptions {
    fn default() -> Self {
        Self {
            primary_active: true,
            request_focus_on_press: true,
            clear_pointer_move: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ActiveTriggerResponseInput {
    pub(super) enabled: bool,
    pub(super) clicked: bool,
    pub(super) changed: bool,
    pub(super) lifecycle_edited: bool,
}

pub(super) fn install_active_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    options: ActiveTriggerBehaviorOptions,
) -> ActiveTriggerBehavior {
    cx.pressable_clear_on_pointer_down();
    if options.clear_pointer_move {
        cx.pressable_clear_on_pointer_move();
    }
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::active_item_model_for_window(cx);
    let lifecycle_model = super::lifecycle_session_model_for(cx, id);
    let context_anchor_model = super::context_menu_anchor_model_for(cx, id);

    keyboard::install_context_menu_key_handler(cx, id);
    pointer::install_active_trigger_pointer_handlers(
        cx,
        active_item_model.clone(),
        lifecycle_model.clone(),
        context_anchor_model.clone(),
        options,
    );

    ActiveTriggerBehavior {
        active_item_model,
        context_anchor_model,
        lifecycle_model,
    }
}

pub(super) fn populate_active_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &ActiveTriggerBehavior,
    input: ActiveTriggerResponseInput,
    response: &mut super::ResponseExt,
) {
    response::populate_active_trigger_response(cx, id, state, behavior, input, response);
}
