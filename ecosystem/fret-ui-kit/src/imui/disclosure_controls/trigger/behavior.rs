use std::sync::Arc;

use fret_core::{KeyCode, MouseButton};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{
    KEY_CLICKED, KEY_CONTEXT_MENU_REQUESTED, KEY_DOUBLE_CLICKED, KEY_SECONDARY_CLICKED,
    ResponseExt, context_menu_anchor_model_for,
};
use super::super::spec::DisclosureSpec;

mod response;

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

    response::populate_disclosure_trigger_response(
        cx,
        trigger_id,
        state,
        context_anchor_model_for_report,
        enabled,
        trigger_response,
    );
}
