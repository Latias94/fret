use std::sync::Arc;

use super::super::super::super::{InputTextOptions, InputTextPickerOptions};
use super::super::super::input::prepare_text_picker_input_options;

pub(super) struct PreparedCoreTextPickerInputOptions {
    pub(super) options: InputTextOptions,
    pub(super) item_test_id_base: Option<Arc<str>>,
}

pub(super) fn prepare_core_text_picker_input_options(
    options: &InputTextPickerOptions,
) -> PreparedCoreTextPickerInputOptions {
    let prepared_input = prepare_text_picker_input_options(options);
    PreparedCoreTextPickerInputOptions {
        options: prepared_input.options,
        item_test_id_base: prepared_input.test_id,
    }
}
