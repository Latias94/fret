//! Immediate-mode selectable row helpers.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui::action::{ActivateReason, UiActionHostExt as _};
use fret_ui::element::{Length, PressableA11y, PressableProps};

use super::label_identity::parse_label_identity;
use super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

mod keyboard;
mod visual;

use keyboard::{SelectableKeyboardOptions, install_selectable_keyboard};
use visual::selectable_row_element;

pub(super) fn selectable_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("selectable-label", identity), |ui| {
        selectable_with_options_inner(ui, visible_label, options)
    })
}

fn selectable_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let selected = options.selected;
        let highlighted = enabled && options.highlighted;
        let close_popup = options.close_popup.clone();
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        props.a11y = PressableA11y {
            role: options.a11y_role.or(Some(SemanticsRole::ListBoxOption)),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            test_id: options.test_id.clone(),
            selected,
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, id| {
            let behavior = super::item_behavior::install_pressable_item_behavior_with_options(
                cx,
                id,
                super::item_behavior::PressableItemBehaviorOptions {
                    report_pointer_click: true,
                },
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            if enabled {
                let close_popup_for_activate = close_popup.clone();
                cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                    if reason == ActivateReason::Keyboard {
                        super::mark_lifecycle_instant_if_inactive(
                            host,
                            acx,
                            &lifecycle_model_for_activate,
                            false,
                        );
                    }
                    if let Some(open) = close_popup_for_activate.as_ref() {
                        let _ = host.update_model(open, |v| *v = false);
                    }
                    host.record_transient_event(acx, super::KEY_CLICKED);
                    host.notify(acx);
                }));

                install_selectable_keyboard(
                    cx,
                    id,
                    focusable,
                    behavior.lifecycle_model.clone(),
                    SelectableKeyboardOptions {
                        close_popup: close_popup.clone(),
                        activate_shortcut,
                        shortcut_repeat,
                    },
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

            vec![selectable_row_element(
                cx,
                label.clone(),
                enabled,
                selected,
                highlighted,
                state,
            )]
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
