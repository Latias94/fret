//! Immediate-mode button-style pressable helpers.

use std::sync::Arc;

use fret_core::{KeyCode, MouseButton, SemanticsRole};
use fret_runtime::ActionId;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{ActivateReason, PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{PressableA11y, PressableProps};

use super::{ButtonOptions, ResponseExt, UiWriterImUiFacadeExt};
use crate::command::ElementCommandGatingExt as _;

pub(super) fn button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
) -> ResponseExt {
    button_impl(ui, label, options, None)
}

pub(super) fn action_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: ButtonOptions,
) -> ResponseExt {
    button_impl(ui, label, options, Some(action))
}

fn button_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
    action: Option<ActionId>,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let mut enabled = options.enabled && !super::imui_is_disabled(cx);
        if let Some(action) = action.as_ref() {
            enabled = enabled && cx.action_is_enabled(action);
        }
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Button),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            test_id: options.test_id.clone(),
            ..Default::default()
        };
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        cx.pressable_with_id(props, move |cx, state, id| {
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

            let action_for_activate = action.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                host.record_transient_event(acx, super::KEY_CLICKED);
                if let Some(action) = action_for_activate.clone() {
                    host.record_pending_command_dispatch_source(acx, &action, reason);
                    host.dispatch_command(Some(acx.window), action);
                }
                host.notify(acx);
            }));

            if enabled {
                let action_for_shortcut = action.clone();
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
                                host.record_transient_event(acx, super::KEY_CLICKED);
                                if let Some(action) = action_for_shortcut.clone() {
                                    host.record_pending_command_dispatch_source(
                                        acx,
                                        &action,
                                        ActivateReason::Keyboard,
                                    );
                                    host.dispatch_command(Some(acx.window), action);
                                }
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
                super::finish_pressable_drag_on_pointer_up(
                    host,
                    acx,
                    up,
                    &active_item_model_for_up,
                    &long_press_signal_model_for_up,
                    super::drag_kind_for_element(acx.target),
                );

                if up.is_click && up.button == MouseButton::Right {
                    let _ = host.update_model(&context_anchor_model, |v| *v = Some(up.position));
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
            response.nav_highlighted =
                state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            response.id = Some(id);
            response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            response.secondary_clicked = cx.take_transient_for(id, super::KEY_SECONDARY_CLICKED);
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
            super::sanitize_response_for_enabled(enabled, response);

            vec![cx.text(label.clone())]
        })
    });

    ui.add(element);
    response
}
