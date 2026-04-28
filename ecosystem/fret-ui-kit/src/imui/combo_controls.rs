//! Immediate-mode combo helpers.

use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_core::{KeyCode, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::ActivateReason;
use fret_ui::element::{Length, MainAlign, PressableA11y, PressableProps};

use super::label_identity::parse_label_identity;
use super::{ComboOptions, ComboResponse, ResponseExt, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(super) fn combo_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
) -> ComboResponse {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let popup_open = ui.popup_open_model(id);
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let trigger_a11y_label = combo_trigger_a11y_label(label.as_ref(), preview.as_ref());
    let focusable = options.focusable;
    let a11y_label = options.a11y_label.clone();
    let test_id = options.test_id.clone();
    let activate_shortcut = options.activate_shortcut;
    let shortcut_repeat = options.shortcut_repeat;
    let popup_options = options.popup.clone();

    let mut trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        let mut response = ResponseExt::default();

        let element = ui.with_cx_mut(|cx| {
            let response = &mut response;
            let mut props = PressableProps::default();
            props.enabled = enabled;
            props.focusable = enabled && focusable;
            props.layout.size.width = Length::Fill;
            props.layout.size.min_height =
                Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT));
            props.a11y = PressableA11y {
                role: Some(SemanticsRole::ComboBox),
                label: a11y_label
                    .clone()
                    .or_else(|| Some(trigger_a11y_label.clone())),
                test_id: test_id.clone(),
                expanded: Some(open_before),
                ..Default::default()
            };

            let label_for_visuals = label.clone();
            let preview_for_visuals = preview.clone();
            control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
                let behavior = super::item_behavior::install_pressable_item_behavior(cx, id);
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
                let state_badge = if open_before {
                    super::control_chrome::pill(
                        cx,
                        Arc::from("Open"),
                        palette.accent_background,
                        palette.accent_foreground,
                    )
                } else {
                    super::control_chrome::pill(
                        cx,
                        Arc::from("Menu"),
                        palette.subtle_background,
                        palette.muted_foreground,
                    )
                };

                (props, chrome, move |cx| {
                    vec![
                        cx.flex(super::control_chrome::fill_stack_props(), move |cx| {
                            let mut out = Vec::new();
                            if !label_for_visuals.is_empty() {
                                out.push(super::control_chrome::caption_text(
                                    cx,
                                    label_for_visuals.clone(),
                                    palette,
                                ));
                            }
                            out.push(cx.flex(
                                super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                                move |cx| {
                                    vec![
                                        super::control_chrome::fill_text(
                                            cx,
                                            preview_for_visuals.clone(),
                                            palette.foreground,
                                        ),
                                        state_badge,
                                    ]
                                },
                            ));
                            out
                        }),
                    ]
                })
            })
        });

        ui.add(element);
        response
    });

    if enabled && trigger.clicked() {
        if open_before {
            ui.close_popup(id);
        } else if let Some(anchor) = trigger.core.rect {
            ui.open_popup_at(id, anchor);
        }
    }

    let popup_opened = super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id,
        popup_options,
        false,
        f,
    );
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    let open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let toggled = trigger.id.is_some_and(|element_id| {
        ui.with_cx_mut(|cx| super::model_value_changed_for(cx, element_id, open_after))
    });
    trigger.activated = toggled && open_after;
    trigger.deactivated = toggled && !open_after;
    trigger.deactivated_after_edit = false;

    ComboResponse {
        trigger,
        open: open_after,
        toggled,
    }
}

fn combo_trigger_a11y_label(label: &str, preview: &str) -> Arc<str> {
    if label.is_empty() {
        Arc::from(preview)
    } else {
        Arc::from(format!("{label}: {preview}"))
    }
}

#[cfg(test)]
mod tests {
    use super::combo_trigger_a11y_label;

    #[test]
    fn combo_trigger_a11y_label_formats_label_and_preview_inline() {
        assert_eq!(&*combo_trigger_a11y_label("Theme", "Dark"), "Theme: Dark");
    }

    #[test]
    fn combo_trigger_a11y_label_uses_preview_only_when_label_is_empty() {
        assert_eq!(&*combo_trigger_a11y_label("", "Dark"), "Dark");
    }
}
