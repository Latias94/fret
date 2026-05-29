use std::sync::Arc;

use fret_ui::UiHost;

mod input_root;
mod keyboard_state;
mod session;

use super::super::{InputTextPickerOptions, InputTextPickerResponse, UiWriterImUiFacadeExt};
use super::open_policy::{TextPickerOpenPolicyInput, apply_text_picker_open_policy};
use super::popup::{InputTextPickerPopupInput, render_text_picker_popup};
use super::response::merge_text_picker_pick_response;

pub(in crate::imui) fn input_text_picker_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    candidates: &[Arc<str>],
    options: InputTextPickerOptions,
) -> InputTextPickerResponse {
    let session = session::prepare_text_picker_session(ui, id, model, candidates, &options);
    let input_root = input_root::build_text_picker_input_root(
        ui,
        input_root::TextPickerInputRootInput {
            model: model.clone(),
            options: &options,
            popup_open: session.popup_open.clone(),
            keyboard_state: session.keyboard.state.clone(),
            visible_candidates: &session.visible_candidates,
            keyboard_navigation: options.keyboard_navigation,
            keyboard_repeat: options.keyboard_repeat,
            picker_candidate_visible: session.picker_candidate_visible,
            hide_for_exact_match: session.hide_for_exact_match,
            picker_expanded: session.picker_expanded,
            active_element: session.keyboard.active_element,
            popup_panel_id: session.popup_snapshot.panel_id,
        },
    );
    let mut input = input_root.input;
    let enabled = input.enabled();
    let input_focused = input.focused();

    apply_text_picker_open_policy(
        ui,
        id,
        TextPickerOpenPolicyInput {
            enabled,
            visible_candidates_empty: session.visible_candidates.is_empty(),
            hide_for_exact_match: session.hide_for_exact_match,
            open_on_focus: options.open_on_focus,
            input_focused,
            picker_candidate_visible: session.picker_candidate_visible,
            anchor: input.rect(),
        },
    );

    let popup = render_text_picker_popup(
        ui,
        InputTextPickerPopupInput {
            id,
            trigger: input.id(),
            popup: options.popup,
            model: model.clone(),
            popup_open: session.popup_open.clone(),
            keyboard_state: session.keyboard.state.clone(),
            visible_candidates: &session.visible_candidates,
            selected_value: session.current.clone(),
            active_source_index: session.keyboard.active_source_index,
            pending_keyboard_pick: session.keyboard.pending_keyboard_pick,
            item_test_id_base: input_root.item_test_id_base,
            install_keyboard_handler: enabled
                && options.keyboard_navigation
                && input_focused
                && session.picker_candidate_visible
                && !session.hide_for_exact_match,
            keyboard_repeat: options.keyboard_repeat,
        },
    );
    let opened = popup.opened;
    let picked_index = popup.picked_index;
    let picked = popup.picked;

    merge_text_picker_pick_response(ui, model, &mut input, picked.is_some());

    InputTextPickerResponse {
        input,
        open: opened,
        picked_index,
        picked,
    }
}
