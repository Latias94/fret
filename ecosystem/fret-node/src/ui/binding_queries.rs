use super::*;

impl NodeGraphSurfaceBinding {
    /// Reads the current viewport from the authoritative store-backed controller.
    pub fn viewport<H: UiHost>(&self, host: &H) -> (CanvasPoint, f32) {
        self.controller().viewport(host)
    }

    /// Clones the current graph snapshot from the authoritative store.
    pub fn graph_snapshot<H: UiHost>(&self, host: &H) -> Option<Graph> {
        self.controller().graph_snapshot(host)
    }

    /// Clones the current view-state snapshot from the authoritative store.
    pub fn view_state_snapshot<H: UiHost>(&self, host: &H) -> Option<NodeGraphViewState> {
        self.controller().view_state_snapshot(host)
    }

    /// Returns the outgoing neighbor node ids for the given node.
    pub fn outgoers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.controller().outgoers(host, node)
    }

    /// Returns the incoming neighbor node ids for the given node.
    pub fn incomers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.controller().incomers(host, node)
    }

    /// Returns the edge ids incident to the given node.
    pub fn connected_edges<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<EdgeId> {
        self.controller().connected_edges(host, node)
    }

    /// Returns handle-level connections for the given node-side/port query.
    pub fn port_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphPortConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.controller().port_connections(host, query)
    }

    /// Returns node-level connections for the given query.
    pub fn node_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphNodeConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.controller().node_connections(host, query)
    }

    /// Returns whether the bound store currently has undo history.
    pub fn can_undo<H: UiHost>(&self, host: &H) -> bool {
        self.controller().can_undo(host)
    }

    /// Returns whether the bound store currently has redo history.
    pub fn can_redo<H: UiHost>(&self, host: &H) -> bool {
        self.controller().can_redo(host)
    }
}
