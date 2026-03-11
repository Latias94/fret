mod pending;
mod threshold;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) use threshold::pending_right_click_exceeded_drag_threshold;

pub(super) fn handle_right_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    pending::handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom)
}

pub(super) fn handle_pending_right_click_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: fret_core::MouseButton,
    zoom: f32,
) -> bool {
    pending::handle_pending_right_click_pointer_up(canvas, cx, snapshot, position, button, zoom)
}
