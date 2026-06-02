use fret_ui::UiHost;

mod element;
mod props;

use element::input_text_model_element_with_options;

pub(in crate::imui) use element::input_text_model_element_with_options_and_semantics;
pub(in crate::imui) use props::InputTextAssistiveSemantics;

use super::super::{InputTextOptions, ResponseExt, UiWriterImUiFacadeExt};

pub(super) fn text_model_changed_for<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    current: &str,
) -> bool {
    super::super::model_value_changed_for(cx, id, current.to_string())
}

pub(in crate::imui) fn input_text_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: InputTextOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();
    let element = ui
        .with_cx_mut(|cx| input_text_model_element_with_options(cx, model, options, &mut response));

    ui.add(element);
    response
}
