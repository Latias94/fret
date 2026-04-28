//! Immediate-mode boolean model controls.

use std::sync::Arc;

use fret_core::{Corners, Edges, KeyCode, Px, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, PressableA11y,
    PressableProps,
};

use super::label_identity::parse_label_identity;
use super::{CheckboxOptions, RadioOptions, ResponseExt, SwitchOptions, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

fn radio_indicator<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    palette: super::control_chrome::ImUiControlPalette,
    selected: bool,
) -> AnyElement {
    let mut outer = ContainerProps::default();
    outer.layout.size.width = Length::Px(super::control_chrome::RADIO_INDICATOR_SIZE);
    outer.layout.size.height = Length::Px(super::control_chrome::RADIO_INDICATOR_SIZE);
    outer.border = Edges::all(Px(1.0));
    outer.border_color = Some(if selected {
        palette.accent_background
    } else {
        palette.border
    });
    outer.corner_radii = Corners::all(Px(999.0));

    cx.container(outer, move |cx| {
        if !selected {
            return Vec::new();
        }

        let mut center = FlexProps::default();
        center.layout.size.width = Length::Fill;
        center.layout.size.height = Length::Fill;
        center.justify = MainAlign::Center;
        center.align = CrossAlign::Center;

        let mut dot = ContainerProps::default();
        dot.layout.size.width = Length::Px(super::control_chrome::RADIO_DOT_SIZE);
        dot.layout.size.height = Length::Px(super::control_chrome::RADIO_DOT_SIZE);
        dot.background = Some(palette.accent_background);
        dot.corner_radii = Corners::all(Px(999.0));

        vec![cx.flex(center, move |cx| vec![cx.container(dot, |_| Vec::new())])]
    })
}

pub(super) fn checkbox_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
) -> ResponseExt {
    checkbox_model_with_options(ui, label, model, CheckboxOptions::default())
}

pub(super) fn checkbox_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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
        let enabled = options.enabled && !super::imui_is_disabled(cx);
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
        props.layout.size.min_height = Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Checkbox),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            checked: Some(value),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::item_behavior::install_pressable_item_behavior(cx, id);
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            let model_for_activate = model.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
                host.record_transient_event(acx, super::KEY_CHANGED);
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
                                super::mark_lifecycle_edit(
                                    host,
                                    acx,
                                    &lifecycle_model_for_shortcut,
                                );
                                host.record_transient_event(acx, super::KEY_CHANGED);
                                host.notify(acx);
                                return true;
                            }
                        }

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

            let changed = cx.take_transient_for(id, super::KEY_CHANGED);
            super::item_behavior::populate_pressable_item_response(
                cx,
                id,
                state,
                &behavior,
                super::item_behavior::PressableItemResponseInput {
                    enabled,
                    clicked: false,
                    changed,
                    lifecycle_edited: changed,
                },
                response,
            );

            let (palette, chrome) = super::control_chrome::field_chrome(cx, enabled, state);
            let indicator = super::control_chrome::pill(
                cx,
                Arc::from(if value { "[x]" } else { "[ ]" }),
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
                    super::control_chrome::fill_row_props(MainAlign::Start),
                    move |cx| {
                        vec![
                            indicator,
                            super::control_chrome::fill_text(
                                cx,
                                label_for_visuals.clone(),
                                palette.foreground,
                            ),
                        ]
                    },
                )]
            })
        })
    });

    ui.add(element);
    response
}

pub(super) fn radio_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    selected: bool,
    options: RadioOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("radio-label", identity), |ui| {
        radio_with_options_inner(ui, visible_label, selected, options)
    })
}

fn radio_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    selected: bool,
    options: RadioOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height = Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::RadioButton),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            checked: Some(selected),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::item_behavior::install_pressable_item_behavior(cx, id);
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                super::mark_lifecycle_instant_if_inactive(
                    host,
                    acx,
                    &lifecycle_model_for_activate,
                    false,
                );
                host.record_transient_event(acx, super::KEY_CLICKED);
                host.notify(acx);
            }));

            if enabled {
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
                                super::mark_lifecycle_instant_if_inactive(
                                    host,
                                    acx,
                                    &lifecycle_model_for_shortcut,
                                    false,
                                );
                                host.record_transient_event(acx, super::KEY_CLICKED);
                                host.notify(acx);
                                return true;
                            }
                        }

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

            let (palette, chrome) = super::control_chrome::field_chrome(cx, enabled, state);
            let indicator = radio_indicator(cx, palette, selected);

            (props, chrome, move |cx| {
                vec![cx.flex(
                    super::control_chrome::fill_row_props(MainAlign::Start),
                    move |cx| {
                        vec![
                            indicator,
                            super::control_chrome::fill_text(
                                cx,
                                label_for_visuals.clone(),
                                palette.foreground,
                            ),
                        ]
                    },
                )]
            })
        })
    });

    ui.add(element);
    response
}

pub(super) fn switch_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let value = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height = Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = crate::primitives::switch::switch_a11y(
            options.a11y_label.clone().or_else(|| Some(label.clone())),
            value,
        );
        props.a11y.test_id = options.test_id.clone();

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::active_trigger_behavior::install_active_trigger_behavior(
                cx,
                id,
                super::active_trigger_behavior::ActiveTriggerBehaviorOptions {
                    request_focus_on_press: false,
                    clear_pointer_move: true,
                },
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();
            let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();

            let model_for_activate = model.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
                host.record_transient_event(acx, super::KEY_CLICKED);
                host.record_transient_event(acx, super::KEY_CHANGED);
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
                                super::mark_lifecycle_edit(
                                    host,
                                    acx,
                                    &lifecycle_model_for_shortcut,
                                );
                                host.record_transient_event(acx, super::KEY_CLICKED);
                                host.record_transient_event(acx, super::KEY_CHANGED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        false
                    }),
                );
            }

            let clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            let changed = cx.take_transient_for(id, super::KEY_CHANGED);
            super::active_trigger_behavior::populate_active_trigger_response(
                cx,
                id,
                state,
                &behavior,
                super::active_trigger_behavior::ActiveTriggerResponseInput {
                    enabled,
                    clicked,
                    changed,
                    lifecycle_edited: changed,
                },
                response,
            );

            let (palette, chrome) = super::control_chrome::field_chrome(cx, enabled, state);
            let state_badge = super::control_chrome::pill(
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
                    super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                    move |cx| {
                        vec![
                            super::control_chrome::fill_text(
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
