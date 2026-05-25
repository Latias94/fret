use crate::ui::canvas::widget::{
    CanvasGeometry, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
};
use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::overlay_adapter::PaintRootCachedEdgeOverlayCx<H> for PaintCx<'_, H> {
    fn paint_root_cached_edge_overlays_selected_hovered<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware,
    {
        canvas.paint_edge_overlays_selected_hovered(self, snapshot, geom, zoom);
    }
}
