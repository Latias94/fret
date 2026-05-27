use std::sync::Arc;

use fret_ui::UiHost;

use super::super::{
    InputTextPickerFilter, InputTextPickerOptions, InputTextPickerResponse, UiWriterImUiFacadeExt,
};

pub(in crate::imui) fn input_text_completion_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    candidates: &[Arc<str>],
    options: InputTextPickerOptions,
) -> InputTextPickerResponse {
    super::input_text_picker_model_with_options(ui, id, model, candidates, options)
}

pub(in crate::imui) fn input_text_history_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    history: &[Arc<str>],
    mut options: InputTextPickerOptions,
) -> InputTextPickerResponse {
    options.filter = InputTextPickerFilter::None;
    options.open_when_empty = true;
    options.hide_when_exact_match = false;
    super::input_text_picker_model_with_options(ui, id, model, history, options)
}
