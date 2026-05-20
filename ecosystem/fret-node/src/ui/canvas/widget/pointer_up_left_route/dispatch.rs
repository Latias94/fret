mod active;
mod commit;
mod pending;

use fret_core::Point;
use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_cx::PointerUpCx};
use crate::ui::canvas::state::ViewSnapshot;

pub(in super::super) fn handle_left_release_chain<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCx<H, M>,
{
    if super::super::marquee::handle_left_up(canvas, cx) {
        return true;
    }

    if commit::handle_commit_release_chain(canvas, cx, snapshot) {
        return true;
    }

    if pending::handle_pending_release_chain(canvas, cx, snapshot, position, zoom) {
        return true;
    }

    if active::handle_active_release_chain(canvas, cx, snapshot, position, zoom) {
        return true;
    }

    false
}
