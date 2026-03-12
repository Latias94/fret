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
mod tests;
