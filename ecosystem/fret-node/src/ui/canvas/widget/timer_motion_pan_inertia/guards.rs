use crate::io::NodeGraphPanInertiaTuning;
use crate::ui::canvas::widget::*;

pub(super) fn should_stop_pan_inertia<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    tuning: &NodeGraphPanInertiaTuning,
    zoom: f32,
) -> bool {
    !tuning.enabled
        || !canvas.pan_inertia_should_tick()
        || !zoom.is_finite()
        || zoom <= 0.0
        || !tuning.decay_per_s.is_finite()
        || tuning.decay_per_s <= 0.0
}
