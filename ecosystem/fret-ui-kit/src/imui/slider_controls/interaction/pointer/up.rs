use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::action::PressablePointerUpResult;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};
use crate::imui::mark_lifecycle_deactivated_on_left_pointer_up;

pub(super) fn install_slider_pointer_up_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if up.button == MouseButton::Left {
            mark_lifecycle_deactivated_on_left_pointer_up(host, acx, up.button, &lifecycle_model);
            host.release_pointer_capture();
            let _ = host.update_model(&active_item_model, |st| {
                if st.active == Some(id) {
                    st.active = None;
                }
            });
        }
        PressablePointerUpResult::Continue
    }));
}
