use std::sync::Arc;

use fret_core::SemanticsRole;

use super::super::super::{InputTextOptions, InputTextPickerOptions};

pub(in crate::imui::text_picker_controls) struct PreparedInputTextPickerInput {
    pub(in crate::imui::text_picker_controls) test_id: Option<Arc<str>>,
    pub(in crate::imui::text_picker_controls) options: InputTextOptions,
}

pub(in crate::imui::text_picker_controls) fn prepare_text_picker_input_options(
    options: &InputTextPickerOptions,
) -> PreparedInputTextPickerInput {
    let test_id = options
        .test_id
        .clone()
        .or_else(|| options.input.test_id.clone());
    let mut input_options = options.input.clone();
    if input_options.test_id.is_none() {
        input_options.test_id = test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.input")));
    }
    if matches!(input_options.a11y_role, Some(SemanticsRole::TextField)) {
        input_options.a11y_role = Some(SemanticsRole::ComboBox);
    }

    PreparedInputTextPickerInput {
        test_id,
        options: input_options,
    }
}
