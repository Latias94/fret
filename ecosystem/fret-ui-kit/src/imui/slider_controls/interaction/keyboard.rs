use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::SliderInteractionRange;
use crate::imui::interaction_runtime::ImUiLifecycleSessionState;
use crate::imui::{
    KEY_CHANGED, mark_lifecycle_edit, slider_clamp_and_snap, slider_normalize_range,
    slider_step_or_default,
};

pub(super) fn install_slider_keyboard_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    enabled: bool,
    model: Model<f32>,
    range: SliderInteractionRange,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
) {
    if !enabled {
        return;
    }

    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            let (min, max) = slider_normalize_range(range.min, range.max);
            let step = slider_step_or_default(range.step);
            let delta = match down.key {
                KeyCode::ArrowLeft | KeyCode::ArrowDown => Some(-step),
                KeyCode::ArrowRight | KeyCode::ArrowUp => Some(step),
                KeyCode::PageDown => Some(-step * 10.0),
                KeyCode::PageUp => Some(step * 10.0),
                _ => None,
            };

            let mut changed = false;
            let _ = host.update_model(&model, |value: &mut f32| {
                let current = slider_clamp_and_snap(*value, min, max, step);
                let next = match down.key {
                    KeyCode::Home => min,
                    KeyCode::End => max,
                    _ => {
                        let Some(delta) = delta else {
                            return;
                        };
                        slider_clamp_and_snap(current + delta, min, max, step)
                    }
                };
                if (current - next).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
            });

            if changed {
                mark_lifecycle_edit(host, acx, &lifecycle_model);
                host.record_transient_event(acx, KEY_CHANGED);
                host.notify(acx);
            }

            changed
        }),
    );
}
