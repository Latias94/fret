mod hover;
mod tiles;

use crate::ui::canvas::widget::*;

pub(super) const STATIC_NODES_TILE_MUL: f32 = 2.0;
pub(super) const STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN: u32 = 1024;
pub(super) const STATIC_EDGES_TILE_SIZE_SCREEN_PX: u32 = 2048;

pub(super) struct PaintRootCachePlan {
    pub(super) hovered_edge: Option<EdgeId>,
    pub(super) geom: Arc<CanvasGeometry>,
    pub(super) index: Arc<CanvasSpatialDerived>,
    pub(super) nodes_cache_tile_size_canvas: f32,
    pub(super) edges_cache_tile_size_canvas: f32,
    pub(super) nodes_cache_rect: Option<Rect>,
    pub(super) edges_cache_rect: Option<Rect>,
    pub(super) style_key: u64,
    pub(super) base_key: DerivedBaseKey,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn prepare_paint_root_cache_plan<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        viewport_rect: Rect,
        viewport_w: f32,
        viewport_h: f32,
    ) -> PaintRootCachePlan {
        let hovered_edge = hover::resolve_paint_root_hovered_edge(&self.interaction);
        let (geom, index) = self.canvas_derived(&*cx.app, snapshot);
        self.publish_derived_outputs(&*cx.app, snapshot, cx.bounds, &geom);

        let zoom = snapshot.zoom;
        let can_use_static_scene_cache = tiles::can_use_static_scene_cache(
            snapshot,
            cx.bounds,
            self.geometry.drag_preview.is_none(),
        );
        let tile_sizes = tiles::static_scene_cache_tile_sizes(cx.bounds, zoom);
        let nodes_cache_rect = tiles::static_cache_rect(
            can_use_static_scene_cache,
            viewport_rect,
            viewport_w,
            viewport_h,
            tile_sizes.nodes_cache_tile_size_canvas,
        );
        let edges_cache_rect = tiles::static_cache_rect(
            can_use_static_scene_cache,
            viewport_rect,
            viewport_w,
            viewport_h,
            tile_sizes.edges_cache_tile_size_canvas,
        );

        let style_key = self.static_scene_style_key(cx.scale_factor);
        let geom_key = self
            .geometry
            .geom_key
            .unwrap_or_else(|| self.geometry_key(&*cx.app, snapshot));

        PaintRootCachePlan {
            hovered_edge,
            geom,
            index,
            nodes_cache_tile_size_canvas: tile_sizes.nodes_cache_tile_size_canvas,
            edges_cache_tile_size_canvas: tile_sizes.edges_cache_tile_size_canvas,
            nodes_cache_rect,
            edges_cache_rect,
            style_key,
            base_key: geom_key.base,
        }
    }
}
