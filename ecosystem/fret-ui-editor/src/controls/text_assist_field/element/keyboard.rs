//! Text-assist field input-owned keyboard policy owner.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::UiFocusActionHost;
use fret_ui::{ElementContext, GlobalElementId, UiHost};
use fret_ui_kit::headless::text_assist::{
    InputOwnedTextAssistKeyOptions, TextAssistItem, input_owned_text_assist_key_handler,
};

use super::super::OnTextAssistFieldAccept;
use super::super::accept::accept_text_assist_match;

pub(super) struct TextAssistFieldKeyboardInput {
    pub(super) items: Arc<[TextAssistItem]>,
    pub(super) query_model: Model<String>,
    pub(super) dismissed_query_model: Model<String>,
    pub(super) active_item_id_model: Model<Option<Arc<str>>>,
    pub(super) key_options: InputOwnedTextAssistKeyOptions,
    pub(super) on_accept: Option<OnTextAssistFieldAccept>,
}

pub(super) fn install_text_assist_field_key_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    input: TextAssistFieldKeyboardInput,
) {
    let TextAssistFieldKeyboardInput {
        items,
        query_model,
        dismissed_query_model,
        active_item_id_model,
        key_options,
        on_accept,
    } = input;

    let query_model_for_accept = query_model.clone();
    let dismissed_query_model_for_accept = dismissed_query_model.clone();
    let active_item_id_model_for_accept = active_item_id_model.clone();

    cx.key_add_on_key_down_capture_for(
        root_id,
        input_owned_text_assist_key_handler(
            items,
            query_model,
            dismissed_query_model,
            active_item_id_model,
            key_options,
            Arc::new(move |host: &mut dyn UiFocusActionHost, action_cx, active| {
                accept_text_assist_match(
                    host,
                    action_cx,
                    &query_model_for_accept,
                    &dismissed_query_model_for_accept,
                    &active_item_id_model_for_accept,
                    active,
                    on_accept.as_ref(),
                );
            }),
        ),
    );
}
