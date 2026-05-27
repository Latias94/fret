//! Immediate-mode input text picker recipes.

use std::sync::Arc;

use fret_ui::UiHost;

use super::{InputTextPickerOptions, InputTextPickerResponse, UiWriterImUiFacadeExt};

mod candidates;
mod entry;
mod input;
mod keyboard;
mod open_policy;
mod popup;
mod response;

use candidates::resolve_text_picker_candidates;
use input::{
    InputTextPickerInputRootRequest, prepare_text_picker_input_options,
    render_text_picker_input_root,
};
use keyboard::{InputTextPickerKeyboardState, reconcile_picker_keyboard_state};
use open_policy::{
    TextPickerOpenPolicyInput, apply_text_picker_open_policy, read_text_picker_popup_snapshot,
    text_picker_expanded,
};
use popup::{InputTextPickerPopupInput, render_text_picker_popup};
use response::merge_text_picker_pick_response;

pub(super) use entry::{
    input_text_completion_model_with_options, input_text_history_model_with_options,
};

pub(super) fn input_text_picker_model_with_options<
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

    let prepared_input = prepare_text_picker_input_options(&options);

    let candidate_visibility = resolve_text_picker_candidates(&current, candidates, &options);
    let visible_candidates = candidate_visibility.visible_candidates;
    let hide_for_exact_match = candidate_visibility.hide_for_exact_match;
    let popup_open = ui.popup_open_model(id);
    let picker_candidate_visible = candidate_visibility.picker_candidate_visible;
    let input_enabled_by_scope =
        ui.with_cx_mut(|cx| options.input.enabled && !super::imui_is_disabled(cx));
    let keyboard_state = options.keyboard_navigation.then(|| {
        ui.with_cx_mut(|cx| {
            cx.local_model_keyed(
                format!("fret-ui-kit.imui.input-text-picker.keyboard.{id}"),
                InputTextPickerKeyboardState::default,
            )
        })
    });
    let popup_snapshot = read_text_picker_popup_snapshot(ui, id, &popup_open);
    let (active_source_index, pending_keyboard_pick, active_element) = keyboard_state
        .as_ref()
        .and_then(|state| {
            ui.with_cx_mut(|cx| {
                reconcile_picker_keyboard_state(
                    cx,
                    state,
                    input_enabled_by_scope,
                    &visible_candidates,
                    hide_for_exact_match,
                )
            })
        })
        .map(|snapshot| {
            (
                snapshot.active_source_index,
                snapshot.pending_pick,
                snapshot.active_element,
            )
        })
        .unwrap_or((None, None, None));
    let picker_expanded = text_picker_expanded(
        popup_snapshot.is_open,
        input_enabled_by_scope,
        picker_candidate_visible,
        hide_for_exact_match,
    );
    let input_root = ui.with_cx_mut(|cx| {
        render_text_picker_input_root(
            cx,
            InputTextPickerInputRootRequest {
                model: model.clone(),
                input_options: prepared_input.options,
                popup_open: popup_open.clone(),
                keyboard_state: keyboard_state.clone(),
                visible_candidates: &visible_candidates,
                keyboard_navigation: options.keyboard_navigation,
                keyboard_repeat: options.keyboard_repeat,
                picker_candidate_visible,
                hide_for_exact_match,
                picker_expanded,
                active_element,
                popup_panel_id: popup_snapshot.panel_id,
            },
        )
    });
    let mut input = input_root.response;
    ui.add(input_root.root);
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
            keyboard_state: keyboard_state.clone(),
            visible_candidates: &visible_candidates,
            selected_value: current.clone(),
            active_source_index,
            pending_keyboard_pick,
            item_test_id_base: prepared_input.test_id.clone(),
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
