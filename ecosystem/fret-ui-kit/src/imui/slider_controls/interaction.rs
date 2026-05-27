use std::sync::Arc;

use fret_core::{KeyCode, MouseButton};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::interaction_runtime::ImUiActiveItemState;

pub(super) fn install_slider_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    enabled: bool,
    model: fret_runtime::Model<f32>,
    min: f32,
    max: f32,
    step: f32,
) -> fret_runtime::Model<ImUiActiveItemState> {
    cx.pressable_clear_on_pointer_down();
    cx.pressable_clear_on_pointer_move();
    cx.pressable_clear_on_pointer_up();
    cx.key_clear_on_key_down_for(id);

    let active_item_model = super::super::active_item_model_for_window(cx);
    let active_item_model_for_down = active_item_model.clone();
    let active_item_model_for_move = active_item_model.clone();
    let active_item_model_for_up = active_item_model.clone();
    let lifecycle_model = super::super::lifecycle_session_model_for(cx, id);
    let lifecycle_model_for_down = lifecycle_model.clone();
    let lifecycle_model_for_move = lifecycle_model.clone();
    let lifecycle_model_for_up = lifecycle_model.clone();
    let lifecycle_model_for_key = lifecycle_model.clone();

    let model_for_down = model.clone();
    cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
        if down.button != MouseButton::Left {
            return PressablePointerDownResult::Continue;
        }

        super::super::mark_lifecycle_activated_on_left_pointer_down(
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

        let next =
            super::super::slider_value_from_pointer(host.bounds(), down.position, min, max, step);
        let mut changed = false;
        let _ = host.update_model(&model_for_down, |value: &mut f32| {
            let current = super::super::slider_clamp_and_snap(*value, min, max, step);
            if (current - next).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        });
        if changed {
            super::super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_down);
            host.record_transient_event(acx, super::super::KEY_CHANGED);
            host.notify(acx);
        }

        PressablePointerDownResult::Continue
    }));

    let model_for_move = model.clone();
    cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
        if !mv.buttons.left {
            super::super::mark_lifecycle_deactivated_on_left_pointer_up(
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
            super::super::slider_value_from_pointer(host.bounds(), mv.position, min, max, step);
        let mut changed = false;
        let _ = host.update_model(&model_for_move, |value: &mut f32| {
            let current = super::super::slider_clamp_and_snap(*value, min, max, step);
            if (current - next).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        });
        if changed {
            super::super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_move);
            host.record_transient_event(acx, super::super::KEY_CHANGED);
            host.notify(acx);
        }
        changed
    }));

    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
        if up.button == MouseButton::Left {
            super::super::mark_lifecycle_deactivated_on_left_pointer_up(
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

    if enabled {
        let model_for_key = model;
        cx.key_on_key_down_for(
            id,
            Arc::new(move |host, acx, down| {
                let (min, max) = super::super::slider_normalize_range(min, max);
                let step = super::super::slider_step_or_default(step);
                let delta = match down.key {
                    KeyCode::ArrowLeft | KeyCode::ArrowDown => Some(-step),
                    KeyCode::ArrowRight | KeyCode::ArrowUp => Some(step),
                    KeyCode::PageDown => Some(-step * 10.0),
                    KeyCode::PageUp => Some(step * 10.0),
                    _ => None,
                };

                let mut changed = false;
                let _ = host.update_model(&model_for_key, |value: &mut f32| {
                    let current = super::super::slider_clamp_and_snap(*value, min, max, step);
                    let next = match down.key {
                        KeyCode::Home => min,
                        KeyCode::End => max,
                        _ => {
                            let Some(delta) = delta else {
                                return;
                            };
                            super::super::slider_clamp_and_snap(current + delta, min, max, step)
                        }
                    };
                    if (current - next).abs() > f32::EPSILON {
                        *value = next;
                        changed = true;
                    }
                });

                if changed {
                    super::super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_key);
                    host.record_transient_event(acx, super::super::KEY_CHANGED);
                    host.notify(acx);
                }

                changed
            }),
        );
    }

    active_item_model
}
