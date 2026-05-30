use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::action::{PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::{
    ImUiActiveItemState, ImUiLifecycleSessionState, LongPressSignalState,
};

pub(super) fn install_pointer_up<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: fret_runtime::Model<ImUiActiveItemState>,
    context_anchor_model: fret_runtime::Model<Option<Point>>,
    long_press_signal_model: fret_runtime::Model<LongPressSignalState>,
    lifecycle_model: fret_runtime::Model<ImUiLifecycleSessionState>,
    pointer_click_modifiers_model: Option<fret_runtime::Model<Modifiers>>,
) {
    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        crate::imui::mark_lifecycle_deactivated_on_left_pointer_up(
            host,
            acx,
            up.button,
            &lifecycle_model,
        );
        crate::imui::finish_pressable_drag_on_pointer_up(
            host,
            acx,
            up,
            &active_item_model,
            &long_press_signal_model,
            crate::imui::drag_kind_for_element(acx.target),
        );

        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model, |v| *v = Some(up.position));
            host.record_transient_event(acx, crate::imui::KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            return PressablePointerUpResult::SkipActivate;
        }

        if up.is_click
            && up.button == MouseButton::Left
            && let Some(pointer_click_modifiers_model) = pointer_click_modifiers_model.as_ref()
        {
            let _ = host.update_model(pointer_click_modifiers_model, |value| {
                *value = up.modifiers;
            });
            host.record_transient_event(acx, crate::imui::KEY_POINTER_CLICKED);
        }

        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
            host.record_transient_event(acx, crate::imui::KEY_DOUBLE_CLICKED);
            host.notify(acx);
        }

        PressablePointerUpResult::Continue
    }));
}
