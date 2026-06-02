use std::sync::Arc;

use fret_ui::{Invalidation, UiHost};

use crate::imui::{UiWriterImUiFacadeExt, popup_overlay::ImUiPopupMenuPolicyState};

pub(super) fn read_open_submenu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    policy: &ImUiPopupMenuPolicyState,
) -> Option<Arc<str>> {
    ui.with_cx_mut(|cx| {
        cx.read_model(
            &policy.submenu_models.open_value,
            Invalidation::Paint,
            |_app, value| value.clone(),
        )
        .unwrap_or(None)
    })
}
