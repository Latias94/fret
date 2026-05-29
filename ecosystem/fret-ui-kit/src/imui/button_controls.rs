//! Immediate-mode button-style pressable helpers.

mod behavior;
mod entry;
mod visual;

use std::{any::Any, sync::Arc};

use fret_core::Size;
use fret_runtime::ActionId;
use fret_ui::UiHost;

use super::{
    ButtonArrowDirection, ButtonOptions, ButtonVariant, ResponseExt, UiWriterImUiFacadeExt,
};

pub(super) fn button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
) -> ResponseExt {
    entry::button_impl(ui, label, options, None)
}

pub(super) fn small_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Small;
    entry::button_impl(ui, label, options, None)
}

pub(super) fn arrow_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    direction: ButtonArrowDirection,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Arrow(direction);
    ui.push_id(id, |ui| {
        entry::button_impl(ui, Arc::from(""), options, None)
    })
}

pub(super) fn invisible_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    size: Size,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Invisible { size };
    ui.push_id(id, |ui| {
        entry::button_impl(ui, Arc::from(""), options, None)
    })
}

pub(super) fn action_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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

pub(super) fn action_payload_button_with_options<
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
