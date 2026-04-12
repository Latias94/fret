#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{ChildRegionOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn child_region_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    ui.child_region("child-region", |_ui| {});
    ui.child_region_with_options(
        "child-region.with-options",
        ChildRegionOptions::default(),
        |_ui| {},
    );
}

#[test]
fn child_region_option_defaults_compile() {
    let options = ChildRegionOptions::default();
    assert!(options.scroll.show_scrollbar_y);
    assert!(!options.scroll.show_scrollbar_x);
    assert!(options.test_id.is_none());
    assert!(options.content_test_id.is_none());
}
