use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_release_cx::PointerUpReleaseCx,
    wire_drag::WireCommitCx,
};

pub(super) fn handle_active_release_chain<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: WireCommitCx<H> + PointerUpReleaseCx<H>,
{
    super::super::super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom)
        || super::super::super::edge_insert_drag::handle_edge_insert_left_up(canvas, cx, position)
        || super::super::super::edge_drag::handle_edge_left_up(canvas, cx)
}
