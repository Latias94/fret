use std::sync::Arc;

use fret_core::Size;
use fret_ui::UiHost;

use super::super::{
    ButtonArrowDirection, ButtonOptions, ButtonVariant, ResponseExt, UiWriterImUiFacadeExt,
};
use super::entry;

pub(in crate::imui) fn button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
) -> ResponseExt {
    entry::button_impl(ui, label, options, None)
}

pub(in crate::imui) fn small_button_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Small;
    entry::button_impl(ui, label, options, None)
}

pub(in crate::imui) fn arrow_button_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
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

pub(in crate::imui) fn invisible_button_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
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
