//! Text-assist match acceptance owner.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui_kit::headless::text_assist::TextAssistMatch;

use super::OnTextAssistFieldAccept;

pub(super) fn accept_text_assist_match(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    query_model: &Model<String>,
    dismissed_query_model: &Model<String>,
    active_item_id_model: &Model<Option<Arc<str>>>,
    active: TextAssistMatch,
    on_accept: Option<&OnTextAssistFieldAccept>,
) {
    let next_query = active.label.as_ref().to_string();
    let _ = host.models_mut().update(query_model, |value| {
        value.clear();
        value.push_str(&next_query);
    });
    let _ = host.models_mut().update(dismissed_query_model, |value| {
        value.clear();
        value.push_str(&next_query);
    });
    let _ = host.models_mut().update(active_item_id_model, |value| {
        *value = Some(active.item_id.clone())
    });
    if let Some(on_accept) = on_accept {
        on_accept(host, action_cx, active);
    }
    host.request_redraw(action_cx.window);
}
