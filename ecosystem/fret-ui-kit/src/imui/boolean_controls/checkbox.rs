use std::sync::Arc;

use fret_core::{KeyCode, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{Length, MainAlign, PressableA11y, PressableProps};

use super::super::label_identity::parse_label_identity;
use super::super::{CheckboxOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::visual;
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(in crate::imui) fn checkbox_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
) -> ResponseExt {
    checkbox_model_with_options(ui, label, model, CheckboxOptions::default())
}

pub(in crate::imui) fn checkbox_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: CheckboxOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("checkbox-label", identity), |ui| {
        checkbox_model_with_options_inner(ui, visible_label, model, options)
    })
}

fn checkbox_model_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: CheckboxOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let value = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height =
            Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Checkbox),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            checked: Some(value),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::super::item_behavior::install_pressable_item_behavior(cx, id);
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            let model_for_activate = model.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                super::super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
                host.record_transient_event(acx, super::super::KEY_CHANGED);
                host.notify(acx);
            }));

            if enabled {
                let model_for_shortcut = model.clone();
                let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        if let Some(shortcut) = activate_shortcut {
                            let matches_shortcut =
                                down.key == shortcut.key && down.modifiers == shortcut.mods;
                            if matches_shortcut
                                && (!down.repeat || shortcut_repeat)
                                && !down.ime_composing
                            {
                                let _ =
                                    host.update_model(&model_for_shortcut, |v: &mut bool| *v = !*v);
                                super::super::mark_lifecycle_edit(
                                    host,
                                    acx,
                                    &lifecycle_model_for_shortcut,
                                );
                                host.record_transient_event(acx, super::super::KEY_CHANGED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        let is_menu_key = down.key == KeyCode::ContextMenu;
                        let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                        if !(is_menu_key || is_shift_f10) {
                            return false;
                        }

                        host.record_transient_event(acx, super::super::KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        true
                    }),
                );
            }

            let changed = cx.take_transient_for(id, super::super::KEY_CHANGED);
            super::super::item_behavior::populate_pressable_item_response(
                cx,
                id,
                state,
                &behavior,
                super::super::item_behavior::PressableItemResponseInput {
                    enabled,
                    clicked: false,
                    changed,
                    lifecycle_edited: changed,
                },
                response,
            );

            let (palette, chrome) = super::super::control_chrome::field_chrome(cx, enabled, state);
            let indicator = visual::checkbox_indicator(cx, palette, value);

            (props, chrome, move |cx| {
                vec![cx.flex(
                    super::super::control_chrome::fill_row_props(MainAlign::Start),
                    move |cx| {
                        vec![
                            indicator,
                            visual::boolean_label(cx, label_for_visuals.clone(), palette),
                        ]
                    },
                )]
            })
        })
    });

    ui.add(element);
    response
}
