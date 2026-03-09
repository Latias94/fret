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
    fn resolve_paint_root_hovered_edge(&self) -> Option<EdgeId> {
        let edge_insert_target = self
            .interaction
            .edge_insert_drag
            .as_ref()
            .map(|drag| drag.edge)
            .or_else(|| {
                self.interaction
                    .pending_edge_insert_drag
                    .as_ref()
                    .map(|drag| drag.edge)
            });
        let insert_node_drag_edge = self
            .interaction
            .insert_node_drag_preview
            .as_ref()
            .and_then(|preview| preview.edge);
        edge_insert_target
            .or(insert_node_drag_edge)
            .or(self.interaction.hover_edge)
    }

    fn static_cache_rect(
        &self,
        can_use_static_scene_cache: bool,
        viewport_rect: Rect,
        viewport_w: f32,
        viewport_h: f32,
        tile_size_canvas: f32,
    ) -> Option<Rect> {
        if can_use_static_scene_cache
            && tile_size_canvas >= viewport_w
            && tile_size_canvas >= viewport_h
        {
            crate::ui::canvas::widget::static_scene_cache_plan::centered_single_tile_rect(
                viewport_rect,
                tile_size_canvas,
            )
        } else {
            None
        }
    }

    pub(super) fn prepare_paint_root_cache_plan<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        viewport_rect: Rect,
        viewport_w: f32,
        viewport_h: f32,
    ) -> PaintRootCachePlan {
        let hovered_edge = self.resolve_paint_root_hovered_edge();
        let (geom, index) = self.canvas_derived(&*cx.app, snapshot);
        self.publish_derived_outputs(&*cx.app, snapshot, cx.bounds, &geom);

        let zoom = snapshot.zoom;
        let can_use_static_scene_cache = self.geometry.drag_preview.is_none()
            && snapshot.interaction.only_render_visible_elements
            && zoom.is_finite()
            && zoom > 1.0e-6
            && cx.bounds.size.width.0.is_finite()
            && cx.bounds.size.height.0.is_finite();

        let viewport_max_screen_px = cx.bounds.size.width.0.max(cx.bounds.size.height.0);
        let nodes_tile_size_screen_px =
            crate::ui::canvas::widget::static_scene_cache_plan::next_power_of_two_at_least(
                STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN,
                viewport_max_screen_px * STATIC_NODES_TILE_MUL,
            );
        let nodes_cache_tile_size_canvas = (nodes_tile_size_screen_px as f32 / zoom).max(1.0);
        let edges_cache_tile_size_canvas =
            (STATIC_EDGES_TILE_SIZE_SCREEN_PX as f32 / zoom).max(1.0);

        let nodes_cache_rect = self.static_cache_rect(
            can_use_static_scene_cache,
            viewport_rect,
            viewport_w,
            viewport_h,
            nodes_cache_tile_size_canvas,
        );
        let edges_cache_rect = self.static_cache_rect(
            can_use_static_scene_cache,
            viewport_rect,
            viewport_w,
            viewport_h,
            edges_cache_tile_size_canvas,
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
            nodes_cache_tile_size_canvas,
            edges_cache_tile_size_canvas,
            nodes_cache_rect,
            edges_cache_rect,
            style_key,
            base_key: geom_key.base,
        }
    }
}
