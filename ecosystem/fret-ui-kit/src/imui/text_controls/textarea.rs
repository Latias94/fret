use fret_ui::UiHost;

mod element;
mod props;

use crate::imui::{ResponseExt, TextAreaOptions, UiWriterImUiFacadeExt};
use element::textarea_model_element_with_options;

pub(in crate::imui) fn textarea_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: TextAreaOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();
    let element =
        ui.with_cx_mut(|cx| textarea_model_element_with_options(cx, model, options, &mut response));

    ui.add(element);
    response
}
