use super::*;

impl NodeGraphController {
    pub fn set_viewport<H: UiHost>(&self, host: &mut H, pan: CanvasPoint, zoom: f32) -> bool {
        self.set_viewport_with_options(host, pan, zoom, NodeGraphSetViewportOptions::default())
    }

    pub fn set_viewport_action_host(
        &self,
        host: &mut dyn UiActionHost,
        pan: CanvasPoint,
        zoom: f32,
    ) -> bool {
        self.set_viewport_with_options_action_host(
            host,
            pan,
            zoom,
            NodeGraphSetViewportOptions::default(),
        )
    }

    pub fn set_viewport_with_options<H: UiHost>(
        &self,
        host: &mut H,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        self.set_viewport_in_models(host.models_mut(), pan, zoom, options)
    }

    pub fn set_viewport_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        self.set_viewport_in_models(host.models_mut(), pan, zoom, options)
    }

    pub fn set_center_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        self.set_center_in_bounds_with_options(
            host,
            bounds,
            center,
            None,
            NodeGraphSetViewportOptions::default(),
        )
    }

    pub fn set_center_in_bounds_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        self.set_center_in_bounds_with_options_action_host(
            host,
            bounds,
            center,
            None,
            NodeGraphSetViewportOptions::default(),
        )
    }

    pub fn set_center_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        self.set_center_in_bounds_in_models(host.models_mut(), bounds, center, zoom, options)
    }

    pub fn set_center_in_bounds_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        self.set_center_in_bounds_in_models(host.models_mut(), bounds, center, zoom, options)
    }

    pub fn fit_view_nodes_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        self.fit_view_nodes_in_bounds_with_options(
            host,
            bounds,
            nodes,
            NodeGraphFitViewOptions::default(),
        )
    }

    pub fn fit_view_nodes_in_bounds_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        self.fit_view_nodes_in_bounds_with_options_action_host(
            host,
            bounds,
            nodes,
            NodeGraphFitViewOptions::default(),
        )
    }

    pub fn fit_view_nodes_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        self.fit_view_nodes_in_bounds_in_models(host.models_mut(), bounds, nodes, options)
    }

    pub fn fit_view_nodes_in_bounds_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        self.fit_view_nodes_in_bounds_in_models(host.models_mut(), bounds, nodes, options)
    }

    fn set_viewport_in_models(
        &self,
        models: &mut ModelStore,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let (current_pan, current_zoom) = self.viewport_in_models(models);
        let pan = normalize_requested_pan(current_pan, pan);
        let zoom = normalize_requested_zoom(current_zoom, zoom, options.min_zoom, options.max_zoom);

        models
            .update(&self.store, |store| {
                store.set_viewport(pan, zoom);
            })
            .is_ok()
    }

    fn set_center_in_bounds_in_models(
        &self,
        models: &mut ModelStore,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let (_, current_zoom) = self.viewport_in_models(models);
        let zoom = normalize_requested_zoom(
            current_zoom,
            zoom.unwrap_or(current_zoom),
            options.min_zoom,
            options.max_zoom,
        );
        let pan = pan_for_center(bounds, center, zoom);
        self.set_viewport_in_models(models, pan, zoom, options)
    }

    fn fit_view_nodes_in_bounds_in_models(
        &self,
        models: &mut ModelStore,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let target = models
            .read(&self.store, |store| {
                let interaction = &store.view_state().interaction;
                let node_origin = interaction.node_origin.normalized();
                let padding = normalize_fit_view_padding(
                    options.padding.unwrap_or(interaction.frame_view_padding),
                );
                let (min_zoom, max_zoom) =
                    normalize_fit_view_zoom_bounds(options.min_zoom, options.max_zoom);

                let infos: Vec<FitViewNodeInfo> = nodes
                    .iter()
                    .filter_map(|id| {
                        let entry = store.lookups().node_lookup.get(id)?;
                        if entry.hidden && !options.include_hidden_nodes {
                            return None;
                        }
                        let size = entry.size?;
                        if !size.width.is_finite()
                            || !size.height.is_finite()
                            || size.width <= 0.0
                            || size.height <= 0.0
                        {
                            return None;
                        }
                        Some(FitViewNodeInfo {
                            pos: entry.pos,
                            size_px: (size.width, size.height),
                        })
                    })
                    .collect();

                compute_fit_view_target(
                    &infos,
                    FitViewComputeOptions {
                        viewport_width_px: bounds.size.width.0,
                        viewport_height_px: bounds.size.height.0,
                        node_origin: (node_origin.x, node_origin.y),
                        padding,
                        margin_px_fallback: CONTROLLER_FIT_VIEW_MARGIN_PX_FALLBACK,
                        min_zoom,
                        max_zoom,
                    },
                )
            })
            .ok()
            .flatten();

        let Some((pan, zoom)) = target else {
            return false;
        };

        self.set_viewport_in_models(
            models,
            pan,
            zoom,
            NodeGraphSetViewportOptions {
                min_zoom: options.min_zoom,
                max_zoom: options.max_zoom,
            },
        )
    }

    fn viewport_in_models(&self, models: &ModelStore) -> (CanvasPoint, f32) {
        models
            .read(&self.store, |store| {
                let view = store.view_state();
                (view.pan, view.zoom)
            })
            .ok()
            .unwrap_or((CanvasPoint::default(), 1.0))
    }
}

fn normalize_requested_pan(current_pan: CanvasPoint, requested_pan: CanvasPoint) -> CanvasPoint {
    if requested_pan.x.is_finite() && requested_pan.y.is_finite() {
        requested_pan
    } else {
        current_pan
    }
}

fn normalize_requested_zoom(
    current_zoom: f32,
    requested_zoom: f32,
    min_zoom: Option<f32>,
    max_zoom: Option<f32>,
) -> f32 {
    let mut min_zoom = min_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(f32::MIN_POSITIVE);
    let mut max_zoom = max_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(f32::MAX);
    if min_zoom > max_zoom {
        std::mem::swap(&mut min_zoom, &mut max_zoom);
    }

    let base = if requested_zoom.is_finite() && requested_zoom > 0.0 {
        requested_zoom
    } else if current_zoom.is_finite() && current_zoom > 0.0 {
        current_zoom
    } else {
        1.0
    };

    base.clamp(min_zoom, max_zoom)
}

fn normalize_fit_view_padding(padding: f32) -> f32 {
    if padding.is_finite() {
        padding.clamp(0.0, 0.45)
    } else {
        0.0
    }
}

fn normalize_fit_view_zoom_bounds(min_zoom: Option<f32>, max_zoom: Option<f32>) -> (f32, f32) {
    let mut min_zoom = min_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(CONTROLLER_FIT_VIEW_MIN_ZOOM);
    let mut max_zoom = max_zoom
        .filter(|zoom| zoom.is_finite() && *zoom > 0.0)
        .unwrap_or(CONTROLLER_FIT_VIEW_MAX_ZOOM);
    if min_zoom > max_zoom {
        std::mem::swap(&mut min_zoom, &mut max_zoom);
    }
    (min_zoom, max_zoom)
}
