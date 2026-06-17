use super::*;

impl<H: UiHost> UiTree<H> {
    pub(crate) fn paint_observed_deps_presence_contains(
        &self,
        element: GlobalElementId,
    ) -> Option<bool> {
        self.scratch_paint_observed_deps_presence_active
            .then(|| self.scratch_paint_observed_deps_presence.contains(&element))
    }

    pub(crate) fn request_redraw_coalesced(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            return;
        };
        let tick = app.tick_id();
        if self.last_redraw_request_tick == Some(tick) {
            return;
        }
        self.last_redraw_request_tick = Some(tick);
        app.request_redraw(window);
    }

    /// Resolve the live attached node currently associated with `element` in this tree.
    ///
    /// This is the authoritative element-to-node query when callers already have access to the
    /// current `UiTree` and need a node that is still attached to an active layer tree.
    pub fn live_attached_node_for_element(
        &self,
        app: &mut H,
        element: GlobalElementId,
    ) -> Option<NodeId> {
        self.resolve_live_attached_node_for_element(app, self.window, element)
    }

    pub(crate) fn node_bounds(&self, node: NodeId) -> Option<Rect> {
        self.nodes.get(node).map(|n| n.bounds)
    }

    pub(crate) fn node_needs_layout(&self, node: NodeId) -> bool {
        self.nodes.get(node).is_some_and(|n| n.invalidation.layout)
    }

    pub(crate) fn node_exists(&self, node: NodeId) -> bool {
        self.nodes.contains_key(node)
    }

    pub(crate) fn set_node_element(&mut self, node: NodeId, element: Option<GlobalElementId>) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.element = element;
        }
    }

    pub(crate) fn set_node_widget_prepaint_enabled(&mut self, node: NodeId, enabled: bool) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.widget_prepaint_enabled = enabled;
        }
        self.sync_view_boundary_state_for_node(node);
    }

    pub(crate) fn set_node_inherited_text_style_fingerprint(
        &mut self,
        node: NodeId,
        fingerprint: Option<u64>,
    ) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.inherited_text_style_fingerprint = fingerprint;
        }
    }

    pub(crate) fn set_node_paint_passthrough(
        &mut self,
        node: NodeId,
        passthrough: Option<NodePaintPassthrough>,
    ) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.paint_passthrough = passthrough;
        }
    }

    pub(crate) fn node_element(&self, node: NodeId) -> Option<GlobalElementId> {
        self.nodes.get(node).and_then(|n| n.element)
    }

    pub(crate) fn node_layout_invalidated(&self, node: NodeId) -> bool {
        self.nodes
            .get(node)
            .map(|n| n.invalidation.layout)
            .unwrap_or(false)
    }

    #[cfg(test)]
    pub(crate) fn node_paint_invalidated(&self, node: NodeId) -> bool {
        self.nodes
            .get(node)
            .map(|n| n.invalidation.paint)
            .unwrap_or(false)
    }

    pub(crate) fn node_measured_size(&self, node: NodeId) -> Option<Size> {
        self.nodes.get(node).map(|n| n.measured_size)
    }

    pub(crate) fn node_text_wrap_none_measure_cache(&self, node: NodeId) -> Option<(u64, Size)> {
        self.nodes.get(node).and_then(|n| {
            n.text_wrap_none_measure_cache
                .map(|cache| (cache.fingerprint, cache.size))
        })
    }

    pub(crate) fn node_text_wrapped_measure_cache_for(
        &self,
        node: NodeId,
        fingerprint: u64,
        constraints_max_width: Option<Px>,
    ) -> Option<(Size, Size)> {
        self.nodes
            .get(node)
            .and_then(|n| n.text_wrapped_measure_cache.as_ref())
            .and_then(|cache| cache.get(fingerprint, constraints_max_width))
            .map(|cache| (cache.measured_size, cache.clamped_size))
    }

    pub(crate) fn node_latest_text_wrapped_measure_cache(
        &self,
        node: NodeId,
        fingerprint: u64,
    ) -> Option<(Option<Px>, Size, Size)> {
        self.nodes
            .get(node)
            .and_then(|n| n.text_wrapped_measure_cache.as_ref())
            .and_then(|cache| cache.latest_for_fingerprint(fingerprint))
            .map(|cache| {
                (
                    cache.constraints_max_width,
                    cache.measured_size,
                    cache.clamped_size,
                )
            })
    }
}
