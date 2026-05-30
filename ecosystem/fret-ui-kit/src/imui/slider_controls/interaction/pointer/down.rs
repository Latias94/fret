use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::action::PressablePointerDownResult;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, UiHost};

use super::super::SliderInteractionRange;
use super::value_update::apply_slider_pointer_value_update;
use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};
use crate::imui::{KEY_CHANGED, mark_lifecycle_activated_on_left_pointer_down};

pub(super) fn install_slider_pointer_down_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<f32>,
    range: SliderInteractionRange,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }

        mark_lifecycle_activated_on_left_pointer_down(host, acx, down.button, &lifecycle_model);
        let _ = host.update_model(&active_item_model, |st| {
            st.active = Some(acx.target);
        });
        host.capture_pointer();
        host.request_focus(acx.target);

        let changed = apply_slider_pointer_value_update(host, &model, range, down.position);
        if changed {
            crate::imui::mark_lifecycle_edit(host, acx, &lifecycle_model);
            host.record_transient_event(acx, KEY_CHANGED);
            host.notify(acx);
        }

        PressablePointerDownResult::Continue
    }));
}
