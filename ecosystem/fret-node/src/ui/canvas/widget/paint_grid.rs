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

        let pattern = self.style.grid_pattern;
        let spacing = self.style.grid_spacing;
        if !(spacing.is_finite() && spacing > 1.0e-3) {
            return;
        }

        let line_width_px =
            if self.style.grid_line_width.is_finite() && self.style.grid_line_width > 0.0 {
                self.style.grid_line_width
            } else {
                1.0
            };
        let major_every = self.style.grid_major_every.max(1) as i64;
        let z = zoom.max(1.0e-6);
        let thickness_px = line_width_px.max(0.25);
        let thickness = Px(thickness_px / z);
        let tile_size_canvas = (Self::GRID_TILE_SIZE_SCREEN_PX / zoom.max(1.0e-6)).max(1.0);
        let grid_rect = render_cull_rect.unwrap_or(viewport_rect);

        let grid_tiles = TileGrid2D::new(tile_size_canvas);
        grid_tiles.tiles_in_rect(grid_rect, &mut self.grid_tiles_scratch);
        grid_tiles.sort_tiles_center_first(viewport_rect, &mut self.grid_tiles_scratch);

        let major_color = self.style.grid_major_color;
        let minor_color = self.style.grid_minor_color;
        let spacing_bits = spacing.to_bits();
        let thickness_bits = thickness.0.to_bits();
        let pattern_tag: u32 = match pattern {
            NodeGraphBackgroundPattern::Lines => 0,
            NodeGraphBackgroundPattern::Dots => 1,
            NodeGraphBackgroundPattern::Cross => 2,
        };
        let line_width_bits = line_width_px.to_bits();
        let dot_size = self.style.grid_dot_size;
        let dot_size_bits = dot_size.to_bits();
        let cross_size = self.style.grid_cross_size;
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
            grid_tile_ops(
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

fn grid_tile_ops(
    pattern: NodeGraphBackgroundPattern,
    tile_origin: Point,
    tile_size_canvas: f32,
    spacing: f32,
    major_every: i64,
    major_color: Color,
    minor_color: Color,
    thickness: Px,
    dot_size: f32,
    cross_size: f32,
) -> Vec<SceneOp> {
    let tile_min_x = tile_origin.x.0;
    let tile_min_y = tile_origin.y.0;
    let tile_max_x = tile_min_x + tile_size_canvas;
    let tile_max_y = tile_min_y + tile_size_canvas;

    let x0 = (tile_min_x / spacing).floor() as i64;
    let x1 = (tile_max_x / spacing).ceil() as i64;
    let y0 = (tile_min_y / spacing).floor() as i64;
    let y1 = (tile_max_y / spacing).ceil() as i64;

    let approx_x = (x1 - x0 + 1).max(0) as usize;
    let approx_y = (y1 - y0 + 1).max(0) as usize;
    let approx_points = approx_x.saturating_mul(approx_y);
    let approx_ops = match pattern {
        NodeGraphBackgroundPattern::Lines => approx_x.saturating_add(approx_y),
        NodeGraphBackgroundPattern::Dots => approx_points,
        NodeGraphBackgroundPattern::Cross => approx_points.saturating_mul(2),
    };
    let mut ops: Vec<SceneOp> = Vec::with_capacity(approx_ops);

    match pattern {
        NodeGraphBackgroundPattern::Lines => {
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
                    background: fret_core::Paint::Solid(color),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

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
                    background: fret_core::Paint::Solid(color),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }
        NodeGraphBackgroundPattern::Dots => {
            let d = dot_size.max(0.0);
            let r = 0.5 * d;
            if !(d.is_finite() && d > 0.0) {
                return ops;
            }

            let corner = Corners::all(Px(r));
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let x_local = x - tile_origin.x.0;
                for iy in y0..=y1 {
                    let y = iy as f32 * spacing;
                    let y_local = y - tile_origin.y.0;

                    let is_major =
                        ix.rem_euclid(major_every) == 0 && iy.rem_euclid(major_every) == 0;
                    let color = if is_major { major_color } else { minor_color };

                    ops.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: Rect::new(
                            Point::new(Px(x_local - r), Px(y_local - r)),
                            Size::new(Px(d), Px(d)),
                        ),
                        background: fret_core::Paint::Solid(color),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: corner,
                    });
                }
            }
        }
        NodeGraphBackgroundPattern::Cross => {
            let s = cross_size.max(0.0);
            if !(s.is_finite() && s > 0.0) {
                return ops;
            }

            let half = 0.5 * s;
            for ix in x0..=x1 {
                let x = ix as f32 * spacing;
                let x_local = x - tile_origin.x.0;
                for iy in y0..=y1 {
                    let y = iy as f32 * spacing;
                    let y_local = y - tile_origin.y.0;

                    let is_major =
                        ix.rem_euclid(major_every) == 0 || iy.rem_euclid(major_every) == 0;
                    let color = if is_major { major_color } else { minor_color };

                    // Vertical segment.
                    ops.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: Rect::new(
                            Point::new(Px(x_local - 0.5 * thickness.0), Px(y_local - half)),
                            Size::new(thickness, Px(s)),
                        ),
                        background: fret_core::Paint::Solid(color),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: Corners::all(Px(0.0)),
                    });
                    // Horizontal segment.
                    ops.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: Rect::new(
                            Point::new(Px(x_local - half), Px(y_local - 0.5 * thickness.0)),
                            Size::new(Px(s), thickness),
                        ),
                        background: fret_core::Paint::Solid(color),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::grid_tile_ops;
    use crate::ui::style::NodeGraphBackgroundPattern;
    use fret_core::{Color, DrawOrder, Edges, Px};

    #[test]
    fn dots_pattern_emits_rounded_quads() {
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let ops = grid_tile_ops(
            NodeGraphBackgroundPattern::Dots,
            fret_core::Point::new(Px(0.0), Px(0.0)),
            100.0,
            20.0,
            4,
            white,
            white,
            Px(1.0),
            2.0,
            6.0,
        );

        assert!(!ops.is_empty());
        let any_rounded = ops.iter().any(|op| match op {
            fret_core::SceneOp::Quad { corner_radii, .. } => {
                corner_radii.top_left.0 > 0.0
                    || corner_radii.top_right.0 > 0.0
                    || corner_radii.bottom_left.0 > 0.0
                    || corner_radii.bottom_right.0 > 0.0
            }
            _ => false,
        });
        assert!(any_rounded);
    }

    #[test]
    fn cross_pattern_emits_axis_aligned_segments() {
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let ops = grid_tile_ops(
            NodeGraphBackgroundPattern::Cross,
            fret_core::Point::new(Px(0.0), Px(0.0)),
            40.0,
            20.0,
            4,
            white,
            white,
            Px(1.0),
            1.0,
            6.0,
        );

        assert!(!ops.is_empty());
        assert!(ops.iter().all(|op| matches!(
            op,
            fret_core::SceneOp::Quad {
                order: DrawOrder(1),
                border: Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: Px(0.0),
                    left: Px(0.0)
                },
                ..
            }
        )));
    }
}
