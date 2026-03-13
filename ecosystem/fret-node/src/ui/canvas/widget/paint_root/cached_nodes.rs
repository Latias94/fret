use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_nodes_cached_path<H: UiHost>(
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
        // --- Nodes (static, cached) ---
        self.paint_root_static_layer_cached(
            cx,
            super::static_layer::StaticSceneLayerTarget::Nodes,
            "fret-node.canvas.static_nodes.v1",
            base_key,
            style_key,
            nodes_cache_tile_size_canvas,
            cache_rect,
            |paint_cache, ops| paint_cache.touch_text_blobs_in_scene_ops(ops),
            |canvas, cx| {
                let render_nodes: RenderData = canvas.collect_render_data(
                    &*cx.app,
                    snapshot,
                    Arc::clone(geom),
                    Arc::clone(index),
                    Some(cache_rect),
                    zoom,
                    None,
                    false,
                    true,
                    false,
                );

                let mut tmp = fret_core::Scene::default();
                tmp.push(SceneOp::PushClipRect { rect: cache_rect });
                canvas.paint_nodes_static(
                    &mut tmp,
                    cx.services,
                    cx.scale_factor,
                    &render_nodes,
                    zoom,
                );
                tmp.push(SceneOp::PopClip);
                tmp.ops().to_vec()
            },
        );

        self.paint_root_node_overlay_layers(cx, snapshot, geom, render_cull_rect, zoom);
    }
}
