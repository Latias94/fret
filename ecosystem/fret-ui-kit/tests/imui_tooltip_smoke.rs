#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{ResponseExt, TooltipOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn tooltip_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let trigger = ResponseExt::default();

    let _ = ui.tooltip_text("tooltip.text", trigger, "Tooltip text");
    let _ = ui.tooltip_text_with_options(
        "tooltip.text.with_options",
        trigger,
        "Tooltip text",
        TooltipOptions::default(),
    );
    let _ = ui.tooltip("tooltip.rich", trigger, |ui| {
        ui.text("Tooltip rich body");
    });
    let _ = ui.tooltip_with_options(
        "tooltip.rich.with_options",
        trigger,
        TooltipOptions::default(),
        |ui| {
            ui.text("Tooltip rich body");
        },
    );
}

#[test]
fn tooltip_option_defaults_compile() {
    let options = TooltipOptions::default();
    assert_eq!(options.window_margin, fret_core::Px(8.0));
    assert_eq!(options.open_delay_frames_override, None);
    assert_eq!(options.close_delay_frames_override, None);
    assert!(options.test_id.is_none());
}
