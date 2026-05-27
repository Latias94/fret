use std::any::Any;
use std::rc::Rc;

use fret_ui::UiHost;

use super::super::{DragSourceOptions, DragSourceResponse, ResponseExt, UiWriterImUiFacadeExt};
use super::store::{prune_store, source_response_for, store_model_for};

mod hooks;

pub(in crate::imui) fn drag_source_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: DragSourceOptions,
) -> DragSourceResponse {
    let Some(trigger_id) = trigger.id() else {
        return DragSourceResponse::inactive();
    };

    let payload: Rc<dyn Any> = Rc::new(payload);
    ui.with_cx_mut(|cx| {
        let store = store_model_for(cx);
        prune_store(cx, &store);

        let kind = super::super::drag_kind_for_element(trigger_id);

        hooks::install_drag_source_hooks(cx, trigger_id, kind, store.clone(), payload, &options);

        source_response_for(cx, &store, trigger_id, kind)
    })
}
