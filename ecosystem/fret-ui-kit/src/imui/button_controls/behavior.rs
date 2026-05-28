use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::PressableProps;

use super::super::{ButtonOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::visual;
use crate::command::ElementCommandGatingExt as _;
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod action;
mod activation;
mod keyboard;
mod response;

pub(super) use action::ButtonAction;

pub(super) fn button_pressable<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
    action: Option<ButtonAction>,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let mut enabled = options.enabled && !super::super::imui_is_disabled(cx);
        if let Some(action) = action.as_ref() {
            enabled = enabled && cx.action_is_enabled(&action.action);
        }
        let variant = options.variant;
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        visual::apply_button_variant_layout(&mut props, variant);
        props.a11y = visual::button_a11y(&label, &options, variant);
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::super::item_behavior::install_pressable_item_behavior(cx, id);
            activation::install_button_activation_behavior(
                cx,
                activation::ButtonActivationBehaviorInput {
                    lifecycle_model: behavior.lifecycle_model.clone(),
                    action: action.clone(),
                },
            );

            keyboard::install_button_keyboard_behavior(
                cx,
                id,
                keyboard::ButtonKeyboardBehaviorInput {
                    enabled,
                    activate_shortcut,
                    shortcut_repeat,
                    lifecycle_model: behavior.lifecycle_model.clone(),
                    action: action.clone(),
                },
            );

            response::populate_button_pressable_response(
                cx, id, state, &behavior, enabled, response,
            );

            let (chrome, visual_content) =
                visual::resolve_button_visual(cx, enabled, state, variant, label.clone())
                    .into_parts();

            (props, chrome, move |cx| visual_content.children(cx))
        })
    });

    ui.add(element);
    response
}
