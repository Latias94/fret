#[path = "labels/replay.rs"]
mod replay;
#[path = "labels/single.rs"]
mod single;
#[path = "labels/tiled.rs"]
mod tiled;

use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn store_finished_edge_label_state(&mut self, key: u64, state: EdgeLabelsBuildState) {
        replay::store_finished_edge_label_state(self, key, state);
    }

    pub(super) fn try_replay_cached_edge_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        key: u64,
        replay_delta: Point,
    ) -> bool {
        replay::try_replay_cached_edge_labels(self, cx, key, replay_delta)
    }

    pub(super) fn build_single_rect_edge_labels_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        labels_key: u64,
        edges_cache_rect: Rect,
        zoom: f32,
        view_interacting: bool,
    ) {
        single::build_single_rect_edge_labels_cache(
            self,
            cx,
            snapshot,
            geom,
            index,
            labels_key,
            edges_cache_rect,
            zoom,
            view_interacting,
        );
    }

    pub(super) fn replay_single_rect_edge_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        labels_key: u64,
        replay_delta: Point,
    ) {
        replay::replay_single_rect_edge_labels(self, cx, labels_key, replay_delta);
    }

    pub(super) fn paint_tiled_edge_labels_cache<H: UiHost>(
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
        tiled::paint_tiled_edge_labels_cache(
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
