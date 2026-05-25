//! Paint-root cached edge anchor target adapter contract.
//!
//! This module keeps cached edge anchor target routing behind a named seam. Concrete retained
//! target id and geometry resolution lives next to the cached edge anchor target binding.

use crate::ui::canvas::widget::{
    CanvasGeometry, EdgeId, EdgeRouteKind, NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
    ViewSnapshot,
};
use fret_core::{Color, Point};
use fret_ui::UiHost;

pub(super) type PaintRootCachedEdgeAnchorTarget = (EdgeRouteKind, Point, Point, Color);

pub(super) trait PaintRootCachedEdgeAnchorTargetCx<H: UiHost> {
    fn resolve_paint_root_cached_edge_anchor_target<M>(
        &self,
        canvas: &NodeGraphCanvasWith<M>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
    ) -> (Option<EdgeId>, Option<PaintRootCachedEdgeAnchorTarget>)
    where
        M: NodeGraphCanvasMiddleware;
}

pub(super) fn resolve_paint_root_cached_edge_anchor_target<H, M>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &impl PaintRootCachedEdgeAnchorTargetCx<H>,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
) -> (Option<EdgeId>, Option<PaintRootCachedEdgeAnchorTarget>)
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    cx.resolve_paint_root_cached_edge_anchor_target(canvas, snapshot, geom)
}
