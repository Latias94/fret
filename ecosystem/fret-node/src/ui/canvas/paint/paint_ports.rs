use super::*;

impl CanvasPaintCache {
    pub(crate) fn port_shape_fill_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        shape: PortShapeHint,
        size: Size,
        dir: Option<crate::core::PortDirection>,
        zoom: f32,
        scale_factor: f32,
    ) -> Option<PathId> {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !size.width.0.is_finite()
            || !size.height.0.is_finite()
            || size.width.0 <= 0.0
            || size.height.0 <= 0.0
        {
            return None;
        }

        let q = |value: f32, step: f32| -> i64 {
            if !value.is_finite() {
                return 0;
            }
            (value / step).round() as i64
        };

        let dir_tag = match dir {
            Some(crate::core::PortDirection::In) => 1,
            Some(crate::core::PortDirection::Out) => 2,
            None => 0,
        };
        let shape_tag = match shape {
            PortShapeHint::Circle => 0,
            PortShapeHint::Diamond => 1,
            PortShapeHint::Triangle => 2,
        };

        let key = PortShapePathKey {
            shape: shape_tag,
            dir: dir_tag,
            w: q(size.width.0, 0.01),
            h: q(size.height.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            stroke_width_screen: 0,
        };

        let cache_key = stable_path_key(20, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return Some(id);
        }

        let commands = build_port_shape_commands(shape, size, dir)?;
        let (id, _metrics) = self.paths.prepare(
            services,
            cache_key,
            &commands,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        Some(id)
    }

    pub(crate) fn port_shape_stroke_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        shape: PortShapeHint,
        size: Size,
        dir: Option<crate::core::PortDirection>,
        zoom: f32,
        scale_factor: f32,
        stroke_width_screen_px: f32,
    ) -> Option<PathId> {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !size.width.0.is_finite()
            || !size.height.0.is_finite()
            || size.width.0 <= 0.0
            || size.height.0 <= 0.0
        {
            return None;
        }

        let stroke_width_screen_px =
            if stroke_width_screen_px.is_finite() && stroke_width_screen_px > 0.0 {
                stroke_width_screen_px
            } else {
                return None;
            };

        let q = |value: f32, step: f32| -> i64 {
            if !value.is_finite() {
                return 0;
            }
            (value / step).round() as i64
        };

        let dir_tag = match dir {
            Some(crate::core::PortDirection::In) => 1,
            Some(crate::core::PortDirection::Out) => 2,
            None => 0,
        };
        let shape_tag = match shape {
            PortShapeHint::Circle => 0,
            PortShapeHint::Diamond => 1,
            PortShapeHint::Triangle => 2,
        };

        let key = PortShapePathKey {
            shape: shape_tag,
            dir: dir_tag,
            w: q(size.width.0, 0.01),
            h: q(size.height.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            stroke_width_screen: q(stroke_width_screen_px, 0.001),
        };

        let cache_key = stable_path_key(21, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return Some(id);
        }

        let commands = build_port_shape_commands(shape, size, dir)?;
        let (id, _metrics) = self.paths.prepare(
            services,
            cache_key,
            &commands,
            PathStyle::StrokeV2(StrokeStyleV2 {
                width: Px(stroke_width_screen_px / zoom),
                join: StrokeJoinV1::Miter,
                cap: StrokeCapV1::Butt,
                miter_limit: 4.0,
                dash: None,
            }),
            constraints,
        );
        Some(id)
    }
}

fn build_port_shape_commands(
    shape: PortShapeHint,
    size: Size,
    dir: Option<crate::core::PortDirection>,
) -> Option<Vec<PathCommand>> {
    let width = size.width.0;
    let height = size.height.0;
    let mid_x = 0.5 * width;
    let mid_y = 0.5 * height;

    match shape {
        PortShapeHint::Circle => None,
        PortShapeHint::Diamond => Some(vec![
            PathCommand::MoveTo(Point::new(Px(mid_x), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(width), Px(mid_y))),
            PathCommand::LineTo(Point::new(Px(mid_x), Px(height))),
            PathCommand::LineTo(Point::new(Px(0.0), Px(mid_y))),
            PathCommand::Close,
        ]),
        PortShapeHint::Triangle => {
            let tip_left = matches!(dir, Some(crate::core::PortDirection::In));
            if tip_left {
                Some(vec![
                    PathCommand::MoveTo(Point::new(Px(0.0), Px(mid_y))),
                    PathCommand::LineTo(Point::new(Px(width), Px(0.0))),
                    PathCommand::LineTo(Point::new(Px(width), Px(height))),
                    PathCommand::Close,
                ])
            } else {
                Some(vec![
                    PathCommand::MoveTo(Point::new(Px(width), Px(mid_y))),
                    PathCommand::LineTo(Point::new(Px(0.0), Px(0.0))),
                    PathCommand::LineTo(Point::new(Px(0.0), Px(height))),
                    PathCommand::Close,
                ])
            }
        }
    }
}
