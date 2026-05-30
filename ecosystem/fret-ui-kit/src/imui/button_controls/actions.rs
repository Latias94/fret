use std::{any::Any, sync::Arc};

use fret_runtime::ActionId;
use fret_ui::UiHost;

use super::super::{ButtonOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::{behavior, entry};

pub(in crate::imui) fn action_button_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: ButtonOptions,
) -> ResponseExt {
    entry::button_impl(
        ui,
        label,
        options,
        Some(behavior::ButtonAction {
            action,
            payload: None,
        }),
    )
}

pub(in crate::imui) fn action_payload_button_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T,
>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    payload: T,
    options: ButtonOptions,
) -> ResponseExt
where
    T: Any + Clone + Send + Sync + 'static,
{
    let payload = Arc::new(move || Box::new(payload.clone()) as Box<dyn Any + Send + Sync>);
    entry::button_impl(
        ui,
        label,
        options,
        Some(behavior::ButtonAction {
            action,
            payload: Some(payload),
        }),
    )
}
