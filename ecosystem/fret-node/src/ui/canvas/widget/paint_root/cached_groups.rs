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
        let groups_key = {
            let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_groups.v1");
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
        let groups_hit =
            self.groups_scene_cache
                .try_replay_with(groups_key, cx.scene, replay_delta, |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                });
        if !groups_hit {
            let render_groups: RenderData = self.collect_render_data(
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
            self.paint_groups_static(
                &mut tmp,
                cx.services,
                cx.scale_factor,
                &render_groups.groups,
                zoom,
            );
            tmp.push(SceneOp::PopClip);
            self.groups_scene_cache
                .store_ops(groups_key, tmp.ops().to_vec());
            let _ = self.groups_scene_cache.try_replay_with(
                groups_key,
                cx.scene,
                replay_delta,
                |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                },
            );
        }

        // Selected group border overlay must remain ordered before edges (ADR 0082).
        let group_corner = Px(10.0 / zoom);
        let selected_groups = snapshot.selected_groups.clone();
        let _ = self.graph.read_ref(cx.app, |g| {
            for group_id in selected_groups {
                let Some(group) = g.groups.get(&group_id) else {
                    continue;
                };
                let rect0 = self.group_rect_with_preview(group_id, group.rect);
                let rect = Rect::new(
                    Point::new(Px(rect0.origin.x), Px(rect0.origin.y)),
                    Size::new(Px(rect0.size.width), Px(rect0.size.height)),
                );
                if render_cull_rect.is_some_and(|c| !rects_intersect(rect, c)) {
                    continue;
                }
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect,
                    background: self.style.group_background,
                    border: Edges::all(Px(1.0 / zoom)),
                    border_color: self.style.node_border_selected,
                    corner_radii: Corners::all(group_corner),
                });
            }
        });
    }
}
