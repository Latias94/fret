//! Immediate-mode boolean model controls.

use std::sync::Arc;

use fret_core::{KeyCode, MouseButton, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{PressableA11y, PressableProps};

use super::{ResponseExt, SwitchOptions, ToggleOptions, UiWriterImUiFacadeExt};

pub(super) fn checkbox_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = !super::imui_is_disabled(cx);
        let value = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Checkbox),
            label: Some(label.clone()),
            checked: Some(value),
            ..Default::default()
        };

        let label_for_visuals = label.clone();
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

            let model_for_activate = model.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                host.record_transient_event(acx, super::KEY_CHANGED);
                host.notify(acx);
            }));

            if enabled {
                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
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
            response.core.changed = cx.take_transient_for(id, super::KEY_CHANGED);
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

            let prefix: Arc<str> = if value {
                Arc::from("[x] ")
            } else {
                Arc::from("[ ] ")
            };
            vec![cx.text(Arc::from(format!("{prefix}{label_for_visuals}")))]
        })
    });

    ui.add(element);
    response
}

pub(super) fn toggle_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: ToggleOptions,
) -> ResponseExt {
    switch_model_with_options(ui, label, model, options)
}

pub(super) fn switch_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.a11y = crate::primitives::switch::switch_a11y(
            options.a11y_label.clone().or_else(|| Some(label.clone())),
            value,
        );
        props.a11y.test_id = options.test_id.clone();

        let label_for_visuals = label.clone();
        cx.pressable_with_id(props, move |cx, state, id| {
            cx.pressable_clear_on_pointer_down();
            cx.pressable_clear_on_pointer_move();
            cx.pressable_clear_on_pointer_up();

            let active_item_model = super::active_item_model_for_window(cx);
            let active_item_model_for_down = active_item_model.clone();
            let active_item_model_for_up = active_item_model.clone();

            cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                super::mark_active_item_on_left_pointer_down(
                    host,
                    acx,
                    down.button,
                    &active_item_model_for_down,
                    false,
                );
                PressablePointerDownResult::Continue
            }));

            cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                super::clear_active_item_on_left_pointer_up(
                    host,
                    acx,
                    up.button,
                    &active_item_model_for_up,
                );
                PressablePointerUpResult::Continue
            }));

            let model_for_activate = model.clone();
            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
                host.record_transient_event(acx, super::KEY_CLICKED);
                host.record_transient_event(acx, super::KEY_CHANGED);
                host.notify(acx);
            }));

            response.core.hovered = state.hovered;
            response.core.pressed = state.pressed;
            response.core.focused = state.focused;
            response.nav_highlighted =
                state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            response.id = Some(id);
            response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            response.core.changed = cx.take_transient_for(id, super::KEY_CHANGED);
            response.core.rect = cx.last_bounds_for_element(id);
            let hover_delay =
                super::install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
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

            let prefix: Arc<str> = if value {
                Arc::from("[on] ")
            } else {
                Arc::from("[off] ")
            };
            vec![cx.text(Arc::from(format!("{prefix}{label_for_visuals}")))]
        })
    });

    ui.add(element);
    response
}
