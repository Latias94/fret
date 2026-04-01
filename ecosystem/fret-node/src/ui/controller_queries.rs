use super::*;

impl NodeGraphController {
    pub fn viewport<H: UiHost>(&self, host: &H) -> (CanvasPoint, f32) {
        self.store
            .read_ref(host, |store| {
                let view = store.view_state();
                (view.pan, view.zoom)
            })
            .ok()
            .unwrap_or((CanvasPoint::default(), 1.0))
    }

    pub fn screen_to_canvas<H: UiHost>(
        &self,
        host: &H,
        bounds: Rect,
        screen: Point,
    ) -> Option<CanvasPoint> {
        self.store
            .read_ref(host, |store| {
                let view_state = store.view_state();
                let zoom = PanZoom2D::sanitize_zoom(view_state.zoom, 1.0);
                let view = PanZoom2D {
                    pan: Point::new(Px(view_state.pan.x), Px(view_state.pan.y)),
                    zoom,
                };
                let point = view.screen_to_canvas(bounds, screen);
                CanvasPoint {
                    x: point.x.0,
                    y: point.y.0,
                }
            })
            .ok()
    }

    pub fn canvas_to_screen<H: UiHost>(
        &self,
        host: &H,
        bounds: Rect,
        canvas: CanvasPoint,
    ) -> Option<Point> {
        self.store
            .read_ref(host, |store| {
                let view_state = store.view_state();
                let zoom = PanZoom2D::sanitize_zoom(view_state.zoom, 1.0);
                let view = PanZoom2D {
                    pan: Point::new(Px(view_state.pan.x), Px(view_state.pan.y)),
                    zoom,
                };
                view.canvas_to_screen(bounds, Point::new(Px(canvas.x), Px(canvas.y)))
            })
            .ok()
    }

    pub fn graph_snapshot<H: UiHost>(&self, host: &H) -> Option<Graph> {
        self.store
            .read_ref(host, |store| store.graph().clone())
            .ok()
    }

    pub fn view_state_snapshot<H: UiHost>(&self, host: &H) -> Option<NodeGraphViewState> {
        self.store
            .read_ref(host, |store| store.view_state().clone())
            .ok()
    }

    pub fn can_undo<H: UiHost>(&self, host: &H) -> bool {
        self.store
            .read_ref(host, |store| store.can_undo())
            .ok()
            .unwrap_or(false)
    }

    pub fn can_redo<H: UiHost>(&self, host: &H) -> bool {
        self.store
            .read_ref(host, |store| store.can_redo())
            .ok()
            .unwrap_or(false)
    }

    pub fn outgoers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.store
            .read_ref(host, |store| get_outgoers(store.lookups(), node))
            .ok()
            .unwrap_or_default()
    }

    pub fn incomers<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<NodeId> {
        self.store
            .read_ref(host, |store| get_incomers(store.lookups(), node))
            .ok()
            .unwrap_or_default()
    }

    pub fn connected_edges<H: UiHost>(&self, host: &H, node: NodeId) -> Vec<EdgeId> {
        self.store
            .read_ref(host, |store| get_connected_edges(store.lookups(), node))
            .ok()
            .unwrap_or_default()
    }

    pub fn port_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphPortConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.store
            .read_ref(host, |store| {
                sorted_connections(store.lookups().connections_for_port(
                    query.node_id,
                    query.side,
                    query.port_id,
                ))
            })
            .ok()
            .unwrap_or_default()
    }

    pub fn node_connections<H: UiHost>(
        &self,
        host: &H,
        query: NodeGraphNodeConnectionsQuery,
    ) -> Vec<HandleConnection> {
        self.store
            .read_ref(host, |store| {
                let lookups = store.lookups();
                let connections = match (query.side, query.port_id) {
                    (Some(side), Some(port_id)) => {
                        lookups.connections_for_port(query.node_id, side, port_id)
                    }
                    (Some(side), None) => lookups.connections_for_node_side(query.node_id, side),
                    (None, Some(port_id)) => store
                        .graph()
                        .ports
                        .get(&port_id)
                        .filter(|port| port.node == query.node_id)
                        .and_then(|port| {
                            lookups.connections_for_port(
                                query.node_id,
                                ConnectionSide::from_port_dir(port.dir),
                                port_id,
                            )
                        }),
                    (None, None) => lookups.connections_for_node(query.node_id),
                };
                sorted_connections(connections)
            })
            .ok()
            .unwrap_or_default()
    }
}

fn sorted_connections(
    connections: Option<&std::collections::HashMap<EdgeId, HandleConnection>>,
) -> Vec<HandleConnection> {
    let Some(connections) = connections else {
        return Vec::new();
    };
    let mut out: Vec<_> = connections.values().copied().collect();
    out.sort_by_key(|connection| connection.edge);
    out
}
