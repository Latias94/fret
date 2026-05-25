use crate::ui::canvas::widget::{
    CanvasGeometry, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    paint_render_data::RenderData,
};
use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::fallback_adapter::PaintRootCachedEdgeFallbackCx<H> for PaintCx<'_, H> {
    fn paint_root_cached_edge_fallback_host(&self) -> &H {
        &*self.app
    }

    fn paint_root_cached_edge_fallback_paint_edges<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        snapshot: &ViewSnapshot,
        render_edges: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        view_interacting: bool,
    ) where
        M: NodeGraphCanvasMiddleware,
    {
        canvas.paint_edges(self, snapshot, render_edges, geom, zoom, view_interacting);
    }
}
