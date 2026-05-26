use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_core::{KeyCode, SemanticsRole};
use fret_runtime::KeyChord;
use fret_ui::UiHost;
use fret_ui::action::ActivateReason;
use fret_ui::element::{Length, MainAlign, PressableA11y, PressableProps};

use super::super::{ResponseExt, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(super) struct ComboTriggerOptions {
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) open: bool,
}

pub(super) fn combo_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboTriggerOptions,
) -> ResponseExt {
    ui.push_id(format!("{id}.trigger"), |ui| {
        let mut response = ResponseExt::default();

        let element = ui.with_cx_mut(|cx| {
            let response = &mut response;
            let mut props = PressableProps::default();
            props.enabled = options.enabled;
            props.focusable = options.enabled && options.focusable;
            props.layout.size.width = Length::Fill;
            props.layout.size.min_height =
                Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT));
            props.a11y = PressableA11y {
                role: Some(SemanticsRole::ComboBox),
                label: options
                    .a11y_label
                    .clone()
                    .or_else(|| Some(combo_trigger_a11y_label(label.as_ref(), preview.as_ref()))),
                test_id: options.test_id.clone(),
                expanded: Some(options.open),
                ..Default::default()
            };

            let enabled = options.enabled;
            let open = options.open;
            let activate_shortcut = options.activate_shortcut;
            let shortcut_repeat = options.shortcut_repeat;
            let label_for_visuals = label.clone();
            let preview_for_visuals = preview.clone();
            control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
                let behavior = super::super::item_behavior::install_pressable_item_behavior(cx, id);
                let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

                cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                    if reason == ActivateReason::Keyboard {
                        super::super::mark_lifecycle_instant_if_inactive(
                            host,
                            acx,
                            &lifecycle_model_for_activate,
                            false,
                        );
                    }
                    host.record_transient_event(acx, super::super::KEY_CLICKED);
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
                                    super::super::mark_lifecycle_instant_if_inactive(
                                        host,
                                        acx,
                                        &lifecycle_model_for_shortcut,
                                        false,
                                    );
                                    host.record_transient_event(acx, super::super::KEY_CLICKED);
                                    host.notify(acx);
                                    return true;
                                }
                            }

                            let is_menu_key = down.key == KeyCode::ContextMenu;
                            let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                            if !(is_menu_key || is_shift_f10) {
                                return false;
                            }

                            host.record_transient_event(
                                acx,
                                super::super::KEY_CONTEXT_MENU_REQUESTED,
                            );
                            host.notify(acx);
                            true
                        }),
                    );
                }

                let clicked = cx.take_transient_for(id, super::super::KEY_CLICKED);
                super::super::item_behavior::populate_pressable_item_response(
                    cx,
                    id,
                    state,
                    &behavior,
                    super::super::item_behavior::PressableItemResponseInput {
                        enabled,
                        clicked,
                        changed: false,
                        lifecycle_edited: false,
                    },
                    response,
                );

                let (palette, chrome) =
                    super::super::control_chrome::field_chrome(cx, enabled, state);
                let state_badge = if open {
                    super::super::control_chrome::pill(
                        cx,
                        Arc::from("Open"),
                        palette.accent_background,
                        palette.accent_foreground,
                    )
                } else {
                    super::super::control_chrome::pill(
                        cx,
                        Arc::from("Menu"),
                        palette.subtle_background,
                        palette.muted_foreground,
                    )
                };

                (props, chrome, move |cx| {
                    vec![cx.flex(
                        super::super::control_chrome::fill_stack_props(),
                        move |cx| {
                            let mut out = Vec::new();
                            if !label_for_visuals.is_empty() {
                                out.push(super::super::control_chrome::caption_text(
                                    cx,
                                    label_for_visuals.clone(),
                                    palette,
                                ));
                            }
                            out.push(cx.flex(
                                super::super::control_chrome::fill_row_props(
                                    MainAlign::SpaceBetween,
                                ),
                                move |cx| {
                                    vec![
                                        super::super::control_chrome::fill_text(
                                            cx,
                                            preview_for_visuals.clone(),
                                            palette.foreground,
                                        ),
                                        state_badge,
                                    ]
                                },
                            ));
                            out
                        },
                    )]
                })
            })
        });

        ui.add(element);
        response
    })
}

pub(super) fn combo_trigger_a11y_label(label: &str, preview: &str) -> Arc<str> {
    if label.is_empty() {
        Arc::from(preview)
    } else {
        Arc::from(format!("{label}: {preview}"))
    }
}
