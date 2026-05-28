use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

use super::super::super::ImUiMenubarPolicyState;

pub(in crate::imui::menu_family_controls) fn read_bool_model<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    model: &Model<bool>,
) -> bool {
    ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| *value)
            .unwrap_or(false)
    })
}

pub(in crate::imui::menu_family_controls) fn read_open_menu_model<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    policy: &ImUiMenubarPolicyState,
) -> Option<Arc<str>> {
    ui.with_cx_mut(|cx| {
        cx.read_model(
            &policy.open_menu,
            fret_ui::Invalidation::Paint,
            |_app, value| value.clone(),
        )
        .unwrap_or(None)
    })
}
