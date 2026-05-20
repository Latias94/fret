mod bounds;
mod clamp_delta;

use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    node_drag_geometry_cx::NodeDragGeometryCx,
};

pub(super) fn apply_multi_drag_extent_delta<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    node_ids: &[GraphNodeId],
    delta: CanvasPoint,
    multi_drag: bool,
) -> CanvasPoint
where
    M: NodeGraphCanvasMiddleware,
    Cx: NodeDragGeometryCx<H>,
{
    if !multi_drag {
        return delta;
    }

    let Some(extent) = snapshot.interaction.node_extent else {
        return delta;
    };

    let geometry = {
        let host = cx.host();
        canvas.canvas_geometry(&*host, snapshot)
    };
    let Some((group_min, group_size)) = bounds::dragged_group_bounds(&geometry, node_ids) else {
        return delta;
    };

    clamp_delta::clamp_delta_for_extent_rect(delta, group_min, group_size, extent)
}
