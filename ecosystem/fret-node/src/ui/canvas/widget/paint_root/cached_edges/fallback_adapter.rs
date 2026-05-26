//! Paint-root cached edge fallback adapter contract.
//!
//! This module keeps cached edge fallback host access and retained edge paint dispatch behind a
//! named seam. Concrete retained bindings live next to the cached edge fallback binding.

use crate::ui::canvas::widget::{
    CanvasGeometry, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    paint_render_data::RenderData,
};
use fret_ui::UiHost;

pub(super) trait PaintRootCachedEdgeFallbackCx<H: UiHost> {
    fn paint_root_cached_edge_fallback_host(&self) -> &H;

    fn paint_root_cached_edge_fallback_paint_edges<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        snapshot: &ViewSnapshot,
        render_edges: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        view_interacting: bool,
    ) where
        M: NodeGraphCanvasMiddleware;
}

pub(super) fn paint_root_cached_edge_fallback_host<H>(
    cx: &impl PaintRootCachedEdgeFallbackCx<H>,
) -> &H
where
    H: UiHost,
{
    cx.paint_root_cached_edge_fallback_host()
}

pub(super) fn paint_root_cached_edge_fallback_paint_edges<H, M>(
    cx: &mut impl PaintRootCachedEdgeFallbackCx<H>,
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    render_edges: &RenderData,
    geom: &CanvasGeometry,
    zoom: f32,
    view_interacting: bool,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    cx.paint_root_cached_edge_fallback_paint_edges(
        canvas,
        snapshot,
        render_edges,
        geom,
        zoom,
        view_interacting,
    );
}
