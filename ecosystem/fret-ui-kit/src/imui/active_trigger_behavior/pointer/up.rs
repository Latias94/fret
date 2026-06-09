use std::sync::Arc;

use fret_core::{MouseButton, Point};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

pub(super) fn install_active_trigger_pointer_up<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
    context_anchor_model: Model<Option<Point>>,
    primary_active: bool,
) {
    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if primary_active {
            crate::imui::mark_lifecycle_deactivated_on_left_pointer_up(
                host,
                acx,
                up.button,
                &lifecycle_model,
            );
            crate::imui::clear_active_item_on_left_pointer_up(
                host,
                acx,
                up.button,
                &active_item_model,
            );
        }

        if up.is_click && up.button == MouseButton::Right {
            let _ = host.update_model(&context_anchor_model, |value| *value = Some(up.position));
            host.record_transient_event(acx, crate::imui::KEY_SECONDARY_CLICKED);
            host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
            host.notify(acx);
            return PressablePointerUpResult::SkipActivate;
        }

        PressablePointerUpResult::Continue
    }));
}
