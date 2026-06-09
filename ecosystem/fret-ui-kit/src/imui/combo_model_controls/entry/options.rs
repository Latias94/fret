use crate::imui::{ComboModelOptions, ComboOptions};

pub(super) fn combo_model_trigger_options(
    options: &ComboModelOptions,
    enabled: bool,
) -> ComboOptions {
    ComboOptions {
        enabled,
        focusable: options.focusable,
        a11y_label: options.a11y_label.clone(),
        test_id: options.test_id.clone(),
        popup: options.popup,
        activate_shortcut: options.activate_shortcut,
        shortcut_repeat: options.shortcut_repeat,
    }
}
