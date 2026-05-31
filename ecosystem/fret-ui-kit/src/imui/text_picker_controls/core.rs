use std::sync::Arc;

use fret_ui::UiHost;

mod input_root;
mod keyboard_state;
mod popup;
mod session;

use super::super::{InputTextPickerOptions, InputTextPickerResponse, UiWriterImUiFacadeExt};
use super::open_policy::{TextPickerOpenPolicyInput, apply_text_picker_open_policy};
use super::response::finish_text_picker_response;

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
    let input = input_root.input;
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

    let popup_result = popup::render_text_picker_core_popup(
        ui,
        popup::TextPickerCorePopupInput {
            id,
            model: model.clone(),
            options: &options,
            session: &session,
            trigger: input.id(),
            input_enabled: enabled,
            input_focused,
            item_test_id_base: input_root.item_test_id_base,
        },
    );

    finish_text_picker_response(ui, model, input, popup_result)
}
