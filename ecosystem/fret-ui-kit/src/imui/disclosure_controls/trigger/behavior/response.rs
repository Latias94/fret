use fret_runtime::Model;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, sanitize_response_for_enabled};

mod context;
mod core;
mod hover;

pub(super) fn populate_disclosure_trigger_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    state: &PressableState,
    context_anchor_model: Model<Option<fret_core::Point>>,
    enabled: bool,
    trigger_response: &mut ResponseExt,
) {
    core::populate_disclosure_trigger_core_response(cx, trigger_id, state, trigger_response);
    context::populate_disclosure_trigger_context_response(
        cx,
        trigger_id,
        context_anchor_model,
        trigger_response,
    );
    hover::populate_disclosure_trigger_hover_response(cx, trigger_id, state, trigger_response);
    sanitize_response_for_enabled(enabled, trigger_response);
}
