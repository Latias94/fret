use super::*;

impl<H: UiHost> UiTree<H> {
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

    pub(crate) fn node_element(&self, node: NodeId) -> Option<GlobalElementId> {
        self.nodes.get(node).and_then(|n| n.element)
    }

    pub(crate) fn node_layout_invalidated(&self, node: NodeId) -> bool {
        self.nodes
            .get(node)
            .map(|n| n.invalidation.layout)
            .unwrap_or(false)
    }

    pub(crate) fn node_measured_size(&self, node: NodeId) -> Option<Size> {
        self.nodes.get(node).map(|n| n.measured_size)
    }
}
