use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use super::super::context_menu::opening;
use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(in super::super) fn handle_right_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    opening::handle_right_click_context_menu_event(canvas, cx, snapshot, position, zoom)
}

pub(in super::super) fn handle_pending_right_click_pointer_up<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Right || !snapshot.interaction.pan_on_drag.right {
        return false;
    }

    let Some(pending) = canvas.interaction.pending_right_click.take() else {
        return false;
    };

    cx.release_pointer_capture();
    if super::threshold::pending_right_click_is_click(
        pending,
        position,
        snapshot.interaction.pane_click_distance,
        zoom,
    ) {
        let _ = handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom);
    }
    true
}
