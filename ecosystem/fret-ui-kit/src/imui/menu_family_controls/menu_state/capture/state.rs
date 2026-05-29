use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

use super::super::super::ImUiMenubarPolicyState;
use super::read::{read_bool_model, read_open_menu_model};

pub(in crate::imui::menu_family_controls) struct BeginMenuState {
    pub(in crate::imui::menu_family_controls) menubar_policy: Option<ImUiMenubarPolicyState>,
    pub(in crate::imui::menu_family_controls) popup_open: Model<bool>,
    pub(in crate::imui::menu_family_controls) row_open: Model<bool>,
    pub(in crate::imui::menu_family_controls) was_open_model: Model<bool>,
    pub(in crate::imui::menu_family_controls) was_popup_open_model: Model<bool>,
    pub(in crate::imui::menu_family_controls) open_before: bool,
    pub(in crate::imui::menu_family_controls) popup_open_before: bool,
    pub(in crate::imui::menu_family_controls) was_open_before_render: bool,
    pub(in crate::imui::menu_family_controls) was_popup_open_before_render: bool,
}

pub(in crate::imui::menu_family_controls) struct MenuRenderState {
    pub(in crate::imui::menu_family_controls) popup_open_after: bool,
}

impl BeginMenuState {
    pub(in crate::imui::menu_family_controls) fn read_row_open<
        H: UiHost,
        W: UiWriterImUiFacadeExt<H> + ?Sized,
    >(
        &self,
        ui: &mut W,
    ) -> bool {
        read_bool_model(ui, &self.row_open)
    }

    pub(in crate::imui::menu_family_controls) fn read_menubar_open_menu<
        H: UiHost,
        W: UiWriterImUiFacadeExt<H> + ?Sized,
    >(
        &self,
        ui: &mut W,
    ) -> Option<Arc<str>> {
        self.menubar_policy
            .as_ref()
            .and_then(|policy| read_open_menu_model(ui, policy))
    }
}

pub(in crate::imui::menu_family_controls) fn record_render_state<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    state: &BeginMenuState,
) -> MenuRenderState {
    let open_after = read_bool_model(ui, &state.row_open);
    let popup_open_after = read_bool_model(ui, &state.popup_open);
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&state.was_open_model, |value| *value = open_after);
        let _ = cx
            .app
            .models_mut()
            .update(&state.was_popup_open_model, |value| {
                *value = popup_open_after
            });
    });

    MenuRenderState { popup_open_after }
}
