use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{Length, MainAlign, PressableProps};

use super::super::label_identity::parse_label_identity;
use super::super::{ResponseExt, SwitchOptions, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(in crate::imui) fn switch_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: SwitchOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("switch-label", identity), |ui| {
        switch_model_with_options_inner(ui, visible_label, model, options)
    })
}

fn switch_model_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: SwitchOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::super::imui_is_disabled(cx);
        let value = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height =
            Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = crate::primitives::switch::switch_a11y(
            options.a11y_label.clone().or_else(|| Some(label.clone())),
            value,
        );
        props.a11y.test_id = options.test_id.clone();

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::super::active_trigger_behavior::install_active_trigger_behavior(
                cx,
                id,
                super::super::active_trigger_behavior::ActiveTriggerBehaviorOptions {
                    primary_active: true,
                    request_focus_on_press: false,
                    clear_pointer_move: true,
                },
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();
            let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();

            let model_for_activate = model.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                super::super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
                host.record_transient_event(acx, super::super::KEY_CLICKED);
                host.record_transient_event(acx, super::super::KEY_CHANGED);
                host.notify(acx);
            }));

            if enabled && options.focusable {
                let model_for_shortcut = model.clone();
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
                                host.record_transient_event(acx, super::super::KEY_CLICKED);
                                host.record_transient_event(acx, super::super::KEY_CHANGED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        false
                    }),
                );
            }

            let clicked = cx.take_transient_for(id, super::super::KEY_CLICKED);
            let changed = cx.take_transient_for(id, super::super::KEY_CHANGED);
            super::super::active_trigger_behavior::populate_active_trigger_response(
                cx,
                id,
                state,
                &behavior,
                super::super::active_trigger_behavior::ActiveTriggerResponseInput {
                    enabled,
                    clicked,
                    changed,
                    lifecycle_edited: changed,
                },
                response,
            );

            let (palette, chrome) = super::super::control_chrome::field_chrome(cx, enabled, state);
            let state_badge = super::super::control_chrome::pill(
                cx,
                Arc::from(if value { "On" } else { "Off" }),
                if value {
                    palette.accent_background
                } else {
                    palette.subtle_background
                },
                if value {
                    palette.accent_foreground
                } else {
                    palette.muted_foreground
                },
            );

            (props, chrome, move |cx| {
                vec![cx.flex(
                    super::super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                    move |cx| {
                        vec![
                            super::super::control_chrome::fill_text(
                                cx,
                                label_for_visuals.clone(),
                                palette.foreground,
                            ),
                            state_badge,
                        ]
                    },
                )]
            })
        })
    });

    ui.add(element);
    response
}
