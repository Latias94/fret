mod distance;
mod normalize;

use fret_core::Point;

pub(super) fn exceeds_drag_threshold(
    start: Point,
    position: Point,
    threshold_screen: f32,
    zoom: f32,
) -> bool {
    let threshold_screen = normalize::normalized_threshold_screen(threshold_screen);

    if threshold_screen <= 0.0 {
        return true;
    }

    distance::distance2(start, position)
        >= normalize::threshold_canvas_units(threshold_screen, zoom).powi(2)
}

#[cfg(test)]
mod tests;
