//! Paint-root cached edge overlay adapter contract.
//!
//! This module keeps cached edge selected/hovered overlay routing behind a named seam. Concrete
//! retained overlay painting lives next to the cached edge overlay binding.

use crate::ui::canvas::widget::{
    CanvasGeometry, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
};
use fret_ui::UiHost;

pub(super) trait PaintRootCachedEdgeOverlayCx<H: UiHost> {
    fn paint_root_cached_edge_overlays_selected_hovered<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware;
}

pub(super) fn paint_root_cached_edge_overlays_selected_hovered<H, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PaintRootCachedEdgeOverlayCx<H>,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
    zoom: f32,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    cx.paint_root_cached_edge_overlays_selected_hovered(canvas, snapshot, geom, zoom);
}
