#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::imui::{
    ChildRegionOptions, ChildRegionResizeXOptions, ChildRegionResizeYOptions, ChildRegionResponse,
    UiWriterImUiFacadeExt,
};

#[allow(dead_code)]
fn child_region_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let _: ChildRegionResponse = ui.child_region("child-region", |_ui| {});
    let _: ChildRegionResponse = ui.child_region_with_options(
        "child-region.with-options",
        ChildRegionOptions {
            layout: LayoutRefinement::default().h_px(fret_core::Px(96.0)),
            ..Default::default()
        },
        |_ui| {},
    );
    let _: ChildRegionResponse = ui.child_region_with_options(
        "child-region.resize-x",
        ChildRegionOptions {
            layout: LayoutRefinement::default().w_px(fret_core::Px(160.0)),
            resize_x: Some(
                ChildRegionResizeXOptions::new()
                    .min_width(fret_core::Px(80.0))
                    .max_width(fret_core::Px(360.0))
                    .handle_test_id("child-region.resize-x.handle"),
            ),
            ..Default::default()
        },
        |_ui| {},
    );
    let _: ChildRegionResponse = ui.child_region_with_options(
        "child-region.resize-y",
        ChildRegionOptions {
            layout: LayoutRefinement::default().h_px(fret_core::Px(96.0)),
            resize_y: Some(
                ChildRegionResizeYOptions::new()
                    .min_height(fret_core::Px(48.0))
                    .max_height(fret_core::Px(240.0))
                    .handle_test_id("child-region.resize-y.handle"),
            ),
            ..Default::default()
        },
        |_ui| {},
    );
}

#[test]
fn child_region_option_defaults_compile() {
    let options = ChildRegionOptions::default();
    assert!(options.scroll.show_scrollbar_y);
    assert!(!options.scroll.show_scrollbar_x);
    assert!(options.layout.size.is_none());
    assert!(options.resize_x.is_none());
    assert!(options.resize_y.is_none());
    assert!(options.test_id.is_none());
    assert!(options.content_test_id.is_none());
}

#[test]
fn child_region_resize_x_option_defaults_compile() {
    let options = ChildRegionResizeXOptions::default();
    assert_eq!(options.min_width, Some(fret_core::Px(32.0)));
    assert_eq!(options.max_width, None);
    assert!(options.handle_test_id.is_none());

    let options = ChildRegionResizeXOptions::new()
        .min_width(fret_core::Px(80.0))
        .max_width(fret_core::Px(360.0))
        .handle_test_id("imui-child.resize-x");
    assert_eq!(options.min_width, Some(fret_core::Px(80.0)));
    assert_eq!(options.max_width, Some(fret_core::Px(360.0)));
    assert_eq!(
        options.handle_test_id.as_deref(),
        Some("imui-child.resize-x")
    );
}

#[test]
fn child_region_resize_y_option_defaults_compile() {
    let options = ChildRegionResizeYOptions::default();
    assert_eq!(options.min_height, Some(fret_core::Px(32.0)));
    assert_eq!(options.max_height, None);
    assert!(options.handle_test_id.is_none());

    let options = ChildRegionResizeYOptions::new()
        .min_height(fret_core::Px(48.0))
        .max_height(fret_core::Px(240.0))
        .handle_test_id("imui-child.resize-y");
    assert_eq!(options.min_height, Some(fret_core::Px(48.0)));
    assert_eq!(options.max_height, Some(fret_core::Px(240.0)));
    assert_eq!(
        options.handle_test_id.as_deref(),
        Some("imui-child.resize-y")
    );
}
