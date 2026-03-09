use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn reset_style_state(mut self) -> Self {
        self.background_override = None;
        self.color_mode = None;
        self.color_mode_last = None;
        self.color_mode_theme_rev = None;
        self.reset_geometry_cache_keys();
        self
    }

    pub fn with_geometry_overrides(mut self, overrides: NodeGraphGeometryOverridesRef) -> Self {
        self.geometry_overrides = Some(overrides);
        self.reset_geometry_cache_keys();
        self
    }

    pub fn with_style(mut self, style: NodeGraphStyle) -> Self {
        self.style = style;
        self.reset_style_state()
    }

    pub fn background_style(&self) -> NodeGraphBackgroundStyle {
        self.style.background_style()
    }

    pub fn with_background_style(mut self, background: NodeGraphBackgroundStyle) -> Self {
        self.style = self.style.with_background_style(background);
        self.background_override = Some(background);
        self.grid_scene_cache.clear();
        self
    }

    pub fn with_color_mode(mut self, mode: NodeGraphColorMode) -> Self {
        self.color_mode = Some(mode);
        self.color_mode_last = None;
        self.color_mode_theme_rev = None;
        self.reset_geometry_cache_keys();
        self
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn with_view_queue(mut self, queue: Model<NodeGraphViewQueue>) -> Self {
        self.view_queue = Some(queue);
        self.view_queue_key = None;
        self
    }

    pub fn with_controller(mut self, controller: NodeGraphController) -> Self {
        self.store = Some(controller.store());
        self.store_rev = None;
        self.edit_queue = controller.transport_edit_queue();
        self.edit_queue_key = None;
        self.view_queue = controller.transport_view_queue();
        self.view_queue_key = None;
        self
    }

    pub fn with_overlay_state(mut self, overlays: Model<NodeGraphOverlayState>) -> Self {
        self.overlays = Some(overlays);
        self
    }

    pub fn with_measured_output_store(mut self, store: Arc<MeasuredGeometryStore>) -> Self {
        self.measured_output = Some(store);
        self.measured_output_key = None;
        self
    }

    pub fn with_internals_store(mut self, store: Arc<NodeGraphInternalsStore>) -> Self {
        self.internals = Some(store);
        self
    }

    pub fn with_diagnostics_anchor_ports(
        mut self,
        child_offset: usize,
        ports: Vec<PortId>,
    ) -> Self {
        self.diagnostics_anchor_ports = if ports.is_empty() {
            None
        } else {
            Some(DiagnosticsAnchorPorts {
                child_offset,
                ports,
            })
        };
        self
    }

    pub fn with_store(mut self, store: Model<NodeGraphStore>) -> Self {
        self.store = Some(store);
        self.store_rev = None;
        self
    }
}
