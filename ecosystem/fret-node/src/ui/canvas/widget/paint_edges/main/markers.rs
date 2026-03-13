use crate::ui::canvas::widget::*;

pub(super) fn push_optional_drop_marker(
    scene: &mut fret_core::Scene,
    marker: Option<(Point, Color)>,
    zoom: f32,
) {
    if let Some((pos, color)) = marker {
        super::super::preview::push_drop_marker(scene, pos, color, zoom);
    }
}
