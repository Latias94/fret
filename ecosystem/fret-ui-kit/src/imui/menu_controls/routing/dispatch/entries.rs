use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;

use crate::imui::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};

use super::core;

mod action;
mod checked;

pub(in crate::imui) use action::menu_item_action_with_options;
pub(in crate::imui) use checked::{menu_item_checkbox_with_options, menu_item_radio_with_options};

pub(in crate::imui) fn menu_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_with_options_and_pressable_hook(
        ui,
        label,
        options,
        core::noop_menu_item_pressable_hook::<H>,
    )
}

fn menu_item_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<fret_runtime::ActionId>,
) -> ResponseExt {
    core::menu_item_impl_with_pressable_hook(
        ui,
        label,
        options,
        role,
        checked,
        action,
        core::noop_menu_item_pressable_hook::<H>,
    )
}

pub(in crate::imui) fn menu_item_with_options_and_pressable_hook<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    F,
>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    pressable_hook: F,
) -> ResponseExt
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    core::menu_item_impl_with_pressable_hook(
        ui,
        label,
        options,
        SemanticsRole::MenuItem,
        None,
        None,
        pressable_hook,
    )
}
