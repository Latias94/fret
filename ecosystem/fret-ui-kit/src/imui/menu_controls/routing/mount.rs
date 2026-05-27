use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::UiHost;
use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;

use crate::imui::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};

use super::super::element;

pub(super) fn mount_menu_item_with_pressable_hook<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    F,
>(
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
