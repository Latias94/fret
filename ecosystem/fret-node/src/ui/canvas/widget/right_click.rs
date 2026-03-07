use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::Point;
use fret_ui::UiHost;

use super::context_menu::opening;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingRightClick, ViewSnapshot};

pub(super) fn pending_right_click_is_click(
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

pub(super) fn pending_right_click_exceeded_drag_threshold(
    pending: PendingRightClick,
    position: Point,
    click_distance: f32,
    zoom: f32,
) -> bool {
    !pending_right_click_is_click(pending, position, click_distance, zoom)
}

pub(super) fn handle_right_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    opening::handle_right_click_context_menu_event(canvas, cx, snapshot, position, zoom)
}

pub(super) fn handle_pending_right_click_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: fret_core::MouseButton,
    zoom: f32,
) -> bool {
    if button != fret_core::MouseButton::Right || !snapshot.interaction.pan_on_drag.right {
        return false;
    }

    let Some(pending) = canvas.interaction.pending_right_click.take() else {
        return false;
    };

    cx.release_pointer_capture();
    if pending_right_click_is_click(
        pending,
        position,
        snapshot.interaction.pane_click_distance,
        zoom,
    ) {
        let _ = handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom);
    }
    true
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
