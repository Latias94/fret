use crate::ui::canvas::widget::{
    CanvasGeometry, EdgeId, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
};
use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::anchor_target_adapter::PaintRootCachedEdgeAnchorTargetCx<H>
    for PaintCx<'_, H>
{
    fn resolve_paint_root_cached_edge_anchor_target<M>(
        &self,
        canvas: &NodeGraphCanvasWith<M>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
    ) -> (
        Option<EdgeId>,
        Option<super::anchor_target_adapter::PaintRootCachedEdgeAnchorTarget>,
    )
    where
        M: NodeGraphCanvasMiddleware,
    {
        let edge_anchor_target_id = canvas.resolve_edge_anchor_target_id(self, snapshot);
        let edge_anchor_target =
            canvas.resolve_edge_anchor_target_from_geometry(self, geom, edge_anchor_target_id);
        (edge_anchor_target_id, edge_anchor_target)
    }
}
