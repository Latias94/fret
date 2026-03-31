#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{SeparatorTextOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn separator_text_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    ui.separator_text("General");
    ui.separator_text_with_options(
        "Advanced",
        SeparatorTextOptions {
            test_id: Some("separator.advanced".into()),
        },
    );
}

#[test]
fn separator_text_option_defaults_compile() {
    let options = SeparatorTextOptions::default();
    assert!(options.test_id.is_none());
}
