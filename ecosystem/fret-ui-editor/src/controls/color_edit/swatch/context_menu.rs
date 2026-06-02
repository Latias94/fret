use std::sync::Arc;

use fret_core::{KeyCode, MouseButton};
use fret_runtime::Model;
use fret_ui::action::PressablePointerDownResult;
use fret_ui::{ElementContext, UiHost};

pub(super) fn install_context_menu_pointer_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    open: Model<bool>,
    tooltip_open: Model<bool>,
    copy_menu_open: Model<bool>,
) {
    if !enabled {
        return;
    }

    cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
        let is_context_menu = down.button == MouseButton::Right
            || (cfg!(target_os = "macos")
                && down.button == MouseButton::Left
                && down.modifiers.ctrl);
        if !is_context_menu {
            return PressablePointerDownResult::Continue;
        }

        let _ = host.models_mut().update(&open, |value| *value = false);
        let _ = host
            .models_mut()
            .update(&tooltip_open, |value| *value = false);
        let _ = host
            .models_mut()
            .update(&copy_menu_open, |value| *value = true);
        host.request_focus(action_cx.target);
        host.request_redraw(action_cx.window);
        PressablePointerDownResult::SkipDefaultAndStopPropagation
    }));
}

pub(super) fn install_context_menu_keyboard_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    enabled: bool,
    open: Model<bool>,
    tooltip_open: Model<bool>,
    copy_menu_open: Model<bool>,
) {
    if !enabled {
        return;
    }

    cx.key_on_key_down_for(
        swatch_id,
        Arc::new(move |host, action_cx, down| {
            if down.repeat {
                return false;
            }

            let no_extra_modifiers = !down.modifiers.ctrl
                && !down.modifiers.alt
                && !down.modifiers.meta
                && !down.modifiers.alt_gr;
            let is_shift_f10 =
                down.key == KeyCode::F10 && down.modifiers.shift && no_extra_modifiers;
            let is_context_menu_key =
                down.key == KeyCode::ContextMenu && !down.modifiers.shift && no_extra_modifiers;
            if !is_shift_f10 && !is_context_menu_key {
                return false;
            }

            let _ = host.models_mut().update(&open, |value| *value = false);
            let _ = host
                .models_mut()
                .update(&tooltip_open, |value| *value = false);
            let _ = host
                .models_mut()
                .update(&copy_menu_open, |value| *value = true);
            host.request_redraw(action_cx.window);
            true
        }),
    );
}
