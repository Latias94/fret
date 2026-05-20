mod dispatch;
mod double_click;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_cx::PointerUpCx,
    pointer_up_release_cx::PointerUpReleaseCx,
};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_left_pointer_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    modifiers: Modifiers,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCx<H, M>,
{
    canvas.stop_auto_pan_timer(PointerUpReleaseCx::host(cx));

    if double_click::handle_edge_insert_double_click(canvas, cx, position, click_count, modifiers) {
        return true;
    }

    dispatch::handle_left_release_chain(canvas, cx, snapshot, position, zoom)
}
