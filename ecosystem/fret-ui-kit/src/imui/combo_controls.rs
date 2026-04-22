//! Immediate-mode combo helpers.

use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_core::{KeyCode, MouseButton, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{ActivateReason, PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{Length, MainAlign, PressableA11y, PressableProps};

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
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();
                cx.key_clear_on_key_down_for(id);

                let active_item_model = super::active_item_model_for_window(cx);
                let active_item_model_for_down = active_item_model.clone();
                let active_item_model_for_move = active_item_model.clone();
                let active_item_model_for_up = active_item_model.clone();

                let context_anchor_model = super::context_menu_anchor_model_for(cx, id);
                let context_anchor_model_for_report = context_anchor_model.clone();
                let long_press_signal_model = super::long_press_signal_model_for(cx, id);
                let long_press_signal_model_for_down = long_press_signal_model.clone();
                let long_press_signal_model_for_move = long_press_signal_model.clone();
                let long_press_signal_model_for_up = long_press_signal_model.clone();
                let lifecycle_model = super::lifecycle_session_model_for(cx, id);
                let lifecycle_model_for_activate = lifecycle_model.clone();
                let lifecycle_model_for_down = lifecycle_model.clone();
                let lifecycle_model_for_up = lifecycle_model.clone();

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
                    let lifecycle_model_for_shortcut = lifecycle_model.clone();
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

                cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                    super::mark_lifecycle_activated_on_left_pointer_down(
                        host,
                        acx,
                        down.button,
                        &lifecycle_model_for_down,
                    );
                    super::prepare_pressable_drag_on_pointer_down(
                        host,
                        acx,
                        down,
                        &active_item_model_for_down,
                        &long_press_signal_model_for_down,
                        super::drag_kind_for_element(acx.target),
                    );

                    PressablePointerDownResult::Continue
                }));

                let drag_threshold = super::drag_threshold_for(cx);
                cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
                    super::handle_pressable_drag_move_with_threshold(
                        host,
                        acx,
                        mv,
                        &active_item_model_for_move,
                        &long_press_signal_model_for_move,
                        super::drag_kind_for_element(acx.target),
                        drag_threshold,
                    )
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                    super::mark_lifecycle_deactivated_on_left_pointer_up(
                        host,
                        acx,
                        up.button,
                        &lifecycle_model_for_up,
                    );
                    super::finish_pressable_drag_on_pointer_up(
                        host,
                        acx,
                        up,
                        &active_item_model_for_up,
                        &long_press_signal_model_for_up,
                        super::drag_kind_for_element(acx.target),
                    );

                    if up.is_click && up.button == MouseButton::Right {
                        let _ =
                            host.update_model(&context_anchor_model, |v| *v = Some(up.position));
                        host.record_transient_event(acx, super::KEY_SECONDARY_CLICKED);
                        host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                        host.notify(acx);
                        return PressablePointerUpResult::SkipActivate;
                    }

                    if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
                        host.record_transient_event(acx, super::KEY_DOUBLE_CLICKED);
                        host.notify(acx);
                    }

                    PressablePointerUpResult::Continue
                }));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.nav_highlighted = state.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                response.id = Some(id);
                response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
                response.secondary_clicked =
                    cx.take_transient_for(id, super::KEY_SECONDARY_CLICKED);
                response.double_clicked = cx.take_transient_for(id, super::KEY_DOUBLE_CLICKED);
                response.long_pressed = cx.take_transient_for(id, super::KEY_LONG_PRESSED);
                response.press_holding = cx
                    .read_model(
                        &long_press_signal_model,
                        fret_ui::Invalidation::Paint,
                        |_app, value| value.holding,
                    )
                    .unwrap_or(false);
                response.context_menu_requested =
                    cx.take_transient_for(id, super::KEY_CONTEXT_MENU_REQUESTED);
                response.context_menu_anchor = cx
                    .read_model(
                        &context_anchor_model_for_report,
                        fret_ui::Invalidation::Paint,
                        |_app, v| *v,
                    )
                    .unwrap_or(None);
                super::populate_pressable_drag_response(cx, id, response);
                response.core.rect = cx.last_bounds_for_element(id);
                let hover_delay = super::install_hover_query_hooks_for_pressable(
                    cx,
                    id,
                    state.hovered_raw,
                    Some(long_press_signal_model.clone()),
                );
                response.pointer_hovered_raw = state.hovered_raw;
                response.pointer_hovered_raw_below_barrier = state.hovered_raw_below_barrier;
                response.hover_stationary_met = hover_delay.stationary_met;
                response.hover_delay_short_met = hover_delay.delay_short_met;
                response.hover_delay_normal_met = hover_delay.delay_normal_met;
                response.hover_delay_short_shared_met = hover_delay.shared_delay_short_met;
                response.hover_delay_normal_shared_met = hover_delay.shared_delay_normal_met;
                response.hover_blocked_by_active_item =
                    super::hover_blocked_by_active_item_for(cx, id, &active_item_model);
                super::populate_response_lifecycle_transients(cx, id, response);
                super::populate_response_lifecycle_from_active_state(
                    cx,
                    id,
                    state.pressed,
                    false,
                    response,
                );
                super::sanitize_response_for_enabled(enabled, response);

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
