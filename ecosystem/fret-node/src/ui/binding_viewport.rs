use super::*;

impl NodeGraphSurfaceBinding {
    /// Applies a viewport change through the bound controller.
    pub fn set_viewport<H: UiHost>(&self, host: &mut H, pan: CanvasPoint, zoom: f32) -> bool {
        let applied = self.controller().set_viewport(host, pan, zoom);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Applies a viewport change with store-first viewport options through the bound controller.
    pub fn set_viewport_with_options<H: UiHost>(
        &self,
        host: &mut H,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller()
            .set_viewport_with_options(host, pan, zoom, options);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Applies a viewport change from an object-safe action hook.
    pub fn set_viewport_action_host(
        &self,
        host: &mut dyn UiActionHost,
        pan: CanvasPoint,
        zoom: f32,
    ) -> bool {
        let applied = self.controller().set_viewport_action_host(host, pan, zoom);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Applies a viewport change with store-first viewport options from an object-safe action hook.
    pub fn set_viewport_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller()
            .set_viewport_with_options_action_host(host, pan, zoom, options);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Centers a canvas point inside explicit bounds through the bound controller.
    pub fn set_center_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        let applied = self.controller().set_center_in_bounds(host, bounds, center);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Centers a canvas point inside explicit bounds from an object-safe action hook.
    pub fn set_center_in_bounds_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        center: CanvasPoint,
    ) -> bool {
        let applied = self
            .controller()
            .set_center_in_bounds_action_host(host, bounds, center);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Centers a canvas point inside explicit bounds with store-first viewport options.
    pub fn set_center_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller()
            .set_center_in_bounds_with_options(host, bounds, center, zoom, options);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Centers a canvas point inside explicit bounds with store-first viewport options from an
    /// object-safe action hook.
    pub fn set_center_in_bounds_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) -> bool {
        let applied = self
            .controller()
            .set_center_in_bounds_with_options_action_host(host, bounds, center, zoom, options);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds.
    pub fn fit_view_nodes_in_bounds<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        let applied = self
            .controller()
            .fit_view_nodes_in_bounds(host, bounds, nodes);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds with store-first fit-view
    /// options.
    pub fn fit_view_nodes_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let applied = self
            .controller()
            .fit_view_nodes_in_bounds_with_options(host, bounds, nodes, options);
        self.sync_view_state_after_viewport_update(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds from an object-safe action
    /// hook.
    pub fn fit_view_nodes_in_bounds_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        nodes: Vec<NodeId>,
    ) -> bool {
        let applied = self
            .controller()
            .fit_view_nodes_in_bounds_action_host(host, bounds, nodes);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    /// Fits the viewport to the given nodes inside explicit bounds with store-first fit-view
    /// options from an object-safe action hook.
    pub fn fit_view_nodes_in_bounds_with_options_action_host(
        &self,
        host: &mut dyn UiActionHost,
        bounds: Rect,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) -> bool {
        let applied = self
            .controller()
            .fit_view_nodes_in_bounds_with_options_action_host(host, bounds, nodes, options);
        self.sync_view_state_after_viewport_update_action_host(host, applied)
    }

    fn sync_view_state_after_viewport_update<H: UiHost>(
        &self,
        host: &mut H,
        applied: bool,
    ) -> bool {
        if applied {
            let _ = self
                .controller()
                .sync_view_state_model_from_store(host, &self.view_state);
        }
        applied
    }

    fn sync_view_state_after_viewport_update_action_host(
        &self,
        host: &mut dyn UiActionHost,
        applied: bool,
    ) -> bool {
        if applied {
            let _ = self
                .controller()
                .sync_view_state_model_from_store_action_host(host, &self.view_state);
        }
        applied
    }
}
