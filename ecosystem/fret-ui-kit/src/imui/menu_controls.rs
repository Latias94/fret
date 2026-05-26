//! Immediate-mode menu-item helpers.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::UiHost;
use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;

use super::label_identity::parse_label_identity;
use super::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};

mod element;
mod visual;

pub(super) fn menu_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_with_options_and_pressable_hook(
        ui,
        label,
        options,
        noop_menu_item_pressable_hook::<H>,
    )
}

pub(super) fn menu_item_checkbox_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    checked: bool,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItemCheckbox,
        Some(checked),
        None,
    )
}

pub(super) fn menu_item_radio_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    checked: bool,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItemRadio,
        Some(checked),
        None,
    )
}

pub(super) fn menu_item_action_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItem,
        None,
        Some(action),
    )
}

fn menu_item_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
) -> ResponseExt {
    menu_item_impl_with_pressable_hook(
        ui,
        label,
        options,
        role,
        checked,
        action,
        noop_menu_item_pressable_hook::<H>,
    )
}

pub(super) fn menu_item_with_options_and_pressable_hook<
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
    menu_item_impl_with_pressable_hook(
        ui,
        label,
        options,
        SemanticsRole::MenuItem,
        None,
        None,
        pressable_hook,
    )
}

fn noop_menu_item_pressable_hook<H: UiHost>(
    _cx: &mut fret_ui::ElementContext<'_, H>,
    _state: PressableState,
    _item_id: GlobalElementId,
    _enabled: bool,
) {
}

fn menu_item_impl_with_pressable_hook<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, F>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
    pressable_hook: F,
) -> ResponseExt
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("menu-item-label", identity), |ui| {
        menu_item_impl_with_pressable_hook_inner(
            ui,
            visible_label,
            options,
            role,
            checked,
            action,
            pressable_hook,
        )
    })
}

fn menu_item_impl_with_pressable_hook_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, F>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
    pressable_hook: F,
) -> ResponseExt
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        element::menu_item_element_with_pressable_hook_inner(
            cx,
            label,
            options,
            role,
            checked,
            action,
            pressable_hook,
            &mut response,
        )
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
