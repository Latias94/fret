use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::UiHost;
use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;

use crate::imui::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};

use super::super::{identity, mount};

pub(super) fn noop_menu_item_pressable_hook<H: UiHost>(
    _cx: &mut fret_ui::ElementContext<'_, H>,
    _state: PressableState,
    _item_id: GlobalElementId,
    _enabled: bool,
) {
}

pub(super) fn menu_item_impl_with_pressable_hook<
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
    identity::with_menu_item_label_identity(ui, label, |ui, visible_label| {
        mount::mount_menu_item_with_pressable_hook(
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
