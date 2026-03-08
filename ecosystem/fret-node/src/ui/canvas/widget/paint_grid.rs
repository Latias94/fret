use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

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

        let canvas_hint = if let Some(skin) = self.skin.as_ref() {
            self.graph
                .read_ref(cx.app, |g| skin.canvas_chrome_hint(g, &self.style))
                .ok()
                .unwrap_or_default()
        } else {
            crate::ui::CanvasChromeHint::default()
        };

        let pattern = self.style.paint.grid_pattern;
        let spacing = self.style.paint.grid_spacing;
        if !(spacing.is_finite() && spacing > 1.0e-3) {
            return;
        }

        let line_width_px = if let Some(v) = canvas_hint.grid_line_width_px
            && v.is_finite()
            && v > 0.0
        {
            v
        } else if self.style.paint.grid_line_width.is_finite()
            && self.style.paint.grid_line_width > 0.0
        {
            self.style.paint.grid_line_width
        } else {
            1.0
        };
        let major_every = self.style.paint.grid_major_every.max(1) as i64;
        let z = zoom.max(1.0e-6);
        let thickness_px = line_width_px.max(0.25);
        let thickness = Px(thickness_px / z);
        let tile_size_canvas = (Self::GRID_TILE_SIZE_SCREEN_PX / zoom.max(1.0e-6)).max(1.0);
        let grid_rect = render_cull_rect.unwrap_or(viewport_rect);

        let grid_tiles = TileGrid2D::new(tile_size_canvas);
        grid_tiles.tiles_in_rect(grid_rect, &mut self.grid_tiles_scratch);
        grid_tiles.sort_tiles_center_first(viewport_rect, &mut self.grid_tiles_scratch);

        let major_color = canvas_hint
            .grid_major
            .unwrap_or(self.style.paint.grid_major_color);
        let minor_color = canvas_hint
            .grid_minor
            .unwrap_or(self.style.paint.grid_minor_color);
        let spacing_bits = spacing.to_bits();
        let thickness_bits = thickness.0.to_bits();
        let pattern_tag: u32 = match pattern {
            NodeGraphBackgroundPattern::Lines => 0,
            NodeGraphBackgroundPattern::Dots => 1,
            NodeGraphBackgroundPattern::Cross => 2,
        };
        let line_width_bits = line_width_px.to_bits();
        let dot_size = self.style.paint.grid_dot_size;
        let dot_size_bits = dot_size.to_bits();
        let cross_size = self.style.paint.grid_cross_size;
        let cross_size_bits = cross_size.to_bits();

        if matches!(pattern, NodeGraphBackgroundPattern::Dots)
            && !(dot_size.is_finite() && dot_size > 0.0)
        {
            return;
        }

        if matches!(pattern, NodeGraphBackgroundPattern::Cross)
            && !(cross_size.is_finite() && cross_size > 0.0)
        {
            return;
        }

        let tile_ops_for_key = |tile: TileCoord| -> Vec<SceneOp> {
            let tile_origin = tile.origin(tile_size_canvas);
            super::paint_grid_tiles::grid_tile_ops(
                pattern,
                tile_origin,
                tile_size_canvas,
                spacing,
                major_every,
                major_color,
                minor_color,
                thickness,
                dot_size,
                cross_size,
            )
        };

        let tile_budget_limit =
            Self::GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
        let mut tile_budget = WorkBudget::new(tile_budget_limit);
        let base_key = {
            let mut b = TileCacheKeyBuilder::new("fret-node.grid.tile.v1");
            b.add_f32_bits(zoom);
            b.add_f32_bits(tile_size_canvas);
            b.add_u32(pattern_tag);
            b.add_u32(spacing_bits);
            b.add_u32(line_width_bits);
            b.add_u32(thickness_bits);
            b.add_u32(dot_size_bits);
            b.add_u32(cross_size_bits);
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
        let warmup = warm_scene_op_tiles_u64_with(
            &mut self.grid_scene_cache,
            cx.scene,
            &self.grid_tiles_scratch,
            base_key,
            1,
            &mut tile_budget,
            |tile| tile.origin(tile_size_canvas),
            |_ops| {},
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
