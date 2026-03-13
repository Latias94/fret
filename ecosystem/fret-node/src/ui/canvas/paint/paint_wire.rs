use super::*;

impl CanvasPaintCache {
    pub(crate) fn wire_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        width_px: f32,
        dash: Option<DashPatternV1>,
    ) -> Option<PathId> {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !from.x.0.is_finite()
            || !from.y.0.is_finite()
            || !to.x.0.is_finite()
            || !to.y.0.is_finite()
        {
            return None;
        }

        let stroke_width = width_px / zoom;
        if !stroke_width.is_finite() || stroke_width <= 0.0 {
            return None;
        }

        let dash =
            dash.and_then(|pattern| scale_dash_pattern_screen_px_to_canvas_units(pattern, zoom));

        let q = |value: f32, step: f32| -> i64 {
            if !value.is_finite() {
                return 0;
            }
            (value / step).round() as i64
        };

        let key = WirePathKey {
            route,
            from_x: q(from.x.0, 0.01),
            from_y: q(from.y.0, 0.01),
            to_x: q(to.x.0, 0.01),
            to_y: q(to.y.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            stroke_width: q(stroke_width, 0.0001),
            dash: dash.map(|pattern| q(pattern.dash.0, 0.01)).unwrap_or(0),
            gap: dash.map(|pattern| q(pattern.gap.0, 0.01)).unwrap_or(0),
            phase: dash.map(|pattern| q(pattern.phase.0, 0.01)).unwrap_or(0),
        };

        let cache_key = stable_path_key(1, &key);
        match route {
            EdgeRouteKind::Bezier => {
                let dx = to.x.0 - from.x.0;
                let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
                let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
                let ctrl1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
                let ctrl2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);

                let commands = [
                    PathCommand::MoveTo(from),
                    PathCommand::CubicTo { ctrl1, ctrl2, to },
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::StrokeV2(StrokeStyleV2 {
                        width: Px(width_px / zoom),
                        join: StrokeJoinV1::Round,
                        cap: StrokeCapV1::Round,
                        miter_limit: 4.0,
                        dash,
                    }),
                    PathConstraints {
                        scale_factor: scale_factor * zoom,
                    },
                );
                Some(id)
            }
            EdgeRouteKind::Straight => {
                let commands = [PathCommand::MoveTo(from), PathCommand::LineTo(to)];
                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::StrokeV2(StrokeStyleV2 {
                        width: Px(width_px / zoom),
                        join: StrokeJoinV1::Round,
                        cap: StrokeCapV1::Round,
                        miter_limit: 4.0,
                        dash,
                    }),
                    PathConstraints {
                        scale_factor: scale_factor * zoom,
                    },
                );
                Some(id)
            }
            EdgeRouteKind::Step => {
                let mid_x = 0.5 * (from.x.0 + to.x.0);
                let p1 = Point::new(Px(mid_x), from.y);
                let p2 = Point::new(Px(mid_x), to.y);

                let commands = [
                    PathCommand::MoveTo(from),
                    PathCommand::LineTo(p1),
                    PathCommand::LineTo(p2),
                    PathCommand::LineTo(to),
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::StrokeV2(StrokeStyleV2 {
                        width: Px(width_px / zoom),
                        join: StrokeJoinV1::Round,
                        cap: StrokeCapV1::Round,
                        miter_limit: 4.0,
                        dash,
                    }),
                    PathConstraints {
                        scale_factor: scale_factor * zoom,
                    },
                );
                Some(id)
            }
        }
    }

    pub(crate) fn wire_path_from_commands(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        cache_key: u64,
        commands: &[PathCommand],
        zoom: f32,
        scale_factor: f32,
        width_px: f32,
        dash: Option<DashPatternV1>,
    ) -> Option<PathId> {
        if commands.is_empty() {
            return None;
        }

        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        let stroke_width = width_px / zoom;
        if !stroke_width.is_finite() || stroke_width <= 0.0 {
            return None;
        }

        let dash =
            dash.and_then(|pattern| scale_dash_pattern_screen_px_to_canvas_units(pattern, zoom));

        let cache_key = stable_path_key(3, &cache_key);
        let (id, _metrics) = self.paths.prepare(
            services,
            cache_key,
            commands,
            PathStyle::StrokeV2(StrokeStyleV2 {
                width: Px(width_px / zoom),
                join: StrokeJoinV1::Round,
                cap: StrokeCapV1::Round,
                miter_limit: 4.0,
                dash,
            }),
            PathConstraints {
                scale_factor: scale_factor * zoom,
            },
        );
        Some(id)
    }
}
