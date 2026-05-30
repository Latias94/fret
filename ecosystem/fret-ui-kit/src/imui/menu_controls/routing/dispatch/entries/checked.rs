use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;

use crate::imui::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};

pub(in crate::imui) fn menu_item_checkbox_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    checked: bool,
    options: MenuItemOptions,
) -> ResponseExt {
    super::menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItemCheckbox,
        Some(checked),
        None,
    )
}

pub(in crate::imui) fn menu_item_radio_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    checked: bool,
    options: MenuItemOptions,
) -> ResponseExt {
    super::menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItemRadio,
        Some(checked),
        None,
    )
}
