use fret_ui::{ElementContext, UiHost};

use element::build_text_picker_input_element;
use keyboard::{InputRootKeyboardHandlerRequest, install_input_root_keyboard_handler};
use root::build_text_picker_input_root_container;
use semantics::input_root_assistive_semantics;

mod element;
mod keyboard;
mod options;
mod root;
mod semantics;
mod types;

pub(super) use options::prepare_text_picker_input_options;
pub(super) use types::{BuiltInputTextPickerInputRoot, InputTextPickerInputRootRequest};

pub(super) fn render_text_picker_input_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: InputTextPickerInputRootRequest<'_>,
) -> BuiltInputTextPickerInputRoot {
    let InputTextPickerInputRootRequest {
        model,
        input_options,
        popup_open,
        keyboard_state,
        visible_candidates,
        keyboard_navigation,
        keyboard_repeat,
        picker_candidate_visible,
        hide_for_exact_match,
        picker_expanded,
        active_element,
        popup_panel_id,
    } = request;

    let assistive_semantics =
        input_root_assistive_semantics(picker_expanded, active_element, popup_panel_id);

    let (input_element, response) =
        build_text_picker_input_element(cx, model.clone(), input_options, assistive_semantics);
    let root = build_text_picker_input_root_container(cx, input_element);

    install_input_root_keyboard_handler(
        cx,
        root.id,
        &response,
        InputRootKeyboardHandlerRequest {
            model,
            popup_open,
            keyboard_state,
            visible_candidates,
            keyboard_navigation,
            keyboard_repeat,
            picker_candidate_visible,
            hide_for_exact_match,
        },
    );

    BuiltInputTextPickerInputRoot { root, response }
}
