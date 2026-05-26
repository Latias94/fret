//! Immediate-mode button-style pressable helpers.

mod behavior;
mod visual;

use std::{any::Any, sync::Arc};

use fret_core::Size;
use fret_runtime::ActionId;
use fret_ui::UiHost;

use super::label_identity::parse_label_identity;
use super::{
    ButtonArrowDirection, ButtonOptions, ButtonVariant, ResponseExt, UiWriterImUiFacadeExt,
};

pub(super) fn button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
) -> ResponseExt {
    button_impl(ui, label, options, None)
}

pub(super) fn small_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Small;
    button_impl(ui, label, options, None)
}

pub(super) fn arrow_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    direction: ButtonArrowDirection,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Arrow(direction);
    ui.push_id(id, |ui| button_impl(ui, Arc::from(""), options, None))
}

pub(super) fn invisible_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    size: Size,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Invisible { size };
    ui.push_id(id, |ui| button_impl(ui, Arc::from(""), options, None))
}

pub(super) fn action_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: ButtonOptions,
) -> ResponseExt {
    button_impl(
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
    button_impl(
        ui,
        label,
        options,
        Some(behavior::ButtonAction {
            action,
            payload: Some(payload),
        }),
    )
}

fn button_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
    action: Option<behavior::ButtonAction>,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("button-label", identity), |ui| {
        behavior::button_pressable(ui, visible_label, options, action)
    })
}
