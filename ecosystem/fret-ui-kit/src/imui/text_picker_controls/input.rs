use fret_ui::element::{ContainerProps, Length};
use fret_ui::{ElementContext, UiHost};

use super::super::ResponseExt;
use keyboard::{InputRootKeyboardHandlerRequest, install_input_root_keyboard_handler};
use semantics::input_root_assistive_semantics;

mod keyboard;
mod options;
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

    let mut response = ResponseExt::default();
    let input_element =
        super::super::text_controls::input_text_model_element_with_options_and_semantics(
            cx,
            model.clone(),
            input_options,
            assistive_semantics,
            &mut response,
        );

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    let root = cx.container(props, |_cx| vec![input_element]);

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
