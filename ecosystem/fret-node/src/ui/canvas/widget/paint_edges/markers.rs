use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn push_edge_wire_and_markers_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        zoom: f32,
        scale_factor: f32,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        color: Color,
        width: f32,
        start_marker: Option<&crate::ui::presenter::EdgeMarker>,
        end_marker: Option<&crate::ui::presenter::EdgeMarker>,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
        stop_on_marker_skip: bool,
    ) -> (bool, u32) {
        if !wire_budget.try_consume(1) {
            return (true, 0);
        }

        let mut marker_skipped: u32 = 0;

        let end_path = if stop_on_marker_skip {
            if let Some(marker) = end_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_end_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    return (true, 1);
                }
                path
            } else {
                None
            }
        } else {
            None
        };

        let start_path = if stop_on_marker_skip {
            if let Some(marker) = start_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_start_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    return (true, 1);
                }
                path
            } else {
                None
            }
        } else {
            None
        };

        if let Some(path) =
            self.paint_cache
                .wire_path(services, route, from, to, zoom, scale_factor, width)
        {
            scene.push(SceneOp::Path {
                order: DrawOrder(2),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                color,
            });
        }

        if stop_on_marker_skip {
            if let Some(path) = end_path {
                scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
            if let Some(path) = start_path {
                scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
        } else {
            if let Some(marker) = end_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_end_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    marker_skipped = marker_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }

            if let Some(marker) = start_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_start_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    marker_skipped = marker_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }
        }

        (false, marker_skipped)
    }

    pub(super) fn push_edge_custom_wire_and_markers_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        cache_key: u64,
        commands: &[fret_core::PathCommand],
        start_tangent: Point,
        end_tangent: Point,
        zoom: f32,
        scale_factor: f32,
        from: Point,
        to: Point,
        color: Color,
        width: f32,
        start_marker: Option<&crate::ui::presenter::EdgeMarker>,
        end_marker: Option<&crate::ui::presenter::EdgeMarker>,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
        stop_on_marker_skip: bool,
    ) -> (bool, u32) {
        if !wire_budget.try_consume(1) {
            return (true, 0);
        }

        let mut marker_skipped: u32 = 0;

        let end_path = if stop_on_marker_skip {
            if let Some(marker) = end_marker {
                let (path, skipped_by_budget) =
                    self.paint_cache.edge_end_marker_path_budgeted_with_tangent(
                        services,
                        to,
                        end_tangent,
                        zoom,
                        scale_factor,
                        marker,
                        self.style.pin_radius,
                        marker_budget,
                    );
                if skipped_by_budget {
                    return (true, 1);
                }
                path
            } else {
                None
            }
        } else {
            None
        };

        let start_path = if stop_on_marker_skip {
            if let Some(marker) = start_marker {
                let (path, skipped_by_budget) = self
                    .paint_cache
                    .edge_start_marker_path_budgeted_with_tangent(
                        services,
                        from,
                        start_tangent,
                        zoom,
                        scale_factor,
                        marker,
                        self.style.pin_radius,
                        marker_budget,
                    );
                if skipped_by_budget {
                    return (true, 1);
                }
                path
            } else {
                None
            }
        } else {
            None
        };

        if let Some(path) = self.paint_cache.wire_path_from_commands(
            services,
            cache_key,
            commands,
            zoom,
            scale_factor,
            width,
        ) {
            scene.push(SceneOp::Path {
                order: DrawOrder(2),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                color,
            });
        }

        if stop_on_marker_skip {
            if let Some(path) = end_path {
                scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
            if let Some(path) = start_path {
                scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
        } else {
            if let Some(marker) = end_marker {
                let (path, skipped_by_budget) =
                    self.paint_cache.edge_end_marker_path_budgeted_with_tangent(
                        services,
                        to,
                        end_tangent,
                        zoom,
                        scale_factor,
                        marker,
                        self.style.pin_radius,
                        marker_budget,
                    );
                if skipped_by_budget {
                    marker_skipped = marker_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }

            if let Some(marker) = start_marker {
                let (path, skipped_by_budget) = self
                    .paint_cache
                    .edge_start_marker_path_budgeted_with_tangent(
                        services,
                        from,
                        start_tangent,
                        zoom,
                        scale_factor,
                        marker,
                        self.style.pin_radius,
                        marker_budget,
                    );
                if skipped_by_budget {
                    marker_skipped = marker_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }
        }

        (false, marker_skipped)
    }
}
