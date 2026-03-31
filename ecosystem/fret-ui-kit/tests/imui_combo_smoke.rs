#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{ComboModelOptions, ComboOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn combo_api_compiles<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    model: &fret_runtime::Model<Option<std::sync::Arc<str>>>,
) {
    let _ = ui.combo("combo.basic", "Theme", "Dark", |_ui| {});
    let _ = ui.combo_with_options(
        "combo.with_options",
        "Density",
        "Compact",
        ComboOptions {
            test_id: Some("combo.density".into()),
            ..Default::default()
        },
        |_ui| {},
    );
    let _ = ui.combo_model_with_options(
        "combo.model",
        "Mode",
        model,
        &[],
        ComboModelOptions {
            test_id: Some("combo.model.trigger".into()),
            ..Default::default()
        },
    );
}

#[test]
fn combo_option_defaults_compile() {
    let options = ComboOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert!(options.a11y_label.is_none());
    assert!(options.test_id.is_none());
}

#[test]
fn combo_model_option_defaults_compile() {
    let options = ComboModelOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert_eq!(options.placeholder.as_deref(), Some("Select..."));
    assert!(options.test_id.is_none());
}
