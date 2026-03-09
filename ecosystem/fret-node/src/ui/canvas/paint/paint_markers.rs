use super::*;

impl CanvasPaintCache {
    pub(crate) fn edge_end_marker_path_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_budgeted(
            services,
            MarkerSide::End,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    pub(crate) fn edge_end_marker_path_budgeted_with_tangent(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        endpoint: Point,
        tangent: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_with_tangent_budgeted(
            services,
            MarkerSide::End,
            endpoint,
            tangent,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    pub(crate) fn edge_start_marker_path_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_budgeted(
            services,
            MarkerSide::Start,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    pub(crate) fn edge_start_marker_path_budgeted_with_tangent(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        endpoint: Point,
        tangent: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_with_tangent_budgeted(
            services,
            MarkerSide::Start,
            endpoint,
            tangent,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    fn marker_path_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        side: MarkerSide,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
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
            return (None, false);
        }

        let q = |value: f32, step: f32| -> i64 {
            if !value.is_finite() {
                return 0;
            }
            (value / step).round() as i64
        };

        let kind = match marker.kind {
            EdgeMarkerKind::Arrow => 1,
        };

        let key = MarkerPathKey {
            side,
            kind,
            route,
            from_x: q(from.x.0, 0.01),
            from_y: q(from.y.0, 0.01),
            to_x: q(to.x.0, 0.01),
            to_y: q(to.y.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            size_screen: q(marker.size.max(1.0), 0.01),
            pin_radius_screen: q(pin_radius_screen.max(0.0), 0.01),
        };

        let cache_key = stable_path_key(2, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return (Some(id), false);
        }

        if !budget.try_consume(1) {
            return (None, true);
        }

        let zoom = zoom.max(1.0e-6);
        let tangent = match side {
            MarkerSide::Start => edge_route_start_tangent(route, from, to, zoom),
            MarkerSide::End => edge_route_end_tangent(route, from, to, zoom),
        };

        let len = (tangent.x.0 * tangent.x.0 + tangent.y.0 * tangent.y.0).sqrt();
        if !len.is_finite() || len <= 1.0e-6 {
            return (None, false);
        }
        let ux = tangent.x.0 / len;
        let uy = tangent.y.0 / len;
        let nx = -uy;
        let ny = ux;

        let size_screen = marker.size.max(1.0);
        let size = size_screen / zoom;

        let pin_radius = pin_radius_screen.max(0.0) / zoom;
        let tip = match side {
            MarkerSide::Start => Point::new(
                Px(from.x.0 + ux * pin_radius),
                Px(from.y.0 + uy * pin_radius),
            ),
            MarkerSide::End => {
                Point::new(Px(to.x.0 - ux * pin_radius), Px(to.y.0 - uy * pin_radius))
            }
        };

        match marker.kind {
            EdgeMarkerKind::Arrow => {
                let arrow_len = size;
                let half_w = (0.65 * size).max(0.5 / zoom);
                let base = match side {
                    MarkerSide::Start => {
                        Point::new(Px(tip.x.0 + ux * arrow_len), Px(tip.y.0 + uy * arrow_len))
                    }
                    MarkerSide::End => {
                        Point::new(Px(tip.x.0 - ux * arrow_len), Px(tip.y.0 - uy * arrow_len))
                    }
                };
                let p1 = Point::new(Px(base.x.0 + nx * half_w), Px(base.y.0 + ny * half_w));
                let p2 = Point::new(Px(base.x.0 - nx * half_w), Px(base.y.0 - ny * half_w));

                let commands = [
                    PathCommand::MoveTo(tip),
                    PathCommand::LineTo(p1),
                    PathCommand::LineTo(p2),
                    PathCommand::Close,
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    constraints,
                );
                (Some(id), false)
            }
        }
    }

    fn marker_path_with_tangent_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        side: MarkerSide,
        endpoint: Point,
        tangent: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !endpoint.x.0.is_finite() || !endpoint.y.0.is_finite() {
            return (None, false);
        }

        let q = |value: f32, step: f32| -> i64 {
            if !value.is_finite() {
                return 0;
            }
            (value / step).round() as i64
        };

        let kind = match marker.kind {
            EdgeMarkerKind::Arrow => 1,
        };

        let len = (tangent.x.0 * tangent.x.0 + tangent.y.0 * tangent.y.0).sqrt();
        let (ux, uy) = if len.is_finite() && len > 1.0e-6 {
            (tangent.x.0 / len, tangent.y.0 / len)
        } else {
            (1.0, 0.0)
        };

        let key = MarkerTangentPathKey {
            side,
            kind,
            endpoint_x: q(endpoint.x.0, 0.01),
            endpoint_y: q(endpoint.y.0, 0.01),
            dir_x: q(ux, 0.0001),
            dir_y: q(uy, 0.0001),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            size_screen: q(marker.size.max(1.0), 0.01),
            pin_radius_screen: q(pin_radius_screen.max(0.0), 0.01),
        };

        let cache_key = stable_path_key(4, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return (Some(id), false);
        }

        if !budget.try_consume(1) {
            return (None, true);
        }

        let nx = -uy;
        let ny = ux;

        let zoom = zoom.max(1.0e-6);
        let size_screen = marker.size.max(1.0);
        let size = size_screen / zoom;

        let pin_radius = pin_radius_screen.max(0.0) / zoom;
        let tip = match side {
            MarkerSide::Start => Point::new(
                Px(endpoint.x.0 + ux * pin_radius),
                Px(endpoint.y.0 + uy * pin_radius),
            ),
            MarkerSide::End => Point::new(
                Px(endpoint.x.0 - ux * pin_radius),
                Px(endpoint.y.0 - uy * pin_radius),
            ),
        };

        match marker.kind {
            EdgeMarkerKind::Arrow => {
                let arrow_len = size;
                let half_w = (0.65 * size).max(0.5 / zoom);
                let base = match side {
                    MarkerSide::Start => {
                        Point::new(Px(tip.x.0 + ux * arrow_len), Px(tip.y.0 + uy * arrow_len))
                    }
                    MarkerSide::End => {
                        Point::new(Px(tip.x.0 - ux * arrow_len), Px(tip.y.0 - uy * arrow_len))
                    }
                };
                let p1 = Point::new(Px(base.x.0 + nx * half_w), Px(base.y.0 + ny * half_w));
                let p2 = Point::new(Px(base.x.0 - nx * half_w), Px(base.y.0 - ny * half_w));

                let commands = [
                    PathCommand::MoveTo(tip),
                    PathCommand::LineTo(p1),
                    PathCommand::LineTo(p2),
                    PathCommand::Close,
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    constraints,
                );
                (Some(id), false)
            }
        }
    }
}
