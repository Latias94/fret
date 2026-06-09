use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use crate::imui::{
    KEY_CONTEXT_MENU_REQUESTED, KEY_DOUBLE_CLICKED, KEY_SECONDARY_CLICKED, ResponseExt,
};

pub(super) fn populate_disclosure_trigger_context_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    context_anchor_model: Model<Option<fret_core::Point>>,
    trigger_response: &mut ResponseExt,
) {
    trigger_response
        .set_secondary_clicked(cx.take_transient_for(trigger_id, KEY_SECONDARY_CLICKED));
    trigger_response.set_double_clicked(cx.take_transient_for(trigger_id, KEY_DOUBLE_CLICKED));
    trigger_response
        .set_context_menu_requested(cx.take_transient_for(trigger_id, KEY_CONTEXT_MENU_REQUESTED));
    trigger_response.set_context_menu_anchor(
        cx.read_model(&context_anchor_model, Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(None),
    );
}
