use super::*;

#[path = "widget_surface/builders.rs"]
mod builders;
#[path = "widget_surface/construct.rs"]
mod construct;
#[path = "widget_surface/fit_view.rs"]
mod fit_view;
#[path = "widget_surface/sync.rs"]
mod sync;

impl NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        Self::new_with_middleware(graph, view_state, NoopNodeGraphCanvasMiddleware)
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) const REROUTE_INPUTS: usize = 1;
    pub(super) const REROUTE_OUTPUTS: usize = 1;
    pub(super) const AUTO_PAN_TICK_HZ: f32 = 60.0;
    pub(super) const AUTO_PAN_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::AUTO_PAN_TICK_HZ) as u64);
    pub(super) const PAN_INERTIA_TICK_HZ: f32 = 60.0;
    pub(super) const PAN_INERTIA_TICK_INTERVAL: Duration =
        Duration::from_nanos((1.0e9 / Self::PAN_INERTIA_TICK_HZ) as u64);
    pub(super) const VIEWPORT_MOVE_END_DEBOUNCE: Duration = Duration::from_millis(180);
    pub(super) const EDGE_FOCUS_ANCHOR_SIZE_SCREEN: f32 = 16.0;
    pub(super) const EDGE_FOCUS_ANCHOR_PAD_SCREEN: f32 = 1.0;
    pub(super) const EDGE_FOCUS_ANCHOR_BORDER_SCREEN: f32 = 2.0;
    pub(super) const EDGE_FOCUS_ANCHOR_OFFSET_SCREEN: f32 = 18.0;
    pub(super) const GRID_TILE_SIZE_SCREEN_PX: f32 = 2048.0;
    pub(super) const GRID_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(32, 8);
    pub(super) const EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(4, 1);
    pub(super) const EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME: InteractionBudget =
        InteractionBudget::new(2, 1);
    pub(super) const EDGE_WIRE_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(super) const EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(super) const EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(256, 64);
    pub(super) const EDGE_MARKER_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(96, 24);
    pub(super) const EDGE_LABEL_BUILD_BUDGET_PER_FRAME: InteractionBudget =
        InteractionBudget::new(16, 4);
    pub(super) const STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES: u64 = 60 * 30;
    pub(super) const STATIC_SCENE_TILE_CACHE_MAX_ENTRIES: usize = 16;

    pub(super) fn compute_render_cull_rect(
        &self,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) -> Option<Rect> {
        if !snapshot.interaction.only_render_visible_elements {
            return None;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 1.0e-6 {
            return None;
        }

        let viewport = Self::viewport_from_pan_zoom(bounds, snapshot.pan, zoom);
        let viewport_rect = viewport.visible_canvas_rect();
        let viewport_w = viewport_rect.size.width.0;
        let viewport_h = viewport_rect.size.height.0;
        let margin_screen = self.style.paint.render_cull_margin_px;

        if !margin_screen.is_finite()
            || margin_screen <= 0.0
            || !viewport_w.is_finite()
            || !viewport_h.is_finite()
            || viewport_w <= 0.0
            || viewport_h <= 0.0
        {
            return None;
        }

        let margin = margin_screen / zoom;
        Some(inflate_rect(viewport_rect, margin))
    }

    #[cfg(test)]
    pub(super) fn debug_derived_build_counters(&self) -> super::super::state::DerivedBuildCounters {
        self.geometry.counters
    }

    #[cfg(test)]
    pub(super) fn debug_render_metrics_for_bounds<H: UiHost>(
        &mut self,
        host: &mut H,
        bounds: Rect,
    ) -> paint_render_data::RenderMetrics {
        let snapshot = self.sync_view_state(host);
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 1.0e-6 {
            return paint_render_data::RenderMetrics::default();
        }

        let render_cull_rect = self.compute_render_cull_rect(&snapshot, bounds);
        let (geom, index) = self.canvas_derived(host, &snapshot);
        self.collect_render_data(
            host,
            &snapshot,
            geom,
            index,
            render_cull_rect,
            zoom,
            None,
            true,
            true,
            true,
        )
        .metrics
    }

    pub(super) fn view_interacting(&self) -> bool {
        self.interaction.viewport_move_debounce.is_some()
            || self.interaction.panning
            || self.interaction.pan_inertia.is_some()
            || self.interaction.viewport_animation.is_some()
            || self.interaction.pending_marquee.is_some()
            || self.interaction.marquee.is_some()
            || self.interaction.pending_node_drag.is_some()
            || self.interaction.node_drag.is_some()
            || self.interaction.pending_group_drag.is_some()
            || self.interaction.group_drag.is_some()
            || self.interaction.pending_group_resize.is_some()
            || self.interaction.group_resize.is_some()
            || self.interaction.pending_node_resize.is_some()
            || self.interaction.node_resize.is_some()
            || self.interaction.pending_wire_drag.is_some()
            || self.interaction.wire_drag.is_some()
            || self.interaction.suspended_wire_drag.is_some()
            || self.interaction.pending_edge_insert_drag.is_some()
            || self.interaction.edge_insert_drag.is_some()
            || self.interaction.edge_drag.is_some()
            || self.interaction.pending_insert_node_drag.is_some()
            || self.interaction.insert_node_drag_preview.is_some()
            || self.interaction.context_menu.is_some()
            || self.interaction.searcher.is_some()
    }

    pub(super) fn edge_render_hint(&self, graph: &Graph, edge_id: EdgeId) -> EdgeRenderHint {
        EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_render_hint(graph, edge_id)
    }

    pub(super) fn edge_custom_path(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> Option<crate::ui::edge_types::EdgeCustomPath> {
        EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_custom_path(graph, edge_id, hint, from, to, zoom)
    }

    pub fn with_presenter(mut self, presenter: impl NodeGraphPresenter + 'static) -> Self {
        self.presenter = Box::new(FallbackMeasuredNodeGraphPresenter::new(
            presenter,
            self.auto_measured.clone(),
        ));
        self
    }

    pub fn with_skin(mut self, skin: NodeGraphSkinRef) -> Self {
        self.skin = Some(skin);
        self.skin_last_rev = None;
        self
    }

    pub fn with_paint_overrides(mut self, overrides: NodeGraphPaintOverridesRef) -> Self {
        self.paint_overrides = Some(overrides);
        self.paint_overrides_last_rev = None;
        self
    }

    pub fn with_edge_types(mut self, edge_types: NodeGraphEdgeTypes) -> Self {
        self.edge_types = Some(edge_types);
        self
    }

    pub fn with_callbacks(mut self, callbacks: impl NodeGraphCallbacks) -> Self {
        self.callbacks = Some(Box::new(callbacks));
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

    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = Some(command);
        self
    }
}
