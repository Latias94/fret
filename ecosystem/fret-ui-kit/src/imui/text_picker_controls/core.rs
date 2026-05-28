use std::sync::Arc;

use fret_ui::UiHost;

mod input_root;
mod keyboard_state;

use keyboard_state::prepare_text_picker_keyboard;

use super::super::{InputTextPickerOptions, InputTextPickerResponse, UiWriterImUiFacadeExt};
use super::candidates::resolve_text_picker_candidates;
use super::open_policy::{
    TextPickerOpenPolicyInput, apply_text_picker_open_policy, read_text_picker_popup_snapshot,
    text_picker_expanded,
};
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
    let current = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
            value.clone()
        })
        .unwrap_or_default()
    });

    let candidate_visibility = resolve_text_picker_candidates(&current, candidates, &options);
    let visible_candidates = candidate_visibility.visible_candidates;
    let hide_for_exact_match = candidate_visibility.hide_for_exact_match;
    let popup_open = ui.popup_open_model(id);
    let picker_candidate_visible = candidate_visibility.picker_candidate_visible;
    let input_enabled_by_scope =
        ui.with_cx_mut(|cx| options.input.enabled && !super::super::imui_is_disabled(cx));
    let keyboard = prepare_text_picker_keyboard(
        ui,
        id,
        options.keyboard_navigation,
        input_enabled_by_scope,
        &visible_candidates,
        hide_for_exact_match,
    );
    let popup_snapshot = read_text_picker_popup_snapshot(ui, id, &popup_open);
    let picker_expanded = text_picker_expanded(
        popup_snapshot.is_open,
        input_enabled_by_scope,
        picker_candidate_visible,
        hide_for_exact_match,
    );
    let input_root = input_root::build_text_picker_input_root(
        ui,
        input_root::TextPickerInputRootInput {
            model: model.clone(),
            options: &options,
            popup_open: popup_open.clone(),
            keyboard_state: keyboard.state.clone(),
            visible_candidates: &visible_candidates,
            keyboard_navigation: options.keyboard_navigation,
            keyboard_repeat: options.keyboard_repeat,
            picker_candidate_visible,
            hide_for_exact_match,
            picker_expanded,
            active_element: keyboard.active_element,
            popup_panel_id: popup_snapshot.panel_id,
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
            visible_candidates_empty: visible_candidates.is_empty(),
            hide_for_exact_match,
            open_on_focus: options.open_on_focus,
            input_focused,
            picker_candidate_visible,
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
            popup_open: popup_open.clone(),
            keyboard_state: keyboard.state.clone(),
            visible_candidates: &visible_candidates,
            selected_value: current.clone(),
            active_source_index: keyboard.active_source_index,
            pending_keyboard_pick: keyboard.pending_keyboard_pick,
            item_test_id_base: input_root.item_test_id_base,
            install_keyboard_handler: enabled
                && options.keyboard_navigation
                && input_focused
                && picker_candidate_visible
                && !hide_for_exact_match,
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
