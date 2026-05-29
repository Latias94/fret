use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, UiHost};

use super::super::SliderInteractionRange;
use super::value_update::apply_slider_pointer_value_update;
use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};
use crate::imui::{KEY_CHANGED, mark_lifecycle_deactivated_on_left_pointer_up};

pub(super) fn install_slider_pointer_move_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<f32>,
    range: SliderInteractionRange,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
        if !mv.buttons.left {
            mark_lifecycle_deactivated_on_left_pointer_up(
                host,
                acx,
                MouseButton::Left,
                &lifecycle_model,
            );
            host.release_pointer_capture();
            let _ = host.update_model(&active_item_model, |st| {
                if st.active == Some(acx.target) {
                    st.active = None;
                }
            });
            return false;
        }

        let changed = apply_slider_pointer_value_update(host, &model, range, mv.position);
        if changed {
            crate::imui::mark_lifecycle_edit(host, acx, &lifecycle_model);
            host.record_transient_event(acx, KEY_CHANGED);
            host.notify(acx);
        }
        changed
    }));
}
