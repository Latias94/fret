use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{InputTextOptions, ResponseExt};
use super::keyboard::InputTextPickerKeyboardState;
use keyboard::{InputRootKeyboardHandlerRequest, install_input_root_keyboard_handler};
use semantics::input_root_assistive_semantics;

mod keyboard;
mod options;
mod semantics;

pub(super) use options::prepare_text_picker_input_options;

pub(super) struct InputTextPickerInputRootRequest<'a> {
    pub(super) model: Model<String>,
    pub(super) input_options: InputTextOptions,
    pub(super) popup_open: Model<bool>,
    pub(super) keyboard_state: Option<Model<InputTextPickerKeyboardState>>,
    pub(super) visible_candidates: &'a [(usize, Arc<str>)],
    pub(super) keyboard_navigation: bool,
    pub(super) keyboard_repeat: bool,
    pub(super) picker_candidate_visible: bool,
    pub(super) hide_for_exact_match: bool,
    pub(super) picker_expanded: bool,
    pub(super) active_element: Option<GlobalElementId>,
    pub(super) popup_panel_id: Option<GlobalElementId>,
}

pub(super) struct BuiltInputTextPickerInputRoot {
    pub(super) root: AnyElement,
    pub(super) response: ResponseExt,
}

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
