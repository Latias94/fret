use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::super::InputTextOptions;
use crate::imui::ResponseExt;
use crate::imui::text_controls::{
    InputTextAssistiveSemantics, input_text_model_element_with_options_and_semantics,
};

pub(super) fn build_text_picker_input_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    input_options: InputTextOptions,
    assistive_semantics: InputTextAssistiveSemantics,
) -> (AnyElement, ResponseExt) {
    let mut response = ResponseExt::default();
    let input_element = input_text_model_element_with_options_and_semantics(
        cx,
        model,
        input_options,
        assistive_semantics,
        &mut response,
    );

    (input_element, response)
}
