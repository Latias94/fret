use std::any::Any;

use fret_ui::UiHost;

use super::super::{DropTargetOptions, DropTargetResponse, ResponseExt, UiWriterImUiFacadeExt};
use super::store::{
    first_active_payload_for, prune_store, store_model_for, take_delivered_payload_for,
};

pub(in crate::imui) fn drop_target_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    options: DropTargetOptions,
) -> DropTargetResponse<T> {
    let Some(trigger_id) = trigger.id() else {
        return DropTargetResponse::empty();
    };

    ui.with_cx_mut(|cx| {
        let store = store_model_for(cx);
        prune_store(cx, &store);

        let mut response = DropTargetResponse::empty();
        if !options.enabled {
            return response;
        }

        if let Some((session_id, source_id, position, payload)) =
            take_delivered_payload_for::<T, _>(cx, &store, trigger_id)
        {
            response.active = true;
            response.delivered = true;
            response.source_id = Some(source_id);
            response.session_id = Some(session_id);
            response.delivered_position = Some(position);
            response.delivered_payload = Some(payload);
        }

        if let Some((session_id, source_id, position, payload)) =
            first_active_payload_for::<T, _>(cx, &store)
        {
            response.active = true;
            if response.source_id.is_none() {
                response.source_id = Some(source_id);
            }
            if response.session_id.is_none() {
                response.session_id = Some(session_id);
            }
            response.preview_position = Some(position);
            let _ = cx.app.models_mut().update(&store, |st| {
                if let Some(active) = st.active.get_mut(&session_id) {
                    if trigger.pointer_hovered_raw() {
                        active.hovered_target = Some(trigger_id);
                    } else if active.hovered_target == Some(trigger_id) {
                        active.hovered_target = None;
                    }
                }
            });
            if trigger.pointer_hovered_raw() {
                response.over = true;
                response.preview_payload = Some(payload);
            }
        }

        response
    })
}
