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
        let nodes_key = {
            let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_nodes.v1");
            b.add_u64(base_key.graph_rev);
            b.add_u32(base_key.zoom_bits);
            b.add_u32(base_key.node_origin_x_bits);
            b.add_u32(base_key.node_origin_y_bits);
            b.add_u64(base_key.draw_order.lo);
            b.add_u64(base_key.draw_order.hi);
            b.add_u64(base_key.presenter_rev);
            b.add_u64(base_key.edge_types_rev);
            b.add_u64(style_key);
            b.add_f32_bits(nodes_cache_tile_size_canvas);
            b.add_u32(cache_rect.origin.x.0.to_bits());
            b.add_u32(cache_rect.origin.y.0.to_bits());
            b.finish()
        };

        let replay_delta = Point::new(Px(0.0), Px(0.0));
        let nodes_hit =
            self.nodes_scene_cache
                .try_replay_with(nodes_key, cx.scene, replay_delta, |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                });
        if !nodes_hit {
            let render_nodes: RenderData = self.collect_render_data(
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
            self.paint_nodes_static(&mut tmp, cx.services, cx.scale_factor, &render_nodes, zoom);
            tmp.push(SceneOp::PopClip);
            self.nodes_scene_cache
                .store_ops(nodes_key, tmp.ops().to_vec());
            let _ =
                self.nodes_scene_cache
                    .try_replay_with(nodes_key, cx.scene, replay_delta, |ops| {
                        self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                    });
        }

        if snapshot.interaction.elevate_nodes_on_select {
            let render_selected = self.collect_selected_nodes_render_data(
                &*cx.app,
                snapshot,
                geom,
                render_cull_rect,
                zoom,
            );
            if !render_selected.nodes.is_empty() {
                self.paint_nodes_static(
                    cx.scene,
                    cx.services,
                    cx.scale_factor,
                    &render_selected,
                    zoom,
                );
            }
        }

        // --- Nodes (dynamic overlays) ---
        self.paint_nodes_dynamic_from_geometry(cx, snapshot, geom, zoom);
    }
}
