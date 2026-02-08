#![cfg(feature = "imui")]

use std::hash::Hash;
use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui_kit::imui::adapters::{AdapterSeamOptions, AdapterSignalRecord, report_adapter_signal};
use fret_ui_kit::imui::{ResponseExt, UiWriterImUiFacadeExt};

mod external_widget_crate_scaffold {
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
}

#[allow(dead_code)]
fn external_adapter_example_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let mut records: Vec<AdapterSignalRecord> = Vec::new();
    let mut reporter = |record: AdapterSignalRecord| records.push(record);

    let _ = external_widget_crate_scaffold::toolbar_button_adapter(
        ui,
        "external.toolbar.button",
        "Run",
        AdapterSeamOptions {
            reporter: Some(&mut reporter),
            focus_restore_target: None,
        },
    );
}
