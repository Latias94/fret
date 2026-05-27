use std::sync::Arc;

use fret_core::{KeyCode, MouseButton};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::super::super::{
    KEY_CLICKED, KEY_CONTEXT_MENU_REQUESTED, KEY_DOUBLE_CLICKED, KEY_SECONDARY_CLICKED,
    ResponseExt, active_item_model_for_window, context_menu_anchor_model_for,
    hover_blocked_by_active_item_for, install_hover_query_hooks_for_pressable,
    sanitize_response_for_enabled,
};
use super::super::spec::DisclosureSpec;

pub(super) fn install_disclosure_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: &PressableState,
    trigger_id: GlobalElementId,
    spec: &DisclosureSpec,
    open_model: Model<bool>,
    enabled: bool,
    trigger_response: &mut ResponseExt,
) {
    let context_anchor_model = context_menu_anchor_model_for(cx, trigger_id);
    let context_anchor_model_for_report = context_anchor_model.clone();
    cx.pressable_clear_on_pointer_down();
    cx.pressable_clear_on_pointer_move();
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(trigger_id);

    let open_model_for_activate = open_model.clone();
    let has_children = spec.has_children();
    let activate_shortcut = spec.activate_shortcut;
    let shortcut_repeat = spec.shortcut_repeat;
    cx.pressable_on_activate(crate::on_activate(move |host, action_cx, _reason| {
        host.record_transient_event(action_cx, KEY_CLICKED);
        if has_children {
            let _ = host
                .models_mut()
                .update(&open_model_for_activate, |value| *value = !*value);
        }
        host.notify(action_cx);
    }));

    if enabled {
        let open_model_for_key = open_model.clone();
        cx.key_on_key_down_for(
            trigger_id,
            Arc::new(move |host, acx, down| {
                if let Some(shortcut) = activate_shortcut {
                    let matches_shortcut =
                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                    if matches_shortcut && (!down.repeat || shortcut_repeat) && !down.ime_composing
                    {
                        host.record_transient_event(acx, KEY_CLICKED);
                        if has_children {
                            let _ = host
                                .models_mut()
                                .update(&open_model_for_key, |value| *value = !*value);
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

                host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
                host.notify(acx);
                true
            }),
        );
    }

    cx.pressable_on_pointer_down(Arc::new(|_host, _acx, _down| {
        PressablePointerDownResult::Continue
    }));
    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model, |value| *value = Some(up.position));
            host.record_transient_event(acx, KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            return PressablePointerUpResult::SkipActivate;
        }

        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
            host.record_transient_event(acx, KEY_DOUBLE_CLICKED);
            host.notify(acx);
        }

        PressablePointerUpResult::Continue
    }));

    let active_item_model = active_item_model_for_window(cx);
    trigger_response.set_core_hovered(state.hovered);
    trigger_response.set_core_pressed(state.pressed);
    trigger_response.set_core_focused(state.focused);
    trigger_response.set_nav_highlighted(
        state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
    );
    trigger_response.set_id(Some(trigger_id));
    trigger_response.set_core_clicked(cx.take_transient_for(trigger_id, KEY_CLICKED));
    trigger_response
        .set_secondary_clicked(cx.take_transient_for(trigger_id, KEY_SECONDARY_CLICKED));
    trigger_response.set_double_clicked(cx.take_transient_for(trigger_id, KEY_DOUBLE_CLICKED));
    trigger_response
        .set_context_menu_requested(cx.take_transient_for(trigger_id, KEY_CONTEXT_MENU_REQUESTED));
    trigger_response.set_context_menu_anchor(
        cx.read_model(
            &context_anchor_model_for_report,
            Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(None),
    );
    trigger_response.set_core_rect(cx.last_bounds_for_element(trigger_id));
    let hover_delay =
        install_hover_query_hooks_for_pressable(cx, trigger_id, state.hovered_raw, None);
    trigger_response.set_pointer_hovered_raw(state.hovered_raw);
    trigger_response.set_pointer_hovered_raw_below_barrier(state.hovered_raw_below_barrier);
    trigger_response.set_hover_stationary_met(hover_delay.stationary_met);
    trigger_response.set_hover_delay_short_met(hover_delay.delay_short_met);
    trigger_response.set_hover_delay_normal_met(hover_delay.delay_normal_met);
    trigger_response.set_hover_delay_short_shared_met(hover_delay.shared_delay_short_met);
    trigger_response.set_hover_delay_normal_shared_met(hover_delay.shared_delay_normal_met);
    trigger_response.set_hover_blocked_by_active_item(hover_blocked_by_active_item_for(
        cx,
        trigger_id,
        &active_item_model,
    ));
    sanitize_response_for_enabled(enabled, trigger_response);
}
