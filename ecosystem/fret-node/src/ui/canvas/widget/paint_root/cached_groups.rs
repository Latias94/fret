use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_groups_cached_path<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        cache_rect: Rect,
        render_cull_rect: Option<Rect>,
        zoom: f32,
        base_key: DerivedBaseKey,
        style_key: u64,
        nodes_cache_tile_size_canvas: f32,
    ) {
        // --- Groups (static, cached) ---
        self.paint_root_static_layer_cached(
            cx,
            super::static_layer::StaticSceneLayerTarget::Groups,
            "fret-node.canvas.static_groups.v1",
            base_key,
            style_key,
            nodes_cache_tile_size_canvas,
            cache_rect,
            |paint_cache, ops| paint_cache.touch_text_blobs_in_scene_ops(ops),
            |canvas, cx| {
                let render_groups: RenderData = canvas.collect_render_data(
                    &*cx.app,
                    snapshot,
                    Arc::clone(geom),
                    Arc::clone(index),
                    Some(cache_rect),
                    zoom,
                    None,
                    true,
                    false,
                    false,
                );

                let mut tmp = fret_core::Scene::default();
                tmp.push(SceneOp::PushClipRect { rect: cache_rect });
                canvas.paint_groups_static(
                    &mut tmp,
                    cx.services,
                    cx.scale_factor,
                    &render_groups.groups,
                    zoom,
                );
                tmp.push(SceneOp::PopClip);
                tmp.ops().to_vec()
            },
        );

        // Selected group border overlay must remain ordered before edges (ADR 0081).
        self.paint_selected_groups_overlay_from_snapshot(cx, snapshot, render_cull_rect, zoom);
    }
}
