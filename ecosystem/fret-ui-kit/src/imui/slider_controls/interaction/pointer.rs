use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::SliderInteractionRange;
use crate::imui::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};
use crate::imui::{
    KEY_CHANGED, mark_lifecycle_activated_on_left_pointer_down,
    mark_lifecycle_deactivated_on_left_pointer_up, mark_lifecycle_edit, slider_clamp_and_snap,
    slider_value_from_pointer,
};

pub(super) fn install_slider_pointer_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    model: Model<f32>,
    range: SliderInteractionRange,
    active_item_model: Model<ImUiActiveItemState>,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_move = active_item_model.clone();
    let active_item_model_for_up = active_item_model;
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_move = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model;

    let model_for_down = model.clone();
    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }

        mark_lifecycle_activated_on_left_pointer_down(
            host,
            acx,
            down.button,
            &lifecycle_model_for_down,
        );
        let _ = host.update_model(&active_item_model_for_down, |st| {
            st.active = Some(acx.target);
        });
        host.capture_pointer();
        host.request_focus(acx.target);

        let next = slider_value_from_pointer(
            host.bounds(),
            down.position,
            range.min,
            range.max,
            range.step,
        );
        let mut changed = false;
        let _ = host.update_model(&model_for_down, |value: &mut f32| {
            let current = slider_clamp_and_snap(*value, range.min, range.max, range.step);
            if (current - next).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        });
        if changed {
            mark_lifecycle_edit(host, acx, &lifecycle_model_for_down);
            host.record_transient_event(acx, KEY_CHANGED);
            host.notify(acx);
        }

        PressablePointerDownResult::Continue
    }));

    let model_for_move = model;
    cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
        if !mv.buttons.left {
            mark_lifecycle_deactivated_on_left_pointer_up(
                host,
                acx,
                MouseButton::Left,
                &lifecycle_model_for_move,
            );
            host.release_pointer_capture();
            let _ = host.update_model(&active_item_model_for_move, |st| {
                if st.active == Some(acx.target) {
                    st.active = None;
                }
            });
            return false;
        }

        let next =
            slider_value_from_pointer(host.bounds(), mv.position, range.min, range.max, range.step);
        let mut changed = false;
        let _ = host.update_model(&model_for_move, |value: &mut f32| {
            let current = slider_clamp_and_snap(*value, range.min, range.max, range.step);
            if (current - next).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        });
        if changed {
            mark_lifecycle_edit(host, acx, &lifecycle_model_for_move);
            host.record_transient_event(acx, KEY_CHANGED);
            host.notify(acx);
        }
        changed
    }));

    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if up.button == MouseButton::Left {
            mark_lifecycle_deactivated_on_left_pointer_up(
                host,
                acx,
                up.button,
                &lifecycle_model_for_up,
            );
            host.release_pointer_capture();
            let _ = host.update_model(&active_item_model_for_up, |st| {
                if st.active == Some(id) {
                    st.active = None;
                }
            });
        }
        PressablePointerUpResult::Continue
    }));
}
