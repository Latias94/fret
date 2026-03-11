use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::Point;

use crate::ui::canvas::state::PendingRightClick;

pub(in super::super) fn pending_right_click_is_click(
    pending: PendingRightClick,
    position: Point,
    click_distance: f32,
    zoom: f32,
) -> bool {
    let click_distance = click_distance.max(0.0);
    let threshold = canvas_units_from_screen_px(click_distance, zoom);
    let dx = position.x.0 - pending.start_pos.x.0;
    let dy = position.y.0 - pending.start_pos.y.0;
    click_distance == 0.0 || (dx * dx + dy * dy) <= threshold * threshold
}

pub(in super::super) fn pending_right_click_exceeded_drag_threshold(
    pending: PendingRightClick,
    position: Point,
    click_distance: f32,
    zoom: f32,
) -> bool {
    !pending_right_click_is_click(pending, position, click_distance, zoom)
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px};

    use super::*;

    fn pending(start_x: f32, start_y: f32) -> PendingRightClick {
        PendingRightClick {
            start_pos: Point::new(Px(start_x), Px(start_y)),
        }
    }

    #[test]
    fn pending_right_click_zero_distance_is_always_click() {
        assert!(pending_right_click_is_click(
            pending(10.0, 20.0),
            Point::new(Px(100.0), Px(200.0)),
            0.0,
            2.0,
        ));
    }

    #[test]
    fn pending_right_click_exceeded_threshold_matches_distance_check() {
        let pending = pending(10.0, 10.0);
        assert!(!pending_right_click_exceeded_drag_threshold(
            pending.clone(),
            Point::new(Px(11.0), Px(10.0)),
            4.0,
            1.0,
        ));
        assert!(pending_right_click_exceeded_drag_threshold(
            pending,
            Point::new(Px(20.0), Px(10.0)),
            4.0,
            1.0,
        ));
    }
}
