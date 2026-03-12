#[path = "edge_anchor/geometry.rs"]
mod geometry;
#[path = "edge_anchor/render.rs"]
mod render;
#[path = "edge_anchor/target_id.rs"]
mod target_id;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

pub(super) type EdgeAnchorTarget = (EdgeRouteKind, Point, Point, Color);

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn resolve_edge_anchor_target_id<H: UiHost>(
        &self,
        cx: &PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> Option<EdgeId> {
        target_id::resolve_edge_anchor_target_id(self, cx, snapshot)
    }

    pub(super) fn resolve_edge_anchor_target_from_render(
        &self,
        render: &RenderData,
        edge_id: Option<EdgeId>,
    ) -> Option<EdgeAnchorTarget> {
        render::resolve_edge_anchor_target_from_render(render, edge_id)
    }

    pub(super) fn resolve_edge_anchor_target_from_geometry<H: UiHost>(
        &self,
        cx: &PaintCx<'_, H>,
        geom: &CanvasGeometry,
        edge_id: Option<EdgeId>,
    ) -> Option<EdgeAnchorTarget> {
        geometry::resolve_edge_anchor_target_from_geometry(self, cx, geom, edge_id)
    }
}
