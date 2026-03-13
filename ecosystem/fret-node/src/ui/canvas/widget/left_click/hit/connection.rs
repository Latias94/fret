use fret_core::Point;

use crate::core::Graph;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{HitTestCtx, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

use super::Hit;

pub(super) fn compute_connection_hit<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    snapshot: &ViewSnapshot,
    ctx: &mut HitTestCtx<'_>,
    position: Point,
) -> Option<Hit> {
    if let Some(port) = canvas.hit_port(ctx, position) {
        return Some(Hit::Port(port));
    }

    canvas
        .hit_edge_focus_anchor(graph, snapshot, ctx, position)
        .map(|(edge, endpoint, fixed)| Hit::EdgeAnchor(edge, endpoint, fixed))
}
