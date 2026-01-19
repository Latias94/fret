use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_grid<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        viewport_rect: Rect,
        render_cull_rect: Option<Rect>,
        zoom: f32,
        view_interacting: bool,
    ) {
        self.grid_scene_cache.begin_frame();

        let spacing = self.style.grid_spacing;
        if !(spacing.is_finite() && spacing > 1.0e-3) {
            return;
        }

        let major_every = self.style.grid_major_every.max(1) as i64;
        let thickness = Px((1.0 / zoom).max(0.25 / zoom));
        let tile_size_canvas = (Self::GRID_TILE_SIZE_SCREEN_PX / zoom.max(1.0e-6)).max(1.0);
        let grid_rect = render_cull_rect.unwrap_or(viewport_rect);

        let grid_tiles = TileGrid2D::new(tile_size_canvas);
        grid_tiles.tiles_in_rect(grid_rect, &mut self.grid_tiles_scratch);
        grid_tiles.sort_tiles_center_first(viewport_rect, &mut self.grid_tiles_scratch);

        let major_color = self.style.grid_major_color;
        let minor_color = self.style.grid_minor_color;
        let spacing_bits = spacing.to_bits();
        let thickness_bits = thickness.0.to_bits();

        let tile_ops_for_key = |tile: TileCoord| -> Vec<SceneOp> {
            let tile_origin = tile.origin(tile_size_canvas);
            let tile_min_x = tile_origin.x.0;
            let tile_min_y = tile_origin.y.0;
            let tile_max_x = tile_min_x + tile_size_canvas;
            let tile_max_y = tile_min_y + tile_size_canvas;

            let x0 = (tile_min_x / spacing).floor() as i64;
            let x1 = (tile_max_x / spacing).ceil() as i64;
            let y0 = (tile_min_y / spacing).floor() as i64;
            let y1 = (tile_max_y / spacing).ceil() as i64;

            let approx_v = (x1 - x0 + 1).max(0) as usize;
            let approx_h = (y1 - y0 + 1).max(0) as usize;
            let mut ops: Vec<SceneOp> = Vec::with_capacity(approx_v + approx_h);

            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let color = if ix.rem_euclid(major_every) == 0 {
                    major_color
                } else {
                    minor_color
                };
                ops.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(x - tile_origin.x.0 - 0.5 * thickness.0), Px(0.0)),
                        Size::new(thickness, Px(tile_size_canvas)),
                    ),
                    background: color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            for iy in y0..=y1 {
                let y = iy as f32 * spacing;
                let color = if iy.rem_euclid(major_every) == 0 {
                    major_color
                } else {
                    minor_color
                };
                ops.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(0.0), Px(y - tile_origin.y.0 - 0.5 * thickness.0)),
                        Size::new(Px(tile_size_canvas), thickness),
                    ),
                    background: color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            ops
        };

        let tile_budget_limit =
            Self::GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
        let mut tile_budget = WorkBudget::new(tile_budget_limit);
        let base_key = {
            let mut b = TileCacheKeyBuilder::new("fret-node.grid.tile.v1");
            b.add_f32_bits(zoom);
            b.add_f32_bits(tile_size_canvas);
            b.add_u32(spacing_bits);
            b.add_u32(thickness_bits);
            b.add_i64(major_every);
            b.add_u32(major_color.r.to_bits());
            b.add_u32(major_color.g.to_bits());
            b.add_u32(major_color.b.to_bits());
            b.add_u32(major_color.a.to_bits());
            b.add_u32(minor_color.r.to_bits());
            b.add_u32(minor_color.g.to_bits());
            b.add_u32(minor_color.b.to_bits());
            b.add_u32(minor_color.a.to_bits());
            b.finish()
        };
        let warmup = warm_scene_op_tiles_u64(
            &mut self.grid_scene_cache,
            cx.scene,
            &self.grid_tiles_scratch,
            base_key,
            1,
            &mut tile_budget,
            |tile| tile.origin(tile_size_canvas),
            tile_ops_for_key,
        );
        let skipped_tiles = warmup.skipped_tiles;

        if skipped_tiles > 0 {
            cx.request_redraw();
        }

        if let Some(window) = cx.window {
            let frame_id = cx.app.frame_id().0;
            let tile_entries = self.grid_scene_cache.entries_len();
            let tile_stats = self.grid_scene_cache.stats();
            let requested_tiles = self.grid_tiles_scratch.len();
            let tile_key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-node.canvas.grid_tiles",
            };
            cx.app
                .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                    registry.record_scene_op_tile_cache_with_budget(
                        tile_key,
                        frame_id,
                        tile_entries,
                        requested_tiles,
                        tile_budget_limit,
                        tile_budget.used(),
                        skipped_tiles,
                        tile_stats,
                    );
                });
        }
    }
}
