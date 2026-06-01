use std::sync::Arc;

use fret_core::Rect;
use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;

use super::super::menu_state::{self, BeginMenuState};

pub(in crate::imui::menu_family_controls) fn open_begin_menu_popup_if_requested<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_menu_before: Option<Arc<str>>,
    trigger_id: Option<GlobalElementId>,
    trigger_rect: Option<Rect>,
) {
    let open_requested = menu_state::resolve_open_requested(ui, id, state, open_menu_before);

    menu_state::activate_menubar_trigger_if_requested(ui, open_requested, state, trigger_id);
    if open_requested && let Some(anchor) = trigger_rect {
        ui.open_popup_at(id, anchor);
    }
}
