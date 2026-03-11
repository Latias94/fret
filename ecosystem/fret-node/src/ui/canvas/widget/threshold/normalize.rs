use fret_canvas::scale::canvas_units_from_screen_px;

pub(super) fn normalized_threshold_screen(threshold_screen: f32) -> f32 {
    if threshold_screen.is_finite() {
        threshold_screen.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn threshold_canvas_units(threshold_screen: f32, zoom: f32) -> f32 {
    canvas_units_from_screen_px(threshold_screen, zoom)
}
