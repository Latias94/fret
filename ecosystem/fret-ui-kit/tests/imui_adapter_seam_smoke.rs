#![cfg(feature = "imui")]

use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui_kit::imui::UiWriterImUiFacadeExt;
use fret_ui_kit::imui::adapters::{
    AdapterSeamOptions, AdapterSignalRecord, button_adapter, checkbox_model_adapter,
};

#[allow(dead_code)]
fn adapter_examples_compile<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    model: &Model<bool>,
) {
    let mut records: Vec<AdapterSignalRecord> = Vec::new();
    let mut reporter = |record: AdapterSignalRecord| records.push(record);

    let _ = button_adapter(
        ui,
        "adapter.button",
        "Run",
        AdapterSeamOptions {
            reporter: Some(&mut reporter),
            focus_restore_target: None,
        },
    );

    let _ = checkbox_model_adapter(
        ui,
        "adapter.checkbox",
        "Enabled",
        model,
        AdapterSeamOptions {
            reporter: Some(&mut reporter),
            focus_restore_target: None,
        },
    );
}

#[test]
fn adapter_seam_option_defaults_compile() {
    let options = AdapterSeamOptions::default();
    assert!(options.reporter.is_none());
    assert!(options.focus_restore_target.is_none());
}
