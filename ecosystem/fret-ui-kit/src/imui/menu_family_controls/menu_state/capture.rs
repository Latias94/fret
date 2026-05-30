use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

use super::super::ImUiMenubarPolicyState;

mod read;
mod state;

pub(in crate::imui::menu_family_controls) use read::{read_bool_model, read_open_menu_model};
pub(in crate::imui::menu_family_controls) use state::{BeginMenuState, record_render_state};

pub(in crate::imui::menu_family_controls) fn capture_begin_menu_state<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
) -> BeginMenuState {
    let menubar_policy = ui.with_cx_mut(|cx| cx.provided::<ImUiMenubarPolicyState>().cloned());
    let popup_open = ui.popup_open_model(id);
    let row_open = if menubar_policy.is_some() {
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("menubar_row_open.{id}"), || false))
    } else {
        popup_open.clone()
    };
    let was_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_open.{id}"), || false));
    let was_popup_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_popup_open.{id}"), || false));
    let open_before = read_bool_model(ui, &row_open);
    let popup_open_before = read_bool_model(ui, &popup_open);
    let was_open_before_render = read_bool_model(ui, &was_open_model);
    let was_popup_open_before_render = read_bool_model(ui, &was_popup_open_model);

    BeginMenuState {
        menubar_policy,
        popup_open,
        row_open,
        was_open_model,
        was_popup_open_model,
        open_before,
        popup_open_before,
        was_open_before_render,
        was_popup_open_before_render,
    }
}
