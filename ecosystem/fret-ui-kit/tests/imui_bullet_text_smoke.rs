#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{BulletTextOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn bullet_text_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    ui.bullet_text("Default bullet");
    ui.bullet_text_with_options(
        "Configured bullet",
        BulletTextOptions {
            test_id: Some("bullet.configured".into()),
        },
    );
}

#[test]
fn bullet_text_option_defaults_compile() {
    let options = BulletTextOptions::default();
    assert!(options.test_id.is_none());
}
