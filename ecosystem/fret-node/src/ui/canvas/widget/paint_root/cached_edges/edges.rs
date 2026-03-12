#[path = "edges/fallback.rs"]
mod fallback;
#[path = "edges/replay.rs"]
mod replay;
#[path = "edges/single.rs"]
mod single;
#[path = "edges/tiled.rs"]
mod tiled;

use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn replay_cached_edge_build_state<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        state: &EdgesBuildState,
        replay_delta: Point,
    ) {
        replay::replay_cached_edge_build_state(self, cx, state, replay_delta);
    }

    fn store_finished_edge_build_state(&mut self, key: u64, state: EdgesBuildState) {
        replay::store_finished_edge_build_state(self, key, state);
    }

    pub(super) fn paint_root_edges_uncached<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        render_cull_rect: Option<Rect>,
        hovered_edge: Option<EdgeId>,
        zoom: f32,
        view_interacting: bool,
    ) {
        fallback::paint_root_edges_uncached(
            self,
            cx,
            snapshot,
            geom,
            index,
            render_cull_rect,
            hovered_edge,
            zoom,
            view_interacting,
        );
    }

    pub(super) fn try_replay_cached_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        key: u64,
        replay_delta: Point,
    ) -> bool {
        replay::try_replay_cached_edges(self, cx, key, replay_delta)
    }

    pub(super) fn build_single_rect_edges_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        edges_key: u64,
        edges_cache_rect: Rect,
        zoom: f32,
        view_interacting: bool,
        replay_delta: Point,
    ) {
        single::build_single_rect_edges_cache(
            self,
            cx,
            snapshot,
            geom,
            index,
            edges_key,
            edges_cache_rect,
            zoom,
            view_interacting,
            replay_delta,
        );
    }

    pub(super) fn paint_tiled_edges_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        tiles: &[TileCoord],
        base_key: DerivedBaseKey,
        style_key: u64,
        edges_cache_tile_size_canvas: f32,
        zoom: f32,
        view_interacting: bool,
        replay_delta: Point,
    ) {
        tiled::paint_tiled_edges_cache(
            self,
            cx,
            snapshot,
            geom,
            index,
            tiles,
            base_key,
            style_key,
            edges_cache_tile_size_canvas,
            zoom,
            view_interacting,
            replay_delta,
        );
    }
}
