mod dispatch;
mod double_click;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_left_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    canvas.stop_auto_pan_timer(cx.app);

    if double_click::handle_edge_insert_double_click(canvas, cx, position, click_count, modifiers) {
        return true;
    }

    dispatch::handle_left_release_chain(canvas, cx, snapshot, position, zoom)
}
