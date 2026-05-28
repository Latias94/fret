use fret_core::Point;
use fret_runtime::Model;
use fret_ui::action::{UiActionHostExt as _, UiPointerActionHost};

use super::super::SliderInteractionRange;

pub(super) fn apply_slider_pointer_value_update(
    host: &mut dyn UiPointerActionHost,
    model: &Model<f32>,
    range: SliderInteractionRange,
    pointer_position: Point,
) -> bool {
    let next = crate::imui::slider_value_from_pointer(
        host.bounds(),
        pointer_position,
        range.min,
        range.max,
        range.step,
    );
    let mut changed = false;
    let _ = host.update_model(model, |value: &mut f32| {
        let current = crate::imui::slider_clamp_and_snap(*value, range.min, range.max, range.step);
        if (current - next).abs() > f32::EPSILON {
            *value = next;
            changed = true;
        }
    });
    changed
}
