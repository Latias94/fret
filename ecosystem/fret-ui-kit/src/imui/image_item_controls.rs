//! Immediate-mode response-bearing image item helpers.

use fret_core::{ImageId, KeyCode, SemanticsRole, Size};
use fret_ui::UiHost;
use fret_ui::action::ActivateReason;
use fret_ui::element::{Length, PressableA11y, PressableKeyActivation, PressableProps};

use super::{ImageItemOptions, ImageItemVariant, ResponseExt, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod visual;

use visual::{image_item_chrome, image_props_for_item, sanitize_item_size};

pub(super) fn image_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    image: ImageId,
    size: Size,
    options: ImageItemOptions,
) -> ResponseExt {
    ui.push_id(("image-item", id), |ui| {
        image_item_with_options_inner(ui, image, size, options)
    })
}

fn image_item_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    image: ImageId,
    size: Size,
    options: ImageItemOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let variant = options.variant;
        let item_size = sanitize_item_size(size);

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = focusable;
        props.layout.size.width = Length::Px(item_size.width);
        props.layout.size.height = Length::Px(item_size.height);
        if matches!(variant, ImageItemVariant::Image) {
            props.key_activation = PressableKeyActivation::None;
        }
        props.a11y = PressableA11y {
            role: Some(match variant {
                ImageItemVariant::Image => SemanticsRole::Image,
                ImageItemVariant::Button => SemanticsRole::Button,
            }),
            label: options.a11y_label.clone(),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::item_behavior::install_pressable_item_behavior_with_options(
                cx,
                id,
                super::item_behavior::PressableItemBehaviorOptions {
                    report_pointer_click: true,
                },
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                if reason == ActivateReason::Keyboard {
                    super::mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_activate,
                        false,
                    );
                }
                host.record_transient_event(acx, super::KEY_CLICKED);
                host.notify(acx);
            }));

            if enabled {
                cx.key_on_key_down_for(
                    id,
                    std::sync::Arc::new(move |host, acx, down| {
                        let is_menu_key = down.key == KeyCode::ContextMenu;
                        let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                        if !(is_menu_key || is_shift_f10) {
                            return false;
                        }

                        host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        true
                    }),
                );
            }

            let clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            super::item_behavior::populate_pressable_item_response(
                cx,
                id,
                state,
                &behavior,
                super::item_behavior::PressableItemResponseInput {
                    enabled,
                    clicked,
                    changed: false,
                    lifecycle_edited: false,
                },
                response,
            );

            let chrome = image_item_chrome(cx, enabled, state, variant);
            let image_props = image_props_for_item(
                image,
                options.fit,
                options.sampling,
                options.opacity,
                options.uv,
            );

            (props, chrome, move |cx| vec![cx.image_props(image_props)])
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
