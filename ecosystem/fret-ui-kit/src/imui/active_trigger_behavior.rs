//! Private shared behavior for active-only immediate-mode triggers.

use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod install;
mod keyboard;
mod options;
mod pointer;
mod response;
mod types;

pub(super) use options::ActiveTriggerBehaviorOptions;
pub(super) use types::{ActiveTriggerBehavior, ActiveTriggerResponseInput};

pub(super) fn install_active_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    options: ActiveTriggerBehaviorOptions,
) -> ActiveTriggerBehavior {
    install::install_active_trigger_behavior(cx, id, options)
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
