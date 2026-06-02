use fret_ui::{GlobalElementId, Invalidation, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

pub(in crate::imui::menu_family_controls) fn trigger_is_active<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    policy: &ImUiMenubarPolicyState,
    trigger_id: GlobalElementId,
) -> bool {
    ui.with_cx_mut(|cx| {
        cx.read_model(
            &policy.group_active,
            Invalidation::Paint,
            |_app, value: &Option<menubar_trigger_row::MenubarActiveTrigger>| {
                value
                    .as_ref()
                    .is_some_and(|active| active.trigger == trigger_id)
            },
        )
        .unwrap_or(false)
    })
}
