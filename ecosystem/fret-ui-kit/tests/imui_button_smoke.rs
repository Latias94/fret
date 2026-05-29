#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_core::Size;
use fret_runtime::{ActionId, CommandId};
use fret_ui::UiHost;
use fret_ui_kit::imui::{
    ButtonArrowDirection, ButtonOptions, ButtonVariant, RadioOptions, UiWriterImUiFacadeExt,
};

#[allow(dead_code)]
fn button_family_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let _ = ui.button("button.default");
    let _ = ui.small_button("button.small");
    let _ = ui.small_button_with_options(
        "button.small.options",
        ButtonOptions {
            test_id: Some(Arc::from("imui-button-smoke.small")),
            ..Default::default()
        },
    );
    let _ = ui.arrow_button("button.arrow.left", ButtonArrowDirection::Left);
    let _ = ui.arrow_button_with_options(
        "button.arrow.right",
        ButtonArrowDirection::Right,
        ButtonOptions {
            test_id: Some(Arc::from("imui-button-smoke.arrow")),
            ..Default::default()
        },
    );
    let _ = ui.invisible_button_with_options(
        "button.hotspot",
        Size::new(24.0.into(), 18.0.into()),
        ButtonOptions {
            a11y_label: Some(Arc::from("Hotspot region")),
            test_id: Some(Arc::from("imui-button-smoke.invisible")),
            ..Default::default()
        },
    );
    let _ = ui.radio("radio.default", false);
    let _ = ui.radio_with_options(
        "radio.with_options",
        true,
        RadioOptions {
            test_id: Some(Arc::from("imui-button-smoke.radio")),
            ..Default::default()
        },
    );
    let _ = ui.action_button("button.action", ActionId::from("imui.button.action"));
    let _ = ui.action_button_with_options(
        "button.action.options",
        ActionId::from("imui.button.action.options"),
        ButtonOptions {
            test_id: Some(Arc::from("imui-button-smoke.action")),
            ..Default::default()
        },
    );
    let _ = ui.action_payload_button(
        "button.payload",
        ActionId::from("imui.button.payload"),
        Arc::<str>::from("payload"),
    );
    let _ = ui.action_payload_button_with_options(
        "button.payload.options",
        ActionId::from("imui.button.payload.options"),
        Arc::<str>::from("payload-options"),
        ButtonOptions {
            test_id: Some(Arc::from("imui-button-smoke.payload")),
            ..Default::default()
        },
    );
    let _ = ui.button_command(CommandId::from("imui.button.command"));
    let _ = ui.button_command_with_options(
        CommandId::from("imui.button.command.options"),
        ButtonOptions {
            test_id: Some(Arc::from("imui-button-smoke.command")),
            ..Default::default()
        },
    );
}

#[test]
fn button_option_defaults_compile() {
    let options = ButtonOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert!(matches!(options.variant, ButtonVariant::Default));
}
