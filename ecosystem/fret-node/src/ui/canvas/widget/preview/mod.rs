mod drag;
mod node_resize;

use std::collections::HashMap;

use super::*;

pub(super) fn ports_for_node<'a>(
    node_ports: &'a HashMap<GraphNodeId, Vec<PortId>>,
    node_id: GraphNodeId,
) -> &'a [PortId] {
    node_ports
        .get(&node_id)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

pub(super) fn resolve_edge_endpoints_from_model<H: UiHost>(
    host: &H,
    graph: &fret_runtime::Model<Graph>,
    edge_ids: &[EdgeId],
) -> Vec<(EdgeId, PortId, PortId)> {
    graph
        .read_ref(host, |g| {
            edge_ids
                .iter()
                .filter_map(|edge_id| g.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to)))
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default()
}
