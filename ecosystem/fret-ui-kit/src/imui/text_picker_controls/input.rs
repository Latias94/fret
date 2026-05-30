use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{InputTextOptions, ResponseExt};
use super::keyboard::{InputTextPickerKeyboardState, install_picker_keyboard_handler};

mod options;

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
    let assistive_semantics = super::super::text_controls::InputTextAssistiveSemantics {
        active_descendant: None,
        active_descendant_element: request
            .picker_expanded
            .then_some(request.active_element)
            .flatten()
            .map(|element| element.0),
        controls_element: request
            .picker_expanded
            .then_some(request.popup_panel_id)
            .flatten()
            .map(|element| element.0),
        expanded: Some(request.picker_expanded),
    };

    let mut response = ResponseExt::default();
    let input_element =
        super::super::text_controls::input_text_model_element_with_options_and_semantics(
            cx,
            request.model.clone(),
            request.input_options,
            assistive_semantics,
            &mut response,
        );

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    let root = cx.container(props, |_cx| vec![input_element]);

    if response.enabled()
        && request.keyboard_navigation
        && response.focused()
        && request.picker_candidate_visible
        && !request.hide_for_exact_match
        && let Some(state) = request.keyboard_state.clone()
    {
        install_picker_keyboard_handler(
            cx,
            root.id,
            request.model,
            request.popup_open,
            state,
            request.visible_candidates.to_vec(),
            request.keyboard_repeat,
        );
    }

    BuiltInputTextPickerInputRoot { root, response }
}
