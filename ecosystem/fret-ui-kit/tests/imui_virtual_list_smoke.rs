#![cfg(feature = "imui")]

use fret_core::Px;
use fret_ui::UiHost;
use fret_ui_kit::imui::{
    UiWriterImUiFacadeExt, VirtualListKeyCacheMode, VirtualListMeasureMode, VirtualListOptions,
};

#[allow(dead_code)]
fn virtual_list_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let _ = ui.virtual_list(
        "virtual.basic",
        128,
        |index| index as fret_ui::ItemKey,
        |ui, index| {
            ui.text(format!("Row {index}"));
        },
    );

    let _ = ui.virtual_list_with_options(
        "virtual.with_options",
        512,
        VirtualListOptions {
            viewport_height: Px(180.0),
            estimate_row_height: Px(24.0),
            overscan: 2,
            measure_mode: VirtualListMeasureMode::Fixed,
            key_cache: VirtualListKeyCacheMode::VisibleOnly,
            test_id: Some("virtual.list".into()),
            ..Default::default()
        },
        |index| (index as u64).wrapping_mul(17),
        |ui, index| {
            let _ = ui.selectable(format!("Item {index}"), index == 3);
        },
    );
}

#[test]
fn virtual_list_option_defaults_compile() {
    let options = VirtualListOptions::default();
    assert_eq!(options.viewport_height, Px(240.0));
    assert_eq!(options.estimate_row_height, Px(28.0));
    assert_eq!(options.overscan, 6);
    assert_eq!(options.measure_mode, VirtualListMeasureMode::Measured);
    assert_eq!(options.key_cache, VirtualListKeyCacheMode::AllKeys);
    assert_eq!(options.keep_alive, 0);
    assert_eq!(options.gap, Px(0.0));
    assert!(options.handle.is_none());
    assert!(options.test_id.is_none());
}
