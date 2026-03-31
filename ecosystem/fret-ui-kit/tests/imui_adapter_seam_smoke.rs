#![cfg(feature = "imui")]

use std::hash::Hash;
use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui_kit::imui::ResponseExt;
use fret_ui_kit::imui::UiWriterImUiFacadeExt;
use fret_ui_kit::imui::adapters::{AdapterSeamOptions, AdapterSignalRecord, report_adapter_signal};

const ADAPTERS_RS: &str = include_str!("../src/imui/adapters.rs");

mod local_adapter_scaffold {
    use super::*;

    pub fn toolbar_button_adapter<H: UiHost, K: Hash>(
        ui: &mut impl UiWriterImUiFacadeExt<H>,
        identity_key: K,
        label: impl Into<Arc<str>>,
        mut options: AdapterSeamOptions<'_>,
    ) -> ResponseExt {
        let label = label.into();
        let response = ui.push_id(identity_key, |ui| ui.button(label.clone()));
        report_adapter_signal(response, &mut options)
    }

    pub fn toggle_adapter<H: UiHost, K: Hash>(
        ui: &mut impl UiWriterImUiFacadeExt<H>,
        identity_key: K,
        label: impl Into<Arc<str>>,
        model: &Model<bool>,
        mut options: AdapterSeamOptions<'_>,
    ) -> ResponseExt {
        let label = label.into();
        let response = ui.push_id(identity_key, |ui| ui.checkbox_model(label.clone(), model));
        report_adapter_signal(response, &mut options)
    }
}

#[allow(dead_code)]
fn adapter_examples_compile<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    model: &Model<bool>,
) {
    let mut records: Vec<AdapterSignalRecord> = Vec::new();
    let mut reporter = |record: AdapterSignalRecord| records.push(record);

    let _ = local_adapter_scaffold::toolbar_button_adapter(
        ui,
        "adapter.button",
        "Run",
        AdapterSeamOptions {
            reporter: Some(&mut reporter),
            focus_restore_target: None,
        },
    );

    let _ = local_adapter_scaffold::toggle_adapter(
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

#[test]
fn adapter_seam_module_stays_contract_only() {
    for marker in [
        "pub struct AdapterSignalMetadata",
        "pub struct AdapterSignalRecord",
        "pub struct AdapterSeamOptions<'a>",
        "pub fn report_adapter_signal(",
    ] {
        assert!(
            ADAPTERS_RS.contains(marker),
            "adapters.rs should keep the minimal seam contract explicit"
        );
    }

    for marker in ["pub fn button_adapter", "pub fn checkbox_model_adapter"] {
        assert!(
            !ADAPTERS_RS.contains(marker),
            "adapters.rs should not grow built-in sample wrappers: {marker}"
        );
    }

    for marker in ["fret_imui", "ImUi<"] {
        assert!(
            !ADAPTERS_RS.contains(marker),
            "adapters.rs should stay free of concrete frontend coupling: {marker}"
        );
    }
}
